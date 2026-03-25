use skia_safe::{Canvas, Color, Paint, Rect};

/// Draw a solid color rectangle filling the given dimensions.
pub fn draw(canvas: &Canvas, color: &str, width: u32, height: u32, base_paint: &Paint) {
    let hex = color.trim_start_matches('#');
    if hex.len() < 6 {
        return; // malformed color — skip
    }
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
    let mut paint = base_paint.clone();
    paint.set_color(Color::from_argb(
        (base_paint.alpha_f() * 255.0) as u8,
        r,
        g,
        b,
    ));
    canvas.draw_rect(Rect::from_wh(width as f32, height as f32), &paint);
}
