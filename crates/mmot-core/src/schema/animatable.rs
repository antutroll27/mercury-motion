use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer, Serialize};

use crate::schema::EasingValue;

/// A 2D vector value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Vec2 {
    pub x: f64,
    pub y: f64,
}

impl From<[f64; 2]> for Vec2 {
    fn from([x, y]: [f64; 2]) -> Self {
        Self { x, y }
    }
}

/// A single keyframe.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Keyframe<T> {
    /// Frame number (integer).
    pub t: u64,
    /// Value at this keyframe.
    pub v: T,
    /// Easing from this keyframe to the next. Ignored on the final keyframe.
    #[serde(default = "EasingValue::linear")]
    pub easing: EasingValue,
}

/// A property that can be either a static value or an animated sequence of keyframes.
///
/// Disambiguation rule: if the JSON value is an array whose first element
/// is an object with a `"t"` field, it is a keyframe array. Otherwise it is
/// a static value.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub enum AnimatableValue<T> {
    Static(T),
    Animated(Vec<Keyframe<T>>),
}

impl<'de, T> Deserialize<'de> for AnimatableValue<T>
where
    T: DeserializeOwned,
{
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        use serde_json::Value;
        let raw = Value::deserialize(de)?;
        // Disambiguation: array-of-objects-with-"t" -> Animated; else -> Static
        if let Value::Array(ref arr) = raw
            && arr.first().is_some_and(|v| v.get("t").is_some())
        {
            let kfs: Vec<Keyframe<T>> =
                serde_json::from_value(raw).map_err(serde::de::Error::custom)?;
            return Ok(AnimatableValue::Animated(kfs));
        }
        let val: T = serde_json::from_value(raw).map_err(serde::de::Error::custom)?;
        Ok(AnimatableValue::Static(val))
    }
}

impl<T: Default> Default for AnimatableValue<T> {
    fn default() -> Self {
        Self::Static(T::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn static_scalar_f64() {
        let json = "1.0";
        let v: AnimatableValue<f64> = serde_json::from_str(json).unwrap();
        assert!(matches!(v, AnimatableValue::Static(x) if x == 1.0));
    }

    #[test]
    fn static_vec2() {
        let json = "[960.0, 540.0]";
        let v: AnimatableValue<Vec2> = serde_json::from_str(json).unwrap();
        match v {
            AnimatableValue::Static(Vec2 { x, y }) => {
                assert_eq!(x, 960.0);
                assert_eq!(y, 540.0);
            }
            _ => panic!("expected Static"),
        }
    }

    #[test]
    fn animated_scalar() {
        let json = r#"[{"t":0,"v":0.0,"easing":"ease_in"},{"t":15,"v":1.0}]"#;
        let v: AnimatableValue<f64> = serde_json::from_str(json).unwrap();
        match v {
            AnimatableValue::Animated(kfs) => {
                assert_eq!(kfs.len(), 2);
                assert_eq!(kfs[0].t, 0);
                assert_eq!(kfs[0].v, 0.0);
                assert_eq!(kfs[1].t, 15);
            }
            _ => panic!("expected Animated"),
        }
    }

    #[test]
    fn animated_vec2() {
        let json = r#"[{"t":10,"v":[960.0,620.0],"easing":"ease_out"},{"t":25,"v":[960.0,540.0]}]"#;
        let v: AnimatableValue<Vec2> = serde_json::from_str(json).unwrap();
        assert!(matches!(v, AnimatableValue::Animated(_)));
    }
}
