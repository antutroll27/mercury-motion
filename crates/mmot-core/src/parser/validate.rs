use std::collections::HashSet;

use crate::error::{MmotError, Result};
use crate::schema::{AnimatableValue, Keyframe, Layer, LayerContent, PropType, Scene};

/// Post-deserialisation validation.
/// Checks referential integrity and value constraints that serde cannot express.
pub fn validate(scene: &Scene) -> Result<()> {
    validate_version(&scene.version)?;
    validate_meta(scene)?;
    validate_safe_zone(scene)?;

    for (comp_name, comp) in &scene.compositions {
        validate_unique_layer_ids(comp_name, &comp.layers)?;
        for (i, layer) in comp.layers.iter().enumerate() {
            validate_layer(scene, comp_name, i, layer)?;
        }
    }

    validate_no_circular_compositions(scene)?;
    validate_prop_defaults(scene)?;

    Ok(())
}

/// Version must be a supported version string.
fn validate_version(version: &str) -> Result<()> {
    const SUPPORTED: &[&str] = &["0.1", "1.0"];
    if !SUPPORTED.contains(&version) {
        return Err(MmotError::Parse {
            message: format!(
                "unsupported version '{}', expected one of: {}",
                version,
                SUPPORTED.join(", ")
            ),
            pointer: "/version".into(),
        });
    }
    Ok(())
}

/// fps > 0, duration > 0, root composition exists.
fn validate_meta(scene: &Scene) -> Result<()> {
    if scene.meta.fps <= 0.0 {
        return Err(MmotError::Parse {
            message: "fps must be > 0".into(),
            pointer: "/meta/fps".into(),
        });
    }
    if scene.meta.duration == 0 {
        return Err(MmotError::Parse {
            message: "duration must be > 0".into(),
            pointer: "/meta/duration".into(),
        });
    }
    if !scene.compositions.contains_key(&scene.meta.root) {
        return Err(MmotError::Parse {
            message: format!(
                "root composition '{}' is not defined in compositions",
                scene.meta.root
            ),
            pointer: "/meta/root".into(),
        });
    }
    Ok(())
}

/// safe_zone must be within canvas bounds and have positive dimensions.
fn validate_safe_zone(scene: &Scene) -> Result<()> {
    if let Some(ref sz) = scene.meta.safe_zone {
        if sz.x < 0.0 {
            return Err(MmotError::Parse {
                message: "safe_zone x must be >= 0".into(),
                pointer: "/meta/safe_zone/x".into(),
            });
        }
        if sz.y < 0.0 {
            return Err(MmotError::Parse {
                message: "safe_zone y must be >= 0".into(),
                pointer: "/meta/safe_zone/y".into(),
            });
        }
        if sz.width <= 0.0 {
            return Err(MmotError::Parse {
                message: "safe_zone width must be > 0".into(),
                pointer: "/meta/safe_zone/width".into(),
            });
        }
        if sz.height <= 0.0 {
            return Err(MmotError::Parse {
                message: "safe_zone height must be > 0".into(),
                pointer: "/meta/safe_zone/height".into(),
            });
        }
        if sz.x + sz.width > scene.meta.width as f64 {
            return Err(MmotError::Parse {
                message: format!(
                    "safe_zone exceeds canvas width: x({}) + width({}) > {}",
                    sz.x, sz.width, scene.meta.width
                ),
                pointer: "/meta/safe_zone".into(),
            });
        }
        if sz.y + sz.height > scene.meta.height as f64 {
            return Err(MmotError::Parse {
                message: format!(
                    "safe_zone exceeds canvas height: y({}) + height({}) > {}",
                    sz.y, sz.height, scene.meta.height
                ),
                pointer: "/meta/safe_zone".into(),
            });
        }
    }
    Ok(())
}

/// No two layers in the same composition may share an ID.
fn validate_unique_layer_ids(comp_name: &str, layers: &[Layer]) -> Result<()> {
    let mut seen = HashSet::new();
    for (i, layer) in layers.iter().enumerate() {
        if !seen.insert(&layer.id) {
            return Err(MmotError::Parse {
                message: format!("duplicate layer id '{}'", layer.id),
                pointer: format!("/compositions/{}/layers/{}/id", comp_name, i),
            });
        }
    }
    Ok(())
}

/// Per-layer validation: in < out, composition refs exist, font weight, keyframe order.
fn validate_layer(scene: &Scene, comp_name: &str, i: usize, layer: &Layer) -> Result<()> {
    let pointer_prefix = format!("/compositions/{}/layers/{}", comp_name, i);

    // in_point must be < out_point
    if layer.in_point >= layer.out_point {
        return Err(MmotError::Parse {
            message: format!(
                "layer '{}': in ({}) must be less than out ({})",
                layer.id, layer.in_point, layer.out_point
            ),
            pointer: format!("{}/in", pointer_prefix),
        });
    }

    // Composition references must resolve
    if let LayerContent::Composition { id } = &layer.content
        && !scene.compositions.contains_key(id.as_str())
    {
        return Err(MmotError::Parse {
            message: format!("composition reference '{}' not defined", id),
            pointer: format!("{}/composition_id", pointer_prefix),
        });
    }

    // Font weight: 100-900 in steps of 100
    if let LayerContent::Text { font, .. } = &layer.content
        && (font.weight < 100 || font.weight > 900 || font.weight % 100 != 0)
    {
        return Err(MmotError::Parse {
            message: format!(
                "font weight {} is invalid; must be 100-900 in steps of 100",
                font.weight
            ),
            pointer: format!("{}/font/weight", pointer_prefix),
        });
    }

    // Validate ascending keyframe order in transform properties
    validate_keyframes_ascending(&layer.transform.position, &format!("{}/transform/position", pointer_prefix))?;
    validate_keyframes_ascending(&layer.transform.scale, &format!("{}/transform/scale", pointer_prefix))?;
    validate_keyframes_ascending(&layer.transform.opacity, &format!("{}/transform/opacity", pointer_prefix))?;
    validate_keyframes_ascending(&layer.transform.rotation, &format!("{}/transform/rotation", pointer_prefix))?;

    // Also check volume keyframes on audio layers
    if let LayerContent::Audio { volume, .. } = &layer.content {
        validate_keyframes_ascending(volume, &format!("{}/volume", pointer_prefix))?;
    }

    Ok(())
}

/// Keyframe `t` values must be in strictly ascending order.
fn validate_keyframes_ascending<T>(value: &AnimatableValue<T>, pointer: &str) -> Result<()> {
    if let AnimatableValue::Animated(kfs) = value {
        validate_keyframe_vec_ascending(kfs, pointer)?;
    }
    Ok(())
}

fn validate_keyframe_vec_ascending<T>(kfs: &[Keyframe<T>], pointer: &str) -> Result<()> {
    for window in kfs.windows(2) {
        if window[1].t <= window[0].t {
            return Err(MmotError::Parse {
                message: format!(
                    "keyframe times must be strictly ascending, but found t={} after t={}",
                    window[1].t, window[0].t
                ),
                pointer: pointer.to_string(),
            });
        }
    }
    Ok(())
}

/// Detect circular composition references via DFS.
fn validate_no_circular_compositions(scene: &Scene) -> Result<()> {
    for comp_name in scene.compositions.keys() {
        let mut visited = HashSet::new();
        let mut stack = vec![comp_name.as_str()];
        while let Some(current) = stack.pop() {
            if !visited.insert(current) {
                return Err(MmotError::Parse {
                    message: format!(
                        "circular composition reference detected involving '{}'",
                        current
                    ),
                    pointer: format!("/compositions/{}", current),
                });
            }
            if let Some(comp) = scene.compositions.get(current) {
                for layer in &comp.layers {
                    if let LayerContent::Composition { id } = &layer.content {
                        stack.push(id.as_str());
                    }
                }
            }
        }
    }
    Ok(())
}

/// Prop defaults must match their declared type.
fn validate_prop_defaults(scene: &Scene) -> Result<()> {
    for (name, def) in &scene.props {
        if let Some(default) = &def.default {
            let ok = match def.prop_type {
                PropType::String | PropType::Color | PropType::Url => default.is_string(),
                PropType::Number => default.is_number(),
            };
            if !ok {
                return Err(MmotError::Parse {
                    message: format!(
                        "prop '{}' is declared as {:?} but default is {}",
                        name,
                        def.prop_type,
                        default
                    ),
                    pointer: format!("/props/{}/default", name),
                });
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::parser::parse;

    #[test]
    fn rejects_unsupported_version() {
        let json = r##"{
            "version": "99.0",
            "meta": {"name":"T","width":640,"height":360,"fps":30,"duration":30,"background":"#000000","root":"main"},
            "compositions": {"main": {"layers": []}}
        }"##;
        let err = parse(json).unwrap_err();
        assert!(err.to_string().contains("unsupported version"));
    }

    #[test]
    fn rejects_zero_duration() {
        let json = r##"{
            "version": "1.0",
            "meta": {"name":"T","width":640,"height":360,"fps":30,"duration":0,"background":"#000000","root":"main"},
            "compositions": {"main": {"layers": []}}
        }"##;
        let err = parse(json).unwrap_err();
        assert!(err.to_string().contains("duration must be > 0"));
    }

    #[test]
    fn rejects_duplicate_layer_ids() {
        let json = r##"{
            "version": "1.0",
            "meta": {"name":"T","width":640,"height":360,"fps":30,"duration":30,"background":"#000000","root":"main"},
            "compositions": {"main": {"layers": [
                {"id":"a","type":"solid","in":0,"out":10,"color":"#ff0000","transform":{"position":[0,0],"scale":[1,1],"opacity":1.0,"rotation":0.0}},
                {"id":"a","type":"solid","in":0,"out":10,"color":"#00ff00","transform":{"position":[0,0],"scale":[1,1],"opacity":1.0,"rotation":0.0}}
            ]}}
        }"##;
        let err = parse(json).unwrap_err();
        assert!(err.to_string().contains("duplicate layer id"));
    }

    #[test]
    fn rejects_bad_font_weight() {
        let json = r##"{
            "version": "1.0",
            "meta": {"name":"T","width":640,"height":360,"fps":30,"duration":30,"background":"#000000","root":"main"},
            "compositions": {"main": {"layers": [
                {"id":"t","type":"text","in":0,"out":30,"text":"Hi","font":{"family":"Arial","weight":450},
                 "transform":{"position":[320,180],"scale":[1,1],"opacity":1.0,"rotation":0.0}}
            ]}}
        }"##;
        let err = parse(json).unwrap_err();
        assert!(err.to_string().contains("font weight"));
    }

    #[test]
    fn rejects_non_ascending_keyframes() {
        let json = r##"{
            "version": "1.0",
            "meta": {"name":"T","width":640,"height":360,"fps":30,"duration":30,"background":"#000000","root":"main"},
            "compositions": {"main": {"layers": [
                {"id":"bg","type":"solid","in":0,"out":30,"color":"#ff0000",
                 "transform":{"position":[0,0],"scale":[1,1],
                   "opacity":[{"t":10,"v":1.0},{"t":5,"v":0.0}],
                   "rotation":0.0}}
            ]}}
        }"##;
        let err = parse(json).unwrap_err();
        assert!(err.to_string().contains("strictly ascending"));
    }

    #[test]
    fn rejects_circular_composition() {
        let json = r##"{
            "version": "1.0",
            "meta": {"name":"T","width":640,"height":360,"fps":30,"duration":30,"background":"#000000","root":"a"},
            "compositions": {
                "a": {"layers": [
                    {"id":"ref","type":"composition","composition_id":"b","in":0,"out":30,
                     "transform":{"position":[0,0],"scale":[1,1],"opacity":1.0,"rotation":0.0}}
                ]},
                "b": {"layers": [
                    {"id":"ref","type":"composition","composition_id":"a","in":0,"out":30,
                     "transform":{"position":[0,0],"scale":[1,1],"opacity":1.0,"rotation":0.0}}
                ]}
            }
        }"##;
        let err = parse(json).unwrap_err();
        assert!(err.to_string().contains("circular"));
    }

    #[test]
    fn accepts_valid_version_0_1() {
        let json = r##"{
            "version": "0.1",
            "meta": {"name":"T","width":640,"height":360,"fps":30,"duration":30,"background":"#000000","root":"main"},
            "compositions": {"main": {"layers": []}}
        }"##;
        assert!(parse(json).is_ok());
    }

    #[test]
    fn accepts_valid_safe_zone() {
        let json = r##"{
            "version": "1.0",
            "meta": {"name":"T","width":1920,"height":1080,"fps":30,"duration":30,"background":"#000000","root":"main",
                     "safe_zone":{"x":560,"y":140,"width":800,"height":800}},
            "compositions": {"main": {"layers": []}}
        }"##;
        assert!(parse(json).is_ok());
    }

    #[test]
    fn rejects_safe_zone_exceeds_canvas() {
        let json = r##"{
            "version": "1.0",
            "meta": {"name":"T","width":640,"height":360,"fps":30,"duration":30,"background":"#000000","root":"main",
                     "safe_zone":{"x":0,"y":0,"width":800,"height":400}},
            "compositions": {"main": {"layers": []}}
        }"##;
        let err = parse(json).unwrap_err();
        assert!(err.to_string().contains("safe_zone exceeds canvas width"));
    }

    #[test]
    fn rejects_safe_zone_negative_x() {
        let json = r##"{
            "version": "1.0",
            "meta": {"name":"T","width":640,"height":360,"fps":30,"duration":30,"background":"#000000","root":"main",
                     "safe_zone":{"x":-10,"y":0,"width":100,"height":100}},
            "compositions": {"main": {"layers": []}}
        }"##;
        let err = parse(json).unwrap_err();
        assert!(err.to_string().contains("safe_zone x must be >= 0"));
    }

    #[test]
    fn rejects_safe_zone_zero_width() {
        let json = r##"{
            "version": "1.0",
            "meta": {"name":"T","width":640,"height":360,"fps":30,"duration":30,"background":"#000000","root":"main",
                     "safe_zone":{"x":0,"y":0,"width":0,"height":100}},
            "compositions": {"main": {"layers": []}}
        }"##;
        let err = parse(json).unwrap_err();
        assert!(err.to_string().contains("safe_zone width must be > 0"));
    }
}
