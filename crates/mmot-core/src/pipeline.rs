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
use crate::schema::{Layer, LayerContent, Scene, ShapeSpec, TransitionSpec, Vec2};

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

/// Scene metadata returned by `get_scene_info`.
#[derive(Debug, Clone)]
pub struct SceneInfo {
    pub width: u32,
    pub height: u32,
    pub fps: f64,
    pub duration_frames: u64,
    pub duration_secs: f64,
    pub composition_count: usize,
    pub root_layer_count: usize,
}

/// Get metadata about a scene without rendering.
pub fn get_scene_info(json: &str) -> Result<SceneInfo> {
    let scene = parse(json)?;
    let root_layers = scene
        .compositions
        .get(&scene.meta.root)
        .map(|c| c.layers.len())
        .unwrap_or(0);
    Ok(SceneInfo {
        width: scene.meta.width,
        height: scene.meta.height,
        fps: scene.meta.fps,
        duration_frames: scene.meta.duration,
        duration_secs: scene.meta.duration as f64 / scene.meta.fps,
        composition_count: scene.compositions.len(),
        root_layer_count: root_layers,
    })
}

/// Render a single frame from a JSON scene as raw RGBA bytes.
///
/// Returns `(width, height, rgba_bytes)`. Use this for live preview in a UI
/// — it skips encoding entirely and just returns the pixel buffer.
pub fn render_single_frame(
    json: &str,
    frame_number: u64,
) -> Result<(u32, u32, Vec<u8>)> {
    render_single_frame_with_props(json, &HashMap::new(), frame_number)
}

/// Render a single frame with props substitution.
pub fn render_single_frame_with_props(
    json: &str,
    cli_props: &HashMap<String, String>,
    frame_number: u64,
) -> Result<(u32, u32, Vec<u8>)> {
    let json = props::substitute(json, cli_props);
    let scene = parse(&json)?;

    let font_cache: HashMap<String, Vec<u8>> = {
        let mut cache = HashMap::new();
        for font_asset in &scene.assets.fonts {
            match crate::assets::font::load_font(Path::new(&font_asset.src)) {
                Ok(data) => { cache.insert(font_asset.id.clone(), data); }
                Err(e) => { tracing::warn!("failed to load font '{}': {e}", font_asset.id); }
            }
        }
        cache
    };

    let frame_scene = evaluate_scene(&scene, frame_number, &font_cache)?;
    let w = frame_scene.width;
    let h = frame_scene.height;
    let rgba = render_frame(&frame_scene)?;
    Ok((w, h, rgba))
}

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
    let has_motion_blur = scene_has_motion_blur(&scene);

    let frames: Vec<Result<Vec<u8>>> = (start..start + total)
        .into_par_iter()
        .map(|frame_num| {
            let rgba = if has_motion_blur {
                // Render multiple sub-frames and average for temporal motion blur
                const MOTION_BLUR_OFFSETS: [f64; 5] = [-0.4, -0.2, 0.0, 0.2, 0.4];
                let sub_frames: Vec<Vec<u8>> = MOTION_BLUR_OFFSETS
                    .iter()
                    .filter_map(|offset| {
                        let sub = (frame_num as f64 + offset).max(0.0) as u64;
                        let fs = evaluate_scene(&scene, sub, font_cache_ref).ok()?;
                        render_frame(&fs).ok()
                    })
                    .collect();
                if sub_frames.is_empty() {
                    // Fallback: render normally
                    let fs = evaluate_scene(&scene, frame_num, font_cache_ref)?;
                    render_frame(&fs).map_err(|e| match e {
                        MmotError::RenderFailed { reason, .. } => MmotError::RenderFailed {
                            frame: frame_num,
                            reason,
                        },
                        other => other,
                    })?
                } else {
                    average_frames(&sub_frames)
                }
            } else {
                let frame_scene = evaluate_scene(&scene, frame_num, font_cache_ref)?;
                render_frame(&frame_scene).map_err(|e| match e {
                    MmotError::RenderFailed { reason, .. } => MmotError::RenderFailed {
                        frame: frame_num,
                        reason,
                    },
                    other => other,
                })?
            };
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

/// Check if any layer in the scene has motion blur enabled.
fn scene_has_motion_blur(scene: &Scene) -> bool {
    scene.compositions.values().any(|comp| {
        comp.layers.iter().any(|layer| layer.motion_blur)
    })
}

/// Average multiple RGBA frame buffers by computing the mean of each byte.
fn average_frames(frames: &[Vec<u8>]) -> Vec<u8> {
    let len = frames[0].len();
    let n = frames.len() as u32;
    (0..len)
        .map(|i| {
            let sum: u32 = frames.iter().map(|f| f[i] as u32).sum();
            (sum / n) as u8
        })
        .collect()
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

/// Evaluate position along a polyline path at normalised time t (0.0–1.0).
/// Linearly interpolates between consecutive points.
fn evaluate_path_position(points: &[[f64; 2]], t: f64) -> (f64, f64) {
    if points.is_empty() {
        return (0.0, 0.0);
    }
    if points.len() == 1 {
        return (points[0][0], points[0][1]);
    }
    let t = t.clamp(0.0, 1.0);
    let segments = (points.len() - 1) as f64;
    let raw = t * segments;
    let idx = (raw.floor() as usize).min(points.len() - 2);
    let frac = raw - idx as f64;
    let a = &points[idx];
    let b = &points[idx + 1];
    (
        a[0] + (b[0] - a[0]) * frac,
        a[1] + (b[1] - a[1]) * frac,
    )
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

/// Walk the parent chain and concatenate transforms.
/// Guards against circular parenting with a max depth of 32.
fn resolve_parent_chain(
    layer_id: &str,
    transforms: &HashMap<String, (ResolvedTransform, Option<String>, u64)>,
    depth: u32,
) -> ResolvedTransform {
    if depth > 32 {
        return ResolvedTransform {
            position: Vec2 { x: 0.0, y: 0.0 },
            scale: Vec2 { x: 1.0, y: 1.0 },
            rotation: 0.0,
            opacity: 1.0,
        };
    }
    let (transform, parent) = match transforms.get(layer_id) {
        Some(data) => (&data.0, &data.1),
        None => {
            return ResolvedTransform {
                position: Vec2 { x: 0.0, y: 0.0 },
                scale: Vec2 { x: 1.0, y: 1.0 },
                rotation: 0.0,
                opacity: 1.0,
            };
        }
    };
    match parent {
        None => transform.clone(),
        Some(pid) => {
            let parent_t = resolve_parent_chain(pid, transforms, depth + 1);
            ResolvedTransform {
                position: Vec2 {
                    x: transform.position.x + parent_t.position.x,
                    y: transform.position.y + parent_t.position.y,
                },
                scale: Vec2 {
                    x: transform.scale.x * parent_t.scale.x,
                    y: transform.scale.y * parent_t.scale.y,
                },
                rotation: transform.rotation + parent_t.rotation,
                opacity: transform.opacity * parent_t.opacity,
            }
        }
    }
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

    // ── Pass 1: Compute raw transforms for ALL active layers (including Null) ──
    // This is needed so parent transforms can be looked up by child layers.
    // Also stores effective_frame for use by trim_paths evaluation in pass 2.
    let mut transform_map: HashMap<String, (ResolvedTransform, Option<String>, u64)> =
        HashMap::new();

    for (i, (layer, eff_in, eff_out)) in timed_layers.iter().enumerate() {
        if frame < *eff_in || frame >= *eff_out {
            continue;
        }

        // Compute effective frame with time remapping
        let effective_frame = match &layer.time_remap {
            Some(tr) => {
                let duration = (layer.out_point - layer.in_point) as f64;
                let local_frame = (frame - *eff_in) as f64;
                let f = local_frame * tr.speed + tr.offset;
                let f = if tr.reverse { duration - f } else { f };
                f.clamp(0.0, duration - 1.0) as u64 + layer.in_point
            }
            None => frame,
        };

        let mut position = evaluate_vec2(&layer.transform.position, effective_frame);
        let scale = evaluate_vec2(&layer.transform.scale, effective_frame);
        let mut opacity = evaluate_f64(&layer.transform.opacity, effective_frame);
        let mut rotation = evaluate_f64(&layer.transform.rotation, effective_frame);

        // Apply path animation: move position along a polyline path
        if let Some(ref path_anim) = layer.path_animation {
            let t = if layer.out_point > layer.in_point {
                (effective_frame.saturating_sub(layer.in_point)) as f64
                    / (layer.out_point - layer.in_point) as f64
            } else {
                0.0
            };
            let t = t.clamp(0.0, 1.0);
            let (px, py) = evaluate_path_position(&path_anim.points, t);
            position = Vec2 { x: px, y: py };
            if path_anim.auto_orient {
                let dt = 0.001;
                let (px2, py2) =
                    evaluate_path_position(&path_anim.points, (t + dt).min(1.0));
                rotation = f64::atan2(py2 - py, px2 - px).to_degrees();
            }
        }

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

        transform_map.insert(
            layer.id.clone(),
            (transform, layer.parent.clone(), effective_frame),
        );
    }

    // ── Pass 2: Resolve parent chains and build final layers ──
    let mut resolved_layers = Vec::new();
    for (layer, eff_in, eff_out) in timed_layers.iter() {
        if frame < *eff_in || frame >= *eff_out {
            continue;
        }

        // Resolve the transform with parent chain
        let transform = resolve_parent_chain(&layer.id, &transform_map, 0);
        let opacity = transform.opacity;

        // Retrieve the effective frame stored in pass 1 for trim_paths evaluation
        let effective_frame = transform_map
            .get(&layer.id)
            .map(|(_, _, ef)| *ef)
            .unwrap_or(frame);

        // Evaluate trim paths for shape layers
        let (trim_start, trim_end) = match &layer.trim_paths {
            Some(tp) => (
                evaluate_f64(&tp.start, effective_frame).clamp(0.0, 1.0),
                evaluate_f64(&tp.end, effective_frame).clamp(0.0, 1.0),
            ),
            None => (0.0, 1.0),
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
            masks: layer.masks.clone(),
            effects: layer.effects.clone(),
            adjustment: layer.adjustment,
            track_matte_source: layer.track_matte.as_ref().map(|tm| tm.source.clone()),
            trim_start,
            trim_end,
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

    #[test]
    fn time_remap_double_speed() {
        // Create a scene with a layer that has time_remap speed=2.0
        // At frame 5, keyframes should evaluate as if at frame 10
        let json = r##"{
            "version": "1.0",
            "meta": { "name": "T", "width": 64, "height": 64, "fps": 30, "duration": 30, "root": "main", "background": "#000000" },
            "compositions": {
                "main": {
                    "layers": [{
                        "id": "fast",
                        "in": 0, "out": 30,
                        "transform": {
                            "position": [
                                { "t": 0, "v": [0, 0] },
                                { "t": 30, "v": [100, 0] }
                            ]
                        },
                        "type": "solid",
                        "color": "#ff0000",
                        "time_remap": { "speed": 2.0 }
                    }]
                }
            },
            "assets": { "fonts": [] }
        }"##;
        let scene = crate::parser::parse(json).unwrap();
        let font_cache = std::collections::HashMap::new();
        // At frame 5 with 2x speed, position should be further than normal
        let fs = evaluate_scene(&scene, 5, &font_cache).unwrap();
        assert!(!fs.layers.is_empty());
        let pos_x = fs.layers[0].transform.position.x;
        // At 2x speed, frame 5 evaluates as frame 10 — position ~33.3
        assert!(pos_x > 20.0, "expected position > 20 with 2x speed, got {pos_x}");
    }

    #[test]
    fn parent_transform_offsets_child() {
        let json = r##"{
            "version": "1.0",
            "meta": { "name": "T", "width": 64, "height": 64, "fps": 30, "duration": 30, "root": "main", "background": "#000000" },
            "compositions": {
                "main": {
                    "layers": [
                        {
                            "id": "parent_null",
                            "in": 0, "out": 30,
                            "transform": { "position": [100, 100] },
                            "type": "null"
                        },
                        {
                            "id": "child",
                            "in": 0, "out": 30,
                            "transform": { "position": [50, 50] },
                            "type": "solid",
                            "color": "#ff0000",
                            "parent": "parent_null"
                        }
                    ]
                }
            },
            "assets": { "fonts": [] }
        }"##;
        let scene = crate::parser::parse(json).unwrap();
        let font_cache = std::collections::HashMap::new();
        let fs = evaluate_scene(&scene, 0, &font_cache).unwrap();
        // child should be at (150, 150) = parent(100,100) + child(50,50)
        let child = &fs.layers[0]; // Null layers are skipped, so child is first visible
        assert!(
            (child.transform.position.x - 150.0).abs() < 1.0,
            "expected x=150, got {}", child.transform.position.x
        );
        assert!(
            (child.transform.position.y - 150.0).abs() < 1.0,
            "expected y=150, got {}", child.transform.position.y
        );
    }

    #[test]
    fn time_remap_reverse() {
        let json = r##"{
            "version": "1.0",
            "meta": { "name": "T", "width": 64, "height": 64, "fps": 30, "duration": 30, "root": "main", "background": "#000000" },
            "compositions": {
                "main": {
                    "layers": [{
                        "id": "rev",
                        "in": 0, "out": 30,
                        "transform": {
                            "position": [
                                { "t": 0, "v": [0, 0] },
                                { "t": 29, "v": [100, 0] }
                            ]
                        },
                        "type": "solid",
                        "color": "#ff0000",
                        "time_remap": { "speed": 1.0, "reverse": true }
                    }]
                }
            },
            "assets": { "fonts": [] }
        }"##;
        let scene = crate::parser::parse(json).unwrap();
        let font_cache = std::collections::HashMap::new();
        // At frame 0 reversed, position should be near the end (~100)
        let fs = evaluate_scene(&scene, 0, &font_cache).unwrap();
        assert!(!fs.layers.is_empty());
        let pos_x = fs.layers[0].transform.position.x;
        assert!(pos_x > 90.0, "expected position > 90 for reversed frame 0, got {pos_x}");
    }

    #[test]
    fn parent_chain_scale_multiplies() {
        let json = r##"{
            "version": "1.0",
            "meta": { "name": "T", "width": 64, "height": 64, "fps": 30, "duration": 30, "root": "main", "background": "#000000" },
            "compositions": {
                "main": {
                    "layers": [
                        {
                            "id": "p",
                            "in": 0, "out": 30,
                            "transform": { "position": [0, 0], "scale": [2.0, 2.0] },
                            "type": "null"
                        },
                        {
                            "id": "c",
                            "in": 0, "out": 30,
                            "transform": { "position": [10, 10], "scale": [0.5, 0.5] },
                            "type": "solid",
                            "color": "#ff0000",
                            "parent": "p"
                        }
                    ]
                }
            },
            "assets": { "fonts": [] }
        }"##;
        let scene = crate::parser::parse(json).unwrap();
        let font_cache = std::collections::HashMap::new();
        let fs = evaluate_scene(&scene, 0, &font_cache).unwrap();
        assert_eq!(fs.layers.len(), 1);
        let child = &fs.layers[0];
        // scale should be 2.0 * 0.5 = 1.0
        assert!(
            (child.transform.scale.x - 1.0).abs() < 0.01,
            "expected scale.x=1.0, got {}", child.transform.scale.x
        );
    }

    #[test]
    fn circular_parent_does_not_crash() {
        // Two layers pointing to each other as parents — should not stack overflow
        let json = r##"{
            "version": "1.0",
            "meta": { "name": "T", "width": 64, "height": 64, "fps": 30, "duration": 30, "root": "main", "background": "#000000" },
            "compositions": {
                "main": {
                    "layers": [
                        {
                            "id": "a",
                            "in": 0, "out": 30,
                            "transform": { "position": [10, 10] },
                            "type": "solid",
                            "color": "#ff0000",
                            "parent": "b"
                        },
                        {
                            "id": "b",
                            "in": 0, "out": 30,
                            "transform": { "position": [20, 20] },
                            "type": "solid",
                            "color": "#00ff00",
                            "parent": "a"
                        }
                    ]
                }
            },
            "assets": { "fonts": [] }
        }"##;
        let scene = crate::parser::parse(json).unwrap();
        let font_cache = std::collections::HashMap::new();
        // Should not panic or hang — depth guard kicks in
        let fs = evaluate_scene(&scene, 0, &font_cache);
        assert!(fs.is_ok());
    }

    #[test]
    fn motion_blur_produces_output() {
        // Simple scene with motion_blur on a moving layer
        let json = r##"{
            "version": "1.0",
            "meta": { "name": "MotionBlur", "width": 32, "height": 32, "fps": 30, "duration": 10, "root": "main", "background": "#000000" },
            "compositions": {
                "main": {
                    "layers": [{
                        "id": "moving",
                        "in": 0, "out": 10,
                        "transform": {
                            "position": [
                                { "t": 0, "v": [0, 16] },
                                { "t": 10, "v": [32, 16] }
                            ]
                        },
                        "type": "solid",
                        "color": "#ffffff",
                        "motion_blur": true
                    }]
                }
            },
            "assets": { "fonts": [] }
        }"##;
        let scene = crate::parser::parse(json).unwrap();
        assert!(scene_has_motion_blur(&scene));

        let font_cache = std::collections::HashMap::new();
        let fs = evaluate_scene(&scene, 5, &font_cache).unwrap();
        let rgba = crate::renderer::render(&fs).unwrap();
        assert_eq!(rgba.len(), 32 * 32 * 4);
    }

    #[test]
    fn scene_without_motion_blur_detected() {
        let json = r##"{
            "version": "1.0",
            "meta": { "name": "NoBlur", "width": 32, "height": 32, "fps": 30, "duration": 10, "root": "main", "background": "#000000" },
            "compositions": {
                "main": {
                    "layers": [{
                        "id": "still",
                        "in": 0, "out": 10,
                        "transform": { "position": [16, 16] },
                        "type": "solid",
                        "color": "#ffffff"
                    }]
                }
            }
        }"##;
        let scene = crate::parser::parse(json).unwrap();
        assert!(!scene_has_motion_blur(&scene));
    }

    #[test]
    fn average_frames_computes_mean() {
        let a = vec![0u8, 100, 200, 255];
        let b = vec![100u8, 100, 0, 255];
        let result = average_frames(&[a, b]);
        assert_eq!(result, vec![50, 100, 100, 255]);
    }

    #[test]
    fn trim_paths_changes_output() {
        // A horizontal line with full stroke vs half stroke should produce different output.
        // Lines are drawn at absolute coordinates in local space, so we use coordinates
        // that are fully within the canvas.
        let full_json = r##"{
            "version": "1.0",
            "meta": { "name": "TrimFull", "width": 100, "height": 100, "fps": 30, "duration": 1, "root": "main", "background": "#000000" },
            "compositions": {
                "main": {
                    "layers": [{
                        "id": "line",
                        "in": 0, "out": 1,
                        "transform": { "position": [50, 50] },
                        "type": "shape",
                        "shape": {
                            "shape_type": "line",
                            "x1": 0, "y1": 0, "x2": 99, "y2": 0,
                            "stroke": { "color": "#ffffff", "width": 2.0 }
                        }
                    }]
                }
            },
            "assets": { "fonts": [] }
        }"##;
        let half_json = r##"{
            "version": "1.0",
            "meta": { "name": "TrimHalf", "width": 100, "height": 100, "fps": 30, "duration": 1, "root": "main", "background": "#000000" },
            "compositions": {
                "main": {
                    "layers": [{
                        "id": "line",
                        "in": 0, "out": 1,
                        "transform": { "position": [50, 50] },
                        "type": "shape",
                        "shape": {
                            "shape_type": "line",
                            "x1": 0, "y1": 0, "x2": 99, "y2": 0,
                            "stroke": { "color": "#ffffff", "width": 2.0 }
                        },
                        "trim_paths": { "start": 0.0, "end": 0.5 }
                    }]
                }
            },
            "assets": { "fonts": [] }
        }"##;

        let scene_full = crate::parser::parse(full_json).expect("full scene parses");
        let scene_half = crate::parser::parse(half_json).expect("half scene parses");
        let font_cache = std::collections::HashMap::new();

        let fs_full = evaluate_scene(&scene_full, 0, &font_cache).expect("full evaluates");
        let fs_half = evaluate_scene(&scene_half, 0, &font_cache).expect("half evaluates");

        // The half-trimmed layer should have trim_end = 0.5
        assert!(
            (fs_half.layers[0].trim_end - 0.5).abs() < 0.01,
            "expected trim_end 0.5, got {}",
            fs_half.layers[0].trim_end,
        );

        let rgba_full = crate::renderer::render(&fs_full).expect("full renders");
        let rgba_half = crate::renderer::render(&fs_half).expect("half renders");

        // Full stroke and half stroke should produce different pixel output
        assert_ne!(rgba_full, rgba_half, "trim_paths should change rendered output");
    }

    #[test]
    fn path_animation_moves_position() {
        let json = r##"{
            "version": "1.0",
            "meta": { "name": "PathAnim", "width": 64, "height": 64, "fps": 30, "duration": 30, "root": "main", "background": "#000000" },
            "compositions": {
                "main": {
                    "layers": [{
                        "id": "mover",
                        "in": 0, "out": 30,
                        "transform": { "position": [0, 0] },
                        "type": "solid",
                        "color": "#ff0000",
                        "path_animation": {
                            "points": [[0, 0], [64, 0], [64, 64]],
                            "auto_orient": false
                        }
                    }]
                }
            },
            "assets": { "fonts": [] }
        }"##;
        let scene = crate::parser::parse(json).expect("path_animation scene should parse");
        let font_cache = std::collections::HashMap::new();

        // At frame 0, should be at start (0,0)
        let fs0 = evaluate_scene(&scene, 0, &font_cache).expect("frame 0 evaluates");
        assert!(!fs0.layers.is_empty());
        assert!(
            (fs0.layers[0].transform.position.x - 0.0).abs() < 1.0,
            "frame 0: expected x near 0, got {}",
            fs0.layers[0].transform.position.x,
        );

        // At frame 15 (midpoint), should be at (64, 0)
        let fs15 = evaluate_scene(&scene, 15, &font_cache).expect("frame 15 evaluates");
        assert!(
            (fs15.layers[0].transform.position.x - 64.0).abs() < 2.0,
            "frame 15: expected x near 64, got {}",
            fs15.layers[0].transform.position.x,
        );
        assert!(
            (fs15.layers[0].transform.position.y - 0.0).abs() < 2.0,
            "frame 15: expected y near 0, got {}",
            fs15.layers[0].transform.position.y,
        );
    }

    #[test]
    fn path_animation_auto_orient_rotates() {
        let json = r##"{
            "version": "1.0",
            "meta": { "name": "AutoOrient", "width": 64, "height": 64, "fps": 30, "duration": 30, "root": "main", "background": "#000000" },
            "compositions": {
                "main": {
                    "layers": [{
                        "id": "mover",
                        "in": 0, "out": 30,
                        "transform": { "position": [0, 0] },
                        "type": "solid",
                        "color": "#ff0000",
                        "path_animation": {
                            "points": [[0, 0], [64, 0], [64, 64]],
                            "auto_orient": true
                        }
                    }]
                }
            },
            "assets": { "fonts": [] }
        }"##;
        let scene = crate::parser::parse(json).expect("auto_orient scene should parse");
        let font_cache = std::collections::HashMap::new();

        // At frame 0, moving right along X-axis: rotation should be ~0 degrees
        let fs0 = evaluate_scene(&scene, 0, &font_cache).expect("frame 0 evaluates");
        assert!(
            fs0.layers[0].transform.rotation.abs() < 5.0,
            "frame 0: expected rotation near 0, got {}",
            fs0.layers[0].transform.rotation,
        );

        // At frame 20 (past midpoint), moving down along Y-axis: rotation should be ~90 degrees
        let fs20 = evaluate_scene(&scene, 20, &font_cache).expect("frame 20 evaluates");
        assert!(
            (fs20.layers[0].transform.rotation - 90.0).abs() < 5.0,
            "frame 20: expected rotation near 90, got {}",
            fs20.layers[0].transform.rotation,
        );
    }

    #[test]
    fn evaluate_path_position_edge_cases() {
        // Empty points
        let (x, y) = super::evaluate_path_position(&[], 0.5);
        assert_eq!(x, 0.0);
        assert_eq!(y, 0.0);

        // Single point
        let (x, y) = super::evaluate_path_position(&[[10.0, 20.0]], 0.5);
        assert_eq!(x, 10.0);
        assert_eq!(y, 20.0);

        // Two points, midpoint
        let (x, y) = super::evaluate_path_position(&[[0.0, 0.0], [100.0, 0.0]], 0.5);
        assert!((x - 50.0).abs() < 0.01);
        assert!(y.abs() < 0.01);

        // Three points, at t=0.5 should be at second point
        let (x, y) =
            super::evaluate_path_position(&[[0.0, 0.0], [50.0, 50.0], [100.0, 0.0]], 0.5);
        assert!((x - 50.0).abs() < 0.01);
        assert!((y - 50.0).abs() < 0.01);
    }

    #[test]
    fn ae_features_render_without_crash() {
        let json =
            std::fs::read_to_string("../../tests/fixtures/valid/ae_features.mmot.json").unwrap();
        let scene = crate::parser::parse(&json).unwrap();
        let font_cache = std::collections::HashMap::new();
        // Render 5 frames spread across the timeline
        for frame in [0, 10, 29, 30, 59] {
            let frame_scene = evaluate_scene(&scene, frame, &font_cache).unwrap();
            let rgba = crate::renderer::render(&frame_scene).unwrap();
            assert_eq!(
                rgba.len(),
                (640 * 360 * 4) as usize,
                "frame {frame} wrong size"
            );
            // Verify not all black (something rendered)
            let has_color = rgba
                .chunks(4)
                .any(|px| px[0] > 10 || px[1] > 10 || px[2] > 10);
            assert!(has_color, "frame {frame} is all black — nothing rendered");
        }
    }

    #[test]
    fn render_single_frame_returns_rgba() {
        let json = include_str!("../../../tests/fixtures/valid/minimal.mmot.json");
        let (w, h, rgba) = render_single_frame(json, 0).unwrap();
        assert!(w > 0);
        assert!(h > 0);
        assert_eq!(rgba.len(), (w * h * 4) as usize);
    }

    #[test]
    fn render_single_frame_different_frames_differ() {
        let json = r##"{
            "version": "1.0",
            "meta": { "name": "test", "width": 64, "height": 64, "fps": 30, "duration": 30, "root": "main", "background": "#000000" },
            "compositions": {
                "main": {
                    "layers": [{
                        "id": "fader",
                        "in": 0, "out": 30,
                        "transform": {
                            "position": [32, 32],
                            "opacity": [
                                { "t": 0, "v": 0.0 },
                                { "t": 29, "v": 1.0 }
                            ]
                        },
                        "type": "solid",
                        "color": "#ffffff",
                        "fill": "parent"
                    }]
                }
            },
            "assets": { "fonts": [] }
        }"##;
        let (_, _, rgba0) = render_single_frame(json, 0).unwrap();
        let (_, _, rgba20) = render_single_frame(json, 20).unwrap();
        assert_ne!(rgba0, rgba20, "different frames should produce different pixels");
    }

    #[test]
    fn get_scene_info_returns_metadata() {
        let json = include_str!("../../../tests/fixtures/valid/minimal.mmot.json");
        let info = get_scene_info(json).unwrap();
        assert!(info.width > 0);
        assert!(info.height > 0);
        assert!(info.fps > 0.0);
        assert!(info.duration_frames > 0);
        assert!(info.duration_secs > 0.0);
        assert!(info.composition_count > 0);
        assert!(info.root_layer_count > 0);
    }
}
