# @mmot/react

**Write React. Render natively.**

A Remotion-compatible API that serializes to `.mmot.json` for Mercury-Motion's native Rust renderer. Same familiar API, 100x faster than headless Chrome.

## Install

```bash
npm install @mmot/react
```

## Quick Start

```typescript
import { Scene, interpolate, spring, Easing, Effects, rect, star, linearGradient } from '@mmot/react'
import { render } from '@mmot/react/render'

const scene = new Scene({
  width: 1920, height: 1080, fps: 30, durationInFrames: 90,
  name: 'My Video', background: '#0a0a1a',
})

// Background gradient
scene.addGradient('bg', {
  gradient: linearGradient([
    { offset: 0, color: '#1a1a3e' },
    { offset: 1, color: '#0a0a1a' },
  ]),
  fill: 'parent',
})

// Animated title
scene.addText('title', {
  text: 'Hello World',
  font: { family: 'Inter', size: 72, weight: 700, color: '#ffffff' },
  transform: {
    position: [960, 540],
    opacity: interpolate(0, [0, 30], [0, 1]),
  },
})

// Render to MP4 via native Rust engine
await render(scene, { output: 'video.mp4' })
```

## Why?

| | Remotion | @mmot/react |
|---|---|---|
| Rendering | Headless Chrome | Native Rust (CPU/GPU) |
| Speed | ~1 FPS | ~100 FPS |
| Dependencies | Chrome, Node.js | Single binary (`mmot`) |
| Output | MP4, WebM, GIF | MP4, WebM, GIF |
| API | React components | Builder API (Remotion-compatible) |

## API Reference

### Scene

The `Scene` class builds a complete `.mmot.json` video description.

```typescript
const scene = new Scene({
  width: 1920,      // Canvas width in pixels
  height: 1080,     // Canvas height in pixels
  fps: 30,          // Frames per second
  durationInFrames: 90,  // Total frames
  background: '#000000', // Background color (optional)
  name: 'My Video',     // Scene name (optional)
})
```

#### Layer Methods

All layer methods return `this` for chaining.

```typescript
scene
  .addSolid(id, { color, transform?, effects?, fill?, blendMode? })
  .addText(id, { text, font?, align?, transform?, effects? })
  .addShape(id, { shape, transform?, effects?, blendMode? })
  .addGradient(id, { gradient, transform?, fill? })
  .addImage(id, { src, transform?, effects? })
  .addVideo(id, { src, trimStart?, trimEnd?, transform? })
  .addAudio(id, { src, volume? })
  .addNull(id, { transform? })
  .setSequence(transition?)
```

#### Serialization

```typescript
scene.toJSON()    // Returns plain object
scene.toString()  // Returns formatted JSON string
```

### Animation

#### `interpolate(frame, inputRange, outputRange, options?)`

Create keyframes that interpolate a value over a frame range. Matches Remotion's API.

```typescript
const opacity = interpolate(0, [0, 30], [0, 1])
const position = interpolate(0, [0, 60], [0, 1920], { easing: Easing.easeOut })
```

#### `spring(config?)`

Create keyframes with spring physics easing.

```typescript
const scale = spring({ fps: 30, config: { stiffness: 170, damping: 26 } })
const bounce = spring({ from: 0, to: 100, config: { stiffness: 300, damping: 10 } })
```

#### `keyframes(kfs)`

Create explicit keyframe arrays for complex animations.

```typescript
const position = keyframes([
  { frame: 0, value: [0, 540], easing: 'ease_in_out' },
  { frame: 30, value: [960, 540] },
  { frame: 60, value: [1920, 540] },
])
```

#### `Easing`

Easing presets matching Remotion.

```typescript
Easing.linear      // 'linear'
Easing.easeIn      // 'ease_in'
Easing.easeOut     // 'ease_out'
Easing.easeInOut   // 'ease_in_out'
Easing.bezier(x1, y1, x2, y2)  // Custom cubic bezier
```

### Shapes

```typescript
rect(width, height, { cornerRadius?, fill?, stroke? })
ellipse(width, height, { fill?, stroke? })
line(x1, y1, x2, y2, stroke)
polygon(points, { fill?, stroke? })
star(outerRadius, innerRadius, numPoints, { fill?, stroke? })
```

### Gradients

```typescript
linearGradient(colors, start?, end?)
radialGradient(colors, center?, radius?)
```

### Effects

```typescript
Effects.blur(radius)
Effects.shadow({ color?, x?, y?, blur?, opacity? })
Effects.glow({ color?, radius?, intensity? })
Effects.brightnessContrast(brightness, contrast)
Effects.hueSaturation(hue, saturation, lightness?)
Effects.invert()
Effects.tint(color, amount?)
Effects.fill(color, opacity?)
```

### Render

```typescript
import { render, renderMp4, renderGif, renderWebm, exportJson } from '@mmot/react/render'

await render(scene, { output: 'video.mp4', quality: 80, verbose: true })
await renderMp4(scene, 'video.mp4')
await renderGif(scene, 'animation.gif')
await renderWebm(scene, 'video.webm')

// Export JSON without rendering
exportJson(scene, 'scene.mmot.json')
```

## Links

- [Mercury-Motion](https://github.com/antutroll27/mercury-motion) - Native Rust video engine
- [mmot CLI](https://crates.io/crates/mmot) - Command-line renderer
- [.mmot.json format](https://github.com/antutroll27/mercury-motion/blob/main/docs/superpowers/specs/mmot-json-format.md) - Scene description format

## License

MIT
