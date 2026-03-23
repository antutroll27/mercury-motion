use skia_safe::{images, Canvas, Data, Paint, Rect};

/// Draw an RGBA image onto the canvas.
pub fn draw(canvas: &Canvas, rgba: &[u8], width: u32, height: u32, base_paint: &Paint) {
    let info = skia_safe::ImageInfo::new(
        (width as i32, height as i32),
        skia_safe::ColorType::RGBA8888,
        skia_safe::AlphaType::Premul,
        None,
    );
    let row_bytes = (width * 4) as usize;
    let image = images::raster_from_data(&info, Data::new_copy(rgba), row_bytes)
        .expect("failed to create Skia image from RGBA");
    let dst = Rect::from_wh(width as f32, height as f32);
    canvas.draw_image_rect(&image, None, dst, base_paint);
}
