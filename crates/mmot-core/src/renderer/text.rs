use skia_safe::{Canvas, Color, Font, FontMgr, FontStyle, Paint, TextBlob};

use crate::schema::TextAlign;

/// Draw text onto the canvas at the given position.
#[allow(clippy::too_many_arguments)]
pub fn draw(
    canvas: &Canvas,
    text: &str,
    x: f32,
    y: f32,
    font_family: &str,
    font_size: f64,
    font_weight: u32,
    color: &str,
    align: &TextAlign,
) {
    let font_mgr = FontMgr::new();
    let style = FontStyle::new(
        skia_weight(font_weight),
        skia_safe::font_style::Width::NORMAL,
        skia_safe::font_style::Slant::Upright,
    );
    let typeface = font_mgr
        .match_family_style(font_family, style)
        .or_else(|| font_mgr.match_family_style("sans-serif", style))
        .unwrap_or_else(|| font_mgr.legacy_make_typeface(None, style).unwrap());

    let font = Font::new(typeface, font_size as f32);
    let (hex_r, hex_g, hex_b) = parse_hex_color(color);

    let mut paint = Paint::default();
    paint.set_color(Color::from_rgb(hex_r, hex_g, hex_b));
    paint.set_anti_alias(true);

    let blob = TextBlob::new(text, &font).unwrap_or_else(|| TextBlob::new("?", &font).unwrap());

    let bounds = blob.bounds();
    let draw_x = match align {
        TextAlign::Left => x,
        TextAlign::Center => x - bounds.width() / 2.0,
        TextAlign::Right => x - bounds.width(),
    };
    let draw_y = y + bounds.height() / 2.0;

    canvas.draw_text_blob(&blob, (draw_x, draw_y), &paint);
}

fn skia_weight(weight: u32) -> skia_safe::font_style::Weight {
    skia_safe::font_style::Weight::from(weight as i32)
}

fn parse_hex_color(hex: &str) -> (u8, u8, u8) {
    let hex = hex.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(255);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(255);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(255);
    (r, g, b)
}
