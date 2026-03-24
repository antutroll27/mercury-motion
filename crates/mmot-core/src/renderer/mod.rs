mod image;
pub mod layers;
pub mod shape;
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
    Shape {
        shape: shape::ResolvedShape,
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
        assert_eq!(rgba[0], 255);
        assert_eq!(rgba[1], 0);
        assert_eq!(rgba[2], 0);
        assert_eq!(rgba[3], 255);
    }

    #[test]
    fn output_dimensions_match_frame() {
        let frame = make_solid_frame("#ffffff", 16, 9);
        let rgba = render(&frame).unwrap();
        assert_eq!(rgba.len(), 16 * 9 * 4);
    }

    #[test]
    fn shape_rect_renders() {
        let frame = FrameScene {
            width: 100,
            height: 100,
            background: "#000000".into(),
            layers: vec![ResolvedLayer {
                opacity: 1.0,
                transform: ResolvedTransform {
                    position: Vec2 { x: 50.0, y: 50.0 },
                    scale: Vec2 { x: 1.0, y: 1.0 },
                    rotation: 0.0,
                    opacity: 1.0,
                },
                content: ResolvedContent::Shape {
                    shape: shape::ResolvedShape::Rect {
                        width: 80.0,
                        height: 40.0,
                        corner_radius: 5.0,
                        fill: Some("#00ff00".into()),
                        stroke_color: None,
                        stroke_width: 0.0,
                    },
                },
            }],
        };
        let rgba = render(&frame).unwrap();
        assert_eq!(rgba.len(), 100 * 100 * 4);
        // Should have some green pixels
        let has_green = rgba.chunks(4).any(|px| px[1] > 200 && px[0] < 50);
        assert!(has_green, "expected green pixels from shape");
    }

    #[test]
    fn shape_line_renders() {
        let frame = FrameScene {
            width: 100,
            height: 100,
            background: "#000000".into(),
            layers: vec![ResolvedLayer {
                opacity: 1.0,
                transform: ResolvedTransform {
                    position: Vec2 { x: 50.0, y: 50.0 },
                    scale: Vec2 { x: 1.0, y: 1.0 },
                    rotation: 0.0,
                    opacity: 1.0,
                },
                content: ResolvedContent::Shape {
                    shape: shape::ResolvedShape::Line {
                        x1: 0.0,
                        y1: 0.0,
                        x2: 100.0,
                        y2: 100.0,
                        stroke_color: "#ffffff".into(),
                        stroke_width: 2.0,
                    },
                },
            }],
        };
        let rgba = render(&frame).unwrap();
        let has_white =
            rgba.chunks(4).any(|px| px[0] > 200 && px[1] > 200 && px[2] > 200);
        assert!(has_white, "line should produce white pixels");
    }

    #[test]
    fn shape_polygon_renders() {
        let frame = FrameScene {
            width: 100,
            height: 100,
            background: "#000000".into(),
            layers: vec![ResolvedLayer {
                opacity: 1.0,
                transform: ResolvedTransform {
                    position: Vec2 { x: 50.0, y: 50.0 },
                    scale: Vec2 { x: 1.0, y: 1.0 },
                    rotation: 0.0,
                    opacity: 1.0,
                },
                content: ResolvedContent::Shape {
                    shape: shape::ResolvedShape::Polygon {
                        points: vec![[50.0, 10.0], [90.0, 90.0], [10.0, 90.0]],
                        fill: Some("#00ff00".into()),
                        stroke_color: None,
                        stroke_width: 0.0,
                    },
                },
            }],
        };
        let rgba = render(&frame).unwrap();
        let has_green = rgba.chunks(4).any(|px| px[1] > 200 && px[0] < 50);
        assert!(has_green, "polygon triangle should have green pixels");
    }
}
