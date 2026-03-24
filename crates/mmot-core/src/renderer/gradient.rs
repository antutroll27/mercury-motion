use skia_safe::{Canvas, Color, Paint, Point, Rect, Shader, TileMode};

use crate::schema::GradientSpec;

/// Draw a gradient (linear or radial) filling the given dimensions.
pub fn draw(canvas: &Canvas, spec: &GradientSpec, width: u32, height: u32, base_paint: &Paint) {
    let mut paint = base_paint.clone();

    let shader = match spec {
        GradientSpec::Linear {
            start,
            end,
            colors,
        } => {
            let skia_colors: Vec<Color> = colors.iter().map(|s| parse_color(&s.color)).collect();
            let positions: Vec<f32> = colors.iter().map(|s| s.offset as f32).collect();

            let start_pt = Point::new(
                start[0] as f32 * width as f32,
                start[1] as f32 * height as f32,
            );
            let end_pt = Point::new(
                end[0] as f32 * width as f32,
                end[1] as f32 * height as f32,
            );

            Shader::linear_gradient(
                (start_pt, end_pt),
                skia_colors.as_slice(),
                positions.as_slice(),
                TileMode::Clamp,
                None,
                None,
            )
        }
        GradientSpec::Radial {
            center,
            radius,
            colors,
        } => {
            let skia_colors: Vec<Color> = colors.iter().map(|s| parse_color(&s.color)).collect();
            let positions: Vec<f32> = colors.iter().map(|s| s.offset as f32).collect();

            // Radius is specified as a fraction of the largest dimension
            let max_dim = width.max(height) as f32;
            let center_pt = Point::new(
                center[0] as f32 * width as f32,
                center[1] as f32 * height as f32,
            );
            let abs_radius = *radius as f32 * max_dim;

            Shader::radial_gradient(
                center_pt,
                abs_radius,
                skia_colors.as_slice(),
                positions.as_slice(),
                TileMode::Clamp,
                None,
                None,
            )
        }
    };

    if let Some(shader) = shader {
        paint.set_shader(shader);
        canvas.draw_rect(Rect::from_wh(width as f32, height as f32), &paint);
    }
}

fn parse_color(hex: &str) -> Color {
    let hex = hex.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
    Color::from_argb(255, r, g, b)
}
