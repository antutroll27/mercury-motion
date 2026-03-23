# Mercury-Motion — Technical Architecture
**Version:** 0.1.0-draft
**Date:** 2026-03-22
**Status:** Draft

---

## System Overview

Mercury-Motion is two products sharing one codebase:

1. **Mercury-Motion Core** — programmatic video creation engine (JSON → video)
2. **Mercury-Motion FilmTools** — professional color science and VFX toolkit

Both are built on a shared Rust rendering engine and share the same React frontend (desktop via Tauri, browser via WASM).

---

## Repository Structure

```
mercury-motion/
├── crates/
│   ├── mmot-core/          # Core engine library
│   │   ├── parser/            # JSON → typed Rust structs (serde)
│   │   ├── evaluator/         # Keyframe interpolation (frame N → FrameScene)
│   │   ├── renderer/          # FrameScene → pixel buffer (Skia / wgpu)
│   │   └── encoder/           # Pixel buffers → MP4/WebM/GIF (rav1e / ffmpeg-next)
│   │
│   ├── mmot-filmtools/     # FilmTools engine library
│   │   ├── color/             # ACES pipeline, OCIO bindings, log curves, LUT engine
│   │   ├── emulation/         # Film grain, halation, bloom, gate weave
│   │   ├── restoration/       # Stabilization, NR, deflicker, interpolation
│   │   ├── vfx/               # Depth DOF, chroma key, color match
│   │   └── audio/             # RNNoise denoising, audio sync
│   │
│   ├── mmot-cli/           # mmot CLI binary (clap)
│   └── mmot-wasm/          # WASM bindings (wasm-bindgen)
│
├── editor/                    # React 19 + TypeScript frontend
│   ├── src/
│   │   ├── editor/            # Core editor (timeline, properties, preview)
│   │   ├── filmtools/         # FilmTools UI (scopes, color science, VFX panels)
│   │   └── shared/            # Shared components (design system)
│   └── package.json           # Bun (not npm)
│
├── tauri/                     # Tauri 2.0 app shell + IPC commands
├── tests/
│   ├── fixtures/              # .mmot.json test files (valid + invalid)
│   └── golden/                # Reference renders for pixel-exact comparison
└── Cargo.toml                 # Workspace root
```

---

## Core Engine

### Data Flow

```
.mmot.json file
      │
      ▼
  [1] Parser (serde_json)
      │  Deserializes to typed Rust structs
      │  Validates schema (schemars)
      │  Returns actionable errors with JSON pointer paths
      ▼
  [2] Asset Resolver
      │  Fonts rasterized (Skia SkShaper)
      │  Images decoded (image crate)
      │  Video clips demuxed (ffmpeg-next / OxiMedia)
      │  Props substituted (${var} interpolation)
      ▼
  [3] Frame Dispatcher (rayon parallel iterator)
      │  Spawns N worker threads (one per logical CPU)
      │  Each thread handles a range of frames
      ▼
  [4] Keyframe Evaluator (pure function, per frame)
      │  Binary search + easing interpolation
      │  Returns flat DrawCommand list
      ▼
  [5] Renderer (Skia / wgpu)
      │  Paints layers in order (painter's algorithm)
      │  Applies blend modes, opacity, clipping, compositing
      │  Returns RGBA pixel buffer
      ▼
  [6] Encoder (rav1e / ffmpeg-next)
      │  Receives frames via mpsc channel (pipelined with rendering)
      │  Encodes to MP4 (H.264 / AV1) / WebM / GIF
      │  Muxes audio tracks
      ▼
  output.mp4
```

### Parallelism Model

Frame rendering is **embarrassingly parallel** — each frame is a pure function of the scene definition and the frame number N. No shared mutable state between frames.

```
rayon thread pool
├── Thread 0: frames 0–299
├── Thread 1: frames 300–599
├── Thread 2: frames 600–899
└── Thread 3: frames 900–1199
         │
         ▼ (all threads feed into)
    mpsc::channel → Encoder (single thread, pipelined)
```

GPU rendering uses a **single wgpu context** pipelined across frames. Multiple GPU contexts would create race conditions on GPU memory — avoided entirely by design.

### Key Crates

| Concern | Crate | Notes |
|---|---|---|
| JSON | `serde` + `serde_json` | Zero-copy deserialization |
| Schema | `schemars` | Derives schema from types |
| 2D rendering | `skia-safe` | Skia C++ via FFI; GPU capable |
| Text & Lottie | `skia-safe` (SkShaper + Skottie) | Unified pipeline |
| GPU | `wgpu` | Vulkan/Metal/DX12/WebGL/WebGPU |
| Encoder (default) | `rav1e` + pure-Rust MP4 mux | Zero C deps |
| Encoder (extended) | `ffmpeg-next` | Feature flag `--features ffmpeg` |
| Parallelism | `rayon` | Work-stealing thread pool |
| Image decode | `image` | PNG/JPEG/WebP/GIF |
| Errors (lib) | `thiserror` | Typed error enums |
| Errors (CLI) | `anyhow` | Context-chained messages |
| CLI | `clap` | Derive-macro parsing |
| Logging | `tracing` | Structured, async-aware |

---

## FilmTools Engine

### Color Science Pipeline

```
Input frame (log / RAW)
      │
      ▼
  [1] Input Device Transform (IDT)
      │  Camera-specific: ARRI LogC4, Sony S-Log3, RED Log3G10, etc.
      │  Implemented: OCIO v2 (C FFI) or native Rust matrix math
      │  Output: ACES AP0 scene-linear
      ▼
  [2] Look Transform (optional)
      │  CDL adjustments (Slope/Offset/Power/Saturation)
      │  LUT application (3D .cube / .3dl → GPU texture lookup via wgpu)
      ▼
  [3] Reference Rendering Transform (RRT)
      │  ACES RRT (OCIO v2 built-in)
      │  Tone maps scene-linear to output-referred
      ▼
  [4] Output Device Transform (ODT)
      │  Rec.709 / DCI-P3 / HDR10 / sRGB
      ▼
  Output frame (graded, display-referred)
```

All transforms run on GPU via `wgpu` compute shaders where possible. 3D LUT application is a single texture sample per pixel — negligible cost at any resolution.

### Film Emulation Pipeline

Reference implementations: **agx-emulsion** (spectral simulation from manufacturer datasheets) and **Newson et al. 2023** (publication-grade stochastic grain, GPU version available). Port algorithms to Rust + wgpu.

```
Graded frame (ACES / scene-linear)
      │
      ▼
  [1] Film response curve
      │  Spectral density → characteristic curve → dye transfer
      │  Reference: agx-emulsion spectral model (Kodak Vision3, Fuji Eterna)
      │  Simplified: S-curve from manufacturer densitometry in density space
      ▼
  [2] Halation
      │  Extract highlights above threshold (luma mask)
      │  Gaussian blur (sigma = scatter radius) → models light diffusion
      │  Red channel weighted most (red layer re-exposed by reflected light)
      │  Optional green-channel contribution for extreme cases
      │  Physics: light penetrates 3 emulsion layers, reflects off anti-halation
      │           backing, re-exposes red (and sometimes green) layer
      │  Reference: agx-emulsion, utility-dctls, halation-dctl
      ▼
  [3] Lens effects
      │  Bloom: Airy disk convolution on highlights
      │  Chromatic aberration: Brown-Conrady radial distortion, different
      │                        coefficients per R/G/B channel
      │  Vignetting: radial cos⁴ falloff
      ▼
  [4] Film grain (Newson et al. algorithm)
      │  Poisson process sampling in image space
      │  Grain size: log-normal distribution per film stock data
      │  Per-emulsion-layer (R/G/B) with different characteristics
      │  Exposure-dependent: fine grain in highlights, coarser in shadows
      │  Convolved with point spread function
      │  Applied in linear space before final tone mapping
      │  Reference: Newson et al. ACM ToG 2023, agx-emulsion
      ▼
  [5] Gate weave
      │  Sub-pixel frame offset: Gaussian noise on X/Y per frame
      │  Temporally coherent (per-frame seed, not per-pixel)
      ▼
  Output: emulated film frame
```

### AI-Assisted VFX Pipeline

These tools are invoked as background processes / ONNX inference, never blocking the render pipeline:

```
User selects "Depth DOF" tool on a clip
      │
      ▼
  [1] Depth Estimation
      │  Video Depth Anything (small, Apache 2.0)
      │  ONNX model → ort crate inference
      │  Output: per-frame depth maps (temporally consistent)
      ▼
  [2] Bokeh Synthesis
      │  Per-pixel Gaussian blur kernel scaled by |depth - focus_distance|
      │  Implemented as wgpu compute shader
      │  Parameters: focus distance, f-stop (controls blur radius)
      ▼
  Output: depth-of-field simulation

User clicks on object in frame for masking
      │
      ▼
  [1] SAM2 Inference
      │  Prompt: click coordinates (positive/negative points)
      │  SAM2 model → subprocess or ONNX → ort
      │  Output: binary mask for frame N
      ▼
  [2] Video Propagation (SAM2VideoPredictor)
      │  Streams mask forward/backward through video
      │  Output: per-frame alpha matte
      ▼
  Output: rotoscoped isolation mask
```

---

## Browser (WASM) Architecture

The same Rust core compiles to WASM via `wasm-pack`. The same React frontend deploys to the browser.

```
Browser
├── Main thread
│   └── React app (editor UI)
│       ├── Sends render commands to Web Worker
│       └── Receives ImageData frames → paints to <canvas>
│
├── Web Worker (mercury_wasm.js)
│   └── mmot-wasm (Rust → WASM)
│       ├── Parser + Evaluator (same code as native)
│       ├── Skia WASM renderer (official Google WASM build)
│       └── wasm-bindgen-rayon (parallel frames via SharedArrayBuffer)
│
└── Export Worker (lazy-loaded)
    └── ffmpeg.wasm
        └── Encodes ImageData frames → MP4/WebM for download
```

### WASM Constraints & Mitigations

| Constraint | Mitigation |
|---|---|
| `ffmpeg-next` (C) not WASM-compatible | Use `ffmpeg.wasm` (JS) for browser export; lazy-loaded only when user clicks Export |
| Default WASM is single-threaded | `wasm-bindgen-rayon` + `SharedArrayBuffer` enables parallel frame rendering in WASM threads |
| WebGPU not universally available | Default: Skia CPU backend. WebGPU: opt-in for Chrome 113+, Firefox 121+ |
| WASM binary size | Core WASM ~5–8 MB gzip. Heavy AI models (ESRGAN, SAM2) not bundled in browser — FilmTools AI features require desktop build |
| FilmTools AI inference in browser | Depth Anything V2 tiny model (~25MB) potentially WASM-viable; SAM2 too large for browser — desktop only |

---

## Desktop Architecture (Tauri 2.0)

```
Tauri App
├── Rust backend (Tauri commands)
│   ├── mmot-core (rendering, encoding)
│   ├── mmot-filmtools (color science, VFX)
│   ├── File I/O (.mmot.json read/write)
│   └── Preview: Rust render → base64 PNG → Tauri event → React
│
└── React frontend (Webview)
    ├── Editor page (timeline, properties, preview, asset browser)
    └── FilmTools page (scopes, color science, film emulation, VFX, audio)
```

### IPC Pattern

```rust
// Tauri command (Rust side)
#[tauri::command]
async fn render_preview_frame(
    scene: MercuryScene,
    frame: u32,
    state: State<'_, AppState>
) -> Result<String, String> {
    let buffer = state.renderer.render_frame(&scene, frame)?;
    Ok(base64::encode(&buffer))
}

// React side
const frame = await invoke('render_preview_frame', { scene, frame: currentFrame });
previewImg.src = `data:image/png;base64,${frame}`;
```

Preview pipeline Phase 3: base64 PNG via Tauri events (simple, correct).
Preview pipeline Phase 4+: shared GPU texture (wgpu surface → Tauri webview shared memory, zero-copy).

---

## Build System

### Rust
- **Package manager:** Cargo (workspace)
- **WASM build:** `wasm-pack build --target web`
- **Cross-compilation:** `cross` crate for CI (Windows/macOS/Linux from any host)
- **Feature flags:**
  - `--features ffmpeg` — enables `ffmpeg-next` extended codec support
  - `--features gpu` — enables wgpu GPU backend
  - `--features filmtools-ai` — enables ONNX model loading (Depth Anything, SAM2)

### Frontend
- **Runtime:** Bun (not npm)
- **Bundler:** Vite
- **Language:** TypeScript strict mode
- **Styling:** Tailwind CSS
- **State:** Zustand
- **JSON editor:** Monaco Editor (VS Code engine)

### CI — GitHub Actions Matrix

| Platform | Architecture | Tests |
|---|---|---|
| Windows | x86-64 | Unit, integration, golden images |
| macOS | arm64 (M-series) | Unit, integration, golden images |
| Ubuntu 22.04 | x86-64 | Unit, integration, golden images, WASM build |

Golden image tests run CPU backend only (pixel-identical output guaranteed). GPU tests: smoke only (render without crash, output not blank).

---

## Performance Targets

### Core Renderer (1080p, 30fps, 10-second video = 300 frames, 6-core CPU)

| Scenario | Target (CPU) | Target (GPU) |
|---|---|---|
| Simple text animation | < 4 sec | < 1 sec |
| Image + text overlay | < 6 sec | < 1.5 sec |
| Video clip + text | < 10 sec | < 2.5 sec |
| Complex (5 layers, animations) | < 15 sec | < 4 sec |

### FilmTools (per frame, 1080p)

| Operation | Target latency |
|---|---|
| ACES + LUT (GPU) | < 0.5 ms |
| Film grain + halation (GPU) | < 1 ms |
| Depth DOF (ONNX inference) | < 5 ms |
| SAM2 mask (ONNX inference) | < 50 ms |
| RNNoise audio denoising | Real-time (< 1x audio duration) |

### Memory Targets

| Scenario | Target peak RAM |
|---|---|
| 1080p render (default) | < 400 MB |
| 1080p render with FilmTools pipeline | < 700 MB |
| Browser WASM (1080p preview) | < 200 MB |

---

## Security Model

- **No network calls** from the renderer or FilmTools engine. Zero. Rendering is offline by design.
- **No eval, no dynamic code execution** — the JSON format is data, not code. No expression language.
- **ONNX models** are loaded from disk; no auto-download without explicit user action.
- **File access** is scoped to the Tauri allowlist — the renderer can only access paths the user explicitly opens.
- **WASM sandbox** — the browser WASM build has no filesystem access; assets are loaded via the fetch API from user-provided URLs or File API.

---

## Future Architecture Considerations

### Plugin System (Phase 6)
A Rust trait-based plugin system for custom effects and layer types. Plugins compile to native code (Rust crates) or WASM modules. The JSON format remains the source of truth — plugins add new `type` values to the layer schema.

### Self-Hostable Render API (Phase 6)
An HTTP server wrapping the `mmot-core` library. Accepts `.mmot.json` + props via POST, returns a video file. Stateless, horizontally scalable. Dockerized. Directly replaces Creatomate/Shotstack for self-hosters.

### Shared GPU Texture Preview (Phase 4+)
Replace base64 PNG preview with a zero-copy shared GPU texture between the wgpu renderer and the Tauri webview. Eliminates the encode/decode cycle on every frame for real-time playback.

---

*This document covers architectural decisions. For implementation decisions and their rationale, see the main design spec (2026-03-22-remotion-rust-design.md) Section 18: Resolved Decisions.*
