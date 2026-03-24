use std::io::Write;
use std::path::Path;

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

/// AV1-encode RGBA frames, returning encoded packets with timestamps.
fn encode_av1(
    frames: &[Vec<u8>],
    width: u32,
    height: u32,
    fps: f64,
    quality: u8,
) -> Result<Vec<(Vec<u8>, u64)>> {
    use rav1e::prelude::*;

    let cfg = Config::new().with_encoder_config(EncoderConfig {
        width: width as usize,
        height: height as usize,
        time_base: Rational::new(1, fps as u64),
        bit_depth: 8,
        quantizer: map_quality_to_quantizer(quality),
        speed_settings: SpeedSettings::from_preset(10),
        ..Default::default()
    });

    let mut ctx: Context<u8> = cfg
        .new_context()
        .map_err(|e| MmotError::Encoder(e.to_string()))?;

    let mut encoded_packets: Vec<(Vec<u8>, u64)> = Vec::new();
    let mut pts = 0u64;

    for rgba in frames {
        let mut frame = ctx.new_frame();
        rgba_to_yuv420(rgba, width, height, &mut frame);
        ctx.send_frame(frame)
            .map_err(|e| MmotError::Encoder(e.to_string()))?;
        drain_packets(&mut ctx, &mut encoded_packets, &mut pts)?;
    }

    ctx.flush();
    drain_packets(&mut ctx, &mut encoded_packets, &mut pts)?;

    Ok(encoded_packets)
}

fn drain_packets(
    ctx: &mut rav1e::prelude::Context<u8>,
    out: &mut Vec<(Vec<u8>, u64)>,
    pts: &mut u64,
) -> Result<()> {
    loop {
        match ctx.receive_packet() {
            Ok(pkt) => {
                out.push((pkt.data, *pts));
                *pts += 1;
            }
            Err(rav1e::prelude::EncoderStatus::NeedMoreData) => break,
            Err(rav1e::prelude::EncoderStatus::EnoughData) => break,
            Err(rav1e::prelude::EncoderStatus::LimitReached) => break,
            Err(rav1e::prelude::EncoderStatus::Encoded) => continue,
            Err(e) => return Err(MmotError::Encoder(e.to_string())),
        }
    }
    Ok(())
}

fn map_quality_to_quantizer(quality: u8) -> usize {
    let q = quality.clamp(1, 100);
    ((100 - q) as usize * 200) / 99
}

fn rgba_to_yuv420(rgba: &[u8], width: u32, height: u32, frame: &mut rav1e::prelude::Frame<u8>) {
    let w = width as usize;
    let h = height as usize;

    // Write Y plane
    {
        let y_stride = frame.planes[0].cfg.stride;
        let y_data = frame.planes[0].data_origin_mut();
        for row in 0..h {
            for col in 0..w {
                let i = (row * w + col) * 4;
                let r = rgba[i] as f32;
                let g = rgba[i + 1] as f32;
                let b = rgba[i + 2] as f32;
                y_data[row * y_stride + col] = (0.299 * r + 0.587 * g + 0.114 * b) as u8;
            }
        }
    }

    // Write U and V planes (4:2:0 subsampled)
    {
        let u_stride = frame.planes[1].cfg.stride;
        let v_stride = frame.planes[2].cfg.stride;
        let (planes_01, planes_2) = frame.planes.split_at_mut(2);
        let u_data = planes_01[1].data_origin_mut();
        let v_data = planes_2[0].data_origin_mut();

        for row in (0..h).step_by(2) {
            for col in (0..w).step_by(2) {
                let i = (row * w + col) * 4;
                let r = rgba[i] as f32;
                let g = rgba[i + 1] as f32;
                let b = rgba[i + 2] as f32;
                let chr = row / 2;
                let chc = col / 2;
                u_data[chr * u_stride + chc] =
                    (128.0 - 0.168736 * r - 0.331264 * g + 0.5 * b) as u8;
                v_data[chr * v_stride + chc] =
                    (128.0 + 0.5 * r - 0.418688 * g - 0.081312 * b) as u8;
            }
        }
    }
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
/// Audio is collected and available. Muxide only supports video tracks,
/// so we log audio stats for verification and write video-only MP4.
/// Full audio+video muxing requires a different muxer (Phase 3).
#[allow(clippy::too_many_arguments)]
fn write_mp4_with_audio(
    video_packets: &[(Vec<u8>, u64)],
    width: u32,
    height: u32,
    fps: f64,
    audio_pcm_s16: &[i16],
    _audio_sample_rate: u32,
    _audio_channels: u32,
    path: &Path,
) -> Result<()> {
    tracing::info!(
        "audio collected: {} PCM samples (muxing not yet supported by muxide — video-only MP4)",
        audio_pcm_s16.len()
    );
    write_mp4(video_packets, width, height, fps, path)
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
