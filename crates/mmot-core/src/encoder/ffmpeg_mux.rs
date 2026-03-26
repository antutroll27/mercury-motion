use std::path::Path;

use crate::error::{MmotError, Result};

/// Mux AV1 video packets + PCM audio into an MP4 container using ffmpeg.
///
/// Requires the `ffmpeg` feature flag. Without it, returns an error
/// explaining that ffmpeg is needed for audio muxing.
#[allow(clippy::too_many_arguments)]
#[cfg(feature = "ffmpeg")]
pub fn mux_mp4_with_audio(
    video_packets: &[(Vec<u8>, u64)],
    width: u32,
    height: u32,
    fps: f64,
    audio_pcm_s16: &[i16],
    audio_sample_rate: u32,
    audio_channels: u32,
    path: &Path,
) -> Result<()> {
    let ivf_tmp = path.with_extension("tmp.ivf");
    let wav_tmp = path.with_extension("tmp.wav");

    super::mp4::write_ivf(video_packets, width, height, fps, &ivf_tmp)?;
    write_wav_file(&wav_tmp, audio_pcm_s16, audio_sample_rate, audio_channels)?;

    let result = run_ffmpeg_command(&[
        "-y".into(),
        "-hide_banner".into(),
        "-loglevel".into(),
        "error".into(),
        "-i".into(),
        ivf_tmp.to_string_lossy().into_owned(),
        "-i".into(),
        wav_tmp.to_string_lossy().into_owned(),
        "-c:v".into(),
        "copy".into(),
        "-c:a".into(),
        "aac".into(),
        "-shortest".into(),
        path.to_string_lossy().into_owned(),
    ]);

    std::fs::remove_file(&ivf_tmp).ok();
    std::fs::remove_file(&wav_tmp).ok();

    result?;
    tracing::info!("muxed MP4 with audio via ffmpeg CLI: {}", path.display());
    Ok(())
}

#[allow(clippy::too_many_arguments)]
#[cfg(not(feature = "ffmpeg"))]
pub fn mux_mp4_with_audio(
    _video_packets: &[(Vec<u8>, u64)],
    _width: u32,
    _height: u32,
    _fps: f64,
    _audio_pcm_s16: &[i16],
    _audio_sample_rate: u32,
    _audio_channels: u32,
    _path: &Path,
) -> Result<()> {
    Err(MmotError::Encoder(
        "audio muxing requires the 'ffmpeg' feature flag: \
         cargo build --features ffmpeg"
            .into(),
    ))
}

/// Mux AV1 video packets into a WebM container using ffmpeg.
///
/// Requires the `ffmpeg` feature flag. Without it, returns an error
/// explaining that ffmpeg is needed for WebM output.
#[cfg(feature = "ffmpeg")]
pub fn mux_webm(
    video_packets: &[(Vec<u8>, u64)],
    width: u32,
    height: u32,
    fps: f64,
    path: &Path,
) -> Result<()> {
    let ivf_tmp = path.with_extension("tmp.ivf");
    super::mp4::write_ivf(video_packets, width, height, fps, &ivf_tmp)?;

    let result = run_ffmpeg_command(&[
        "-y".into(),
        "-hide_banner".into(),
        "-loglevel".into(),
        "error".into(),
        "-i".into(),
        ivf_tmp.to_string_lossy().into_owned(),
        "-c:v".into(),
        "copy".into(),
        path.to_string_lossy().into_owned(),
    ]);

    std::fs::remove_file(&ivf_tmp).ok();

    result?;
    tracing::info!("wrote WebM via ffmpeg CLI: {}", path.display());
    Ok(())
}

#[cfg(not(feature = "ffmpeg"))]
pub fn mux_webm(
    _video_packets: &[(Vec<u8>, u64)],
    _width: u32,
    _height: u32,
    _fps: f64,
    _path: &Path,
) -> Result<()> {
    Err(MmotError::Encoder(
        "WebM output requires the 'ffmpeg' feature flag: \
         cargo build --features ffmpeg"
            .into(),
    ))
}

#[cfg(feature = "ffmpeg")]
fn write_wav_file(
    path: &Path,
    audio_pcm_s16: &[i16],
    sample_rate: u32,
    channels: u32,
) -> Result<()> {
    use std::io::Write;

    let channels_u16 = u16::try_from(channels).map_err(|_| {
        MmotError::Encoder(format!("unsupported channel count for WAV: {channels}"))
    })?;
    let bytes_per_sample = 2u16;
    let block_align = channels_u16
        .checked_mul(bytes_per_sample)
        .ok_or_else(|| MmotError::Encoder("WAV block align overflow".into()))?;
    let byte_rate = sample_rate
        .checked_mul(block_align as u32)
        .ok_or_else(|| MmotError::Encoder("WAV byte rate overflow".into()))?;
    let data_len = u32::try_from(audio_pcm_s16.len() * std::mem::size_of::<i16>())
        .map_err(|_| MmotError::Encoder("WAV data length overflow".into()))?;
    let riff_len = 36u32
        .checked_add(data_len)
        .ok_or_else(|| MmotError::Encoder("WAV RIFF length overflow".into()))?;

    let mut file = std::fs::File::create(path).map_err(MmotError::Io)?;
    file.write_all(b"RIFF").map_err(MmotError::Io)?;
    file.write_all(&riff_len.to_le_bytes())
        .map_err(MmotError::Io)?;
    file.write_all(b"WAVE").map_err(MmotError::Io)?;
    file.write_all(b"fmt ").map_err(MmotError::Io)?;
    file.write_all(&16u32.to_le_bytes())
        .map_err(MmotError::Io)?;
    file.write_all(&1u16.to_le_bytes()).map_err(MmotError::Io)?;
    file.write_all(&channels_u16.to_le_bytes())
        .map_err(MmotError::Io)?;
    file.write_all(&sample_rate.to_le_bytes())
        .map_err(MmotError::Io)?;
    file.write_all(&byte_rate.to_le_bytes())
        .map_err(MmotError::Io)?;
    file.write_all(&block_align.to_le_bytes())
        .map_err(MmotError::Io)?;
    file.write_all(&16u16.to_le_bytes())
        .map_err(MmotError::Io)?;
    file.write_all(b"data").map_err(MmotError::Io)?;
    file.write_all(&data_len.to_le_bytes())
        .map_err(MmotError::Io)?;

    for sample in audio_pcm_s16 {
        file.write_all(&sample.to_le_bytes())
            .map_err(MmotError::Io)?;
    }

    Ok(())
}

#[cfg(feature = "ffmpeg")]
fn run_ffmpeg_command(args: &[String]) -> Result<()> {
    let output = std::process::Command::new("ffmpeg")
        .args(args)
        .output()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                MmotError::Encoder(
                    "ffmpeg executable not found on PATH; install FFmpeg or disable ffmpeg-backed output".into(),
                )
            } else {
                MmotError::Io(e)
            }
        })?;

    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    Err(MmotError::Encoder(format!(
        "ffmpeg failed with status {:?}: {}",
        output.status.code(),
        if stderr.is_empty() {
            "no stderr output".to_string()
        } else {
            stderr
        }
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(not(feature = "ffmpeg"))]
    #[test]
    fn mux_mp4_with_audio_returns_error_without_feature() {
        let err = mux_mp4_with_audio(&[], 64, 64, 30.0, &[], 44100, 2, Path::new("out.mp4"));
        assert!(err.is_err());
        let msg = err.unwrap_err().to_string();
        assert!(
            msg.contains("ffmpeg"),
            "error should mention ffmpeg feature, got: {msg}"
        );
    }

    #[cfg(not(feature = "ffmpeg"))]
    #[test]
    fn mux_webm_returns_error_without_feature() {
        let err = mux_webm(&[], 64, 64, 30.0, Path::new("out.webm"));
        assert!(err.is_err());
        let msg = err.unwrap_err().to_string();
        assert!(
            msg.contains("ffmpeg"),
            "error should mention ffmpeg feature, got: {msg}"
        );
    }
}
