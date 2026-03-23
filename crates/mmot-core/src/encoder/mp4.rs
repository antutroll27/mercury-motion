use std::io::Write;
use std::path::Path;

use crate::error::{MmotError, Result};

/// Encode a sequence of RGBA frames to an IVF file (AV1 bitstream).
/// Uses rav1e for AV1 encoding. Phase 1 outputs IVF; MP4 muxing added in Phase 2.
pub fn encode(
    frames: Vec<Vec<u8>>,
    width: u32,
    height: u32,
    fps: f64,
    quality: u8,
    output: &Path,
) -> Result<()> {
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

    for rgba in &frames {
        let mut frame = ctx.new_frame();
        rgba_to_yuv420(rgba, width, height, &mut frame);
        ctx.send_frame(frame)
            .map_err(|e| MmotError::Encoder(e.to_string()))?;
        drain_packets(&mut ctx, &mut encoded_packets, &mut pts)?;
    }

    ctx.flush();
    drain_packets(&mut ctx, &mut encoded_packets, &mut pts)?;

    write_ivf(&encoded_packets, width, height, fps, output)?;
    Ok(())
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
    // rav1e quantizer range: 0 (best) to 255 (worst)
    // quality 100 -> quantizer ~0, quality 1 -> quantizer ~200
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
        // Split planes mutably by index
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
                u_data[chr * u_stride + chc] = (128.0 - 0.168736 * r - 0.331264 * g + 0.5 * b) as u8;
                v_data[chr * v_stride + chc] = (128.0 + 0.5 * r - 0.418688 * g - 0.081312 * b) as u8;
            }
        }
    }
}

/// Write encoded AV1 packets as IVF container (simple AV1 bitstream format).
fn write_ivf(
    packets: &[(Vec<u8>, u64)],
    width: u32,
    height: u32,
    fps: f64,
    path: &Path,
) -> Result<()> {
    let mut file = std::fs::File::create(path).map_err(MmotError::Io)?;

    // IVF file header (32 bytes)
    file.write_all(b"DKIF").map_err(MmotError::Io)?; // signature
    file.write_all(&0u16.to_le_bytes()).map_err(MmotError::Io)?; // version
    file.write_all(&32u16.to_le_bytes()).map_err(MmotError::Io)?; // header length
    file.write_all(b"AV01").map_err(MmotError::Io)?; // codec FourCC
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
    file.write_all(&0u32.to_le_bytes()).map_err(MmotError::Io)?; // unused

    // IVF frame packets
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
    fn encode_single_black_frame() {
        let width = 64u32;
        let height = 64u32;
        let frame = vec![0u8; (width * height * 4) as usize];
        let path = std::env::temp_dir().join("mmot-encoder-test.ivf");
        encode(vec![frame], width, height, 30.0, 80, &path).unwrap();
        assert!(path.exists());
        let metadata = std::fs::metadata(&path).unwrap();
        assert!(metadata.len() > 0, "output file is empty");
        std::fs::remove_file(&path).ok();
    }
}
