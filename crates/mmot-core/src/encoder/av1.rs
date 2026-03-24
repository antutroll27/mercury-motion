use crate::error::{MmotError, Result};

/// AV1-encode RGBA frames, returning encoded packets with timestamps.
pub(crate) fn encode_av1(
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

/// Map a quality value (1–100) to a rav1e quantizer value (0–200).
pub(crate) fn map_quality_to_quantizer(quality: u8) -> usize {
    let q = quality.clamp(1, 100);
    ((100 - q) as usize * 200) / 99
}

fn rgba_to_yuv420(rgba: &[u8], width: u32, height: u32, frame: &mut rav1e::prelude::Frame<u8>) {
    let w = width as usize;
    let h = height as usize;
    let expected = w * h * 4;
    debug_assert!(
        rgba.len() >= expected,
        "RGBA buffer too small: {} < {} ({}x{}x4)",
        rgba.len(), expected, w, h
    );

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
