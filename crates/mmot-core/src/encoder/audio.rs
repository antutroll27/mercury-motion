//! Opus audio encoder (feature-gated behind `audio-codec`).

use crate::error::MmotError;

/// Encode raw f32 PCM samples into Opus packets.
///
/// Returns a `Vec` of Opus-encoded packets (one per 20 ms frame).
///
/// # Requirements
/// - Requires the `audio-codec` feature flag.
/// - `sample_rate` must be 48 000 Hz (Opus native rate).
/// - `channels` must be 1 (mono) or 2 (stereo).
#[cfg(feature = "audio-codec")]
pub fn encode_opus(
    samples: &[f32],
    sample_rate: u32,
    channels: u32,
) -> crate::error::Result<Vec<Vec<u8>>> {
    use opus::{Application, Channels, Encoder};

    let ch = match channels {
        1 => Channels::Mono,
        2 => Channels::Stereo,
        _ => {
            return Err(MmotError::AudioEncode(format!(
                "unsupported channel count: {channels}"
            )))
        }
    };

    if sample_rate != 48000 {
        return Err(MmotError::AudioEncode(format!(
            "Opus requires 48kHz, got {sample_rate}Hz"
        )));
    }

    let mut encoder = Encoder::new(48000, ch, Application::Audio)
        .map_err(|e| MmotError::AudioEncode(format!("opus init: {e}")))?;

    let frame_size = 960; // 20 ms at 48 kHz
    let frame_samples = frame_size * channels as usize;
    let mut packets = Vec::new();
    let mut output_buf = vec![0u8; 4000];

    for chunk in samples.chunks(frame_samples) {
        let frame: Vec<f32> = if chunk.len() < frame_samples {
            let mut padded = chunk.to_vec();
            padded.resize(frame_samples, 0.0);
            padded
        } else {
            chunk.to_vec()
        };

        let len = encoder
            .encode_float(&frame, &mut output_buf)
            .map_err(|e| MmotError::AudioEncode(format!("opus encode: {e}")))?;
        packets.push(output_buf[..len].to_vec());
    }

    Ok(packets)
}

/// Stub when the `audio-codec` feature is not enabled.
#[cfg(not(feature = "audio-codec"))]
pub fn encode_opus(
    _samples: &[f32],
    _sample_rate: u32,
    _channels: u32,
) -> crate::error::Result<Vec<Vec<u8>>> {
    Err(MmotError::AudioEncode(
        "requires audio-codec feature flag".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "audio-codec")]
    #[test]
    fn encode_opus_mono_silence() {
        // 20 ms of silence at 48 kHz mono = 960 samples
        let samples = vec![0.0f32; 960];
        let packets = encode_opus(&samples, 48000, 1).unwrap();
        assert_eq!(packets.len(), 1);
        assert!(!packets[0].is_empty());
    }

    #[cfg(feature = "audio-codec")]
    #[test]
    fn encode_opus_stereo_multiple_frames() {
        // 2 frames of stereo (960 * 2 * 2 = 3840 samples)
        let samples = vec![0.0f32; 3840];
        let packets = encode_opus(&samples, 48000, 2).unwrap();
        assert_eq!(packets.len(), 2);
    }

    #[cfg(feature = "audio-codec")]
    #[test]
    fn encode_opus_rejects_wrong_sample_rate() {
        let samples = vec![0.0f32; 960];
        let err = encode_opus(&samples, 44100, 1).unwrap_err();
        assert!(err.to_string().contains("48kHz"));
    }

    #[cfg(feature = "audio-codec")]
    #[test]
    fn encode_opus_rejects_unsupported_channels() {
        let samples = vec![0.0f32; 960];
        let err = encode_opus(&samples, 48000, 5).unwrap_err();
        assert!(err.to_string().contains("unsupported channel count"));
    }

    #[cfg(not(feature = "audio-codec"))]
    #[test]
    fn encode_opus_without_feature_returns_error() {
        let samples = vec![0.0f32; 960];
        let err = encode_opus(&samples, 48000, 1).unwrap_err();
        assert!(
            err.to_string().contains("requires audio-codec feature flag"),
            "expected feature-gate error, got: {err}"
        );
    }
}
