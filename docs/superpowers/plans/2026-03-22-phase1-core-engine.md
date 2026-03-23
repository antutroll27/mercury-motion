# Mercury-Motion Phase 1: Core Engine — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the Rust core engine — JSON parser, keyframe evaluator, Skia CPU renderer (solid/image/text/video layers), rav1e MP4 encoder, and the `mmot render` / `mmot validate` CLI commands.

**Architecture:** A pure Rust library crate (`mmot`) with a thin CLI binary on top. The pipeline is strictly linear: parse `.mmot.json` → resolve assets → evaluate keyframes per frame (parallel via rayon) → render frame to RGBA (Skia CPU) → encode frames to MP4 (rav1e). Each stage is a separate module with a clean interface; nothing leaks across boundaries.

**Tech Stack:** Rust 2021 edition, `skia-safe` (2D rendering + text), `rav1e` (AV1 encoder), `serde`/`serde_json`/`schemars` (JSON), `rayon` (parallelism), `image` (asset decode), `thiserror`/`anyhow` (errors), `clap` (CLI).

---

## Multi-Plan Note

This is Plan 1 of 4. Subsequent plans:
- **Plan 2** — Full Renderer (audio, precomps, Lottie, shapes, props/templates, GPU backend)
- **Plan 3** — Editor MVP (Tauri + React 19 + Zustand desktop editor)
- **Plan 4** — Editor UI/UX (keyframe curve editor, animations, design polish)

Each plan produces independently working, testable software.

---

## File Structure

```
mercury-motion/
├── Cargo.toml                          # Workspace root
├── Cargo.lock
├── CLAUDE.md                           # Agent instructions (read first)
├── .github/
│   └── workflows/
│       └── ci.yml                      # GitHub Actions: test on Win/Mac/Linux
│
├── crates/
│   └── mmot-core/                      # Core library crate
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs                  # Public API surface
│           ├── error.rs                # MmotError enum (thiserror)
│           │
│           ├── schema/                 # JSON types (serde structs)
│           │   ├── mod.rs
│           │   ├── scene.rs            # Scene, Meta, Props root types
│           │   ├── composition.rs      # Composition, Layer, LayerContent
│           │   ├── transform.rs        # Transform, AnimatableValue<T>
│           │   └── easing.rs           # EasingValue enum
│           │
│           ├── parser/
│           │   ├── mod.rs              # parse() entry point
│           │   └── validate.rs         # Post-deserialise validation
│           │
│           ├── evaluator/
│           │   ├── mod.rs              # evaluate(scene, frame) → FrameScene
│           │   ├── interpolate.rs      # Keyframe binary search + lerp
│           │   └── easing.rs           # Easing curve maths
│           │
│           ├── assets/
│           │   ├── mod.rs              # AssetCache, resolve_assets()
│           │   ├── image.rs            # PNG/JPEG/WebP decode
│           │   └── font.rs             # Skia font loading
│           │
│           ├── renderer/
│           │   ├── mod.rs              # render(frame_scene) → RgbaFrame
│           │   ├── surface.rs          # Skia surface creation (CPU)
│           │   ├── layers.rs           # Per-layer draw dispatch
│           │   ├── solid.rs            # Draw solid layer
│           │   ├── image.rs            # Draw image layer
│           │   ├── text.rs             # Draw text layer (Skia SkShaper)
│           │   └── video.rs            # Draw video layer (ffmpeg-next frame)
│           │
│           ├── encoder/
│           │   ├── mod.rs              # Encoder trait + factory
│           │   └── mp4.rs              # rav1e + MP4 mux pipeline
│           │
│           └── pipeline.rs             # render_scene(): orchestrates all stages
│
├── crates/
│   └── mmot-cli/                       # CLI binary crate
│       ├── Cargo.toml
│       └── src/
│           └── main.rs                 # clap commands: render, validate
│
└── tests/                              # Workspace-level test fixtures (shared by all crates)
    └── fixtures/
        ├── valid/
        │   ├── minimal.mmot.json
        │   ├── text_fade.mmot.json
        │   └── image_scale.mmot.json
        └── invalid/
            ├── missing_root.mmot.json
            ├── bad_easing.mmot.json
            └── prop_type_mismatch.mmot.json

# Golden images and integration tests live INSIDE mmot-core:
# crates/mmot-core/tests/
#   ├── generate_goldens.rs             # #[ignore] generator (run once)
#   ├── golden_test.rs                  # Determinism tests
#   └── golden/
#       └── minimal/frame-000.png      # Committed reference PNGs
```

---

## Task 1: Project Scaffold

**Files:**
- Create: `Cargo.toml` (workspace)
- Create: `crates/mmot-core/Cargo.toml`
- Create: `crates/mmot-cli/Cargo.toml`
- Create: `crates/mmot-core/src/lib.rs`
- Create: `crates/mmot-cli/src/main.rs`
- Create: `CLAUDE.md`
- Create: `.github/workflows/ci.yml`

- [ ] **Step 1: Create workspace Cargo.toml**

```toml
# Cargo.toml
[workspace]
members = ["crates/mmot-core", "crates/mmot-cli"]
resolver = "2"

[workspace.dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
schemars = { version = "0.8", features = ["derive"] }
thiserror = "1"
anyhow = "1"
rayon = "1"
image = { version = "0.25", default-features = false, features = ["png", "jpeg", "webp"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

- [ ] **Step 2: Create mmot-core Cargo.toml**

```toml
# crates/mmot-core/Cargo.toml
[package]
name = "mmot-core"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { workspace = true }
serde_json = { workspace = true }
schemars = { workspace = true }
thiserror = { workspace = true }
anyhow = { workspace = true }
rayon = { workspace = true }
image = { workspace = true }
tracing = { workspace = true }
skia-safe = { version = "0.75", features = ["gl", "textlayout"] }
rav1e = "0.7"
minimp4 = "0.1"
serde-path-to-error = "0.1"

[dev-dependencies]
pretty_assertions = "1"
image = { workspace = true }
```

- [ ] **Step 3: Create mmot-cli Cargo.toml**

```toml
# crates/mmot-cli/Cargo.toml
[package]
name = "mmot"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "mmot"
path = "src/main.rs"

[dependencies]
mmot-core = { path = "../mmot-core" }
clap = { version = "4", features = ["derive"] }
anyhow = { workspace = true }
tracing = { workspace = true }
tracing-subscriber = { workspace = true }
```

- [ ] **Step 4: Create stub lib.rs**

```rust
// crates/mmot-core/src/lib.rs
pub mod error;
pub mod schema;
pub mod parser;
pub mod evaluator;
pub mod assets;
pub mod renderer;
pub mod encoder;
pub mod pipeline;
```

- [ ] **Step 5: Create stub main.rs**

```rust
// crates/mmot-cli/src/main.rs
fn main() {
    println!("mmot 0.1.0");
}
```

- [ ] **Step 6: Create CLAUDE.md**

```markdown
# Mercury-Motion — Agent Instructions

## Project Overview
Mercury-Motion (binary: `mmot`) is a Rust-native programmatic video renderer.
Videos are defined in `.mmot.json` files and rendered to MP4/WebM/GIF.

## Key Facts
- Binary name: `mmot`
- File format: `.mmot.json`
- Core crate: `crates/mmot-core`
- CLI crate: `crates/mmot-cli`
- Spec: `docs/superpowers/specs/2026-03-22-remotion-rust-design.md`
- Integrations: `docs/technical/integrations.md`

## Code Rules
- No `unwrap()` or `expect()` in library code (`mmot-core`). Use `?` and `thiserror`.
- `unwrap()` is acceptable in tests and CLI entry point only.
- All errors must be `MmotError` variants with context (JSON pointer paths for parse errors).
- Every public function must have a doc comment.
- TDD: write the test first, then the implementation.

## Commands
- Build: `cargo build`
- Test: `cargo test`
- Run CLI: `cargo run -p mmot -- render path/to/file.mmot.json`
- Lint: `cargo clippy -- -D warnings`
- Format: `cargo fmt`

## Architecture
Parse → AssetResolution → Evaluate (parallel, rayon) → Render (Skia CPU) → Encode (rav1e)
Each stage is a separate module. Stages communicate through typed structs only.
```

- [ ] **Step 7: Create CI workflow**

```yaml
# .github/workflows/ci.yml
name: CI
on: [push, pull_request]
jobs:
  test:
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - name: Install Skia build deps (Linux)
        if: runner.os == 'Linux'
        run: sudo apt-get update && sudo apt-get install -y python3 libfontconfig-dev libgl-dev libgles2-mesa-dev
      - run: cargo fmt --check
      - run: cargo clippy -- -D warnings
      - run: cargo test
```

- [ ] **Step 8: Verify project compiles**

```bash
cargo build
```
Expected: compiles with no errors (just the stubs).

- [ ] **Step 9: Commit**

```bash
git init
git add .
git commit -m "feat: scaffold cargo workspace with mmot-core and mmot-cli crates"
```

---

## Task 2: Error Types

**Files:**
- Create: `crates/mmot-core/src/error.rs`

- [ ] **Step 1: Write the test first**

```rust
// crates/mmot-core/src/error.rs (bottom of file, in #[cfg(test)])
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
            path: std::path::PathBuf::from("./assets/logo.png"),
        };
        assert!(e.to_string().contains("logo.png"));
    }
}
```

- [ ] **Step 2: Run test — expect compile failure**

```bash
cargo test -p mmot-core error
```
Expected: compile error — `MmotError` not defined yet.

- [ ] **Step 3: Implement error types**

```rust
// crates/mmot-core/src/error.rs
use std::path::PathBuf;

#[derive(thiserror::Error, Debug)]
pub enum MmotError {
    /// JSON parse or schema validation error.
    /// `pointer` is a JSON Pointer path (e.g. "/compositions/main/layers/2/transform/opacity/0/t")
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

    /// Generic asset load error (e.g. image decode failure).
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
```

- [ ] **Step 4: Run tests — expect pass**

```bash
cargo test -p mmot-core error
```
Expected: 2 tests pass.

- [ ] **Step 5: Commit**

```bash
git add crates/mmot-core/src/error.rs
git commit -m "feat: add MmotError types with thiserror"
```

---

## Task 3: Schema Types — Easing

**Files:**
- Create: `crates/mmot-core/src/schema/easing.rs`
- Create: `crates/mmot-core/src/schema/mod.rs`

- [ ] **Step 1: Write tests first**

```rust
// crates/mmot-core/src/schema/easing.rs (bottom)
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
            EasingValue::CubicBezier { x1, y1, x2, y2 } => {
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
}
```

- [ ] **Step 2: Run test — expect compile failure**

```bash
cargo test -p mmot-core schema::easing
```

- [ ] **Step 3: Implement EasingValue**

```rust
// crates/mmot-core/src/schema/easing.rs
use serde::{Deserialize, Serialize};

/// Easing curve for keyframe interpolation.
/// Applied from the keyframe it is attached to toward the next keyframe.
/// Ignored on the final keyframe.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EasingValue {
    /// A named easing preset.
    Named(NamedEasing),
    /// A custom cubic Bézier curve.
    CubicBezier {
        #[serde(rename = "type")]
        kind: CubicBezierTag, // must be "cubic_bezier"
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
    },
}

// Make match arms cleaner
impl EasingValue {
    pub fn linear() -> Self { Self::Named(NamedEasing::Linear) }
    pub fn ease_in() -> Self { Self::Named(NamedEasing::EaseIn) }
    pub fn ease_out() -> Self { Self::Named(NamedEasing::EaseOut) }
    pub fn ease_in_out() -> Self { Self::Named(NamedEasing::EaseInOut) }
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
pub enum CubicBezierTag { CubicBezier }
```

- [ ] **Step 4: Create schema/mod.rs**

```rust
// crates/mmot-core/src/schema/mod.rs
pub mod easing;
pub use easing::EasingValue;
```

- [ ] **Step 5: Run tests — expect pass**

```bash
cargo test -p mmot-core schema
```

- [ ] **Step 6: Commit**

```bash
git add crates/mmot-core/src/schema/
git commit -m "feat: add EasingValue schema type with serde"
```

---

## Task 4: Schema Types — AnimatableValue

**Files:**
- Create: `crates/mmot-core/src/schema/animatable.rs`

This is the most important type in the schema. An `AnimatableValue<T>` is either a static `T` or a `Vec<Keyframe<T>>`. The disambiguation rule: if the JSON array's first element has a `"t"` field → keyframe array; otherwise → static vector.

- [ ] **Step 1: Write tests first**

```rust
// crates/mmot-core/src/schema/animatable.rs (bottom)
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
```

- [ ] **Step 2: Run test — expect compile failure**

```bash
cargo test -p mmot-core schema::animatable
```

- [ ] **Step 3: Implement AnimatableValue**

```rust
// crates/mmot-core/src/schema/animatable.rs
use serde::{Deserialize, Deserializer, Serialize};
use crate::schema::EasingValue;

/// A 2D vector value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Vec2 {
    pub x: f64,
    pub y: f64,
}

impl From<[f64; 2]> for Vec2 {
    fn from([x, y]: [f64; 2]) -> Self { Self { x, y } }
}

/// A single keyframe.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize)]
pub enum AnimatableValue<T> {
    Static(T),
    Animated(Vec<Keyframe<T>>),
}

impl<'de, T> Deserialize<'de> for AnimatableValue<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        use serde_json::Value;
        let raw = Value::deserialize(de)?;
        // Disambiguation: array-of-objects-with-"t" → Animated; else → Static
        if let Value::Array(ref arr) = raw {
            if arr.first().map(|v| v.get("t").is_some()).unwrap_or(false) {
                let kfs: Vec<Keyframe<T>> =
                    serde_json::from_value(raw).map_err(serde::de::Error::custom)?;
                return Ok(AnimatableValue::Animated(kfs));
            }
        }
        let val: T = serde_json::from_value(raw).map_err(serde::de::Error::custom)?;
        Ok(AnimatableValue::Static(val))
    }
}

impl<T: Default> Default for AnimatableValue<T> {
    fn default() -> Self { Self::Static(T::default()) }
}
```

- [ ] **Step 4: Export from schema/mod.rs**

```rust
// Add to crates/mmot-core/src/schema/mod.rs
pub mod animatable;
pub use animatable::{AnimatableValue, Keyframe, Vec2};
```

- [ ] **Step 5: Run tests — expect pass**

```bash
cargo test -p mmot-core schema
```

- [ ] **Step 6: Commit**

```bash
git add crates/mmot-core/src/schema/animatable.rs crates/mmot-core/src/schema/mod.rs
git commit -m "feat: add AnimatableValue<T> with keyframe/static disambiguation"
```

---

## Task 5: Schema Types — Scene, Composition, Layers

**Files:**
- Create: `crates/mmot-core/src/schema/scene.rs`
- Create: `crates/mmot-core/src/schema/composition.rs`
- Create: `crates/mmot-core/src/schema/transform.rs`

- [ ] **Step 1: Write tests**

```rust
// At bottom of crates/mmot-core/src/schema/scene.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialise_minimal_scene() {
        let json = r#"{
            "version": "1.0",
            "meta": {
                "name": "Test",
                "width": 1920, "height": 1080,
                "fps": 30, "duration": 90,
                "background": "#000000",
                "root": "main"
            },
            "compositions": {
                "main": { "layers": [] }
            }
        }"#;
        let scene: Scene = serde_json::from_str(json).unwrap();
        assert_eq!(scene.meta.width, 1920);
        assert_eq!(scene.meta.fps, 30.0);
        assert_eq!(scene.meta.duration, 90);
        assert!(scene.compositions.contains_key("main"));
    }

    #[test]
    fn deserialise_solid_layer() {
        let json = r#"{
            "version": "1.0",
            "meta": {"name":"T","width":1920,"height":1080,"fps":30,"duration":30,"background":"#000","root":"main"},
            "compositions": {
                "main": {
                    "layers": [{
                        "id": "bg",
                        "type": "solid",
                        "in": 0, "out": 30,
                        "color": "#ff0000",
                        "transform": {
                            "position": [960.0, 540.0],
                            "scale": [1.0, 1.0],
                            "opacity": 1.0,
                            "rotation": 0.0
                        }
                    }]
                }
            }
        }"#;
        let scene: Scene = serde_json::from_str(json).unwrap();
        let layer = &scene.compositions["main"].layers[0];
        assert_eq!(layer.id, "bg");
        assert!(matches!(layer.content, LayerContent::Solid { .. }));
    }
}
```

- [ ] **Step 2: Run test — expect compile failure**

```bash
cargo test -p mmot-core schema::scene
```

- [ ] **Step 3: Implement Transform**

```rust
// crates/mmot-core/src/schema/transform.rs
use serde::{Deserialize, Serialize};
use crate::schema::{AnimatableValue, Vec2};

/// Per-layer transform properties. All fields are animatable.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transform {
    #[serde(default = "default_center")]
    pub position: AnimatableValue<Vec2>,
    #[serde(default = "default_scale")]
    pub scale: AnimatableValue<Vec2>,
    #[serde(default = "default_one")]
    pub opacity: AnimatableValue<f64>,
    #[serde(default = "default_zero")]
    pub rotation: AnimatableValue<f64>,
}

fn default_center() -> AnimatableValue<Vec2> {
    AnimatableValue::Static(Vec2 { x: 0.0, y: 0.0 })
}
fn default_scale() -> AnimatableValue<Vec2> {
    AnimatableValue::Static(Vec2 { x: 1.0, y: 1.0 })
}
fn default_one() -> AnimatableValue<f64> { AnimatableValue::Static(1.0) }
fn default_zero() -> AnimatableValue<f64> { AnimatableValue::Static(0.0) }
```

- [ ] **Step 4: Implement LayerContent and Layer**

```rust
// crates/mmot-core/src/schema/composition.rs
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::schema::{AnimatableValue, Transform};

/// A single layer in a composition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layer {
    pub id: String,
    /// Frame this layer becomes active (inclusive).
    #[serde(rename = "in")]
    pub in_point: u64,
    /// Frame this layer becomes inactive (exclusive).
    #[serde(rename = "out")]
    pub out_point: u64,
    pub transform: Transform,
    #[serde(flatten)]
    pub content: LayerContent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LayerContent {
    Solid { color: String },
    Image { src: String },
    Video { src: String, #[serde(default)] trim_start: f64, #[serde(default)] trim_end: Option<f64> },
    Text { text: String, font: FontSpec, #[serde(default = "default_center_align")] align: TextAlign },
    Audio { src: String, #[serde(default = "default_one_anim")] volume: AnimatableValue<f64> },
    Lottie { src: String },
    Composition { #[serde(rename = "composition_id")] id: String },
    Shape { shape: ShapeSpec },
}

fn default_center_align() -> TextAlign { TextAlign::Center }
fn default_one_anim() -> AnimatableValue<f64> { AnimatableValue::Static(1.0) }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontSpec {
    pub family: String,
    #[serde(default = "default_font_size")]
    pub size: f64,
    #[serde(default = "default_font_weight")]
    pub weight: u32,
    #[serde(default = "default_white")]
    pub color: String,
}
fn default_font_size() -> f64 { 32.0 }
fn default_font_weight() -> u32 { 400 }
fn default_white() -> String { "#ffffff".into() }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TextAlign { Left, #[default] Center, Right }

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "shape_type", rename_all = "snake_case")]
pub enum ShapeSpec {
    Rect { width: f64, height: f64, corner_radius: Option<f64>, fill: Option<String>, stroke: Option<StrokeSpec> },
    Ellipse { width: f64, height: f64, fill: Option<String>, stroke: Option<StrokeSpec> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrokeSpec { pub color: String, pub width: f64 }

/// A composition — an ordered list of layers (first = bottom of visual stack).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Composition {
    pub layers: Vec<Layer>,
}

pub type Compositions = HashMap<String, Composition>;
```

- [ ] **Step 5: Implement Scene**

```rust
// crates/mmot-core/src/schema/scene.rs
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::schema::composition::Compositions;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scene {
    pub version: String,
    pub meta: Meta,
    #[serde(default)]
    pub props: HashMap<String, PropDef>,
    pub compositions: Compositions,
    #[serde(default)]
    pub assets: Assets,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Meta {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub fps: f64,
    /// Duration in frames.
    pub duration: u64,
    #[serde(default = "default_black")]
    pub background: String,
    pub root: String,
}
fn default_black() -> String { "#000000".into() }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropDef {
    #[serde(rename = "type")]
    pub prop_type: PropType,
    pub default: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PropType { String, Color, Number, Url }

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Assets {
    #[serde(default)]
    pub fonts: Vec<FontAsset>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontAsset {
    pub id: String,
    pub src: String,
}
```

- [ ] **Step 6: Wire up schema/mod.rs**

```rust
// crates/mmot-core/src/schema/mod.rs (full replacement)
pub mod animatable;
pub mod easing;
pub mod transform;
pub mod composition;
pub mod scene;

pub use animatable::{AnimatableValue, Keyframe, Vec2};
pub use easing::EasingValue;
pub use transform::Transform;
pub use composition::{Composition, Compositions, Layer, LayerContent, FontSpec, TextAlign};
pub use scene::{Scene, Meta, PropDef, PropType, Assets};
```

- [ ] **Step 7: Run tests — expect pass**

```bash
cargo test -p mmot-core schema
```

- [ ] **Step 8: Commit**

```bash
git add crates/mmot-core/src/schema/
git commit -m "feat: add Scene/Composition/Layer schema types with serde"
```

---

## Task 6: Parser

**Files:**
- Create: `crates/mmot-core/src/parser/mod.rs`
- Create: `crates/mmot-core/src/parser/validate.rs`
- Create: `tests/fixtures/valid/minimal.mmot.json`
- Create: `tests/fixtures/invalid/missing_root.mmot.json`

- [ ] **Step 1: Create test fixtures**

```json
// tests/fixtures/valid/minimal.mmot.json
{
  "version": "1.0",
  "meta": {
    "name": "Minimal",
    "width": 640, "height": 360,
    "fps": 30, "duration": 30,
    "background": "#000000",
    "root": "main"
  },
  "compositions": {
    "main": {
      "layers": [
        {
          "id": "bg",
          "type": "solid",
          "in": 0, "out": 30,
          "color": "#1a1a2e",
          "transform": { "position": [320.0, 180.0], "scale": [1.0,1.0], "opacity": 1.0, "rotation": 0.0 }
        }
      ]
    }
  }
}
```

```json
// tests/fixtures/invalid/missing_root.mmot.json
{
  "version": "1.0",
  "meta": {
    "name": "Bad",
    "width": 640, "height": 360,
    "fps": 30, "duration": 30,
    "background": "#000000",
    "root": "nonexistent_composition"
  },
  "compositions": {
    "main": { "layers": [] }
  }
}
```

- [ ] **Step 2: Write parser tests**

```rust
// crates/mmot-core/src/parser/mod.rs (bottom)
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_minimal() {
        let json = include_str!("../../../tests/fixtures/valid/minimal.mmot.json");
        let scene = parse(json).unwrap();
        assert_eq!(scene.meta.name, "Minimal");
        assert_eq!(scene.meta.duration, 30);
    }

    #[test]
    fn parse_missing_root_returns_error() {
        let json = include_str!("../../../tests/fixtures/invalid/missing_root.mmot.json");
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
```

- [ ] **Step 3: Run test — expect compile failure**

```bash
cargo test -p mmot-core parser
```

- [ ] **Step 4: Implement parser**

```rust
// crates/mmot-core/src/parser/mod.rs
mod validate;

use crate::error::{MmotError, Result};
use crate::schema::Scene;

/// Parse and validate a `.mmot.json` string into a `Scene`.
///
/// Returns `MmotError::Parse` with a JSON pointer path on failure.
pub fn parse(json: &str) -> Result<Scene> {
    let deserializer = &mut serde_json::Deserializer::from_str(json);
    let scene: Scene = serde_path_to_error::deserialize(deserializer)
        .map_err(|e| MmotError::Parse {
            message: e.inner().to_string(),
            pointer: e.path().to_string(),
        })?;
    validate::validate(&scene)?;
    Ok(scene)
}
```

```rust
// crates/mmot-core/src/parser/validate.rs
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
            if let crate::schema::LayerContent::Composition { id } = &layer.content {
                if !scene.compositions.contains_key(id.as_str()) {
                    return Err(MmotError::Parse {
                        message: format!("composition reference '{}' not defined", id),
                        pointer: format!("/compositions/{}/layers/{}/composition_id", comp_name, i),
                    });
                }
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
```

- [ ] **Step 5: Run tests — expect pass**

```bash
cargo test -p mmot-core parser
```

- [ ] **Step 6: Commit**

```bash
git add crates/mmot-core/src/parser/ tests/fixtures/
git commit -m "feat: add JSON parser with post-deserialise validation"
```

---

## Task 7: Keyframe Evaluator

**Files:**
- Create: `crates/mmot-core/src/evaluator/easing.rs`
- Create: `crates/mmot-core/src/evaluator/interpolate.rs`
- Create: `crates/mmot-core/src/evaluator/mod.rs`

This is the mathematical core of the engine. It must be correct. Test every easing function and every boundary condition.

- [ ] **Step 1: Write easing function tests**

```rust
// crates/mmot-core/src/evaluator/easing.rs (bottom)
#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f64, b: f64) -> bool { (a - b).abs() < 1e-6 }

    #[test]
    fn linear_midpoint() {
        assert!(approx(apply(EasingKind::Linear, 0.5), 0.5));
    }

    #[test]
    fn linear_endpoints() {
        assert!(approx(apply(EasingKind::Linear, 0.0), 0.0));
        assert!(approx(apply(EasingKind::Linear, 1.0), 1.0));
    }

    #[test]
    fn ease_in_starts_slow() {
        // ease_in: progress at t=0.25 should be less than 0.25
        let p = apply(EasingKind::EaseIn, 0.25);
        assert!(p < 0.25, "ease_in should start slow: got {}", p);
    }

    #[test]
    fn ease_out_ends_slow() {
        // ease_out: progress at t=0.75 should be greater than 0.75
        let p = apply(EasingKind::EaseOut, 0.75);
        assert!(p > 0.75, "ease_out should end slow: got {}", p);
    }

    #[test]
    fn cubic_bezier_identity() {
        // cubic_bezier(0,0,1,1) should be equivalent to linear
        let p = cubic_bezier(0.0, 0.0, 1.0, 1.0, 0.5);
        assert!(approx(p, 0.5), "expected ~0.5, got {}", p);
    }

    #[test]
    fn all_easings_start_and_end_at_zero_and_one() {
        for kind in [EasingKind::Linear, EasingKind::EaseIn, EasingKind::EaseOut, EasingKind::EaseInOut] {
            assert!(approx(apply(kind, 0.0), 0.0));
            assert!(approx(apply(kind, 1.0), 1.0));
        }
    }
}
```

- [ ] **Step 2: Implement easing functions**

```rust
// crates/mmot-core/src/evaluator/easing.rs

/// Normalised easing input/output: both in [0.0, 1.0].
#[derive(Debug, Clone, Copy)]
pub enum EasingKind {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    CubicBezier { x1: f64, y1: f64, x2: f64, y2: f64 },
}

/// Apply an easing function to a normalised `t` in [0.0, 1.0].
pub fn apply(kind: EasingKind, t: f64) -> f64 {
    let t = t.clamp(0.0, 1.0);
    match kind {
        EasingKind::Linear => t,
        EasingKind::EaseIn => cubic_bezier(0.42, 0.0, 1.0, 1.0, t),
        EasingKind::EaseOut => cubic_bezier(0.0, 0.0, 0.58, 1.0, t),
        EasingKind::EaseInOut => cubic_bezier(0.42, 0.0, 0.58, 1.0, t),
        EasingKind::CubicBezier { x1, y1, x2, y2 } => cubic_bezier(x1, y1, x2, y2, t),
    }
}

/// Solve a CSS-style cubic Bézier for `y` given `x = t`.
/// Uses Newton's method to find the parameter, then evaluates y.
pub fn cubic_bezier(x1: f64, y1: f64, x2: f64, y2: f64, t: f64) -> f64 {
    // Solve for parameter `s` such that B_x(s) = t, then return B_y(s)
    let s = solve_t(x1, x2, t);
    bezier_component(y1, y2, s)
}

fn bezier_component(p1: f64, p2: f64, t: f64) -> f64 {
    let c1 = 3.0 * p1;
    let c2 = 3.0 * (p2 - p1) - c1;
    let c3 = 1.0 - c1 - c2;
    ((c3 * t + c2) * t + c1) * t
}

fn bezier_slope(p1: f64, p2: f64, t: f64) -> f64 {
    let c1 = 3.0 * p1;
    let c2 = 3.0 * (p2 - p1) - c1;
    let c3 = 1.0 - c1 - c2;
    (3.0 * c3 * t + 2.0 * c2) * t + c1
}

fn solve_t(x1: f64, x2: f64, x: f64) -> f64 {
    let mut t = x;
    for _ in 0..8 {
        let x_est = bezier_component(x1, x2, t) - x;
        let slope = bezier_slope(x1, x2, t);
        if slope.abs() < 1e-6 { break; }
        t -= x_est / slope;
        t = t.clamp(0.0, 1.0);
    }
    t
}
```

- [ ] **Step 3: Write interpolation tests**

```rust
// crates/mmot-core/src/evaluator/interpolate.rs (bottom)
#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{AnimatableValue, EasingValue, Keyframe, Vec2};

    fn approx(a: f64, b: f64) -> bool { (a - b).abs() < 1e-6 }

    fn kf(t: u64, v: f64) -> Keyframe<f64> {
        Keyframe { t, v, easing: EasingValue::linear() }
    }

    #[test]
    fn static_value_returns_as_is() {
        let av = AnimatableValue::Static(42.0_f64);
        assert_eq!(evaluate_f64(&av, 0), 42.0);
        assert_eq!(evaluate_f64(&av, 99), 42.0);
    }

    #[test]
    fn before_first_keyframe_holds_first_value() {
        let av = AnimatableValue::Animated(vec![kf(10, 0.0), kf(20, 1.0)]);
        assert!(approx(evaluate_f64(&av, 0), 0.0));
        assert!(approx(evaluate_f64(&av, 9), 0.0));
    }

    #[test]
    fn after_last_keyframe_holds_last_value() {
        let av = AnimatableValue::Animated(vec![kf(0, 0.0), kf(15, 1.0)]);
        assert!(approx(evaluate_f64(&av, 16), 1.0));
        assert!(approx(evaluate_f64(&av, 999), 1.0));
    }

    #[test]
    fn on_keyframe_returns_exact_value() {
        let av = AnimatableValue::Animated(vec![kf(0, 0.0), kf(10, 0.5), kf(20, 1.0)]);
        assert!(approx(evaluate_f64(&av, 10), 0.5));
    }

    #[test]
    fn linear_midpoint_interpolates() {
        let av = AnimatableValue::Animated(vec![kf(0, 0.0), kf(10, 1.0)]);
        assert!(approx(evaluate_f64(&av, 5), 0.5));
    }

    #[test]
    fn single_keyframe_always_returns_its_value() {
        let av = AnimatableValue::Animated(vec![kf(5, 0.75)]);
        assert!(approx(evaluate_f64(&av, 0), 0.75));
        assert!(approx(evaluate_f64(&av, 5), 0.75));
        assert!(approx(evaluate_f64(&av, 100), 0.75));
    }

    #[test]
    fn vec2_linear_interpolates() {
        let av = AnimatableValue::Animated(vec![
            Keyframe { t: 0, v: Vec2 { x: 0.0, y: 0.0 }, easing: EasingValue::linear() },
            Keyframe { t: 10, v: Vec2 { x: 100.0, y: 200.0 }, easing: EasingValue::linear() },
        ]);
        let v = evaluate_vec2(&av, 5);
        assert!(approx(v.x, 50.0));
        assert!(approx(v.y, 100.0));
    }
}
```

- [ ] **Step 4: Implement interpolation**

```rust
// crates/mmot-core/src/evaluator/interpolate.rs
use crate::schema::{AnimatableValue, EasingValue, Keyframe, Vec2};
use super::easing::{apply as apply_easing, EasingKind};

fn easing_kind(e: &EasingValue) -> EasingKind {
    use crate::schema::easing::NamedEasing;
    match e {
        EasingValue::Named(NamedEasing::Linear) => EasingKind::Linear,
        EasingValue::Named(NamedEasing::EaseIn) => EasingKind::EaseIn,
        EasingValue::Named(NamedEasing::EaseOut) => EasingKind::EaseOut,
        EasingValue::Named(NamedEasing::EaseInOut) => EasingKind::EaseInOut,
        EasingValue::CubicBezier { x1, y1, x2, y2, .. } =>
            EasingKind::CubicBezier { x1: *x1, y1: *y1, x2: *x2, y2: *y2 },
    }
}

fn normalised_t(from: u64, to: u64, frame: u64, easing: &EasingValue) -> f64 {
    if to == from { return 1.0; }
    let raw = (frame - from) as f64 / (to - from) as f64;
    apply_easing(easing_kind(easing), raw.clamp(0.0, 1.0))
}

fn find_segment<T>(kfs: &[Keyframe<T>], frame: u64) -> (usize, usize) {
    // Binary search for the last keyframe with t <= frame
    let idx = kfs.partition_point(|k| k.t <= frame);
    if idx == 0 { return (0, 0); }
    if idx >= kfs.len() { return (kfs.len() - 1, kfs.len() - 1); }
    (idx - 1, idx)
}

/// Evaluate an `AnimatableValue<f64>` at the given frame.
pub fn evaluate_f64(av: &AnimatableValue<f64>, frame: u64) -> f64 {
    match av {
        AnimatableValue::Static(v) => *v,
        AnimatableValue::Animated(kfs) => {
            if kfs.is_empty() { return 0.0; }
            let (from, to) = find_segment(kfs, frame);
            if from == to { return kfs[from].v; }
            let t = normalised_t(kfs[from].t, kfs[to].t, frame, &kfs[from].easing);
            kfs[from].v + (kfs[to].v - kfs[from].v) * t
        }
    }
}

/// Evaluate an `AnimatableValue<Vec2>` at the given frame.
pub fn evaluate_vec2(av: &AnimatableValue<Vec2>, frame: u64) -> Vec2 {
    match av {
        AnimatableValue::Static(v) => v.clone(),
        AnimatableValue::Animated(kfs) => {
            if kfs.is_empty() { return Vec2 { x: 0.0, y: 0.0 }; }
            let (from, to) = find_segment(kfs, frame);
            if from == to { return kfs[from].v.clone(); }
            let t = normalised_t(kfs[from].t, kfs[to].t, frame, &kfs[from].easing);
            Vec2 {
                x: kfs[from].v.x + (kfs[to].v.x - kfs[from].v.x) * t,
                y: kfs[from].v.y + (kfs[to].v.y - kfs[from].v.y) * t,
            }
        }
    }
}
```

- [ ] **Step 5: Create evaluator mod.rs**

```rust
// crates/mmot-core/src/evaluator/mod.rs
pub mod easing;
pub mod interpolate;

pub use interpolate::{evaluate_f64, evaluate_vec2};
```

- [ ] **Step 6: Run all evaluator tests**

```bash
cargo test -p mmot-core evaluator
```
Expected: all tests pass.

- [ ] **Step 7: Commit**

```bash
git add crates/mmot-core/src/evaluator/
git commit -m "feat: add keyframe evaluator with all easing functions"
```

---

## Task 8: Skia CPU Renderer — Solid + Image Layers

**Files:**
- Create: `crates/mmot-core/src/renderer/surface.rs`
- Create: `crates/mmot-core/src/renderer/solid.rs`
- Create: `crates/mmot-core/src/renderer/image.rs`
- Create: `crates/mmot-core/src/renderer/layers.rs`
- Create: `crates/mmot-core/src/renderer/mod.rs`
- Create: `crates/mmot-core/src/assets/mod.rs`
- Create: `crates/mmot-core/src/assets/image.rs`

- [ ] **Step 1: Write renderer tests**

```rust
// crates/mmot-core/src/renderer/mod.rs (bottom)
#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{AnimatableValue, Vec2};

    fn make_solid_frame(color: &str, width: u32, height: u32) -> FrameScene {
        FrameScene {
            width, height,
            background: "#000000".into(),
            layers: vec![ResolvedLayer {
                opacity: 1.0,
                transform: ResolvedTransform {
                    position: Vec2 { x: (width / 2) as f64, y: (height / 2) as f64 },
                    scale: Vec2 { x: 1.0, y: 1.0 },
                    rotation: 0.0,
                    opacity: 1.0,
                },
                content: ResolvedContent::Solid { color: color.into() },
            }],
        }
    }

    #[test]
    fn solid_red_fills_buffer() {
        let frame = make_solid_frame("#ff0000", 4, 4);
        let rgba = render(&frame).unwrap();
        assert_eq!(rgba.len(), 4 * 4 * 4);
        // First pixel should be red
        assert_eq!(rgba[0], 255); // R
        assert_eq!(rgba[1], 0);   // G
        assert_eq!(rgba[2], 0);   // B
        assert_eq!(rgba[3], 255); // A
    }

    #[test]
    fn output_dimensions_match_frame() {
        let frame = make_solid_frame("#ffffff", 16, 9);
        let rgba = render(&frame).unwrap();
        assert_eq!(rgba.len(), 16 * 9 * 4);
    }
}
```

- [ ] **Step 2: Define FrameScene (resolved types)**

```rust
// crates/mmot-core/src/renderer/mod.rs
mod surface;
mod solid;
mod image;
pub mod layers;
// mod text; — added in Task 9

use crate::error::Result;
use crate::schema::Vec2;

/// A fully resolved frame — all animatable values are concrete.
/// This is what the renderer receives. No JSON, no keyframes.
pub struct FrameScene {
    pub width: u32,
    pub height: u32,
    pub background: String,
    pub layers: Vec<ResolvedLayer>,
}

pub struct ResolvedLayer {
    pub opacity: f64,
    pub transform: ResolvedTransform,
    pub content: ResolvedContent,
}

pub struct ResolvedTransform {
    pub position: Vec2,
    pub scale: Vec2,
    pub rotation: f64,
    pub opacity: f64,
}

pub enum ResolvedContent {
    Solid { color: String },
    Image { data: Vec<u8>, width: u32, height: u32 },
    Text { text: String, font_family: String, font_size: f64, font_weight: u32, color: String, align: crate::schema::TextAlign },
    // Video and others added in Phase 2
}

/// Render a FrameScene to raw RGBA bytes (width × height × 4).
pub fn render(frame_scene: &FrameScene) -> Result<Vec<u8>> {
    let w = frame_scene.width;
    let h = frame_scene.height;
    let mut surface = surface::create_cpu_surface(w, h);
    let canvas = surface.canvas();

    // Clear with background colour
    canvas.clear(parse_color(&frame_scene.background));

    // Draw layers in order (first = bottom of visual stack)
    for layer in &frame_scene.layers {
        layers::draw_layer(canvas, layer, w, h);
    }

    // Extract RGBA pixels directly — no PNG roundtrip
    let row_bytes = (w * 4) as usize;
    let mut rgba = vec![0u8; (w * h * 4) as usize];
    let info = skia_safe::ImageInfo::new(
        (w as i32, h as i32),
        skia_safe::ColorType::RGBA8888,
        skia_safe::AlphaType::Premul,
        None,
    );
    surface.read_pixels(&info, &mut rgba, row_bytes, (0, 0));
    Ok(rgba)
}

fn parse_color(hex: &str) -> skia_safe::Color {
    let hex = hex.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
    skia_safe::Color::from_argb(255, r, g, b)
}
```

- [ ] **Step 3: Implement Skia surface creation**

```rust
// crates/mmot-core/src/renderer/surface.rs
use skia_safe::{surfaces, Surface, ColorType, AlphaType, ImageInfo};

pub fn create_cpu_surface(width: u32, height: u32) -> Surface {
    let info = ImageInfo::new(
        (width as i32, height as i32),
        ColorType::RGBA8888,
        AlphaType::Premul,
        None,
    );
    surfaces::raster(&info, None, None)
        .expect("failed to create Skia CPU surface")
}
```

- [ ] **Step 4: Implement layer draw dispatch**

```rust
// crates/mmot-core/src/renderer/layers.rs
use skia_safe::Canvas;
use crate::renderer::{ResolvedLayer, ResolvedContent};
use super::{solid, image as img_renderer};

pub fn draw_layer(canvas: &Canvas, layer: &ResolvedLayer, width: u32, height: u32) {
    canvas.save();
    let paint = apply_transform(canvas, layer);
    match &layer.content {
        ResolvedContent::Solid { color } => solid::draw(canvas, color, width, height, &paint),
        ResolvedContent::Image { data, width: iw, height: ih } =>
            img_renderer::draw(canvas, data, *iw, *ih, &paint),
        ResolvedContent::Text { .. } => {
            // Text rendering added in Task 9
            tracing::warn!("text layer rendering not yet implemented");
        }
    }
    canvas.restore();
}

fn apply_transform(canvas: &Canvas, layer: &ResolvedLayer) -> skia_safe::Paint {
    let t = &layer.transform;
    let mut m = skia_safe::Matrix::new_identity();
    m.pre_translate((t.position.x as f32, t.position.y as f32));
    m.pre_rotate(t.rotation as f32, None);
    m.pre_scale((t.scale.x as f32, t.scale.y as f32), None);
    m.pre_translate((-t.position.x as f32, -t.position.y as f32));
    canvas.concat(&m);
    let mut paint = skia_safe::Paint::default();
    paint.set_alpha_f(layer.opacity as f32);
    paint
}
```

- [ ] **Step 5: Implement solid layer**

```rust
// crates/mmot-core/src/renderer/solid.rs
use skia_safe::{Canvas, Paint, Color, Rect};

pub fn draw(canvas: &Canvas, color: &str, width: u32, height: u32, base_paint: &Paint) {
    let hex = color.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
    let mut paint = base_paint.clone();
    paint.set_color(Color::from_argb(
        (base_paint.alpha_f() * 255.0) as u8, r, g, b,
    ));
    canvas.draw_rect(
        Rect::from_wh(width as f32, height as f32),
        &paint,
    );
}
```

- [ ] **Step 6: Implement image layer**

```rust
// crates/mmot-core/src/renderer/image.rs
use skia_safe::{Canvas, Paint, Image, Data, Rect};

pub fn draw(canvas: &Canvas, rgba: &[u8], width: u32, height: u32, base_paint: &Paint) {
    let info = skia_safe::ImageInfo::new(
        (width as i32, height as i32),
        skia_safe::ColorType::RGBA8888,
        skia_safe::AlphaType::Premul,
        None,
    );
    let row_bytes = (width * 4) as usize;
    let image = Image::from_raster_data(&info, Data::new_copy(rgba), row_bytes)
        .expect("failed to create Skia image from RGBA");
    let dst = Rect::from_wh(width as f32, height as f32);
    canvas.draw_image_rect(&image, None, dst, base_paint);
}
```

- [ ] **Step 7: Implement assets module**

```rust
// crates/mmot-core/src/assets/mod.rs
pub mod image;

/// Decoded image asset: raw RGBA bytes + dimensions.
pub struct DecodedImage {
    pub rgba: Vec<u8>,
    pub width: u32,
    pub height: u32,
}
```

```rust
// crates/mmot-core/src/assets/image.rs
use crate::error::{MmotError, Result};
use super::DecodedImage;

/// Decode PNG/JPEG/WebP bytes to RGBA.
pub fn decode(data: &[u8]) -> Result<DecodedImage> {
    let img = image::load_from_memory(data)
        .map_err(|e| MmotError::AssetLoad(e.to_string()))?
        .into_rgba8();
    let (width, height) = img.dimensions();
    Ok(DecodedImage { rgba: img.into_raw(), width, height })
}
```

- [ ] **Step 8: Run tests**

```bash
cargo test -p mmot-core renderer
```
Expected: all pass.

- [ ] **Step 9: Commit**

```bash
git add crates/mmot-core/src/renderer/ crates/mmot-core/src/assets/
git commit -m "feat: add Skia CPU renderer for solid and image layers"
```

---

## Task 9: Text Layer Rendering

**Files:**
- Create: `crates/mmot-core/src/renderer/text.rs`
- Modify: `crates/mmot-core/src/renderer/layers.rs`

- [ ] **Step 1: Write text rendering test**

```rust
// crates/mmot-core/src/renderer/text.rs (bottom)
#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::{FrameScene, ResolvedLayer, ResolvedTransform, ResolvedContent};
    use crate::schema::{Vec2, TextAlign};

    #[test]
    fn text_layer_renders_without_panic() {
        let frame = FrameScene {
            width: 320, height: 180,
            background: "#000000".into(),
            layers: vec![ResolvedLayer {
                opacity: 1.0,
                transform: ResolvedTransform {
                    position: Vec2 { x: 160.0, y: 90.0 },
                    scale: Vec2 { x: 1.0, y: 1.0 },
                    rotation: 0.0,
                    opacity: 1.0,
                },
                content: ResolvedContent::Text {
                    text: "Hello Mercury".into(),
                    font_family: "sans-serif".into(),
                    font_size: 24.0,
                    font_weight: 400,
                    color: "#ffffff".into(),
                    align: TextAlign::Center,
                },
            }],
        };
        let rgba = crate::renderer::render(&frame).unwrap();
        // Output is not blank (has some non-zero pixels from the text)
        assert!(rgba.iter().any(|&b| b > 0));
    }
}
```

- [ ] **Step 2: Implement text rendering**

```rust
// crates/mmot-core/src/renderer/text.rs
use skia_safe::{Canvas, Paint, Color, Font, FontStyle, FontMgr, TextBlob};
use crate::schema::TextAlign;

pub fn draw(
    canvas: &Canvas,
    text: &str,
    x: f32, y: f32,
    font_family: &str,
    font_size: f64,
    font_weight: u32,
    color: &str,
    align: &TextAlign,
) {
    let font_mgr = FontMgr::new();
    let style = FontStyle::new(
        skia_weight(font_weight),
        skia_safe::font_style::Width::NORMAL,
        skia_safe::font_style::Slant::Upright,
    );
    let typeface = font_mgr
        .match_family_style(font_family, style)
        .or_else(|| font_mgr.match_family_style("sans-serif", style))
        .unwrap_or_else(|| font_mgr.legacy_make_typeface(None, style).unwrap());

    let font = Font::new(typeface, font_size as f32);
    let (hex_r, hex_g, hex_b) = parse_hex_color(color);

    let mut paint = Paint::default();
    paint.set_color(Color::from_rgb(hex_r, hex_g, hex_b));
    paint.set_anti_alias(true);

    let blob = TextBlob::new(text, &font).unwrap_or_else(|| {
        TextBlob::new("?", &font).unwrap()
    });

    let bounds = blob.bounds();
    let draw_x = match align {
        TextAlign::Left => x,
        TextAlign::Center => x - bounds.width() / 2.0,
        TextAlign::Right => x - bounds.width(),
    };
    let draw_y = y + bounds.height() / 2.0;

    canvas.draw_text_blob(&blob, (draw_x, draw_y), &paint);
}

fn skia_weight(weight: u32) -> skia_safe::font_style::Weight {
    skia_safe::font_style::Weight::from(weight as i32)
}

fn parse_hex_color(hex: &str) -> (u8, u8, u8) {
    let hex = hex.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(255);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(255);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(255);
    (r, g, b)
}
```

- [ ] **Step 3: Register text module and wire into layers.rs**

In `renderer/mod.rs`, replace the comment placeholder with the real declaration:
```rust
// Change: // mod text; — added in Task 9
// To:
pub mod text;
```

In `layers.rs`, add `use super::text;` to the imports and replace the `Text` arm stub:
```rust
use super::{solid, image as img_renderer, text};

// In draw_layer(), replace the Text arm:
ResolvedContent::Text { text: t, font_family, font_size, font_weight, color, align } => {
    let t_ref = &layer.transform;
    text::draw(
        canvas, t, t_ref.position.x as f32, t_ref.position.y as f32,
        font_family, *font_size, *font_weight, color, align,
    );
}
```

- [ ] **Step 4: Run tests**

```bash
cargo test -p mmot-core renderer
```

- [ ] **Step 5: Commit**

```bash
git add crates/mmot-core/src/renderer/text.rs crates/mmot-core/src/renderer/layers.rs
git commit -m "feat: add text layer rendering via Skia SkShaper"
```

---

## Task 10: Pipeline + Parallel Frame Rendering

**Files:**
- Create: `crates/mmot-core/src/pipeline.rs`

The pipeline wires everything together: parse → evaluate (parallel) → render (parallel) → encode.

- [ ] **Step 1: Write pipeline integration test**

```rust
// crates/mmot-core/src/pipeline.rs (bottom)
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pipeline_renders_minimal_scene() {
        let json = include_str!("../../../tests/fixtures/valid/minimal.mmot.json");
        let opts = RenderOptions {
            output_path: std::env::temp_dir().join("mmot-test-output.mp4"),
            format: OutputFormat::Mp4,
            quality: 80,
            frame_range: None,
            concurrency: Some(2),
            backend: RenderBackend::Cpu,
            include_audio: false,
        };
        // Just verify it doesn't panic and produces some bytes
        render_scene(json, opts, None).unwrap();
        // Output file should exist (for real test; skip in CI if no encoder)
    }
}
```

- [ ] **Step 2: Implement pipeline**

```rust
// crates/mmot-core/src/pipeline.rs
use std::path::PathBuf;
use std::sync::Arc;
use rayon::prelude::*;

use crate::error::{MmotError, Result};
use crate::parser::parse;
use crate::evaluator::interpolate::{evaluate_f64, evaluate_vec2};
use crate::renderer::{FrameScene, ResolvedLayer, ResolvedTransform, ResolvedContent, render as render_frame};
use crate::schema::{LayerContent, Scene, TextAlign};

pub struct RenderOptions {
    pub output_path: PathBuf,
    pub format: OutputFormat,
    pub quality: u8,
    pub frame_range: Option<(u64, u64)>,
    pub concurrency: Option<usize>,
    pub backend: RenderBackend,
    pub include_audio: bool,
}

#[derive(Debug, Clone)]
pub enum OutputFormat { Mp4, Gif, Webm }

#[derive(Debug, Clone)]
pub enum RenderBackend { Cpu, Gpu }

/// Progress callback: called with (current_frame, total_frames).
pub type ProgressFn = Arc<dyn Fn(u64, u64) + Send + Sync>;

/// Main entry point: parse JSON, render all frames, encode.
pub fn render_scene(json: &str, opts: RenderOptions, progress: Option<ProgressFn>) -> Result<()> {
    let scene = parse(json)?;
    let total = match opts.frame_range {
        Some((s, e)) => e - s,
        None => scene.meta.duration,
    };
    let start = opts.frame_range.map(|(s, _)| s).unwrap_or(0);

    // Set rayon thread count if specified
    if let Some(n) = opts.concurrency {
        rayon::ThreadPoolBuilder::new()
            .num_threads(n)
            .build_global()
            .ok(); // Ignore error if already set
    }

    // Render all frames in parallel, collect in order
    let scene = Arc::new(scene);
    let frames: Vec<Result<Vec<u8>>> = (start..start + total)
        .into_par_iter()
        .map(|frame_num| {
            let frame_scene = evaluate_scene(&scene, frame_num)?;
            let rgba = render_frame(&frame_scene)
                .map_err(|e| match e {
                    MmotError::RenderFailed { reason, .. } =>
                        MmotError::RenderFailed { frame: frame_num, reason },
                    other => other,
                })?;
            if let Some(ref cb) = progress {
                cb(frame_num - start, total);
            }
            Ok(rgba)
        })
        .collect();

    // Check for errors
    let frames: Vec<Vec<u8>> = frames.into_iter().collect::<Result<_>>()?;

    // Encode
    crate::encoder::mp4::encode(
        frames,
        scene.meta.width,
        scene.meta.height,
        scene.meta.fps,
        opts.quality,
        &opts.output_path,
    )?;

    Ok(())
}

/// Evaluate a scene at a specific frame number → FrameScene.
pub fn evaluate_scene(scene: &Scene, frame: u64) -> Result<FrameScene> {
    let comp = scene.compositions.get(&scene.meta.root)
        .ok_or_else(|| MmotError::Parse {
            message: format!("root composition '{}' not found", scene.meta.root),
            pointer: "/meta/root".into(),
        })?;

    let mut resolved_layers = Vec::new();
    for layer in &comp.layers {
        if frame < layer.in_point || frame >= layer.out_point { continue; }

        let position = evaluate_vec2(&layer.transform.position, frame);
        let scale = evaluate_vec2(&layer.transform.scale, frame);
        let opacity = evaluate_f64(&layer.transform.opacity, frame);
        let rotation = evaluate_f64(&layer.transform.rotation, frame);

        let content = match &layer.content {
            LayerContent::Solid { color } =>
                ResolvedContent::Solid { color: color.clone() },
            LayerContent::Text { text, font, align } =>
                ResolvedContent::Text {
                    text: text.clone(),
                    font_family: font.family.clone(),
                    font_size: font.size,
                    font_weight: font.weight,
                    color: font.color.clone(),
                    align: align.clone(),
                },
            _ => continue, // Other layer types added in Phase 2
        };

        resolved_layers.push(ResolvedLayer {
            opacity,
            transform: ResolvedTransform { position, scale, rotation, opacity },
            content,
        });
    }

    Ok(FrameScene {
        width: scene.meta.width,
        height: scene.meta.height,
        background: scene.meta.background.clone(),
        layers: resolved_layers,
    })
}
```

- [ ] **Step 3: Run tests**

```bash
cargo test -p mmot-core pipeline
```

- [ ] **Step 4: Commit**

```bash
git add crates/mmot-core/src/pipeline.rs
git commit -m "feat: add parallel frame pipeline with rayon"
```

---

## Task 11: Encoder — rav1e + MP4

**Files:**
- Create: `crates/mmot-core/src/encoder/mod.rs`
- Create: `crates/mmot-core/src/encoder/mp4.rs`

- [ ] **Step 1: Write encoder test**

```rust
// crates/mmot-core/src/encoder/mp4.rs (bottom)
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_single_black_frame() {
        let width = 64u32;
        let height = 64u32;
        // Black RGBA frame
        let frame = vec![0u8; (width * height * 4) as usize];
        let path = std::env::temp_dir().join("mmot-encoder-test.mp4");
        encode(vec![frame], width, height, 30.0, 80, &path).unwrap();
        assert!(path.exists());
        let metadata = std::fs::metadata(&path).unwrap();
        assert!(metadata.len() > 0, "output file is empty");
        std::fs::remove_file(&path).ok();
    }
}
```

- [ ] **Step 2: Implement encoder**

```rust
// crates/mmot-core/src/encoder/mp4.rs
use std::path::Path;
use crate::error::{MmotError, Result};

/// Encode a sequence of RGBA frames to an MP4 file.
/// Uses rav1e for AV1 video encoding and a pure-Rust MP4 muxer.
pub fn encode(
    frames: Vec<Vec<u8>>,
    width: u32,
    height: u32,
    fps: f64,
    quality: u8,
    output: &Path,
) -> Result<()> {
    use rav1e::prelude::*;

    let cfg = Config::new()
        .with_encoder_config(EncoderConfig {
            width: width as usize,
            height: height as usize,
            time_base: Rational::new(1, fps as u64),
            bit_depth: 8,
            quantizer: map_quality_to_quantizer(quality),
            ..Default::default()
        });

    let mut ctx: Context<u8> = cfg
        .new_context()
        .map_err(|e| MmotError::Encoder(e.to_string()))?;

    let mut encoded_packets: Vec<Vec<u8>> = Vec::new();

    for rgba in &frames {
        let mut frame = ctx.new_frame();
        rgba_to_yuv420(rgba, width, height, &mut frame);
        ctx.send_frame(frame).map_err(|e| MmotError::Encoder(e.to_string()))?;
        drain_packets(&mut ctx, &mut encoded_packets)?;
    }

    ctx.flush();
    drain_packets(&mut ctx, &mut encoded_packets)?;

    write_mp4(&encoded_packets, width, height, fps, output)?;
    Ok(())
}

fn drain_packets(ctx: &mut rav1e::prelude::Context<u8>, out: &mut Vec<Vec<u8>>) -> Result<()> {
    loop {
        match ctx.receive_packet() {
            Ok(pkt) => out.push(pkt.data.into()),
            Err(rav1e::prelude::EncoderStatus::NeedMoreData) => break,
            Err(rav1e::prelude::EncoderStatus::EnoughData) => break,
            Err(e) => return Err(MmotError::Encoder(e.to_string())),
        }
    }
    Ok(())
}

fn map_quality_to_quantizer(quality: u8) -> usize {
    // quality 100 = quantizer 0 (best), quality 1 = quantizer 63 (worst)
    let q = quality.clamp(1, 100);
    ((100 - q) as usize * 63) / 99
}

fn rgba_to_yuv420(rgba: &[u8], width: u32, height: u32, frame: &mut rav1e::prelude::Frame<u8>) {
    // Simplified RGBA → YUV420 conversion
    let w = width as usize;
    let h = height as usize;
    let y_plane = &mut frame.planes[0];
    let u_plane = &mut frame.planes[1];
    let v_plane = &mut frame.planes[2];

    for row in 0..h {
        for col in 0..w {
            let i = (row * w + col) * 4;
            let r = rgba[i] as f32;
            let g = rgba[i + 1] as f32;
            let b = rgba[i + 2] as f32;
            let y = (0.299 * r + 0.587 * g + 0.114 * b) as u8;
            *y_plane.p(col, row) = y;
            if row % 2 == 0 && col % 2 == 0 {
                let u = (128.0 - 0.168736 * r - 0.331264 * g + 0.5 * b) as u8;
                let v = (128.0 + 0.5 * r - 0.418688 * g - 0.081312 * b) as u8;
                *u_plane.p(col / 2, row / 2) = u;
                *v_plane.p(col / 2, row / 2) = v;
            }
        }
    }
}

fn write_mp4(packets: &[Vec<u8>], width: u32, height: u32, fps: f64, path: &Path) -> Result<()> {
    use minimp4::Mp4Muxer;
    use std::io::Cursor;

    let mut cursor = Cursor::new(Vec::new());
    let mut muxer = Mp4Muxer::new(&mut cursor);
    muxer.init_video(width as i32, height as i32, false, "AV01");
    for pkt in packets {
        muxer.write_video(pkt);
    }
    muxer.close();
    std::fs::write(path, cursor.into_inner()).map_err(MmotError::Io)?;
    Ok(())
}
```

- [ ] **Step 3: Create encoder/mod.rs**

```rust
// crates/mmot-core/src/encoder/mod.rs
pub mod mp4;
```

- [ ] **Step 4: Run tests**

```bash
cargo test -p mmot-core encoder
```

- [ ] **Step 5: Commit**

```bash
git add crates/mmot-core/src/encoder/
git commit -m "feat: add rav1e encoder with AV1/MP4 output via minimp4"
```

---

## Task 12: CLI — `mmot render` and `mmot validate`

**Files:**
- Modify: `crates/mmot-cli/src/main.rs`

- [ ] **Step 1: Write CLI integration test**

```rust
// crates/mmot-cli/tests/cli_test.rs
use std::process::Command;

fn mmot_bin() -> std::path::PathBuf {
    let mut p = std::env::current_exe().unwrap();
    p.pop(); p.pop();
    let name = if cfg!(windows) { "mmot.exe" } else { "mmot" };
    p.push(name);
    p
}

#[test]
fn validate_valid_file_exits_zero() {
    let status = Command::new(mmot_bin())
        .args(["validate", "../../tests/fixtures/valid/minimal.mmot.json"])
        .status().unwrap();
    assert!(status.success());
}

#[test]
fn validate_invalid_file_exits_two() {
    let status = Command::new(mmot_bin())
        .args(["validate", "../../tests/fixtures/invalid/missing_root.mmot.json"])
        .status().unwrap();
    assert_eq!(status.code(), Some(2));
}
```

- [ ] **Step 2: Implement full CLI**

```rust
// crates/mmot-cli/src/main.rs
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use anyhow::Context;

#[derive(Parser)]
#[command(name = "mmot", version = "0.1.0", about = "Mercury-Motion programmatic video renderer")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Render a .mmot.json file to video
    Render {
        file: PathBuf,
        #[arg(short, long, default_value = "output.mp4")]
        output: PathBuf,
        #[arg(short, long, default_value = "mp4")]
        format: String,
        #[arg(short, long, default_value_t = 80)]
        quality: u8,
        #[arg(long)]
        props: Option<String>,
        #[arg(long)]
        frames: Option<String>,
        #[arg(long)]
        concurrency: Option<usize>,
        #[arg(long, default_value = "cpu")]
        backend: String,
        #[arg(long)]
        no_audio: bool,
        #[arg(short, long)]
        verbose: bool,
    },
    /// Validate a .mmot.json file without rendering
    Validate {
        file: PathBuf,
    },
}

fn main() {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();
    let exit_code = match run(cli) {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("{e}");
            match e.downcast_ref::<mmot_core::error::MmotError>() {
                Some(mmot_core::error::MmotError::Parse { .. }) => 2,
                Some(mmot_core::error::MmotError::AssetNotFound { .. }) => 3,
                _ => 1,
            }
        }
    };
    std::process::exit(exit_code);
}

fn run(cli: Cli) -> anyhow::Result<()> {
    match cli.command {
        Commands::Validate { file } => {
            let json = std::fs::read_to_string(&file)
                .with_context(|| format!("cannot read {}", file.display()))?;
            mmot_core::parser::parse(&json)?;
            println!("✓ {}: valid", file.display());
            Ok(())
        }
        Commands::Render { file, output, format, quality, concurrency, no_audio, verbose, .. } => {
            let json = std::fs::read_to_string(&file)
                .with_context(|| format!("cannot read {}", file.display()))?;

            let fmt = match format.as_str() {
                "mp4" => mmot_core::pipeline::OutputFormat::Mp4,
                "gif" => mmot_core::pipeline::OutputFormat::Gif,
                "webm" => mmot_core::pipeline::OutputFormat::Webm,
                other => anyhow::bail!("unknown format: {other}"),
            };

            let progress: Option<mmot_core::pipeline::ProgressFn> = if verbose {
                Some(std::sync::Arc::new(|current, total| {
                    eprint!("\rRendering frame {current}/{total}");
                }))
            } else {
                None
            };

            let opts = mmot_core::pipeline::RenderOptions {
                output_path: output.clone(),
                format: fmt,
                quality,
                frame_range: None,
                concurrency,
                backend: mmot_core::pipeline::RenderBackend::Cpu,
                include_audio: !no_audio,
            };

            mmot_core::pipeline::render_scene(&json, opts, progress)?;
            if verbose { eprintln!(); }
            println!("✓ rendered to {}", output.display());
            Ok(())
        }
    }
}
```

- [ ] **Step 3: Build the binary**

```bash
cargo build -p mmot
```

- [ ] **Step 4: Smoke test the CLI manually**

```bash
./target/debug/mmot validate tests/fixtures/valid/minimal.mmot.json
# Expected: ✓ tests/fixtures/valid/minimal.mmot.json: valid

./target/debug/mmot validate tests/fixtures/invalid/missing_root.mmot.json
# Expected: exits with code 2 and an error message

./target/debug/mmot render tests/fixtures/valid/minimal.mmot.json -o /tmp/test.mp4 -v
# Expected: renders frames, produces /tmp/test.mp4
```

- [ ] **Step 5: Run CLI tests**

```bash
cargo test -p mmot
```

- [ ] **Step 6: Commit**

```bash
git add crates/mmot-cli/src/main.rs crates/mmot-cli/tests/
git commit -m "feat: add mmot render and mmot validate CLI commands"
```

---

## Task 13: Golden Image Tests

**Files:**
- Create: `crates/mmot-core/tests/golden/minimal/frame-000.png` (generated)
- Create: `crates/mmot-core/tests/golden_test.rs`
- Create: `crates/mmot-core/tests/generate_goldens.rs`

- [ ] **Step 1: Generate reference renders**

Write a one-shot binary target (or use `cargo test -- --ignored`) to produce the initial golden files:

```rust
// crates/mmot-core/tests/generate_goldens.rs  (run once with: cargo test -p mmot-core --test generate_goldens -- --ignored)
use mmot_core::{parser, pipeline, renderer};

#[test]
#[ignore]
fn generate_golden_minimal() {
    let json = std::fs::read_to_string("tests/fixtures/valid/minimal.mmot.json").unwrap();
    let scene = parser::parse(&json).unwrap();
    let frame_scene = pipeline::evaluate_scene(&scene, 0).unwrap();
    let w = frame_scene.width;
    let h = frame_scene.height;
    let rgba = renderer::render(&frame_scene).unwrap();
    // Encode raw RGBA → PNG for the reference file
    let img = image::RgbaImage::from_raw(w, h, rgba).expect("invalid dimensions");
    std::fs::create_dir_all("crates/mmot-core/tests/golden/minimal").unwrap();
    img.save("crates/mmot-core/tests/golden/minimal/frame-000.png").unwrap();
}
```

```bash
# Run once to produce the reference PNG, then commit it:
cargo test -p mmot-core --test generate_goldens -- --ignored
git add crates/mmot-core/tests/golden/minimal/frame-000.png
```

- [ ] **Step 2: Write golden test**

```rust
// crates/mmot-core/tests/golden_test.rs
use std::path::Path;
use mmot_core::{parser, pipeline, renderer};

fn render_frame_to_png(fixture: &str, frame: u64) -> Vec<u8> {
    let json = std::fs::read_to_string(
        Path::new("tests/fixtures/valid").join(fixture)
    ).unwrap();
    let scene = parser::parse(&json).unwrap();
    let frame_scene = pipeline::evaluate_scene(&scene, frame).unwrap();
    renderer::render(&frame_scene).unwrap()
}

fn load_reference_png(name: &str, frame: u64) -> Vec<u8> {
    let path = format!("crates/mmot-core/tests/golden/{name}/frame-{frame:03}.png");
    let img = image::open(&path)
        .unwrap_or_else(|_| panic!("reference image not found: {path}"))
        .into_rgba8();
    img.into_raw()
}

#[test]
fn golden_minimal_frame_0() {
    let rendered = render_frame_to_png("minimal.mmot.json", 0);
    let reference = load_reference_png("minimal", 0);
    assert_eq!(rendered.len(), reference.len(), "frame size mismatch");
    assert_eq!(rendered, reference, "pixel mismatch — renderer output changed");
}
```

- [ ] **Step 3: Run golden tests**

```bash
cargo test -p mmot-core --test golden_test
```

- [ ] **Step 4: Commit golden images + test**

```bash
git add crates/mmot-core/tests/golden/ crates/mmot-core/tests/golden_test.rs crates/mmot-core/tests/generate_goldens.rs
git commit -m "test: add golden image tests for CPU renderer"
```

---

## Task 14: CI Green + Final Phase 1 Commit

- [ ] **Step 1: Run full test suite locally**

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```
Expected: all green, no warnings.

- [ ] **Step 2: Fix any clippy warnings**

Run `cargo clippy --fix` for auto-fixable issues, manually fix the rest.

- [ ] **Step 3: Push and verify CI**

```bash
git push origin main
# Check GitHub Actions → all 3 platform jobs green
```

- [ ] **Step 4: Tag Phase 1**

```bash
git tag v0.1.0-phase1
git push origin v0.1.0-phase1
```

---

## Phase 1 Done — What's Next

With Phase 1 complete you have:
- `mmot validate` — validates any `.mmot.json` with useful errors
- `mmot render` — renders solid + text + image layers to AV1/MP4 output
- Parallel CPU rendering via rayon
- Golden image test suite proving deterministic output
- CI green on Windows, macOS, Linux

**Remaining plans:**
- **Plan 2** — Full Renderer (audio, video layers, Lottie, shapes, props, GPU)
- **Plan 3** — Editor MVP (Tauri + React 19 + Zustand)
- **Plan 4** — Editor UI/UX (keyframe curves, animations, design polish) ← *UI/UX work lives here*

---

## Open Questions to Resolve Before Starting

| # | Question | Blocks |
|---|----------|--------|
| ~~Q1~~ | ~~Font fallback on missing font~~ | **Resolved:** silent fallback to `sans-serif` (see Task 9 `text.rs`) |
