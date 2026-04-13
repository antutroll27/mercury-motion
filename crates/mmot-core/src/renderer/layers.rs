use image::{GrayImage, Luma};
use skia_safe::{images, AlphaType, Canvas, ColorType, Data, ImageInfo, Paint, Path};

use crate::error::{MmotError, Result};
use crate::renderer::{ResolvedContent, ResolvedLayer};
use crate::schema::effects::{Mask, MaskMode, MaskPath};

use super::{blend, effects, gradient, image as img_renderer, masks, shape, solid, surface, text};

/// Draw a single resolved layer onto the canvas.
pub fn draw_layer(canvas: &Canvas, layer: &ResolvedLayer, width: u32, height: u32) -> Result<()> {
    let image = render_layer_image(layer, width, height)?;
    let mut paint = Paint::default();
    paint.set_alpha_f(layer.opacity as f32);
    if let Some(ref mode) = layer.blend_mode {
        paint.set_blend_mode(blend::to_skia_blend_mode(mode));
    }
    if let Some(ref effects_list) = layer.effects
        && !effects_list.is_empty()
        && let Some(filter) = effects::build_image_filter(effects_list)
    {
        paint.set_image_filter(filter);
    }
    canvas.draw_image(&image, (0, 0), Some(&paint));
    Ok(())
}

fn render_layer_image(layer: &ResolvedLayer, width: u32, height: u32) -> Result<skia_safe::Image> {
    let mut surface = surface::create_cpu_surface(width, height)?;
    surface.canvas().clear(skia_safe::Color::TRANSPARENT);
    render_layer_content(surface.canvas(), layer, width, height)?;

    if let Some(ref layer_masks) = layer.masks
        && !layer_masks.is_empty()
    {
        let rgba = crate::renderer::read_surface_rgba(&mut surface, width, height);
        let masked = apply_masks_to_rgba(&rgba, layer, layer_masks, width, height)?;
        return image_from_rgba(&masked, width, height);
    }

    Ok(surface.image_snapshot())
}

fn render_layer_content(canvas: &Canvas, layer: &ResolvedLayer, width: u32, height: u32) -> Result<()> {
    canvas.save();
    let base_paint = if layer.fill_parent {
        Paint::default()
    } else {
        apply_transform(canvas, layer)
    };

    match &layer.content {
        ResolvedContent::Solid { color } => solid::draw(canvas, color, width, height, &base_paint),
        ResolvedContent::Image {
            data,
            width: iw,
            height: ih,
        } => img_renderer::draw(canvas, data, *iw, *ih, &base_paint),
        ResolvedContent::Text {
            text: t,
            font_family,
            font_size,
            font_weight,
            color,
            align,
            custom_font_data,
        } => {
            let t_ref = &layer.transform;
            text::draw(
                canvas,
                t,
                t_ref.position.x as f32,
                t_ref.position.y as f32,
                font_family,
                *font_size,
                *font_weight,
                color,
                align,
                custom_font_data.as_deref(),
                &base_paint,
            );
        }
        ResolvedContent::Shape { shape: s } => {
            shape::draw(canvas, s, &base_paint, layer.trim_start, layer.trim_end);
        }
        ResolvedContent::Gradient {
            gradient: g,
            width,
            height,
        } => {
            gradient::draw(canvas, g, *width, *height, &base_paint);
        }
        ResolvedContent::Composition {
            layers,
            width,
            height,
        } => {
            let image = crate::renderer::render_layers_to_image(layers, *width, *height)?;
            canvas.draw_image(&image, (0, 0), Some(&base_paint));
        }
    }
    canvas.restore();
    Ok(())
}

pub(crate) fn image_from_rgba(rgba: &[u8], width: u32, height: u32) -> Result<skia_safe::Image> {
    let info = ImageInfo::new(
        (width as i32, height as i32),
        ColorType::RGBA8888,
        AlphaType::Premul,
        None,
    );
    images::raster_from_data(&info, Data::new_copy(rgba), (width * 4) as usize).ok_or_else(|| {
        MmotError::RenderFailed {
            frame: 0,
            reason: format!("failed to create Skia image from RGBA ({width}x{height})"),
        }
    })
}

fn apply_masks_to_rgba(
    rgba: &[u8],
    layer: &ResolvedLayer,
    masks: &[Mask],
    width: u32,
    height: u32,
) -> Result<Vec<u8>> {
    let alpha = render_combined_mask_alpha(layer, masks, width, height)?;
    let mut out = rgba.to_vec();

    for (pixel, factor) in out.chunks_exact_mut(4).zip(alpha.iter().copied()) {
        let factor = factor.clamp(0.0, 1.0);
        for channel in pixel.iter_mut() {
            *channel = ((*channel as f32) * factor).round().clamp(0.0, 255.0) as u8;
        }
    }

    Ok(out)
}

fn render_combined_mask_alpha(
    layer: &ResolvedLayer,
    masks: &[Mask],
    width: u32,
    height: u32,
) -> Result<Vec<f32>> {
    let mut combined: Option<Vec<f32>> = None;

    for mask in masks {
        let mut surface = surface::create_cpu_surface(width, height)?;
        surface.canvas().clear(skia_safe::Color::TRANSPARENT);
        surface.canvas().save();

        if !layer.fill_parent {
            let _ = apply_transform(surface.canvas(), layer);
        }

        draw_mask_path(surface.canvas(), mask);
        surface.canvas().restore();

        let mut alpha =
            rgba_to_alpha(&crate::renderer::read_surface_rgba(&mut surface, width, height));
        if mask.feather > 0.0 {
            alpha = blur_alpha(&alpha, width, height, mask.feather as f32);
        }
        if mask.inverted {
            for value in &mut alpha {
                *value = 1.0 - *value;
            }
        }

        combined = Some(match combined {
            None => match mask.mode {
                MaskMode::Add | MaskMode::Intersect => alpha,
                MaskMode::Subtract | MaskMode::Difference => {
                    alpha.into_iter().map(|value| 1.0 - value).collect()
                }
            },
            Some(existing) => existing
                .into_iter()
                .zip(alpha.into_iter())
                .map(|(lhs, rhs)| match mask.mode {
                    MaskMode::Add => lhs.max(rhs),
                    MaskMode::Subtract => lhs * (1.0 - rhs),
                    MaskMode::Intersect => lhs * rhs,
                    MaskMode::Difference => lhs * (1.0 - rhs) + (1.0 - lhs) * rhs,
                })
                .collect(),
        });
    }

    Ok(combined.unwrap_or_else(|| vec![1.0; (width * height) as usize]))
}

fn draw_mask_path(canvas: &Canvas, mask: &Mask) {
    let path = mask_path_to_skia(&mask.path);
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_color(skia_safe::Color::from_argb(
        (mask.opacity.clamp(0.0, 1.0) * 255.0) as u8,
        255,
        255,
        255,
    ));
    canvas.draw_path(&path, &paint);
}

fn mask_path_to_skia(mask_path: &MaskPath) -> Path {
    masks::mask_path_to_skia(mask_path)
}

fn rgba_to_alpha(rgba: &[u8]) -> Vec<f32> {
    rgba.chunks_exact(4).map(|pixel| pixel[3] as f32 / 255.0).collect()
}

fn blur_alpha(alpha: &[f32], width: u32, height: u32, sigma: f32) -> Vec<f32> {
    let mut gray = GrayImage::new(width, height);
    for (idx, pixel) in gray.pixels_mut().enumerate() {
        *pixel = Luma([((alpha[idx].clamp(0.0, 1.0) * 255.0).round()) as u8]);
    }
    let blurred = image::imageops::blur(&gray, sigma.max(0.1));
    blurred
        .pixels()
        .map(|pixel| pixel.0[0] as f32 / 255.0)
        .collect()
}

fn apply_transform(canvas: &Canvas, layer: &ResolvedLayer) -> skia_safe::Paint {
    let t = &layer.transform;
    let mut m = skia_safe::Matrix::new_identity();
    m.pre_translate((t.position.x as f32, t.position.y as f32));
    m.pre_rotate(t.rotation as f32, None);
    m.pre_scale((t.scale.x as f32, t.scale.y as f32), None);
    m.pre_translate((-t.position.x as f32, -t.position.y as f32));
    canvas.concat(&m);
    skia_safe::Paint::default()
}
