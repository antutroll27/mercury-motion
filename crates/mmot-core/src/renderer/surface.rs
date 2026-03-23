use skia_safe::{surfaces, AlphaType, ColorType, ImageInfo, Surface};

/// Create a CPU-backed Skia raster surface.
pub fn create_cpu_surface(width: u32, height: u32) -> Surface {
    let info = ImageInfo::new(
        (width as i32, height as i32),
        ColorType::RGBA8888,
        AlphaType::Premul,
        None,
    );
    surfaces::raster(&info, None, None).expect("failed to create Skia CPU surface")
}
