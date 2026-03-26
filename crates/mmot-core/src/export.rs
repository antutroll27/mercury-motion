use std::path::{Path, PathBuf};

use rayon::prelude::*;

use crate::error::{MmotError, Result};
use crate::schema::scene::SafeZone;

/// A named export profile defining target dimensions.
#[derive(Debug, Clone)]
pub struct ExportProfile {
    pub name: String,
    pub width: u32,
    pub height: u32,
}

/// Options for multi-format export.
pub struct ExportOptions {
    pub output_dir: PathBuf,
    pub profiles: Vec<ExportProfile>,
    pub quality: u8,
    pub concurrency: Option<usize>,
    pub format: crate::pipeline::OutputFormat,
}

/// Result of exporting a single profile.
pub struct ExportResult {
    pub profile_name: String,
    pub output_path: PathBuf,
    pub width: u32,
    pub height: u32,
}

/// Built-in export profiles for common social media platforms.
pub fn builtin_profiles() -> Vec<ExportProfile> {
    vec![
        ExportProfile { name: "youtube".into(), width: 1920, height: 1080 },
        ExportProfile { name: "instagram_post".into(), width: 1080, height: 1080 },
        ExportProfile { name: "instagram_story".into(), width: 1080, height: 1920 },
        ExportProfile { name: "tiktok".into(), width: 1080, height: 1920 },
        ExportProfile { name: "linkedin".into(), width: 1080, height: 1350 },
        ExportProfile { name: "twitter".into(), width: 1280, height: 720 },
    ]
}

/// A crop rectangle within the source canvas.
#[derive(Debug, Clone)]
pub struct CropRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// Compute the crop rectangle from the original canvas to fit the target aspect ratio,
/// keeping the safe zone centered and fully visible.
///
/// The crop rect will:
/// 1. Have the same aspect ratio as `dst_w`/`dst_h`
/// 2. Contain the entire safe zone
/// 3. Be centered on the safe zone center
/// 4. Be clamped to the source canvas bounds
pub fn compute_crop_rect(
    src_w: u32, src_h: u32,
    safe_zone: &SafeZone,
    dst_w: u32, dst_h: u32,
) -> Result<CropRect> {
    if dst_w == 0 || dst_h == 0 {
        return Err(MmotError::Encoder("export profile dimensions must be > 0".into()));
    }
    if src_w == 0 || src_h == 0 {
        return Err(MmotError::Encoder("source dimensions must be > 0".into()));
    }

    let safe_cx = safe_zone.x + safe_zone.width / 2.0;
    let safe_cy = safe_zone.y + safe_zone.height / 2.0;
    let src_wf = src_w as f64;
    let src_hf = src_h as f64;

    let target_aspect = dst_w as f64 / dst_h as f64;

    // Start with the safe zone dimensions and expand to match target aspect ratio.
    // We always expand (never shrink below the safe zone) so the safe zone is fully visible.
    let mut crop_w = safe_zone.width;
    let mut crop_h = safe_zone.height;

    let safe_aspect = crop_w / crop_h;
    if target_aspect > safe_aspect {
        // Target is wider -- expand width, keep height >= safe zone height
        crop_w = crop_h * target_aspect;
    } else {
        // Target is taller -- expand height, keep width >= safe zone width
        crop_h = crop_w / target_aspect;
    }

    // If the crop exceeds canvas bounds, scale down to fit while maintaining
    // the target aspect ratio. The safe zone may not be fully contained when
    // the canvas itself is too small for the requested aspect ratio.
    if crop_w > src_wf || crop_h > src_hf {
        // Find the largest crop at the target aspect ratio that fits the canvas
        let fit_by_w_h = src_wf / target_aspect;
        let fit_by_h_w = src_hf * target_aspect;

        if fit_by_w_h <= src_hf {
            // Fit by width
            crop_w = src_wf;
            crop_h = fit_by_w_h;
        } else {
            // Fit by height
            crop_w = fit_by_h_w.min(src_wf);
            crop_h = src_hf;
        }
    }

    // Center on safe zone center, clamp to canvas
    let mut x = safe_cx - crop_w / 2.0;
    let mut y = safe_cy - crop_h / 2.0;
    x = x.max(0.0).min((src_wf - crop_w).max(0.0));
    y = y.max(0.0).min((src_hf - crop_h).max(0.0));

    Ok(CropRect { x, y, width: crop_w, height: crop_h })
}

/// Crop and scale an RGBA frame buffer using nearest-neighbour sampling.
pub fn crop_and_scale_frame(
    rgba: &[u8],
    src_w: u32, src_h: u32,
    crop: &CropRect,
    dst_w: u32, dst_h: u32,
) -> Vec<u8> {
    let mut out = vec![0u8; (dst_w * dst_h * 4) as usize];
    for dy in 0..dst_h {
        for dx in 0..dst_w {
            // Map destination pixel to source pixel within the crop rect
            let sx = crop.x + (dx as f64 / dst_w as f64) * crop.width;
            let sy = crop.y + (dy as f64 / dst_h as f64) * crop.height;
            let sx = (sx as u32).min(src_w.saturating_sub(1));
            let sy = (sy as u32).min(src_h.saturating_sub(1));
            let src_idx = ((sy * src_w + sx) * 4) as usize;
            let dst_idx = ((dy * dst_w + dx) * 4) as usize;
            if src_idx + 3 < rgba.len() && dst_idx + 3 < out.len() {
                out[dst_idx..dst_idx + 4].copy_from_slice(&rgba[src_idx..src_idx + 4]);
            }
        }
    }
    out
}

/// Export a scene to multiple aspect ratios.
///
/// Renders all frames at original resolution, then crops and scales
/// for each profile, encoding to the requested format.
pub fn export_all(
    json: &str,
    opts: ExportOptions,
    progress: Option<crate::pipeline::ProgressFn>,
) -> Result<Vec<ExportResult>> {
    use std::collections::HashMap;

    use crate::parser::parse;
    use crate::pipeline::{evaluate_scene, OutputFormat};
    use crate::renderer::render as render_frame;

    let scene = parse(json)?;
    let font_cache: HashMap<String, Vec<u8>> = {
        let mut cache = HashMap::new();
        for font_asset in &scene.assets.fonts {
            match crate::assets::font::load_font(Path::new(&font_asset.src)) {
                Ok(data) => {
                    cache.insert(font_asset.id.clone(), data);
                }
                Err(e) => {
                    tracing::warn!("failed to load font '{}': {e}", font_asset.id);
                }
            }
        }
        cache
    };

    // Get safe zone (default to full canvas if not specified)
    let safe_zone = scene.meta.safe_zone.clone().unwrap_or(SafeZone {
        x: 0.0,
        y: 0.0,
        width: scene.meta.width as f64,
        height: scene.meta.height as f64,
    });

    let total_frames = scene.meta.duration;
    let src_w = scene.meta.width;
    let src_h = scene.meta.height;

    // Render all frames at original resolution
    let frame_results: Vec<Result<Vec<u8>>> = (0..total_frames)
        .into_par_iter()
        .map(|frame_num| {
            let fs = evaluate_scene(&scene, frame_num, &font_cache)?;
            render_frame(&fs)
        })
        .collect();

    // Propagate any render errors
    let frames: Vec<Vec<u8>> = frame_results.into_iter().collect::<Result<Vec<_>>>()?;

    // Create output directory
    std::fs::create_dir_all(&opts.output_dir).map_err(MmotError::Io)?;

    let mut results = Vec::new();

    for profile in &opts.profiles {
        let crop = compute_crop_rect(src_w, src_h, &safe_zone, profile.width, profile.height)?;

        // Crop and scale all frames for this profile
        let profile_frames: Vec<Vec<u8>> = frames.iter()
            .map(|f| crop_and_scale_frame(f, src_w, src_h, &crop, profile.width, profile.height))
            .collect();

        // Determine output path and format extension
        let ext = match opts.format {
            OutputFormat::Mp4 => "mp4",
            OutputFormat::Gif => "gif",
            OutputFormat::Webm => "webm",
        };
        let output_path = opts.output_dir.join(format!("{}.{}", profile.name, ext));

        // Encode using the appropriate encoder
        match opts.format {
            OutputFormat::Mp4 => {
                crate::encoder::mp4::encode(
                    profile_frames, profile.width, profile.height,
                    scene.meta.fps, opts.quality, &output_path,
                )?;
            }
            OutputFormat::Gif => {
                crate::encoder::gif::encode(
                    profile_frames, profile.width, profile.height,
                    scene.meta.fps, &output_path,
                )?;
            }
            OutputFormat::Webm => {
                #[cfg(feature = "ffmpeg")]
                {
                    let packets = crate::encoder::av1::encode_av1(
                        &profile_frames, profile.width, profile.height,
                        scene.meta.fps, opts.quality,
                    )?;
                    crate::encoder::ffmpeg_mux::mux_webm(
                        &packets, profile.width, profile.height,
                        scene.meta.fps, &output_path,
                    )?;
                }
                #[cfg(not(feature = "ffmpeg"))]
                {
                    return Err(MmotError::Encoder(
                        "WebM export requires --features ffmpeg".into(),
                    ));
                }
            }
        }

        if let Some(ref prog) = progress {
            prog(results.len() as u64 + 1, opts.profiles.len() as u64);
        }

        results.push(ExportResult {
            profile_name: profile.name.clone(),
            output_path,
            width: profile.width,
            height: profile.height,
        });
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crop_rect_wider_target() {
        // 1920x1080 canvas, 800x800 safe zone centered, export to 16:9
        let sz = SafeZone { x: 560.0, y: 140.0, width: 800.0, height: 800.0 };
        let crop = compute_crop_rect(1920, 1080, &sz, 1920, 1080).unwrap();
        let aspect = crop.width / crop.height;
        assert!((aspect - 16.0 / 9.0).abs() < 0.05, "expected ~16:9, got {aspect}");
        // Safe zone must be fully inside crop
        assert!(crop.x <= sz.x, "crop should start at or before safe zone x");
        assert!(crop.y <= sz.y, "crop should start at or before safe zone y");
        assert!(crop.x + crop.width >= sz.x + sz.width, "crop should end at or after safe zone right edge");
        assert!(crop.y + crop.height >= sz.y + sz.height, "crop should end at or after safe zone bottom edge");
    }

    #[test]
    fn crop_rect_taller_target() {
        // 1920x1080 canvas, 800x800 safe zone, export to 9:16 (TikTok)
        // Canvas height (1080) constrains the crop; safe zone cannot be fully
        // contained because 9:16 at full height only gives 607.5 width, less
        // than the 800-wide safe zone. The crop should be the largest 9:16
        // rectangle that fits the canvas, centered on the safe zone center.
        let sz = SafeZone { x: 560.0, y: 140.0, width: 800.0, height: 800.0 };
        let crop = compute_crop_rect(1920, 1080, &sz, 1080, 1920).unwrap();
        let aspect = crop.width / crop.height;
        assert!((aspect - 9.0 / 16.0).abs() < 0.05, "expected ~9:16, got {aspect}");
        // Crop should use full canvas height
        assert!((crop.height - 1080.0).abs() < 1.0, "should use full canvas height");
        // Crop should be within canvas bounds
        assert!(crop.x >= 0.0);
        assert!(crop.y >= 0.0);
        assert!(crop.x + crop.width <= 1920.0 + 0.01);
        assert!(crop.y + crop.height <= 1080.0 + 0.01);
    }

    #[test]
    fn crop_rect_taller_target_fits() {
        // 1080x1920 canvas (portrait), 400x400 safe zone, export to 9:16
        // Canvas is tall enough; safe zone should be fully contained.
        let sz = SafeZone { x: 340.0, y: 760.0, width: 400.0, height: 400.0 };
        let crop = compute_crop_rect(1080, 1920, &sz, 1080, 1920).unwrap();
        let aspect = crop.width / crop.height;
        assert!((aspect - 9.0 / 16.0).abs() < 0.05, "expected ~9:16, got {aspect}");
        // Safe zone must be fully inside crop
        assert!(crop.x <= sz.x, "crop should start at or before safe zone x");
        assert!(crop.y <= sz.y, "crop should start at or before safe zone y");
        assert!(crop.x + crop.width >= sz.x + sz.width, "crop should cover safe zone width");
        assert!(crop.y + crop.height >= sz.y + sz.height, "crop should cover safe zone height");
    }

    #[test]
    fn crop_rect_square_target() {
        // 1920x1080 canvas, 800x800 safe zone, export to 1:1 (Instagram)
        let sz = SafeZone { x: 560.0, y: 140.0, width: 800.0, height: 800.0 };
        let crop = compute_crop_rect(1920, 1080, &sz, 1080, 1080).unwrap();
        let aspect = crop.width / crop.height;
        assert!((aspect - 1.0).abs() < 0.05, "expected ~1:1, got {aspect}");
        // For square target with square safe zone, crop should match safe zone dimensions
        assert!((crop.width - 800.0).abs() < 1.0);
        assert!((crop.height - 800.0).abs() < 1.0);
    }

    #[test]
    fn crop_rect_clamped_to_canvas() {
        // Small canvas, large safe zone relative to canvas
        let sz = SafeZone { x: 10.0, y: 10.0, width: 80.0, height: 80.0 };
        let crop = compute_crop_rect(100, 100, &sz, 1920, 1080).unwrap();
        // Crop must not exceed canvas bounds
        assert!(crop.x >= 0.0, "crop x must be >= 0");
        assert!(crop.y >= 0.0, "crop y must be >= 0");
        assert!(crop.x + crop.width <= 100.0 + 0.01, "crop must not exceed canvas width");
        assert!(crop.y + crop.height <= 100.0 + 0.01, "crop must not exceed canvas height");
    }

    #[test]
    fn crop_and_scale_identity() {
        // 2x2 RGBA image with distinct colors per pixel
        let rgba: Vec<u8> = vec![
            255, 0, 0, 255,   // top-left: red
            0, 255, 0, 255,   // top-right: green
            0, 0, 255, 255,   // bottom-left: blue
            255, 255, 0, 255, // bottom-right: yellow
        ];
        // Identity crop (full source) scaled to same size
        let crop = CropRect { x: 0.0, y: 0.0, width: 2.0, height: 2.0 };
        let result = crop_and_scale_frame(&rgba, 2, 2, &crop, 2, 2);
        assert_eq!(result.len(), 16);
        // Top-left should still be red
        assert_eq!(result[0], 255);
        assert_eq!(result[1], 0);
        assert_eq!(result[2], 0);
    }

    #[test]
    fn crop_and_scale_upscale() {
        // 2x2 all-red image, crop full, scale to 4x4
        let rgba: Vec<u8> = vec![
            255, 0, 0, 255,
            255, 0, 0, 255,
            255, 0, 0, 255,
            255, 0, 0, 255,
        ];
        let crop = CropRect { x: 0.0, y: 0.0, width: 2.0, height: 2.0 };
        let result = crop_and_scale_frame(&rgba, 2, 2, &crop, 4, 4);
        assert_eq!(result.len(), 4 * 4 * 4);
        // All pixels should be red
        for chunk in result.chunks(4) {
            assert_eq!(chunk[0], 255, "R channel");
            assert_eq!(chunk[1], 0, "G channel");
            assert_eq!(chunk[2], 0, "B channel");
        }
    }

    #[test]
    fn crop_subregion() {
        // 4x4 image: top-left quadrant is red, rest is black
        let mut rgba = vec![0u8; 4 * 4 * 4];
        // Set top-left 2x2 to red
        for row in 0..2u32 {
            for col in 0..2u32 {
                let idx = ((row * 4 + col) * 4) as usize;
                rgba[idx] = 255;
                rgba[idx + 3] = 255;
            }
        }
        // Crop the top-left 2x2 region, scale to 2x2
        let crop = CropRect { x: 0.0, y: 0.0, width: 2.0, height: 2.0 };
        let result = crop_and_scale_frame(&rgba, 4, 4, &crop, 2, 2);
        assert_eq!(result.len(), 2 * 2 * 4);
        // All output pixels should be red
        for chunk in result.chunks(4) {
            assert_eq!(chunk[0], 255, "cropped region should be red");
        }
    }

    #[test]
    fn builtin_profiles_has_expected_count() {
        let profiles = builtin_profiles();
        assert_eq!(profiles.len(), 6);
        // Check that youtube profile exists
        assert!(profiles.iter().any(|p| p.name == "youtube"));
        assert!(profiles.iter().any(|p| p.name == "tiktok"));
        assert!(profiles.iter().any(|p| p.name == "instagram_post"));
    }
}
