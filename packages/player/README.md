# @mmot/player

Embed Mercury-Motion animations anywhere -- zero dependencies, pure Canvas 2D.

Renders `.mmot.json` scenes directly in the browser with play/pause/scrub controls. No WASM, no backend, no build step required.

For production rendering, use `mmot render` CLI. This player is for preview and embedding.

## Install

```bash
npm install @mmot/player
```

## Usage

### Vanilla HTML

```html
<div id="preview" style="max-width: 800px"></div>

<script type="module">
  import { MmotPlayer } from '@mmot/player'

  const scene = {
    version: '1.0',
    meta: { name: 'Demo', width: 1920, height: 1080, fps: 30, duration: 90, background: '#000000', root: 'main' },
    compositions: {
      main: {
        layers: [
          {
            id: 'title',
            type: 'text',
            in: 0, out: 90,
            text: 'Hello World',
            font: { family: 'Inter', size: 72, weight: 700, color: '#ffffff' },
            transform: { position: [960, 540], scale: [1, 1], opacity: 1, rotation: 0 },
          }
        ]
      }
    }
  }

  const player = new MmotPlayer('#preview', {
    scene,
    autoplay: true,
    loop: true,
    controls: true,
  })
</script>
```

### React

```tsx
import { useEffect, useRef } from 'react'
import { MmotPlayer } from '@mmot/player'

export function Player({ scene }) {
  const ref = useRef<HTMLDivElement>(null)

  useEffect(() => {
    if (!ref.current) return
    const player = new MmotPlayer(ref.current, {
      scene,
      autoplay: true,
      loop: true,
      controls: true,
    })
    return () => player.destroy()
  }, [scene])

  return <div ref={ref} style={{ maxWidth: 800 }} />
}
```

### Vue 3

```vue
<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue'
import { MmotPlayer } from '@mmot/player'

const props = defineProps<{ scene: any }>()
const containerRef = ref<HTMLElement>()
let player: MmotPlayer

onMounted(() => {
  if (!containerRef.value) return
  player = new MmotPlayer(containerRef.value, {
    scene: props.scene,
    autoplay: true,
    loop: true,
    controls: true,
  })
})

onUnmounted(() => player?.destroy())
</script>

<template>
  <div ref="containerRef" style="max-width: 800px" />
</template>
```

## API

### Constructor

```ts
new MmotPlayer(selector: string | HTMLElement, options: PlayerOptions)
```

### PlayerOptions

| Option | Type | Default | Description |
|---|---|---|---|
| `scene` | `object` | *required* | Parsed `.mmot.json` scene data |
| `width` | `number` | scene.meta.width | Canvas width in pixels |
| `height` | `number` | scene.meta.height | Canvas height in pixels |
| `fps` | `number` | scene.meta.fps | Playback framerate |
| `autoplay` | `boolean` | `false` | Start playing immediately |
| `loop` | `boolean` | `false` | Loop playback |
| `controls` | `boolean` | `true` | Show built-in controls |
| `background` | `string` | `'#000'` | Container background color |
| `onFrameChange` | `(frame) => void` | - | Frame change callback |
| `onPlay` | `() => void` | - | Playback start callback |
| `onPause` | `() => void` | - | Playback pause callback |
| `onEnd` | `() => void` | - | Playback end callback |

### Methods

| Method | Description |
|---|---|
| `play()` | Start playback |
| `pause()` | Pause playback |
| `toggle()` | Toggle play/pause |
| `seekTo(frame)` | Seek to a specific frame |
| `setScene(scene)` | Replace the scene data and re-render |
| `destroy()` | Remove from DOM and clean up |

### Properties

| Property | Type | Description |
|---|---|---|
| `totalFrames` | `number` | Total frame count from scene metadata |
| `currentFrame` | `number` | Current frame number |
| `isPlaying` | `boolean` | Whether playback is active |
| `canvasElement` | `HTMLCanvasElement` | The underlying canvas element |

### Standalone Exports

For custom rendering without the player UI:

```ts
import { renderFrame, evaluateValue, evaluateVec2, evaluateColor } from '@mmot/player'

// Render a single frame to your own canvas
const ctx = myCanvas.getContext('2d')
renderFrame(ctx, sceneData, frameNumber, 1920, 1080)

// Evaluate a keyframed value at a specific frame
const opacity = evaluateValue([{ t: 0, v: 0 }, { t: 30, v: 1 }], 15)  // 0.5
const pos = evaluateVec2([960, 540], 0)  // [960, 540]
```

## Supported Features

### Layer types
- **solid** -- filled rectangle with color
- **text** -- styled text with font family, size, weight, color, alignment
- **shape** -- rect (with corner radius), ellipse, line, polygon (fill + stroke)
- **gradient** -- linear and radial gradients with color stops
- **null** -- invisible transform-only layer (for parenting)

### Animation
- Keyframe interpolation with `t` (frame) and `v` (value)
- Cubic bezier easing (custom control points or presets: linear, ease_in, ease_out, ease_in_out)
- Spring physics easing (mass, stiffness, damping)
- Vec2 keyframes for position/scale animation
- Color keyframe interpolation

### Transforms
- Position (x, y)
- Scale (x, y)
- Rotation (degrees)
- Opacity (0-1)

### Effects
- Gaussian blur
- Drop shadow (color, offset, blur)
- Glow
- Brightness/Contrast
- Hue/Saturation/Lightness
- Invert

### Compositing
- Blend modes: normal, multiply, screen, overlay, darken, lighten, color-dodge, color-burn, hard-light, soft-light, difference, exclusion, add
- Fill mode (`"parent"` fills entire canvas)

### Masks
- Rect masks (with corner radius)
- Ellipse masks
- Freeform path masks
- Mask modes: add, subtract, intersect, difference

## License

MIT
