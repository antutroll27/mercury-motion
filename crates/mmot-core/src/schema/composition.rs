use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::schema::{AnimatableValue, Transform};

/// A single layer in a composition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layer {
    pub id: String,
    /// Frame this layer becomes active (inclusive).
    #[serde(rename = "in")]
    pub in_point: u64,
    /// Frame this layer becomes inactive (exclusive).
    #[serde(rename = "out")]
    pub out_point: u64,
    pub transform: Transform,
    #[serde(flatten)]
    pub content: LayerContent,
}

/// Type-specific layer content, discriminated by the `"type"` JSON field.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
}

fn default_center_align() -> TextAlign {
    TextAlign::Center
}

fn default_one_anim() -> AnimatableValue<f64> {
    AnimatableValue::Static(1.0)
}

/// Font specification for text layers.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TextAlign {
    Left,
    #[default]
    Center,
    Right,
}

/// Shape specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrokeSpec {
    pub color: String,
    pub width: f64,
}

/// A composition — an ordered list of layers (first = bottom of visual stack).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Composition {
    pub layers: Vec<Layer>,
}

/// Map of composition ID to composition.
pub type Compositions = HashMap<String, Composition>;
