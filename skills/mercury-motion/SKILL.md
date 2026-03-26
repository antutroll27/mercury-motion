# Mercury-Motion Skill

Create videos from JSON. Mercury-Motion is a native Rust video engine that renders `.mmot.json` scene files to MP4, GIF, or WebM -- like Remotion but 100x faster, no Node.js required.

**When to use this skill:** The user wants to create animations, videos, motion graphics, slideshows, social media content, or any programmatic video from structured data.

---

## Install

```bash
# Rust (recommended)
cargo install mmot

# npm wrapper
npm install -g mercury-motion
```

---

## Quick Start

### 1. Create a scene file

Save as `hello.mmot.json`:

```json
{
  "version": "1.0",
  "meta": {
    "name": "Hello",
    "width": 1920,
    "height": 1080,
    "fps": 30,
    "duration": 90,
    "background": "#0a0a1a",
    "root": "main"
  },
  "compositions": {
    "main": {
      "layers": [
        {
          "id": "bg",
          "type": "gradient",
          "in": 0,
          "out": 90,
          "gradient": {
            "gradient_type": "radial",
            "center": [0.5, 0.4],
            "radius": 0.8,
            "colors": [
              { "offset": 0.0, "color": "#1a1a3e" },
              { "offset": 1.0, "color": "#0a0a1a" }
            ]
          },
          "fill": "parent",
          "transform": { "position": [960, 540] }
        },
        {
          "id": "title",
          "type": "text",
          "in": 0,
          "out": 90,
          "text": "Hello World",
          "font": { "family": "Inter", "size": 72, "weight": 700, "color": "#f5f0e8" },
          "transform": {
            "position": [
              { "t": 0, "v": [960, 580], "easing": "ease_out" },
              { "t": 12, "v": [960, 540] }
            ],
            "opacity": [
              { "t": 0, "v": 0.0, "easing": "ease_out" },
              { "t": 12, "v": 1.0 }
            ]
          }
        }
      ]
    }
  }
}
```

### 2. Render

```bash
mmot render hello.mmot.json --output hello.mp4
```

### 3. Validate (no render)

```bash
mmot validate hello.mmot.json
```

---

## Design Principles -- Make It Look Professional

These rules separate polished motion design from amateur slideshows. Follow them and your output will look like it came from a studio.

### Timing

- **Entrances**: Use `ease_out` (fast start, gentle land). 8-12 frames for snappy, 15-20 for smooth.
- **Exits**: Use `ease_in` (gentle start, fast exit). Same frame counts.
- **Holds**: Let elements breathe. Don't animate everything simultaneously. Hold a title for 30-60 frames before the next move.
- **Stagger**: When multiple elements enter, delay each by 3-5 frames. Never all at once. Use different `in` values on layers.
- **Spring**: Use for playful/bouncy feels. `stiffness: 170, damping: 26` is a good default. Lower damping = more bounce. `stiffness: 200, damping: 15` for dramatic overshoot.
- **Total duration**: 3 seconds (90 frames at 30fps) is the sweet spot for social clips. 5 seconds (150 frames) for intros. Keep it tight.

### Color

- **Never use pure black (#000000)** as background. Use near-black with a hint of color: `#0a0a0f` (warm), `#0a0a1a` (cool), `#0f0a0a` (red tint).
- **Never use pure white (#ffffff)** for text on dark backgrounds. Use warm white `#f5f0e8` or cool white `#e8f0f5`.
- **Limit your palette**: 1 background, 1 text, 1-2 accent colors max. Three colors total is ideal.
- **Gradients add depth**: Even a subtle radial gradient background (e.g., slightly lighter center) beats a flat solid. Always prefer a gradient layer over a solid for backgrounds.
- **Opacity creates hierarchy**: Text at 0.5-0.7 opacity reads as secondary/supporting. Full 1.0 is primary only.

### Typography

- **Size hierarchy**: Headlines 48-72px, body 24-36px, captions 14-18px. Never use the same size for everything.
- **Weight hierarchy**: Bold (700) for headlines, Regular (400) for body. Never make everything bold.
- **Alignment**: Center for hero text and titles. Left-align for lower thirds, captions, and body text.
- **Font choice**: `Inter` for modern/clean. `Arial` as universal fallback. Avoid decorative fonts unless the design demands it.

### Effects

- **Drop shadows**: Use sparingly. `offset_x: 0, offset_y: 0` with `blur: 8-15` and `opacity: 0.3-0.5` for a soft glow. `offset_x: 2, offset_y: 4` for directional shadow.
- **Gaussian blur**: 3-8px for subtle depth-of-field. 20-40px for glow/light effects. Never blur text directly.
- **Glow technique**: Use a blurred shape layer with `"blend_mode": "add"` behind the glowing element. This looks more cinematic than the `glow` effect alone.
- **Less is more**: 1-2 effects per layer max. Overloading effects looks amateur.

### Composition

- **Center of interest**: Put the main element at the center `[960, 540]` for 1080p, or at golden ratio points (1/3 and 2/3 of width/height).
- **Breathing room**: Don't fill every pixel. Leave 10-20% padding on all sides. On a 1920px canvas, keep content within x: 192-1728.
- **Layering for depth**: Background (gradient) -> mid-ground (shapes/images) -> foreground (text/UI). Three layers minimum for a polished look.
- **Motion direction**: Left-to-right reads as "forward/progress". Bottom-to-top reads as "rising/growing". Match the motion direction to the content's meaning.
- **Layer order**: First layer in the array = bottom of the visual stack. Last = top. Always put background first, text last.

---

## Common Patterns

Copy these patterns directly. They are tested, tasteful building blocks.

### Pattern: Fade In From Bottom

The most versatile entrance. Works for any element. The 40px upward drift gives the impression of the element rising into place.

```json
"transform": {
  "position": [
    { "t": 0, "v": [960, 580], "easing": "ease_out" },
    { "t": 12, "v": [960, 540] }
  ],
  "opacity": [
    { "t": 0, "v": 0.0, "easing": "ease_out" },
    { "t": 12, "v": 1.0 }
  ]
}
```

### Pattern: Scale Pop (Spring Bounce)

Great for logos, icons, and emphasis moments. The spring overshoot gives it life.

```json
"transform": {
  "position": [960, 540],
  "scale": [
    { "t": 0, "v": [0.0, 0.0], "easing": { "type": "spring", "stiffness": 170, "damping": 20, "mass": 1.0 } },
    { "t": 20, "v": [1.0, 1.0] }
  ]
}
```

### Pattern: Slide In From Left

For elements that enter the frame laterally. Start off-screen at negative x, fade in quickly so the element is visible before it reaches its destination.

```json
"transform": {
  "position": [
    { "t": 0, "v": [-200, 540], "easing": "ease_out" },
    { "t": 15, "v": [960, 540] }
  ],
  "opacity": [
    { "t": 0, "v": 0.0 },
    { "t": 5, "v": 1.0 }
  ]
}
```

### Pattern: Pulse / Breathe

A looping subtle scale animation. Good for drawing attention to a CTA or icon without being distracting.

```json
"scale": [
  { "t": 0, "v": [1.0, 1.0], "easing": "ease_in_out" },
  { "t": 15, "v": [1.05, 1.05], "easing": "ease_in_out" },
  { "t": 30, "v": [1.0, 1.0] }
]
```

### Pattern: Typewriter / Staggered Entry

Use separate text layers with staggered `in` values. Each word appears 5 frames after the previous one:

```json
{ "id": "word1", "type": "text", "in": 0, "out": 90, "text": "HELLO", "font": { "family": "Inter", "size": 64, "weight": 700, "color": "#f5f0e8" }, "transform": { "position": [860, 540], "opacity": [{ "t": 0, "v": 0.0, "easing": "ease_out" }, { "t": 8, "v": 1.0 }] } },
{ "id": "word2", "type": "text", "in": 5, "out": 90, "text": "WORLD", "font": { "family": "Inter", "size": 64, "weight": 700, "color": "#f5f0e8" }, "transform": { "position": [1060, 540], "opacity": [{ "t": 5, "v": 0.0, "easing": "ease_out" }, { "t": 13, "v": 1.0 }] } }
```

### Pattern: Glow Behind Element

Layer a blurred shape behind your main element for a cinematic glow. The `add` blend mode makes it luminous.

```json
{
  "id": "glow",
  "type": "shape",
  "in": 0,
  "out": 90,
  "shape": { "shape_type": "ellipse", "width": 200, "height": 200, "fill": "#ff6b6b" },
  "effects": [{ "type": "gaussian_blur", "radius": 30.0 }],
  "blend_mode": "add",
  "transform": { "position": [960, 540], "opacity": 0.6 }
},
{
  "id": "main_text",
  "type": "text",
  "in": 0,
  "out": 90,
  "text": "GLOW",
  "font": { "family": "Inter", "size": 72, "weight": 700, "color": "#f5f0e8" },
  "transform": { "position": [960, 540] }
}
```

### Pattern: Professional Background

A radial gradient with the bright spot shifted slightly above center. This mimics top-down lighting and instantly looks more cinematic than a flat color.

```json
{
  "id": "bg",
  "type": "gradient",
  "in": 0,
  "out": 90,
  "fill": "parent",
  "gradient": {
    "gradient_type": "radial",
    "center": [0.5, 0.4],
    "radius": 0.8,
    "colors": [
      { "offset": 0.0, "color": "#1a1a3e" },
      { "offset": 1.0, "color": "#0a0a1a" }
    ]
  },
  "transform": { "position": [960, 540] }
}
```

### Pattern: Fade Out Exit

Mirror the entrance. Start easing out a few frames before the layer's `out` point:

```json
"opacity": [
  { "t": 0, "v": 0.0, "easing": "ease_out" },
  { "t": 12, "v": 1.0 },
  { "t": 75, "v": 1.0, "easing": "ease_in" },
  { "t": 88, "v": 0.0 }
]
```

### Pattern: Line Draw Reveal

Use `trim_paths` on a shape to progressively draw a line or outline. Perfect for underlines, borders, and decorative elements.

```json
{
  "id": "underline",
  "type": "shape",
  "in": 10,
  "out": 90,
  "shape": {
    "shape_type": "line",
    "x1": 760,
    "y1": 580,
    "x2": 1160,
    "y2": 580,
    "stroke": { "color": "#e94560", "width": 3.0 }
  },
  "trim_paths": {
    "start": 0.0,
    "end": [
      { "t": 10, "v": 0.0, "easing": "ease_out" },
      { "t": 28, "v": 1.0 }
    ]
  },
  "transform": { "position": [0, 0] }
}
```

---

## Resolution Quick Reference

| Use Case | Size | FPS | Notes |
|----------|------|-----|-------|
| YouTube intro | 1920x1080 | 30 | 16:9, safe for titles. Center = `[960, 540]` |
| Instagram/TikTok | 1080x1080 | 30 | Square, keep text large (48px+). Center = `[540, 540]` |
| Stories/Reels | 1080x1920 | 30 | 9:16 vertical. Center = `[540, 960]` |
| Twitter/X | 1280x720 | 30 | Smaller = faster upload. Center = `[640, 360]` |
| 4K | 3840x2160 | 30 | Double all sizes and positions from 1080p |
| GIF (web) | 400x400 | 15 | Keep under 200KB. Center = `[200, 200]` |
| GIF (Slack) | 320x320 | 10 | Keep under 100KB. Center = `[160, 160]` |

**Duration math**: `seconds * fps = frames`. A 3-second video at 30fps = 90 frames.

---

## Anti-Patterns -- Don't Do This

- **Don't animate everything at once** -- Stagger entrances by 3-5 frames. Use different `in` values.
- **Don't use random colors** -- Pick a palette of 2-3 colors and stick to it.
- **Don't forget easing** -- Linear motion looks robotic. Always use `ease_out` for entrances, `ease_in` for exits.
- **Don't make GIFs at 1920x1080** -- They'll be 5MB+. Use 400x400 or smaller for GIFs.
- **Don't stack 5 effects on one layer** -- 1-2 effects max. Let the design breathe.
- **Don't use pure black/white** -- Use near-black (`#0a0a1a`) and warm white (`#f5f0e8`).
- **Don't set all opacity to 1.0** -- Vary opacity (0.6-0.9) on secondary elements for depth and hierarchy.
- **Don't forget the background** -- A gradient background beats a flat solid every time. Use the Professional Background pattern.
- **Don't use the same font size for everything** -- Create hierarchy: headline (48-72px), body (24-36px), caption (14-18px).
- **Don't place text at the very edge** -- Keep 10-20% padding. On 1920px wide canvas, stay within x: 192-1728.
- **Don't use `"background": "#000000"` in meta** -- Use a near-black like `#0a0a1a` and add a gradient layer on top.

---

## CLI Reference

```
mmot render <file> [options]
mmot validate <file>
mmot help
```

### Render Options

| Flag | Default | Description |
|---|---|---|
| `-o, --output <path>` | `output.mp4` | Output file path |
| `-f, --format <fmt>` | `mp4` | Output format: `mp4`, `gif`, `webm` |
| `-q, --quality <0-100>` | `80` | Encoding quality |
| `--prop <key=value>` | -- | Set template variable (repeatable) |
| `--include-audio` | false | Include audio tracks in output |
| `--concurrency <n>` | auto | Max parallel render threads |
| `-v, --verbose` | false | Show detailed progress |

### Examples

```bash
mmot render scene.mmot.json -o video.mp4              # MP4
mmot render scene.mmot.json -o anim.gif -f gif         # GIF
mmot render scene.mmot.json -o video.webm -f webm      # WebM (requires ffmpeg feature)
mmot render t.mmot.json --prop title=Hello --prop color=#ff0000  # With props
mmot render scene.mmot.json -q 95 --concurrency 4     # High quality, 4 threads
mmot render scene.mmot.json --include-audio -o out.mp4 # Include audio
```

---

## Complete Schema Reference

### Scene (top-level)

Every `.mmot.json` file has this structure:

```json
{
  "version": "1.0",
  "meta": { ... },
  "compositions": { ... },
  "props": { ... },
  "assets": { ... }
}
```

| Field | Type | Required | Description |
|---|---|---|---|
| `version` | `string` | yes | Always `"1.0"` |
| `meta` | `Meta` | yes | Video metadata |
| `compositions` | `map<string, Composition>` | yes | Named compositions (at least one) |
| `props` | `map<string, PropDef>` | no | Template variable definitions |
| `assets` | `Assets` | no | External asset references |

### Meta

```json
{
  "name": "My Video",
  "width": 1920,
  "height": 1080,
  "fps": 30,
  "duration": 90,
  "background": "#0a0a1a",
  "root": "main"
}
```

| Field | Type | Required | Default | Description |
|---|---|---|---|---|
| `name` | `string` | yes | -- | Scene name |
| `width` | `integer` | yes | -- | Canvas width in pixels |
| `height` | `integer` | yes | -- | Canvas height in pixels |
| `fps` | `number` | yes | -- | Frames per second |
| `duration` | `integer` | yes | -- | Total duration in **frames** (not seconds) |
| `background` | `string` | no | `"#000000"` | Canvas background color (hex). Prefer near-black like `#0a0a1a` over pure black. |
| `root` | `string` | yes | -- | ID of the root composition to render |

Duration in seconds = `duration / fps`. Example: 90 frames at 30fps = 3 seconds.

### Composition

```json
{
  "main": {
    "layers": [ ... ],
    "sequence": false,
    "transition": { "type": "crossfade", "duration": 10 }
  }
}
```

| Field | Type | Required | Default | Description |
|---|---|---|---|---|
| `layers` | `Layer[]` | yes | -- | Ordered list of layers (first = bottom of visual stack) |
| `sequence` | `boolean` | no | `false` | When true, layers play back-to-back instead of overlapping |
| `transition` | `TransitionSpec` | no | none | Transition between consecutive layers (only used when `sequence: true`) |

### Layer (common fields)

Every layer has these fields regardless of type:

```json
{
  "id": "my_layer",
  "type": "solid",
  "in": 0,
  "out": 30,
  "transform": { ... },
  "fill": "parent",
  "blend_mode": "normal",
  "parent": "null_1",
  "time_remap": { ... },
  "masks": [ ... ],
  "track_matte": { ... },
  "adjustment": false,
  "effects": [ ... ],
  "motion_blur": false,
  "trim_paths": { ... },
  "path_animation": { ... }
}
```

| Field | Type | Required | Default | Description |
|---|---|---|---|---|
| `id` | `string` | yes | -- | Unique layer identifier |
| `type` | `string` | yes | -- | Layer type (see Layer Types below) |
| `in` | `integer` | yes | -- | Frame this layer becomes active (inclusive) |
| `out` | `integer` | yes | -- | Frame this layer becomes inactive (exclusive) |
| `transform` | `Transform` | yes | -- | Position, scale, rotation, opacity |
| `fill` | `"parent"` | no | none | When set to `"parent"`, layer fills the entire canvas (position/rotation/scale ignored) |
| `blend_mode` | `string` | no | `"normal"` | Compositing blend mode |
| `parent` | `string` | no | none | ID of parent layer -- inherits parent's transform |
| `time_remap` | `TimeRemap` | no | none | Playback speed/direction controls |
| `masks` | `Mask[]` | no | none | Clipping/compositing masks |
| `track_matte` | `TrackMatte` | no | none | Use another layer's alpha/luma as visibility mask |
| `adjustment` | `boolean` | no | `false` | When true, effects apply to all layers below |
| `effects` | `Effect[]` | no | none | Visual effects applied to this layer |
| `motion_blur` | `boolean` | no | `false` | Enable motion blur for this layer |
| `trim_paths` | `TrimPaths` | no | none | Animate which portion of a shape path is visible |
| `path_animation` | `PathAnimation` | no | none | Move the layer along a path of control points |

---

## Layer Types

### Solid

A solid color rectangle.

```json
{
  "id": "bg",
  "type": "solid",
  "in": 0,
  "out": 90,
  "color": "#1a1a2e",
  "fill": "parent",
  "transform": { "position": [960, 540] }
}
```

| Field | Type | Required | Description |
|---|---|---|---|
| `color` | `string` | yes | Hex color (`"#rrggbb"`) |

### Text

Rendered text with font control.

```json
{
  "id": "title",
  "type": "text",
  "in": 0,
  "out": 90,
  "text": "Hello World",
  "font": {
    "family": "Inter",
    "size": 48.0,
    "weight": 700,
    "color": "#f5f0e8"
  },
  "align": "center",
  "transform": { "position": [960, 540] }
}
```

| Field | Type | Required | Default | Description |
|---|---|---|---|---|
| `text` | `string` | yes | -- | The text content to display |
| `font` | `FontSpec` | yes | -- | Font specification |
| `align` | `string` | no | `"center"` | Text alignment: `"left"`, `"center"`, `"right"` |

**FontSpec:**

| Field | Type | Default | Description |
|---|---|---|---|
| `family` | `string` | required | Font family name (e.g., `"Inter"`, `"Arial"`) |
| `size` | `number` | `32.0` | Font size in pixels |
| `weight` | `integer` | `400` | Font weight (100-900) |
| `color` | `string` | `"#ffffff"` | Text color (hex) |

### Image

Display an image from disk.

```json
{
  "id": "photo",
  "type": "image",
  "in": 0,
  "out": 90,
  "src": "assets/photo.png",
  "transform": { "position": [960, 540] }
}
```

| Field | Type | Required | Description |
|---|---|---|---|
| `src` | `string` | yes | Path to image file (PNG, JPG, WebP) |

### Video

Embed a video clip as a layer (requires ffmpeg feature).

```json
{
  "id": "clip",
  "type": "video",
  "in": 0,
  "out": 150,
  "src": "assets/clip.mp4",
  "trim_start": 2.0,
  "trim_end": 7.0,
  "transform": { "position": [960, 540] }
}
```

| Field | Type | Required | Default | Description |
|---|---|---|---|---|
| `src` | `string` | yes | -- | Path to video file |
| `trim_start` | `number` | no | `0.0` | Start time offset in seconds within the source video |
| `trim_end` | `number` | no | none (plays to end) | End time in seconds within the source video |

### Audio

An audio track (no visual output).

```json
{
  "id": "music",
  "type": "audio",
  "in": 0,
  "out": 300,
  "src": "assets/bgm.mp3",
  "volume": 0.8,
  "transform": { "position": [0, 0] }
}
```

| Field | Type | Required | Default | Description |
|---|---|---|---|---|
| `src` | `string` | yes | -- | Path to audio file (MP3, WAV, OGG) |
| `volume` | `number` or animated | no | `1.0` | Volume level (0.0 to 1.0), can be keyframed |

Use `--include-audio` flag when rendering to include audio tracks.

### Shape (rect)

A rectangle shape, optionally with rounded corners and stroke.

```json
{
  "id": "card",
  "type": "shape",
  "in": 0,
  "out": 90,
  "shape": {
    "shape_type": "rect",
    "width": 400,
    "height": 250,
    "corner_radius": 16,
    "fill": "#2d3436",
    "stroke": { "color": "#ffffff", "width": 2.0 }
  },
  "transform": { "position": [960, 540] }
}
```

| Field | Type | Required | Default | Description |
|---|---|---|---|---|
| `shape_type` | `"rect"` | yes | -- | Shape discriminator |
| `width` | `number` | yes | -- | Rectangle width in pixels |
| `height` | `number` | yes | -- | Rectangle height in pixels |
| `corner_radius` | `number` | no | none | Border radius for rounded corners |
| `fill` | `string` | no | none | Fill color (hex). Omit for no fill. |
| `stroke` | `StrokeSpec` | no | none | Stroke outline |

### Shape (ellipse)

An ellipse or circle.

```json
{
  "id": "circle",
  "type": "shape",
  "in": 0,
  "out": 90,
  "shape": {
    "shape_type": "ellipse",
    "width": 200,
    "height": 200,
    "fill": "#e94560",
    "stroke": { "color": "#ffffff", "width": 1.5 }
  },
  "transform": { "position": [960, 540] }
}
```

| Field | Type | Required | Default | Description |
|---|---|---|---|---|
| `shape_type` | `"ellipse"` | yes | -- | Shape discriminator |
| `width` | `number` | yes | -- | Ellipse width (diameter along x) |
| `height` | `number` | yes | -- | Ellipse height (diameter along y) |
| `fill` | `string` | no | none | Fill color (hex) |
| `stroke` | `StrokeSpec` | no | none | Stroke outline |

### Shape (line)

A straight line between two points.

```json
{
  "id": "divider",
  "type": "shape",
  "in": 0,
  "out": 90,
  "shape": {
    "shape_type": "line",
    "x1": 100,
    "y1": 540,
    "x2": 1820,
    "y2": 540,
    "stroke": { "color": "#ffffff", "width": 2.0 }
  },
  "transform": { "position": [0, 0] }
}
```

| Field | Type | Required | Description |
|---|---|---|---|
| `shape_type` | `"line"` | yes | Shape discriminator |
| `x1` | `number` | yes | Start x coordinate |
| `y1` | `number` | yes | Start y coordinate |
| `x2` | `number` | yes | End x coordinate |
| `y2` | `number` | yes | End y coordinate |
| `stroke` | `StrokeSpec` | yes | Stroke (required for lines) |

### Shape (polygon)

A closed polygon defined by an array of points.

```json
{
  "id": "triangle",
  "type": "shape",
  "in": 0,
  "out": 90,
  "shape": {
    "shape_type": "polygon",
    "points": [[960, 340], [760, 740], [1160, 740]],
    "fill": "#6c5ce7",
    "stroke": { "color": "#ffffff", "width": 2.0 }
  },
  "transform": { "position": [0, 0] }
}
```

| Field | Type | Required | Default | Description |
|---|---|---|---|---|
| `shape_type` | `"polygon"` | yes | -- | Shape discriminator |
| `points` | `[number, number][]` | yes | -- | Array of `[x, y]` vertex coordinates |
| `fill` | `string` | no | none | Fill color (hex) |
| `stroke` | `StrokeSpec` | no | none | Stroke outline |

### StrokeSpec

Used by all shape types for outline strokes.

```json
{ "color": "#ffffff", "width": 2.0 }
```

| Field | Type | Required | Description |
|---|---|---|---|
| `color` | `string` | yes | Stroke color (hex) |
| `width` | `number` | yes | Stroke width in pixels |

### Gradient (linear)

A linear gradient fill.

```json
{
  "id": "bg_grad",
  "type": "gradient",
  "in": 0,
  "out": 90,
  "gradient": {
    "gradient_type": "linear",
    "start": [0, 0],
    "end": [1, 1],
    "colors": [
      { "offset": 0.0, "color": "#1a1a3e" },
      { "offset": 0.5, "color": "#2d1b69" },
      { "offset": 1.0, "color": "#0a0a1a" }
    ]
  },
  "fill": "parent",
  "transform": { "position": [960, 540] }
}
```

| Field | Type | Required | Description |
|---|---|---|---|
| `gradient_type` | `"linear"` | yes | Gradient discriminator |
| `start` | `[number, number]` | yes | Start point in normalized coords (0-1) |
| `end` | `[number, number]` | yes | End point in normalized coords (0-1) |
| `colors` | `GradientStop[]` | yes | Array of color stops |

### Gradient (radial)

A radial gradient fill.

```json
{
  "id": "bg_radial",
  "type": "gradient",
  "in": 0,
  "out": 90,
  "gradient": {
    "gradient_type": "radial",
    "center": [0.5, 0.4],
    "radius": 0.8,
    "colors": [
      { "offset": 0.0, "color": "#1a1a3e" },
      { "offset": 1.0, "color": "#0a0a1a" }
    ]
  },
  "fill": "parent",
  "transform": { "position": [960, 540] }
}
```

| Field | Type | Required | Description |
|---|---|---|---|
| `gradient_type` | `"radial"` | yes | Gradient discriminator |
| `center` | `[number, number]` | yes | Center point in normalized coords (0-1) |
| `radius` | `number` | yes | Radius in normalized coords |
| `colors` | `GradientStop[]` | yes | Array of color stops |

### GradientStop

```json
{ "offset": 0.5, "color": "#ff6600" }
```

| Field | Type | Description |
|---|---|---|
| `offset` | `number` | Position along gradient (0.0 to 1.0) |
| `color` | `string` | Hex color at this stop |

### Composition (nested)

Reference another composition as a layer. Used for reusable components and nesting.

```json
{
  "id": "logo_instance",
  "type": "composition",
  "in": 0,
  "out": 90,
  "composition_id": "logo_comp",
  "transform": { "position": [960, 540], "scale": [0.5, 0.5] }
}
```

| Field | Type | Required | Description |
|---|---|---|---|
| `composition_id` | `string` | yes | ID of the composition to render as this layer |

### Null

An invisible layer used for grouping transforms. Child layers reference a null layer via `parent` to inherit its transform.

```json
{
  "id": "group_anchor",
  "type": "null",
  "in": 0,
  "out": 90,
  "transform": {
    "position": [960, 540],
    "rotation": [
      { "t": 0, "v": 0, "easing": "ease_in_out" },
      { "t": 90, "v": 360 }
    ]
  }
}
```

### Lottie

Embed a Lottie animation file.

```json
{
  "id": "anim",
  "type": "lottie",
  "in": 0,
  "out": 90,
  "src": "assets/animation.json",
  "transform": { "position": [960, 540] }
}
```

| Field | Type | Required | Description |
|---|---|---|---|
| `src` | `string` | yes | Path to Lottie JSON file |

---

## Transform

All transform properties are **animatable** (can be static values or keyframed).

```json
{
  "transform": {
    "position": [960, 540],
    "scale": [1.0, 1.0],
    "rotation": 0.0,
    "opacity": 1.0
  }
}
```

| Property | Type | Default | Description |
|---|---|---|---|
| `position` | `[x, y]` or animated | `[0, 0]` | Layer center position in pixels |
| `scale` | `[sx, sy]` or animated | `[1, 1]` | Scale factors (1.0 = 100%) |
| `rotation` | `number` or animated | `0` | Rotation in degrees |
| `opacity` | `number` or animated | `1.0` | Opacity from 0.0 (transparent) to 1.0 (opaque) |

---

## Animation System

Any transform property (and `volume`, `trim_paths.start`, `trim_paths.end`, `trim_paths.offset`) can be animated with keyframes.

### Static value

Just provide the value directly:

```json
"position": [960, 540]
"opacity": 1.0
"rotation": 45
```

### Keyframed value

Provide an array of keyframe objects:

```json
"position": [
  { "t": 0,  "v": [0, 540],    "easing": "ease_out" },
  { "t": 15, "v": [960, 540],  "easing": "ease_in_out" },
  { "t": 30, "v": [1920, 540] }
]
```

| Field | Type | Required | Default | Description |
|---|---|---|---|---|
| `t` | `integer` | yes | -- | Frame number |
| `v` | value | yes | -- | Value at this keyframe (same type as the property) |
| `easing` | `EasingValue` | no | `"linear"` | Easing from this keyframe to the next. Ignored on last keyframe. |

### Easing options

**Named presets (string):**

| Value | Description | When to use |
|---|---|---|
| `"linear"` | Constant speed (default) | Continuous rotation, scrolling |
| `"ease_in"` | Slow start, fast end | Exits, things leaving the frame |
| `"ease_out"` | Fast start, slow end | Entrances, things settling into place |
| `"ease_in_out"` | Slow start and end | Looping animations, back-and-forth |

**Cubic bezier (object):**

```json
{ "type": "cubic_bezier", "x1": 0.4, "y1": 0.0, "x2": 0.2, "y2": 1.0 }
```

| Field | Type | Default | Description |
|---|---|---|---|
| `type` | `"cubic_bezier"` | required | Discriminator |
| `x1` | `number` | required | Control point 1 x |
| `y1` | `number` | required | Control point 1 y |
| `x2` | `number` | required | Control point 2 x |
| `y2` | `number` | required | Control point 2 y |

**Spring physics (object):**

```json
{ "type": "spring", "mass": 1.0, "stiffness": 170.0, "damping": 26.0 }
```

| Field | Type | Default | Description |
|---|---|---|---|
| `type` | `"spring"` | required | Discriminator |
| `mass` | `number` | `1.0` | Mass of the spring |
| `stiffness` | `number` | `100.0` | Spring stiffness. 170 is snappy, 100 is gentle. |
| `damping` | `number` | `10.0` | Damping factor. 26 = smooth settle. 15 = visible bounce. 8 = very bouncy. |

---

## Effects

Add to any layer via the `effects` array:

```json
"effects": [
  { "type": "gaussian_blur", "radius": 5.0 },
  { "type": "drop_shadow", "color": "#000000", "offset_x": 4, "offset_y": 4, "blur": 10, "opacity": 0.5 }
]
```

### gaussian_blur

```json
{ "type": "gaussian_blur", "radius": 5.0 }
```

| Field | Type | Description |
|---|---|---|
| `radius` | `number` | Blur radius in pixels |

### drop_shadow

```json
{ "type": "drop_shadow", "color": "#000000", "offset_x": 0.0, "offset_y": 4.0, "blur": 15.0, "opacity": 0.3 }
```

| Field | Type | Default | Description |
|---|---|---|---|
| `color` | `string` | required | Shadow color (hex) |
| `offset_x` | `number` | required | Horizontal offset in pixels |
| `offset_y` | `number` | required | Vertical offset in pixels |
| `blur` | `number` | required | Shadow blur radius |
| `opacity` | `number` | `1.0` | Shadow opacity (0.0 to 1.0) |

### glow

```json
{ "type": "glow", "color": "#ffffff", "radius": 10.0, "intensity": 1.5 }
```

| Field | Type | Default | Description |
|---|---|---|---|
| `color` | `string` | required | Glow color (hex) |
| `radius` | `number` | required | Glow radius in pixels |
| `intensity` | `number` | `1.0` | Glow intensity multiplier |

### brightness_contrast

```json
{ "type": "brightness_contrast", "brightness": 10.0, "contrast": 5.0 }
```

| Field | Type | Default | Description |
|---|---|---|---|
| `brightness` | `number` | `0.0` | Brightness adjustment |
| `contrast` | `number` | `0.0` | Contrast adjustment |

### hue_saturation

```json
{ "type": "hue_saturation", "hue": 30.0, "saturation": -20.0, "lightness": 10.0 }
```

| Field | Type | Default | Description |
|---|---|---|---|
| `hue` | `number` | `0.0` | Hue shift in degrees |
| `saturation` | `number` | `0.0` | Saturation adjustment |
| `lightness` | `number` | `0.0` | Lightness adjustment |

### invert

```json
{ "type": "invert" }
```

No parameters. Inverts all color channels.

### tint

```json
{ "type": "tint", "color": "#ff8800", "amount": 0.7 }
```

| Field | Type | Default | Description |
|---|---|---|---|
| `color` | `string` | required | Target tint color (hex) |
| `amount` | `number` | `1.0` | Tint strength (0.0 to 1.0) |

### fill (effect)

```json
{ "type": "fill", "color": "#00ff00", "opacity": 0.5 }
```

| Field | Type | Default | Description |
|---|---|---|---|
| `color` | `string` | required | Fill color (hex) |
| `opacity` | `number` | `1.0` | Fill opacity (0.0 to 1.0) |

---

## Blend Modes

Set on any layer via `blend_mode`:

```json
"blend_mode": "multiply"
```

Available modes: `"normal"` (default), `"multiply"`, `"screen"`, `"overlay"`, `"darken"`, `"lighten"`, `"color_dodge"`, `"color_burn"`, `"hard_light"`, `"soft_light"`, `"difference"`, `"exclusion"`, `"add"`.

**When to use**: `"add"` for glow/light effects (makes bright areas luminous). `"multiply"` for darkening overlays. `"screen"` for lightening. `"overlay"` for contrast-boosting texture overlays.

---

## Masks

Clip a layer to a geometric shape:

```json
"masks": [
  {
    "path": { "type": "ellipse", "cx": 320, "cy": 180, "rx": 100, "ry": 100 },
    "mode": "add",
    "feather": 5.0,
    "opacity": 1.0,
    "inverted": false
  }
]
```

| Field | Type | Default | Description |
|---|---|---|---|
| `path` | `MaskPath` | required | Geometric shape for the mask |
| `mode` | `string` | `"add"` | How this mask combines with others: `"add"`, `"subtract"`, `"intersect"`, `"difference"` |
| `feather` | `number` | `0.0` | Edge softness in pixels |
| `opacity` | `number` | `1.0` | Mask opacity |
| `inverted` | `boolean` | `false` | Invert the mask region |

### Mask path types

**Rect:**
```json
{ "type": "rect", "x": 0, "y": 0, "width": 100, "height": 100, "corner_radius": 5 }
```

**Ellipse:**
```json
{ "type": "ellipse", "cx": 50, "cy": 50, "rx": 30, "ry": 20 }
```

**Freeform path:**
```json
{ "type": "path", "points": [[0,0], [100,0], [100,100], [0,100]], "closed": true }
```

---

## Track Matte

Use another layer's alpha or luminance to define visibility:

```json
"track_matte": {
  "source": "matte_layer_id",
  "mode": "alpha"
}
```

| Field | Type | Default | Description |
|---|---|---|---|
| `source` | `string` | required | ID of the layer used as the matte |
| `mode` | `string` | `"alpha"` | Channel mode: `"alpha"`, `"alpha_inverted"`, `"luma"`, `"luma_inverted"` |

---

## Time Remap

Control playback speed and direction:

```json
"time_remap": {
  "speed": 2.0,
  "offset": 0.5,
  "reverse": false
}
```

| Field | Type | Default | Description |
|---|---|---|---|
| `speed` | `number` | `1.0` | Playback speed multiplier (2.0 = double speed, 0.5 = half speed) |
| `offset` | `number` | `0.0` | Time offset in seconds |
| `reverse` | `boolean` | `false` | Play in reverse |

---

## Trim Paths

Animate which portion of a shape path is visible (for line-drawing effects):

```json
"trim_paths": {
  "start": 0.0,
  "end": [
    { "t": 0, "v": 0.0, "easing": "ease_out" },
    { "t": 30, "v": 1.0 }
  ],
  "offset": 0.0
}
```

| Field | Type | Default | Description |
|---|---|---|---|
| `start` | `number` or animated | `0.0` | Start of visible portion (0.0 to 1.0) |
| `end` | `number` or animated | `1.0` | End of visible portion (0.0 to 1.0) |
| `offset` | `number` or animated | `0.0` | Rotates start/end along the path (0.0 to 1.0) |

---

## Path Animation

Move a layer along a motion path:

```json
"path_animation": {
  "points": [[100, 100], [540, 100], [540, 260], [100, 260]],
  "auto_orient": true
}
```

| Field | Type | Default | Description |
|---|---|---|---|
| `points` | `[number, number][]` | required | Control points defining the motion path |
| `auto_orient` | `boolean` | `false` | When true, layer rotation follows the path tangent |

---

## Sequences and Transitions

Set `"sequence": true` on a composition to make its layers play back-to-back. Add a `transition` for overlap effects.

```json
{
  "slideshow": {
    "sequence": true,
    "transition": { "type": "crossfade", "duration": 10 },
    "layers": [
      { "id": "slide1", "type": "solid", "in": 0, "out": 30, "color": "#ff0000", "transform": { "position": [50, 50] } },
      { "id": "slide2", "type": "solid", "in": 0, "out": 30, "color": "#0000ff", "transform": { "position": [50, 50] } }
    ]
  }
}
```

In sequence mode, each layer's `in`/`out` defines its individual duration. The engine automatically offsets them to play consecutively.

### Transition types

**Crossfade:**
```json
{ "type": "crossfade", "duration": 10 }
```

**Wipe:**
```json
{ "type": "wipe", "duration": 15, "direction": "left" }
```

**Slide:**
```json
{ "type": "slide", "duration": 15, "direction": "right" }
```

Directions: `"left"`, `"right"`, `"up"`, `"down"`.

Duration is in frames.

---

## Template Props

Define variables that can be overridden from the CLI:

```json
{
  "props": {
    "title": { "type": "string", "default": "Hello" },
    "accent_color": { "type": "color", "default": "#ff6600" },
    "count": { "type": "number", "default": 42 },
    "logo_url": { "type": "url", "default": "assets/logo.png" }
  }
}
```

Use `${varName}` in any string value within the JSON:

```json
"text": "${title}",
"color": "${accent_color}",
"src": "${logo_url}"
```

Override at render time:

```bash
mmot render template.mmot.json --prop title="My Title" --prop accent_color="#00ff00"
```

Prop types: `"string"`, `"color"`, `"number"`, `"url"`.

---

## Assets

### Fonts

Load custom fonts by specifying an ID and file path:

```json
{
  "assets": {
    "fonts": [
      { "id": "custom_font", "src": "assets/fonts/Inter-Bold.ttf" }
    ]
  }
}
```

Reference the font by family name in text layers:

```json
"font": { "family": "Inter", "size": 48, "weight": 700, "color": "#f5f0e8" }
```

---

## Recipes

### 1. Social Media Intro -- Vertical with Staggered Text and Accent Glow

A polished 1080x1920 vertical video (3 seconds). Gradient background with a warm glow spot, headline bounces in with spring, subtitle fades in staggered, and a decorative accent line draws on.

```json
{
  "version": "1.0",
  "meta": {
    "name": "SocialIntro",
    "width": 1080,
    "height": 1920,
    "fps": 30,
    "duration": 90,
    "background": "#0a0a1a",
    "root": "main"
  },
  "compositions": {
    "main": {
      "layers": [
        {
          "id": "bg",
          "type": "gradient",
          "in": 0,
          "out": 90,
          "gradient": {
            "gradient_type": "radial",
            "center": [0.5, 0.35],
            "radius": 0.9,
            "colors": [
              { "offset": 0.0, "color": "#2d1b69" },
              { "offset": 0.5, "color": "#1a1a3e" },
              { "offset": 1.0, "color": "#0a0a1a" }
            ]
          },
          "fill": "parent",
          "transform": { "position": [540, 960] }
        },
        {
          "id": "glow_spot",
          "type": "shape",
          "in": 0,
          "out": 90,
          "shape": { "shape_type": "ellipse", "width": 400, "height": 400, "fill": "#667eea" },
          "blend_mode": "add",
          "effects": [{ "type": "gaussian_blur", "radius": 60.0 }],
          "transform": { "position": [540, 860], "opacity": 0.15 }
        },
        {
          "id": "accent_line",
          "type": "shape",
          "in": 5,
          "out": 90,
          "shape": {
            "shape_type": "line",
            "x1": 390,
            "y1": 920,
            "x2": 690,
            "y2": 920,
            "stroke": { "color": "#e94560", "width": 3.0 }
          },
          "trim_paths": {
            "start": 0.0,
            "end": [
              { "t": 5, "v": 0.0, "easing": "ease_out" },
              { "t": 25, "v": 1.0 }
            ]
          },
          "transform": { "position": [0, 0] }
        },
        {
          "id": "title",
          "type": "text",
          "in": 0,
          "out": 90,
          "text": "Welcome",
          "font": { "family": "Inter", "size": 96, "weight": 900, "color": "#f5f0e8" },
          "align": "center",
          "transform": {
            "position": [
              { "t": 0, "v": [540, 920], "easing": { "type": "spring", "stiffness": 200, "damping": 18, "mass": 1.0 } },
              { "t": 25, "v": [540, 880] }
            ],
            "opacity": [
              { "t": 0, "v": 0.0, "easing": "ease_out" },
              { "t": 12, "v": 1.0 }
            ]
          },
          "effects": [
            { "type": "drop_shadow", "color": "#000000", "offset_x": 0, "offset_y": 4, "blur": 20, "opacity": 0.25 }
          ]
        },
        {
          "id": "subtitle",
          "type": "text",
          "in": 10,
          "out": 90,
          "text": "to my channel",
          "font": { "family": "Inter", "size": 36, "weight": 400, "color": "#b8b0c8" },
          "align": "center",
          "transform": {
            "position": [
              { "t": 10, "v": [540, 970], "easing": "ease_out" },
              { "t": 22, "v": [540, 950] }
            ],
            "opacity": [
              { "t": 10, "v": 0.0, "easing": "ease_out" },
              { "t": 22, "v": 0.8 }
            ]
          }
        }
      ]
    }
  }
}
```

### 2. Logo Reveal -- Mask Expand with Cinematic Glow

An elliptical mask expands from center to reveal a logo image. The glow overlay adds a cinematic flash on reveal, then fades. Near-black background with subtle warmth.

```json
{
  "version": "1.0",
  "meta": {
    "name": "LogoReveal",
    "width": 1920,
    "height": 1080,
    "fps": 30,
    "duration": 90,
    "background": "#0a0a0f",
    "root": "main"
  },
  "compositions": {
    "main": {
      "layers": [
        {
          "id": "bg",
          "type": "gradient",
          "in": 0,
          "out": 90,
          "gradient": {
            "gradient_type": "radial",
            "center": [0.5, 0.45],
            "radius": 0.7,
            "colors": [
              { "offset": 0.0, "color": "#1a1520" },
              { "offset": 1.0, "color": "#0a0a0f" }
            ]
          },
          "fill": "parent",
          "transform": { "position": [960, 540] }
        },
        {
          "id": "logo",
          "type": "image",
          "in": 0,
          "out": 90,
          "src": "assets/logo.png",
          "fill": "parent",
          "masks": [
            {
              "path": { "type": "ellipse", "cx": 960, "cy": 540, "rx": 600, "ry": 600 },
              "mode": "add",
              "feather": 25.0
            }
          ],
          "transform": {
            "position": [960, 540],
            "scale": [
              { "t": 0, "v": [0.01, 0.01], "easing": { "type": "spring", "stiffness": 120, "damping": 18, "mass": 1.0 } },
              { "t": 30, "v": [1.0, 1.0] }
            ],
            "opacity": [
              { "t": 0, "v": 0.0, "easing": "ease_out" },
              { "t": 12, "v": 1.0 }
            ]
          }
        },
        {
          "id": "reveal_flash",
          "type": "shape",
          "in": 0,
          "out": 40,
          "shape": { "shape_type": "ellipse", "width": 300, "height": 300, "fill": "#ffffff" },
          "blend_mode": "add",
          "effects": [{ "type": "gaussian_blur", "radius": 80.0 }],
          "transform": {
            "position": [960, 540],
            "opacity": [
              { "t": 0, "v": 0.0 },
              { "t": 5, "v": 0.4, "easing": "ease_in" },
              { "t": 25, "v": 0.0 }
            ]
          }
        }
      ]
    }
  }
}
```

### 3. Slideshow -- Images with Crossfade and Ken Burns Drift

Three images cross-dissolving, each with a subtle slow zoom (Ken Burns effect) to prevent static feels. Gradient background visible during transitions.

```json
{
  "version": "1.0",
  "meta": {
    "name": "Slideshow",
    "width": 1920,
    "height": 1080,
    "fps": 30,
    "duration": 270,
    "background": "#0a0a1a",
    "root": "main"
  },
  "compositions": {
    "main": {
      "sequence": true,
      "transition": { "type": "crossfade", "duration": 15 },
      "layers": [
        {
          "id": "slide1",
          "type": "image",
          "in": 0,
          "out": 90,
          "src": "assets/photo1.jpg",
          "fill": "parent",
          "transform": {
            "position": [960, 540],
            "scale": [
              { "t": 0, "v": [1.0, 1.0], "easing": "linear" },
              { "t": 90, "v": [1.05, 1.05] }
            ]
          }
        },
        {
          "id": "slide2",
          "type": "image",
          "in": 0,
          "out": 90,
          "src": "assets/photo2.jpg",
          "fill": "parent",
          "transform": {
            "position": [960, 540],
            "scale": [
              { "t": 0, "v": [1.05, 1.05], "easing": "linear" },
              { "t": 90, "v": [1.0, 1.0] }
            ]
          }
        },
        {
          "id": "slide3",
          "type": "image",
          "in": 0,
          "out": 90,
          "src": "assets/photo3.jpg",
          "fill": "parent",
          "transform": {
            "position": [960, 540],
            "scale": [
              { "t": 0, "v": [1.0, 1.0], "easing": "linear" },
              { "t": 90, "v": [1.04, 1.04] }
            ]
          }
        }
      ]
    }
  }
}
```

### 4. Animated Counter -- Data Visualization with Depth

A stat counter with gradient background, glowing accent, spring-animated number, and staggered label. Props make it reusable for any metric.

```json
{
  "version": "1.0",
  "meta": {
    "name": "Counter",
    "width": 1920,
    "height": 1080,
    "fps": 30,
    "duration": 90,
    "background": "#0a0a1a",
    "root": "main"
  },
  "props": {
    "value": { "type": "string", "default": "1,247" },
    "label": { "type": "string", "default": "Total Users" }
  },
  "compositions": {
    "main": {
      "layers": [
        {
          "id": "bg",
          "type": "gradient",
          "in": 0,
          "out": 90,
          "gradient": {
            "gradient_type": "radial",
            "center": [0.5, 0.45],
            "radius": 0.9,
            "colors": [
              { "offset": 0.0, "color": "#1a1a2e" },
              { "offset": 1.0, "color": "#0a0a12" }
            ]
          },
          "fill": "parent",
          "transform": { "position": [960, 540] }
        },
        {
          "id": "accent_glow",
          "type": "shape",
          "in": 0,
          "out": 90,
          "shape": { "shape_type": "ellipse", "width": 250, "height": 250, "fill": "#667eea" },
          "blend_mode": "add",
          "effects": [{ "type": "gaussian_blur", "radius": 50.0 }],
          "transform": { "position": [960, 500], "opacity": 0.12 }
        },
        {
          "id": "number",
          "type": "text",
          "in": 0,
          "out": 90,
          "text": "${value}",
          "font": { "family": "Inter", "size": 144, "weight": 900, "color": "#f5f0e8" },
          "align": "center",
          "transform": {
            "position": [
              { "t": 0, "v": [960, 580], "easing": { "type": "spring", "stiffness": 120, "damping": 16, "mass": 1.0 } },
              { "t": 25, "v": [960, 500] }
            ],
            "opacity": [
              { "t": 0, "v": 0.0, "easing": "ease_out" },
              { "t": 12, "v": 1.0 }
            ]
          },
          "effects": [
            { "type": "drop_shadow", "color": "#000000", "offset_x": 0, "offset_y": 4, "blur": 20, "opacity": 0.25 }
          ]
        },
        {
          "id": "divider",
          "type": "shape",
          "in": 12,
          "out": 90,
          "shape": {
            "shape_type": "line",
            "x1": 880,
            "y1": 555,
            "x2": 1040,
            "y2": 555,
            "stroke": { "color": "#e94560", "width": 2.0 }
          },
          "trim_paths": {
            "start": 0.0,
            "end": [
              { "t": 12, "v": 0.0, "easing": "ease_out" },
              { "t": 28, "v": 1.0 }
            ]
          },
          "transform": { "position": [0, 0] }
        },
        {
          "id": "label_text",
          "type": "text",
          "in": 15,
          "out": 90,
          "text": "${label}",
          "font": { "family": "Inter", "size": 28, "weight": 400, "color": "#8888aa" },
          "align": "center",
          "transform": {
            "position": [
              { "t": 15, "v": [960, 600], "easing": "ease_out" },
              { "t": 27, "v": [960, 585] }
            ],
            "opacity": [
              { "t": 15, "v": 0.0, "easing": "ease_out" },
              { "t": 27, "v": 0.7 }
            ]
          }
        }
      ]
    }
  }
}
```

Render with custom values:
```bash
mmot render counter.mmot.json --prop value="8,392" --prop label="Downloads" -o counter.mp4
```

### 5. Lower Third -- Broadcast-Quality Name Banner

A name/title banner that slides in from the left with a colored accent bar, semi-transparent card, staggered text, and smooth exit. Transparent background so it overlays on video.

```json
{
  "version": "1.0",
  "meta": {
    "name": "LowerThird",
    "width": 1920,
    "height": 1080,
    "fps": 30,
    "duration": 150,
    "background": "#00000000",
    "root": "main"
  },
  "props": {
    "name": { "type": "string", "default": "Jane Smith" },
    "title": { "type": "string", "default": "Senior Engineer" }
  },
  "compositions": {
    "main": {
      "layers": [
        {
          "id": "anchor",
          "type": "null",
          "in": 0,
          "out": 150,
          "transform": {
            "position": [
              { "t": 0, "v": [-500, 850], "easing": "ease_out" },
              { "t": 15, "v": [350, 850] },
              { "t": 120, "v": [350, 850], "easing": "ease_in" },
              { "t": 135, "v": [-500, 850] }
            ]
          }
        },
        {
          "id": "accent_bar",
          "type": "shape",
          "in": 0,
          "out": 150,
          "shape": {
            "shape_type": "rect",
            "width": 4,
            "height": 80,
            "corner_radius": 2,
            "fill": "#e94560"
          },
          "parent": "anchor",
          "transform": { "position": [-2, 0] }
        },
        {
          "id": "bg_card",
          "type": "shape",
          "in": 0,
          "out": 150,
          "shape": {
            "shape_type": "rect",
            "width": 500,
            "height": 80,
            "corner_radius": 4,
            "fill": "#0a0a1a"
          },
          "parent": "anchor",
          "transform": { "position": [253, 0], "opacity": 0.85 }
        },
        {
          "id": "person_name",
          "type": "text",
          "in": 5,
          "out": 150,
          "text": "${name}",
          "font": { "family": "Inter", "size": 30, "weight": 700, "color": "#f5f0e8" },
          "align": "left",
          "parent": "anchor",
          "transform": {
            "position": [30, -12],
            "opacity": [
              { "t": 5, "v": 0.0, "easing": "ease_out" },
              { "t": 15, "v": 1.0 }
            ]
          }
        },
        {
          "id": "person_title",
          "type": "text",
          "in": 8,
          "out": 150,
          "text": "${title}",
          "font": { "family": "Inter", "size": 18, "weight": 400, "color": "#8888aa" },
          "align": "left",
          "parent": "anchor",
          "transform": {
            "position": [30, 18],
            "opacity": [
              { "t": 8, "v": 0.0, "easing": "ease_out" },
              { "t": 18, "v": 0.75 }
            ]
          }
        }
      ]
    }
  }
}
```

### 6. Orbiting Shapes -- Particles with Cinematic Depth

Small shapes orbiting around a glowing center point. Multiple orbit speeds, varied colors from a limited palette, and an adjustment layer for unified color grading.

```json
{
  "version": "1.0",
  "meta": {
    "name": "OrbitingShapes",
    "width": 1920,
    "height": 1080,
    "fps": 30,
    "duration": 120,
    "background": "#0a0a1a",
    "root": "main"
  },
  "compositions": {
    "main": {
      "layers": [
        {
          "id": "bg",
          "type": "gradient",
          "in": 0,
          "out": 120,
          "gradient": {
            "gradient_type": "radial",
            "center": [0.5, 0.5],
            "radius": 0.7,
            "colors": [
              { "offset": 0.0, "color": "#12122a" },
              { "offset": 1.0, "color": "#0a0a14" }
            ]
          },
          "fill": "parent",
          "transform": { "position": [960, 540] }
        },
        {
          "id": "center_glow",
          "type": "shape",
          "in": 0,
          "out": 120,
          "shape": {
            "shape_type": "ellipse",
            "width": 40,
            "height": 40,
            "fill": "#f5f0e8"
          },
          "transform": { "position": [960, 540] },
          "effects": [
            { "type": "glow", "color": "#667eea", "radius": 30, "intensity": 2.0 }
          ]
        },
        {
          "id": "orbit1",
          "type": "shape",
          "in": 0,
          "out": 120,
          "shape": {
            "shape_type": "ellipse",
            "width": 20,
            "height": 20,
            "fill": "#e94560"
          },
          "transform": { "position": [960, 540] },
          "path_animation": {
            "points": [[1160, 540], [960, 340], [760, 540], [960, 740]],
            "auto_orient": false
          },
          "effects": [
            { "type": "glow", "color": "#e94560", "radius": 15, "intensity": 1.5 }
          ],
          "motion_blur": true
        },
        {
          "id": "orbit2",
          "type": "shape",
          "in": 0,
          "out": 120,
          "shape": {
            "shape_type": "rect",
            "width": 14,
            "height": 14,
            "corner_radius": 3,
            "fill": "#667eea"
          },
          "transform": {
            "position": [960, 540],
            "rotation": [
              { "t": 0, "v": 0, "easing": "linear" },
              { "t": 120, "v": 720 }
            ]
          },
          "path_animation": {
            "points": [[810, 540], [960, 390], [1110, 540], [960, 690]],
            "auto_orient": true
          },
          "effects": [
            { "type": "glow", "color": "#667eea", "radius": 12, "intensity": 1.2 }
          ],
          "motion_blur": true
        },
        {
          "id": "orbit3",
          "type": "shape",
          "in": 0,
          "out": 120,
          "shape": {
            "shape_type": "ellipse",
            "width": 10,
            "height": 10,
            "fill": "#f5f0e8"
          },
          "transform": { "position": [960, 540], "opacity": 0.7 },
          "time_remap": { "speed": 0.7 },
          "path_animation": {
            "points": [[1260, 540], [960, 240], [660, 540], [960, 840]],
            "auto_orient": false
          },
          "effects": [
            { "type": "glow", "color": "#f5f0e8", "radius": 8, "intensity": 0.8 }
          ]
        },
        {
          "id": "color_grade",
          "type": "solid",
          "in": 0,
          "out": 120,
          "color": "#000000",
          "adjustment": true,
          "transform": { "position": [0, 0] },
          "effects": [
            { "type": "brightness_contrast", "brightness": 3, "contrast": 6 }
          ]
        }
      ]
    }
  }
}
```

### 7. Hero Title Card -- YouTube / Presentation Opener

A centered title with subtitle, gradient background, glow accent, and staggered entrance. Clean, professional, works for any topic.

```json
{
  "version": "1.0",
  "meta": {
    "name": "TitleCard",
    "width": 1920,
    "height": 1080,
    "fps": 30,
    "duration": 120,
    "background": "#0a0a1a",
    "root": "main"
  },
  "props": {
    "headline": { "type": "string", "default": "The Future of Video" },
    "tagline": { "type": "string", "default": "Faster. Sharper. Open Source." }
  },
  "compositions": {
    "main": {
      "layers": [
        {
          "id": "bg",
          "type": "gradient",
          "in": 0,
          "out": 120,
          "gradient": {
            "gradient_type": "radial",
            "center": [0.5, 0.4],
            "radius": 0.8,
            "colors": [
              { "offset": 0.0, "color": "#1a1a3e" },
              { "offset": 1.0, "color": "#0a0a1a" }
            ]
          },
          "fill": "parent",
          "transform": { "position": [960, 540] }
        },
        {
          "id": "glow_accent",
          "type": "shape",
          "in": 0,
          "out": 120,
          "shape": { "shape_type": "ellipse", "width": 500, "height": 300, "fill": "#e94560" },
          "blend_mode": "add",
          "effects": [{ "type": "gaussian_blur", "radius": 80.0 }],
          "transform": { "position": [960, 520], "opacity": 0.08 }
        },
        {
          "id": "headline",
          "type": "text",
          "in": 0,
          "out": 120,
          "text": "${headline}",
          "font": { "family": "Inter", "size": 72, "weight": 900, "color": "#f5f0e8" },
          "align": "center",
          "transform": {
            "position": [
              { "t": 0, "v": [960, 560], "easing": "ease_out" },
              { "t": 15, "v": [960, 520] }
            ],
            "opacity": [
              { "t": 0, "v": 0.0, "easing": "ease_out" },
              { "t": 15, "v": 1.0 }
            ]
          },
          "effects": [
            { "type": "drop_shadow", "color": "#000000", "offset_x": 0, "offset_y": 2, "blur": 15, "opacity": 0.3 }
          ]
        },
        {
          "id": "divider_line",
          "type": "shape",
          "in": 10,
          "out": 120,
          "shape": {
            "shape_type": "line",
            "x1": 860,
            "y1": 560,
            "x2": 1060,
            "y2": 560,
            "stroke": { "color": "#e94560", "width": 2.0 }
          },
          "trim_paths": {
            "start": 0.0,
            "end": [
              { "t": 10, "v": 0.0, "easing": "ease_out" },
              { "t": 25, "v": 1.0 }
            ]
          },
          "transform": { "position": [0, 0] }
        },
        {
          "id": "tagline",
          "type": "text",
          "in": 15,
          "out": 120,
          "text": "${tagline}",
          "font": { "family": "Inter", "size": 28, "weight": 400, "color": "#8888aa" },
          "align": "center",
          "transform": {
            "position": [
              { "t": 15, "v": [960, 610], "easing": "ease_out" },
              { "t": 27, "v": [960, 595] }
            ],
            "opacity": [
              { "t": 15, "v": 0.0, "easing": "ease_out" },
              { "t": 27, "v": 0.75 }
            ]
          }
        }
      ]
    }
  }
}
```

---

## Tips and Best Practices

1. **Full-canvas backgrounds:** Use `"fill": "parent"` on background layers (solid, gradient, image). This ignores transform position and stretches the layer to fill the canvas.

2. **Fade in/out:** Animate `opacity` with keyframes. Use `ease_out` for fade in, `ease_in` for fade out. See the Fade Out Exit pattern.

3. **Null layers for grouped transforms:** Create a `"type": "null"` layer and set `"parent": "null_id"` on child layers. Moving the null moves all children together. Essential for lower thirds and complex UI.

4. **Stacking effects:** The `effects` array applies effects in order. Keep it to 1-2 effects per layer. Use separate layers for complex compositions instead of stacking 5 effects.

5. **Sequences for slideshows:** Set `"sequence": true` with a `"transition"` on the composition. Each layer defines its own duration via `in`/`out`. The engine chains them automatically.

6. **Spring easing for organic motion:** Use `{ "type": "spring", "stiffness": 170, "damping": 26 }` for snappy settling. Lower damping (15-18) for visible bounce. Very low (8-10) for playful/cartoon feels.

7. **Adjustment layers:** Set `"adjustment": true` on a solid layer with effects to apply those effects to all layers below it. Perfect for unified color grading at the end.

8. **Template props for reusable videos:** Define `"props"` in the scene, use `${varName}` in strings, and override with `--prop` at render time. Make every recipe reusable.

9. **Layer ordering:** Layers render bottom-to-top. The first layer in the array is the bottom-most (background), the last is the top-most (foreground). Always: gradient bg -> shapes/glow -> text.

10. **Duration math:** `duration` is in frames. To calculate: `seconds * fps = frames`. A 5-second video at 30fps = 150 frames.

11. **Line-drawing effect:** Use `trim_paths` on shape layers. Animate `end` from 0 to 1 to draw a shape progressively. Great for underlines, borders, and decorative accents.

12. **Track mattes for reveals:** Use `track_matte` to reveal one layer based on another layer's shape/brightness -- great for text reveals and wipe effects.

13. **Ken Burns effect:** On slideshow images, animate scale from `[1.0, 1.0]` to `[1.05, 1.05]` over the slide duration with linear easing. Prevents the "dead slide" look.

14. **Glow technique:** Place a blurred shape with `"blend_mode": "add"` and low opacity (0.08-0.15) behind your main content. This creates ambient lighting that adds cinematic depth without overpowering.

15. **Color hierarchy:** Primary content at full opacity (1.0). Secondary text at 0.7-0.8. Tertiary/decorative elements at 0.5-0.6. This creates visual depth without extra effects.
