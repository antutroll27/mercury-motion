# Mercury-Motion — `.mmot.json` Format Specification
**Version:** 0.1.0
**Date:** 2026-03-23
**Status:** Draft — authoritative reference for parser implementation

---

## Overview

A `.mmot.json` file is a complete, portable video project. One file = one video.
No project directories, no binary blobs, no build artifacts.

Assets can be:
- Relative paths: `"./assets/logo.png"` — resolved relative to the `.mmot.json` file
- Absolute paths: `/home/user/footage/clip.mp4`
- Inline base64: `"data:image/png;base64,iVBORw0K..."` — maximum portability

---

## Top-Level Structure

```json
{
  "version": "0.1",
  "meta": { ... },
  "props": { ... },
  "compositions": [ ... ]
}
```

| Field | Type | Required | Description |
|---|---|---|---|
| `version` | `string` | Yes | Schema version. Currently `"0.1"`. |
| `meta` | `Meta` | Yes | Video dimensions, frame rate, duration. |
| `props` | `object` | No | Named variables substituted via `${varName}` anywhere in the file. |
| `compositions` | `Composition[]` | Yes | One or more compositions. The first is the root output. |

---

## `Meta`

```json
"meta": {
  "width": 1920,
  "height": 1080,
  "fps": 30,
  "duration_frames": 300,
  "background": "#000000"
}
```

| Field | Type | Required | Description |
|---|---|---|---|
| `width` | `integer` | Yes | Output width in pixels. |
| `height` | `integer` | Yes | Output height in pixels. |
| `fps` | `number` | Yes | Frames per second. Accepts fractional values (e.g. `23.976`). |
| `duration_frames` | `integer` | Yes | Total frame count. Duration in seconds = `duration_frames / fps`. |
| `background` | `string` | No | CSS hex colour. Default: `"#000000"`. |

---

## `Props`

Named variables interpolated anywhere in the file using `${varName}` syntax.
Passed at render time via CLI: `mmot render video.mmot.json --prop title="Hello World"`.

```json
"props": {
  "title": "My Video",
  "accent_color": "#FF5500",
  "logo_path": "./assets/logo.png"
}
```

Substitution is string-level: `"${title}"` → `"My Video"`.
Type coercion: if a numeric field receives a string prop value, the parser casts it.

---

## `Composition`

```json
{
  "id": "root",
  "width": 1920,
  "height": 1080,
  "layers": [ ... ]
}
```

| Field | Type | Required | Description |
|---|---|---|---|
| `id` | `string` | Yes | Unique identifier. The first composition is the render root. |
| `width` | `integer` | No | Defaults to `meta.width`. |
| `height` | `integer` | No | Defaults to `meta.height`. |
| `layers` | `Layer[]` | Yes | Rendered bottom-to-top (painter's algorithm). |

---

## `Layer`

Every layer shares these base fields:

```json
{
  "id": "my-layer",
  "type": "solid",
  "label": "Background",
  "in_frame": 0,
  "out_frame": 300,
  "transform": { ... },
  "blend_mode": "normal",
  "opacity": 1.0,
  "content": { ... }
}
```

| Field | Type | Required | Description |
|---|---|---|---|
| `id` | `string` | Yes | Unique within the composition. |
| `type` | `string` | Yes | One of: `solid`, `text`, `image`, `video`, `composition`, `shape`, `null` |
| `label` | `string` | No | Display name in the editor. Ignored by renderer. |
| `in_frame` | `integer` | No | Frame the layer becomes visible. Default: `0`. |
| `out_frame` | `integer` | No | Frame the layer disappears (exclusive). Default: `meta.duration_frames`. |
| `transform` | `Transform` | No | Position, scale, rotation, anchor. Default: identity. |
| `blend_mode` | `string` | No | CSS blend mode name. Default: `"normal"`. |
| `opacity` | `Animatable<number>` | No | 0.0–1.0. Animatable. Default: `1.0`. |
| `content` | `object` | Yes (most types) | Type-specific content. See below. |

---

## Animatable Values

Any field marked `Animatable<T>` can be either a static value or a keyframe array.

**Static:**
```json
"opacity": 0.8
```

**Animated (keyframe array):**
```json
"opacity": [
  { "frame": 0,   "value": 0.0, "easing": "ease_in_out" },
  { "frame": 30,  "value": 1.0, "easing": "linear" },
  { "frame": 270, "value": 1.0 },
  { "frame": 300, "value": 0.0 }
]
```

| Keyframe Field | Type | Required | Description |
|---|---|---|---|
| `frame` | `integer` | Yes | Frame number this keyframe applies at. |
| `value` | `T` | Yes | The value at this frame. |
| `easing` | `string` | No | Easing to the NEXT keyframe. Default: `"linear"`. |

**Easing values:**
`"linear"`, `"ease_in"`, `"ease_out"`, `"ease_in_out"`, `"step_start"`, `"step_end"`,
`"cubic_bezier(x1, y1, x2, y2)"` — e.g. `"cubic_bezier(0.25, 0.1, 0.25, 1.0)"`

Values between keyframes are interpolated. Before the first keyframe = first value. After the last keyframe = last value.

---

## `Transform`

```json
"transform": {
  "x": { "keyframes": [{ "frame": 0, "value": 960 }, { "frame": 60, "value": 100, "easing": "ease_out" }] },
  "y": 540,
  "scale_x": 1.0,
  "scale_y": 1.0,
  "rotation": 0.0,
  "anchor_x": 0.5,
  "anchor_y": 0.5
}
```

| Field | Type | Default | Description |
|---|---|---|---|
| `x` | `Animatable<number>` | `0` | X position in pixels from left edge. |
| `y` | `Animatable<number>` | `0` | Y position in pixels from top edge. |
| `scale_x` | `Animatable<number>` | `1.0` | Horizontal scale multiplier. |
| `scale_y` | `Animatable<number>` | `1.0` | Vertical scale multiplier. |
| `rotation` | `Animatable<number>` | `0.0` | Rotation in degrees, clockwise. |
| `anchor_x` | `number` | `0.5` | Horizontal anchor point (0=left, 0.5=center, 1=right). Static only. |
| `anchor_y` | `number` | `0.5` | Vertical anchor point (0=top, 0.5=center, 1=bottom). Static only. |

---

## Layer Types & Content

### `solid`

```json
{
  "type": "solid",
  "content": {
    "color": "#FF5500",
    "width": 1920,
    "height": 1080
  }
}
```

| Field | Type | Required | Description |
|---|---|---|---|
| `color` | `Animatable<string>` | Yes | CSS hex colour (`#RRGGBB` or `#RRGGBBAA`). |
| `width` | `integer` | No | Defaults to composition width. |
| `height` | `integer` | No | Defaults to composition height. |

---

### `text`

```json
{
  "type": "text",
  "content": {
    "text": "${title}",
    "font_family": "Inter",
    "font_size": 72,
    "font_weight": 700,
    "color": "#FFFFFF",
    "align": "center",
    "line_height": 1.2,
    "letter_spacing": 0.0,
    "max_width": 1600
  }
}
```

| Field | Type | Required | Description |
|---|---|---|---|
| `text` | `string` | Yes | The text content. Supports `${prop}` substitution. |
| `font_family` | `string` | Yes | Font name. System fonts + fonts in `assets/fonts/`. |
| `font_size` | `Animatable<number>` | Yes | Size in pixels. |
| `font_weight` | `integer` | No | 100–900. Default: `400`. |
| `color` | `Animatable<string>` | No | CSS hex. Default: `"#FFFFFF"`. |
| `align` | `string` | No | `"left"`, `"center"`, `"right"`. Default: `"left"`. |
| `line_height` | `number` | No | Multiplier. Default: `1.2`. |
| `letter_spacing` | `number` | No | Extra spacing in pixels. Default: `0`. |
| `max_width` | `number` | No | Wraps at this pixel width. No wrap if omitted. |
| `italic` | `boolean` | No | Default: `false`. |

---

### `image`

```json
{
  "type": "image",
  "content": {
    "src": "./assets/logo.png",
    "fit": "contain",
    "width": 400,
    "height": 400
  }
}
```

| Field | Type | Required | Description |
|---|---|---|---|
| `src` | `string` | Yes | File path or `data:` URI. |
| `fit` | `string` | No | `"fill"`, `"contain"`, `"cover"`, `"none"`. Default: `"contain"`. |
| `width` | `number` | No | Display width. Defaults to intrinsic image width. |
| `height` | `number` | No | Display height. Defaults to intrinsic image height. |

Supported formats: PNG, JPEG, WebP, GIF (first frame).

---

### `video`

```json
{
  "type": "video",
  "content": {
    "src": "./footage/clip.mp4",
    "start_frame": 0,
    "volume": 1.0,
    "fit": "cover",
    "playback_rate": 1.0,
    "loop": false
  }
}
```

| Field | Type | Required | Description |
|---|---|---|---|
| `src` | `string` | Yes | File path. Supports MP4, MOV, WebM, ProRes (with `--features ffmpeg`). BRAW/R3D via `mmot-filmtools`. |
| `start_frame` | `integer` | No | Frame offset into the source clip to start from. Default: `0`. |
| `volume` | `Animatable<number>` | No | 0.0–1.0. Default: `1.0`. |
| `fit` | `string` | No | `"fill"`, `"contain"`, `"cover"`. Default: `"cover"`. |
| `playback_rate` | `number` | No | Speed multiplier. Default: `1.0`. |
| `loop` | `boolean` | No | Loop if the clip is shorter than `out_frame - in_frame`. Default: `false`. |

---

### `shape`

```json
{
  "type": "shape",
  "content": {
    "shape": "rectangle",
    "width": 200,
    "height": 4,
    "fill": "#FFFFFF",
    "stroke": null,
    "corner_radius": 2
  }
}
```

| Field | Type | Required | Description |
|---|---|---|---|
| `shape` | `string` | Yes | `"rectangle"`, `"ellipse"`, `"line"` |
| `width` | `Animatable<number>` | Yes | Width in pixels. |
| `height` | `Animatable<number>` | Yes | Height in pixels. |
| `fill` | `Animatable<string>` | No | CSS hex fill colour. Null = no fill. |
| `stroke` | `Stroke` | No | Stroke definition. Null = no stroke. |
| `corner_radius` | `number` | No | Rectangle only. Default: `0`. |

`Stroke`: `{ "color": "#FFFFFF", "width": 2.0 }`

---

### `composition` (precomp)

```json
{
  "type": "composition",
  "content": {
    "composition_id": "intro-sequence",
    "time_remap": null
  }
}
```

Renders another composition inline. Enables reusable sub-sequences.

---

### `null`

```json
{ "type": "null" }
```

A null layer renders nothing. Used as a parent for transform inheritance (not yet implemented in Phase 1 — parenting is Phase 2).

---

## Blend Modes

Standard CSS compositing keywords:
`"normal"`, `"multiply"`, `"screen"`, `"overlay"`, `"darken"`, `"lighten"`,
`"color-dodge"`, `"color-burn"`, `"hard-light"`, `"soft-light"`, `"difference"`,
`"exclusion"`, `"hue"`, `"saturation"`, `"color"`, `"luminosity"`

---

## Complete Working Example

```json
{
  "version": "0.1",
  "meta": {
    "width": 1920,
    "height": 1080,
    "fps": 30,
    "duration_frames": 90,
    "background": "#0A0A0A"
  },
  "props": {
    "title": "Mercury-Motion",
    "subtitle": "Video at the speed of thought.",
    "accent": "#FF5500"
  },
  "compositions": [
    {
      "id": "root",
      "layers": [
        {
          "id": "bg",
          "type": "solid",
          "in_frame": 0,
          "out_frame": 90,
          "content": {
            "color": "#0A0A0A"
          }
        },
        {
          "id": "accent-bar",
          "type": "shape",
          "in_frame": 10,
          "out_frame": 90,
          "transform": {
            "x": 960,
            "y": 480,
            "scale_x": [
              { "frame": 10, "value": 0.0, "easing": "ease_out" },
              { "frame": 35, "value": 1.0 }
            ]
          },
          "content": {
            "shape": "rectangle",
            "width": 800,
            "height": 3,
            "fill": "${accent}",
            "corner_radius": 1
          }
        },
        {
          "id": "title-text",
          "type": "text",
          "in_frame": 20,
          "out_frame": 90,
          "transform": {
            "x": 960,
            "y": 520
          },
          "opacity": [
            { "frame": 20, "value": 0.0, "easing": "ease_out" },
            { "frame": 45, "value": 1.0 }
          ],
          "content": {
            "text": "${title}",
            "font_family": "Inter",
            "font_size": 96,
            "font_weight": 700,
            "color": "#FFFFFF",
            "align": "center"
          }
        },
        {
          "id": "subtitle-text",
          "type": "text",
          "in_frame": 35,
          "out_frame": 90,
          "transform": {
            "x": 960,
            "y": 620
          },
          "opacity": [
            { "frame": 35, "value": 0.0, "easing": "ease_out" },
            { "frame": 60, "value": 0.7 }
          ],
          "content": {
            "text": "${subtitle}",
            "font_family": "Inter",
            "font_size": 32,
            "font_weight": 300,
            "color": "#CCCCCC",
            "align": "center",
            "letter_spacing": 4.0
          }
        }
      ]
    }
  ]
}
```

This renders a 3-second 1080p intro sequence: accent bar wipes in, title fades up, subtitle fades in below. Renders in under 1 second on CPU.

---

## Validation Rules

The parser (`mmot validate file.mmot.json`) enforces:

- `version` must be a supported schema version
- `meta.fps` must be > 0
- `meta.duration_frames` must be > 0
- All `id` fields must be unique within their composition
- `in_frame` must be ≥ 0, `out_frame` must be > `in_frame`
- Keyframe `frame` values must be in ascending order
- `font_weight` must be one of: 100, 200, 300, 400, 500, 600, 700, 800, 900
- `composition` layers must reference a composition `id` that exists in the file
- Circular composition references are rejected
- Asset `src` paths are validated to exist at parse time (warning, not error — allows template files)

Errors use JSON Pointer paths: `"$.compositions[0].layers[2].content.font_size: expected number, got string"`

---

## CLI Usage

```bash
# Render to MP4 (default: AV1 via rav1e)
mmot render video.mmot.json

# Render with props override
mmot render video.mmot.json --prop title="Hello World" --prop accent="#00FF88"

# Render to specific output
mmot render video.mmot.json -o output.mp4

# Render with extended codecs (H.264)
mmot render video.mmot.json --features ffmpeg -c h264

# Validate without rendering
mmot validate video.mmot.json

# Scaffold a new project
mmot new my-project
```

---

## Future Extensions (Phase 2+)

These fields are reserved and will be added in later phases:

| Field | Phase | Description |
|---|---|---|
| `layer.parent_id` | 2 | Transform parenting (null layers as parents) |
| `layer.effects[]` | 2 | Per-layer effect stack |
| `layer.mask` | 2 | Alpha matte / track matte |
| `meta.audio_tracks[]` | 2 | Standalone audio layers |
| `filmtools` | 3 | Color science pipeline (per-layer or global) |
| `layer.content.lottie_src` | 2 | Lottie animation source |

---

*This spec is the source of truth for the parser. If the spec and the code disagree, fix the code.*
