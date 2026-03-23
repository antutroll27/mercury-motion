mod validate;

use crate::error::{MmotError, Result};
use crate::schema::Scene;

/// Parse and validate a `.mmot.json` string into a `Scene`.
///
/// Returns `MmotError::Parse` with a JSON pointer path on failure.
pub fn parse(json: &str) -> Result<Scene> {
    let deserializer = &mut serde_json::Deserializer::from_str(json);
    let scene: Scene =
        serde_path_to_error::deserialize(deserializer).map_err(|e| MmotError::Parse {
            message: e.inner().to_string(),
            pointer: e.path().to_string(),
        })?;
    validate::validate(&scene)?;
    Ok(scene)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_minimal() {
        let json = include_str!("../../../../tests/fixtures/valid/minimal.mmot.json");
        let scene = parse(json).unwrap();
        assert_eq!(scene.meta.name, "Minimal");
        assert_eq!(scene.meta.duration, 30);
    }

    #[test]
    fn parse_missing_root_returns_error() {
        let json = include_str!("../../../../tests/fixtures/invalid/missing_root.mmot.json");
        let err = parse(json).unwrap_err();
        assert!(matches!(err, crate::error::MmotError::Parse { .. }));
        let msg = err.to_string();
        assert!(msg.contains("nonexistent_composition") || msg.contains("root"));
    }

    #[test]
    fn parse_bad_json_returns_error() {
        let err = parse("{not valid json}").unwrap_err();
        assert!(matches!(err, crate::error::MmotError::Parse { .. }));
    }
}
