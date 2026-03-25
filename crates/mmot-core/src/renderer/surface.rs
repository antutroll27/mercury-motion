use skia_safe::{surfaces, AlphaType, ColorType, ImageInfo, Surface};

use crate::error::{MmotError, Result};

/// Create a CPU-backed Skia raster surface.
pub fn create_cpu_surface(width: u32, height: u32) -> Result<Surface> {
    let info = ImageInfo::new(
        (width as i32, height as i32),
        ColorType::RGBA8888,
        AlphaType::Premul,
        None,
    );
    surfaces::raster(&info, None, None).ok_or_else(|| MmotError::RenderFailed {
        frame: 0,
        reason: format!("failed to create Skia CPU surface ({width}x{height})"),
    })
}
