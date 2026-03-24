# Mercury-Motion — Agent Instructions

Read this before touching any file. Everything here is a resolved decision.

---

## What This Project Is

**Mercury-Motion** is a native Rust video creation engine and professional filmmaker toolkit.
Two products, one codebase: **Core** (JSON → video, like Remotion but 100× faster) and **FilmTools** (ACES color science, film emulation, AI-assisted VFX — zero generative AI).

Format: `.mmot.json` files describe videos. `mmot render file.mmot.json` produces MP4/WebM/GIF.
CLI binary: `mmot`. Desktop editor: Tauri 2.0 + Vue 3. Browser: WASM via wasm-pack.

---

## Repo Structure

```
crates/
  mmot-core/        # Core engine library (parser, evaluator, renderer, encoder)
  mmot-filmtools/   # FilmTools engine (color, emulation, VFX, audio)
  mmot-cli/         # mmot binary (clap)
  mmot-wasm/        # WASM bindings (wasm-bindgen)
editor/             # Vue 3 + TypeScript (Bun, Vite, Tailwind, Pinia)
tauri/              # Tauri 2.0 shell + IPC commands
tests/
  fixtures/         # .mmot.json test files (valid and invalid)
  golden/           # Reference pixel buffers for CPU render comparison
Cargo.toml          # Workspace root
```

---

## Non-Negotiable Decisions

**Frontend:** Vue 3 + TypeScript. State: Pinia. NOT React, NOT Zustand, NOT Redux.
**Package manager:** Bun. NEVER npm or yarn.
**Rust edition:** 2024 for all new crates.
**File format:** `.mmot.json` everywhere. Never `.mercury.json`.
**Error handling:**
  - Library crates (`mmot-core`, `mmot-filmtools`): `thiserror` — typed error enums only
  - Binary/app crates (`mmot-cli`, `mmot-wasm`): `anyhow` — context-chained messages
  - Never use `unwrap()` outside of tests. Use `expect("reason")` if you must.
**Logging:** `tracing` crate. No `println!` in library code.
**Parallelism:** `rayon` for CPU frame rendering. Single `wgpu` context per process.
**No network calls** from the renderer or FilmTools engine. Zero. Rendering is offline.
**No eval, no dynamic code** — `.mmot.json` is data, not code.

---

## Feature Flags

```toml
# mmot-core/Cargo.toml
[features]
default = []
audio-codec = ["opus"]                  # Opus audio encoding (requires C libopus)
gpu = ["wgpu"]                          # wgpu GPU backend (Vulkan/Metal/DX12)
ffmpeg = ["ffmpeg-next", "ffmpeg-sys-next"]  # Extended codec support
filmtools-ai = ["ort"]                  # ONNX model loading (Depth Anything, SAM2)
```

Default build uses CPU Skia renderer + rav1e encoder (zero C deps).
`--features audio-codec` adds Opus audio encoding (requires C compiler + cmake for libopus).
`--features ffmpeg` adds H.264/HEVC/ProRes support via libav.

---

## Testing Rules

- **Golden image tests:** CPU backend only. Pixel-identical output is the invariant.
  Location: `crates/mmot-core/tests/golden/`. Runner: compare raw RGBA buffers.
- **GPU tests:** Smoke only — render without crash, output is not blank.
- **Never mock the file system** in integration tests — use real fixture `.mmot.json` files.
- Fixture files live in `tests/fixtures/` (valid) and `tests/fixtures/invalid/` (error cases).
- Property-based tests for the keyframe evaluator (use `proptest`).

---

## Key Crates — Don't Reinvent

| Concern | Crate | Notes |
|---|---|---|
| 2D rendering | `skia-safe` | Skia via FFI. GPU-capable. |
| GPU compute | `wgpu` | Vulkan/Metal/DX12/WebGPU. Single context per process. |
| Encoder (default) | `rav1e` | Pure Rust AV1. Zero C deps. |
| Encoder (extended) | `ffmpeg-next` | Feature flag only. |
| Parallelism | `rayon` | Frame rendering is embarrassingly parallel. |
| JSON | `serde` + `serde_json` | Zero-copy deserialization. |
| Schema | `schemars` | Derive JSON Schema from types. |
| Color math | `palette` | Type-safe color space conversions. |
| Errors (lib) | `thiserror` | |
| Errors (app) | `anyhow` | |
| CLI | `clap` | Derive macros. |
| Logging | `tracing` | |
| Film log curves | Custom (mmot-filmtools/color/) | See color-science skill. |
| Film stabilization | `gyroflow-core` | Pure Rust + wgpu. Embed, don't wrap. |
| ONNX inference | `ort` | Feature flag `filmtools-ai`. |

---

## FilmTools Color Pipeline

All color math runs in **scene-linear light**. Never transform in gamma-encoded space.

Pipeline order (strict):
1. Log decode (FLog2 / SLog3 / LogC4 / etc.) → scene-linear
2. Gamut matrix → target gamut (e.g. F-Gamut → ARRI Wide Gamut 4)
3. CDL (Slope/Offset/Power/Sat)
4. 3D LUT application (GPU texture sample)
5. ACES RRT + ODT (via OCIO C FFI or baked LUT)
6. agx-emulsion film response curve
7. Halation
8. Newson grain (wgpu compute shader)
9. Display gamma encode (Rec.709 / sRGB / PQ)

See `~/.claude/skills/color-science/SKILL.md` for all log curve math and gamut matrices.

---

## WASM Constraints

- `ffmpeg-next` is NOT WASM-compatible. Use `ffmpeg.wasm` (JS-side) for browser export.
- `wasm-bindgen-rayon` + `SharedArrayBuffer` for parallel frame rendering in browser.
- AI models (SAM2, ESRGAN) are desktop-only. Depth Anything V2 tiny may be browser-viable.
- No filesystem access in WASM — assets via fetch API or File API.

---

## Skills to Load for This Project

| Task | Skill |
|---|---|
| Any Rust work | `actionbook/rust-skills` + `/sync-crate-skills` |
| Tauri IPC / distribution | `dchuk/claude-code-tauri-skills` |
| Color science / log curves | `~/.claude/skills/color-science` |
| FFmpeg / media | `~/.claude/skills/skill-rust-ffmpeg` |
| Code review | `superpowers:requesting-code-review` |
| Debugging | `superpowers:systematic-debugging` |
| New feature | `superpowers:test-driven-development` |
| Implementation plan | `superpowers:writing-plans` |

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
