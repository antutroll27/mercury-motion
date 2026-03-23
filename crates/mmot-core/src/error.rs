use std::path::PathBuf;

/// All errors produced by the mmot-core library.
#[derive(thiserror::Error, Debug)]
pub enum MmotError {
    /// JSON parse or schema validation error.
    /// `pointer` is a JSON Pointer path.
    #[error("parse error at {pointer}: {message}")]
    Parse { message: String, pointer: String },

    /// A required prop was not provided and has no default.
    #[error("missing required prop: '{prop}' has no default and was not provided via --props")]
    MissingRequiredProp { prop: String },

    /// A prop value has the wrong type.
    #[error("prop type mismatch: '{prop}' expected {expected}, got {got}")]
    PropTypeMismatch {
        prop: String,
        expected: String,
        got: String,
    },

    /// An asset file could not be found at the resolved path.
    #[error("asset not found: {path}")]
    AssetNotFound { path: PathBuf },

    /// An asset file could not be decoded.
    #[error("asset decode failed ({path}): {reason}")]
    AssetDecode { path: PathBuf, reason: String },

    /// Generic asset load error.
    #[error("asset load error: {0}")]
    AssetLoad(String),

    /// Frame rendering failed.
    #[error("render failed at frame {frame}: {reason}")]
    RenderFailed { frame: u64, reason: String },

    /// Encoding error.
    #[error("encoder error: {0}")]
    Encoder(String),

    /// IO error (file read/write).
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

/// Convenience alias.
pub type Result<T> = std::result::Result<T, MmotError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_error_includes_pointer() {
        let e = MmotError::Parse {
            message: "bad value".into(),
            pointer: "/compositions/main/layers/0/type".into(),
        };
        let msg = e.to_string();
        assert!(msg.contains("/compositions/main/layers/0/type"));
        assert!(msg.contains("bad value"));
    }

    #[test]
    fn asset_not_found_includes_path() {
        let e = MmotError::AssetNotFound {
            path: PathBuf::from("./assets/logo.png"),
        };
        assert!(e.to_string().contains("logo.png"));
    }
}
