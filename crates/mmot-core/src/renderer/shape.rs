use skia_safe::{Canvas, Color, Paint, PaintStyle, Path, PathMeasure, Rect, RRect};

use super::ResolvedShape;

/// Returns true if trimming is the default (no-op) range.
fn is_default_trim(start: f64, end: f64) -> bool {
    (start - 0.0).abs() < f64::EPSILON && (end - 1.0).abs() < f64::EPSILON
}

/// Trim a path using PathMeasure, returning a new path that represents
/// only the portion from `start` to `end` (both 0.0–1.0 normalised).
/// Returns `None` if trimming produces an empty result or if start >= end.
fn apply_trim(path: &Path, start: f64, end: f64) -> Option<Path> {
    if start >= end {
        return None;
    }
    let mut measure = PathMeasure::new(path, false, None);
    let total = measure.length() as f64;
    if total <= 0.0 {
        return None;
    }
    let start_dist = (start * total) as f32;
    let end_dist = (end * total) as f32;
    measure.segment(start_dist, end_dist, true)
}

/// Draw a shape onto the canvas centered at the current transform origin.
/// `trim_start` and `trim_end` control trim paths (0.0–1.0, default 0.0 and 1.0).
pub fn draw(
    canvas: &Canvas,
    shape: &ResolvedShape,
    base_paint: &Paint,
    trim_start: f64,
    trim_end: f64,
) {
    let trimming = !is_default_trim(trim_start, trim_end);

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

            if trimming {
                // Convert to a path for trimming
                let mut path = Path::new();
                if *corner_radius > 0.0 {
                    let rrect = RRect::new_rect_xy(
                        rect,
                        *corner_radius as f32,
                        *corner_radius as f32,
                    );
                    path.add_rrect(rrect, None);
                } else {
                    path.add_rect(rect, None);
                }
                if let Some(trimmed) = apply_trim(&path, trim_start, trim_end) {
                    draw_path_with_styles(canvas, &trimmed, base_paint, fill, stroke_color, *stroke_width);
                }
            } else {
                if let Some(fill_color) = fill {
                    let mut paint = base_paint.clone();
                    paint.set_style(PaintStyle::Fill);
                    apply_color(&mut paint, fill_color);
                    if *corner_radius > 0.0 {
                        let rrect = RRect::new_rect_xy(
                            rect,
                            *corner_radius as f32,
                            *corner_radius as f32,
                        );
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
                        let rrect = RRect::new_rect_xy(
                            rect,
                            *corner_radius as f32,
                            *corner_radius as f32,
                        );
                        canvas.draw_rrect(rrect, &paint);
                    } else {
                        canvas.draw_rect(rect, &paint);
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
            let rect = Rect::from_xywh(
                -(*width as f32) / 2.0,
                -(*height as f32) / 2.0,
                *width as f32,
                *height as f32,
            );

            if trimming {
                let mut path = Path::new();
                path.add_oval(rect, None);
                if let Some(trimmed) = apply_trim(&path, trim_start, trim_end) {
                    draw_path_with_styles(canvas, &trimmed, base_paint, fill, stroke_color, *stroke_width);
                }
            } else {
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
        ResolvedShape::Line {
            x1,
            y1,
            x2,
            y2,
            stroke_color,
            stroke_width,
        } => {
            if trimming {
                let mut path = Path::new();
                path.move_to(skia_safe::Point::new(*x1 as f32, *y1 as f32));
                path.line_to(skia_safe::Point::new(*x2 as f32, *y2 as f32));
                if let Some(trimmed) = apply_trim(&path, trim_start, trim_end) {
                    let mut paint = base_paint.clone();
                    paint.set_style(PaintStyle::Stroke);
                    paint.set_stroke_width(*stroke_width as f32);
                    paint.set_anti_alias(true);
                    apply_color(&mut paint, stroke_color);
                    canvas.draw_path(&trimmed, &paint);
                }
            } else {
                let mut paint = base_paint.clone();
                paint.set_style(PaintStyle::Stroke);
                paint.set_stroke_width(*stroke_width as f32);
                paint.set_anti_alias(true);
                apply_color(&mut paint, stroke_color);
                canvas.draw_line(
                    skia_safe::Point::new(*x1 as f32, *y1 as f32),
                    skia_safe::Point::new(*x2 as f32, *y2 as f32),
                    &paint,
                );
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
            let mut path = Path::new();
            path.move_to(skia_safe::Point::new(
                points[0][0] as f32,
                points[0][1] as f32,
            ));
            for pt in &points[1..] {
                path.line_to(skia_safe::Point::new(pt[0] as f32, pt[1] as f32));
            }
            path.close();

            if trimming {
                if let Some(trimmed) = apply_trim(&path, trim_start, trim_end) {
                    draw_path_with_styles(canvas, &trimmed, base_paint, fill, stroke_color, *stroke_width);
                }
            } else {
                if let Some(fill_color) = fill {
                    let mut paint = base_paint.clone();
                    paint.set_style(PaintStyle::Fill);
                    apply_color(&mut paint, fill_color);
                    canvas.draw_path(&path, &paint);
                }
                if let Some(sc) = stroke_color {
                    let mut paint = base_paint.clone();
                    paint.set_style(PaintStyle::Stroke);
                    paint.set_stroke_width(*stroke_width as f32);
                    apply_color(&mut paint, sc);
                    canvas.draw_path(&path, &paint);
                }
            }
        }
    }
}

/// Draw a trimmed path with optional fill and stroke styles.
fn draw_path_with_styles(
    canvas: &Canvas,
    path: &Path,
    base_paint: &Paint,
    fill: &Option<String>,
    stroke_color: &Option<String>,
    stroke_width: f64,
) {
    if let Some(fill_color) = fill {
        let mut paint = base_paint.clone();
        paint.set_style(PaintStyle::Fill);
        apply_color(&mut paint, fill_color);
        canvas.draw_path(path, &paint);
    }
    if let Some(sc) = stroke_color {
        let mut paint = base_paint.clone();
        paint.set_style(PaintStyle::Stroke);
        paint.set_stroke_width(stroke_width as f32);
        apply_color(&mut paint, sc);
        canvas.draw_path(path, &paint);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_measure_works_for_oval() {
        let rect = Rect::from_xywh(-25.0, -25.0, 50.0, 50.0);
        let mut path = Path::new();
        path.add_oval(rect, None);
        let mut measure = PathMeasure::new(&path, false, None);
        let length = measure.length();
        assert!(length > 0.0, "oval path length should be > 0, got {length}");

        // Get a segment (first half)
        let seg = measure.segment(0.0, length * 0.5, true);
        assert!(seg.is_some(), "segment should succeed for first half of oval");
    }

    #[test]
    fn apply_trim_returns_trimmed_path() {
        let rect = Rect::from_xywh(-25.0, -25.0, 50.0, 50.0);
        let mut path = Path::new();
        path.add_oval(rect, None);
        let trimmed = apply_trim(&path, 0.0, 0.5);
        assert!(trimmed.is_some(), "apply_trim should return Some for 0.0..0.5");

        // Verify via PathMeasure that the trimmed path is shorter
        let mut full_m = PathMeasure::new(&path, false, None);
        let full_len = full_m.length();
        let mut trim_m = PathMeasure::new(trimmed.as_ref().unwrap(), false, None);
        let trim_len = trim_m.length();
        assert!(
            trim_len < full_len,
            "trimmed path ({trim_len}) should be shorter than full ({full_len})"
        );
    }
}

fn apply_color(paint: &mut Paint, hex: &str) {
    let hex = hex.trim_start_matches('#');
    if hex.len() < 6 {
        return; // malformed color
    }
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
