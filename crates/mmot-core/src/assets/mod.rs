pub mod audio;
pub mod font;
pub mod image;

/// Decoded image asset: raw RGBA bytes + dimensions.
pub struct DecodedImage {
    pub rgba: Vec<u8>,
    pub width: u32,
    pub height: u32,
}
