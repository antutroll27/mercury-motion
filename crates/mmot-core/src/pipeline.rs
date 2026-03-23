use std::path::PathBuf;
use std::sync::Arc;

use rayon::prelude::*;

use crate::error::{MmotError, Result};
use crate::evaluator::interpolate::{evaluate_f64, evaluate_vec2};
use crate::parser::parse;
use crate::renderer::{
    render as render_frame, FrameScene, ResolvedContent, ResolvedLayer, ResolvedTransform,
};
use crate::schema::{LayerContent, Scene};

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

/// Main entry point: parse JSON, render all frames, encode.
pub fn render_scene(
    json: &str,
    opts: RenderOptions,
    progress: Option<ProgressFn>,
) -> Result<()> {
    let scene = parse(json)?;
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
            .ok(); // Ignore error if already set
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

    // Check for errors
    let frames: Vec<Vec<u8>> = frames.into_iter().collect::<Result<_>>()?;

    // Encode
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
pub fn evaluate_scene(scene: &Scene, frame: u64) -> Result<FrameScene> {
    let comp = scene.compositions.get(&scene.meta.root).ok_or_else(|| {
        MmotError::Parse {
            message: format!("root composition '{}' not found", scene.meta.root),
            pointer: "/meta/root".into(),
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
            _ => continue, // Other layer types added in Phase 2
        };

        resolved_layers.push(ResolvedLayer {
            opacity,
            transform: ResolvedTransform {
                position,
                scale,
                rotation,
                opacity,
            },
            content,
        });
    }

    Ok(FrameScene {
        width: scene.meta.width,
        height: scene.meta.height,
        background: scene.meta.background.clone(),
        layers: resolved_layers,
    })
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
}
