use skia_safe::{Canvas, Color, Paint, PaintStyle, Rect, RRect};

/// Resolved shape data ready for rendering.
pub enum ResolvedShape {
    Rect {
        width: f64,
        height: f64,
        corner_radius: f64,
        fill: Option<String>,
        stroke_color: Option<String>,
        stroke_width: f64,
    },
    Ellipse {
        width: f64,
        height: f64,
        fill: Option<String>,
        stroke_color: Option<String>,
        stroke_width: f64,
    },
}

/// Draw a shape onto the canvas centered at the current transform origin.
pub fn draw(canvas: &Canvas, shape: &ResolvedShape, base_paint: &Paint) {
    match shape {
        ResolvedShape::Rect {
            width,
            height,
            corner_radius,
            fill,
            stroke_color,
            stroke_width,
        } => {
            let rect = Rect::from_xywh(
                -(*width as f32) / 2.0,
                -(*height as f32) / 2.0,
                *width as f32,
                *height as f32,
            );

            if let Some(fill_color) = fill {
                let mut paint = base_paint.clone();
                paint.set_style(PaintStyle::Fill);
                apply_color(&mut paint, fill_color);
                if *corner_radius > 0.0 {
                    let rrect = RRect::new_rect_xy(rect, *corner_radius as f32, *corner_radius as f32);
                    canvas.draw_rrect(rrect, &paint);
                } else {
                    canvas.draw_rect(rect, &paint);
                }
            }

            if let Some(sc) = stroke_color {
                let mut paint = base_paint.clone();
                paint.set_style(PaintStyle::Stroke);
                paint.set_stroke_width(*stroke_width as f32);
                apply_color(&mut paint, sc);
                if *corner_radius > 0.0 {
                    let rrect = RRect::new_rect_xy(rect, *corner_radius as f32, *corner_radius as f32);
                    canvas.draw_rrect(rrect, &paint);
                } else {
                    canvas.draw_rect(rect, &paint);
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
            let rect = Rect::from_xywh(
                -(*width as f32) / 2.0,
                -(*height as f32) / 2.0,
                *width as f32,
                *height as f32,
            );

            if let Some(fill_color) = fill {
                let mut paint = base_paint.clone();
                paint.set_style(PaintStyle::Fill);
                apply_color(&mut paint, fill_color);
                canvas.draw_oval(rect, &paint);
            }

            if let Some(sc) = stroke_color {
                let mut paint = base_paint.clone();
                paint.set_style(PaintStyle::Stroke);
                paint.set_stroke_width(*stroke_width as f32);
                apply_color(&mut paint, sc);
                canvas.draw_oval(rect, &paint);
            }
        }
    }
}

fn apply_color(paint: &mut Paint, hex: &str) {
    let hex = hex.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
    let a = if hex.len() >= 8 {
        u8::from_str_radix(&hex[6..8], 16).unwrap_or(255)
    } else {
        255
    };
    paint.set_color(Color::from_argb(a, r, g, b));
}
