use skia_safe::Canvas;

use crate::renderer::{ResolvedContent, ResolvedLayer};

use super::{blend, gradient, image as img_renderer, shape, solid, text};

/// Draw a single resolved layer onto the canvas.
pub fn draw_layer(canvas: &Canvas, layer: &ResolvedLayer, width: u32, height: u32) {
    canvas.save();
    let mut paint = if layer.fill_parent {
        // AbsoluteFill: skip transform, render at (0,0) filling the full canvas.
        let mut paint = skia_safe::Paint::default();
        paint.set_alpha_f(layer.opacity as f32);
        paint
    } else {
        apply_transform(canvas, layer)
    };
    if let Some(ref mode) = layer.blend_mode {
        paint.set_blend_mode(blend::to_skia_blend_mode(mode));
    }
    match &layer.content {
        ResolvedContent::Solid { color } => solid::draw(canvas, color, width, height, &paint),
        ResolvedContent::Image {
            data,
            width: iw,
            height: ih,
        } => img_renderer::draw(canvas, data, *iw, *ih, &paint),
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
            );
        }
        ResolvedContent::Shape { shape: s } => {
            shape::draw(canvas, s, &paint);
        }
        ResolvedContent::Gradient {
            gradient: g,
            width,
            height,
        } => {
            gradient::draw(canvas, g, *width, *height, &paint);
        }
    }
    canvas.restore();
}

fn apply_transform(canvas: &Canvas, layer: &ResolvedLayer) -> skia_safe::Paint {
    let t = &layer.transform;
    let mut m = skia_safe::Matrix::new_identity();
    m.pre_translate((t.position.x as f32, t.position.y as f32));
    m.pre_rotate(t.rotation as f32, None);
    m.pre_scale((t.scale.x as f32, t.scale.y as f32), None);
    m.pre_translate((-t.position.x as f32, -t.position.y as f32));
    canvas.concat(&m);
    let mut paint = skia_safe::Paint::default();
    paint.set_alpha_f(layer.opacity as f32);
    paint
}
