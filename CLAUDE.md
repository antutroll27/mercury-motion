# Mercury-Motion — Agent Instructions

Read this before touching any file. Everything here is a resolved decision.

---

## What This Project Is

**Mercury-Motion** is a native Rust video creation engine — the Supabase of motion design.
Two products, one codebase: **Core** (JSON → video, 100× faster than Remotion) and **FilmTools** (ACES color science, film emulation, AI-assisted VFX — zero generative AI).

Format: `.mmot.json` files describe videos. `mmot render file.mmot.json` produces MP4/WebM/GIF.
CLI binary: `mmot`. Desktop editor: Tauri 2.0 + Vue 3. Browser editor: Vite + Vue 3. WASM: `mmot-wasm`.

**Published:** crates.io (`mmot`, `mmot-core`), npm (`mercury-motion`, `mmot-mcp`), GitHub (`antutroll27/mercury-motion`)

---

## Repo Structure

```
crates/
  mmot-core/          # Core engine library (parser, evaluator, renderer, encoder, diff, audit, export, batch, tokens)
  mmot-cli/           # mmot binary (clap) — render, validate, diff, audit, export-all, test, batch, interactive
  mmot-wasm/          # WASM bindings (wasm-bindgen + tiny-skia) — excluded from workspace, build with wasm-pack
editor/               # Vue 3 + TypeScript (Bun, Vite, Tailwind CSS 4, Pinia)
  src/components/     # TopBar, CanvasPreview, CanvasGizmos, Timeline, LayerPanel, PropertyInspector,
                      # EffectsPanel, MaskEditor, MediaBrowser, AddLayerDialog, KeyframeStrip, EasingPicker, ExportModal
  src/stores/         # Pinia scene store (undo/redo, keyframe CRUD, playback, preview)
  src-tauri/          # Tauri 2.0 shell + IPC commands (browser-only mode is default)
packages/
  react/              # @mmot/react — Remotion-compatible Scene builder API, renders via mmot CLI
  player/             # @mmot/player — embeddable web player, Canvas 2D, zero dependencies
mcp/                  # MCP server (Node.js, 7 tools for AI-driven animation)
skills/
  mercury-motion/     # SKILL.md — 2,100+ lines teaching AI the .mmot.json format + design principles
schema/               # mmot.schema.json — auto-generated JSON Schema (draft-07) from Rust types
examples/             # 15 polished animation examples
templates/            # 16 categorized templates (social, business, creative, UI, transitions)
tests/
  fixtures/valid/     # Valid .mmot.json files (18 files)
  fixtures/invalid/   # Invalid .mmot.json files (3 files, correctly rejected)
docs/                 # Plans, specs, roadmaps (gitignored)
Cargo.toml            # Workspace root
```

---

## Non-Negotiable Decisions

**Frontend:** Vue 3 + TypeScript. State: Pinia. NOT React, NOT Zustand, NOT Redux.
**Package manager:** Bun. NEVER npm or yarn.
**Rust edition:** 2024 for all crates.
**File format:** `.mmot.json` everywhere. Never `.mercury.json`.
**Error handling:**
  - Library crates (`mmot-core`): `thiserror` — typed error enums only
  - Binary/app crates (`mmot-cli`, `mmot-wasm`): `anyhow` — context-chained messages
  - Never use `unwrap()` outside of tests. Use `expect("reason")` if you must.
**Logging:** `tracing` crate. No `println!` in library code.
**Parallelism:** `rayon` for CPU frame rendering. Single `wgpu` context per process.
**No network calls** from the renderer or FilmTools engine. Zero. Rendering is offline.
**No eval, no dynamic code** — `.mmot.json` is data, not code.
**No Co-Authored-By Claude** lines in git commits.

---

## Feature Flags

```toml
# mmot-core/Cargo.toml
[features]
default = ["native-renderer"]
native-renderer = ["dep:skia-safe", "dep:rav1e", "dep:muxide", "dep:gif", "dep:symphonia"]
audio-codec = ["opus"]
ffmpeg = ["dep:ffmpeg-next", "dep:ffmpeg-sys-next"]
```

- `native-renderer` (default): Skia CPU renderer + rav1e AV1 encoder + all encoders. Disable for WASM builds.
- `audio-codec`: Opus audio encoding (requires C libopus).
- `ffmpeg`: Video layer decoding, audio muxing into MP4, WebM output via ffmpeg CLI.

Without `native-renderer`, only parser, evaluator, schema, diff, accessibility, and tokens are available (WASM-compatible).

### ffmpeg Feature — Windows Setup

```bash
export FFMPEG_DIR="d:/ffmpeg-dev/ffmpeg-n7.1-latest-win64-gpl-shared-7.1"
export LIBCLANG_PATH="d:/llvm/llvm-extracted/bin"
cargo build -p mmot-core --features ffmpeg
cp d:/ffmpeg-dev/ffmpeg-n7.1-latest-win64-gpl-shared-7.1/bin/*.dll target/debug/deps/
```

---

## CLI Commands

```bash
mmot render <file> [--output path] [--format mp4|gif|webm] [--quality 1-100] [--prop key=value] [--verbose]
mmot validate <file>
mmot diff <file_a> <file_b> [--no-color]          # semantic animation diffs for Git
mmot audit <file> [--level aa|aaa] [--no-color]    # WCAG accessibility checker
mmot export-all <file> [--output-dir dir] [--profiles youtube,tiktok,instagram_post]
mmot test <file> [--update] [--golden-dir dir] [--tolerance 0.1]
mmot batch <template> --data <csv|json> [--output-dir dir] [--format mp4]
mmot interactive                                    # REPL mode
```

---

## Core Engine Modules

| Module | File | Purpose |
|--------|------|---------|
| Parser | `parser/mod.rs`, `parser/validate.rs` | JSON → Scene with serde_path_to_error, validation |
| Evaluator | `evaluator/interpolate.rs`, `evaluator/easing.rs`, `evaluator/modifiers.rs` | Keyframe interpolation, cubic bezier, spring, F-curve modifiers |
| Renderer | `renderer/mod.rs`, `renderer/layers.rs`, `renderer/effects.rs`, `renderer/masks.rs`, etc. | Skia CPU rendering, 13 blend modes, 9 effects, masks |
| Encoder | `encoder/av1.rs`, `encoder/mp4.rs`, `encoder/gif.rs`, `encoder/ffmpeg_mux.rs` | AV1 (rav1e), MP4 (muxide), GIF, WebM (ffmpeg) |
| Pipeline | `pipeline.rs` | Parallel frame rendering via rayon, scene evaluation |
| Diff | `diff.rs` | Semantic animation diffs (meta, layers, transforms, effects) |
| Accessibility | `accessibility.rs` | WCAG flash rate, contrast, motion intensity, text size |
| Export | `export.rs` | Multi-format export with safe zones (YouTube, TikTok, Instagram, etc.) |
| Batch | `batch.rs` | CSV/JSON data-driven batch video generation |
| Tokens | `tokens.rs` | Design token system ($token.name references) |
| Visual Test | `visual_test.rs` | Golden frame comparison for visual regression |
| Props | `props.rs` | ${variable} substitution |
| Schema | `schema/*.rs` | Scene, Layer, Transform, Effects, AnimatableValue, Easing, Transitions |

---

## Schema Features

**Layer types:** Solid, Text, Image, Video, Audio, Shape (rect/ellipse/polygon/line), Gradient (linear/radial), Composition, Null, Lottie

**Transform properties (all animatable):** position [x,y], scale [x,y], rotation (degrees), opacity (0-1)

**Animation:** Keyframes `[{"t":0,"v":value,"easing":"ease_out"},{"t":30,"v":value2}]`

**Easing:** linear, ease_in, ease_out, ease_in_out, cubic_bezier, spring

**Effects:** gaussian_blur, drop_shadow, glow, brightness_contrast, hue_saturation, invert, tint, fill

**Blend modes:** normal, multiply, screen, overlay, darken, lighten, color_dodge, color_burn, hard_light, soft_light, difference, exclusion, add

**Masks:** rect, ellipse, path — with mode (add/subtract/intersect/difference), feather, opacity

**AE features:** layer parenting, null objects, time remapping, track mattes, adjustment layers, motion blur, trim paths, path animation, AbsoluteFill (`fill: "parent"`)

**F-curve modifiers:** wiggle (noise), loop (repeat/ping-pong), clamp (min/max)

**Design tokens:** `$token.name` references resolved before parsing

**Sequences:** compositions with `sequence: true` + transitions (crossfade, wipe, slide)

---

## Editor (Vue 3 + Tailwind CSS 4)

**Design system:** Swiss futurism — Cosmos Blue #003049, Crimson #C1121F, Varden #FDF0D5, Blue Marble #669BBC. Fonts: JetBrains Mono (values), Inter (labels), Playfair Display (headings).

**Browser-only mode:** Default. Tauri IPC wrapped with runtime detection — no crashes without Tauri.

**Components:**
- Canvas preview with Canvas 2D renderer (evaluates keyframes, renders effects/blend/masks)
- Transform gizmos (drag to move, handles to scale)
- NLE timeline with Bars/Keys toggle (dope sheet view)
- Inline keyframe editor (diamond toggles, mini timeline strips)
- Easing curve picker (8 presets with visual thumbnails)
- Effects panel (9 effects, add/remove/configure)
- Mask editor (rect/ellipse/path, mode/feather/opacity)
- Property inspector (sliders for all transforms, shape/text/blend/fill controls)
- Export modal (MP4, GIF, WebM, Lottie, PNG, SVG, .mmot.json)
- Undo/redo (Ctrl+Z / Ctrl+Shift+Z, 50-level stack)
- Keyboard shortcuts (Space=play, Delete=remove layer)

**Dev server:** `cd editor && bun dev` (port 1420)

---

## Testing

- **226 tests** across mmot-core (213) + mmot-cli (13), all passing
- **48 .mmot.json files** validated (examples + fixtures + templates)
- **0 clippy warnings**
- Golden image tests for pixel-identical CPU rendering
- `mmot test` command for visual regression in CI

---

## Key Crates — Don't Reinvent

| Concern | Crate | Notes |
|---|---|---|
| 2D rendering | `skia-safe` | Skia via FFI. Feature-gated behind `native-renderer`. |
| GPU compute | `wgpu` | Planned. Vulkan/Metal/DX12/WebGPU. |
| Encoder (default) | `rav1e` | Pure Rust AV1. Feature-gated behind `native-renderer`. |
| Encoder (extended) | `ffmpeg-next` v8 | Feature flag `ffmpeg`. FFmpeg 7.1 headers. |
| WASM renderer | `tiny-skia` | Pure Rust 2D. Used by mmot-wasm. |
| Parallelism | `rayon` | Frame rendering is embarrassingly parallel. |
| JSON | `serde` + `serde_json` | Zero-copy deserialization. |
| Schema | `schemars` | Derive JSON Schema from types. |
| Errors (lib) | `thiserror` | |
| Errors (app) | `anyhow` | |
| CLI | `clap` | Derive macros. |
| Logging | `tracing` | |

---

## Distribution

| Package | Registry | Install |
|---------|----------|---------|
| mmot (CLI) | crates.io | `cargo install mmot` |
| mmot-core (library) | crates.io | `cargo add mmot-core` |
| mercury-motion (npm) | npmjs.com | `npm install mercury-motion` |
| mmot-mcp (MCP server) | npmjs.com | `npx -y mmot-mcp` |
| @mmot/react | repo | Remotion-compatible Scene builder |
| @mmot/player | repo | Embeddable web player |
| GitHub | antutroll27/mercury-motion | Source + releases |

---

## AI Integration

- **MCP server** (`mcp/server.js`): 7 tools — create_scene, render, validate, preview_frame, get_schema, list_effects, list_blend_modes
- **Skill file** (`skills/mercury-motion/SKILL.md`): 2,100+ lines with schema reference, design principles, anti-patterns, 9 animation patterns, 7 recipes
- **@mmot/react**: Remotion-compatible API — Scene builder, interpolate(), spring(), Effects, shape helpers
- **JSON Schema** (`schema/mmot.schema.json`): Auto-generated from Rust types, 1,700+ lines

---

## FilmTools Color Pipeline (Planned)

All color math runs in **scene-linear light**. Never transform in gamma-encoded space.

Pipeline order (strict):
1. Log decode (FLog2 / SLog3 / LogC4 / etc.) → scene-linear
2. Gamut matrix → target gamut
3. CDL (Slope/Offset/Power/Sat)
4. 3D LUT application
5. ACES RRT + ODT
6. agx-emulsion film response curve
7. Halation
8. Newson grain (wgpu compute shader)
9. Display gamma encode (Rec.709 / sRGB / PQ)

---

## WASM Constraints

- `native-renderer` feature MUST be disabled for WASM builds
- `ffmpeg-next` is NOT WASM-compatible
- `mmot-wasm` uses `tiny-skia` (pure Rust) instead of `skia-safe`
- No filesystem access — assets via fetch/File API
- Build: `wasm-pack build crates/mmot-wasm --target web`
- Excluded from workspace (`exclude = ["crates/mmot-wasm"]`)

---

## What NOT to Do

- Do not `unwrap()` in library code
- Do not add network calls to the renderer
- Do not use npm or yarn — Bun only
- Do not reference React, Zustand, or Redux — this project uses Vue 3 + Pinia
- Do not use `.mercury.json` — the format is `.mmot.json`
- Do not create a new crate named `mercury-*` — all crates are `mmot-*`
- Do not add async to the render pipeline — it's CPU-parallel via rayon, not async
- Do not auto-download ONNX models — models load from disk, explicit user action only
- Do not add Co-Authored-By Claude to git commits
