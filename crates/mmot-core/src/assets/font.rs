use std::path::Path;

use crate::error::{MmotError, Result};

/// Load a font file (.ttf/.otf) from disk and return its raw bytes.
pub fn load_font(path: &Path) -> Result<Vec<u8>> {
    if !path.exists() {
        return Err(MmotError::AssetNotFound {
            path: path.to_path_buf(),
        });
    }
    std::fs::read(path).map_err(|e| MmotError::FontLoad(format!("{}: {e}", path.display())))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_nonexistent_font_returns_error() {
        let result = load_font(Path::new("nonexistent.ttf"));
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            MmotError::AssetNotFound { .. }
        ));
    }
}
