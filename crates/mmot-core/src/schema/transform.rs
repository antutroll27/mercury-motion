use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::schema::{AnimatableValue, FcurveModifier, Vec2};

/// Per-layer transform properties. All fields are animatable.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Transform {
    #[serde(default = "default_center")]
    pub position: AnimatableValue<Vec2>,
    #[serde(default = "default_scale")]
    pub scale: AnimatableValue<Vec2>,
    #[serde(default = "default_one")]
    pub opacity: AnimatableValue<f64>,
    #[serde(default = "default_zero")]
    pub rotation: AnimatableValue<f64>,

    /// Optional F-curve modifiers applied after evaluating the opacity keyframes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub opacity_modifiers: Option<Vec<FcurveModifier>>,
    /// Optional F-curve modifiers applied after evaluating the rotation keyframes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rotation_modifiers: Option<Vec<FcurveModifier>>,
    /// Optional F-curve modifiers applied after evaluating the position keyframes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub position_modifiers: Option<Vec<FcurveModifier>>,
    /// Optional F-curve modifiers applied after evaluating the scale keyframes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scale_modifiers: Option<Vec<FcurveModifier>>,
}

fn default_center() -> AnimatableValue<Vec2> {
    AnimatableValue::Static(Vec2 { x: 0.0, y: 0.0 })
}

fn default_scale() -> AnimatableValue<Vec2> {
    AnimatableValue::Static(Vec2 { x: 1.0, y: 1.0 })
}

fn default_one() -> AnimatableValue<f64> {
    AnimatableValue::Static(1.0)
}

fn default_zero() -> AnimatableValue<f64> {
    AnimatableValue::Static(0.0)
}
