use std::path::Path;

use crate::error::{MmotError, Result};

/// A single decoded video frame as raw RGBA pixels.
#[derive(Debug)]
pub struct DecodedVideoFrame {
    pub rgba: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

/// Decode a single video frame at the given timestamp (in seconds).
///
/// Requires the `ffmpeg` feature flag. Without it, this always returns an error.
#[cfg(feature = "ffmpeg")]
pub fn decode_frame(path: &Path, timestamp_secs: f64) -> Result<DecodedVideoFrame> {
    use ffmpeg_next as ffmpeg;

    ffmpeg::init().map_err(|e| MmotError::VideoDecode(format!("ffmpeg init failed: {e}")))?;

    let mut ictx = ffmpeg::format::input(path)
        .map_err(|e| MmotError::VideoDecode(format!("cannot open '{}': {e}", path.display())))?;

    let stream = ictx
        .streams()
        .best(ffmpeg::media::Type::Video)
        .ok_or_else(|| {
            MmotError::VideoDecode(format!("no video stream in '{}'", path.display()))
        })?;

    let stream_index = stream.index();
    let time_base = stream.time_base();

    let mut decoder = ffmpeg::codec::context::Context::from_parameters(stream.parameters())
        .map_err(|e| MmotError::VideoDecode(format!("decoder init failed: {e}")))?
        .decoder()
        .video()
        .map_err(|e| MmotError::VideoDecode(format!("video decoder failed: {e}")))?;

    // Convert target timestamp to stream time base units
    let target_ts = if time_base.1 > 0 {
        (timestamp_secs * time_base.1 as f64 / time_base.0 as f64) as i64
    } else {
        0
    };

    // Seek to the target timestamp
    ictx.seek(target_ts, ..target_ts)
        .map_err(|e| MmotError::VideoDecode(format!("seek failed: {e}")))?;

    let width = decoder.width();
    let height = decoder.height();
    let src_format = decoder.format();

    let mut scaler = ffmpeg::software::scaling::Context::get(
        src_format,
        width,
        height,
        ffmpeg::format::Pixel::RGBA,
        width,
        height,
        ffmpeg::software::scaling::Flags::BILINEAR,
    )
    .map_err(|e| MmotError::VideoDecode(format!("scaler init failed: {e}")))?;

    let mut rgba_frame = ffmpeg::frame::Video::empty();

    // Read packets and decode until we get a frame at or after the target PTS
    for (stream_ref, packet) in ictx.packets() {
        if stream_ref.index() != stream_index {
            continue;
        }

        decoder
            .send_packet(&packet)
            .map_err(|e| MmotError::VideoDecode(format!("send_packet failed: {e}")))?;

        let mut decoded = ffmpeg::frame::Video::empty();
        while decoder.receive_frame(&mut decoded).is_ok() {
            // Convert to RGBA
            scaler
                .run(&decoded, &mut rgba_frame)
                .map_err(|e| MmotError::VideoDecode(format!("scaling failed: {e}")))?;

            // Copy RGBA data accounting for stride/padding
            let stride = rgba_frame.stride(0);
            let data = rgba_frame.data(0);
            let mut rgba = Vec::with_capacity((width * height * 4) as usize);
            for row in 0..height as usize {
                let row_start = row * stride;
                let row_end = row_start + (width as usize * 4);
                rgba.extend_from_slice(&data[row_start..row_end]);
            }

            return Ok(DecodedVideoFrame {
                rgba,
                width,
                height,
            });
        }
    }

    Err(MmotError::VideoDecode(format!(
        "no frame decoded from '{}' at {timestamp_secs:.3}s",
        path.display()
    )))
}

/// Stub when the `ffmpeg` feature is not enabled.
#[cfg(not(feature = "ffmpeg"))]
pub fn decode_frame(_path: &Path, _timestamp_secs: f64) -> Result<DecodedVideoFrame> {
    Err(MmotError::VideoDecode(
        "requires ffmpeg feature flag".into(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_without_ffmpeg_returns_error() {
        // Without the ffmpeg feature, decode_frame should always fail gracefully.
        #[cfg(not(feature = "ffmpeg"))]
        {
            let result = decode_frame(Path::new("nonexistent.mp4"), 0.0);
            assert!(result.is_err());
            let msg = result.unwrap_err().to_string();
            assert!(
                msg.contains("requires ffmpeg feature flag"),
                "unexpected error: {msg}"
            );
        }
    }
}
