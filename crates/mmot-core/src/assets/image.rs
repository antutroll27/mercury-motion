use crate::error::{MmotError, Result};

use super::DecodedImage;

/// Decode PNG/JPEG/WebP bytes to RGBA.
pub fn decode(data: &[u8]) -> Result<DecodedImage> {
    let img = image::load_from_memory(data)
        .map_err(|e| MmotError::AssetLoad(e.to_string()))?
        .into_rgba8();
    let (width, height) = img.dimensions();
    Ok(DecodedImage {
        rgba: img.into_raw(),
        width,
        height,
    })
}
