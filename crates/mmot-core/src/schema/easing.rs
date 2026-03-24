use serde::{Deserialize, Serialize};

/// Easing curve for keyframe interpolation.
/// Applied from the keyframe it is attached to toward the next keyframe.
/// Ignored on the final keyframe.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EasingValue {
    /// A named easing preset.
    Named(NamedEasing),
    /// A custom cubic Bezier curve.
    CubicBezier {
        #[serde(rename = "type")]
        kind: CubicBezierTag,
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
    },
    /// A spring physics easing (damped harmonic oscillator).
    Spring {
        #[serde(rename = "type")]
        kind: SpringTag,
        #[serde(default = "default_one")]
        mass: f64,
        #[serde(default = "default_stiffness")]
        stiffness: f64,
        #[serde(default = "default_damping")]
        damping: f64,
    },
}

impl EasingValue {
    pub fn linear() -> Self {
        Self::Named(NamedEasing::Linear)
    }
    pub fn ease_in() -> Self {
        Self::Named(NamedEasing::EaseIn)
    }
    pub fn ease_out() -> Self {
        Self::Named(NamedEasing::EaseOut)
    }
    pub fn ease_in_out() -> Self {
        Self::Named(NamedEasing::EaseInOut)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NamedEasing {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CubicBezierTag {
    CubicBezier,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpringTag {
    Spring,
}

fn default_one() -> f64 {
    1.0
}
fn default_stiffness() -> f64 {
    100.0
}
fn default_damping() -> f64 {
    10.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialise_named_easing() {
        let json = r#""ease_in""#;
        let e: EasingValue = serde_json::from_str(json).unwrap();
        assert!(matches!(e, EasingValue::Named(NamedEasing::EaseIn)));
    }

    #[test]
    fn deserialise_cubic_bezier() {
        let json = r#"{"type":"cubic_bezier","x1":0.4,"y1":0.0,"x2":0.2,"y2":1.0}"#;
        let e: EasingValue = serde_json::from_str(json).unwrap();
        match e {
            EasingValue::CubicBezier { x1, y1, x2, y2, .. } => {
                assert_eq!(x1, 0.4);
                assert_eq!(y1, 0.0);
                assert_eq!(x2, 0.2);
                assert_eq!(y2, 1.0);
            }
            _ => panic!("expected CubicBezier"),
        }
    }

    #[test]
    fn deserialise_linear_default() {
        let json = r#""linear""#;
        let e: EasingValue = serde_json::from_str(json).unwrap();
        assert!(matches!(e, EasingValue::Named(NamedEasing::Linear)));
    }

    #[test]
    fn deserialise_spring() {
        let json = r#"{"type":"spring","mass":1.0,"stiffness":170.0,"damping":26.0}"#;
        let e: EasingValue = serde_json::from_str(json).unwrap();
        match e {
            EasingValue::Spring {
                mass,
                stiffness,
                damping,
                ..
            } => {
                assert_eq!(mass, 1.0);
                assert_eq!(stiffness, 170.0);
                assert_eq!(damping, 26.0);
            }
            _ => panic!("expected Spring"),
        }
    }

    #[test]
    fn deserialise_spring_defaults() {
        let json = r#"{"type":"spring"}"#;
        let e: EasingValue = serde_json::from_str(json).unwrap();
        match e {
            EasingValue::Spring {
                mass,
                stiffness,
                damping,
                ..
            } => {
                assert_eq!(mass, 1.0);
                assert_eq!(stiffness, 100.0);
                assert_eq!(damping, 10.0);
            }
            _ => panic!("expected Spring with defaults"),
        }
    }
}
