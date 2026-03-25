use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use rayon::prelude::*;

use crate::error::{MmotError, Result};
use crate::evaluator::interpolate::{evaluate_f64, evaluate_vec2};
use crate::parser::parse;
use crate::props;
use crate::renderer::shape::ResolvedShape;
use crate::renderer::{
    render as render_frame, FrameScene, ResolvedContent, ResolvedLayer, ResolvedTransform,
};
use crate::schema::{Layer, LayerContent, Scene, ShapeSpec, TransitionSpec};

/// Options for the render pipeline.
pub struct RenderOptions {
    pub output_path: PathBuf,
    pub format: OutputFormat,
    pub quality: u8,
    pub frame_range: Option<(u64, u64)>,
    pub concurrency: Option<usize>,
    pub backend: RenderBackend,
    pub include_audio: bool,
}

/// Output format.
#[derive(Debug, Clone)]
pub enum OutputFormat {
    Mp4,
    Gif,
    Webm,
}

/// Render backend.
#[derive(Debug, Clone)]
pub enum RenderBackend {
    Cpu,
    Gpu,
}

/// Progress callback: called with (current_frame, total_frames).
pub type ProgressFn = Arc<dyn Fn(u64, u64) + Send + Sync>;

/// Main entry point: parse JSON, render all frames, encode to MP4.
pub fn render_scene(
    json: &str,
    opts: RenderOptions,
    progress: Option<ProgressFn>,
) -> Result<()> {
    render_scene_with_props(json, &HashMap::new(), opts, progress)
}

/// Main entry point with props substitution.
pub fn render_scene_with_props(
    json: &str,
    cli_props: &HashMap<String, String>,
    opts: RenderOptions,
    progress: Option<ProgressFn>,
) -> Result<()> {
    // Substitute props in JSON before parsing
    let json = props::substitute(json, cli_props);
    let scene = parse(&json)?;

    let total = match opts.frame_range {
        Some((s, e)) => e - s,
        None => scene.meta.duration,
    };
    let start = opts.frame_range.map(|(s, _)| s).unwrap_or(0);

    // Set rayon thread count if specified
    if let Some(n) = opts.concurrency {
        rayon::ThreadPoolBuilder::new()
            .num_threads(n)
            .build_global()
            .ok();
    }

    // Pre-load all font assets ONCE before the parallel render loop.
    // This avoids N concurrent disk reads inside evaluate_composition.
    let font_cache: HashMap<String, Vec<u8>> = {
        let mut cache = HashMap::new();
        for font_asset in &scene.assets.fonts {
            match crate::assets::font::load_font(Path::new(&font_asset.src)) {
                Ok(data) => {
                    cache.insert(font_asset.id.clone(), data);
                }
                Err(e) => {
                    tracing::warn!("failed to load custom font '{}': {e}", font_asset.id);
                }
            }
        }
        cache
    };

    // Render all frames in parallel, collect in order
    let scene = Arc::new(scene);
    let font_cache_ref = &font_cache;
    let frames: Vec<Result<Vec<u8>>> = (start..start + total)
        .into_par_iter()
        .map(|frame_num| {
            let frame_scene = evaluate_scene(&scene, frame_num, font_cache_ref)?;
            let rgba = render_frame(&frame_scene).map_err(|e| match e {
                MmotError::RenderFailed { reason, .. } => MmotError::RenderFailed {
                    frame: frame_num,
                    reason,
                },
                other => other,
            })?;
            if let Some(ref cb) = progress {
                cb(frame_num - start, total);
            }
            Ok(rgba)
        })
        .collect();

    let frames: Vec<Vec<u8>> = frames.into_iter().collect::<Result<_>>()?;

    // Collect audio from audio layers if requested
    let audio_data = if opts.include_audio {
        collect_audio(&scene)?
    } else {
        None
    };

    // Encode to output format
    match opts.format {
        OutputFormat::Mp4 => {
            match audio_data {
                Some((samples, sample_rate, channels)) => {
                    let pcm_s16 = crate::assets::audio::samples_to_pcm_s16(&samples);
                    crate::encoder::mp4::encode_with_audio(
                        frames,
                        scene.meta.width,
                        scene.meta.height,
                        scene.meta.fps,
                        opts.quality,
                        &pcm_s16,
                        sample_rate,
                        channels,
                        &opts.output_path,
                    )?;
                }
                None => {
                    crate::encoder::mp4::encode(
                        frames,
                        scene.meta.width,
                        scene.meta.height,
                        scene.meta.fps,
                        opts.quality,
                        &opts.output_path,
                    )?;
                }
            }
        }
        OutputFormat::Gif => {
            crate::encoder::gif::encode(
                frames,
                scene.meta.width,
                scene.meta.height,
                scene.meta.fps,
                &opts.output_path,
            )?;
        }
        OutputFormat::Webm => {
            let encoded = crate::encoder::av1::encode_av1(
                &frames,
                scene.meta.width,
                scene.meta.height,
                scene.meta.fps,
                opts.quality,
            )?;
            crate::encoder::ffmpeg_mux::mux_webm(
                &encoded,
                scene.meta.width,
                scene.meta.height,
                scene.meta.fps,
                &opts.output_path,
            )?;
        }
    }

    Ok(())
}

/// Collect audio from all audio layers in the root composition.
fn collect_audio(scene: &Scene) -> Result<Option<(Vec<f32>, u32, u32)>> {
    let comp = scene
        .compositions
        .get(&scene.meta.root)
        .ok_or_else(|| MmotError::Parse {
            message: format!("root composition '{}' not found", scene.meta.root),
            pointer: "/meta/root".into(),
        })?;

    let mut audio_layers = Vec::new();
    for layer in &comp.layers {
        if let LayerContent::Audio { src, .. } = &layer.content {
            let path = std::path::Path::new(src);
            match crate::assets::audio::decode_file(path) {
                Ok(decoded) => audio_layers.push(decoded),
                Err(e) => {
                    tracing::warn!("skipping audio layer '{}': {e}", layer.id);
                }
            }
        }
    }

    if audio_layers.is_empty() {
        return Ok(None);
    }

    // Use the first audio layer (multi-track mixing is Phase 3)
    let first = &audio_layers[0];
    Ok(Some((
        first.samples.clone(),
        first.sample_rate,
        first.channels,
    )))
}

/// Evaluate a scene at a specific frame number into a FrameScene.
/// Supports recursive precomp rendering.
///
/// `font_cache` maps font asset IDs to their raw bytes, loaded once before rendering.
pub fn evaluate_scene(
    scene: &Scene,
    frame: u64,
    font_cache: &HashMap<String, Vec<u8>>,
) -> Result<FrameScene> {
    let layers = evaluate_composition(scene, &scene.meta.root, frame, 0, font_cache)?;
    Ok(FrameScene {
        width: scene.meta.width,
        height: scene.meta.height,
        background: scene.meta.background.clone(),
        layers,
    })
}

/// Compute effective (start, end) frame pairs for layers in sequence mode.
///
/// Each layer plays immediately after the previous one finishes, with an
/// optional overlap derived from the composition-level transition duration.
fn compute_sequence_timing<'a>(
    layers: &'a [Layer],
    transition: Option<&TransitionSpec>,
) -> Vec<(&'a Layer, u64, u64)> {
    let overlap = match transition {
        Some(TransitionSpec::Crossfade { duration }) => *duration,
        Some(TransitionSpec::Wipe { duration, .. }) => *duration,
        Some(TransitionSpec::Slide { duration, .. }) => *duration,
        None => 0,
    };

    let mut result = Vec::new();
    let mut cursor = 0u64;
    for (i, layer) in layers.iter().enumerate() {
        let duration = layer.out_point - layer.in_point;
        let start = if i > 0 { cursor.saturating_sub(overlap) } else { cursor };
        let end = start + duration;
        result.push((layer, start, end));
        cursor = end;
    }
    result
}

/// Recursively evaluate a composition, resolving precomp references.
fn evaluate_composition(
    scene: &Scene,
    comp_id: &str,
    frame: u64,
    depth: u32,
    font_cache: &HashMap<String, Vec<u8>>,
) -> Result<Vec<ResolvedLayer>> {
    // Guard against circular composition references
    if depth > 32 {
        return Err(MmotError::RenderFailed {
            frame,
            reason: format!("composition nesting too deep (>32) — possible circular reference at '{comp_id}'"),
        });
    }

    let comp = scene.compositions.get(comp_id).ok_or_else(|| {
        MmotError::Parse {
            message: format!("composition '{comp_id}' not found"),
            pointer: format!("/compositions/{comp_id}"),
        }
    })?;

    // Compute effective timing: sequence mode lays layers back-to-back,
    // otherwise each layer uses its own in/out points.
    let timed_layers: Vec<(&Layer, u64, u64)> = if comp.sequence {
        compute_sequence_timing(&comp.layers, comp.transition.as_ref())
    } else {
        comp.layers.iter().map(|l| (l, l.in_point, l.out_point)).collect()
    };

    let mut resolved_layers = Vec::new();
    for (i, (layer, eff_in, eff_out)) in timed_layers.iter().enumerate() {
        if frame < *eff_in || frame >= *eff_out {
            continue;
        }

        let position = evaluate_vec2(&layer.transform.position, frame);
        let scale = evaluate_vec2(&layer.transform.scale, frame);
        let mut opacity = evaluate_f64(&layer.transform.opacity, frame);
        let rotation = evaluate_f64(&layer.transform.rotation, frame);

        // Apply transition opacity for overlapping layers in sequence mode
        if comp.sequence
            && let Some(ref transition) = comp.transition
        {
            let transition_dur = match transition {
                TransitionSpec::Crossfade { duration } => *duration,
                TransitionSpec::Wipe { duration, .. } => *duration,
                TransitionSpec::Slide { duration, .. } => *duration,
            };

            if transition_dur > 0 {
                // Check overlap with next layer (this layer is the outgoing one)
                if i + 1 < timed_layers.len() {
                    let (_, next_eff_in, _) = timed_layers[i + 1];
                    if frame >= next_eff_in && frame < *eff_out {
                        let overlap_len = *eff_out - next_eff_in;
                        if overlap_len > 0 {
                            let progress =
                                (frame - next_eff_in) as f64 / overlap_len as f64;
                            let (out_mult, _) =
                                crate::renderer::transition::transition_opacity(
                                    transition, progress,
                                );
                            opacity *= out_mult;
                        }
                    }
                }

                // Check overlap with previous layer (this layer is the incoming one)
                if i > 0 {
                    let (_, _, prev_eff_out) = timed_layers[i - 1];
                    if frame < prev_eff_out && frame >= *eff_in {
                        let overlap_len = prev_eff_out - *eff_in;
                        if overlap_len > 0 {
                            let progress =
                                (frame - *eff_in) as f64 / overlap_len as f64;
                            let (_, in_mult) =
                                crate::renderer::transition::transition_opacity(
                                    transition, progress,
                                );
                            opacity *= in_mult;
                        }
                    }
                }
            }
        }

        let transform = ResolvedTransform {
            position,
            scale,
            rotation,
            opacity,
        };

        let content = match &layer.content {
            LayerContent::Solid { color } => ResolvedContent::Solid {
                color: color.clone(),
            },
            LayerContent::Text { text, font, align } => {
                let custom_font_data = font_cache.get(&font.family).cloned();
                ResolvedContent::Text {
                    text: text.clone(),
                    font_family: font.family.clone(),
                    font_size: font.size,
                    font_weight: font.weight,
                    color: font.color.clone(),
                    align: align.clone(),
                    custom_font_data,
                }
            }
            LayerContent::Shape { shape } => {
                let resolved = match shape {
                    ShapeSpec::Rect {
                        width,
                        height,
                        corner_radius,
                        fill,
                        stroke,
                    } => ResolvedShape::Rect {
                        width: *width,
                        height: *height,
                        corner_radius: corner_radius.unwrap_or(0.0),
                        fill: fill.clone(),
                        stroke_color: stroke.as_ref().map(|s| s.color.clone()),
                        stroke_width: stroke.as_ref().map(|s| s.width).unwrap_or(0.0),
                    },
                    ShapeSpec::Ellipse {
                        width,
                        height,
                        fill,
                        stroke,
                    } => ResolvedShape::Ellipse {
                        width: *width,
                        height: *height,
                        fill: fill.clone(),
                        stroke_color: stroke.as_ref().map(|s| s.color.clone()),
                        stroke_width: stroke.as_ref().map(|s| s.width).unwrap_or(0.0),
                    },
                    ShapeSpec::Line {
                        x1,
                        y1,
                        x2,
                        y2,
                        stroke,
                    } => ResolvedShape::Line {
                        x1: *x1,
                        y1: *y1,
                        x2: *x2,
                        y2: *y2,
                        stroke_color: stroke.color.clone(),
                        stroke_width: stroke.width,
                    },
                    ShapeSpec::Polygon {
                        points,
                        fill,
                        stroke,
                    } => ResolvedShape::Polygon {
                        points: points.clone(),
                        fill: fill.clone(),
                        stroke_color: stroke.as_ref().map(|s| s.color.clone()),
                        stroke_width: stroke.as_ref().map(|s| s.width).unwrap_or(0.0),
                    },
                };
                ResolvedContent::Shape { shape: resolved }
            }
            LayerContent::Gradient { gradient } => ResolvedContent::Gradient {
                gradient: gradient.clone(),
                width: scene.meta.width,
                height: scene.meta.height,
            },
            LayerContent::Composition { id } => {
                // Recursively render the referenced composition
                let sub_layers =
                    evaluate_composition(scene, id, frame, depth + 1, font_cache)?;
                resolved_layers.extend(sub_layers);
                continue;
            }
            LayerContent::Image { src } => {
                // Load image from disk
                match load_image_asset(src) {
                    Ok((data, w, h)) => ResolvedContent::Image {
                        data,
                        width: w,
                        height: h,
                    },
                    Err(e) => {
                        tracing::warn!("skipping image layer '{}': {e}", layer.id);
                        continue;
                    }
                }
            }
            // Audio, Video, Lottie — skip with warning for now
            LayerContent::Audio { .. } => {
                // Audio doesn't produce visual output — handled separately
                continue;
            }
            LayerContent::Video { src, trim_start, .. } => {
                let scene_time = frame as f64 / scene.meta.fps;
                let video_time = scene_time + *trim_start;
                match crate::assets::video::decode_frame(Path::new(src), video_time) {
                    Ok(decoded) => ResolvedContent::Image {
                        data: decoded.rgba,
                        width: decoded.width,
                        height: decoded.height,
                    },
                    Err(e) => {
                        tracing::warn!("skipping video layer '{}': {e}", layer.id);
                        continue;
                    }
                }
            }
            LayerContent::Lottie { .. } => {
                tracing::warn!(
                    "lottie layer '{}' not yet implemented — skipping",
                    layer.id
                );
                continue;
            }
            LayerContent::Null => {
                // Null layers produce no visual output — used only for parenting
                continue;
            }
        };

        resolved_layers.push(ResolvedLayer {
            opacity,
            transform,
            content,
            fill_parent: layer.fill.as_ref().is_some_and(|f| matches!(f, crate::schema::composition::FillMode::Parent)),
            blend_mode: layer.blend_mode.clone(),
        });
    }

    Ok(resolved_layers)
}

/// Load an image file and decode it to RGBA.
fn load_image_asset(src: &str) -> Result<(Vec<u8>, u32, u32)> {
    let path = Path::new(src);
    if !path.exists() {
        return Err(MmotError::AssetNotFound {
            path: path.to_path_buf(),
        });
    }
    let data = std::fs::read(path).map_err(MmotError::Io)?;
    let decoded = crate::assets::image::decode(&data)?;
    Ok((decoded.rgba, decoded.width, decoded.height))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pipeline_renders_minimal_scene() {
        let json = include_str!("../../../tests/fixtures/valid/minimal.mmot.json");
        let opts = RenderOptions {
            output_path: std::env::temp_dir().join("mmot-test-output.mp4"),
            format: OutputFormat::Mp4,
            quality: 80,
            frame_range: None,
            concurrency: Some(2),
            backend: RenderBackend::Cpu,
            include_audio: false,
        };
        render_scene(json, opts, None).unwrap();
    }

    #[test]
    fn props_substitution_works() {
        let json = r##"{
            "version": "1.0",
            "meta": {"name":"Props Test","width":64,"height":64,"fps":30,"duration":1,"background":"#000000","root":"main"},
            "compositions": {
                "main": {
                    "layers": [{
                        "id": "bg",
                        "type": "solid",
                        "in": 0, "out": 1,
                        "color": "${bg_color}",
                        "transform": {"position":[32.0,32.0],"scale":[1.0,1.0],"opacity":1.0,"rotation":0.0}
                    }]
                }
            }
        }"##;

        let mut cli_props = HashMap::new();
        cli_props.insert("bg_color".into(), "#ff0000".into());

        let substituted = props::substitute(json, &cli_props);
        let scene = parse(&substituted).unwrap();
        let layer = &scene.compositions["main"].layers[0];
        if let LayerContent::Solid { color } = &layer.content {
            assert_eq!(color, "#ff0000");
        } else {
            panic!("expected solid layer");
        }
    }

    #[test]
    fn pipeline_renders_with_audio() {
        let json = include_str!("../../../tests/fixtures/valid/audio_mix.mmot.json");
        let opts = RenderOptions {
            output_path: std::env::temp_dir().join("mmot-test-audio.mp4"),
            format: OutputFormat::Mp4,
            quality: 80,
            frame_range: None,
            concurrency: Some(2),
            backend: RenderBackend::Cpu,
            include_audio: true,
        };
        render_scene(json, opts, None).unwrap();
        let metadata =
            std::fs::metadata(std::env::temp_dir().join("mmot-test-audio.mp4")).unwrap();
        assert!(metadata.len() > 0);
    }

    #[test]
    fn precomp_renders_nested_composition() {
        let json = r##"{
            "version": "1.0",
            "meta": {"name":"Precomp","width":64,"height":64,"fps":30,"duration":1,"background":"#000000","root":"main"},
            "compositions": {
                "main": {
                    "layers": [{
                        "id": "precomp-ref",
                        "type": "composition",
                        "in": 0, "out": 1,
                        "composition_id": "sub",
                        "transform": {"position":[32.0,32.0],"scale":[1.0,1.0],"opacity":1.0,"rotation":0.0}
                    }]
                },
                "sub": {
                    "layers": [{
                        "id": "sub-bg",
                        "type": "solid",
                        "in": 0, "out": 1,
                        "color": "#00ff00",
                        "transform": {"position":[32.0,32.0],"scale":[1.0,1.0],"opacity":1.0,"rotation":0.0}
                    }]
                }
            }
        }"##;

        let scene = parse(json).unwrap();
        let no_fonts = HashMap::new();
        let frame_scene = evaluate_scene(&scene, 0, &no_fonts).unwrap();
        // The precomp should have resolved the sub composition's solid layer
        assert_eq!(frame_scene.layers.len(), 1);
        assert!(matches!(
            frame_scene.layers[0].content,
            ResolvedContent::Solid { .. }
        ));
    }

    #[test]
    fn sequence_layers_play_back_to_back() {
        let json = include_str!("../../../tests/fixtures/valid/sequence.mmot.json");
        let scene = parse(json).unwrap();
        let no_fonts = HashMap::new();

        // With crossfade duration=10 and each layer duration=30:
        //   Layer 0 (red):  frames 0..30
        //   Layer 1 (blue): frames 20..50  (starts at 30-10=20)

        // Frame 0: only red visible
        let fs0 = evaluate_scene(&scene, 0, &no_fonts).unwrap();
        assert_eq!(fs0.layers.len(), 1);
        assert!(matches!(
            fs0.layers[0].content,
            ResolvedContent::Solid { ref color } if color == "#ff0000"
        ));

        // Frame 25: both visible (overlap zone: red 0..30, blue 20..50)
        let fs25 = evaluate_scene(&scene, 25, &no_fonts).unwrap();
        assert_eq!(
            fs25.layers.len(),
            2,
            "during crossfade overlap, both layers should be active"
        );

        // Frame 40: only blue visible
        let fs40 = evaluate_scene(&scene, 40, &no_fonts).unwrap();
        assert_eq!(fs40.layers.len(), 1);
        assert!(matches!(
            fs40.layers[0].content,
            ResolvedContent::Solid { ref color } if color == "#0000ff"
        ));
    }

    #[test]
    fn sequence_without_transition_no_overlap() {
        let json = r##"{
            "version": "1.0",
            "meta": {"name":"SeqNoTrans","width":64,"height":64,"fps":30,"duration":60,"background":"#000000","root":"main"},
            "compositions": {
                "main": {
                    "sequence": true,
                    "layers": [
                        {
                            "id": "a",
                            "type": "solid",
                            "in": 0, "out": 20,
                            "color": "#ff0000",
                            "transform": {"position":[32,32],"scale":[1,1],"opacity":1.0,"rotation":0.0}
                        },
                        {
                            "id": "b",
                            "type": "solid",
                            "in": 0, "out": 20,
                            "color": "#00ff00",
                            "transform": {"position":[32,32],"scale":[1,1],"opacity":1.0,"rotation":0.0}
                        }
                    ]
                }
            }
        }"##;

        let scene = parse(json).unwrap();
        let no_fonts = HashMap::new();

        // Layer a: 0..20, Layer b: 20..40 (no overlap)

        // Frame 10: only layer a
        let fs10 = evaluate_scene(&scene, 10, &no_fonts).unwrap();
        assert_eq!(fs10.layers.len(), 1);
        assert!(matches!(
            fs10.layers[0].content,
            ResolvedContent::Solid { ref color } if color == "#ff0000"
        ));

        // Frame 19: still only layer a (out_point is exclusive)
        let fs19 = evaluate_scene(&scene, 19, &no_fonts).unwrap();
        assert_eq!(fs19.layers.len(), 1);

        // Frame 20: only layer b (a ended at 20, b starts at 20)
        let fs20 = evaluate_scene(&scene, 20, &no_fonts).unwrap();
        assert_eq!(fs20.layers.len(), 1);
        assert!(matches!(
            fs20.layers[0].content,
            ResolvedContent::Solid { ref color } if color == "#00ff00"
        ));
    }

    #[test]
    fn non_sequence_composition_unchanged() {
        // Ensure sequence=false (default) uses in/out points as-is
        let json = r##"{
            "version": "1.0",
            "meta": {"name":"Normal","width":64,"height":64,"fps":30,"duration":30,"background":"#000000","root":"main"},
            "compositions": {
                "main": {
                    "layers": [{
                        "id": "bg",
                        "type": "solid",
                        "in": 5, "out": 20,
                        "color": "#ff0000",
                        "transform": {"position":[32,32],"scale":[1,1],"opacity":1.0,"rotation":0.0}
                    }]
                }
            }
        }"##;

        let scene = parse(json).unwrap();
        let no_fonts = HashMap::new();

        // Frame 0: layer not yet visible
        let fs0 = evaluate_scene(&scene, 0, &no_fonts).unwrap();
        assert_eq!(fs0.layers.len(), 0);

        // Frame 10: layer visible
        let fs10 = evaluate_scene(&scene, 10, &no_fonts).unwrap();
        assert_eq!(fs10.layers.len(), 1);

        // Frame 20: layer no longer visible (exclusive out)
        let fs20 = evaluate_scene(&scene, 20, &no_fonts).unwrap();
        assert_eq!(fs20.layers.len(), 0);
    }

    #[test]
    fn crossfade_modifies_opacity() {
        let json = include_str!("../../../tests/fixtures/valid/sequence.mmot.json");
        let scene = parse(json).unwrap();
        let no_fonts = HashMap::new();

        // Frame 25 is in the overlap zone (red: 0-30, blue: 20-50, overlap: 20-30)
        // Progress at frame 25: (25-20)/(30-20) = 0.5
        let fs = evaluate_scene(&scene, 25, &no_fonts).unwrap();
        assert_eq!(fs.layers.len(), 2);

        // First layer (red/outgoing) should have opacity ~0.5
        let out_opacity = fs.layers[0].opacity;
        assert!(
            (out_opacity - 0.5).abs() < 0.1,
            "outgoing opacity should be ~0.5, got {out_opacity}"
        );

        // Second layer (blue/incoming) should have opacity ~0.5
        let in_opacity = fs.layers[1].opacity;
        assert!(
            (in_opacity - 0.5).abs() < 0.1,
            "incoming opacity should be ~0.5, got {in_opacity}"
        );
    }

    #[test]
    fn video_layer_without_ffmpeg_skips_gracefully() {
        let json = r##"{
            "version": "1.0",
            "meta": {"name":"Vid","width":64,"height":64,"fps":30,"duration":1,"background":"#000000","root":"main"},
            "compositions": {"main": {"layers": [{
                "id": "vid", "type": "video", "in": 0, "out": 1,
                "src": "nonexistent.mp4",
                "transform": {"position":[32,32],"scale":[1,1],"opacity":1.0,"rotation":0.0}
            }]}}
        }"##;
        let scene = parse(json).expect("should parse video layer JSON");
        let font_cache = HashMap::new();
        let fs = evaluate_scene(&scene, 0, &font_cache).expect("evaluate should not fail");
        // Without the ffmpeg feature (or with a missing file), the video layer is skipped
        assert_eq!(fs.layers.len(), 0);
    }

    #[test]
    fn deserialise_fill_parent() {
        let json = r##"{
            "version": "1.0",
            "meta": {"name":"Fill","width":100,"height":100,"fps":30,"duration":1,"background":"#000","root":"main"},
            "compositions": {"main": {"layers": [{
                "id": "bg", "type": "solid", "in": 0, "out": 1,
                "color": "#ff0000", "fill": "parent",
                "transform": {"position":[0,0],"scale":[1,1],"opacity":1.0,"rotation":0.0}
            }]}}
        }"##;
        let scene = parse(json).unwrap();
        let layer = &scene.compositions["main"].layers[0];
        assert!(layer.fill.is_some());
    }

    #[test]
    fn fill_parent_resolves_in_pipeline() {
        let json = r##"{
            "version": "1.0",
            "meta": {"name":"Fill","width":100,"height":100,"fps":30,"duration":1,"background":"#000","root":"main"},
            "compositions": {"main": {"layers": [{
                "id": "bg", "type": "solid", "in": 0, "out": 1,
                "color": "#ff0000", "fill": "parent",
                "transform": {"position":[0,0],"scale":[1,1],"opacity":1.0,"rotation":0.0}
            }]}}
        }"##;
        let scene = parse(json).unwrap();
        let font_cache = HashMap::new();
        let fs = evaluate_scene(&scene, 0, &font_cache).unwrap();
        assert_eq!(fs.layers.len(), 1);
        assert!(fs.layers[0].fill_parent);
    }
}
