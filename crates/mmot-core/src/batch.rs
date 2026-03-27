use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::error::{MmotError, Result};
use crate::pipeline::{OutputFormat, RenderBackend, RenderOptions};

/// Options for batch rendering.
pub struct BatchOptions {
    pub template_json: String,
    pub output_dir: PathBuf,
    pub format: OutputFormat,
    pub quality: u8,
    pub concurrency: Option<usize>,
}

/// Summary of a batch render run.
pub struct BatchResult {
    pub total: usize,
    pub rendered: usize,
    pub failed: Vec<(usize, String)>, // (row index, error message)
}

/// Parse a CSV file into a list of prop maps.
///
/// Expects the first line to be column headers. Each subsequent non-empty line
/// becomes one `HashMap<column_name, cell_value>`. Values are trimmed and
/// outer double-quotes are stripped.
pub fn parse_csv(path: &Path) -> Result<Vec<HashMap<String, String>>> {
    let content = std::fs::read_to_string(path).map_err(MmotError::Io)?;
    let mut lines = content.lines();

    let header = lines.next().ok_or_else(|| MmotError::Parse {
        message: "CSV file is empty".into(),
        pointer: String::new(),
    })?;

    let columns: Vec<&str> = header
        .split(',')
        .map(|s| s.trim().trim_matches('"'))
        .collect();

    let mut rows = Vec::new();
    for line in lines {
        if line.trim().is_empty() {
            continue;
        }
        let values: Vec<&str> = line
            .split(',')
            .map(|s| s.trim().trim_matches('"'))
            .collect();
        let mut map = HashMap::new();
        for (i, col) in columns.iter().enumerate() {
            let val = values.get(i).copied().unwrap_or("");
            map.insert(col.to_string(), val.to_string());
        }
        rows.push(map);
    }

    Ok(rows)
}

/// Parse a JSON array file into a list of prop maps.
///
/// Expects the file to contain a top-level JSON array of objects. Each object's
/// keys become prop names; values are stringified.
pub fn parse_json_data(path: &Path) -> Result<Vec<HashMap<String, String>>> {
    let content = std::fs::read_to_string(path).map_err(MmotError::Io)?;
    let array: Vec<serde_json::Value> =
        serde_json::from_str(&content).map_err(|e| MmotError::Parse {
            message: format!("data file JSON error: {e}"),
            pointer: String::new(),
        })?;

    let mut rows = Vec::new();
    for item in array {
        let mut map = HashMap::new();
        if let serde_json::Value::Object(obj) = item {
            for (key, value) in obj {
                let str_val = match value {
                    serde_json::Value::String(s) => s,
                    other => other.to_string(),
                };
                map.insert(key, str_val);
            }
        }
        rows.push(map);
    }

    Ok(rows)
}

/// Render a batch of videos from a template + data rows.
///
/// For each row the template's `${key}` placeholders are substituted and a
/// separate video file is rendered into `opts.output_dir`. The output filename
/// is derived from the row's `name`, `id`, or `title` field (falling back to
/// a zero-padded index).
pub fn render_batch(
    opts: BatchOptions,
    data_rows: &[HashMap<String, String>],
    progress: Option<Box<dyn Fn(usize, usize) + Send + Sync>>,
) -> Result<BatchResult> {
    std::fs::create_dir_all(&opts.output_dir).map_err(MmotError::Io)?;

    let mut result = BatchResult {
        total: data_rows.len(),
        rendered: 0,
        failed: Vec::new(),
    };

    for (i, row) in data_rows.iter().enumerate() {
        // Substitute props in template
        let json = crate::props::substitute(&opts.template_json, row);

        // Generate output filename from row data or index
        let name = row
            .get("name")
            .or_else(|| row.get("id"))
            .or_else(|| row.get("title"))
            .cloned()
            .unwrap_or_else(|| format!("{:04}", i));

        // Sanitize filename
        let safe_name: String = name
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '-' || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .collect();

        let ext = match opts.format {
            OutputFormat::Mp4 => "mp4",
            OutputFormat::Gif => "gif",
            OutputFormat::Webm => "webm",
        };
        let output_path = opts.output_dir.join(format!("{safe_name}.{ext}"));

        let render_opts = RenderOptions {
            output_path,
            format: opts.format.clone(),
            quality: opts.quality,
            frame_range: None,
            concurrency: opts.concurrency,
            backend: RenderBackend::Cpu,
            include_audio: false,
        };

        match crate::pipeline::render_scene(&json, render_opts, None) {
            Ok(()) => {
                result.rendered += 1;
            }
            Err(e) => {
                result.failed.push((i, e.to_string()));
            }
        }

        if let Some(ref prog) = progress {
            prog(i + 1, data_rows.len());
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn parse_csv_basic() {
        let dir = tempfile::tempdir().expect("tempdir");
        let csv_path = dir.path().join("data.csv");
        {
            let mut f = std::fs::File::create(&csv_path).expect("create csv");
            writeln!(f, "title,bg_color").expect("write");
            writeln!(f, "Hello World,#003049").expect("write");
            writeln!(f, "Mercury Motion,#1a1a2e").expect("write");
        }

        let rows = parse_csv(&csv_path).expect("parse_csv");
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].get("title").map(String::as_str), Some("Hello World"));
        assert_eq!(rows[0].get("bg_color").map(String::as_str), Some("#003049"));
        assert_eq!(
            rows[1].get("title").map(String::as_str),
            Some("Mercury Motion")
        );
    }

    #[test]
    fn parse_csv_empty_file_error() {
        let dir = tempfile::tempdir().expect("tempdir");
        let csv_path = dir.path().join("empty.csv");
        std::fs::File::create(&csv_path).expect("create");

        let result = parse_csv(&csv_path);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("empty"));
    }

    #[test]
    fn parse_json_data_basic() {
        let dir = tempfile::tempdir().expect("tempdir");
        let json_path = dir.path().join("data.json");
        std::fs::write(
            &json_path,
            r##"[{"title":"Hello","bg_color":"#003049"},{"title":"World","bg_color":"#1a1a2e"}]"##,
        )
        .expect("write");

        let rows = parse_json_data(&json_path).expect("parse_json_data");
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].get("title").map(String::as_str), Some("Hello"));
        assert_eq!(rows[1].get("bg_color").map(String::as_str), Some("#1a1a2e"));
    }

    #[test]
    fn parse_json_data_numeric_values_stringified() {
        let dir = tempfile::tempdir().expect("tempdir");
        let json_path = dir.path().join("nums.json");
        std::fs::write(&json_path, r#"[{"price":9.99,"count":42}]"#).expect("write");

        let rows = parse_json_data(&json_path).expect("parse");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].get("price").map(String::as_str), Some("9.99"));
        assert_eq!(rows[0].get("count").map(String::as_str), Some("42"));
    }

    #[test]
    fn parse_csv_skips_blank_lines() {
        let dir = tempfile::tempdir().expect("tempdir");
        let csv_path = dir.path().join("blanks.csv");
        std::fs::write(&csv_path, "name\nAlice\n\n  \nBob\n").expect("write");

        let rows = parse_csv(&csv_path).expect("parse");
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].get("name").map(String::as_str), Some("Alice"));
        assert_eq!(rows[1].get("name").map(String::as_str), Some("Bob"));
    }
}
