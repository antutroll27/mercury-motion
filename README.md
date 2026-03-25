# Mercury-Motion

Native Rust video creation engine. Define videos in JSON, render to MP4/WebM/GIF. 100x faster than Remotion.

## Install

```bash
cargo install mmot
```

## Quick Start

Create a file called `hello.mmot.json`:

```json
{
  "meta": { "width": 1920, "height": 1080, "fps": 30, "duration": 90, "root": "main" },
  "compositions": {
    "main": {
      "layers": [
        {
          "id": "bg", "type": "solid", "in": 0, "out": 90,
          "color": "#1a1a2e",
          "transform": { "position": [0, 0], "scale": [1, 1], "rotation": 0, "opacity": 1 }
        },
        {
          "id": "title", "type": "text", "in": 0, "out": 90,
          "text": "Hello Mercury-Motion",
          "font_size": 72, "color": "#ffffff", "align": "center",
          "transform": { "position": [960, 540], "scale": [1, 1], "rotation": 0, "opacity": 1 }
        }
      ]
    }
  },
  "assets": { "files": [], "fonts": [] }
}
```

Render it:

```bash
mmot render hello.mmot.json --output hello.mp4 --quality 80
```

## Features

- **Layers:** Solid colors, text, images, video, shapes (rect, ellipse, line, polygon), gradients (linear, radial)
- **Animation:** Keyframe interpolation with easing (ease-in, ease-out, cubic bezier, spring physics)
- **Composition:** Nested precomps, sequences, crossfade transitions, AbsoluteFill layout
- **Output:** MP4 (AV1), WebM, animated GIF
- **Audio:** Decode MP3/WAV/FLAC/OGG/AAC, Opus encoding, audio muxing into MP4 (with `ffmpeg` feature)
- **Video layers:** Composite video clips at specific timestamps (with `ffmpeg` feature)
- **Performance:** Parallel frame rendering via rayon, pure Rust AV1 encoder (rav1e)
- **Templates:** `${variable}` substitution in JSON for dynamic content
- **Custom fonts:** Load .ttf/.otf font files

## Feature Flags

```bash
# Default: CPU rendering + AV1/MP4 output (zero C dependencies)
cargo install mmot

# With audio encoding (requires libopus)
cargo install mmot --features audio-codec

# With video layers, audio muxing, WebM output (requires FFmpeg dev libs)
cargo install mmot --features ffmpeg
```

## CLI

```bash
# Render to MP4
mmot render scene.mmot.json --output video.mp4

# Render to GIF
mmot render scene.mmot.json --output anim.gif --format gif

# Render to WebM (requires --features ffmpeg)
mmot render scene.mmot.json --output video.webm --format webm

# With template variables
mmot render template.mmot.json --prop name=Alice --prop color=#ff0000

# Validate without rendering
mmot validate scene.mmot.json

# Control quality and concurrency
mmot render scene.mmot.json --quality 90 --concurrency 8 --verbose
```

## After Effects Features

Mercury-Motion supports core After Effects compositing features:

- **Blend Modes:** Normal, Multiply, Screen, Overlay, Darken, Lighten, ColorDodge, ColorBurn, HardLight, SoftLight, Difference, Exclusion, Add
- **Layer Parenting:** Child layers inherit parent transforms. Use Null layers for grouping.
- **Masks:** Rect, Ellipse, and freeform Path masks with feathering
- **Effects:** Gaussian Blur, Drop Shadow, Glow, Brightness/Contrast, Hue/Saturation, Invert, Tint, Fill
- **Adjustment Layers:** Apply effects to all layers below
- **Motion Blur:** 5-sample temporal averaging for moving layers
- **Time Remapping:** Speed up, slow down, or reverse layer playback
- **Trim Paths:** Animate stroke drawing on shapes
- **Path Animation:** Move layers along bezier paths with auto-orient

## Use with AI (Claude Code, Codex, Gemini)

Mercury-Motion works as an AI-native video tool. Instead of a GUI, describe what you want and the AI creates it.

### MCP Server (Claude Code)

Add to your `.mcp.json`:

```json
{
  "mcpServers": {
    "mmot": {
      "command": "npx",
      "args": ["-y", "mmot-mcp"]
    }
  }
}
```

Then tell Claude: *"Create a 5-second animation with a red circle moving left to right with a drop shadow and motion blur"*

### Skill File (Any AI Tool)

Copy `skills/mercury-motion/SKILL.md` into your AI tool's system prompt. Works with Claude Code, Codex, Gemini CLI, or any LLM.

### JSON Schema

Full JSON Schema at `schema/mmot.schema.json` for IDE autocomplete and validation.

### npm

```bash
npm install mercury-motion
npx mmot render scene.mmot.json --output video.mp4
```

## As a Library

```rust
use mmot_core::pipeline::{render_scene, RenderOptions, OutputFormat, RenderBackend};
use std::collections::HashMap;

let json = std::fs::read_to_string("scene.mmot.json")?;
let opts = RenderOptions {
    output_path: "output.mp4".into(),
    format: OutputFormat::Mp4,
    quality: 80,
    frame_range: None,
    concurrency: None,
    backend: RenderBackend::Cpu,
    include_audio: false,
};
render_scene(&json, opts, None)?;
```

## License

MIT
