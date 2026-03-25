use std::path::Path;

use crate::error::{MmotError, Result};

/// Encode RGBA frames to an animated GIF.
pub fn encode(
    frames: Vec<Vec<u8>>,
    width: u32,
    height: u32,
    fps: f64,
    output: &Path,
) -> Result<()> {
    use gif::{Encoder, Frame, Repeat};

    let file = std::fs::File::create(output).map_err(MmotError::Io)?;
    let mut encoder = Encoder::new(file, width as u16, height as u16, &[])
        .map_err(|e| MmotError::Encoder(format!("gif init: {e}")))?;

    encoder
        .set_repeat(Repeat::Infinite)
        .map_err(|e| MmotError::Encoder(format!("gif repeat: {e}")))?;

    let delay = (100.0 / fps).round() as u16; // GIF delay is in 1/100th seconds

    for rgba in &frames {
        let mut rgba_copy = rgba.clone();
        let mut frame = Frame::from_rgba_speed(width as u16, height as u16, &mut rgba_copy, 10);
        frame.delay = delay;
        encoder
            .write_frame(&frame)
            .map_err(|e| MmotError::Encoder(format!("gif write frame: {e}")))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_solid_frames_to_gif() {
        let width = 16u32;
        let height = 16u32;
        let red: Vec<u8> = (0..width * height)
            .flat_map(|_| [255, 0, 0, 255])
            .collect();
        let blue: Vec<u8> = (0..width * height)
            .flat_map(|_| [0, 0, 255, 255])
            .collect();
        let path = std::env::temp_dir().join("mmot-test.gif");
        encode(vec![red, blue], width, height, 10.0, &path).unwrap();
        assert!(path.exists());
        let data = std::fs::read(&path).unwrap();
        assert_eq!(&data[..6], b"GIF89a");
        std::fs::remove_file(&path).ok();
    }
}
