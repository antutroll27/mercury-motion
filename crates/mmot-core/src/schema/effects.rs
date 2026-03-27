use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::schema::AnimatableValue;

// ── BlendMode ──────────────────────────────────────────────────────────────────

/// Compositing blend mode for a layer, matching After Effects blend modes.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BlendMode {
    #[default]
    Normal,
    Multiply,
    Screen,
    Overlay,
    Darken,
    Lighten,
    ColorDodge,
    ColorBurn,
    HardLight,
    SoftLight,
    Difference,
    Exclusion,
    Add,
}

// ── TimeRemap ──────────────────────────────────────────────────────────────────

/// Time remapping controls for a layer.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TimeRemap {
    /// Playback speed multiplier (1.0 = normal speed).
    #[serde(default = "default_speed")]
    pub speed: f64,
    /// Time offset in seconds.
    #[serde(default)]
    pub offset: f64,
    /// Whether to play the layer in reverse.
    #[serde(default)]
    pub reverse: bool,
}

fn default_speed() -> f64 {
    1.0
}

impl Default for TimeRemap {
    fn default() -> Self {
        Self {
            speed: 1.0,
            offset: 0.0,
            reverse: false,
        }
    }
}

// ── Mask ────────────────────────────────────────────────────────────────────────

/// A mask applied to a layer, defining a clipping/compositing region.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Mask {
    /// The geometric path defining the mask shape.
    pub path: MaskPath,
    /// How this mask combines with other masks on the same layer.
    #[serde(default)]
    pub mode: MaskMode,
    /// Feather radius in pixels for softening the mask edge.
    #[serde(default)]
    pub feather: f64,
    /// Opacity of the mask (0.0 = fully transparent, 1.0 = fully opaque).
    #[serde(default = "default_opacity")]
    pub opacity: f64,
    /// When true, the mask region is inverted.
    #[serde(default)]
    pub inverted: bool,
}

fn default_opacity() -> f64 {
    1.0
}

/// Geometric path for a mask.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MaskPath {
    /// A rectangular mask region.
    Rect {
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        #[serde(default)]
        corner_radius: f64,
    },
    /// An elliptical mask region.
    Ellipse {
        cx: f64,
        cy: f64,
        rx: f64,
        ry: f64,
    },
    /// A freeform path mask defined by control points.
    Path {
        points: Vec<[f64; 2]>,
        #[serde(default)]
        closed: bool,
    },
}

/// How a mask combines with other masks on the same layer.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MaskMode {
    #[default]
    Add,
    Subtract,
    Intersect,
    Difference,
}

// ── TrackMatte ─────────────────────────────────────────────────────────────────

/// A track matte uses another layer's alpha or luma channel to define visibility.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TrackMatte {
    /// ID of the source layer used as the matte.
    pub source: String,
    /// Which channel of the source layer drives the matte.
    #[serde(default)]
    pub mode: TrackMatteMode,
}

/// Which channel of the matte source layer drives visibility.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TrackMatteMode {
    #[default]
    Alpha,
    AlphaInverted,
    Luma,
    LumaInverted,
}

// ── Effect ──────────────────────────────────────────────────────────────────────

/// A visual effect applied to a layer, matching common After Effects effects.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Effect {
    /// Gaussian blur with configurable radius.
    GaussianBlur {
        radius: f64,
    },
    /// Drop shadow cast behind the layer.
    DropShadow {
        color: String,
        offset_x: f64,
        offset_y: f64,
        blur: f64,
        #[serde(default = "default_opacity")]
        opacity: f64,
    },
    /// Glow effect emanating from bright regions.
    Glow {
        color: String,
        radius: f64,
        #[serde(default = "default_intensity")]
        intensity: f64,
    },
    /// Brightness and contrast adjustment.
    BrightnessContrast {
        #[serde(default)]
        brightness: f64,
        #[serde(default)]
        contrast: f64,
    },
    /// Hue, saturation, and lightness adjustment.
    HueSaturation {
        #[serde(default)]
        hue: f64,
        #[serde(default)]
        saturation: f64,
        #[serde(default)]
        lightness: f64,
    },
    /// Inverts all color channels.
    Invert,
    /// Tints the layer toward a target color.
    Tint {
        color: String,
        #[serde(default = "default_opacity")]
        amount: f64,
    },
    /// Fills the entire layer with a solid color.
    Fill {
        color: String,
        #[serde(default = "default_opacity")]
        opacity: f64,
    },
}

fn default_intensity() -> f64 {
    1.0
}

// ── TrimPaths ──────────────────────────────────────────────────────────────────

/// Trim paths controls for shape layers — animates which portion of a path is visible.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TrimPaths {
    /// Start of the visible portion (0.0 to 1.0).
    #[serde(default)]
    pub start: AnimatableValue<f64>,
    /// End of the visible portion (0.0 to 1.0).
    #[serde(default = "default_end")]
    pub end: AnimatableValue<f64>,
    /// Offset rotates the start/end along the path (0.0 to 1.0).
    #[serde(default)]
    pub offset: AnimatableValue<f64>,
}

fn default_end() -> AnimatableValue<f64> {
    AnimatableValue::Static(1.0)
}

// ── PathAnimation ──────────────────────────────────────────────────────────────

/// Animates the position of a layer along a path defined by control points.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PathAnimation {
    /// Control points defining the motion path.
    pub points: Vec<[f64; 2]>,
    /// When true, the layer's rotation follows the tangent of the path.
    #[serde(default)]
    pub auto_orient: bool,
}

// ── FcurveModifier ──────────────────────────────────────────────────────────────

/// A modifier that post-processes an interpolated animation value.
/// Modifiers are applied in order after keyframe evaluation.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum FcurveModifier {
    /// Add deterministic pseudo-random noise to the value.
    Wiggle {
        #[serde(default = "default_amplitude")]
        amplitude: f64,
        #[serde(default = "default_frequency")]
        frequency: f64,
        #[serde(default)]
        seed: u32,
    },
    /// Loop the animation using repeat or ping-pong.
    Loop {
        #[serde(default)]
        mode: LoopMode,
    },
    /// Clamp the value to a min/max range.
    Clamp {
        min: f64,
        max: f64,
    },
}

/// How a looped animation repeats after the last keyframe.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum LoopMode {
    #[default]
    Repeat,
    PingPong,
}

fn default_amplitude() -> f64 {
    0.1
}

fn default_frequency() -> f64 {
    3.0
}

// ── Tests ───────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blend_mode_deserializes_snake_case() {
        let mode: BlendMode = serde_json::from_str(r#""multiply""#).unwrap();
        assert!(matches!(mode, BlendMode::Multiply));

        let mode: BlendMode = serde_json::from_str(r#""screen""#).unwrap();
        assert!(matches!(mode, BlendMode::Screen));

        let mode: BlendMode = serde_json::from_str(r#""color_dodge""#).unwrap();
        assert!(matches!(mode, BlendMode::ColorDodge));

        let mode: BlendMode = serde_json::from_str(r#""hard_light""#).unwrap();
        assert!(matches!(mode, BlendMode::HardLight));

        let mode: BlendMode = serde_json::from_str(r#""soft_light""#).unwrap();
        assert!(matches!(mode, BlendMode::SoftLight));
    }

    #[test]
    fn blend_mode_default_is_normal() {
        let mode = BlendMode::default();
        assert!(matches!(mode, BlendMode::Normal));
    }

    #[test]
    fn effect_gaussian_blur_from_json() {
        let json = r#"{"type":"gaussian_blur","radius":5.0}"#;
        let effect: Effect = serde_json::from_str(json).unwrap();
        match effect {
            Effect::GaussianBlur { radius } => assert_eq!(radius, 5.0),
            _ => panic!("expected GaussianBlur"),
        }
    }

    #[test]
    fn effect_drop_shadow_from_json() {
        let json = r##"{"type":"drop_shadow","color":"#000000","offset_x":4.0,"offset_y":4.0,"blur":10.0,"opacity":0.5}"##;
        let effect: Effect = serde_json::from_str(json).unwrap();
        match effect {
            Effect::DropShadow {
                color,
                offset_x,
                offset_y,
                blur,
                opacity,
            } => {
                assert_eq!(color, "#000000");
                assert_eq!(offset_x, 4.0);
                assert_eq!(offset_y, 4.0);
                assert_eq!(blur, 10.0);
                assert_eq!(opacity, 0.5);
            }
            _ => panic!("expected DropShadow"),
        }
    }

    #[test]
    fn effect_invert_from_json() {
        let json = r#"{"type":"invert"}"#;
        let effect: Effect = serde_json::from_str(json).unwrap();
        assert!(matches!(effect, Effect::Invert));
    }

    #[test]
    fn effect_brightness_contrast_defaults() {
        let json = r#"{"type":"brightness_contrast"}"#;
        let effect: Effect = serde_json::from_str(json).unwrap();
        match effect {
            Effect::BrightnessContrast {
                brightness,
                contrast,
            } => {
                assert_eq!(brightness, 0.0);
                assert_eq!(contrast, 0.0);
            }
            _ => panic!("expected BrightnessContrast"),
        }
    }

    #[test]
    fn effect_hue_saturation_from_json() {
        let json = r#"{"type":"hue_saturation","hue":30.0,"saturation":-20.0,"lightness":10.0}"#;
        let effect: Effect = serde_json::from_str(json).unwrap();
        match effect {
            Effect::HueSaturation {
                hue,
                saturation,
                lightness,
            } => {
                assert_eq!(hue, 30.0);
                assert_eq!(saturation, -20.0);
                assert_eq!(lightness, 10.0);
            }
            _ => panic!("expected HueSaturation"),
        }
    }

    #[test]
    fn mask_with_rect_path() {
        let json = r#"{
            "path": {"type":"rect","x":0,"y":0,"width":100,"height":100,"corner_radius":5},
            "mode": "subtract",
            "feather": 2.0,
            "opacity": 0.8,
            "inverted": true
        }"#;
        let mask: Mask = serde_json::from_str(json).unwrap();
        assert!(matches!(mask.path, MaskPath::Rect { .. }));
        assert!(matches!(mask.mode, MaskMode::Subtract));
        assert_eq!(mask.feather, 2.0);
        assert_eq!(mask.opacity, 0.8);
        assert!(mask.inverted);
    }

    #[test]
    fn mask_with_ellipse_path() {
        let json = r#"{
            "path": {"type":"ellipse","cx":50,"cy":50,"rx":30,"ry":20}
        }"#;
        let mask: Mask = serde_json::from_str(json).unwrap();
        match mask.path {
            MaskPath::Ellipse { cx, cy, rx, ry } => {
                assert_eq!(cx, 50.0);
                assert_eq!(cy, 50.0);
                assert_eq!(rx, 30.0);
                assert_eq!(ry, 20.0);
            }
            _ => panic!("expected Ellipse"),
        }
        // Defaults
        assert!(matches!(mask.mode, MaskMode::Add));
        assert_eq!(mask.feather, 0.0);
        assert_eq!(mask.opacity, 1.0);
        assert!(!mask.inverted);
    }

    #[test]
    fn mask_with_freeform_path() {
        let json = r#"{
            "path": {"type":"path","points":[[0,0],[100,0],[100,100],[0,100]],"closed":true}
        }"#;
        let mask: Mask = serde_json::from_str(json).unwrap();
        match mask.path {
            MaskPath::Path { ref points, closed } => {
                assert_eq!(points.len(), 4);
                assert!(closed);
            }
            _ => panic!("expected Path"),
        }
    }

    #[test]
    fn time_remap_defaults() {
        let json = r#"{}"#;
        let tr: TimeRemap = serde_json::from_str(json).unwrap();
        assert_eq!(tr.speed, 1.0);
        assert_eq!(tr.offset, 0.0);
        assert!(!tr.reverse);
    }

    #[test]
    fn time_remap_custom_values() {
        let json = r#"{"speed":2.0,"offset":0.5,"reverse":true}"#;
        let tr: TimeRemap = serde_json::from_str(json).unwrap();
        assert_eq!(tr.speed, 2.0);
        assert_eq!(tr.offset, 0.5);
        assert!(tr.reverse);
    }

    #[test]
    fn track_matte_defaults() {
        let json = r#"{"source":"matte_layer"}"#;
        let tm: TrackMatte = serde_json::from_str(json).unwrap();
        assert_eq!(tm.source, "matte_layer");
        assert!(matches!(tm.mode, TrackMatteMode::Alpha));
    }

    #[test]
    fn track_matte_luma_inverted() {
        let json = r#"{"source":"luma_src","mode":"luma_inverted"}"#;
        let tm: TrackMatte = serde_json::from_str(json).unwrap();
        assert!(matches!(tm.mode, TrackMatteMode::LumaInverted));
    }

    #[test]
    fn trim_paths_static_values() {
        let json = r#"{"start":0.2,"end":0.8,"offset":0.0}"#;
        let tp: TrimPaths = serde_json::from_str(json).unwrap();
        match tp.start {
            AnimatableValue::Static(v) => assert_eq!(v, 0.2),
            _ => panic!("expected Static start"),
        }
        match tp.end {
            AnimatableValue::Static(v) => assert_eq!(v, 0.8),
            _ => panic!("expected Static end"),
        }
    }

    #[test]
    fn trim_paths_with_keyframes() {
        let json = r#"{
            "start": [{"t":0,"v":0.0},{"t":30,"v":1.0}],
            "end": 1.0,
            "offset": 0.0
        }"#;
        let tp: TrimPaths = serde_json::from_str(json).unwrap();
        match tp.start {
            AnimatableValue::Animated(ref kfs) => {
                assert_eq!(kfs.len(), 2);
                assert_eq!(kfs[0].t, 0);
                assert_eq!(kfs[0].v, 0.0);
                assert_eq!(kfs[1].t, 30);
                assert_eq!(kfs[1].v, 1.0);
            }
            _ => panic!("expected Animated start"),
        }
        match tp.end {
            AnimatableValue::Static(v) => assert_eq!(v, 1.0),
            _ => panic!("expected Static end"),
        }
    }

    #[test]
    fn trim_paths_defaults() {
        let json = r#"{}"#;
        let tp: TrimPaths = serde_json::from_str(json).unwrap();
        match tp.start {
            AnimatableValue::Static(v) => assert_eq!(v, 0.0),
            _ => panic!("expected default Static start of 0.0"),
        }
        match tp.end {
            AnimatableValue::Static(v) => assert_eq!(v, 1.0),
            _ => panic!("expected default Static end of 1.0"),
        }
    }

    #[test]
    fn path_animation_deserializes() {
        let json = r#"{"points":[[0,0],[50,100],[100,0]],"auto_orient":true}"#;
        let pa: PathAnimation = serde_json::from_str(json).unwrap();
        assert_eq!(pa.points.len(), 3);
        assert!(pa.auto_orient);
    }

    #[test]
    fn path_animation_auto_orient_default() {
        let json = r#"{"points":[[0,0],[100,100]]}"#;
        let pa: PathAnimation = serde_json::from_str(json).unwrap();
        assert!(!pa.auto_orient);
    }

    #[test]
    fn effect_glow_from_json() {
        let json = r##"{"type":"glow","color":"#ffffff","radius":10.0,"intensity":1.5}"##;
        let effect: Effect = serde_json::from_str(json).unwrap();
        match effect {
            Effect::Glow {
                color,
                radius,
                intensity,
            } => {
                assert_eq!(color, "#ffffff");
                assert_eq!(radius, 10.0);
                assert_eq!(intensity, 1.5);
            }
            _ => panic!("expected Glow"),
        }
    }

    #[test]
    fn effect_tint_from_json() {
        let json = r##"{"type":"tint","color":"#ff8800","amount":0.7}"##;
        let effect: Effect = serde_json::from_str(json).unwrap();
        match effect {
            Effect::Tint { color, amount } => {
                assert_eq!(color, "#ff8800");
                assert_eq!(amount, 0.7);
            }
            _ => panic!("expected Tint"),
        }
    }

    #[test]
    fn effect_fill_from_json() {
        let json = r##"{"type":"fill","color":"#00ff00","opacity":0.5}"##;
        let effect: Effect = serde_json::from_str(json).unwrap();
        match effect {
            Effect::Fill { color, opacity } => {
                assert_eq!(color, "#00ff00");
                assert_eq!(opacity, 0.5);
            }
            _ => panic!("expected Fill"),
        }
    }
}
