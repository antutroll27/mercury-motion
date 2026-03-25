use std::collections::HashMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::schema::{AnimatableValue, Transform};
use crate::schema::transition::TransitionSpec;

/// Fill mode for layers — equivalent to Remotion's `<AbsoluteFill>`.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FillMode {
    Parent,
}

/// A single layer in a composition.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Layer {
    pub id: String,
    /// Frame this layer becomes active (inclusive).
    #[serde(rename = "in")]
    pub in_point: u64,
    /// Frame this layer becomes inactive (exclusive).
    #[serde(rename = "out")]
    pub out_point: u64,
    pub transform: Transform,
    /// When set to `"parent"`, the layer fills the entire canvas (position/rotation/scale ignored).
    #[serde(default)]
    pub fill: Option<FillMode>,

    #[serde(default)]
    pub blend_mode: Option<super::effects::BlendMode>,

    #[serde(default)]
    pub parent: Option<String>,

    #[serde(default)]
    pub time_remap: Option<super::effects::TimeRemap>,

    #[serde(default)]
    pub masks: Option<Vec<super::effects::Mask>>,

    #[serde(default)]
    pub track_matte: Option<super::effects::TrackMatte>,

    #[serde(default)]
    pub adjustment: bool,

    #[serde(default)]
    pub effects: Option<Vec<super::effects::Effect>>,

    #[serde(default)]
    pub motion_blur: bool,

    #[serde(default)]
    pub trim_paths: Option<super::effects::TrimPaths>,

    #[serde(default)]
    pub path_animation: Option<super::effects::PathAnimation>,

    #[serde(flatten)]
    pub content: LayerContent,
}

/// Type-specific layer content, discriminated by the `"type"` JSON field.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LayerContent {
    Solid {
        color: String,
    },
    Image {
        src: String,
    },
    Video {
        src: String,
        #[serde(default)]
        trim_start: f64,
        #[serde(default)]
        trim_end: Option<f64>,
    },
    Text {
        text: String,
        font: FontSpec,
        #[serde(default = "default_center_align")]
        align: TextAlign,
    },
    Audio {
        src: String,
        #[serde(default = "default_one_anim")]
        volume: AnimatableValue<f64>,
    },
    Lottie {
        src: String,
    },
    Composition {
        #[serde(rename = "composition_id")]
        id: String,
    },
    Shape {
        shape: ShapeSpec,
    },
    Gradient {
        gradient: GradientSpec,
    },
    Null,
}

fn default_center_align() -> TextAlign {
    TextAlign::Center
}

fn default_one_anim() -> AnimatableValue<f64> {
    AnimatableValue::Static(1.0)
}

/// Font specification for text layers.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FontSpec {
    pub family: String,
    #[serde(default = "default_font_size")]
    pub size: f64,
    #[serde(default = "default_font_weight")]
    pub weight: u32,
    #[serde(default = "default_white")]
    pub color: String,
}

fn default_font_size() -> f64 {
    32.0
}

fn default_font_weight() -> u32 {
    400
}

fn default_white() -> String {
    "#ffffff".into()
}

/// Text alignment.
#[derive(Debug, Clone, Serialize, Deserialize, Default, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TextAlign {
    Left,
    #[default]
    Center,
    Right,
}

/// Shape specification.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "shape_type", rename_all = "snake_case")]
pub enum ShapeSpec {
    Rect {
        width: f64,
        height: f64,
        corner_radius: Option<f64>,
        fill: Option<String>,
        stroke: Option<StrokeSpec>,
    },
    Ellipse {
        width: f64,
        height: f64,
        fill: Option<String>,
        stroke: Option<StrokeSpec>,
    },
    Line {
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        stroke: StrokeSpec,
    },
    Polygon {
        points: Vec<[f64; 2]>,
        fill: Option<String>,
        stroke: Option<StrokeSpec>,
    },
}

/// Stroke specification.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct StrokeSpec {
    pub color: String,
    pub width: f64,
}

/// Gradient specification — linear or radial.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "gradient_type", rename_all = "snake_case")]
pub enum GradientSpec {
    Linear {
        start: [f64; 2],
        end: [f64; 2],
        colors: Vec<GradientStop>,
    },
    Radial {
        center: [f64; 2],
        radius: f64,
        colors: Vec<GradientStop>,
    },
}

/// A single color stop in a gradient.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GradientStop {
    pub offset: f64,
    pub color: String,
}

/// A composition — an ordered list of layers (first = bottom of visual stack).
///
/// When `sequence` is `true`, layers play back-to-back instead of using their
/// individual `in`/`out` points for global timing. An optional `transition`
/// controls overlap between consecutive layers.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Composition {
    pub layers: Vec<Layer>,
    #[serde(default)]
    pub sequence: bool,
    #[serde(default)]
    pub transition: Option<TransitionSpec>,
}

/// Map of composition ID to composition.
pub type Compositions = HashMap<String, Composition>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn existing_minimal_scene_still_parses() {
        let json = std::fs::read_to_string("../../tests/fixtures/valid/minimal.mmot.json").unwrap();
        let scene: crate::schema::Scene = serde_json::from_str(&json).unwrap();
        assert!(!scene.compositions.is_empty());
    }

    #[test]
    fn layer_with_ae_fields_deserializes() {
        let json = r##"{
            "id": "test",
            "in": 0, "out": 30,
            "transform": { "position": [0, 0] },
            "type": "solid",
            "color": "#ff0000",
            "blend_mode": "multiply",
            "parent": "null_1",
            "adjustment": false,
            "effects": [
                { "type": "gaussian_blur", "radius": 5.0 }
            ]
        }"##;
        let layer: Layer = serde_json::from_str(json).unwrap();
        assert!(layer.blend_mode.is_some());
        assert_eq!(layer.parent.as_deref(), Some("null_1"));
        assert!(layer.effects.is_some());
    }

    #[test]
    fn null_layer_deserializes() {
        let json = r#"{
            "id": "null_1",
            "in": 0, "out": 30,
            "transform": { "position": [320, 180] },
            "type": "null"
        }"#;
        let layer: Layer = serde_json::from_str(json).unwrap();
        assert!(matches!(layer.content, LayerContent::Null));
    }

    #[test]
    fn layer_defaults_are_none() {
        let json = r##"{
            "id": "basic",
            "in": 0, "out": 30,
            "transform": { "position": [0, 0] },
            "type": "solid",
            "color": "#000000"
        }"##;
        let layer: Layer = serde_json::from_str(json).unwrap();
        assert!(layer.blend_mode.is_none());
        assert!(layer.parent.is_none());
        assert!(layer.effects.is_none());
        assert!(layer.masks.is_none());
        assert!(!layer.adjustment);
        assert!(!layer.motion_blur);
    }
}
