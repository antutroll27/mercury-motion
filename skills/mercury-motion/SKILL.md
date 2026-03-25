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
    "background": "#1a1a2e",
    "root": "main"
  },
  "compositions": {
    "main": {
      "layers": [
        {
          "id": "title",
          "type": "text",
          "in": 0,
          "out": 90,
          "text": "Hello World",
          "font": { "family": "Arial", "size": 72, "weight": 700, "color": "#ffffff" },
          "transform": {
            "position": [960, 540],
            "opacity": [
              { "t": 0, "v": 0.0, "easing": "ease_out" },
              { "t": 15, "v": 1.0 }
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
  "background": "#000000",
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
| `background` | `string` | no | `"#000000"` | Canvas background color (hex) |
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
    "family": "Arial",
    "size": 48.0,
    "weight": 700,
    "color": "#ffffff"
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
| `family` | `string` | required | Font family name (e.g., `"Arial"`, `"Inter"`) |
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
      { "offset": 0.0, "color": "#ff0000" },
      { "offset": 0.5, "color": "#00ff00" },
      { "offset": 1.0, "color": "#0000ff" }
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
    "center": [0.5, 0.5],
    "radius": 0.8,
    "colors": [
      { "offset": 0.0, "color": "#16213e" },
      { "offset": 1.0, "color": "#0f3460" }
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

| Value | Description |
|---|---|
| `"linear"` | Constant speed (default) |
| `"ease_in"` | Slow start, fast end |
| `"ease_out"` | Fast start, slow end |
| `"ease_in_out"` | Slow start and end |

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
| `stiffness` | `number` | `100.0` | Spring stiffness |
| `damping` | `number` | `10.0` | Damping factor (lower = more bouncy) |

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
{ "type": "drop_shadow", "color": "#000000", "offset_x": 4.0, "offset_y": 4.0, "blur": 10.0, "opacity": 0.5 }
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
"font": { "family": "Inter", "size": 48, "weight": 700, "color": "#ffffff" }
```

---

## Recipes

### 1. Social Media Intro -- Animated Text with Gradient Background

A 1080x1920 vertical video (3 seconds) with a gradient background and text that fades in with spring bounce.

```json
{
  "version": "1.0",
  "meta": {
    "name": "SocialIntro",
    "width": 1080,
    "height": 1920,
    "fps": 30,
    "duration": 90,
    "background": "#000000",
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
            "gradient_type": "linear",
            "start": [0.5, 0],
            "end": [0.5, 1],
            "colors": [
              { "offset": 0.0, "color": "#667eea" },
              { "offset": 1.0, "color": "#764ba2" }
            ]
          },
          "fill": "parent",
          "transform": { "position": [540, 960] }
        },
        {
          "id": "title",
          "type": "text",
          "in": 0,
          "out": 90,
          "text": "Welcome",
          "font": { "family": "Arial", "size": 96, "weight": 900, "color": "#ffffff" },
          "align": "center",
          "transform": {
            "position": [
              { "t": 0, "v": [540, 1100], "easing": { "type": "spring", "stiffness": 200, "damping": 15 } },
              { "t": 30, "v": [540, 960] }
            ],
            "opacity": [
              { "t": 0, "v": 0.0, "easing": "ease_out" },
              { "t": 20, "v": 1.0 }
            ]
          }
        },
        {
          "id": "subtitle",
          "type": "text",
          "in": 10,
          "out": 90,
          "text": "to my channel",
          "font": { "family": "Arial", "size": 48, "weight": 400, "color": "#ffffffcc" },
          "align": "center",
          "transform": {
            "position": [540, 1060],
            "opacity": [
              { "t": 10, "v": 0.0, "easing": "ease_out" },
              { "t": 25, "v": 1.0 }
            ]
          }
        }
      ]
    }
  }
}
```

### 2. Logo Reveal -- Shape Mask Expanding to Reveal an Image

An elliptical mask expands from center to reveal a logo image.

```json
{
  "version": "1.0",
  "meta": {
    "name": "LogoReveal",
    "width": 1920,
    "height": 1080,
    "fps": 30,
    "duration": 60,
    "background": "#0a0a0a",
    "root": "main"
  },
  "compositions": {
    "main": {
      "layers": [
        {
          "id": "bg",
          "type": "solid",
          "in": 0,
          "out": 60,
          "color": "#0a0a0a",
          "fill": "parent",
          "transform": { "position": [960, 540] }
        },
        {
          "id": "logo",
          "type": "image",
          "in": 0,
          "out": 60,
          "src": "assets/logo.png",
          "fill": "parent",
          "masks": [
            {
              "path": { "type": "ellipse", "cx": 960, "cy": 540, "rx": 600, "ry": 600 },
              "mode": "add",
              "feather": 20.0
            }
          ],
          "transform": {
            "position": [960, 540],
            "scale": [
              { "t": 0, "v": [0.01, 0.01], "easing": "ease_out" },
              { "t": 30, "v": [1.0, 1.0] }
            ],
            "opacity": [
              { "t": 0, "v": 0.0, "easing": "ease_out" },
              { "t": 15, "v": 1.0 }
            ]
          }
        },
        {
          "id": "glow_overlay",
          "type": "solid",
          "in": 0,
          "out": 60,
          "color": "#ffffff",
          "fill": "parent",
          "blend_mode": "add",
          "transform": {
            "opacity": [
              { "t": 0, "v": 0.3, "easing": "ease_out" },
              { "t": 20, "v": 0.0 }
            ]
          }
        }
      ]
    }
  }
}
```

### 3. Slideshow -- Sequence of Images with Crossfade Transitions

Three images cross-dissolving into each other over 9 seconds.

```json
{
  "version": "1.0",
  "meta": {
    "name": "Slideshow",
    "width": 1920,
    "height": 1080,
    "fps": 30,
    "duration": 270,
    "background": "#000000",
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
          "transform": { "position": [960, 540] }
        },
        {
          "id": "slide2",
          "type": "image",
          "in": 0,
          "out": 90,
          "src": "assets/photo2.jpg",
          "fill": "parent",
          "transform": { "position": [960, 540] }
        },
        {
          "id": "slide3",
          "type": "image",
          "in": 0,
          "out": 90,
          "src": "assets/photo3.jpg",
          "fill": "parent",
          "transform": { "position": [960, 540] }
        }
      ]
    }
  }
}
```

### 4. Animated Counter -- Numbers Counting Up with Easing

A number label that smoothly slides upward into frame. Combine with text updates via props for dynamic data.

```json
{
  "version": "1.0",
  "meta": {
    "name": "Counter",
    "width": 1920,
    "height": 1080,
    "fps": 30,
    "duration": 90,
    "background": "#1a1a2e",
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
            "center": [0.5, 0.5],
            "radius": 1.0,
            "colors": [
              { "offset": 0.0, "color": "#2d3436" },
              { "offset": 1.0, "color": "#0a0a0a" }
            ]
          },
          "fill": "parent",
          "transform": { "position": [960, 540] }
        },
        {
          "id": "number",
          "type": "text",
          "in": 0,
          "out": 90,
          "text": "${value}",
          "font": { "family": "Arial", "size": 144, "weight": 900, "color": "#ffffff" },
          "align": "center",
          "transform": {
            "position": [
              { "t": 0, "v": [960, 640], "easing": { "type": "spring", "stiffness": 120, "damping": 14 } },
              { "t": 25, "v": [960, 500] }
            ],
            "opacity": [
              { "t": 0, "v": 0.0, "easing": "ease_out" },
              { "t": 15, "v": 1.0 }
            ]
          },
          "effects": [
            { "type": "drop_shadow", "color": "#000000", "offset_x": 0, "offset_y": 4, "blur": 20, "opacity": 0.3 }
          ]
        },
        {
          "id": "label_text",
          "type": "text",
          "in": 10,
          "out": 90,
          "text": "${label}",
          "font": { "family": "Arial", "size": 36, "weight": 400, "color": "#aaaaaa" },
          "align": "center",
          "transform": {
            "position": [960, 580],
            "opacity": [
              { "t": 10, "v": 0.0, "easing": "ease_out" },
              { "t": 25, "v": 1.0 }
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

### 5. Lower Third -- News-Style Name Banner Sliding In

A name/title banner that slides in from the left with a colored accent bar.

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
            "width": 6,
            "height": 80,
            "fill": "#e94560"
          },
          "parent": "anchor",
          "transform": { "position": [-3, 0] }
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
            "fill": "#1a1a2eee"
          },
          "parent": "anchor",
          "transform": { "position": [253, 0] }
        },
        {
          "id": "person_name",
          "type": "text",
          "in": 5,
          "out": 150,
          "text": "${name}",
          "font": { "family": "Arial", "size": 32, "weight": 700, "color": "#ffffff" },
          "align": "left",
          "parent": "anchor",
          "transform": {
            "position": [30, -12],
            "opacity": [
              { "t": 5, "v": 0.0, "easing": "ease_out" },
              { "t": 18, "v": 1.0 }
            ]
          }
        },
        {
          "id": "person_title",
          "type": "text",
          "in": 8,
          "out": 150,
          "text": "${title}",
          "font": { "family": "Arial", "size": 20, "weight": 400, "color": "#aaaaaa" },
          "align": "left",
          "parent": "anchor",
          "transform": {
            "position": [30, 18],
            "opacity": [
              { "t": 8, "v": 0.0, "easing": "ease_out" },
              { "t": 20, "v": 1.0 }
            ]
          }
        }
      ]
    }
  }
}
```

### 6. Orbiting Shapes -- Multiple Shapes with Path Animation

Small shapes orbiting around a center point with trail effects.

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
          "id": "center_glow",
          "type": "shape",
          "in": 0,
          "out": 120,
          "shape": {
            "shape_type": "ellipse",
            "width": 40,
            "height": 40,
            "fill": "#ffffff"
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
            "width": 16,
            "height": 16,
            "corner_radius": 3,
            "fill": "#00b894"
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
            { "type": "glow", "color": "#00b894", "radius": 12, "intensity": 1.2 }
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
            "width": 12,
            "height": 12,
            "fill": "#fdcb6e"
          },
          "transform": { "position": [960, 540] },
          "time_remap": { "speed": 0.7 },
          "path_animation": {
            "points": [[1260, 540], [960, 240], [660, 540], [960, 840]],
            "auto_orient": false
          },
          "effects": [
            { "type": "glow", "color": "#fdcb6e", "radius": 10, "intensity": 1.0 }
          ]
        },
        {
          "id": "color_adjust",
          "type": "solid",
          "in": 0,
          "out": 120,
          "color": "#000000",
          "adjustment": true,
          "transform": { "position": [0, 0] },
          "effects": [
            { "type": "brightness_contrast", "brightness": 5, "contrast": 8 }
          ]
        }
      ]
    }
  }
}
```

---

## Tips and Best Practices

1. **Full-canvas backgrounds:** Use `"fill": "parent"` on background layers (solid, gradient, image). This ignores transform position and stretches the layer to fill the canvas.

2. **Fade in/out:** Animate `opacity` with keyframes. Fade in from 0 to 1 at the start; fade out from 1 to 0 before `out`.

3. **Null layers for grouped transforms:** Create a `"type": "null"` layer and set `"parent": "null_id"` on child layers. Moving the null moves all children together.

4. **Stacking effects:** The `effects` array applies effects in order. Combine blur + shadow + color correction for polished looks.

5. **Sequences for slideshows:** Set `"sequence": true` with a `"transition"` on the composition. Each layer defines its own duration via `in`/`out`. The engine chains them automatically.

6. **Spring easing for organic motion:** Use `{ "type": "spring", "stiffness": 170, "damping": 26 }` for bouncy, natural-feeling animations. Lower damping = more bounce.

7. **Adjustment layers:** Set `"adjustment": true` on a solid layer with effects to apply those effects to all layers below it in the composition.

8. **Template props for reusable videos:** Define `"props"` in the scene, use `${varName}` in strings, and override with `--prop` at render time.

9. **Layer ordering:** Layers render bottom-to-top. The first layer in the array is the bottom-most (background), the last is the top-most (foreground).

10. **Duration math:** `duration` is in frames. To calculate: `seconds * fps = frames`. A 5-second video at 30fps = 150 frames.

11. **Common resolutions:**
    - 1920x1080 (16:9 landscape, YouTube/1080p)
    - 1080x1920 (9:16 portrait, Instagram Reels/TikTok)
    - 1080x1080 (1:1 square, Instagram post)
    - 3840x2160 (4K)

12. **Line-drawing effect:** Use `trim_paths` on shape layers. Animate `end` from 0 to 1 to draw a shape progressively.

13. **Track mattes for reveals:** Use `track_matte` to reveal one layer based on another layer's shape/brightness -- great for text reveals and wipe effects.
