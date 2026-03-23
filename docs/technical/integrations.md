# Mercury-Motion: Technical Integrations Specification

**Project:** Mercury-Motion (`mmot`)
**Document version:** 1.0
**Engine version target:** 1.0 – 1.x
**License:** MIT

---

## Table of Contents

1. [Lottie / dotLottie](#1-lottie--dotlottie)
2. [Resolver Plugin System](#2-resolver-plugin-system)
3. [Google Fonts](#3-google-fonts)
4. [Adobe After Effects Export Plugin](#4-adobe-after-effects-export-plugin)
5. [Figma Plugin](#5-figma-plugin)
6. [Unsplash Integration](#6-unsplash-integration)
7. [Pexels Integration](#7-pexels-integration)
8. [ElevenLabs AI Voiceover](#8-elevenlabs-ai-voiceover)
9. [Social Media Presets](#9-social-media-presets)
10. [REST Render Server](#10-rest-render-server)
11. [VSCode Extension](#11-vscode-extension)
12. [JavaScript / TypeScript SDK](#12-javascript--typescript-sdk)
13. [Python SDK](#13-python-sdk)

---

## 1. Lottie / dotLottie

**Roadmap phase:** v1.0 (shipped)

### Overview

Mercury-Motion natively renders Lottie animations as first-class layer types. Both the legacy `.json` Lottie format and the modern `.lottie` (dotLottie) container format are supported. Rendering is performed by the Skia Skottie module, which is exposed through the `skia-safe` Rust bindings. Because Skottie is a CPU/GPU-path rasterizer rather than a JavaScript runtime, playback is deterministic and frame-accurate — there is no animation runtime behavior that differs between platforms.

### JSON Layer Syntax

```json
{
  "id": "hero-animation",
  "type": "lottie",
  "src": "./animations/hero.lottie",
  "in": 0,
  "out": 90,
  "playback_speed": 1.0,
  "loop": false,
  "segment": [0, 60],
  "theme": {
    "fills": {
      "primary": "#FF3B30",
      "secondary": "#FFFFFF"
    }
  },
  "transform": {
    "x": 0,
    "y": 0,
    "width": 1920,
    "height": 1080,
    "opacity": 1.0
  }
}
```

**Field reference:**

| Field | Type | Default | Description |
|---|---|---|---|
| `src` | string | required | Path to `.lottie` or `.json` file, or `data:` URI |
| `in` | number | required | Timeline frame at which the layer starts |
| `out` | number | required | Timeline frame at which the layer ends |
| `playback_speed` | number | `1.0` | Multiplier applied to the Lottie animation's internal playback rate |
| `loop` | boolean | `false` | Whether the animation loops within its `in`/`out` window |
| `segment` | [number, number] | full animation | Internal frame range of the Lottie animation to play |
| `theme.fills` | object | none | Color overrides keyed by Lottie slot ID |

### Implementation Approach

The Lottie layer is handled inside the `mmot-core` crate in `src/layers/lottie.rs`.

**dotLottie unpacking.** When `src` ends with `.lottie`, the file is treated as a zip archive (the dotLottie spec). The archive is opened with the `zip` crate, and the first `animations/*.json` entry is extracted to a temporary buffer in memory. Embedded fonts referenced in the manifest are extracted and registered with Skia's `FontMgr` before Skottie animation initialization.

**Skottie initialization.**

```rust
use skia_safe::interop::RustStream;
use skia_safe::skottie::{Animation, InitFlags};

pub fn load_skottie(json_bytes: &[u8], resource_provider: &dyn ResourceProvider) -> Result<Animation> {
    Animation::from_reader(
        &mut RustStream::new(json_bytes),
        resource_provider,
        InitFlags::default(),
    )
    .ok_or(MmotError::LottieLoadFailed)
}
```

**Frame mapping.** At render time, for each output frame `F` within `[in, out)`:

```
lottie_t = ((F - layer.in) * playback_speed) / composition.fps
```

If `segment` is specified, `lottie_t` is further clamped to `[segment[0]/lottie_fps, segment[1]/lottie_fps]`. If `loop` is `true`, the modulo of the segment duration is applied. The resulting `lottie_t` is passed to `Animation::seek_frame_time` and then `Animation::render` draws to a Skia canvas.

**Resource provider.** A custom `ResourceProvider` implementation resolves relative asset paths inside Lottie files (e.g., embedded images) relative to the `.lottie` file's directory. Assets embedded as base64 in the Lottie JSON are decoded in-memory.

**Theme overrides.** Color slots are applied via Skottie's `SlotManager`. Each key in `theme.fills` is matched against named color slots in the Lottie file and overridden with the specified hex color before the first frame is rendered.

### Supported Features

- `.json` (Lottie 5.x) and `.lottie` (dotLottie 1.x) file formats
- Playback speed control
- Segment-based playback (play a sub-range of the animation)
- Loop within timeline window
- Color theme overrides via named Lottie slots
- Embedded image assets (PNG, JPEG, WebP)
- Embedded font assets in dotLottie containers
- `src` as `data:application/json;base64,...` inline

### Limitations

- **No expression support.** Lottie expressions (JavaScript) are not evaluated. Layers relying on expressions will render to their unanimated base value. This is a fundamental constraint of the Skottie renderer.
- **No AE effects.** Lottie effects (drop shadow, Gaussian blur via AE effects) are not supported by Skottie.
- **3D layers ignored.** Lottie files exported with 3D layer positioning will be flattened to 2D.
- **Audio tracks in dotLottie** are not extracted or played back. If an audio layer is required alongside a Lottie animation, use a separate `audio` layer in the Mercury-Motion composition.
- **Text animators** with complex path-following or per-character 3D transforms may not render accurately.
- **Maximum animation size:** Performance degrades for Lottie files with more than ~1,000 shapes per frame at 4K resolution on a single CPU thread. Use `rayon` parallelism at the composition level, not within a single Skottie render call.
- **Interactive states** (dotLottie interactivity manifest) are ignored — Mercury-Motion always drives playback programmatically.

---

## 2. Resolver Plugin System

**Roadmap phase:** v1.0 (core architecture, ships with engine)

### Overview

The Resolver Plugin System intercepts non-filesystem `src` field values in any layer that accepts a `src` (image, video, audio, lottie, font). When the engine encounters a `src` string that is not a relative/absolute filesystem path and not a `data:` URI, it dispatches the value to a chain of registered `AssetResolver` implementations. The resolver that claims the URI scheme fetches or generates the asset and returns a `ResolvedAsset` containing either an in-memory byte buffer or a local cache path. The rest of the engine then proceeds as if a filesystem path had been provided.

This system is the foundation for all external service integrations (Google Fonts, Unsplash, Pexels, ElevenLabs, etc.).

### Rust Trait Definition

```rust
// crate: mmot-core, module: src/resolver/mod.rs

use async_trait::async_trait;
use std::path::PathBuf;

/// The asset returned by a resolver. The engine prefers `cache_path` when
/// available (zero-copy path), falling back to `data` for in-memory assets.
pub struct ResolvedAsset {
    /// MIME type of the resolved content (e.g. "image/png", "video/mp4").
    pub mime: String,
    /// If the resolver wrote to disk, the absolute path to the cached file.
    pub cache_path: Option<PathBuf>,
    /// In-memory bytes if `cache_path` is None.
    pub data: Option<Vec<u8>>,
}

#[derive(Debug, thiserror::Error)]
pub enum ResolverError {
    #[error("URI scheme not supported by this resolver: {0}")]
    UnsupportedScheme(String),
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("API error {status}: {body}")]
    Api { status: u16, body: String },
    #[error("Cache error: {0}")]
    Cache(String),
    #[error("Configuration missing: {0}")]
    Config(String),
}

#[async_trait]
pub trait AssetResolver: Send + Sync {
    /// Returns the URI scheme this resolver handles, e.g. "gfont".
    fn scheme(&self) -> &'static str;

    /// Resolve a URI to a local asset. Implementations must be idempotent
    /// and cache-aware. The `no_cache` flag bypasses reading from cache but
    /// implementations MAY still write to cache after fetching.
    async fn resolve(&self, uri: &str, no_cache: bool) -> Result<ResolvedAsset, ResolverError>;

    /// Optional: called once at engine startup to validate config/credentials.
    async fn health_check(&self) -> Result<(), ResolverError> {
        Ok(())
    }
}
```

### Resolver Registry

```rust
// src/resolver/registry.rs

pub struct ResolverRegistry {
    resolvers: HashMap<String, Arc<dyn AssetResolver>>,
}

impl ResolverRegistry {
    pub fn new() -> Self {
        Self { resolvers: HashMap::new() }
    }

    pub fn register(&mut self, resolver: Arc<dyn AssetResolver>) {
        self.resolvers.insert(resolver.scheme().to_string(), resolver);
    }

    /// Dispatches a `src` value. Returns `None` if the URI is a plain path
    /// or `data:` URI (caller handles those directly).
    pub async fn resolve(
        &self,
        src: &str,
        no_cache: bool,
    ) -> Option<Result<ResolvedAsset, ResolverError>> {
        let scheme = src.split("://").next()?;
        // Plain paths and data URIs are not dispatched.
        if scheme == "data" || src.starts_with('/') || src.starts_with('.') {
            return None;
        }
        let resolver = self.resolvers.get(scheme)?;
        Some(resolver.resolve(src, no_cache).await)
    }
}
```

**Default registered resolvers (built-in):**

| Scheme | Resolver struct | Ships in |
|---|---|---|
| `gfont` | `GoogleFontsResolver` | v1.0 |
| `unsplash` | `UnsplashResolver` | v1.0 |
| `pexels` | `PexelsResolver` | v1.0 |
| `elevenlabs` | `ElevenLabsResolver` | v1.1 |

### Configuration File

Resolver API keys and options are stored in `~/.mmot/config.toml` (XDG-aware; falls back to `$MMOT_CONFIG_DIR`). Environment variables override config file values.

```toml
# ~/.mmot/config.toml

[resolvers.google_fonts]
# No API key required for public Google Fonts API v2.
cache_ttl_days = 365

[resolvers.unsplash]
access_key = "YOUR_UNSPLASH_ACCESS_KEY"
cache_ttl_days = 30
default_resolution = "regular"   # thumb | small | regular | full | raw

[resolvers.pexels]
api_key = "YOUR_PEXELS_API_KEY"
cache_ttl_days = 30
default_photo_size = "original"
default_video_quality = "hd"     # sd | hd | uhd

[resolvers.elevenlabs]
api_key = "YOUR_ELEVENLABS_API_KEY"
cache_ttl_days = 0               # 0 = indefinite (content-addressed)

[cache]
dir = "~/.mmot/cache"            # override default cache root
max_size_gb = 10                 # evict LRU entries when exceeded
```

**Environment variable overrides** (all prefixed `MMOT_`):

```
MMOT_UNSPLASH_ACCESS_KEY
MMOT_PEXELS_API_KEY
MMOT_ELEVENLABS_API_KEY
MMOT_CACHE_DIR
MMOT_CACHE_MAX_SIZE_GB
```

### Caching Strategy

All resolvers write fetched assets into `~/.mmot/cache/<scheme>/<content-hash>.<ext>`. The cache entry is accompanied by a sidecar JSON file `<content-hash>.meta.json`:

```json
{
  "uri": "unsplash://photo-abc123?resolution=regular",
  "mime": "image/jpeg",
  "fetched_at": "2025-10-01T12:00:00Z",
  "ttl_days": 30,
  "size_bytes": 4823021
}
```

At resolve time, the engine:

1. Computes a deterministic cache key from the full URI string (SHA-256 hex).
2. Checks for a matching `.meta.json` in the cache directory.
3. If found and not expired (based on `fetched_at + ttl_days`), returns the cached file path without a network call.
4. If expired or missing, fetches from the network, writes to cache, updates the sidecar, and returns.

**`--no-cache` flag:** When passed on the CLI (`mmot render --no-cache`), the resolver registry sets `no_cache = true` on all `resolve()` calls. Resolvers skip the cache-read step but still write the freshly-fetched asset to cache (so subsequent renders without `--no-cache` benefit).

**Cache eviction:** `mmot cache clean` prunes entries where TTL has expired. `mmot cache clean --all` removes all cached assets. When `max_size_gb` is exceeded, the oldest-accessed entries are evicted first.

### Adding a Custom Resolver

Third-party resolvers are loaded as shared libraries (`.so`/`.dll`/`.dylib`) placed in `~/.mmot/plugins/`. The plugin must export a C-ABI symbol:

```rust
// In the plugin crate:
#[no_mangle]
pub extern "C" fn mmot_resolver_create() -> *mut dyn AssetResolver {
    Box::into_raw(Box::new(MyCustomResolver::new()))
}
```

Plugin loading uses `libloading`. Plugins are registered automatically at engine startup by scanning `~/.mmot/plugins/*.mmot_plugin`.

### URI Dispatch in the Engine

```rust
// src/render/asset_loader.rs (simplified)

pub async fn load_asset(src: &str, registry: &ResolverRegistry, no_cache: bool) -> Result<AssetData> {
    if src.starts_with("data:") {
        return AssetData::from_data_uri(src);
    }
    if let Some(result) = registry.resolve(src, no_cache).await {
        let resolved = result?;
        return AssetData::from_resolved(resolved);
    }
    // Plain filesystem path
    AssetData::from_path(src)
}
```

---

## 3. Google Fonts

**Roadmap phase:** v1.0

### Overview

Mercury-Motion can reference any font from the Google Fonts catalog directly in `.mmot.json` without manually downloading font files. The `gfont://` URI scheme is intercepted by the `GoogleFontsResolver`, which downloads the appropriate TTF/variable font file via the Google Fonts Developer API v2, caches it locally, and passes the path to Skia's font registration pipeline. No API key is required for basic usage.

### JSON Syntax

**In a text layer `font` object:**

```json
{
  "id": "headline",
  "type": "text",
  "in": 0,
  "out": 150,
  "text": "${title}",
  "font": {
    "src": "gfont://Inter:wght@400;700",
    "size": 72,
    "weight": 700,
    "style": "normal"
  }
}
```

**URI format:**

```
gfont://<FamilyName>:<axis>@<value>[;<value>...]
```

Examples:

```
gfont://Inter
gfont://Inter:wght@400
gfont://Inter:wght@400;700
gfont://Roboto+Mono:ital,wght@0,400;1,700
gfont://Playfair+Display:ital@1
```

The URI format mirrors the Google Fonts CSS v2 API parameter syntax so that existing `fonts.googleapis.com` URLs can be adapted by replacing the scheme.

### CLI Font Management

```bash
# Download a font to the project's ./fonts/ directory (for offline/portable use)
mmot fonts add "Inter"
mmot fonts add "Inter:wght@400;700"

# List fonts available in project fonts/ directory
mmot fonts list

# Remove a cached font from ~/.mmot/cache/gfont/
mmot fonts remove "Inter"

# Search Google Fonts catalog (requires network)
mmot fonts search "sans-serif geometric"
```

`mmot fonts add` downloads the font file(s) and rewrites the relevant `gfont://` src values in the project's `.mmot.json` to relative `./fonts/Inter.ttf` paths, making the project fully portable and offline.

### Implementation: GoogleFontsResolver

```rust
// src/resolver/google_fonts.rs

const GFONTS_API_BASE: &str = "https://www.googleapis.com/webfonts/v1/webfonts";
const GFONTS_CSS_BASE: &str = "https://fonts.googleapis.com/css2";

pub struct GoogleFontsResolver {
    http: reqwest::Client,
    cache_dir: PathBuf,
    ttl_days: u64,
}

#[async_trait]
impl AssetResolver for GoogleFontsResolver {
    fn scheme(&self) -> &'static str { "gfont" }

    async fn resolve(&self, uri: &str, no_cache: bool) -> Result<ResolvedAsset, ResolverError> {
        // Parse: "gfont://Inter:wght@400;700"  ->  family="Inter", axes="wght@400;700"
        let (family, spec) = parse_gfont_uri(uri)?;

        let cache_key = sha256_hex(uri);
        let cache_path = self.cache_dir.join(format!("{cache_key}.ttf"));

        if !no_cache && cache_path.exists() && !self.is_expired(&cache_path)? {
            return Ok(ResolvedAsset {
                mime: "font/ttf".into(),
                cache_path: Some(cache_path),
                data: None,
            });
        }

        // Build CSS2 API URL and fetch to get the actual font file URL
        let css_url = build_css2_url(&family, &spec);
        let css = self.http.get(&css_url)
            .header("User-Agent", "Mozilla/5.0") // required to receive TTF URLs
            .send().await?.text().await?;

        let font_url = extract_font_url_from_css(&css)
            .ok_or_else(|| ResolverError::Api { status: 200, body: "no font URL in CSS".into() })?;

        let font_bytes = self.http.get(&font_url).send().await?.bytes().await?;
        tokio::fs::write(&cache_path, &font_bytes).await
            .map_err(|e| ResolverError::Cache(e.to_string()))?;

        self.write_meta(uri, "font/ttf", font_bytes.len(), &cache_path).await?;

        Ok(ResolvedAsset {
            mime: "font/ttf".into(),
            cache_path: Some(cache_path),
            data: None,
        })
    }
}
```

**Variable font handling.** When the requested spec includes an axis range (e.g. `wght@100;900`), the CSS2 API returns a variable font URL. The resolver detects this by inspecting the `format('woff2')` hints and prefers the variable TTF when available. Skia's `SkFontMgr` registers the variable font once; individual weights within a text layer are handled via `SkFontArguments::VariationPosition`.

**Offline support.** After the first successful resolution, the font is cached indefinitely (`cache_ttl_days = 365` by default). Render invocations with no network access succeed as long as the font was previously resolved. To guarantee offline availability before shipping a project, run `mmot fonts add` for each `gfont://` reference.

### Editor Autocomplete

The Tauri desktop editor pre-fetches the Google Fonts catalog JSON (`https://www.googleapis.com/webfonts/v1/webfonts?key=...&sort=popularity`) at editor startup and stores it in the editor's local SQLite database. When a user types in a `font.src` field, the Monaco Editor language server extension queries this local database to provide inline autocomplete with font name and available styles.

### Configuration

```toml
[resolvers.google_fonts]
cache_ttl_days = 365
# Optional API key for higher quota on the Fonts catalog API.
# The font download itself does not require a key.
api_key = ""
```

### Limitations

- Only TTF/variable TTF fonts are downloaded. WOFF2 is not used (Skia does not support WOFF2 natively).
- Font subsetting is not performed; full font files are cached, which may be large for CJK fonts (5–20 MB per file).
- Google Fonts does not expose all axis values via the CSS2 API for some variable fonts; in these cases, the resolver downloads the closest static weight.
- Icon fonts (Material Symbols, Noto Emoji) are supported but require correct Unicode codepoints in the `text` field; emoji fallback rendering depends on the host system's emoji font.

---

## 4. Adobe After Effects Export Plugin

**Roadmap phase:** v1.1

### Overview

The After Effects Export Plugin allows motion designers to export an AE composition directly to a `.mmot.json` file. The plugin is built as a UXP script panel (replacing the legacy CEP/ExtendScript approach for AE 24+) but also ships a CEP fallback for AE 22–23. The export is a best-effort mapping: unsupported AE features are logged and skipped or flattened, and the designer is shown a compatibility report before saving.

### AE Feature Mapping

| After Effects concept | Mercury-Motion layer type | Notes |
|---|---|---|
| Solid layer | `solid` | Color mapped directly |
| Footage (image) | `image` | Source file path preserved |
| Footage (video) | `video` | Source file path preserved |
| Text layer | `text` | Font, size, color, position exported |
| Pre-composition | `composition` | Nested comp exported as sub-composition |
| Shape layer | `shape` | Basic shapes (rect, ellipse, path) exported |
| Null object | ignored | In/out timing used to inform child layer animation |
| Audio layer | `audio` | Source file path preserved |
| Adjustment layer | ignored | Warning emitted; affected layers pre-rendered if possible |
| Camera layer | ignored | Warning emitted |

**Keyframe export:** Position, scale, rotation, opacity keyframes on supported layer types are exported as Mercury-Motion keyframe arrays. Easing is converted from AE's cubic Bézier handles to Mercury-Motion's `cubic-bezier` easing format.

```json
{
  "id": "logo",
  "type": "image",
  "src": "./assets/logo.png",
  "in": 0,
  "out": 150,
  "transform": {
    "keyframes": {
      "opacity": [
        { "frame": 0, "value": 0, "easing": "linear" },
        { "frame": 30, "value": 1.0, "easing": "cubic-bezier(0.4, 0, 0.2, 1)" }
      ],
      "x": [
        { "frame": 0, "value": -200 },
        { "frame": 30, "value": 0 }
      ]
    }
  }
}
```

### Unsupported AE Features and Fallback Behavior

| Feature | Fallback behavior |
|---|---|
| AE expressions | Expression evaluated at frame 0 only; static value used |
| 3D layers (Z position, cameras) | Z position ignored; layer treated as 2D |
| Track mattes | Matte ignored; base layer exported without masking |
| Layer styles (drop shadow, glow) | Ignored; warning in compatibility report |
| Time remapping | Linear time map applied at export time |
| Parenting (parent/child chains) | Parent transform baked into child at export time |
| Blend modes (other than Normal) | Ignored; warning emitted |
| Shape layer path animations | Path at frame 0 exported as static shape |
| Effects (all AE effects) | Ignored; warning emitted |
| 3D renderer (Cinema 4D, Ray-traced) | Ignored |

### Plugin Architecture

**UXP panel entry point (`src/index.ts`):**

```typescript
import { app } from 'photoshop'; // AE UXP shares module namespace
import { exportComposition } from './exporter';

document.getElementById('btn-export').addEventListener('click', async () => {
  const comp = app.project.activeItem;
  if (!comp || comp.typeName !== 'CompItem') {
    alert('Select a composition first.');
    return;
  }
  const result = await exportComposition(comp, getExportOptions());
  await saveToFile(result.mmotJson, result.report);
});
```

**Export options dialog fields:**

- **Output path** — file picker for `.mmot.json` destination
- **Asset handling** — `copy-relative` (copy footage to `./assets/`, rewrite paths) | `absolute` (keep absolute paths) | `embed-base64` (inline small images < 1 MB as `data:` URIs)
- **Include audio** — toggle
- **Flatten pre-comps** — when enabled, nested compositions are inlined rather than exported as `composition` layer type
- **Export range** — Work Area | Entire Composition | Custom (in–out frames)
- **Frame rate override** — use AE comp fps or override
- **Compatibility report** — always shown pre-save; lists unsupported features encountered

**Compatibility report format (console + UI):**

```
Mercury-Motion Export Report
============================
Composition: "Main Title Sequence" (900 frames @ 30fps)

Exported layers: 14
Skipped layers: 2

Warnings:
  [layer "Glow FX"]   Effect "Glow" is not supported. Layer exported without effect.
  [layer "Camera 1"]  Camera layers are not supported and have been skipped.
  [layer "BG Loop"]   Expression on property "Rotation" evaluated at frame 0 only.

Unsupported features detected: 3
Output: /Users/alice/Projects/title.mmot.json
```

### Installation

The plugin ships as a `.ccx` archive installable via Adobe Creative Cloud desktop app or manually via `manifest.json` + UXP Developer Tools. A CEP fallback ships in the same `.ccx` for AE versions < 24.

```
plugin/
├── manifest.json          # UXP manifest (targetApps: ["AEFT"])
├── index.html
├── dist/
│   └── index.js           # Bundled with esbuild
└── cep/                   # CEP fallback
    ├── .debug
    ├── CSXS/manifest.xml
    └── jsx/exporter.jsx
```

### Limitations

- Expressions are evaluated at frame 0 only. Complex expression-driven animations will be wrong.
- AE effects are not mapped. Any layer whose visual output depends entirely on effects will appear as a plain solid or image.
- 3D space is collapsed to 2D; cameras and lights are ignored.
- AE text animators (range selectors, character offset animations) are not exported; only the base text and unanimated style are captured.
- Maximum tested composition length is 10 minutes at 60 fps. Very long compositions with thousands of keyframes may cause the UXP panel to become slow during export.

---

## 5. Figma Plugin

**Roadmap phase:** v1.1

### Overview

The Figma Plugin allows designers to export static frames or component-set animations from Figma to `.mmot.json`. The plugin runs inside Figma's plugin sandbox (TypeScript + Figma Plugin API) and communicates with a plugin UI (iframe) for export options. The output is a Mercury-Motion scene file suitable for further editing or direct rendering.

### Figma Element Mapping

| Figma element | Mercury-Motion layer type | Notes |
|---|---|---|
| Frame / Group | `solid` (background) + child layers | Background fill becomes solid; children recursively exported |
| Rectangle / Ellipse | `shape` | Fill color, border radius exported |
| Text node | `text` | Font family, size, weight, fill color, alignment |
| Image fill | `image` | Image exported as PNG and referenced |
| Component instance | `composition` | Component definition exported as sub-composition |
| Vector path | `shape` | SVG path data embedded in shape layer |
| Boolean group | `shape` | Flattened to single SVG path at export |

**Coordinate mapping.** Figma's origin is top-left, y increases downward — identical to Mercury-Motion's coordinate system. Absolute positions within the selected frame are preserved directly. If the frame is not 1920×1080, the plugin offers to scale all coordinates proportionally.

### Animation: Figma Smart Animate to Keyframes

When exporting a component set that uses Smart Animate transitions between variants, the plugin maps each variant's property differences to Mercury-Motion keyframe arrays.

**Supported Smart Animate properties:**

| Figma variant delta | Mercury-Motion keyframe property |
|---|---|
| X / Y position | `transform.x` / `transform.y` |
| Width / Height | `transform.width` / `transform.height` |
| Opacity | `transform.opacity` |
| Rotation | `transform.rotation` |
| Fill color | `color` (solid layers only) |

**Easing mapping:**

| Figma easing | Mercury-Motion easing |
|---|---|
| Ease In | `ease-in` |
| Ease Out | `ease-out` |
| Ease In and Out | `ease-in-out` |
| Linear | `linear` |
| Spring | `cubic-bezier(0.5, 0, 0.1, 1)` (approximated) |

Transition duration in Figma (milliseconds) is converted to frame count: `frames = round(duration_ms / 1000 * fps)`.

### Plugin Architecture

```
figma-plugin/
├── manifest.json          # Figma plugin manifest
├── src/
│   ├── main.ts            # Runs in Figma sandbox (no DOM)
│   ├── ui/
│   │   ├── App.vue        # Plugin iframe UI (Vue)
│   │   └── index.html
│   └── exporter/
│       ├── mapping.ts     # Element-to-layer mapping logic
│       ├── animation.ts   # Smart Animate extraction
│       └── assets.ts      # Image export helpers
└── dist/
    ├── main.js
    └── ui.html
```

**Main sandbox entry (`src/main.ts`):**

```typescript
figma.showUI(__html__, { width: 400, height: 560 });

figma.ui.onmessage = async (msg) => {
  if (msg.type === 'export') {
    const selection = figma.currentPage.selection;
    if (selection.length === 0) {
      figma.ui.postMessage({ type: 'error', message: 'Select a frame to export.' });
      return;
    }
    const exporter = new MmotExporter(msg.options);
    const result = await exporter.exportNode(selection[0]);
    figma.ui.postMessage({ type: 'result', data: result });
  }
};
```

**Image asset export.** For nodes with image fills, the plugin calls `node.exportAsync({ format: 'PNG', constraint: { type: 'SCALE', value: 2 } })` to get PNG bytes, then base64-encodes them as `data:image/png;base64,...` inline `src` values (or writes them as files if the user enables the "export assets folder" option).

### Plugin UI

The plugin UI panel presents:

- **Selection preview** — thumbnail of the currently selected frame/component
- **Export mode** — `Static Frame` | `Animated (Smart Animate)` | `All Variants`
- **Dimensions** — detected from frame; option to override or select a preset (1920×1080, 1080×1920, etc.)
- **FPS** — default 30; editable
- **Duration** — auto-calculated from animation duration or manually set
- **Asset handling** — `Inline (base64)` | `Export to folder`
- **Scale factor** — 1x / 2x for exported images
- **Copy JSON** / **Save file** — output options

### Export Options

```json
{
  "mode": "animated",
  "fps": 30,
  "duration_ms": 2000,
  "scale": 2,
  "asset_mode": "inline",
  "include_prototype_interactions": false
}
```

### Limitations

- **Auto Layout** containers are flattened to absolute positioning. The exported layer positions are the rendered absolute positions, not the flex layout.
- **Prototype interactions** (click triggers, overlay transitions) are not exported.
- **Figma effects** (blur, shadows, blending) are not mapped; they are visible in the Figma design but not in the Mercury-Motion output.
- **Variable fonts** in Figma may export with a specific weight value; variable axis export is best-effort.
- **Component overrides** on instances may not be fully reflected if the override affects deeply nested children.
- **Prototype flows** with multiple screens are not automatically stitched into a timeline; each frame must be exported individually.
- Maximum tested frame complexity: ~500 nodes per frame. Deeply nested frames (>10 levels) may produce large output files.

---

## 6. Unsplash Integration

**Roadmap phase:** v1.0

### Overview

The Unsplash integration enables `image` layers to reference Unsplash photos by ID or via a random-photo query without manually downloading images. The `UnsplashResolver` fetches the appropriate resolution image from the Unsplash API, caches it locally, and returns the file path to the engine.

**Attribution requirement:** Unsplash's API terms of service require displaying photographer attribution when images are used. Mercury-Motion writes attribution metadata to a sidecar `attributions.json` file in the project directory at render time. It is the developer's or end user's responsibility to display this attribution in the final video or accompanying materials.

### JSON Syntax

**By photo ID:**

```json
{
  "id": "background",
  "type": "image",
  "src": "unsplash://photo-1529651737248-dad5e287768e",
  "in": 0,
  "out": 300
}
```

**Random photo by query:**

```json
{
  "id": "background",
  "type": "image",
  "src": "unsplash://random?query=mountains&orientation=landscape",
  "in": 0,
  "out": 300
}
```

**URI parameters:**

| Parameter | Applies to | Description |
|---|---|---|
| `resolution` | both | `thumb` \| `small` \| `regular` \| `full` \| `raw` (default: `regular`) |
| `query` | random only | Search terms for random photo selection |
| `orientation` | random only | `landscape` \| `portrait` \| `squarish` |
| `w` / `h` | both | Custom pixel dimensions (appended to Unsplash image URL) |

Example with resolution override:

```json
"src": "unsplash://photo-1529651737248-dad5e287768e?resolution=full"
```

### Implementation: UnsplashResolver

```rust
// src/resolver/unsplash.rs

const UNSPLASH_API: &str = "https://api.unsplash.com";

pub struct UnsplashResolver {
    http: reqwest::Client,
    access_key: String,
    cache_dir: PathBuf,
    default_resolution: String,
    ttl_days: u64,
}

#[async_trait]
impl AssetResolver for UnsplashResolver {
    fn scheme(&self) -> &'static str { "unsplash" }

    async fn resolve(&self, uri: &str, no_cache: bool) -> Result<ResolvedAsset, ResolverError> {
        let parsed = parse_unsplash_uri(uri)?; // -> UnsplashRequest { kind, id_or_query, params }

        let cache_key = sha256_hex(uri);
        let cache_path = self.cache_dir.join(format!("{cache_key}.jpg"));

        if !no_cache && self.cache_valid(&cache_path)? {
            return Ok(ResolvedAsset { mime: "image/jpeg".into(), cache_path: Some(cache_path), data: None });
        }

        let photo_meta = match parsed.kind {
            UnsplashKind::ById => {
                self.fetch_photo_meta(&parsed.id).await?
            }
            UnsplashKind::Random => {
                self.fetch_random_photo(&parsed.query, &parsed.orientation).await?
            }
        };

        let resolution = parsed.params.get("resolution")
            .map(String::as_str)
            .unwrap_or(&self.default_resolution);
        let image_url = photo_meta.urls.get(resolution)
            .ok_or_else(|| ResolverError::Api { status: 200, body: format!("unknown resolution: {resolution}") })?;

        // Trigger download tracking event (Unsplash API requirement)
        let _ = self.http.get(&photo_meta.links.download_location)
            .header("Authorization", format!("Client-ID {}", self.access_key))
            .send().await;

        let image_bytes = self.http.get(image_url).send().await?.bytes().await?;
        tokio::fs::write(&cache_path, &image_bytes).await
            .map_err(|e| ResolverError::Cache(e.to_string()))?;

        // Write attribution sidecar
        self.write_attribution(&photo_meta).await?;
        self.write_meta(uri, "image/jpeg", image_bytes.len(), &cache_path).await?;

        Ok(ResolvedAsset { mime: "image/jpeg".into(), cache_path: Some(cache_path), data: None })
    }
}
```

**Attribution sidecar.** At the end of a render, the engine collects all Unsplash attributions gathered during the resolve phase and writes `<project-dir>/attributions.json`:

```json
{
  "generated_at": "2025-10-01T12:00:00Z",
  "attributions": [
    {
      "layer_id": "background",
      "photo_id": "photo-1529651737248-dad5e287768e",
      "photographer": "John Doe",
      "photographer_url": "https://unsplash.com/@johndoe",
      "photo_url": "https://unsplash.com/photos/photo-1529651737248-dad5e287768e",
      "utm_source": "mercury_motion",
      "attribution_text": "Photo by John Doe on Unsplash"
    }
  ]
}
```

### API Key Configuration

```toml
[resolvers.unsplash]
access_key = "YOUR_UNSPLASH_ACCESS_KEY"
```

Or via environment variable: `MMOT_UNSPLASH_ACCESS_KEY`.

Register an application at [unsplash.com/developers](https://unsplash.com/developers) to obtain an Access Key. The Demo plan allows 50 requests/hour; production applications must apply for Production access (5,000 requests/hour).

### Rate Limiting

The resolver tracks API call timestamps in a local rate-limit state file (`~/.mmot/cache/unsplash_rate.json`). When the Demo limit of 50 requests/hour is approached (≥ 45 calls in the last 60 minutes), the resolver:

1. Returns a cached version if available (ignoring TTL expiry).
2. If no cache is available, logs a warning and waits until the rate window resets.
3. Emits a structured log event: `WARN [unsplash] Rate limit threshold reached; waiting 42s`.

HTTP 429 responses from the Unsplash API cause an immediate exponential-backoff retry (1s, 2s, 4s, max 3 retries).

### Limitations

- `random` URIs are non-deterministic unless combined with `--no-cache=false`; successive renders may resolve to different photos if the cache TTL has expired. Use explicit photo IDs for reproducible renders.
- Images are cached as JPEG regardless of the original Unsplash format. For PNG-required use cases (transparency), download manually.
- Unsplash does not provide video assets; use the Pexels integration for video.
- Attribution display in the final video is the developer's responsibility; Mercury-Motion only generates the sidecar file.

---

## 7. Pexels Integration

**Roadmap phase:** v1.0

### Overview

The Pexels integration provides access to Pexels' library of free stock photos and videos via the `pexels://` URI scheme. Unlike Unsplash (photos only), Pexels supports both photo and video assets, making it the primary integration for royalty-free stock video clips. No attribution is legally required by Pexels, but it is recommended.

### JSON Syntax

**Photo by ID:**

```json
{
  "id": "hero-image",
  "type": "image",
  "src": "pexels://photo/2014422",
  "in": 0,
  "out": 90
}
```

**Video by ID:**

```json
{
  "id": "background-video",
  "type": "video",
  "src": "pexels://video/4434208",
  "in": 0,
  "out": 300
}
```

**Photo search:**

```json
{
  "id": "hero-image",
  "type": "image",
  "src": "pexels://search/photo?q=mountain+sunrise&orientation=landscape&size=large"
}
```

**Video search:**

```json
{
  "id": "background-video",
  "type": "video",
  "src": "pexels://search/video?q=ocean+waves&orientation=landscape"
}
```

**URI format:**

```
pexels://photo/<id>[?size=<size>]
pexels://video/<id>[?quality=<quality>]
pexels://search/photo?q=<query>[&orientation=<o>][&size=<s>][&page=<n>]
pexels://search/video?q=<query>[&orientation=<o>][&page=<n>]
```

**Photo size parameter:** `original` | `large2x` | `large` | `medium` | `small` | `portrait` | `landscape` | `tiny`

**Video quality parameter:** `uhd` | `hd` | `sd`

### Implementation: PexelsResolver

```rust
// src/resolver/pexels.rs

const PEXELS_API: &str = "https://api.pexels.com/v1";
const PEXELS_VIDEO_API: &str = "https://api.pexels.com/videos";

pub struct PexelsResolver {
    http: reqwest::Client,
    api_key: String,
    cache_dir: PathBuf,
    default_photo_size: String,
    default_video_quality: String,
    ttl_days: u64,
}

#[async_trait]
impl AssetResolver for PexelsResolver {
    fn scheme(&self) -> &'static str { "pexels" }

    async fn resolve(&self, uri: &str, no_cache: bool) -> Result<ResolvedAsset, ResolverError> {
        let request = parse_pexels_uri(uri)?;

        let cache_key = sha256_hex(uri);
        let ext = if request.is_video { "mp4" } else { "jpg" };
        let cache_path = self.cache_dir.join(format!("{cache_key}.{ext}"));

        if !no_cache && self.cache_valid(&cache_path)? {
            let mime = if request.is_video { "video/mp4" } else { "image/jpeg" };
            return Ok(ResolvedAsset { mime: mime.into(), cache_path: Some(cache_path), data: None });
        }

        let download_url = match request.kind {
            PexelsKind::PhotoById(id) => self.get_photo_url(id, &request.size).await?,
            PexelsKind::VideoById(id) => self.get_video_url(id, &request.quality).await?,
            PexelsKind::SearchPhoto(params) => self.search_photo_url(&params).await?,
            PexelsKind::SearchVideo(params) => self.search_video_url(&params).await?,
        };

        let bytes = self.http.get(&download_url)
            .header("Authorization", &self.api_key)
            .send().await?.bytes().await?;

        tokio::fs::write(&cache_path, &bytes).await
            .map_err(|e| ResolverError::Cache(e.to_string()))?;
        self.write_meta(uri, if request.is_video { "video/mp4" } else { "image/jpeg" }, bytes.len(), &cache_path).await?;

        Ok(ResolvedAsset {
            mime: if request.is_video { "video/mp4".into() } else { "image/jpeg".into() },
            cache_path: Some(cache_path),
            data: None,
        })
    }
}
```

**Video file selection.** For a given video ID, the Pexels API returns multiple video files at different resolutions. The resolver selects the best match for the requested quality by comparing the video file's `quality` field (`hd`, `sd`) and picking the highest resolution within that tier.

**Search result determinism.** Search URIs resolve to the first result on page 1. To get a specific result, use `&page=<n>`. Search results are cached by full URI (including query string). For reproducible renders, prefer explicit ID URIs after identifying the desired asset.

### API Key Configuration

```toml
[resolvers.pexels]
api_key = "YOUR_PEXELS_API_KEY"
```

Or: `MMOT_PEXELS_API_KEY`. Obtain a key at [pexels.com/api](https://www.pexels.com/api/).

### Rate Limiting

The Pexels free API allows 200 requests/hour and 20,000 requests/month. The resolver tracks call counts in `~/.mmot/cache/pexels_rate.json` and applies the same threshold-based warning and wait behavior as the Unsplash resolver.

### Limitations

- Video files are downloaded in full before rendering begins. Large UHD video files (multi-GB) will be fetched to disk; ensure adequate cache space.
- Search results may change as the Pexels library is updated; use explicit IDs for reproducibility.
- Pexels API does not support animated GIF or WebM; only MP4 video files are returned.
- The free API tier is rate-limited; burst-rendering large batches may hit limits.

---

## 8. ElevenLabs AI Voiceover

**Roadmap phase:** v1.1

### Overview

The ElevenLabs integration synthesizes AI voiceover audio from text and attaches it to `audio` layers in a Mercury-Motion composition. The `elevenlabs://` URI scheme is resolved during a **pre-render TTS pass** that runs before frame rendering begins, eliminating any latency impact on the frame-parallel rendering pipeline.

Text content can be templated using Mercury-Motion props (e.g., `${narration}`), enabling data-driven voiceover generation — each render invocation can produce a different voiceover by passing different prop values.

### JSON Syntax

```json
{
  "id": "narration",
  "type": "audio",
  "src": "elevenlabs://21m00Tcm4TlvDq8ikWAM?text=${narration}&model=eleven_turbo_v2&stability=0.5&similarity_boost=0.75",
  "in": 0,
  "out": 300,
  "volume": 0.9
}
```

**URI format:**

```
elevenlabs://<voice-id>?text=<text>&model=<model>[&stability=<float>][&similarity_boost=<float>][&style=<float>][&speaker_boost=<bool>]
```

**URI parameters:**

| Parameter | Type | Default | Description |
|---|---|---|---|
| `text` | string | required | Text to synthesize. Supports `${prop}` template interpolation. |
| `model` | string | `eleven_turbo_v2` | ElevenLabs model ID |
| `stability` | float 0–1 | `0.5` | Voice stability |
| `similarity_boost` | float 0–1 | `0.75` | Similarity boost |
| `style` | float 0–1 | `0.0` | Style exaggeration |
| `speaker_boost` | bool | `true` | Speaker boost |

**Available models:**

| Model ID | Use case |
|---|---|
| `eleven_turbo_v2` | Low latency, good quality |
| `eleven_multilingual_v2` | Multilingual support |
| `eleven_monolingual_v1` | Legacy English |

### Pre-Render TTS Pass

Because TTS generation has non-trivial latency (1–5 seconds per audio clip), all `elevenlabs://` URIs in a composition are resolved **before** the frame render loop begins:

```rust
// src/render/pipeline.rs

pub async fn pre_render_pass(scene: &Scene, registry: &ResolverRegistry, no_cache: bool) -> Result<()> {
    let tts_uris: Vec<_> = scene.collect_uris_by_scheme("elevenlabs");

    // Resolve all TTS URIs concurrently (bounded by API rate limits)
    let semaphore = Arc::new(tokio::sync::Semaphore::new(3)); // max 3 concurrent TTS calls
    let tasks: Vec<_> = tts_uris.into_iter().map(|uri| {
        let registry = registry.clone();
        let sem = semaphore.clone();
        tokio::spawn(async move {
            let _permit = sem.acquire().await.unwrap();
            registry.resolve(&uri, no_cache).await
        })
    }).collect();

    futures::future::try_join_all(tasks).await?;
    Ok(())
}
```

After the pre-render pass, all TTS URIs are in cache and subsequent `resolve()` calls during frame rendering return immediately from the filesystem.

### Implementation: ElevenLabsResolver

```rust
// src/resolver/elevenlabs.rs

const ELEVENLABS_API: &str = "https://api.elevenlabs.io/v1";

pub struct ElevenLabsResolver {
    http: reqwest::Client,
    api_key: String,
    cache_dir: PathBuf,
}

#[async_trait]
impl AssetResolver for ElevenLabsResolver {
    fn scheme(&self) -> &'static str { "elevenlabs" }

    async fn resolve(&self, uri: &str, no_cache: bool) -> Result<ResolvedAsset, ResolverError> {
        let req = parse_elevenlabs_uri(uri)?;

        // Cache key is content-addressed: hash of voice_id + text + model + settings
        let cache_input = format!("{}:{}:{}:{}:{}", req.voice_id, req.text, req.model, req.stability, req.similarity_boost);
        let cache_key = sha256_hex(&cache_input);
        let cache_path = self.cache_dir.join(format!("{cache_key}.mp3"));

        if !no_cache && cache_path.exists() {
            return Ok(ResolvedAsset { mime: "audio/mpeg".into(), cache_path: Some(cache_path), data: None });
        }

        let body = serde_json::json!({
            "text": req.text,
            "model_id": req.model,
            "voice_settings": {
                "stability": req.stability,
                "similarity_boost": req.similarity_boost,
                "style": req.style,
                "use_speaker_boost": req.speaker_boost
            }
        });

        let response = self.http
            .post(format!("{}/text-to-speech/{}", ELEVENLABS_API, req.voice_id))
            .header("xi-api-key", &self.api_key)
            .json(&body)
            .send().await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            return Err(ResolverError::Api { status, body });
        }

        let audio_bytes = response.bytes().await?;
        tokio::fs::write(&cache_path, &audio_bytes).await
            .map_err(|e| ResolverError::Cache(e.to_string()))?;

        Ok(ResolvedAsset { mime: "audio/mpeg".into(), cache_path: Some(cache_path), data: None })
    }
}
```

### Props Integration

Template interpolation in `src` URI values happens before URI dispatch. The engine resolves all `${prop}` placeholders using the render-time props object:

```rust
let resolved_src = interpolate_props(&layer.src, &render_props);
// "elevenlabs://voice-id?text=${narration}" -> "elevenlabs://voice-id?text=Welcome+to+the+show"
registry.resolve(&resolved_src, no_cache).await
```

This means different prop values produce different cache keys, and different voiceovers are cached independently. A composition rendered with 50 different narration texts accumulates 50 cached MP3 files; none are re-generated on subsequent renders with the same text.

### Voice Selection

To list available voices:

```bash
mmot elevenlabs voices list
```

This calls `GET /v1/voices` and prints a table of voice IDs and names. Voice IDs are stable; use them directly in the URI.

```bash
mmot elevenlabs voices preview <voice-id> --text "Hello, world"
```

### API Key Configuration

```toml
[resolvers.elevenlabs]
api_key = "YOUR_ELEVENLABS_API_KEY"
cache_ttl_days = 0   # 0 = indefinite (content-addressed; same input always produces same output)
```

Or: `MMOT_ELEVENLABS_API_KEY`.

### Caching Strategy

ElevenLabs audio is cached content-addressedly: the cache key is a hash of the voice ID, text, model, and all voice settings. The same inputs always produce semantically equivalent audio, so `cache_ttl_days = 0` (indefinite) is safe. The cache is only invalidated by `mmot cache clean --all` or manual deletion.

**Estimated cache sizes:** A 30-second voiceover at ElevenLabs' default bitrate is approximately 400–600 KB MP3.

### Limitations

- TTS audio duration is not predictable before generation; if the synthesized audio is longer than the layer's `in`/`out` window, it will be trimmed. If shorter, the layer will be silent after the audio ends.
- The turbo model produces audio faster but at slightly lower quality than the standard model.
- ElevenLabs API is rate-limited based on plan tier; the pre-render pass uses a concurrency semaphore (`max 3`) to avoid bursting the limit.
- Non-English text requires the `eleven_multilingual_v2` model; the turbo model is English-optimized.
- Character limits per API call depend on ElevenLabs plan; long narration texts should be split across multiple audio layers.

---

## 9. Social Media Presets

**Roadmap phase:** v1.0

### Overview

Mercury-Motion ships a set of built-in composition presets for common social media video formats. A preset is a template `.mmot.json` file with pre-configured `meta` fields (dimensions, fps, duration) and guide layers marking safe zones. Presets are accessed via the CLI and the desktop editor's new-composition dialog.

### CLI Usage

```bash
mmot new --preset youtube         # Creates my-project.mmot.json with YouTube settings
mmot new --preset tiktok --name "promo"
mmot new --preset instagram-reel
mmot new --preset instagram-square
mmot new --preset twitter
mmot new --preset linkedin
```

### Preset Specifications

#### YouTube (16:9 Landscape)

```json
{
  "$schema": "https://mercury-motion.dev/schema/v1.json",
  "version": "1.0",
  "meta": {
    "width": 1920,
    "height": 1080,
    "fps": 30,
    "duration": 900,
    "preset": "youtube",
    "safe_zone": { "top": 54, "right": 108, "bottom": 54, "left": 108 }
  }
}
```

| Property | Value |
|---|---|
| Dimensions | 1920 × 1080 px |
| Aspect ratio | 16:9 |
| FPS | 30 (60 supported) |
| Default duration | 900 frames (30s) |
| Recommended max duration | 43,200 frames (24 min) |
| Safe zone (title/action) | 5% from all edges |
| Output bitrate guidance | 8 Mbps (1080p), 35–45 Mbps (4K) |

#### TikTok (9:16 Portrait)

```json
{
  "meta": {
    "width": 1080,
    "height": 1920,
    "fps": 30,
    "duration": 450,
    "preset": "tiktok",
    "safe_zone": { "top": 192, "right": 54, "bottom": 384, "left": 54 }
  }
}
```

| Property | Value |
|---|---|
| Dimensions | 1080 × 1920 px |
| Aspect ratio | 9:16 |
| FPS | 30 |
| Default duration | 450 frames (15s) |
| Recommended max duration | 10 min (18,000 frames at 30fps) |
| Safe zone | Top 10% (UI overlay), bottom 20% (action bar, caption area) |

#### Instagram Reel (9:16 Portrait)

```json
{
  "meta": {
    "width": 1080,
    "height": 1920,
    "fps": 30,
    "duration": 450,
    "preset": "instagram-reel",
    "safe_zone": { "top": 192, "right": 54, "bottom": 480, "left": 54 }
  }
}
```

| Property | Value |
|---|---|
| Dimensions | 1080 × 1920 px |
| Aspect ratio | 9:16 |
| FPS | 30 |
| Default duration | 450 frames (15s) |
| Recommended max duration | 2,700 frames (90s) |
| Safe zone | Top 10%, bottom 25% (action bar taller than TikTok) |

#### Instagram Square

```json
{
  "meta": {
    "width": 1080,
    "height": 1080,
    "fps": 30,
    "duration": 1800,
    "preset": "instagram-square"
  }
}
```

| Property | Value |
|---|---|
| Dimensions | 1080 × 1080 px |
| Aspect ratio | 1:1 |
| FPS | 30 |
| Default duration | 1,800 frames (60s) |
| Recommended max duration | 3,600 frames (120s) |

#### Twitter / X

```json
{
  "meta": {
    "width": 1280,
    "height": 720,
    "fps": 30,
    "duration": 420,
    "preset": "twitter"
  }
}
```

| Property | Value |
|---|---|
| Dimensions | 1280 × 720 px |
| Aspect ratio | 16:9 |
| FPS | 30 |
| Default duration | 420 frames (14s) |
| Recommended max duration | 4,200 frames (140s) |
| Max file size | 512 MB |

#### LinkedIn

```json
{
  "meta": {
    "width": 1920,
    "height": 1080,
    "fps": 30,
    "duration": 900,
    "preset": "linkedin"
  }
}
```

| Property | Value |
|---|---|
| Dimensions | 1920 × 1080 px |
| Aspect ratio | 16:9 (also supports 1:1 and 9:16) |
| FPS | 30 |
| Default duration | 900 frames (30s) |
| Recommended max duration | 54,000 frames (30 min) |
| Max file size | 5 GB |

### Safe Zone Layers

Each preset template includes guide `shape` layers on a locked "guides" sub-composition representing the safe zone boundaries:

```json
{
  "id": "safe-zone-guide",
  "type": "shape",
  "locked": true,
  "guide_only": true,
  "in": 0,
  "out": 9999999,
  "shape": {
    "type": "rect",
    "x": 108, "y": 54,
    "width": 1704, "height": 972,
    "stroke": { "color": "#00FFFF", "width": 2, "dash": [10, 10] },
    "fill": "none"
  }
}
```

`guide_only: true` layers are rendered in the editor preview but excluded from the final render output.

### Editor: New Composition Dialog

The Tauri desktop editor presents a new-composition dialog at startup or via `File > New`. The dialog shows:

- A grid of preset thumbnails with name and dimensions labeled
- A "Custom" option with manual dimension/fps/duration inputs
- "Blank" option (no guides, no guide layers)

On selection, the preset template is copied into the project directory and opened in the editor.

### Preset Storage

Built-in presets are bundled as static JSON files in the `mmot-core` crate under `assets/presets/*.mmot.json`, included via `include_str!()` at compile time. Custom user presets can be stored in `~/.mmot/presets/` and are discovered by the CLI and editor automatically.

---

## 10. REST Render Server

**Roadmap phase:** v1.1

### Overview

`mmot server` starts an HTTP render server that accepts `.mmot.json` documents via a REST API and renders them asynchronously. The server is suitable for self-hosted render farms, CI/CD video generation pipelines, and SaaS platforms built on top of Mercury-Motion. It is built on `axum` and `tokio`, using an async job queue backed by an in-memory store (with optional SQLite persistence for durability across restarts).

### Starting the Server

```bash
mmot server --port 3000
mmot server --port 3000 --concurrency 4
mmot server --port 3000 --auth-token "secret-token"
mmot server --port 3000 --no-cache
```

**Flags:**

| Flag | Default | Description |
|---|---|---|
| `--port` | `3000` | TCP port to listen on |
| `--host` | `127.0.0.1` | Bind address |
| `--concurrency` | `CPU count / 2` | Max simultaneous render jobs |
| `--auth-token` | none | Bearer token for authentication (disables auth if omitted) |
| `--no-cache` | false | Bypass asset cache for all jobs |
| `--output-dir` | `/tmp/mmot-server` | Directory for completed video files |
| `--max-job-age` | `3600` | Seconds after which completed jobs and files are purged |

### API Endpoints

#### `POST /render`

Submit a render job.

**Request body:**

```json
{
  "scene": { ... },
  "props": {
    "title": "My Video",
    "narration": "Welcome to the show."
  },
  "format": {
    "codec": "av1",
    "crf": 30,
    "output_name": "my-video.mp4"
  }
}
```

**`scene`** — the full `.mmot.json` object (not a file path).
**`props`** — prop overrides merged with defaults defined in `scene.props`.
**`format`** — optional; defaults to AV1 at CRF 30.

| `format` field | Type | Default | Options |
|---|---|---|---|
| `codec` | string | `av1` | `av1`, `h264`, `h265` |
| `crf` | number | `30` | 0–63 |
| `output_name` | string | `output.mp4` | Filename for download |

**Response `202 Accepted`:**

```json
{
  "job_id": "j_a3f8c2d1e4b5",
  "status": "queued",
  "created_at": "2025-10-01T12:00:00Z",
  "estimated_duration_s": null
}
```

---

#### `GET /render/{id}`

Poll job status and progress.

**Response `200 OK`:**

```json
{
  "job_id": "j_a3f8c2d1e4b5",
  "status": "rendering",
  "progress": {
    "frames_total": 900,
    "frames_done": 432,
    "percent": 48.0,
    "elapsed_s": 12.4,
    "eta_s": 13.5
  },
  "created_at": "2025-10-01T12:00:00Z",
  "started_at": "2025-10-01T12:00:02Z",
  "completed_at": null,
  "error": null
}
```

**Status values:** `queued` | `pre_rendering` | `rendering` | `encoding` | `completed` | `failed` | `cancelled`

**Response `200 OK` (completed):**

```json
{
  "job_id": "j_a3f8c2d1e4b5",
  "status": "completed",
  "progress": { "frames_total": 900, "frames_done": 900, "percent": 100.0, "elapsed_s": 28.1, "eta_s": 0 },
  "created_at": "2025-10-01T12:00:00Z",
  "started_at": "2025-10-01T12:00:02Z",
  "completed_at": "2025-10-01T12:00:30Z",
  "download_url": "/render/j_a3f8c2d1e4b5/download",
  "file_size_bytes": 18304921,
  "error": null
}
```

---

#### `GET /render/{id}/download`

Download the completed video file. Returns `404` if the job is not completed or has expired.

**Response:** Binary MP4 stream with `Content-Type: video/mp4` and `Content-Disposition: attachment; filename="my-video.mp4"`.

---

#### `DELETE /render/{id}`

Cancel a queued or in-progress job, or delete a completed job and its output file.

**Response `200 OK`:**

```json
{ "job_id": "j_a3f8c2d1e4b5", "cancelled": true }
```

---

#### `GET /health`

Health check endpoint.

**Response `200 OK`:**

```json
{
  "status": "ok",
  "version": "1.1.0",
  "queue": {
    "queued": 2,
    "rendering": 1,
    "concurrency": 4
  },
  "uptime_s": 3621
}
```

### Authentication

When `--auth-token` is set, all endpoints except `GET /health` require:

```
Authorization: Bearer <token>
```

Requests without a valid token receive `401 Unauthorized`. Token validation is constant-time to prevent timing attacks.

### Job Queue Architecture

```rust
// src/server/queue.rs

pub struct RenderQueue {
    jobs: Arc<RwLock<HashMap<String, RenderJob>>>,
    sender: mpsc::Sender<String>,  // job IDs
    semaphore: Arc<Semaphore>,     // concurrency control
}

impl RenderQueue {
    pub async fn submit(&self, job: RenderJob) -> String {
        let id = generate_job_id();
        self.jobs.write().await.insert(id.clone(), job);
        self.sender.send(id.clone()).await.unwrap();
        id
    }
}
```

Workers are spawned at server startup. Each worker acquires a semaphore permit before beginning a render, ensuring at most `--concurrency` renders run simultaneously.

### OpenAPI Schema

The server serves its OpenAPI 3.1 schema at `GET /openapi.json` and a Swagger UI at `GET /docs`.

```yaml
openapi: "3.1.0"
info:
  title: Mercury-Motion Render Server
  version: "1.1.0"
paths:
  /render:
    post:
      summary: Submit a render job
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/RenderRequest'
      responses:
        '202':
          description: Job accepted
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/JobStatus'
        '400':
          description: Invalid scene document
        '401':
          description: Unauthorized
  /render/{id}:
    get:
      summary: Get job status
      parameters:
        - name: id
          in: path
          required: true
          schema: { type: string }
      responses:
        '200':
          description: Job status
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/JobStatus'
        '404':
          description: Job not found
    delete:
      summary: Cancel or delete a job
      responses:
        '200':
          description: Cancelled
        '404':
          description: Job not found
  /render/{id}/download:
    get:
      summary: Download completed video
      responses:
        '200':
          description: Video file
          content:
            video/mp4:
              schema:
                type: string
                format: binary
        '404':
          description: Not ready or not found
  /health:
    get:
      summary: Health check
      responses:
        '200':
          description: Server status
components:
  schemas:
    RenderRequest:
      type: object
      required: [scene]
      properties:
        scene:
          type: object
          description: Full .mmot.json scene document
        props:
          type: object
          additionalProperties: true
        format:
          $ref: '#/components/schemas/RenderFormat'
    RenderFormat:
      type: object
      properties:
        codec:
          type: string
          enum: [av1, h264, h265]
          default: av1
        crf:
          type: integer
          minimum: 0
          maximum: 63
          default: 30
        output_name:
          type: string
          default: output.mp4
    JobStatus:
      type: object
      properties:
        job_id: { type: string }
        status:
          type: string
          enum: [queued, pre_rendering, rendering, encoding, completed, failed, cancelled]
        progress:
          $ref: '#/components/schemas/RenderProgress'
        created_at: { type: string, format: date-time }
        started_at: { type: string, format: date-time, nullable: true }
        completed_at: { type: string, format: date-time, nullable: true }
        download_url: { type: string, nullable: true }
        file_size_bytes: { type: integer, nullable: true }
        error: { type: string, nullable: true }
    RenderProgress:
      type: object
      properties:
        frames_total: { type: integer }
        frames_done: { type: integer }
        percent: { type: number }
        elapsed_s: { type: number }
        eta_s: { type: number, nullable: true }
  securitySchemes:
    bearerAuth:
      type: http
      scheme: bearer
security:
  - bearerAuth: []
```

### Docker Deployment

```dockerfile
# Dockerfile
FROM rust:1.82-slim AS builder
WORKDIR /build
COPY . .
RUN apt-get update && apt-get install -y \
    libssl-dev pkg-config cmake clang libfontconfig1-dev \
    && rm -rf /var/lib/apt/lists/*
RUN cargo build --release --bin mmot

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    libssl3 ca-certificates libfontconfig1 \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /build/target/release/mmot /usr/local/bin/mmot
EXPOSE 3000
ENV MMOT_CACHE_DIR=/var/cache/mmot
VOLUME ["/var/cache/mmot"]
ENTRYPOINT ["mmot", "server", "--host", "0.0.0.0", "--port", "3000"]
```

```bash
docker build -t mercury-motion-server .
docker run -p 3000:3000 \
  -e MMOT_UNSPLASH_ACCESS_KEY=xxx \
  -e MMOT_PEXELS_API_KEY=yyy \
  -e MMOT_ELEVENLABS_API_KEY=zzz \
  -v mmot-cache:/var/cache/mmot \
  mercury-motion-server \
  --auth-token "my-secret" \
  --concurrency 4
```

### Limitations

- The server does not persist job state across restarts (jobs are in-memory). In-progress renders at restart time are lost.
- Large scene documents (>50 MB JSON) are rejected with `413 Payload Too Large`.
- The server does not implement job prioritization; jobs are processed FIFO.
- Completed video files are stored on the server's local filesystem; in a multi-instance deployment, a shared NFS or object storage mount is required for the output directory.

---

## 11. VSCode Extension

**Roadmap phase:** v1.0

### Overview

The `mercury-motion.mmot` VSCode extension provides first-class developer experience for working with `.mmot.json` files: syntax highlighting, JSON Schema validation, inline previews, integration with the `mmot` CLI, and layer-type snippets.

### Extension Metadata

```json
{
  "name": "mmot",
  "displayName": "Mercury-Motion",
  "publisher": "mercury-motion",
  "version": "1.0.0",
  "engines": { "vscode": "^1.85.0" },
  "categories": ["Languages", "Snippets", "Linters"],
  "activationEvents": ["onLanguage:mmot"],
  "contributes": {
    "languages": [
      {
        "id": "mmot",
        "aliases": ["Mercury-Motion Scene", "mmot"],
        "extensions": [".mmot.json"],
        "configuration": "./language-configuration.json"
      }
    ]
  }
}
```

### JSON Schema Validation

The extension registers the Mercury-Motion JSON Schema with VSCode's built-in JSON language server:

```json
{
  "contributes": {
    "jsonValidation": [
      {
        "fileMatch": "*.mmot.json",
        "url": "https://mercury-motion.dev/schema/v1.json"
      }
    ]
  }
}
```

For offline use, the schema is also bundled with the extension and used as a fallback when the remote schema is unavailable. The schema is generated from the Rust `schemars` output and re-bundled on each extension release.

**Schema features provided by validation:**

- Required field highlighting (missing `id`, `type`, `in`, `out` on layers)
- Enum validation on `type`, `easing`, `blend_mode` fields
- Type checking on numeric fields (`fps`, `width`, `height`, `crf`)
- Hover documentation for all schema properties (descriptions pulled from `#[schemars(description = "...")]` Rust annotations)

### Syntax Highlighting

`.mmot.json` files are highlighted as JSON with additional semantic token providers that color-code layer type values (`"solid"`, `"text"`, `"video"`, etc.) using Mercury-Motion's theme colors. Layer IDs referenced in `"src"` fields of `composition` layers are highlighted as cross-references.

### Inline Frame Preview on Hover

When the cursor hovers over a composition name string (a key in `compositions`), the extension:

1. Calls `mmot preview --composition <name> --frame 0 --format png --output -` via a child process.
2. Receives the PNG on stdout.
3. Displays it as a Markdown image in a hover widget.

```typescript
// src/hoverProvider.ts
vscode.languages.registerHoverProvider('mmot', {
  async provideHover(document, position) {
    const word = getCompositionNameAtPosition(document, position);
    if (!word) return;

    const png = await spawnMmot(['preview', '--composition', word, '--frame', '0', '--format', 'png', '--output', '-']);
    const b64 = png.toString('base64');
    const md = new vscode.MarkdownString(`![preview](data:image/png;base64,${b64})`);
    md.isTrusted = true;
    return new vscode.Hover(md);
  }
});
```

The preview is cached in the extension's global state for 5 seconds to avoid re-rendering on rapid cursor movement.

### Problems Panel Integration (`mmot validate`)

The extension runs `mmot validate <file>` on save (debounced 500ms) and parses its JSON output to populate VSCode's Problems panel:

```typescript
// mmot validate --format json output:
[
  { "severity": "error", "message": "Layer 'title' has out (290) before in (300)", "line": 14, "col": 5 },
  { "severity": "warning", "message": "Layer 'bg' src file not found: ./missing.png", "line": 8, "col": 12 }
]
```

```typescript
const diagnosticsCollection = vscode.languages.createDiagnosticCollection('mmot');

async function validateDocument(doc: vscode.TextDocument) {
  const output = await spawnMmot(['validate', doc.uri.fsPath, '--format', 'json']);
  const issues = JSON.parse(output);
  const diagnostics = issues.map(issue => new vscode.Diagnostic(
    new vscode.Range(issue.line - 1, issue.col - 1, issue.line - 1, 999),
    issue.message,
    issue.severity === 'error' ? vscode.DiagnosticSeverity.Error : vscode.DiagnosticSeverity.Warning
  ));
  diagnosticsCollection.set(doc.uri, diagnostics);
}
```

### Command Palette

| Command | ID | Action |
|---|---|---|
| Mercury-Motion: Render | `mmot.render` | Runs `mmot render` on the active `.mmot.json` file; output shown in integrated terminal |
| Mercury-Motion: Validate | `mmot.validate` | Runs `mmot validate` and populates Problems panel |
| Mercury-Motion: Preview Frame | `mmot.previewFrame` | Opens a side panel with a rendered preview of the current frame |
| Mercury-Motion: Open in Editor | `mmot.openEditor` | Opens the file in the Tauri desktop editor |
| Mercury-Motion: Add Google Font | `mmot.addFont` | Quick-pick font from Google Fonts catalog and inserts `gfont://` src |

### Snippets

```json
{
  "Solid Layer": {
    "prefix": "mmot-solid",
    "body": [
      "{",
      "  \"id\": \"$1\",",
      "  \"type\": \"solid\",",
      "  \"in\": $2,",
      "  \"out\": $3,",
      "  \"color\": \"$4\"",
      "}"
    ]
  },
  "Text Layer": {
    "prefix": "mmot-text",
    "body": [
      "{",
      "  \"id\": \"$1\",",
      "  \"type\": \"text\",",
      "  \"in\": $2,",
      "  \"out\": $3,",
      "  \"text\": \"$4\",",
      "  \"font\": { \"size\": $5 }",
      "}"
    ]
  },
  "Image Layer": {
    "prefix": "mmot-image",
    "body": [
      "{",
      "  \"id\": \"$1\",",
      "  \"type\": \"image\",",
      "  \"src\": \"$2\",",
      "  \"in\": $3,",
      "  \"out\": $4",
      "}"
    ]
  },
  "Video Layer": {
    "prefix": "mmot-video",
    "body": [
      "{",
      "  \"id\": \"$1\",",
      "  \"type\": \"video\",",
      "  \"src\": \"$2\",",
      "  \"in\": $3,",
      "  \"out\": $4",
      "}"
    ]
  },
  "Audio Layer": {
    "prefix": "mmot-audio",
    "body": [
      "{",
      "  \"id\": \"$1\",",
      "  \"type\": \"audio\",",
      "  \"src\": \"$2\",",
      "  \"in\": $3,",
      "  \"out\": $4,",
      "  \"volume\": 1.0",
      "}"
    ]
  },
  "Lottie Layer": {
    "prefix": "mmot-lottie",
    "body": [
      "{",
      "  \"id\": \"$1\",",
      "  \"type\": \"lottie\",",
      "  \"src\": \"$2\",",
      "  \"in\": $3,",
      "  \"out\": $4,",
      "  \"loop\": false",
      "}"
    ]
  }
}
```

### `mmot` Binary Discovery

The extension discovers the `mmot` binary in order:

1. `mmot.binaryPath` VSCode setting (user-configured absolute path)
2. `MMOT_BIN` environment variable
3. `mmot` on `$PATH`
4. Platform-specific default install locations: `~/.cargo/bin/mmot` (Linux/macOS), `%USERPROFILE%\.cargo\bin\mmot.exe` (Windows)

If `mmot` is not found, the extension shows an informational notification with a link to the installation docs and disables all CLI-dependent features (preview, render, validate commands) while keeping schema validation and snippets active.

### Limitations

- The inline preview hover requires the `mmot` CLI to be installed and accessible.
- Frame preview for complex compositions (many video/lottie layers) may be slow (1–5 seconds); the extension cancels preview requests after a 10-second timeout.
- The extension does not support multi-root workspaces where different roots use different `mmot` binary versions.

---

## 12. JavaScript / TypeScript SDK

**Roadmap phase:** v1.1

### Overview

`@mercury-motion/sdk` is an npm package that provides a fluent, type-safe TypeScript API for constructing `.mmot.json` scenes programmatically. It is designed for code-driven video generation in Node.js build pipelines, server-side rendering, and CI/CD workflows. In Node.js, rendering is performed by spawning the `mmot` CLI. In browser environments, a WASM build of the core engine is used.

### Installation

```bash
npm install @mercury-motion/sdk
# or
pnpm add @mercury-motion/sdk
```

### TypeScript API

```typescript
import {
  Scene,
  Composition,
  SolidLayer,
  TextLayer,
  ImageLayer,
  VideoLayer,
  AudioLayer,
  LottieLayer,
  ShapeLayer,
  keyframe,
  easing,
} from '@mercury-motion/sdk'

// Create a scene
const scene = new Scene({
  width: 1920,
  height: 1080,
  fps: 30,
  duration: 300,
})

// Define props
scene.prop('title', { type: 'string', default: 'Hello World' })
scene.prop('narration', { type: 'string', default: '' })

// Build a composition
const main = new Composition('main')

main.add(new SolidLayer('bg', {
  in: 0,
  out: 300,
  color: '#1a1a2e',
}))

main.add(new TextLayer('title', {
  in: 10,
  out: 290,
  text: '${title}',
  font: {
    src: 'gfont://Inter:wght@700',
    size: 72,
    weight: 700,
  },
  transform: {
    x: 960,
    y: 540,
    anchor: 'center',
    keyframes: {
      opacity: [
        keyframe(0, 0),
        keyframe(30, 1, easing.easeOut),
      ],
    },
  },
}))

main.add(new AudioLayer('vo', {
  src: 'elevenlabs://21m00Tcm4TlvDq8ikWAM?text=${narration}&model=eleven_turbo_v2',
  in: 0,
  out: 290,
}))

scene.addComposition(main)

// Serialize to .mmot.json
const json = scene.toJSON()
await fs.writeFile('output.mmot.json', JSON.stringify(json, null, 2))

// Or render directly (spawns mmot CLI)
await scene.render(main, {
  output: 'video.mp4',
  props: { title: 'My Video', narration: 'Welcome to Mercury-Motion.' },
  codec: 'av1',
  crf: 28,
})
```

### Class Reference

#### `Scene`

```typescript
class Scene {
  constructor(meta: SceneMeta)
  prop(name: string, def: PropDefinition): this
  addComposition(comp: Composition): this
  toJSON(): MmotDocument
  render(comp: Composition, options: RenderOptions): Promise<RenderResult>
  renderServer(serverUrl: string, options: ServerRenderOptions): Promise<RenderResult>
}

interface SceneMeta {
  width: number
  height: number
  fps: number
  duration: number
}

interface RenderOptions {
  output: string
  props?: Record<string, unknown>
  codec?: 'av1' | 'h264' | 'h265'
  crf?: number
  noCache?: boolean
  mmotBin?: string  // path to mmot binary; defaults to 'mmot' on PATH
}
```

#### `Composition`

```typescript
class Composition {
  constructor(id: string)
  add(layer: Layer): this
  layers: Layer[]
}
```

#### Layer classes

All layer classes accept their corresponding JSON layer schema as constructor arguments. Type definitions are auto-generated from the Rust `schemars` output via a build script in the SDK repository.

```typescript
class SolidLayer   extends Layer { constructor(id: string, props: SolidLayerProps) }
class TextLayer    extends Layer { constructor(id: string, props: TextLayerProps) }
class ImageLayer   extends Layer { constructor(id: string, props: ImageLayerProps) }
class VideoLayer   extends Layer { constructor(id: string, props: VideoLayerProps) }
class AudioLayer   extends Layer { constructor(id: string, props: AudioLayerProps) }
class LottieLayer  extends Layer { constructor(id: string, props: LottieLayerProps) }
class ShapeLayer   extends Layer { constructor(id: string, props: ShapeLayerProps) }
```

#### Keyframe helpers

```typescript
function keyframe(frame: number, value: number, easing?: EasingValue): Keyframe
const easing = {
  linear: 'linear',
  easeIn: 'ease-in',
  easeOut: 'ease-out',
  easeInOut: 'ease-in-out',
  cubicBezier: (x1: number, y1: number, x2: number, y2: number) =>
    `cubic-bezier(${x1}, ${y1}, ${x2}, ${y2})`,
}
```

### TypeScript Type Generation

Types are generated from the Rust `schemars`-derived JSON Schema:

```bash
# In SDK repo, after updating mmot-core:
cargo run --bin generate-schema > schema.json
npx json-schema-to-typescript schema.json --out src/types/generated.ts
```

This ensures the TypeScript SDK types are always in sync with the Rust engine's accepted schema.

### Node.js: CLI Spawn Mechanism

```typescript
// src/renderer/node.ts

import { spawn } from 'child_process'

export async function renderViaCliNode(
  sceneJson: string,
  options: RenderOptions
): Promise<void> {
  const bin = options.mmotBin ?? findMmotBinary()
  const args = [
    'render',
    '--stdin',          // read scene from stdin
    '--output', options.output,
    '--codec', options.codec ?? 'av1',
    '--crf', String(options.crf ?? 30),
    ...(options.noCache ? ['--no-cache'] : []),
    ...(options.props ? ['--props', JSON.stringify(options.props)] : []),
  ]

  return new Promise((resolve, reject) => {
    const proc = spawn(bin, args, { stdio: ['pipe', 'inherit', 'inherit'] })
    proc.stdin.write(sceneJson)
    proc.stdin.end()
    proc.on('close', code => code === 0 ? resolve() : reject(new Error(`mmot exited with code ${code}`)))
  })
}
```

### Browser / WASM

In browser environments, the SDK detects the absence of Node.js globals and switches to the WASM renderer:

```typescript
import init, { render_scene } from '@mercury-motion/wasm'

export async function renderViaWasm(sceneJson: string, options: WasmRenderOptions): Promise<Uint8Array> {
  await init()
  return render_scene(sceneJson, JSON.stringify(options))
}
```

The WASM build is a separate `mmot-wasm` crate compiled with `wasm-pack`. It excludes audio encoding (rav1e) and provides a `render_frames(scene_json) -> Vec<ImageData>` function returning individual frame pixel data, which the browser can display or encode via the WebCodecs API.

**WASM limitations:** No filesystem access; all `src` values must be `data:` URIs or pre-resolved absolute URLs. No ElevenLabs/Unsplash/Pexels resolver support (CORS and API key security constraints).

### Rendering via REST Server

```typescript
await scene.renderServer('http://localhost:3000', {
  props: { title: 'Hello' },
  authToken: 'my-secret',
  output: './video.mp4',     // downloaded locally after job completes
  pollIntervalMs: 1000,
  onProgress: (progress) => console.log(`${progress.percent.toFixed(1)}% complete`),
})
```

### Limitations

- The Node.js renderer requires `mmot` to be installed and on `PATH` (or `mmotBin` specified).
- The WASM renderer does not support audio layers or external asset resolvers.
- Large compositions with many video layers may cause memory pressure in the WASM environment due to browser memory limits.
- The SDK does not validate the scene document before passing it to `mmot`; invalid scenes will fail at render time with CLI error output.

---

## 13. Python SDK

**Roadmap phase:** v1.2

### Overview

`mercury-motion` is a PyPI package providing a Pythonic API for generating `.mmot.json` scenes and invoking the Mercury-Motion render pipeline. It mirrors the JavaScript SDK's class structure but adopts Python conventions (snake_case, keyword arguments, context managers). A `pandas` integration enables data-driven video generation from tabular data.

### Installation

```bash
pip install mercury-motion
```

### Basic API

```python
from mercury_motion import Scene, Composition
from mercury_motion.layers import SolidLayer, TextLayer, ImageLayer, AudioLayer
from mercury_motion.animation import keyframe, Easing

# Create a scene
scene = Scene(width=1920, height=1080, fps=30, duration=300)

# Define props
scene.prop("title", type="string", default="Hello World")
scene.prop("narration", type="string", default="")

# Build composition
main = Composition("main")

main.add(SolidLayer("bg", in_frame=0, out_frame=300, color="#1a1a2e"))

main.add(TextLayer(
    "title",
    in_frame=10,
    out_frame=290,
    text="${title}",
    font={"src": "gfont://Inter:wght@700", "size": 72, "weight": 700},
    transform={
        "x": 960, "y": 540, "anchor": "center",
        "keyframes": {
            "opacity": [keyframe(0, 0), keyframe(30, 1.0, Easing.EASE_OUT)]
        }
    }
))

scene.add_composition(main)

# Serialize
import json
with open("output.mmot.json", "w") as f:
    json.dump(scene.to_dict(), f, indent=2)

# Render via CLI
result = scene.render(main, output="video.mp4", props={"title": "My Video"})
print(f"Rendered in {result.elapsed_s:.1f}s")
```

### Class Reference

#### `Scene`

```python
class Scene:
    def __init__(self, width: int, height: int, fps: int, duration: int): ...
    def prop(self, name: str, *, type: str, default: Any = None) -> None: ...
    def add_composition(self, comp: "Composition") -> None: ...
    def to_dict(self) -> dict: ...
    def to_json(self, indent: int = 2) -> str: ...
    def render(
        self,
        comp: "Composition",
        output: str,
        *,
        props: dict | None = None,
        codec: str = "av1",
        crf: int = 30,
        no_cache: bool = False,
        mmot_bin: str = "mmot",
    ) -> "RenderResult": ...
    def render_server(
        self,
        server_url: str,
        *,
        props: dict | None = None,
        auth_token: str | None = None,
        output: str,
        poll_interval: float = 1.0,
        on_progress: Callable | None = None,
    ) -> "RenderResult": ...

    @classmethod
    def from_dataframe(cls, df: "pd.DataFrame", **scene_meta) -> "DataFrameScene": ...
```

#### Layer Classes

```python
# All layer classes accept in_frame and out_frame as positional-ish keyword args
# (Python reserves 'in' as a keyword; the SDK uses in_frame/out_frame)

class SolidLayer:
    def __init__(self, id: str, *, in_frame: int, out_frame: int, color: str, **kwargs): ...

class TextLayer:
    def __init__(self, id: str, *, in_frame: int, out_frame: int, text: str, font: dict, **kwargs): ...

class ImageLayer:
    def __init__(self, id: str, *, src: str, in_frame: int, out_frame: int, **kwargs): ...

class VideoLayer:
    def __init__(self, id: str, *, src: str, in_frame: int, out_frame: int, **kwargs): ...

class AudioLayer:
    def __init__(self, id: str, *, src: str, in_frame: int, out_frame: int, volume: float = 1.0, **kwargs): ...

class LottieLayer:
    def __init__(self, id: str, *, src: str, in_frame: int, out_frame: int, loop: bool = False, **kwargs): ...
```

### Pandas Integration

`Scene.from_dataframe(df)` generates a composition with one text or image layer per row in a DataFrame. This is the primary API for data-driven video workflows such as automated social media content, data visualization clips, or personalized video generation at scale.

```python
import pandas as pd
from mercury_motion import Scene

df = pd.DataFrame([
    {"name": "Alice",   "score": 98, "avatar": "unsplash://photo-aaa"},
    {"name": "Bob",     "score": 87, "avatar": "unsplash://photo-bbb"},
    {"name": "Charlie", "score": 76, "avatar": "unsplash://photo-ccc"},
])

scene = Scene.from_dataframe(
    df,
    width=1080, height=1080, fps=30,
    template="leaderboard-card",     # built-in template or path to .mmot.json template
    row_duration=90,                  # frames per row
    columns={
        "name":   {"layer_id": "name-text",  "type": "text"},
        "score":  {"layer_id": "score-text", "type": "text"},
        "avatar": {"layer_id": "avatar-img", "type": "image", "src_column": True},
    }
)

scene.render_batch(
    output_dir="./renders/",
    filename_column="name",    # use df["name"] as output filename
    props_per_row=True,        # each row becomes a separate render job
    concurrency=4,
)
```

**`from_dataframe` behavior:**

- Each row in the DataFrame produces one rendered video (when `props_per_row=True`) or one segment in a single video timeline (when `props_per_row=False`).
- Column values are mapped to prop values; the template `.mmot.json` uses `${column_name}` props.
- `src_column=True` on a column indicates the column contains `src` values (file paths, `data:` URIs, or resolver URIs like `unsplash://`).
- `render_batch()` spawns up to `concurrency` parallel `mmot render` processes.

**Example: single-video mode (all rows in sequence):**

```python
scene = Scene.from_dataframe(df, width=1080, height=1080, fps=30, row_duration=90)
scene.render(scene.main_composition, output="leaderboard.mp4")
# Produces a 270-frame video (3 rows × 90 frames each)
```

### CLI Spawn Mechanism (Python)

```python
import subprocess
import json

def render_via_cli(scene_dict: dict, output: str, **kwargs) -> subprocess.CompletedProcess:
    mmot_bin = kwargs.get("mmot_bin", "mmot")
    props = kwargs.get("props", {})
    args = [
        mmot_bin, "render", "--stdin",
        "--output", output,
        "--codec", kwargs.get("codec", "av1"),
        "--crf", str(kwargs.get("crf", 30)),
    ]
    if props:
        args += ["--props", json.dumps(props)]
    if kwargs.get("no_cache"):
        args.append("--no-cache")

    return subprocess.run(
        args,
        input=json.dumps(scene_dict),
        text=True,
        check=True,
    )
```

### Type Hints and IDE Support

The SDK ships a `py.typed` marker and full type stubs (`mercury_motion/*.pyi`). All public API functions and classes are fully annotated. `pandas` types are annotated with `TYPE_CHECKING` guards so the package is importable without pandas installed.

### Configuration

The Python SDK respects the same `~/.mmot/config.toml` and `MMOT_*` environment variables as the Rust engine. No additional Python-specific configuration is required. API keys set in `config.toml` are used by the `mmot` CLI subprocess automatically.

### Limitations

- Rendering requires the `mmot` binary to be installed separately; the Python package does not bundle the engine.
- `render_batch()` parallelism is process-level (spawns multiple `mmot` processes); it does not use `rayon` intra-process parallelism, so CPU utilization across jobs may not be optimal on very large machines.
- The `from_dataframe` template system is limited to prop substitution; arbitrary layer manipulation per row requires constructing the `Scene` programmatically rather than using `from_dataframe`.
- The WASM renderer is not available from Python; all rendering goes through the CLI subprocess.
- Python 3.10+ is required (uses `match` statements internally and `X | Y` union type syntax in annotations).
