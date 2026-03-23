mod image;
pub mod layers;
mod solid;
mod surface;
pub mod text;

use crate::error::Result;
use crate::schema::{TextAlign, Vec2};

/// A fully resolved frame — all animatable values are concrete.
/// This is what the renderer receives. No JSON, no keyframes.
pub struct FrameScene {
    pub width: u32,
    pub height: u32,
    pub background: String,
    pub layers: Vec<ResolvedLayer>,
}

/// A layer with all values resolved for a specific frame.
pub struct ResolvedLayer {
    pub opacity: f64,
    pub transform: ResolvedTransform,
    pub content: ResolvedContent,
}

/// Resolved transform values (no keyframes).
pub struct ResolvedTransform {
    pub position: Vec2,
    pub scale: Vec2,
    pub rotation: f64,
    pub opacity: f64,
}

/// Resolved layer content (no keyframes).
pub enum ResolvedContent {
    Solid {
        color: String,
    },
    Image {
        data: Vec<u8>,
        width: u32,
        height: u32,
    },
    Text {
        text: String,
        font_family: String,
        font_size: f64,
        font_weight: u32,
        color: String,
        align: TextAlign,
    },
}

/// Render a FrameScene to raw RGBA bytes (width x height x 4).
pub fn render(frame_scene: &FrameScene) -> Result<Vec<u8>> {
    let w = frame_scene.width;
    let h = frame_scene.height;
    let mut surface = surface::create_cpu_surface(w, h);
    let canvas = surface.canvas();

    // Clear with background colour
    canvas.clear(parse_color(&frame_scene.background));

    // Draw layers in order (first = bottom of visual stack)
    for layer in &frame_scene.layers {
        layers::draw_layer(canvas, layer, w, h);
    }

    // Extract RGBA pixels directly — no PNG roundtrip
    let row_bytes = (w * 4) as usize;
    let mut rgba = vec![0u8; (w * h * 4) as usize];
    let info = skia_safe::ImageInfo::new(
        (w as i32, h as i32),
        skia_safe::ColorType::RGBA8888,
        skia_safe::AlphaType::Premul,
        None,
    );
    surface.read_pixels(&info, &mut rgba, row_bytes, (0, 0));
    Ok(rgba)
}

fn parse_color(hex: &str) -> skia_safe::Color {
    let hex = hex.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
    skia_safe::Color::from_argb(255, r, g, b)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_solid_frame(color: &str, width: u32, height: u32) -> FrameScene {
        FrameScene {
            width,
            height,
            background: "#000000".into(),
            layers: vec![ResolvedLayer {
                opacity: 1.0,
                transform: ResolvedTransform {
                    position: Vec2 {
                        x: (width / 2) as f64,
                        y: (height / 2) as f64,
                    },
                    scale: Vec2 { x: 1.0, y: 1.0 },
                    rotation: 0.0,
                    opacity: 1.0,
                },
                content: ResolvedContent::Solid {
                    color: color.into(),
                },
            }],
        }
    }

    #[test]
    fn solid_red_fills_buffer() {
        let frame = make_solid_frame("#ff0000", 4, 4);
        let rgba = render(&frame).unwrap();
        assert_eq!(rgba.len(), 4 * 4 * 4);
        // First pixel should be red
        assert_eq!(rgba[0], 255); // R
        assert_eq!(rgba[1], 0); // G
        assert_eq!(rgba[2], 0); // B
        assert_eq!(rgba[3], 255); // A
    }

    #[test]
    fn output_dimensions_match_frame() {
        let frame = make_solid_frame("#ffffff", 16, 9);
        let rgba = render(&frame).unwrap();
        assert_eq!(rgba.len(), 16 * 9 * 4);
    }
}
