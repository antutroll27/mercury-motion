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
use crate::schema::{LayerContent, Scene, ShapeSpec};

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

    // Render all frames in parallel, collect in order
    let scene = Arc::new(scene);
    let frames: Vec<Result<Vec<u8>>> = (start..start + total)
        .into_par_iter()
        .map(|frame_num| {
            let frame_scene = evaluate_scene(&scene, frame_num)?;
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

    // Encode to MP4
    crate::encoder::mp4::encode(
        frames,
        scene.meta.width,
        scene.meta.height,
        scene.meta.fps,
        opts.quality,
        &opts.output_path,
    )?;

    Ok(())
}

/// Evaluate a scene at a specific frame number into a FrameScene.
/// Supports recursive precomp rendering.
pub fn evaluate_scene(scene: &Scene, frame: u64) -> Result<FrameScene> {
    let layers = evaluate_composition(scene, &scene.meta.root, frame, 0)?;
    Ok(FrameScene {
        width: scene.meta.width,
        height: scene.meta.height,
        background: scene.meta.background.clone(),
        layers,
    })
}

/// Recursively evaluate a composition, resolving precomp references.
fn evaluate_composition(
    scene: &Scene,
    comp_id: &str,
    frame: u64,
    depth: u32,
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

    let mut resolved_layers = Vec::new();
    for layer in &comp.layers {
        if frame < layer.in_point || frame >= layer.out_point {
            continue;
        }

        let position = evaluate_vec2(&layer.transform.position, frame);
        let scale = evaluate_vec2(&layer.transform.scale, frame);
        let opacity = evaluate_f64(&layer.transform.opacity, frame);
        let rotation = evaluate_f64(&layer.transform.rotation, frame);

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
            LayerContent::Text { text, font, align } => ResolvedContent::Text {
                text: text.clone(),
                font_family: font.family.clone(),
                font_size: font.size,
                font_weight: font.weight,
                color: font.color.clone(),
                align: align.clone(),
            },
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
                };
                ResolvedContent::Shape { shape: resolved }
            }
            LayerContent::Composition { id } => {
                // Recursively render the referenced composition
                let sub_layers = evaluate_composition(scene, id, frame, depth + 1)?;
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
            LayerContent::Video { .. } => {
                tracing::warn!("video layer '{}' not yet implemented — skipping", layer.id);
                continue;
            }
            LayerContent::Lottie { .. } => {
                tracing::warn!(
                    "lottie layer '{}' not yet implemented — skipping",
                    layer.id
                );
                continue;
            }
        };

        resolved_layers.push(ResolvedLayer {
            opacity,
            transform,
            content,
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
        let frame_scene = evaluate_scene(&scene, 0).unwrap();
        // The precomp should have resolved the sub composition's solid layer
        assert_eq!(frame_scene.layers.len(), 1);
        assert!(matches!(
            frame_scene.layers[0].content,
            ResolvedContent::Solid { .. }
        ));
    }
}
