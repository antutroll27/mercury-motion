use crate::error::{MmotError, Result};
use crate::schema::Scene;

/// Post-deserialisation validation.
/// Checks referential integrity and value constraints that serde cannot express.
pub fn validate(scene: &Scene) -> Result<()> {
    // Root composition must exist
    if !scene.compositions.contains_key(&scene.meta.root) {
        return Err(MmotError::Parse {
            message: format!(
                "root composition '{}' is not defined in compositions",
                scene.meta.root
            ),
            pointer: "/meta/root".into(),
        });
    }

    // All composition references in layers must resolve
    for (comp_name, comp) in &scene.compositions {
        for (i, layer) in comp.layers.iter().enumerate() {
            if let crate::schema::LayerContent::Composition { id } = &layer.content
                && !scene.compositions.contains_key(id.as_str())
            {
                return Err(MmotError::Parse {
                    message: format!("composition reference '{}' not defined", id),
                    pointer: format!(
                        "/compositions/{}/layers/{}/composition_id",
                        comp_name, i
                    ),
                });
            }
            // in_point must be < out_point
            if layer.in_point >= layer.out_point {
                return Err(MmotError::Parse {
                    message: format!(
                        "layer '{}': in ({}) must be less than out ({})",
                        layer.id, layer.in_point, layer.out_point
                    ),
                    pointer: format!("/compositions/{}/layers/{}/in", comp_name, i),
                });
            }
        }
    }

    // fps must be positive
    if scene.meta.fps <= 0.0 {
        return Err(MmotError::Parse {
            message: "fps must be > 0".into(),
            pointer: "/meta/fps".into(),
        });
    }

    Ok(())
}
