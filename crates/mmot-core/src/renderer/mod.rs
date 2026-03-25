pub mod blend;
pub mod effects;
mod gradient;
mod image;
pub mod layers;
pub mod masks;
pub mod shape;
mod solid;
mod surface;
pub mod text;
pub mod transition;

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
    /// When `true`, the layer fills the entire canvas (AbsoluteFill).
    pub fill_parent: bool,
    /// Optional compositing blend mode for this layer.
    pub blend_mode: Option<crate::schema::effects::BlendMode>,
    /// Optional masks for clipping this layer.
    pub masks: Option<Vec<crate::schema::effects::Mask>>,
    /// Optional visual effects (blur, shadow, color correction, etc.).
    pub effects: Option<Vec<crate::schema::effects::Effect>>,
}

/// Resolved transform values (no keyframes).
#[derive(Clone)]
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
        custom_font_data: Option<Vec<u8>>,
    },
    Shape {
        shape: shape::ResolvedShape,
    },
    Gradient {
        gradient: crate::schema::GradientSpec,
        width: u32,
        height: u32,
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
    if hex.len() < 6 {
        return skia_safe::Color::BLACK;
    }
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
                fill_parent: false,
                blend_mode: None,
                masks: None,
                effects: None,
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
                fill_parent: false,
                blend_mode: None,
                masks: None,
                effects: None,
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
                fill_parent: false,
                blend_mode: None,
                masks: None,
                effects: None,
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
                fill_parent: false,
                blend_mode: None,
                masks: None,
                effects: None,
            }],
        };
        let rgba = render(&frame).unwrap();
        let has_green = rgba.chunks(4).any(|px| px[1] > 200 && px[0] < 50);
        assert!(has_green, "polygon triangle should have green pixels");
    }

    #[test]
    fn gradient_linear_renders() {
        use crate::schema::{GradientSpec, GradientStop};
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
                content: ResolvedContent::Gradient {
                    gradient: GradientSpec::Linear {
                        start: [0.0, 0.0],
                        end: [1.0, 0.0],
                        colors: vec![
                            GradientStop {
                                offset: 0.0,
                                color: "#ff0000".into(),
                            },
                            GradientStop {
                                offset: 1.0,
                                color: "#0000ff".into(),
                            },
                        ],
                    },
                    width: 100,
                    height: 100,
                },
                fill_parent: false,
                blend_mode: None,
                masks: None,
                effects: None,
            }],
        };
        let rgba = render(&frame).unwrap();
        assert_eq!(rgba.len(), 100 * 100 * 4);
    }

    #[test]
    fn gradient_radial_renders() {
        use crate::schema::{GradientSpec, GradientStop};
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
                content: ResolvedContent::Gradient {
                    gradient: GradientSpec::Radial {
                        center: [0.5, 0.5],
                        radius: 0.5,
                        colors: vec![
                            GradientStop {
                                offset: 0.0,
                                color: "#ffffff".into(),
                            },
                            GradientStop {
                                offset: 1.0,
                                color: "#000000".into(),
                            },
                        ],
                    },
                    width: 100,
                    height: 100,
                },
                fill_parent: false,
                blend_mode: None,
                masks: None,
                effects: None,
            }],
        };
        let rgba = render(&frame).unwrap();
        assert_eq!(rgba.len(), 100 * 100 * 4);
    }

    #[test]
    fn fill_parent_layer_fills_canvas() {
        let frame = FrameScene {
            width: 100,
            height: 100,
            background: "#000000".into(),
            layers: vec![ResolvedLayer {
                opacity: 1.0,
                transform: ResolvedTransform {
                    position: Vec2 { x: 0.0, y: 0.0 },
                    scale: Vec2 { x: 1.0, y: 1.0 },
                    rotation: 0.0,
                    opacity: 1.0,
                },
                content: ResolvedContent::Solid {
                    color: "#ff0000".into(),
                },
                fill_parent: true,
                blend_mode: None,
                masks: None,
                effects: None,
            }],
        };
        let rgba = render(&frame).unwrap();
        // Every pixel should be red (layer fills entire canvas)
        assert!(rgba.chunks(4).all(|px| px[0] == 255 && px[1] == 0 && px[2] == 0));
    }

    #[test]
    fn blend_mode_multiply_changes_output() {
        use crate::schema::effects::BlendMode;
        // Red layer on white background with Normal blend
        let normal_frame = FrameScene {
            width: 4,
            height: 4,
            background: "#ffffff".into(),
            layers: vec![ResolvedLayer {
                opacity: 1.0,
                transform: ResolvedTransform {
                    position: Vec2 { x: 2.0, y: 2.0 },
                    scale: Vec2 { x: 1.0, y: 1.0 },
                    rotation: 0.0,
                    opacity: 1.0,
                },
                content: ResolvedContent::Solid {
                    color: "#ff0000".into(),
                },
                fill_parent: true,
                blend_mode: None,
                masks: None,
                effects: None,
            }],
        };
        // Red layer on gray background with Multiply blend
        let multiply_frame = FrameScene {
            width: 4,
            height: 4,
            background: "#808080".into(),
            layers: vec![ResolvedLayer {
                opacity: 1.0,
                transform: ResolvedTransform {
                    position: Vec2 { x: 2.0, y: 2.0 },
                    scale: Vec2 { x: 1.0, y: 1.0 },
                    rotation: 0.0,
                    opacity: 1.0,
                },
                content: ResolvedContent::Solid {
                    color: "#ff0000".into(),
                },
                fill_parent: true,
                blend_mode: Some(BlendMode::Multiply),
                masks: None,
                effects: None,
            }],
        };
        let normal_rgba = render(&normal_frame).unwrap();
        let multiply_rgba = render(&multiply_frame).unwrap();
        // Multiply of red on gray should produce darker red than normal
        // Normal: red layer replaces gray -> pure red (255,0,0)
        // Multiply: red * gray -> (128,0,0) approximately
        assert_ne!(normal_rgba, multiply_rgba);
    }

    #[test]
    fn mask_clips_layer() {
        use crate::schema::effects::{Mask, MaskMode, MaskPath};
        // Red solid with ellipse mask — only center pixels should be red
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
                content: ResolvedContent::Solid {
                    color: "#ff0000".into(),
                },
                fill_parent: true,
                blend_mode: None,
                masks: Some(vec![Mask {
                    path: MaskPath::Ellipse {
                        cx: 50.0,
                        cy: 50.0,
                        rx: 20.0,
                        ry: 20.0,
                    },
                    mode: MaskMode::Add,
                    feather: 0.0,
                    opacity: 1.0,
                    inverted: false,
                }]),
                effects: None,
            }],
        };
        let rgba = render(&frame).unwrap();
        // Corner pixel (0,0) should be black (masked out)
        assert_eq!(rgba[0], 0, "corner should be black (masked)");
        // Center pixel (50,50) should be red
        let center = (50 * 100 + 50) * 4;
        assert_eq!(rgba[center], 255, "center should be red (inside mask)");
    }

    #[test]
    fn blur_effect_changes_output() {
        use crate::schema::effects::Effect;
        // Use a small rectangle shape so the blur has edges to soften
        let sharp = FrameScene {
            width: 40,
            height: 40,
            background: "#000000".into(),
            layers: vec![ResolvedLayer {
                opacity: 1.0,
                transform: ResolvedTransform {
                    position: Vec2 { x: 20.0, y: 20.0 },
                    scale: Vec2 { x: 1.0, y: 1.0 },
                    rotation: 0.0,
                    opacity: 1.0,
                },
                content: ResolvedContent::Shape {
                    shape: shape::ResolvedShape::Rect {
                        width: 16.0,
                        height: 16.0,
                        corner_radius: 0.0,
                        fill: Some("#ffffff".into()),
                        stroke_color: None,
                        stroke_width: 0.0,
                    },
                },
                fill_parent: false,
                blend_mode: None,
                masks: None,
                effects: None,
            }],
        };
        let blurred = FrameScene {
            width: 40,
            height: 40,
            background: "#000000".into(),
            layers: vec![ResolvedLayer {
                opacity: 1.0,
                transform: ResolvedTransform {
                    position: Vec2 { x: 20.0, y: 20.0 },
                    scale: Vec2 { x: 1.0, y: 1.0 },
                    rotation: 0.0,
                    opacity: 1.0,
                },
                content: ResolvedContent::Shape {
                    shape: shape::ResolvedShape::Rect {
                        width: 16.0,
                        height: 16.0,
                        corner_radius: 0.0,
                        fill: Some("#ffffff".into()),
                        stroke_color: None,
                        stroke_width: 0.0,
                    },
                },
                fill_parent: false,
                blend_mode: None,
                masks: None,
                effects: Some(vec![Effect::GaussianBlur { radius: 3.0 }]),
            }],
        };
        let sharp_rgba = render(&sharp).unwrap();
        let blurred_rgba = render(&blurred).unwrap();
        assert_ne!(sharp_rgba, blurred_rgba, "blur should change output");
    }

    #[test]
    fn invert_effect_inverts_colors() {
        use crate::schema::effects::Effect;
        let frame = FrameScene {
            width: 4,
            height: 4,
            background: "#000000".into(),
            layers: vec![ResolvedLayer {
                opacity: 1.0,
                transform: ResolvedTransform {
                    position: Vec2 { x: 2.0, y: 2.0 },
                    scale: Vec2 { x: 1.0, y: 1.0 },
                    rotation: 0.0,
                    opacity: 1.0,
                },
                content: ResolvedContent::Solid {
                    color: "#ff0000".into(),
                },
                fill_parent: true,
                blend_mode: None,
                masks: None,
                effects: Some(vec![Effect::Invert]),
            }],
        };
        let rgba = render(&frame).unwrap();
        // Red inverted should be cyan (0, 255, 255)
        assert_eq!(rgba[0], 0, "R should be 0 after invert");
        assert_eq!(rgba[1], 255, "G should be 255 after invert");
        assert_eq!(rgba[2], 255, "B should be 255 after invert");
    }

    #[test]
    fn drop_shadow_effect_produces_filter() {
        use crate::schema::effects::Effect;
        let frame = FrameScene {
            width: 20,
            height: 20,
            background: "#ffffff".into(),
            layers: vec![ResolvedLayer {
                opacity: 1.0,
                transform: ResolvedTransform {
                    position: Vec2 { x: 10.0, y: 10.0 },
                    scale: Vec2 { x: 1.0, y: 1.0 },
                    rotation: 0.0,
                    opacity: 1.0,
                },
                content: ResolvedContent::Solid {
                    color: "#ff0000".into(),
                },
                fill_parent: false,
                blend_mode: None,
                masks: None,
                effects: Some(vec![Effect::DropShadow {
                    color: "#000000".into(),
                    offset_x: 3.0,
                    offset_y: 3.0,
                    blur: 2.0,
                    opacity: 0.8,
                }]),
            }],
        };
        // Should render without panicking
        let rgba = render(&frame).unwrap();
        assert_eq!(rgba.len(), 20 * 20 * 4);
    }

    #[test]
    fn brightness_contrast_effect_changes_output() {
        use crate::schema::effects::Effect;
        let normal = FrameScene {
            width: 4,
            height: 4,
            background: "#000000".into(),
            layers: vec![ResolvedLayer {
                opacity: 1.0,
                transform: ResolvedTransform {
                    position: Vec2 { x: 2.0, y: 2.0 },
                    scale: Vec2 { x: 1.0, y: 1.0 },
                    rotation: 0.0,
                    opacity: 1.0,
                },
                content: ResolvedContent::Solid {
                    color: "#808080".into(),
                },
                fill_parent: true,
                blend_mode: None,
                masks: None,
                effects: None,
            }],
        };
        let bright = FrameScene {
            width: 4,
            height: 4,
            background: "#000000".into(),
            layers: vec![ResolvedLayer {
                opacity: 1.0,
                transform: ResolvedTransform {
                    position: Vec2 { x: 2.0, y: 2.0 },
                    scale: Vec2 { x: 1.0, y: 1.0 },
                    rotation: 0.0,
                    opacity: 1.0,
                },
                content: ResolvedContent::Solid {
                    color: "#808080".into(),
                },
                fill_parent: true,
                blend_mode: None,
                masks: None,
                effects: Some(vec![Effect::BrightnessContrast {
                    brightness: 50.0,
                    contrast: 0.0,
                }]),
            }],
        };
        let normal_rgba = render(&normal).unwrap();
        let bright_rgba = render(&bright).unwrap();
        assert_ne!(normal_rgba, bright_rgba, "brightness effect should change output");
    }
}
