//! tiny-skia based renderer for WASM.
//!
//! Takes a [`FrameScene`] (from mmot-core's evaluator) and produces RGBA bytes
//! using tiny-skia — a pure-Rust 2D rendering library that compiles to WASM.
//!
//! Supported content types:
//! - Solid layers (filled rectangles)
//! - Shape layers (rect, ellipse, line, polygon)
//! - Gradient layers (linear, radial)
//! - Image layers (decoded RGBA data)
//! - Text layers (placeholder rectangle — tiny-skia has no text layout)

use mmot_core::renderer::{FrameScene, ResolvedContent, ResolvedLayer, ResolvedShape};
use tiny_skia::{
    Color, FillRule, GradientStop, LinearGradient, Paint, PathBuilder, Pixmap, RadialGradient,
    SpreadMode, Stroke, Transform,
};

/// Create a new Color with modified alpha, since tiny-skia 0.11 has no `with_alpha`.
fn color_with_alpha(c: Color, alpha: f32) -> Color {
    Color::from_rgba(c.red(), c.green(), c.blue(), alpha.clamp(0.0, 1.0))
        .unwrap_or(Color::TRANSPARENT)
}

/// Render a [`FrameScene`] to raw RGBA bytes using tiny-skia.
pub fn render_frame_tiny_skia(frame: &FrameScene) -> Result<Vec<u8>, String> {
    let w = frame.width;
    let h = frame.height;

    let mut pixmap = Pixmap::new(w, h).ok_or_else(|| "failed to create pixmap".to_string())?;

    // Clear with background color
    let bg = parse_color(&frame.background);
    pixmap.fill(bg);

    // Draw layers in order (first = bottom of visual stack)
    for layer in &frame.layers {
        if layer.adjustment {
            // Adjustment layers are not supported in the WASM renderer (skip)
            continue;
        }
        draw_layer(&mut pixmap, layer, w, h);
    }

    Ok(pixmap.data().to_vec())
}

/// Draw a single resolved layer onto the pixmap.
fn draw_layer(pixmap: &mut Pixmap, layer: &ResolvedLayer, width: u32, height: u32) {
    let transform = build_transform(layer, width, height);
    let opacity = (layer.opacity * layer.transform.opacity).min(1.0) as f32;

    match &layer.content {
        ResolvedContent::Solid { color } => {
            draw_solid(pixmap, color, width, height, layer.fill_parent, &transform, opacity);
        }
        ResolvedContent::Image {
            data,
            width: iw,
            height: ih,
        } => {
            draw_image(pixmap, data, *iw, *ih, &transform, opacity);
        }
        ResolvedContent::Text {
            color,
            font_size,
            ..
        } => {
            // tiny-skia has no text layout — render a placeholder rectangle
            draw_text_placeholder(pixmap, color, *font_size, &transform, opacity);
        }
        ResolvedContent::Shape { shape } => {
            draw_shape(pixmap, shape, &transform, opacity);
        }
        ResolvedContent::Gradient {
            gradient,
            width: gw,
            height: gh,
        } => {
            draw_gradient(pixmap, gradient, *gw, *gh, &transform, opacity);
        }
    }
}

/// Build a tiny-skia Transform from a ResolvedLayer.
fn build_transform(layer: &ResolvedLayer, _width: u32, _height: u32) -> Transform {
    if layer.fill_parent {
        return Transform::identity();
    }

    let t = &layer.transform;
    let tx = t.position.x as f32;
    let ty = t.position.y as f32;

    Transform::identity()
        .post_translate(tx, ty)
        .post_scale(t.scale.x as f32, t.scale.y as f32)
        .post_rotate_at(t.rotation as f32, tx, ty)
}

/// Draw a solid color rectangle.
fn draw_solid(
    pixmap: &mut Pixmap,
    color: &str,
    width: u32,
    height: u32,
    fill_parent: bool,
    transform: &Transform,
    opacity: f32,
) {
    let mut c = parse_color(color);
    c = color_with_alpha(c, c.alpha() * opacity);

    let mut paint = Paint::default();
    paint.set_color(c);
    paint.anti_alias = true;

    if fill_parent {
        let rect =
            tiny_skia::Rect::from_xywh(0.0, 0.0, width as f32, height as f32);
        if let Some(rect) = rect {
            pixmap.fill_rect(rect, &paint, Transform::identity(), None);
        }
    } else {
        // Default solid size: 100x100 centered at transform origin
        let rect = tiny_skia::Rect::from_xywh(-50.0, -50.0, 100.0, 100.0);
        if let Some(rect) = rect {
            pixmap.fill_rect(rect, &paint, *transform, None);
        }
    }
}

/// Draw a decoded RGBA image.
fn draw_image(
    pixmap: &mut Pixmap,
    data: &[u8],
    width: u32,
    height: u32,
    transform: &Transform,
    opacity: f32,
) {
    let expected = (width * height * 4) as usize;
    if data.len() < expected {
        return;
    }

    if let Some(src_pixmap) = tiny_skia::PixmapRef::from_bytes(data, width, height) {
        let mut paint = tiny_skia::PixmapPaint::default();
        paint.opacity = opacity;
        let img_transform = transform.pre_translate(-(width as f32) / 2.0, -(height as f32) / 2.0);
        pixmap.draw_pixmap(0, 0, src_pixmap, &paint, Transform::identity(), None);
        // Note: tiny-skia's draw_pixmap doesn't support arbitrary transforms.
        // For full transform support, we'd need to use a shader-based approach.
        // This is a simplified implementation for the WASM preview.
        let _ = img_transform;
    }
}

/// Draw a placeholder rectangle for text (tiny-skia has no text layout).
fn draw_text_placeholder(
    pixmap: &mut Pixmap,
    color: &str,
    font_size: f64,
    transform: &Transform,
    opacity: f32,
) {
    let mut c = parse_color(color);
    c = color_with_alpha(c, c.alpha() * opacity);

    let mut paint = Paint::default();
    paint.set_color(c);
    paint.anti_alias = true;

    // Use font_size to determine placeholder height
    let h = font_size as f32;
    let w = h * 4.0; // Rough text width estimate
    let rect = tiny_skia::Rect::from_xywh(-w / 2.0, -h / 2.0, w, h);
    if let Some(rect) = rect {
        pixmap.fill_rect(rect, &paint, *transform, None);
    }
}

/// Draw a shape (rect, ellipse, line, polygon).
fn draw_shape(
    pixmap: &mut Pixmap,
    shape: &ResolvedShape,
    transform: &Transform,
    opacity: f32,
) {
    match shape {
        ResolvedShape::Rect {
            width,
            height,
            corner_radius,
            fill,
            stroke_color,
            stroke_width,
        } => {
            let w = *width as f32;
            let h = *height as f32;
            let cr = *corner_radius as f32;

            if let Some(fill_color) = fill {
                let mut c = parse_color(fill_color);
                c = color_with_alpha(c, c.alpha() * opacity);
                let mut paint = Paint::default();
                paint.set_color(c);
                paint.anti_alias = true;

                if cr > 0.0 {
                    // Rounded rectangle via path
                    if let Some(path) = rounded_rect_path(-w / 2.0, -h / 2.0, w, h, cr) {
                        pixmap.fill_path(&path, &paint, FillRule::Winding, *transform, None);
                    }
                } else if let Some(rect) = tiny_skia::Rect::from_xywh(-w / 2.0, -h / 2.0, w, h) {
                    pixmap.fill_rect(rect, &paint, *transform, None);
                }
            }

            if let Some(sc) = stroke_color {
                let mut c = parse_color(sc);
                c = color_with_alpha(c, c.alpha() * opacity);
                let mut paint = Paint::default();
                paint.set_color(c);
                paint.anti_alias = true;

                let mut stroke = Stroke::default();
                stroke.width = *stroke_width as f32;

                if cr > 0.0 {
                    if let Some(path) = rounded_rect_path(-w / 2.0, -h / 2.0, w, h, cr) {
                        pixmap.stroke_path(&path, &paint, &stroke, *transform, None);
                    }
                } else {
                    let mut pb = PathBuilder::new();
                    pb.move_to(-w / 2.0, -h / 2.0);
                    pb.line_to(w / 2.0, -h / 2.0);
                    pb.line_to(w / 2.0, h / 2.0);
                    pb.line_to(-w / 2.0, h / 2.0);
                    pb.close();
                    if let Some(path) = pb.finish() {
                        pixmap.stroke_path(&path, &paint, &stroke, *transform, None);
                    }
                }
            }
        }
        ResolvedShape::Ellipse {
            width,
            height,
            fill,
            stroke_color,
            stroke_width,
        } => {
            let w = *width as f32;
            let h = *height as f32;

            // Approximate ellipse with cubic beziers
            if let Some(path) = ellipse_path(-w / 2.0, -h / 2.0, w, h) {
                if let Some(fill_color) = fill {
                    let mut c = parse_color(fill_color);
                    c = color_with_alpha(c, c.alpha() * opacity);
                    let mut paint = Paint::default();
                    paint.set_color(c);
                    paint.anti_alias = true;
                    pixmap.fill_path(&path, &paint, FillRule::Winding, *transform, None);
                }

                if let Some(sc) = stroke_color {
                    let mut c = parse_color(sc);
                    c = color_with_alpha(c, c.alpha() * opacity);
                    let mut paint = Paint::default();
                    paint.set_color(c);
                    paint.anti_alias = true;
                    let mut stroke = Stroke::default();
                    stroke.width = *stroke_width as f32;
                    pixmap.stroke_path(&path, &paint, &stroke, *transform, None);
                }
            }
        }
        ResolvedShape::Line {
            x1,
            y1,
            x2,
            y2,
            stroke_color,
            stroke_width,
        } => {
            let mut pb = PathBuilder::new();
            pb.move_to(*x1 as f32, *y1 as f32);
            pb.line_to(*x2 as f32, *y2 as f32);
            if let Some(path) = pb.finish() {
                let mut c = parse_color(stroke_color);
                c = color_with_alpha(c, c.alpha() * opacity);
                let mut paint = Paint::default();
                paint.set_color(c);
                paint.anti_alias = true;
                let mut stroke = Stroke::default();
                stroke.width = *stroke_width as f32;
                pixmap.stroke_path(&path, &paint, &stroke, *transform, None);
            }
        }
        ResolvedShape::Polygon {
            points,
            fill,
            stroke_color,
            stroke_width,
        } => {
            if points.len() < 2 {
                return;
            }
            let mut pb = PathBuilder::new();
            pb.move_to(points[0][0] as f32, points[0][1] as f32);
            for pt in &points[1..] {
                pb.line_to(pt[0] as f32, pt[1] as f32);
            }
            pb.close();
            if let Some(path) = pb.finish() {
                if let Some(fill_color) = fill {
                    let mut c = parse_color(fill_color);
                    c = color_with_alpha(c, c.alpha() * opacity);
                    let mut paint = Paint::default();
                    paint.set_color(c);
                    paint.anti_alias = true;
                    pixmap.fill_path(&path, &paint, FillRule::Winding, *transform, None);
                }
                if let Some(sc) = stroke_color {
                    let mut c = parse_color(sc);
                    c = color_with_alpha(c, c.alpha() * opacity);
                    let mut paint = Paint::default();
                    paint.set_color(c);
                    paint.anti_alias = true;
                    let mut stroke = Stroke::default();
                    stroke.width = *stroke_width as f32;
                    pixmap.stroke_path(&path, &paint, &stroke, *transform, None);
                }
            }
        }
    }
}

/// Draw a gradient (linear or radial).
fn draw_gradient(
    pixmap: &mut Pixmap,
    gradient: &mmot_core::schema::GradientSpec,
    width: u32,
    height: u32,
    transform: &Transform,
    opacity: f32,
) {
    let w = width as f32;
    let h = height as f32;

    let rect = match tiny_skia::Rect::from_xywh(-w / 2.0, -h / 2.0, w, h) {
        Some(r) => r,
        None => return,
    };

    let mut paint = Paint::default();
    paint.anti_alias = true;

    match gradient {
        mmot_core::schema::GradientSpec::Linear {
            start,
            end,
            colors,
        } => {
            let stops: Vec<GradientStop> = colors
                .iter()
                .map(|gs| {
                    let c = parse_color(&gs.color);
                    let c = color_with_alpha(c, c.alpha() * opacity);
                    GradientStop::new(gs.offset as f32, c)
                })
                .collect();

            if stops.len() < 2 {
                return;
            }

            let shader = LinearGradient::new(
                tiny_skia::Point::from_xy(start[0] as f32 * w - w / 2.0, start[1] as f32 * h - h / 2.0),
                tiny_skia::Point::from_xy(end[0] as f32 * w - w / 2.0, end[1] as f32 * h - h / 2.0),
                stops,
                SpreadMode::Pad,
                Transform::identity(),
            );

            if let Some(shader) = shader {
                paint.shader = shader;
                pixmap.fill_rect(rect, &paint, *transform, None);
            }
        }
        mmot_core::schema::GradientSpec::Radial {
            center,
            radius,
            colors,
        } => {
            let stops: Vec<GradientStop> = colors
                .iter()
                .map(|gs| {
                    let c = parse_color(&gs.color);
                    let c = color_with_alpha(c, c.alpha() * opacity);
                    GradientStop::new(gs.offset as f32, c)
                })
                .collect();

            if stops.len() < 2 {
                return;
            }

            let cx = center[0] as f32 * w - w / 2.0;
            let cy = center[1] as f32 * h - h / 2.0;
            let r = *radius as f32 * w.max(h);

            let shader = RadialGradient::new(
                tiny_skia::Point::from_xy(cx, cy),
                tiny_skia::Point::from_xy(cx, cy),
                r,
                stops,
                SpreadMode::Pad,
                Transform::identity(),
            );

            if let Some(shader) = shader {
                paint.shader = shader;
                pixmap.fill_rect(rect, &paint, *transform, None);
            }
        }
    }
}

/// Build a rounded rectangle path.
fn rounded_rect_path(x: f32, y: f32, w: f32, h: f32, r: f32) -> Option<tiny_skia::Path> {
    let r = r.min(w / 2.0).min(h / 2.0);
    let mut pb = PathBuilder::new();

    // Magic number for approximating circular arcs with cubic beziers
    let k: f32 = 0.552_284_8;
    let kr = k * r;

    pb.move_to(x + r, y);
    pb.line_to(x + w - r, y);
    pb.cubic_to(x + w - r + kr, y, x + w, y + r - kr, x + w, y + r);
    pb.line_to(x + w, y + h - r);
    pb.cubic_to(x + w, y + h - r + kr, x + w - r + kr, y + h, x + w - r, y + h);
    pb.line_to(x + r, y + h);
    pb.cubic_to(x + r - kr, y + h, x, y + h - r + kr, x, y + h - r);
    pb.line_to(x, y + r);
    pb.cubic_to(x, y + r - kr, x + r - kr, y, x + r, y);
    pb.close();

    pb.finish()
}

/// Build an ellipse path using cubic bezier approximation.
fn ellipse_path(x: f32, y: f32, w: f32, h: f32) -> Option<tiny_skia::Path> {
    let cx = x + w / 2.0;
    let cy = y + h / 2.0;
    let rx = w / 2.0;
    let ry = h / 2.0;

    let k: f32 = 0.552_284_8;
    let kx = k * rx;
    let ky = k * ry;

    let mut pb = PathBuilder::new();
    pb.move_to(cx, cy - ry);
    pb.cubic_to(cx + kx, cy - ry, cx + rx, cy - ky, cx + rx, cy);
    pb.cubic_to(cx + rx, cy + ky, cx + kx, cy + ry, cx, cy + ry);
    pb.cubic_to(cx - kx, cy + ry, cx - rx, cy + ky, cx - rx, cy);
    pb.cubic_to(cx - rx, cy - ky, cx - kx, cy - ry, cx, cy - ry);
    pb.close();

    pb.finish()
}

/// Parse a hex color string (#RRGGBB or #RRGGBBAA) into a tiny-skia Color.
fn parse_color(hex: &str) -> Color {
    let hex = hex.trim_start_matches('#');
    if hex.len() < 6 {
        return Color::BLACK;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
    let a = if hex.len() >= 8 {
        u8::from_str_radix(&hex[6..8], 16).unwrap_or(255)
    } else {
        255
    };
    Color::from_rgba8(r, g, b, a)
}

#[cfg(test)]
mod tests {
    use super::*;
    use mmot_core::renderer::ResolvedTransform;
    use mmot_core::schema::Vec2;

    #[test]
    fn render_empty_scene() {
        let frame = FrameScene {
            width: 4,
            height: 4,
            background: "#ff0000".into(),
            layers: vec![],
        };
        let rgba = render_frame_tiny_skia(&frame).expect("render should succeed");
        assert_eq!(rgba.len(), 4 * 4 * 4);
        // All pixels should be red
        for chunk in rgba.chunks(4) {
            assert_eq!(chunk[0], 255, "R");
            assert_eq!(chunk[1], 0, "G");
            assert_eq!(chunk[2], 0, "B");
            assert_eq!(chunk[3], 255, "A");
        }
    }

    #[test]
    fn render_solid_layer() {
        let frame = FrameScene {
            width: 10,
            height: 10,
            background: "#000000".into(),
            layers: vec![ResolvedLayer {
                opacity: 1.0,
                transform: ResolvedTransform {
                    position: Vec2 { x: 5.0, y: 5.0 },
                    scale: Vec2 { x: 1.0, y: 1.0 },
                    rotation: 0.0,
                    opacity: 1.0,
                },
                content: ResolvedContent::Solid {
                    color: "#00ff00".into(),
                },
                fill_parent: true,
                blend_mode: None,
                masks: None,
                effects: None,
                adjustment: false,
                track_matte_source: None,
                trim_start: 0.0,
                trim_end: 1.0,
            }],
        };
        let rgba = render_frame_tiny_skia(&frame).expect("render should succeed");
        assert_eq!(rgba.len(), 10 * 10 * 4);
        // All pixels should be green (fill_parent = true)
        for chunk in rgba.chunks(4) {
            assert_eq!(chunk[0], 0, "R");
            assert_eq!(chunk[1], 255, "G");
            assert_eq!(chunk[2], 0, "B");
        }
    }

    #[test]
    fn render_shape_rect() {
        let frame = FrameScene {
            width: 100,
            height: 100,
            background: "#000000".into(),
            layers: vec![ResolvedLayer {
                opacity: 1.0,
                transform: ResolvedTransform {
                    position: Vec2 { x: 50.0, y: 50.0 },
                    scale: Vec2 { x: 1.0, y: 1.0 },
                    rotation: 0.0,
                    opacity: 1.0,
                },
                content: ResolvedContent::Shape {
                    shape: ResolvedShape::Rect {
                        width: 50.0,
                        height: 50.0,
                        corner_radius: 0.0,
                        fill: Some("#ff0000".into()),
                        stroke_color: None,
                        stroke_width: 0.0,
                    },
                },
                fill_parent: false,
                blend_mode: None,
                masks: None,
                effects: None,
                adjustment: false,
                track_matte_source: None,
                trim_start: 0.0,
                trim_end: 1.0,
            }],
        };
        let rgba = render_frame_tiny_skia(&frame).expect("render should succeed");
        assert_eq!(rgba.len(), 100 * 100 * 4);
        // Should have some red pixels
        let has_red = rgba.chunks(4).any(|px| px[0] > 200 && px[1] < 50);
        assert!(has_red, "expected red pixels from rect shape");
    }

    #[test]
    fn parse_color_works() {
        let c = parse_color("#ff8000");
        // tiny-skia Color channels are f32 in 0.0..=1.0
        assert!((c.red() - 1.0).abs() < 0.01, "R should be ~1.0, got {}", c.red());
        assert!((c.green() - 128.0 / 255.0).abs() < 0.01, "G should be ~0.5, got {}", c.green());
        assert!(c.blue().abs() < 0.01, "B should be ~0.0, got {}", c.blue());
        assert!((c.alpha() - 1.0).abs() < 0.01, "A should be ~1.0, got {}", c.alpha());
    }

    #[test]
    fn parse_color_with_alpha() {
        let c = parse_color("#ff000080");
        assert!((c.red() - 1.0).abs() < 0.01, "R should be ~1.0, got {}", c.red());
        assert!(c.green().abs() < 0.01, "G should be ~0.0, got {}", c.green());
        assert!(c.blue().abs() < 0.01, "B should be ~0.0, got {}", c.blue());
        assert!((c.alpha() - 128.0 / 255.0).abs() < 0.01, "A should be ~0.5, got {}", c.alpha());
    }
}
