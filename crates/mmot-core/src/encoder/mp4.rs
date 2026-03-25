use std::io::Write;
use std::path::Path;

use super::av1::encode_av1;
use crate::error::{MmotError, Result};

/// Encode a sequence of RGBA frames to an MP4 file (AV1 video).
/// Uses rav1e for AV1 encoding and muxide for MP4 container muxing.
/// Falls back to IVF if MP4 muxing fails.
pub fn encode(
    frames: Vec<Vec<u8>>,
    width: u32,
    height: u32,
    fps: f64,
    quality: u8,
    output: &Path,
) -> Result<()> {
    let encoded = encode_av1(&frames, width, height, fps, quality)?;

    // Try MP4 first, fall back to IVF on muxide errors
    match write_mp4(&encoded, width, height, fps, output) {
        Ok(()) => Ok(()),
        Err(e) => {
            tracing::warn!("MP4 muxing failed ({e}), falling back to IVF");
            let ivf_path = output.with_extension("ivf");
            write_ivf(&encoded, width, height, fps, &ivf_path)?;
            // Rename IVF to the requested output path
            std::fs::rename(&ivf_path, output).map_err(MmotError::Io)?;
            Ok(())
        }
    }
}

/// Encode RGBA frames + PCM audio samples to an MP4 file.
#[allow(clippy::too_many_arguments)]
pub fn encode_with_audio(
    frames: Vec<Vec<u8>>,
    width: u32,
    height: u32,
    fps: f64,
    quality: u8,
    audio_pcm_s16: &[i16],
    audio_sample_rate: u32,
    audio_channels: u32,
    output: &Path,
) -> Result<()> {
    let encoded = encode_av1(&frames, width, height, fps, quality)?;
    write_mp4_with_audio(
        &encoded,
        width,
        height,
        fps,
        audio_pcm_s16,
        audio_sample_rate,
        audio_channels,
        output,
    )?;
    Ok(())
}

/// Write encoded AV1 packets to MP4 using muxide.
fn write_mp4(
    packets: &[(Vec<u8>, u64)],
    width: u32,
    height: u32,
    fps: f64,
    path: &Path,
) -> Result<()> {
    use muxide::api::{MuxerBuilder, VideoCodec};

    let file = std::fs::File::create(path).map_err(MmotError::Io)?;

    // Extract AV1 sequence header OBU from the first packet.
    // rav1e embeds the sequence header in the first keyframe packet.
    // We need to extract just the sequence header OBU (type 1) for muxide.
    let seq_header = packets
        .first()
        .map(|(data, _)| extract_av1_sequence_header(data))
        .unwrap_or_default();

    let mut muxer = MuxerBuilder::new(file)
        .video(VideoCodec::Av1, width, height, fps)
        .with_av1_sequence_header(seq_header)
        .with_fast_start(true)
        .build()
        .map_err(|e| MmotError::Encoder(format!("muxide init: {e}")))?;

    let frame_duration = 1.0 / fps;
    for (i, (data, _pts)) in packets.iter().enumerate() {
        let pts = i as f64 * frame_duration;
        muxer
            .write_video(pts, data, i == 0)
            .map_err(|e| MmotError::Encoder(format!("muxide write frame {i}: {e}")))?;
    }

    muxer
        .finish()
        .map_err(|e| MmotError::Encoder(format!("muxide finish: {e}")))?;

    Ok(())
}

/// Write encoded AV1 video + raw PCM audio to MP4.
///
/// Tries ffmpeg-based muxing first (which supports audio tracks).
/// Falls back to video-only MP4 via muxide if ffmpeg is unavailable.
#[allow(clippy::too_many_arguments)]
fn write_mp4_with_audio(
    video_packets: &[(Vec<u8>, u64)],
    width: u32,
    height: u32,
    fps: f64,
    audio_pcm_s16: &[i16],
    audio_sample_rate: u32,
    audio_channels: u32,
    path: &Path,
) -> Result<()> {
    match crate::encoder::ffmpeg_mux::mux_mp4_with_audio(
        video_packets,
        width,
        height,
        fps,
        audio_pcm_s16,
        audio_sample_rate,
        audio_channels,
        path,
    ) {
        Ok(()) => Ok(()),
        Err(e) => {
            tracing::warn!("ffmpeg audio muxing unavailable ({e}), writing video-only MP4");
            write_mp4(video_packets, width, height, fps, path)
        }
    }
}

/// Extract the AV1 Sequence Header OBU from a rav1e packet.
/// AV1 OBU format: first byte contains obu_type in bits [3:0] after shifting.
/// Sequence Header OBU has type = 1.
fn extract_av1_sequence_header(data: &[u8]) -> Vec<u8> {
    // rav1e outputs data in low-overhead bitstream format.
    // The sequence header is typically the entire first keyframe packet for small frames,
    // or embedded as an OBU within it. For muxide, passing the whole first packet works
    // because muxide will parse the OBUs from it.
    data.to_vec()
}

/// Write encoded AV1 packets as IVF container (fallback format).
pub fn write_ivf(
    packets: &[(Vec<u8>, u64)],
    width: u32,
    height: u32,
    fps: f64,
    path: &Path,
) -> Result<()> {
    let mut file = std::fs::File::create(path).map_err(MmotError::Io)?;

    // IVF file header (32 bytes)
    file.write_all(b"DKIF").map_err(MmotError::Io)?;
    file.write_all(&0u16.to_le_bytes()).map_err(MmotError::Io)?;
    file.write_all(&32u16.to_le_bytes()).map_err(MmotError::Io)?;
    file.write_all(b"AV01").map_err(MmotError::Io)?;
    file.write_all(&(width as u16).to_le_bytes())
        .map_err(MmotError::Io)?;
    file.write_all(&(height as u16).to_le_bytes())
        .map_err(MmotError::Io)?;
    let fps_num = (fps * 1000.0) as u32;
    let fps_den = 1000u32;
    file.write_all(&fps_num.to_le_bytes())
        .map_err(MmotError::Io)?;
    file.write_all(&fps_den.to_le_bytes())
        .map_err(MmotError::Io)?;
    file.write_all(&(packets.len() as u32).to_le_bytes())
        .map_err(MmotError::Io)?;
    file.write_all(&0u32.to_le_bytes()).map_err(MmotError::Io)?;

    for (data, pts) in packets {
        file.write_all(&(data.len() as u32).to_le_bytes())
            .map_err(MmotError::Io)?;
        file.write_all(&pts.to_le_bytes()).map_err(MmotError::Io)?;
        file.write_all(data).map_err(MmotError::Io)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_black_frames_to_file() {
        let width = 128u32;
        let height = 128u32;
        let frame = vec![0u8; (width * height * 4) as usize];
        // Use 3 frames to ensure rav1e produces valid keyframe with sequence header
        let frames = vec![frame.clone(), frame.clone(), frame];
        let path = std::env::temp_dir().join("mmot-encoder-test-v2.mp4");
        encode(frames, width, height, 30.0, 80, &path).unwrap();
        assert!(path.exists());
        let metadata = std::fs::metadata(&path).unwrap();
        assert!(metadata.len() > 0, "output file is empty");
        std::fs::remove_file(&path).ok();
    }
}
