use std::path::Path;

use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

use crate::error::{MmotError, Result};

/// Decoded audio: interleaved f32 samples + metadata.
pub struct DecodedAudio {
    /// Interleaved f32 samples (all channels).
    pub samples: Vec<f32>,
    /// Number of channels.
    pub channels: u32,
    /// Sample rate in Hz.
    pub sample_rate: u32,
}

/// Decode an audio file (MP3, WAV, FLAC, OGG, AAC) to raw f32 samples.
pub fn decode_file(path: &Path) -> Result<DecodedAudio> {
    let file = std::fs::File::open(path).map_err(|_| MmotError::AssetNotFound {
        path: path.to_path_buf(),
    })?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    let mut hint = Hint::new();
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }

    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &FormatOptions::default(), &MetadataOptions::default())
        .map_err(|e| MmotError::AssetDecode {
            path: path.to_path_buf(),
            reason: format!("audio probe failed: {e}"),
        })?;

    let mut format = probed.format;

    let track = format
        .default_track()
        .ok_or_else(|| MmotError::AssetDecode {
            path: path.to_path_buf(),
            reason: "no audio track found".into(),
        })?;

    let track_id = track.id;
    let sample_rate = track
        .codec_params
        .sample_rate
        .unwrap_or(44100);
    let channels = track
        .codec_params
        .channels
        .map(|c| c.count() as u32)
        .unwrap_or(2);

    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .map_err(|e| MmotError::AssetDecode {
            path: path.to_path_buf(),
            reason: format!("codec init failed: {e}"),
        })?;

    let mut all_samples: Vec<f32> = Vec::new();

    loop {
        let packet = match format.next_packet() {
            Ok(p) => p,
            Err(symphonia::core::errors::Error::IoError(ref e))
                if e.kind() == std::io::ErrorKind::UnexpectedEof =>
            {
                break;
            }
            Err(e) => {
                return Err(MmotError::AssetDecode {
                    path: path.to_path_buf(),
                    reason: format!("packet read failed: {e}"),
                });
            }
        };

        if packet.track_id() != track_id {
            continue;
        }

        let decoded = match decoder.decode(&packet) {
            Ok(d) => d,
            Err(symphonia::core::errors::Error::DecodeError(_)) => continue,
            Err(e) => {
                return Err(MmotError::AssetDecode {
                    path: path.to_path_buf(),
                    reason: format!("decode failed: {e}"),
                });
            }
        };

        let spec = *decoded.spec();
        let num_channels = spec.channels.count();
        let num_frames = decoded.frames();
        let capacity = decoded.capacity() as u64;
        let mut sample_buf = SampleBuffer::<f32>::new(capacity, spec);
        sample_buf.copy_interleaved_ref(decoded);
        let samples = sample_buf.samples();
        let valid_count = num_frames * num_channels;
        all_samples.extend_from_slice(&samples[..valid_count]);
    }

    Ok(DecodedAudio {
        samples: all_samples,
        channels,
        sample_rate,
    })
}

/// Convert audio samples to signed 16-bit PCM (interleaved).
pub fn samples_to_pcm_s16(samples: &[f32]) -> Vec<i16> {
    samples
        .iter()
        .map(|&s| {
            let clamped = s.clamp(-1.0, 1.0);
            (clamped * 32767.0) as i16
        })
        .collect()
}
