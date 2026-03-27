use std::path::{Path, PathBuf};

use crate::error::{MmotError, Result};
use crate::pipeline;

/// Result of a visual regression test run.
#[derive(Debug)]
pub struct TestResult {
    pub scene_path: PathBuf,
    pub frames_tested: usize,
    pub frames_passed: usize,
    pub failures: Vec<FrameFailure>,
}

/// Details about a single frame that failed visual comparison.
#[derive(Debug)]
pub struct FrameFailure {
    pub frame: u64,
    pub pixel_diff_percent: f64,
    pub golden_path: PathBuf,
    pub actual_path: PathBuf,
}

impl TestResult {
    /// Returns true if all tested frames matched their golden references.
    pub fn passed(&self) -> bool {
        self.failures.is_empty()
    }
}

/// Run visual regression test on a `.mmot.json` scene.
///
/// Renders the specified `frames` and compares each against golden reference
/// PNGs in `golden_dir`. Pixels that differ by more than 2 per channel are
/// counted as different; if the percentage of different pixels exceeds
/// `tolerance` the frame is reported as a failure.
///
/// Returns a `TestResult` summarising pass/fail for each frame.
pub fn run_visual_test(
    json: &str,
    scene_path: &Path,
    golden_dir: &Path,
    frames: &[u64],
    tolerance: f64,
) -> Result<TestResult> {
    let mut result = TestResult {
        scene_path: scene_path.to_path_buf(),
        frames_tested: frames.len(),
        frames_passed: 0,
        failures: Vec::new(),
    };

    for &frame in frames {
        let (w, h, rgba) = pipeline::render_single_frame(json, frame)?;

        let golden_path = golden_dir.join(format!("frame-{frame:03}.png"));

        if !golden_path.exists() {
            result.failures.push(FrameFailure {
                frame,
                pixel_diff_percent: 100.0,
                golden_path,
                actual_path: PathBuf::new(),
            });
            continue;
        }

        // Load golden image
        let golden_img = image::open(&golden_path)
            .map_err(|e| MmotError::AssetLoad(format!("golden image: {e}")))?
            .to_rgba8();

        let golden_rgba = golden_img.as_raw();

        // Dimension or buffer mismatch
        if golden_rgba.len() != rgba.len() {
            result.failures.push(FrameFailure {
                frame,
                pixel_diff_percent: 100.0,
                golden_path,
                actual_path: PathBuf::new(),
            });
            continue;
        }

        let total_pixels = rgba.len() / 4;
        let mut diff_pixels = 0u64;
        for i in (0..rgba.len()).step_by(4) {
            let dr = (rgba[i] as i32 - golden_rgba[i] as i32).unsigned_abs();
            let dg = (rgba[i + 1] as i32 - golden_rgba[i + 1] as i32).unsigned_abs();
            let db = (rgba[i + 2] as i32 - golden_rgba[i + 2] as i32).unsigned_abs();
            let da = (rgba[i + 3] as i32 - golden_rgba[i + 3] as i32).unsigned_abs();
            if dr > 2 || dg > 2 || db > 2 || da > 2 {
                diff_pixels += 1;
            }
        }

        let diff_percent = if total_pixels > 0 {
            (diff_pixels as f64 / total_pixels as f64) * 100.0
        } else {
            0.0
        };

        if diff_percent > tolerance {
            // Save actual frame for visual comparison
            let actual_path = golden_dir.join(format!("frame-{frame:03}-actual.png"));
            if let Some(img) = image::RgbaImage::from_raw(w, h, rgba) {
                let _ = img.save(&actual_path);
            }

            result.failures.push(FrameFailure {
                frame,
                pixel_diff_percent: diff_percent,
                golden_path,
                actual_path,
            });
        } else {
            result.frames_passed += 1;
        }
    }

    Ok(result)
}

/// Generate (or overwrite) golden reference frames for a `.mmot.json` scene.
///
/// Renders each frame in `frames` and saves as `frame-NNN.png` in `golden_dir`.
/// Returns the number of frames written.
pub fn update_goldens(json: &str, golden_dir: &Path, frames: &[u64]) -> Result<usize> {
    std::fs::create_dir_all(golden_dir)?;

    for &frame in frames {
        let (w, h, rgba) = pipeline::render_single_frame(json, frame)?;
        let img = image::RgbaImage::from_raw(w, h, rgba).ok_or_else(|| {
            MmotError::RenderFailed {
                frame,
                reason: "invalid RGBA dimensions".into(),
            }
        })?;
        let path = golden_dir.join(format!("frame-{frame:03}.png"));
        img.save(&path)
            .map_err(|e| MmotError::AssetLoad(format!("save golden: {e}")))?;
    }

    Ok(frames.len())
}

/// Compute default test frames for a scene: first, middle, and last.
///
/// If duration is 0, returns an empty vec. If duration is 1, returns `[0]`.
/// If duration is 2, returns `[0, 1]`.
pub fn default_frames(duration_frames: u64) -> Vec<u64> {
    match duration_frames {
        0 => Vec::new(),
        1 => vec![0],
        2 => vec![0, 1],
        n => vec![0, n / 2, n - 1],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_update_goldens_creates_files() {
        let json = include_str!("../../../tests/fixtures/valid/minimal.mmot.json");
        let tmp = TempDir::new().expect("tempdir");
        let golden_dir = tmp.path().join("goldens");

        let count = update_goldens(json, &golden_dir, &[0]).expect("update_goldens");
        assert_eq!(count, 1);
        assert!(golden_dir.join("frame-000.png").exists());
    }

    #[test]
    fn test_visual_test_passes_with_matching_frames() {
        let json = include_str!("../../../tests/fixtures/valid/minimal.mmot.json");
        let tmp = TempDir::new().expect("tempdir");
        let golden_dir = tmp.path().join("goldens");

        // Generate golden references first
        update_goldens(json, &golden_dir, &[0]).expect("update_goldens");

        // Now test — should pass since we just generated them
        let result = run_visual_test(
            json,
            Path::new("minimal.mmot.json"),
            &golden_dir,
            &[0],
            0.1,
        )
        .expect("run_visual_test");

        assert!(result.passed());
        assert_eq!(result.frames_tested, 1);
        assert_eq!(result.frames_passed, 1);
        assert!(result.failures.is_empty());
    }

    #[test]
    fn test_visual_test_fails_with_missing_golden() {
        let json = include_str!("../../../tests/fixtures/valid/minimal.mmot.json");
        let tmp = TempDir::new().expect("tempdir");
        let golden_dir = tmp.path().join("goldens");
        std::fs::create_dir_all(&golden_dir).expect("mkdir");

        // No golden files exist, so every frame should fail
        let result = run_visual_test(
            json,
            Path::new("minimal.mmot.json"),
            &golden_dir,
            &[0],
            0.1,
        )
        .expect("run_visual_test");

        assert!(!result.passed());
        assert_eq!(result.failures.len(), 1);
        assert_eq!(result.failures[0].frame, 0);
        assert!((result.failures[0].pixel_diff_percent - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_default_frames() {
        assert!(default_frames(0).is_empty());
        assert_eq!(default_frames(1), vec![0]);
        assert_eq!(default_frames(2), vec![0, 1]);
        assert_eq!(default_frames(30), vec![0, 15, 29]);
        assert_eq!(default_frames(100), vec![0, 50, 99]);
    }
}
