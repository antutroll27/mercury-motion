# Mercury-Motion — Design Specification
**Version:** 0.2.0-draft
**Date:** 2026-03-22
**Status:** Draft (post-review revision)

> Named after Mercury — the Roman god renowned for speed beyond human comprehension. That is the promise: video rendering so fast it feels mythological.

---

## Document Index

| Document | Path | Audience |
|----------|------|----------|
| **This file** — Technical Spec | `docs/superpowers/specs/2026-03-22-remotion-rust-design.md` | Engineers / contributors |
| **Integrations** | [`docs/technical/integrations.md`](../../technical/integrations.md) | Engineers building integrations |
| **Business** | [`docs/business/mercury-motion-business.md`](../../business/mercury-motion-business.md) | Founders / investors |
| **Marketing** | [`docs/marketing/mercury-motion-marketing.md`](../../marketing/mercury-motion-marketing.md) | Marketing / DevRel |

---

## Table of Contents

1. [The Vision](#1-the-vision)
2. [The Problem with Remotion](#2-the-problem-with-remotion)
3. [What Mercury-Motion Is](#3-what-mercury-motion-is)
4. [Who It's For](#4-who-its-for)
5. [Core Principles](#5-core-principles)
6. [Market Positioning](#6-market-positioning)
7. [Feature Overview](#7-feature-overview)
8. [Integrations Overview](#8-integrations-overview)
9. [Technical Architecture](#9-technical-architecture)
10. [The JSON Video Format](#10-the-json-video-format)
11. [Rendering Pipeline](#11-rendering-pipeline)
12. [The Editor UI](#12-the-editor-ui)
13. [CLI Tool](#13-cli-tool)
14. [Performance Model](#14-performance-model)
15. [Technology Stack](#15-technology-stack)
16. [Error Handling Strategy](#16-error-handling-strategy)
17. [Testing Strategy](#17-testing-strategy)
18. [Roadmap](#18-roadmap)
19. [Resolved Decisions](#19-resolved-decisions)
20. [Open Questions](#20-open-questions)

**Appendices**
- [Appendix A: Comparison with Remotion](#appendix-a-comparison-with-remotion)
- [Appendix B: Why Not Extend Remotion?](#appendix-b-why-not-extend-remotion)
- [Appendix C: Mercury-Motion FilmTools](#appendix-c-mercury-motion-filmtools)
- [Appendix D: AI Development Skills & Tooling](#appendix-d-ai-development-skills--tooling)

---

## 1. The Vision

> *"Video should be as easy to write as code, and as fast to render as a GPU can go."*

Mercury-Motion is a Rust-native programmatic video creation engine. It replaces the headless Chrome, React, and Node.js stack of Remotion with a deterministic, GPU-accelerated Rust renderer driven entirely by human-readable JSON files.

You describe a video in a `.mmot.json` file. Mercury-Motion renders it. No browser. No JavaScript. No npm. Just a JSON file, a binary, and your imagination.

The editor — a native desktop app — lets you build and tweak that JSON visually in real time. When you're happy, you export. Rendering is fast. Not "fast for a browser-based tool" fast. Just fast.

---

## 2. The Problem with Remotion

Remotion is a genuinely clever idea — use React components as a frame-rendering model — but it inherits every limitation of the browser:

### Speed

Remotion renders video by launching headless Chrome, navigating it to each frame, waiting for it to paint, then screenshotting it. For a 30fps, 60-second video at 1080p: that is **1,800 Chrome screenshots** before FFmpeg even starts encoding.

Each screenshot takes 40–200ms. Even with concurrency, a 2-minute video can take 20–40 minutes to render on a typical developer machine. A Rust GPU renderer produces the same frame in under 1ms.

### Fragility

Chrome was not built to be a video renderer. Remotion has to *lie to the browser about what time it is* — overriding `Date.now()`, `performance.now()`, `requestAnimationFrame`, and CSS animation timers to freeze time at exactly the current frame. When anything slips through (a third-party animation library, a CSS background image, an async API) — you get flickering, non-deterministic frames, and corrupted renders.

### Complexity

Remotion is a React app. To make a video, you need Node.js, npm, webpack, and a full React toolchain. A simple "company intro" video has the same setup overhead as a production web application.

### Licensing

Remotion is source-available, not open source. Teams of 4+ people need a paid license. This has driven forks (Revideo) and alternatives, but none escape the core headless Chrome bottleneck.

### The UI

Remotion Studio is functional but dated. The preview is browser-rendered. The timeline is minimal. There is no visual keyframe editor. Properties are edited by writing TypeScript.

---

## 3. What Mercury-Motion Is

Mercury-Motion is three things:

### 3.1 A JSON Video Format (`.mmot.json`)

A clean, human-readable, versionable JSON schema that fully describes a video: compositions, layers, animations, keyframes, text, images, videos, audio, and transitions. The file on disk *is* the project. No transpilation, no build step, no runtime.

This makes Mercury-Motion videos:
- **Git-friendly** — diff a video like you diff code
- **AI-friendly** — LLMs can generate and edit JSON trivially
- **Tool-friendly** — any language can read/write the format
- **Template-friendly** — swap props to produce 1,000 personalized videos from one file

### 3.2 A Native Renderer (CLI + Library)

A Rust binary (`mmot render`) that takes a `.mmot.json` file and produces an MP4, WebM, or GIF. The renderer uses GPU-accelerated 2D graphics via Skia and encodes natively. Zero runtime dependencies are required for the default MP4 output path.

No Chrome. No Node. A single self-contained binary.

### 3.3 A Native Desktop Editor

A native desktop application (Tauri shell, Rust backend) with a modern timeline-based editor. Real-time GPU preview. Visual keyframe editor. Property panels. Asset browser. JSON split-view. The editor reads and writes `.mmot.json` — the file format is the source of truth, not an internal database.

---

## 4. Who It's For

### Primary: Developer-creators

Software engineers who want to produce programmatic videos — data visualizations, code explainers, product demos, social media content — without a video editing background and without a React/browser toolchain. They want to write a file, run a command, get a video.

**Their pain today:** Remotion's React/npm overhead is a tax. They want the *idea* of Remotion without the browser.

### Secondary: Content teams using templates

Marketing, growth, and content teams that receive video templates from a developer and need to fill in data — names, numbers, URLs, images — to produce batches of personalized videos. They use the editor visually; the developer writes the template once.

**Their pain today:** Cloud tools (Creatomate, Shotstack) are SaaS-priced and require API calls. They want something local and offline.

### Tertiary: AI/automation pipelines

Agents, pipelines, and scripts that need to generate videos programmatically at scale. The JSON format is ideal for LLM generation. The CLI is ideal for batch pipelines.

**Their pain today:** No open-source, self-hostable, fast, JSON-native video renderer exists.

---

## 5. Core Principles

**1. The file is the project.**
A `.mmot.json` file is a complete, portable video project. No project directories, no binary blobs, no build artifacts required to open it. Assets can be referenced by relative path or embedded inline as base64 for maximum portability.

**2. Determinism is not optional.**
Given the same JSON and the same frame number, the renderer always produces identical pixels. No timing hacks, no browser quirks, no non-determinism.

**3. Speed is a feature.**
Mercury-Motion should render 1080p60 video faster than Remotion renders 720p30. This is not a stretch goal — it is an architectural consequence of not using a browser.

**4. JSON is the API.**
Every capability of the engine must be expressible in JSON. The editor, CLI, and library all speak the same format. There are no hidden features that only the editor can use.

**5. Zero mandatory runtime dependencies.**
`mmot render my-video.mmot.json` works on a fresh machine with no additional installs. The binary statically links everything it needs for the default encode path. Optional codecs may require system libraries but are never required for basic use.

**6. Open source, forever.**
Mercury-Motion is MIT-licensed. No license tiers. No "source available for large teams." The core tool is and always will be free.

---

## 6. Market Positioning

| Tool | Speed | Format | Open Source | No-Browser | Native App |
|------|-------|--------|-------------|------------|------------|
| **Mercury-Motion** | ★★★★★ | JSON | MIT | ✓ | ✓ |
| Remotion | ★★ | React/TSX | Source-avail. | ✗ | ✗ |
| Motion Canvas | ★★★ | TypeScript | MIT | Partial | ✗ |
| Revideo | ★★★ | TypeScript | MIT | Partial | ✗ |
| Shotstack | ★★★ | JSON (SaaS) | ✗ | ✓ | ✗ |
| Creatomate | ★★★ | JSON (SaaS) | ✗ | ✓ | ✗ |
| After Effects | ★★★★ | Binary | ✗ | ✓ | ✓ |

Mercury-Motion occupies a unique position: **the only open-source, JSON-native, browser-free, native-speed programmatic video tool with a desktop editor.**

---

## 7. Feature Overview

### MVP (v0.1)
- `.mmot.json` format with solid, text, image, and video layers
- Keyframe animation with linear/ease/cubic-bezier easing
- CLI renderer: `mmot render <file.mmot.json>`
- MP4 output (zero C-dependency encode path)
- Basic desktop editor: timeline, preview, property panel

### v0.2
- GIF and WebM output
- Audio layer support (background music, voiceover)
- Nested compositions (precomps)
- Text animation presets (fade-in, slide-up, typewriter)
- Lottie layer support (via Skia Skottie)

### v0.3
- Props/variables system for template-driven rendering
- `mmot render --props '{"name":"Alice"}'` for batch rendering
- Transition effects between layers (fade, slide, wipe)
- Shape layer (rectangle, ellipse, Bézier path)
- `mmot pack` — embed all assets inline as base64

### v1.0
- Full keyframe curve editor in the editor UI
- Plugin/effect system
- WebAssembly renderer build (run Mercury-Motion in the browser)
- Self-hostable cloud render API

---

## 8. Integrations Overview

Mercury-Motion is built to integrate with the tools and services that developers and creators already use. The full technical specification for each integration is in [`docs/technical/integrations.md`](../../technical/integrations.md).

### The Resolver Plugin System

All external asset integrations share a unified **URI resolver** architecture. Any `src` field in a `.mmot.json` can use a URI scheme rather than a file path:

```json
{ "type": "image",  "src": "unsplash://photo-abc123" }
{ "type": "audio",  "src": "elevenlabs://voice-xyz?text=${narration}" }
{ "type": "text",   "font": { "family": "gfont://Inter:wght@400;700" } }
{ "type": "lottie", "src": "pexels://animation/456" }
```

The resolver intercepts unrecognised URI schemes, fetches or generates the asset, caches it locally (`~/.mmot/cache/`), and hands a resolved file path back to the renderer. API keys live in `~/.mmot/config.toml` or environment variables. Custom resolvers can be added as shared library plugins via a stable C ABI.

### Integrations at a Glance

| Integration | Type | When |
|-------------|------|------|
| **Lottie / dotLottie** | Native layer type | v0.2 |
| **Google Fonts** | URI resolver + CLI | v0.2 |
| **Adobe After Effects** | Export plugin (UXP) | v0.3 |
| **Figma** | Export plugin | v0.3 |
| **Unsplash** | URI resolver | v0.3 |
| **Pexels** | URI resolver (photos + video) | v0.3 |
| **ElevenLabs** | URI resolver (audio layer) | v0.3 |
| **Social media presets** | Built-in templates | v0.3 |
| **REST Render Server** | `mmot server` CLI command | v1.0 |
| **VSCode Extension** | IDE integration | v1.0 |
| **JavaScript / TypeScript SDK** | npm `@mercury-motion/sdk` | v1.0 |
| **Python SDK** | pip `mercury-motion` | v1.0 |

---

## 9. Technical Architecture

```
┌───────────────────────────────────────────────────────┐
│              Mercury-Motion System                    │
├──────────────┬────────────────┬──────────────────────┤
│  Editor UI   │   CLI Tool     │  Library (mmot)      │
│ (Tauri app)  │  (mmot bin)    │  (Rust crate)        │
├──────────────┴────────────────┴──────────────────────┤
│               Core Engine (Rust)                     │
│  ┌───────────┐  ┌─────────────┐  ┌───────────────┐  │
│  │  Parser   │  │  Evaluator  │  │   Renderer    │  │
│  │(serde +   │  │ (timeline + │  │ (skia-safe)   │  │
│  │ schemars) │  │  keyframes) │  │               │  │
│  └───────────┘  └─────────────┘  └───────────────┘  │
│  ┌────────────────────────────────────────────────┐  │
│  │        Encoder (rav1e / ffmpeg-next)           │  │
│  └────────────────────────────────────────────────┘  │
├───────────────────────────────────────────────────────┤
│          .mmot.json  ←→  Asset Files                 │
└───────────────────────────────────────────────────────┘
```

### Component Responsibilities

| Component | Responsibility |
|-----------|---------------|
| **Parser** | Deserializes `.mmot.json` into typed Rust structs. Validates schema. Reports actionable errors with JSON pointer paths. |
| **Evaluator** | Given a parsed scene and frame `N`, resolves all animatable values via keyframe interpolation. Produces a flat `FrameScene`. Pure function — no side effects. |
| **Renderer** | Takes a `FrameScene` and draws it to a pixel buffer using Skia (CPU or GPU backend). Handles layer compositing in visual stack order, blend modes, opacity, clipping. |
| **Encoder** | Receives the pixel buffer stream from the renderer and encodes to the target format. Runs in a pipeline concurrent with rendering. |
| **Editor (Tauri)** | Vue frontend for the editor UI. Communicates with Core Engine via Tauri IPC. Preview frames are passed as base64 PNG (MVP) or shared GPU texture (v1.0). |
| **CLI** | Thin wrapper around the Core Engine. Parses args, invokes `mmot::render()`, streams progress to stdout. |

---

## 10. The JSON Video Format

The format is designed to be readable and writable by humans and machines, expressing everything a video needs without code. The file extension is `.mmot.json`.

### Top-Level Structure

```json
{
  "$schema": "https://mercury-motion.dev/schema/v1.json",
  "version": "1.0",
  "meta": {
    "name": "My Video",
    "width": 1920,
    "height": 1080,
    "fps": 30,
    "duration": 150,
    "background": "#000000",
    "root": "main"
  },
  "props": {
    "title": { "type": "string", "default": "Hello World" },
    "accent": { "type": "color", "default": "#6c63ff" }
  },
  "compositions": { "...": "..." },
  "assets": { "...": "..." }
}
```

- `duration` is always in **frames** — avoids floating-point fps ambiguity.
- `root` names the top-level composition to render.
- Both `$schema` (editor/tooling autocomplete) and `"version"` (runtime compatibility) are present.

### Layer Types

| Type | Description |
|------|-------------|
| `solid` | A filled rectangle. Background plates, color blocks. |
| `image` | A raster image (PNG, JPEG, WebP). `src` accepts relative paths or `data:image/...;base64,...` inline. |
| `video` | A video clip with optional trim points. |
| `text` | Styled text. Rendered via Skia SkShaper. |
| `audio` | An audio file (MP3, WAV, FLAC). No visual output. |
| `lottie` | A Lottie/dotLottie animation rendered via Skia Skottie. |
| `composition` | A nested composition reference (precomp). |
| `shape` | A vector shape (rectangle, ellipse, polygon, Bézier path). |

### Layer Visual Stack Order

Layers in the `layers` array are ordered **bottom-to-top of the visual stack**. The first layer is drawn first and appears *behind* all later layers. This matches After Effects track order.

```
layers: [ bg_plate, logo, title_text, overlay ]
         ▲                                    ▲
     drawn first                          drawn last
     (furthest back)                      (in front)
```

### Animatable Values

Any numeric, vector, or color property can be static or animated.

**Disambiguation rule (enforced at parse time):** If the JSON value is an array whose first element is an object containing a `"t"` field, it is a **keyframe array**. Otherwise it is a **static value** — a number, `[x, y]` vector, or color string.

**Static examples:**
```json
"opacity": 1.0
"position": [960, 540]
"scale": [1.0, 1.0]
```

**Animated examples:**
```json
"opacity": [
  { "t": 0,   "v": 0.0, "easing": "ease_in" },
  { "t": 15,  "v": 1.0 },
  { "t": 120, "v": 1.0 },
  { "t": 135, "v": 0.0 }
]

"position": [
  { "t": 10, "v": [960, 620], "easing": "ease_out" },
  { "t": 25, "v": [960, 540] }
]
```

**Keyframe fields:**
- `t` — frame number (integer ≥ 0, ≤ `meta.duration`).
- `v` — value at this keyframe. Type depends on property: scalar `number`, `[x, y]` array, or `"#rrggbb[aa]"` color.
- `easing` — optional. Curve from *this* keyframe to the *next*. Ignored on the final keyframe (no following segment to apply to). Defaults to `"linear"`.

**Easing values:**
```json
"easing": "linear"
"easing": "ease_in"
"easing": "ease_out"
"easing": "ease_in_out"
"easing": { "type": "cubic_bezier", "x1": 0.4, "y1": 0.0, "x2": 0.2, "y2": 1.0 }
```

`cubic_bezier` uses a structured object — fully JSON Schema validatable, no secondary string parsing.

**Evaluation rules:**
- Before first keyframe → first value held.
- After last keyframe → last value held.
- Between keyframes → interpolated using the easing of the *from* keyframe.

### Props Interpolation

`${prop_name}` substitution works in text content, `src` paths, and color values.

**Prop types and validation (enforced at parse time):**

| Type | Valid values |
|------|-------------|
| `string` | Any string |
| `color` | `#rgb`, `#rrggbb`, `#rrggbbaa` |
| `number` | JSON number |
| `url` | Any string |

Passing a non-color to a `"type":"color"` prop is a hard error. Missing required props (no `default`) at render time is a hard error. Unknown keys in `--props` produce a warning.

### Asset References

`src` fields accept:
- **Relative paths**: `"./assets/logo.png"` — resolved relative to the `.mmot.json` file.
- **Absolute paths**: used as-is.
- **Inline base64**: `"data:image/png;base64,..."` — fully self-contained.

`mmot pack` resolves all relative paths to inline base64, producing a single portable file.

### Full Schema Example

```json
{
  "$schema": "https://mercury-motion.dev/schema/v1.json",
  "version": "1.0",
  "meta": {
    "name": "Product Launch",
    "width": 1920,
    "height": 1080,
    "fps": 30,
    "duration": 300,
    "background": "#0f0f23",
    "root": "main"
  },
  "props": {
    "product_name": { "type": "string", "default": "Acme Widget" },
    "tagline":      { "type": "string", "default": "Built different." },
    "brand_color":  { "type": "color",  "default": "#6c63ff" }
  },
  "compositions": {
    "main": {
      "layers": [
        {
          "id": "bg-plate",
          "type": "solid",
          "in": 0, "out": 300,
          "color": "#0f0f23",
          "transform": {
            "position": [960, 540],
            "scale": [1.0, 1.0],
            "opacity": 1.0
          }
        },
        {
          "id": "logo",
          "type": "image",
          "src": "./assets/logo.png",
          "in": 0, "out": 300,
          "transform": {
            "position": [960, 200],
            "scale": [
              { "t": 0,  "v": [0.8, 0.8], "easing": "ease_out" },
              { "t": 20, "v": [1.0, 1.0] }
            ],
            "opacity": [
              { "t": 0,  "v": 0.0 },
              { "t": 15, "v": 1.0 }
            ]
          }
        },
        {
          "id": "product-name",
          "type": "text",
          "in": 20, "out": 280,
          "text": "${product_name}",
          "font": { "family": "Inter", "size": 80, "weight": 800, "color": "#ffffff" },
          "align": "center",
          "transform": {
            "position": [960, 520],
            "opacity": [
              { "t": 20, "v": 0.0 },
              { "t": 35, "v": 1.0 }
            ]
          }
        },
        {
          "id": "tagline",
          "type": "text",
          "in": 35, "out": 270,
          "text": "${tagline}",
          "font": { "family": "Inter", "size": 36, "weight": 400, "color": "${brand_color}" },
          "align": "center",
          "transform": {
            "position": [960, 620],
            "opacity": [
              { "t": 35, "v": 0.0 },
              { "t": 50, "v": 1.0 }
            ]
          }
        },
        {
          "id": "background-music",
          "type": "audio",
          "src": "./assets/music.mp3",
          "in": 0, "out": 300,
          "volume": [
            { "t": 0,   "v": 0.0 },
            { "t": 30,  "v": 0.8 },
            { "t": 270, "v": 0.8 },
            { "t": 300, "v": 0.0 }
          ]
        }
      ]
    }
  },
  "assets": {
    "fonts": [
      { "id": "inter", "src": "./fonts/Inter-Variable.ttf" }
    ]
  }
}
```

---

## 11. Rendering Pipeline

```
.mmot.json
      │
      ▼
  [1] Parse & Validate
      │  serde_json → typed Rust structs
      │  schemars JSON Schema validation
      │  Props type validation
      │  Errors: MmotError::Parse with JSON pointer path
      ▼
  [2] Asset Resolution
      │  Fonts: Skia font manager
      │  Images: image crate (PNG/JPEG/WebP)
      │  Video: ffmpeg-next demux → frame cache
      │  Audio: decoded into sample buffers
      │  Errors: MmotError::AssetNotFound / AssetDecode
      ▼
  [3] CPU Frame Dispatch  (rayon parallel)
      │  rayon::par_iter over 0..duration
      │    ├─ [4] Keyframe eval → FrameScene  (parallel, CPU)
      │    └─ Send FrameScene to render channel
      ▼
  [4] Keyframe Evaluator  (pure function, per frame)
      │  Binary-search keyframes per AnimatableValue
      │  Interpolate using easing of the from-keyframe
      │  Resolve ${prop_name} substitutions
      │  Returns FrameScene: all values as concrete types
      ▼
  [5] Renderer  (skia-safe)
      │
      │  CPU backend:
      │    Each rayon thread owns its own Skia CPU surface.
      │    Fully parallel: N frames render simultaneously.
      │
      │  GPU backend:
      │    Single wgpu device + single Skia GPU context.
      │    CPU eval (rayon, parallel) → bounded channel
      │    → one dedicated thread drains channel and submits
      │      Skia draw calls sequentially to the GPU context.
      │    CPU eval and GPU draw overlap (pipeline):
      │    while GPU draws frame N, CPU evaluates N+1..N+K.
      │    Throughput is limited by the slower stage.
      │
      │  Both: draw in visual stack order (first = back),
      │  apply blend modes, opacity, clipping → RGBA buffer
      ▼
  [6] Encoder  (concurrent pipeline)
      │  Receives ordered RGBA frames via mpsc channel
      │  Default (zero C deps): rav1e + pure-Rust MP4 muxer
      │  Optional (--features ffmpeg): ffmpeg-next
      │  Audio muxed in final pass
      │  Output file written only on full success
      ▼
  output.mp4 / output.webm / output.gif
```

### CPU Parallelism

Frame evaluation and CPU rendering are embarrassingly parallel — each frame is a pure function of the scene and a frame number. `rayon` work-stealing iterates over all frames. Each thread owns its own Skia CPU surface (surfaces are not shared). Scales linearly with core count.

### GPU Pipelining

The GPU backend does **not** use rayon for draw calls. `wgpu` and Skia GPU contexts require single-threaded ownership. The pipelined model (CPU eval parallel → single GPU draw thread) provides the throughput benefit without multi-context races. The ~0.3ms/frame GPU figure is GPU draw time per frame in this pipelined model.

### Lottie Rendering

Lottie layers use **Skia Skottie** (built into `skia-safe`). Skottie renders directly into the Skia surface — same quality, same blend modes, same GPU path as every other layer. No second rendering engine.

---

## 12. The Editor UI

A **Tauri 2.0** desktop application. Frontend: **Vue 3 + TypeScript** with Tailwind CSS. Rust backend handles rendering, file I/O, encoding. Frontend handles all interaction.

### Design Philosophy

Dark, dense, professional. DaVinci Resolve's layout clarity meets Linear's design language. Not a toy. A tool that looks capable of serious work.

### Layout

```
┌──────────────────────────────────────────────────────────┐
│  Mercury-Motion               [Pack] [Export] [Settings] │
├─────────────┬────────────────────────┬───────────────────┤
│             │                        │                   │
│   Assets    │    Preview Viewport    │   Properties      │
│   Browser   │    (Rust rendered)     │   Panel           │
│             │                        │                   │
│  [Files]    │   ┌──────────────┐     │  Layer: title     │
│  [Fonts]    │   │              │     │  ─────────────    │
│  [Lottie]   │   │   Frame N    │     │  Text: ${title}   │
│             │   │              │     │  Font: Inter 72px │
│             │   └──────────────┘     │  Color: #ffffff   │
│             │   ◀ ▶  [00:03:15]      │                   │
│             │   ────●────────────    │  Transform ▾      │
├─────────────┴────────────────────────┴───────────────────┤
│                    Timeline                               │
│  ┌──────────┬───────────────────────────────────────────┐│
│  │ bg-plate │████████████████████████████████████████  ││
│  │ logo     │   ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░        ││
│  │ title    │         ◆──────────────────◆             ││
│  │ audio    │ ≋≋≋≋≋≋≋≋≋≋≋≋≋≋≋≋≋≋≋≋≋≋≋≋≋≋≋≋≋≋≋≋≋≋≋≋≋≋≋  ││
│  └──────────┴───────────────────────────────────────────┘│
└──────────────────────────────────────────────────────────┘
```

Timeline layer order mirrors the visual stack: first layer in `layers` = bottom row (furthest back).

### Key Panels

**Preview Viewport**
- Renders via the Rust renderer through Tauri IPC.
- MVP: RGBA buffer → base64 PNG → Vue `<img>`.
- v1.0: zero-copy shared GPU texture via WebGL.
- WYSIWYG: identical code path to export.
- Real-time scrubbing. Playback up to display refresh rate.

**Timeline Panel**
- One row per layer. Bottom layer = bottom row.
- Drag blocks to shift in/out. Resize handles for trim.
- Keyframe ◆ visible on property sub-tracks.
- Horizontal zoom via scroll/pinch. Composition collapse.

**Properties Panel**
- Static props: input fields.
- Animated props: current interpolated value + ◆ to add keyframe at current frame.
- Colors: hex picker + alpha.
- Easing: dropdown + visual curve preview for `cubic_bezier`.

**Asset Browser**
- Images, video, audio, fonts, Lottie files.
- Drag to timeline to create layer at playhead.
- Lottie: thumbnail preview on hover.

**JSON Split View**
- Toggle `[Visual]` / `[JSON]`.
- Monaco Editor with `schemars`-generated JSON Schema autocomplete.
- Bidirectional live sync. Invalid JSON highlights inline; visual view freezes until valid.

### Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Space` | Play/pause |
| `←` / `→` | Previous/next frame |
| `Shift+←/→` | Jump 10 frames |
| `K` | Add keyframe at current frame |
| `Ctrl+Z` / `Ctrl+Shift+Z` | Undo / Redo |
| `Ctrl+E` | Export |
| `Ctrl+J` | Toggle JSON view |
| `[` / `]` | Trim layer in/out to playhead |
| `Delete` | Delete selected layer |

### Cross-Platform Notes

- **Windows**: WebView2 (Chromium-based).
- **macOS**: WKWebView.
- **Linux**: WebKitGTK (`libwebkit2gtk-4.1` required).
- CPU backend produces **pixel-identical output** on all platforms. GPU backend may have minor floating-point variance between Vulkan/Metal/DX12 drivers; excluded from pixel-exact golden tests.

---

## 13. CLI Tool

```
mmot 0.1.0 — Mercury-Motion programmatic video renderer

USAGE:
    mmot <COMMAND>

COMMANDS:
    render    Render a .mmot.json file to video
    validate  Validate a .mmot.json file without rendering
    pack      Embed all asset files inline as base64
    preview   Open the editor with a file
    new       Scaffold a new .mmot.json project
    help      Print this message

RENDER OPTIONS:
    mmot render <FILE> [OPTIONS]

    -o, --output <FILE>       Output file (default: <input-name>.mp4)
    -f, --format <FORMAT>     mp4 | webm | gif  (default: mp4)
    -q, --quality <N>         Quality 1–100 (default: 80)
    --props <JSON|@FILE>      Props as JSON string or @path/to/props.json
    --frames <START>-<END>    Render only this frame range (e.g. 0-90)
    --concurrency <N>         CPU workers (default: logical CPU count)
    --backend <BACKEND>       cpu | gpu  (default: cpu)
    --no-audio                Skip audio tracks
    -v, --verbose             Per-frame timing and progress
```

**Exit codes:**
- `0` — Success
- `1` — Render / encoding error
- `2` — Parse / schema validation error (with JSON pointer path on stderr)
- `3` — Asset not found (with resolved path on stderr)

No partial output file is ever written on failure.

### Batch Rendering Example

```bash
jq -c '.[]' data/users.json | while read props; do
  name=$(echo "$props" | jq -r '.name')
  mmot render template.mmot.json \
    --props "$props" \
    -o "output/video-$name.mp4"
done
```

---

## 14. Performance Model

### Target Benchmarks (1080p, 30fps, 10s = 300 frames, 6-core CPU)

| Scenario | Remotion | mmot CPU | mmot GPU |
|----------|----------|----------|----------|
| Simple text animation | ~8 min | ~3 sec | ~0.8 sec |
| Image + text overlay | ~12 min | ~5 sec | ~1.2 sec |
| Video clip + text | ~20 min | ~8 sec | ~2 sec |
| Complex (5 layers, animations) | ~35 min | ~12 sec | ~3 sec |

*Basis: Remotion ~50ms/frame (Chrome screenshot). mmot CPU ~2ms/frame (Skia, 6 workers). mmot GPU ~0.3ms/frame (pipelined). Encoding adds ~1–4s. Pre-implementation estimates; measured benchmarks published after Phase 1.*

### Memory

| Tool | Peak RAM (1080p render) |
|------|------------------------|
| Remotion | ~2–4 GB (Chrome instances) |
| Mercury-Motion | ~200–400 MB |

### Binary Size

| Build | Target |
|-------|--------|
| Skia CPU + rav1e + pure-Rust mux | ~30 MB |
| + GPU backend | ~60–80 MB |
| + `--features ffmpeg` (static libav*) | ~100–120 MB |

---

## 15. Technology Stack

### Core Engine (Rust)

| Concern | Crate | Decision |
|---------|-------|----------|
| JSON parsing | `serde` + `serde_json` | Industry standard |
| JSON Schema | `schemars` | Derives schema from types; shared with editor autocomplete |
| 2D rendering | `skia-safe` | Mature; GPU; Skia text (SkShaper); Skottie |
| Text rendering | **Skia SkShaper** (in `skia-safe`) | Single text pipeline; GPU-capable; HarfBuzz Unicode |
| Lottie | **Skia Skottie** (in `skia-safe`) | Renders into same surface; no second engine |
| GPU backend | `wgpu` | Vulkan/Metal/DX12/WebGL/WebGPU |
| Default encoder | `rav1e` + pure-Rust MP4 muxer | Zero C dependencies |
| Extended codecs | `ffmpeg-next` (`--features ffmpeg`) | Opt-in for ProRes, HEVC, VP9, etc. |
| Parallelism | `rayon` | Work-stealing; CPU frame eval + CPU rendering |
| Image decoding | `image` | PNG/JPEG/WebP/GIF; pure Rust |
| Errors (library) | `thiserror` | Typed error enum |
| Errors (CLI) | `anyhow` | Context-chained error messages |
| CLI | `clap` | Derive-macro argument parsing |
| Logging | `tracing` | Structured, async-aware |

**Explicitly excluded from v0.x:**
- `cosmic-text` — would create a second text layout engine alongside Skia SkShaper. Tracked for future evaluation as a pure-Rust fallback.
- `dotlottie-rs` — Skia Skottie makes it redundant; adding it would create a second renderer.

### Desktop Editor (Tauri 2.0)

| Concern | Technology | Reason |
|---------|-----------|--------|
| App shell | Tauri 2.0 | Lightweight; Rust backend; `.exe`/`.dmg`/`.AppImage` |
| Frontend | Vue 3 + TypeScript | Team familiarity; Composition API ideal for reactive editor state |
| Styling | Tailwind CSS | Utility-first dark theme |
| Timeline UI | Custom Vue component | Full control over interaction model |
| JSON editor | Monaco Editor | VS Code engine; JSON Schema autocomplete |
| State | Pinia | Vue-native; intuitive for reactive state trees |
| IPC | Tauri commands + events | Rust functions as async JS calls |

### Browser (WASM) Target

Browser support is a **first-class target**, not a stretch goal. Most Remotion users live in the browser. Mercury-Motion must run there too — and run better.

**Architecture:**

```
Mercury-Motion (browser)
├── Core renderer (Rust) → compiled to WASM via wasm-pack
│   ├── Runs in a Web Worker (off main thread, no UI jank)
│   ├── skia-safe → Skia compiled to WASM (official Google build)
│   └── wgpu → WebGPU backend (Chrome 113+, Firefox 121+ behind flag)
├── Same Vue frontend as the desktop editor
│   ├── Sends render commands to the WASM worker
│   └── Receives ImageData frames back → paints to <canvas>
└── Encoding
    ├── ffmpeg.wasm for MP4/WebM export
    └── Native GIF encoder (pure Rust WASM-compatible)
```

**Why Mercury-Motion in WASM beats Remotion in browser:**

Remotion renders using the browser's own paint engine (DOM + CSS). Mercury-Motion in WASM renders with its own Skia engine *inside* the browser — completely bypassing the DOM renderer. Same deterministic, frame-exact output as the native build. No timing hacks. No CSS leaking into frames.

**Constraints & Mitigations:**

| Constraint | Mitigation |
|---|---|
| `ffmpeg-next` won't compile to WASM | Use `ffmpeg.wasm` (JS-side) for browser export only |
| WASM is single-threaded by default | Use `wasm-bindgen-rayon` + `SharedArrayBuffer` for parallel frame rendering |
| WebGPU not universally supported | Fallback to Skia CPU backend (WASM); WebGPU as opt-in acceleration |
| Binary size in browser | Split: core WASM (~5–8 MB gzip) + ffmpeg.wasm loaded lazily on export |

**Deployment targets:**
- Hosted web app (`app.mercury-motion.dev`) — same as Remotion Studio experience
- Self-hostable static site (single `dist/` folder, no server required)
- Embeddable widget (iframe or Web Component) for docs/template previews

### Build & Distribution

| Concern | Tool |
|---------|------|
| Rust build (native) | Cargo + cross-compilation |
| Rust build (WASM) | `wasm-pack` + `wasm-bindgen` |
| Frontend | Vite |
| Packaging (desktop) | Tauri |
| Packaging (browser) | Vite static build |
| CI | GitHub Actions |
| Releases | GitHub Releases + hosted web app |

---

## 16. Error Handling Strategy

### Error Types (Library)

```rust
#[derive(thiserror::Error, Debug)]
pub enum MmotError {
    #[error("parse error at {pointer}: {message}")]
    Parse { message: String, pointer: String },

    #[error("prop '{prop}' expected {expected}, got {got}")]
    PropTypeMismatch { prop: String, expected: String, got: String },

    #[error("missing required prop: '{prop}'")]
    MissingRequiredProp { prop: String },

    #[error("asset not found: {path}")]
    AssetNotFound { path: std::path::PathBuf },

    #[error("asset decode failed: {path}: {reason}")]
    AssetDecode { path: std::path::PathBuf, reason: String },

    #[error("render failed at frame {frame}: {reason}")]
    RenderFailed { frame: u64, reason: String },

    #[error("encoder error: {0}")]
    Encoder(String),
}
```

### CLI Error Format

```
error[parse]: invalid easing value
  --> intro.mmot.json
  at: /compositions/main/layers/1/transform/opacity/0/easing
  |
  | "easing": "eaze_in"
  |            ^^^^^^^^ unknown easing; expected: linear, ease_in, ease_out,
  |                     ease_in_out, or {"type":"cubic_bezier",...}
```

```
error[asset-not-found]: ./assets/logo.png
  referenced by layer "logo" in composition "main"
  resolved path: D:\projects\intro\assets\logo.png
  file does not exist
```

### Editor Error Surfaces

| Error | Presentation |
|-------|-------------|
| JSON schema | Inline underline in Monaco + error panel |
| Asset not found | Orange warning on timeline layer; path in properties |
| Render failure | Toast with failing frame number |
| Export failure | Export dialog stays open with message |

---

## 17. Testing Strategy

Deterministic pixel output is Mercury-Motion's core value. Testing enforces it.

### Unit Tests

- **Keyframe evaluator**: Table-driven tests for all easing types. Boundaries: before-first, after-last, on-keyframe, between, single keyframe. Covers scalar, `[x,y]` vector, and color interpolation.
- **Parser**: Test corpus at `tests/fixtures/` — valid and invalid `.mmot.json` files. Each invalid file tests a specific error variant and verifies the JSON pointer path.
- **Props**: Type mismatch, missing required, unknown key.
- **Disambiguation**: `[960, 540]` → static vector; `[{"t":0,"v":960}]` → keyframe array.

### Golden Image Tests

The renderer's pixel output is verified against committed reference images:

- Reference files: `tests/golden/<name>.mmot.json`
- Reference renders: `tests/golden/<name>/frame-<N>.png` (committed; regenerated with `cargo test -- --update-goldens`)
- Test renders with CPU backend; pixel-exact comparison.
- Any pixel difference = test failure.
- Runs on every CI push across Windows, macOS, Linux (CPU output must be identical on all three).

### Integration Tests

- `mmot render`: renders known file; verifies output is decodable with correct frame count.
- `mmot validate`: valid and invalid files → correct exit codes.
- `mmot pack`: all `src` paths become `data:...` URIs.
- Props: template rendered with `--props`; text content verified via pixel region comparison.

### CI Matrix

GitHub Actions: Windows (x86-64), macOS (arm64), Ubuntu 22.04 (x86-64).
GPU backend: smoke tests only (renders without crash, output is not blank). Excluded from pixel-exact tests.

---

## 18. Roadmap

### Phase 1 — Core Engine (Weeks 1–6)
- [ ] Rust struct definitions + `serde` deserialization
- [ ] JSON Schema via `schemars`; validator with JSON pointer error paths
- [ ] Keyframe evaluator: all easing types, scalar/vector/color
- [ ] Golden image test infrastructure
- [ ] Skia CPU renderer: solid, image, text, video layers
- [ ] Default encoder: rav1e + pure-Rust MP4 mux
- [ ] CLI: `mmot render` and `mmot validate`
- [ ] Unit test suite passing on all 3 platforms in CI

### Phase 2 — Full Renderer (Weeks 7–10)
- [ ] Audio mixing and muxing
- [ ] Nested compositions (precomps)
- [ ] Lottie layer (Skia Skottie)
- [ ] Shape layer (rect, ellipse, Bézier path)
- [ ] Props/variables system with type validation
- [ ] WebM + GIF output
- [ ] Skia GPU backend (single-context pipelined model)
- [ ] `mmot pack` command

### Phase 3 — Editor MVP (Weeks 11–18)
- [ ] Tauri 2.0 app scaffold
- [ ] Preview: Rust render → base64 PNG → Vue `<img>`
- [ ] Timeline panel (layer blocks, scrubbing, playback)
- [ ] Properties panel (all static props, all layer types)
- [ ] Asset browser
- [ ] Monaco JSON editor + schema autocomplete
- [ ] Export dialog

### Phase 4 — Editor Polish (Weeks 19–24)
- [ ] Animated properties: ◆ in timeline + keyframe sub-tracks
- [ ] Keyframe curve editor (easing drag handles)
- [ ] Undo/redo (JSON-diff patch stack)
- [ ] Drag-to-add from asset browser
- [ ] Full keyboard shortcuts
- [ ] Cross-platform installers (`.msi`, `.dmg`, `.AppImage`)

### Phase 5 — Browser App (Weeks 25–30)
- [ ] WASM build: `wasm-pack` compilation of core renderer
- [ ] Web Worker integration: renderer off main thread
- [ ] `wasm-bindgen-rayon` parallel frame rendering in browser
- [ ] Vue frontend adapted for browser deployment (same codebase as Tauri)
- [ ] `ffmpeg.wasm` integration for browser-side MP4/WebM export
- [ ] Hosted app at `app.mercury-motion.dev`
- [ ] Self-hostable static build (zero server required)
- [ ] WebGPU backend (opt-in, Chrome/Firefox flag)

### Phase 6 — v1.0 (Weeks 31–38)
- [ ] Plugin/effect system (Rust trait-based)
- [ ] Self-hostable render API server
- [ ] Documentation site with interactive examples
- [ ] Starter template library (10+ templates)
- [ ] Template marketplace

---

## 19. Resolved Decisions

Implementation must not conflict with this table. Conflicts require a spec update first.

| Decision | Resolution | Rationale |
|----------|-----------|-----------|
| App name | **Mercury-Motion** | Named for Mercury, Roman god of speed |
| CLI binary | `mmot` | Short, fast to type |
| File extension | `.mmot.json` | Short, unique, human-readable |
| Text rendering | Skia SkShaper (via `skia-safe`) | Unified text pipeline; no dual-engine risk |
| Lottie rendering | Skia Skottie (via `skia-safe`) | Same surface; no second renderer |
| `cosmic-text` | Excluded from v0.x | Dual text pipeline risk |
| `dotlottie-rs` | Excluded | Skottie makes it redundant |
| Encoder default | `rav1e` + pure-Rust MP4 mux | Zero C deps |
| Encoder extended | `ffmpeg-next` via `--features ffmpeg` | Opt-in; broader codecs |
| Binary size | ~30 MB (default), ~80 MB (GPU), ~120 MB (ffmpeg) | Static FFmpeg too large for default |
| GPU parallelism | CPU eval parallel (rayon) + single GPU context pipelined | No multi-context races |
| Layer visual order | First in `layers` = bottom of visual stack | After Effects convention |
| Easing format | `{"type":"cubic_bezier",...}` structured object | JSON Schema validatable |
| Static vs animated | Array-of-objects-with-`t` = animated; else static | Parse-time enforcement |
| Editor frontend | Vue 3 + TypeScript | Team familiarity; Composition API suits reactive editor state |
| Time unit | Frames only (integer) | Deterministic; no fps rounding |
| Schema versioning | Both `$schema` URL and `"version"` field | Tooling + runtime compat |
| Partial renders | Never written on failure | Clean failure model |
| Browser support | First-class WASM target (Phase 5) | Most Remotion users live in the browser; must beat Remotion there too |
| Browser encoding | `ffmpeg.wasm` (browser only) | `ffmpeg-next` not WASM-compatible; JS-side fallback only for browser export |
| Browser parallelism | `wasm-bindgen-rayon` + `SharedArrayBuffer` | Preserves rayon frame parallelism in WASM threads |

---

## 20. Open Questions

Must be resolved before the phase that requires them.

| # | Question | Options | Leaning | Blocks |
|---|----------|---------|---------|--------|
| 1 | Pure-Rust MP4 mux crate? | `minimp4` vs `mp4` crate vs roll-own | Evaluate `minimp4` (write-focused API) | Phase 1 |
| 2 | GPU preview in editor? | Base64 PNG vs shared GPU texture | Base64 PNG (Phase 3); shared texture (Phase 4+) | Phase 3 |
| 3 | GIF encoder? | `gif` crate vs ffmpeg | `gif` crate (zero deps) | Phase 2 |
| 4 | Font fallback? | Hard error vs warn + system sans-serif | Warn + fallback (templates must not hard-fail) | Phase 1 |
| 5 | Undo/redo? | JSON-diff patch stack vs command pattern | JSON-diff (automatic; covers all changes) | Phase 4 |

---

## Appendix A: Comparison with Remotion

| Aspect | Remotion | Mercury-Motion |
|--------|----------|----------------|
| Renderer | Headless Chromium | Skia (CPU or GPU) |
| Frame isolation | Browser time-override hacks | Pure function of frame N |
| Dependencies | Node.js, npm, Chrome | Single binary, zero runtime deps |
| Authoring format | React/TSX | JSON |
| Parallelism | Chrome process pool | `rayon` (scales to all cores) |
| GPU rendering | Chrome GPU (unreliable headless) | wgpu (Vulkan/Metal/DX12) |
| Render speed (1080p30, 10s) | ~8–35 min | ~3–12 sec |
| Non-determinism risk | High | Zero |
| Binary size | ~200 MB (Node + Chrome) | ~30–80 MB |
| License | Source-available | MIT |

---

## Appendix B: Why Not Extend Remotion?

Remotion's React model *is* the renderer. React's diffing, lifecycle hooks, `useEffect`, and third-party animation libraries all assume a stateful, time-advancing browser environment. You cannot make a React component deterministically frame-renderable without either:

1. Forbidding large parts of React (what Remotion tries to do — and fails at the edges), or
2. Building a custom React renderer backed by Skia (enormous effort; the result is still React with all its complexity).

The JSON format is not a downgrade from React. It is a more honest description of what a video *is*: a timeline of layers with animated properties. React is a UI framework pressed into service as a video description language. JSON is a data format built for exactly this.

---

---

## Appendix C: Mercury-Motion FilmTools

A separate product surface within Mercury-Motion aimed at cinematographers, colorists, and indie filmmakers. Everything here is **deterministic, offline, local, and mathematically explainable**. Zero generative AI. No hallucinated frames. No AI slop.

> *"The color science of a $100,000 ARRI camera. The reproducibility of code. The speed of a GPU. Free, forever."*

AI is used only as an **analytical assistant** — detecting, classifying, segmenting, measuring. Never creating. A colorist should be able to read every transform and understand exactly what it does.

---

### FilmTools Pillars

| Pillar | What it delivers |
|---|---|
| **Color Science** | ACES pipeline, OCIO, every major camera log format, LUT engine, CDL |
| **Film Emulation** | Grain, halation, bloom, film stock response curves — all algorithmic |
| **Smart Restoration** | Deflicker, stabilization, temporal NR, upscaling, frame interpolation |
| **VFX Assist** | SAM2 masking/roto, depth-based rack focus, chroma key, background removal |
| **Camera RAW** | CinemaDNG, OpenEXR, log decoding for ARRI/RED/Sony/Canon/Panasonic/Fuji |
| **Audio** | Dialogue denoising, transcription/subtitle sync, audio-video alignment |

---

### Core Foundation: OxiMedia

**GitHub:** [cool-japan/oximedia](https://github.com/cool-japan/oximedia)
**Released:** March 17, 2026
**Language:** Pure Rust — `#![forbid(unsafe_code)]` across all 92 crates (~1.36M lines)
**License:** Patent-free

A pure-Rust reconstruction of both FFmpeg and OpenCV built for film work. Features directly relevant to FilmTools:
- Codec encode/decode, container mux/demux, filter graphs, transcoding
- **Scene detection, shot boundary analysis, stabilization, denoising**
- **Color science, image I/O: DPX / OpenEXR / TIFF**
- Quality metrics: PSNR / SSIM / VMAF, audio processing, calibration

Codecs: royalty-free only — AV1, VP9, VP8, Opus, FLAC. Too new to fully trust in production today, but architecturally ideal as the primary native Rust substrate for FilmTools.

---

### Color Science Stack

#### Rust Color Math Foundation: `palette` crate
**GitHub:** [Ogeon/palette](https://github.com/Ogeon/palette) — MIT
**Crate:** [crates.io/crates/palette](https://crates.io/crates/palette)

The essential Rust foundation for all color math in FilmTools. Type-safe color space conversions: CIE Lab/LCh, sRGB, linear RGB, HSL/HSV, Oklab, Jzazbz, ACEScg, full gamut mapping. Use this for implementing ACES matrix operations, log curve math, and color space transforms natively in Rust without C FFI.

Also worth evaluating: `breda-color-grading` crate — explicitly designed for color grading operations in Rust.

#### OpenColorIO (OCIO)
**GitHub:** [AcademySoftwareFoundation/OpenColorIO](https://github.com/AcademySoftwareFoundation/OpenColorIO) — BSD 3-Clause, C++
**Rust path:** FFI via `opencolorio-sys` crate + `cxx` bridge (no production-ready bindings exist yet)

**OCIO 2.5 (VFX Reference Platform 2026, September 2025):**
- **Vulkan GPU support** added — GPU renderer now works on Vulkan alongside Metal, DirectX, OpenGL
- **Built-in ACES 2.0 configs** ship with the release
- New hue curve transform, config merging, improved interoperability

Industry backbone for color management — DaVinci Resolve, Nuke, Blender, Houdini all use it. OCIO v2 natively implements the full ACES pipeline, supports Academy/ASC CLF (Common LUT Format).

**OCIO gap:** No production-ready Rust bindings exist. Options: (1) wrap `opencolorio-sys` via `cxx` bridge, (2) implement common transforms natively using `palette` (feasible for matrix + log + LUT trilinear interpolation), (3) subprocess sidecar. For standard camera log + ACES, native Rust implementation is the cleanest path.

#### Supported Camera Log Formats (All Published Math, No Secrets)

| Camera | Log Format | Color Space |
|---|---|---|
| ARRI Alexa | LogC3 / LogC4 | ALEXA Wide Gamut |
| RED | Log3G10 | REDWideGamutRGB |
| Sony | S-Log2 / S-Log3 | S-Gamut3 / S-Gamut3.Cine |
| Canon | C-Log / C-Log2 / C-Log3 | Cinema Gamut |
| Panasonic | V-Log | V-Gamut |
| Fujifilm | F-Log / F-Log2 | F-Gamut |
| Blackmagic | Blackmagic Film Gen 5 | Blackmagic Wide Gamut |

#### ACES Pipeline

```
Camera RAW/Log  →  IDT (Input Device Transform)
               →  ACES (scene-linear)
               →  RRT (Reference Rendering Transform)
               →  ODT (Output Device Transform)
               →  Rec.709 / P3 / HDR10
```

#### LUT Support
`.cube` / `.3dl` 3D lookup tables — on GPU via `wgpu` this is a single 3D texture sample per pixel. Negligibly cheap at any resolution.

**LUTCalc:** [cameramanben/LUTCalc](https://github.com/cameramanben/LUTCalc) — open source, generates LUTs for ARRI/Sony/Canon/RED. Outputs `.cube`, `.3dl`, `.spi3d`.

#### ASC CDL (Color Decision List)
Open ASC standard: Slope, Offset, Power per RGB channel + Saturation. Travels with footage from on-set DIT through VFX to final grade as standardized XML. Supported natively by OCIO.

#### OpenEXR
**GitHub:** [AcademySoftwareFoundation/openexr](https://github.com/AcademySoftwareFoundation/openexr) — BSD 3-Clause
**Rust crate:** `openexr` (native, on crates.io)
16-bit float (half), 32-bit float, multi-channel, deep compositing. The professional HDR/VFX interchange format.

---

### Film Emulation (Zero AI — Pure Physics Math)

#### Reference Implementations

**agx-emulsion** — the most physically rigorous open source film simulation project
**GitHub:** [andreavolpato/agx-emulsion](https://github.com/andreavolpato/agx-emulsion)
Uses published Kodak and Fujifilm datasheets to reconstruct spectral sensitivities of specific film stocks. Full pipeline: virtual negative exposure → chemical dye density modeling → print/paper exposure → development. Includes grain simulation, DIR couplers (inter-layer chemistry), halation. This is the reference implementation for everything below.

**Newson et al. Film Grain Rendering** (ACM Transactions on Graphics 2023)
- CPU: [alasdairnewson/film_grain_rendering](https://github.com/alasdairnewson/film_grain_rendering)
- GPU/CUDA: [alasdairnewson/film_grain_rendering_gpu](https://github.com/alasdairnewson/film_grain_rendering_gpu)
Publication-grade stochastic grain model. Monte Carlo sampling of grain distributions. Configurable grain radius, filter sigma. Resolution-independent. **This is the algorithm to port to Rust + wgpu.**

**AgX** (Troy Sobotka) — now Blender 4.0's default view transform
**GitHub:** [MrLixm/AgXc](https://github.com/MrLixm/AgXc) — standalone port
Physically-grounded OCIO view transform. Handles chromatic attenuation in overexposed areas far better than Filmic. Handles unbounded open-domain linear values via log2 compression.

#### Effects Table

| Effect | Algorithm | Reference |
|---|---|---|
| **Film grain** | Stochastic Monte Carlo — Poisson-sampled grains per emulsion layer; log-normal size distribution; resolution-independent | Newson et al. 2023; agx-emulsion |
| **Halation** | Gaussian blur on highlight luma mask → red-channel weighted composite; models light penetrating emulsion layers and reflecting off anti-halation backing | agx-emulsion; utility-dctls; halation-dctl |
| **Film response curves** | Spectral density → characteristic curve → chemical dye transfer; or S-curve approximation from manufacturer densitometry data | agx-emulsion; agx-blender |
| **Lens bloom** | Airy disk convolution on highlights; large-radius Gaussian falloff | openfx-misc |
| **Chromatic aberration** | Per-channel radial distortion with different coefficients (Brown-Conrady model); R/G/B channels warped independently | openfx-misc LensDistortion |
| **Vignetting** | Radial cos⁴ falloff | — |
| **Gate weave** | Sub-pixel Gaussian noise on X/Y offset per frame | — |
| **Flicker** | Per-frame Poisson noise on exposure value | — |

**Halation physics note:** Halation occurs when bright light penetrates all emulsion layers, partially reflects off the anti-halation backing, and re-exposes the red layer (and sometimes green) — causing a reddish-orange glow around highlights. CineStill (remjet-removed film) shows strong halation because the remjet suppression layer is removed. This is why all halation algorithms weight the red channel most heavily.

**Free print film LUTs (reference/starting point):**
- Juan Melara: Fuji 3510, Kodak 2383, Kodak 2393 — [juanmelara.com.au](https://juanmelara.com.au/blog/print-film-emulation-luts-for-download)
- Cullen Kelly: Kodak 2383 + Fuji 3510 (ACES and DaVinci Wide Gamut compatible)

---

### Smart Restoration & Enhancement

#### Stabilization

**Gyroflow Core — PRIMARY CHOICE**
**GitHub:** [gyroflow/gyroflow](https://github.com/gyroflow/gyroflow)
**Language:** Core library is **pure Rust** — embeddable directly, no Qt/FFmpeg required
**License:** GPL v3 (check for embedding options)

The Gyroflow core handles: gyroscope-data-driven lens undistortion + stabilization, GPU processing via `wgpu` (Vulkan/Metal/DX12/OpenGL), optical flow, synchronization, keyframe smoothing, lens distortion correction. Supports GoPro GPMF, Sony, DJI, Insta360, RunCam telemetry via the companion `telemetry-parser` Rust crate.

This is the most sophisticated open-source stabilization library available and its core is **native Rust**. It is what the Gyroflow desktop app runs on. The `telemetry-parser` crate ([crates.io/crates/telemetry-parser](https://crates.io/crates/telemetry-parser)) parses motion metadata from camera files — enabling gyro-assisted stabilization that is fundamentally superior to optical-flow-only approaches.

**Fallback:** L1 Optimal Path stabilizer — Google's algorithm (L1-norm camera path minimization), implementable natively in Rust without external libraries.

#### Temporal Noise Reduction
- **VBM3D:** [tehret/vbm3d](https://github.com/tehret/vbm3d) — GPL, C++. Block-matching + 3D collaborative filtering. Gold standard for non-generative temporal NR. Rust path: C++ subprocess.

#### Frame Interpolation (Slow Motion / 24→48fps)
- **RIFE-ncnn-vulkan:** [nihui/rife-ncnn-vulkan](https://github.com/nihui/rife-ncnn-vulkan) — MIT, C++, Vulkan. Optical flow based — deterministic given two frames. No CUDA, no Python — runs on Intel/AMD/Nvidia via Vulkan. Rust path: subprocess or ncnn C API.

#### Upscaling
- **Real-ESRGAN-ncnn-vulkan:** [xinntao/Real-ESRGAN-ncnn-vulkan](https://github.com/xinntao/Real-ESRGAN-ncnn-vulkan) — BSD/MIT, C++, Vulkan. Learned upscaling — reconstructs detail, deterministic per input. Rust path: subprocess or ncnn C API.

#### Deflicker (Timelapse)
- [cyberang3l/timelapse-deflicker](https://github.com/cyberang3l/timelapse-deflicker) — Perl, luminance rolling average
- [struffel/simple-deflicker](https://github.com/struffel/simple-deflicker) — Python, histogram matching
Both are trivially re-implementable natively in Rust.

---

### VFX Assist (AI as Tool, Not Creator)

#### Object Masking & Rotoscoping — SAM2
**GitHub:** [facebookresearch/sam2](https://github.com/facebookresearch/sam2) — Apache 2.0
Outputs binary masks for user-selected regions. Tracks objects through video frames with streaming memory (SAM 2.1, December 2024). Fully offline, CPU supported. Direct open-source alternative to Adobe Roto Brush and DaVinci Magic Mask.
Rust path: subprocess or ONNX → `ort` crate.

Related:
- **Sammie-Roto:** [Zarxrax/Sammie-Roto](https://github.com/Zarxrax/Sammie-Roto) — SAM2-based roto GUI
- **EdgeTAM:** [facebookresearch/EdgeTAM](https://github.com/facebookresearch/EdgeTAM) — CVPR 2025, on-device tracking for edge/mobile

#### Depth-Based Rack Focus / Fake Shallow DOF

**Video Depth Anything (CVPR 2025):** [DepthAnything/Video-Depth-Anything](https://github.com/DepthAnything/Video-Depth-Anything) — Small model Apache 2.0
Temporally consistent depth maps for long video at real-time 30 FPS. Feeds a per-pixel Gaussian blur kernel scaled by `|depth - focus_distance|` — implemented as wgpu compute shader. Frame-to-frame depth consistency prevents flickering bokeh artifacts.

**ZoeDepth:** [isl-org/ZoeDepth](https://github.com/isl-org/ZoeDepth) — MiDaS backbone + metric depth binning. Produces **absolute metric depth** (not just relative). Better for realistic DOF simulation where the actual distance matters. PyTorch, ONNX-exportable.

**MiDaS:** [isl-org/MiDaS](https://github.com/isl-org/MiDaS) — MIT (cleanest license for commercial use). Relative depth only. ONNX → `ort` crate.

#### Scene Detection / Shot Boundary
- **PySceneDetect:** [Breakthrough/PySceneDetect](https://github.com/Breakthrough/PySceneDetect) — BSD 3-Clause. Subprocess, JSON output. Easy.
- **TransNetV2:** [soCzech/TransNetV2](https://github.com/soCzech/TransNetV2) — Neural classifier, far more accurate for complex transitions. ONNX → `ort`.

#### Color Matching Between Cameras
**color-matcher:** [hahnec/color-matcher](https://github.com/hahnec/color-matcher) — GPL v3, Python. Reinhard (LAB-space statistics), MKL, histogram matching. The Reinhard method is trivially reimplementable in native Rust without the GPL constraint.

---

### Audio

#### Dialogue Denoising
**RNNoise — Native Rust bindings exist:** [RustAudio/rnnoise-c](https://github.com/RustAudio/rnnoise-c) — Apache 2.0 / MIT
The easiest integration in the entire stack. Non-generative RNN filter for noise suppression (fans, crowd, vehicles). 16-bit mono PCM at 48kHz.

#### Transcription & Subtitle Sync
**whisper.cpp** — MIT, C API callable directly from Rust FFI. Fully offline. Generates SRT/VTT. Used for auto-subtitling and ADR script sync.

#### Multi-Camera Audio Sync
**auto-sound-sync:** [kerryland/auto-sound-sync](https://github.com/kerryland/auto-sound-sync) — waveform cross-correlation. PluralEyes-style double-system sound alignment.

---

### Camera RAW Decoding (Rust-Native)

This was a major gap — now largely solvable in pure Rust:

| Format | Crate / Tool | Notes |
|---|---|---|
| Canon CR2/CR3, Nikon NEF, Sony ARW, Fuji RAF | `rawler` ([dnglab/dnglab](https://github.com/dnglab/dnglab)) — LGPL-2.1 | Alpha state, actively developed |
| DNG (Cinema DNG + stills) | `dng-rs` ([apertus-open-source-cinema/dng-rs](https://github.com/apertus-open-source-cinema/dng-rs)) | Pure Rust, from AXIOM open cinema camera project |
| **BRAW (Blackmagic RAW)** | `gpu-video` ([AdrianEddy/gpu-video](https://github.com/AdrianEddy/gpu-video)) | GPU-accelerated; **decodes BRAW natively in Rust** |
| **RED RAW (R3D)** | `gpu-video` ([AdrianEddy/gpu-video](https://github.com/AdrianEddy/gpu-video)) | **Decodes R3D natively in Rust** — previously thought to require proprietary SDK |
| Camera motion telemetry | `telemetry-parser` ([crates.io](https://crates.io/crates/telemetry-parser)) — by Gyroflow author | Sony, GoPro GPMF, Insta360, DJI, Betaflight |

**The `gpu-video` crate is a major find** — AdrianEddy (Gyroflow author) built GPU-accelerated video decoding AND encoding in Rust, with support for BRAW and R3D. This potentially eliminates the need for proprietary SDKs for professional camera RAW formats.

### Integration Architecture

```
Mercury-Motion FilmTools
├── Native Rust (direct — zero FFI)
│   ├── OxiMedia            — scene detection, stabilization, NR, color, EXR I/O
│   ├── Gyroflow Core       — gyro-assisted stabilization (pure Rust + wgpu)
│   ├── gpu-video           — BRAW / R3D / GPU-accelerated decode+encode
│   ├── rawler / dng-rs     — camera RAW + DNG decoding
│   ├── telemetry-parser    — camera motion telemetry (Sony, GoPro, DJI, etc.)
│   ├── openexr crate       — HDR frame I/O
│   ├── rnnoise-c           — audio denoising (Apache 2.0 / MIT)
│   ├── palette             — color space math foundation
│   └── Film emulation      — grain (Newson alg.), halation, bloom (custom Rust)
│                             Reference: agx-emulsion, Newson et al. 2023
│
├── C FFI
│   ├── OpenColorIO 2.5     — ACES + camera log pipeline; Vulkan GPU (BSD-3)
│   └── whisper.cpp         — transcription (MIT)
│
├── ONNX Runtime (ort crate)
│   ├── ZoeDepth / Depth Anything V2 small  — metric depth-based DOF
│   ├── SAM2                                — object masking / roto (Apache 2.0)
│   └── TransNetV2                          — shot boundary detection
│
└── Subprocess (ncnn-vulkan C++ binaries, MIT)
    ├── RIFE-ncnn-vulkan        — frame interpolation (Vulkan, no CUDA)
    └── Real-ESRGAN-ncnn-vulkan — upscaling (Vulkan, no CUDA)
```

---

### FilmTools UI

Separate page within Mercury-Motion. Same dark, dense, professional design language as the editor. Not a toy — no magic buttons.

**Panels:**
- **Scopes** — Waveform, Parade, Vectorscope, Histogram (GPU-rendered, real-time)
- **Color Science** — ACES/OCIO pipeline, LUT rack, CDL panel, camera log input selector
- **Film Emulation** — Grain, halation, bloom with real-time GPU preview
- **Restoration** — Stabilize, denoise, deflicker, interpolate, upscale
- **VFX** — SAM2 masking, depth DOF, chroma key, color match
- **Audio** — Denoise, sync, transcribe → SRT

**What FilmTools is NOT:**
- No text-to-video, no AI-generated footage or grades
- No "one-click cinematic look" slop
- No cloud processing — fully local, offline, forever
- No subscription

---

## Appendix D: AI Development Skills & Tooling

### Claude Code Skills Stack

The following skills are installed for AI-assisted development on this project.

#### `actionbook/rust-skills` (install via plugin)

A Claude Code plugin providing a **meta-cognition framework** for Rust development. Instead of surface-level answers, it routes questions through three cognitive layers (WHY → WHAT → HOW) before responding.

**Skills directly relevant to Mercury-Motion:**

| Skill | Why it matters |
|---|---|
| `m07-concurrency` | `rayon` (frame parallelism) + `wgpu` (GPU async) + Tauri IPC — the highest-friction area in the codebase. Covers `Arc<RwLock<T>>`, channels, `Send`/`Sync` errors on GPU resources. |
| `unsafe-checker` | `skia-safe` and `ffmpeg-next` are unsafe FFI wrappers. Enforces `// SAFETY:` docs, reviews `transmute`, raw pointers, `extern "C"` correctness. |
| `m10-performance` | Frame pipeline profiling, allocation reduction, SIMD. Video work alternates bottlenecks between CPU (encoding), GPU (wgpu), and memory bandwidth (Skia surface writes). |
| `m06-error-handling` | Multi-crate error hierarchy (skia + wgpu + ffmpeg + serde). Guides `thiserror` (library) vs `anyhow` (CLI/app) split. |
| `m01-ownership` + `m02-resource` | GPU resources (`wgpu::Texture`, `wgpu::Buffer`) and Skia surfaces have strict ownership semantics — E0382/E0597 will be frequent. |
| `m04-zero-cost` | `Renderer` trait with CPU/GPU/WASM backends — guides monomorphization vs dynamic dispatch trade-offs on the hot render loop. |
| `m05-type-driven` | Newtype wrappers for typed pixel formats, color spaces, frame indices — prevents unit confusion in the render pipeline. |
| `m11-ecosystem` | Crate selection decisions: audio crates, wgpu shader compilation (`naga` vs `spirv-cross`), Tauri feature flags. |

**Key commands:**
- `/sync-crate-skills` — auto-generates AI skills for `skia-safe`, `wgpu`, `ffmpeg-next`, `cosmic-text`, `tauri` from their `docs.rs` pages. Run this after setting up `Cargo.toml`.
- `/rust-review` — structured code review with Rust best practices
- `/unsafe-check` — targeted review of all `unsafe` blocks
- `/refactor` — LSP-backed safe rename/extract/move

**Known gap:** No built-in `domain-graphics`, `domain-video`, or `domain-desktop` skill. Mitigation: use `rust-skill-creator` to author a custom `domain-video-renderer` skill covering GPU resource lifetimes, frame timing, codec threading models, and WASM constraints specific to this project.

#### Superpowers Skills (already installed)

| Skill | When to invoke |
|---|---|
| `superpowers:writing-plans` | Before any multi-file implementation |
| `superpowers:brainstorming` | Before designing any new feature or API |
| `superpowers:systematic-debugging` | Before proposing any fix |
| `superpowers:test-driven-development` | Before writing any implementation code |
| `superpowers:verification-before-completion` | Before marking any task done |
| `superpowers:subagent-driven-development` | For parallel independent tasks (e.g. renderer + editor work) |

#### Remotion Official Skills (reference only)
```bash
npx skills add remotion-dev/skills
```
Teaches Claude the full Remotion API — useful as reference context when designing Mercury-Motion features that replace Remotion equivalents. Not used for Mercury-Motion implementation directly.

---

*This document is a living spec. Decisions made during implementation must be recorded in Section 18. Version this file alongside the code.*
