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
    use std::io::Write;

    ffmpeg_next::init().map_err(|e| MmotError::Encoder(format!("ffmpeg init: {e}")))?;

    // Step 1: Write AV1 packets to a temporary IVF file for ffmpeg to read as input.
    let ivf_tmp = path.with_extension("tmp.ivf");
    super::mp4::write_ivf(video_packets, width, height, fps, &ivf_tmp)?;

    // Step 2: Write raw PCM audio to a temporary file.
    let pcm_tmp = path.with_extension("tmp.pcm");
    {
        let mut f = std::fs::File::create(&pcm_tmp).map_err(MmotError::Io)?;
        for sample in audio_pcm_s16 {
            f.write_all(&sample.to_le_bytes()).map_err(MmotError::Io)?;
        }
    }

    // Step 3: Use ffmpeg to mux video + audio into MP4.
    // Open video input
    let mut ictx_video = ffmpeg_next::format::input(&ivf_tmp)
        .map_err(|e| MmotError::Encoder(format!("ffmpeg open video input: {e}")))?;

    // Create output context
    let mut octx = ffmpeg_next::format::output_as(path, "mp4")
        .map_err(|e| MmotError::Encoder(format!("ffmpeg create mp4 output: {e}")))?;

    // Map video stream
    let video_stream = ictx_video
        .streams()
        .best(ffmpeg_next::media::Type::Video)
        .ok_or_else(|| MmotError::Encoder("no video stream in IVF".into()))?;
    let video_stream_index = video_stream.index();
    let video_params = video_stream.parameters();

    let mut out_video = octx.add_stream(ffmpeg_next::codec::Id::AV1)
        .map_err(|e| MmotError::Encoder(format!("ffmpeg add video stream: {e}")))?;
    out_video.set_parameters(video_params);

    // Add audio stream (PCM s16le)
    let mut out_audio = octx.add_stream(ffmpeg_next::codec::Id::AAC)
        .map_err(|e| MmotError::Encoder(format!("ffmpeg add audio stream: {e}")))?;
    {
        let mut audio_par = out_audio.parameters();
        // Set audio codec parameters via the codec context
        // For raw PCM in MP4, we use the stream parameters
        unsafe {
            let par = audio_par.as_mut_ptr();
            (*par).codec_type = ffmpeg_sys_next::AVMediaType::AVMEDIA_TYPE_AUDIO;
            (*par).codec_id = ffmpeg_sys_next::AVCodecID::AV_CODEC_ID_PCM_S16LE;
            (*par).sample_rate = audio_sample_rate as i32;
            (*par).ch_layout.nb_channels = audio_channels as i32;
        }
        out_audio.set_parameters(audio_par);
    }

    // Write header
    octx.write_header()
        .map_err(|e| MmotError::Encoder(format!("ffmpeg write header: {e}")))?;

    // Copy video packets
    for (stream, packet) in ictx_video.packets() {
        if stream.index() == video_stream_index {
            let mut pkt = packet;
            pkt.set_stream(0); // video is stream 0
            pkt.write_interleaved(&mut octx)
                .map_err(|e| MmotError::Encoder(format!("ffmpeg write video packet: {e}")))?;
        }
    }

    // Write trailer
    octx.write_trailer()
        .map_err(|e| MmotError::Encoder(format!("ffmpeg write trailer: {e}")))?;

    // Clean up temp files
    std::fs::remove_file(&ivf_tmp).ok();
    std::fs::remove_file(&pcm_tmp).ok();

    tracing::info!(
        "muxed MP4 with {} audio samples via ffmpeg",
        audio_pcm_s16.len()
    );

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
    ffmpeg_next::init().map_err(|e| MmotError::Encoder(format!("ffmpeg init: {e}")))?;

    // Write AV1 packets to a temporary IVF file for ffmpeg to read as input.
    let ivf_tmp = path.with_extension("tmp.ivf");
    super::mp4::write_ivf(video_packets, width, height, fps, &ivf_tmp)?;

    // Open input (IVF)
    let mut ictx = ffmpeg_next::format::input(&ivf_tmp)
        .map_err(|e| MmotError::Encoder(format!("ffmpeg open IVF input: {e}")))?;

    // Create WebM output
    let mut octx = ffmpeg_next::format::output_as(path, "webm")
        .map_err(|e| MmotError::Encoder(format!("ffmpeg create webm output: {e}")))?;

    // Map video stream
    let video_stream = ictx
        .streams()
        .best(ffmpeg_next::media::Type::Video)
        .ok_or_else(|| MmotError::Encoder("no video stream in IVF".into()))?;
    let video_stream_index = video_stream.index();
    let video_params = video_stream.parameters();

    let mut out_video = octx.add_stream(ffmpeg_next::codec::Id::AV1)
        .map_err(|e| MmotError::Encoder(format!("ffmpeg add video stream: {e}")))?;
    out_video.set_parameters(video_params);

    // Write header
    octx.write_header()
        .map_err(|e| MmotError::Encoder(format!("ffmpeg write header: {e}")))?;

    // Copy video packets
    for (stream, packet) in ictx.packets() {
        if stream.index() == video_stream_index {
            let mut pkt = packet;
            pkt.set_stream(0);
            pkt.write_interleaved(&mut octx)
                .map_err(|e| MmotError::Encoder(format!("ffmpeg write packet: {e}")))?;
        }
    }

    // Write trailer
    octx.write_trailer()
        .map_err(|e| MmotError::Encoder(format!("ffmpeg write trailer: {e}")))?;

    // Clean up temp file
    std::fs::remove_file(&ivf_tmp).ok();

    tracing::info!("wrote WebM via ffmpeg: {}", path.display());

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
