<script setup lang="ts">
import { ref, onMounted, watch, nextTick } from 'vue'
import { useSceneStore } from '../stores/scene'
import CanvasGizmos from './CanvasGizmos.vue'

const store = useSceneStore()
const canvasRef = ref<HTMLCanvasElement | null>(null)
const isTauri = ref(false)

// Image cache: src URL → loaded HTMLImageElement
const imageCache = new Map<string, HTMLImageElement>()

function loadImage(src: string): Promise<HTMLImageElement> {
  if (imageCache.has(src)) return Promise.resolve(imageCache.get(src)!)
  return new Promise((resolve, reject) => {
    const img = new Image()
    img.crossOrigin = 'anonymous'
    img.onload = () => { imageCache.set(src, img); resolve(img) }
    img.onerror = () => reject(new Error(`Failed to load image: ${src}`))
    img.src = src
  })
}

// Preload all image/video layer sources
async function preloadImages() {
  const comp = store.scene.compositions[store.scene.meta.root]
  if (!comp) return
  const loads: Promise<any>[] = []
  for (const layer of comp.layers) {
    if ((layer.type === 'image' || layer.type === 'video') && (layer as any).src) {
      loads.push(loadImage((layer as any).src).catch(() => {}))
    }
  }
  await Promise.all(loads)
}

// Check if we're in Tauri
onMounted(async () => {
  try {
    const { invoke } = await import('@tauri-apps/api/core')
    await invoke('get_scene')
    isTauri.value = true
    store.requestPreview()
  } catch {
    isTauri.value = false
    renderBrowserPreview()
  }
})

watch(() => store.currentFrame, () => {
  if (isTauri.value) {
    store.requestPreview()
  } else {
    renderBrowserPreview()
  }
})

watch(() => store.previewImage, (img) => {
  if (img && isTauri.value) return // Tauri preview uses <img> tag
})

// Watch for any layer changes in browser mode
watch(() => JSON.stringify(store.scene), () => {
  if (!isTauri.value) {
    preloadImages().then(() => nextTick(() => renderBrowserPreview()))
  }
}, { deep: false })

function parseHexColor(hex: string): string {
  return hex.startsWith('#') ? hex : `#${hex}`
}

// --- Keyframe interpolation (matches Rust evaluator) ---

function isKeyframeArray(val: any): val is { t: number; v: any; easing?: any }[] {
  return Array.isArray(val) && val.length > 0 && typeof val[0] === 'object' && 't' in val[0]
}

function lerp(a: number, b: number, t: number): number {
  return a + (b - a) * t
}

function applyEasing(t: number, easing: any): number {
  if (!easing || easing === 'linear') return t
  const presets: Record<string, [number, number, number, number]> = {
    ease_in: [0.42, 0, 1, 1],
    ease_out: [0, 0, 0.58, 1],
    ease_in_out: [0.42, 0, 0.58, 1],
  }
  let cx1 = 0, cy1 = 0, cx2 = 1, cy2 = 1
  if (typeof easing === 'string' && presets[easing]) {
    [cx1, cy1, cx2, cy2] = presets[easing]
  } else if (easing?.type === 'cubic_bezier') {
    cx1 = easing.x1; cy1 = easing.y1; cx2 = easing.x2; cy2 = easing.y2
  } else if (easing?.type === 'spring') {
    // Approximate spring with ease_out
    [cx1, cy1, cx2, cy2] = [0, 0, 0.58, 1]
  } else {
    return t
  }
  // Newton's method cubic bezier solver (8 iterations)
  let x = t
  for (let i = 0; i < 8; i++) {
    const bx = 3 * cx1, dx = 3 * (cx2 - cx1) - bx, ax = 1 - bx - dx
    const curX = ((ax * x + dx) * x + bx) * x
    const slope = (3 * ax * x + 2 * dx) * x + bx
    if (Math.abs(slope) < 1e-6) break
    x -= (curX - t) / slope
  }
  const by = 3 * cy1, dy = 3 * (cy2 - cy1) - by, ay = 1 - by - dy
  return ((ay * x + dy) * x + by) * x
}

function evalNumber(val: any, frame: number): number {
  if (typeof val === 'number') return val
  if (isKeyframeArray(val)) {
    const kfs = val
    if (kfs.length === 0) return 0
    if (frame <= kfs[0].t) return kfs[0].v
    if (frame >= kfs[kfs.length - 1].t) return kfs[kfs.length - 1].v
    for (let i = 0; i < kfs.length - 1; i++) {
      if (frame >= kfs[i].t && frame <= kfs[i + 1].t) {
        const span = kfs[i + 1].t - kfs[i].t
        const rawT = span > 0 ? (frame - kfs[i].t) / span : 0
        const t = applyEasing(Math.max(0, Math.min(1, rawT)), kfs[i].easing)
        return lerp(kfs[i].v, kfs[i + 1].v, t)
      }
    }
    return kfs[kfs.length - 1].v
  }
  return typeof val === 'number' ? val : 0
}

function evalVec2(val: any, frame: number): [number, number] {
  if (Array.isArray(val) && val.length === 2 && typeof val[0] === 'number') {
    return val as [number, number]
  }
  if (isKeyframeArray(val)) {
    const kfs = val
    if (kfs.length === 0) return [0, 0]
    if (frame <= kfs[0].t) return kfs[0].v
    if (frame >= kfs[kfs.length - 1].t) return kfs[kfs.length - 1].v
    for (let i = 0; i < kfs.length - 1; i++) {
      if (frame >= kfs[i].t && frame <= kfs[i + 1].t) {
        const span = kfs[i + 1].t - kfs[i].t
        const rawT = span > 0 ? (frame - kfs[i].t) / span : 0
        const t = applyEasing(Math.max(0, Math.min(1, rawT)), kfs[i].easing)
        return [
          lerp(kfs[i].v[0], kfs[i + 1].v[0], t),
          lerp(kfs[i].v[1], kfs[i + 1].v[1], t),
        ]
      }
    }
    return kfs[kfs.length - 1].v
  }
  return [0, 0]
}

// Map mmot blend modes to Canvas composite operations
const blendModeMap: Record<string, GlobalCompositeOperation> = {
  normal: 'source-over',
  multiply: 'multiply',
  screen: 'screen',
  overlay: 'overlay',
  darken: 'darken',
  lighten: 'lighten',
  color_dodge: 'color-dodge',
  color_burn: 'color-burn',
  hard_light: 'hard-light',
  soft_light: 'soft-light',
  difference: 'difference',
  exclusion: 'exclusion',
  add: 'lighter',
}

function renderBrowserPreview() {
  const canvas = canvasRef.value
  if (!canvas) return

  const meta = store.scene.meta
  canvas.width = meta.width
  canvas.height = meta.height

  const ctx = canvas.getContext('2d')
  if (!ctx) return

  // Clear with background
  ctx.fillStyle = parseHexColor(meta.background || '#000000')
  ctx.fillRect(0, 0, meta.width, meta.height)

  const comp = store.scene.compositions[meta.root]
  if (!comp) return

  const frame = store.currentFrame

  for (const layer of comp.layers) {
    // Skip layers outside their time range
    if (frame < layer.in || frame >= layer.out) continue

    ctx.save()

    // Apply transform — interpolate keyframes at current frame
    const pos = evalVec2(layer.transform.position, frame)
    const scale = evalVec2(layer.transform.scale ?? [1, 1], frame)
    const rotation = evalNumber(layer.transform.rotation ?? 0, frame)
    const opacity = evalNumber(layer.transform.opacity ?? 1, frame)

    ctx.globalAlpha = Math.max(0, Math.min(1, opacity))

    ctx.translate(pos[0], pos[1])
    ctx.rotate((rotation * Math.PI) / 180)
    ctx.scale(scale[0], scale[1])

    // Blend mode
    if ((layer as any).blend_mode && blendModeMap[(layer as any).blend_mode]) {
      ctx.globalCompositeOperation = blendModeMap[(layer as any).blend_mode]
    }

    // Apply effects
    const effects = (layer as any).effects as any[] | undefined
    if (effects?.length) {
      const filters: string[] = []
      for (const effect of effects) {
        switch (effect.type) {
          case 'gaussian_blur':
            filters.push(`blur(${effect.radius || 5}px)`)
            break
          case 'drop_shadow':
            ctx.shadowColor = effect.color || '#000000'
            ctx.shadowBlur = effect.blur || 8
            ctx.shadowOffsetX = effect.offset_x || 0
            ctx.shadowOffsetY = effect.offset_y || 4
            break
          case 'glow':
            ctx.shadowColor = effect.color || '#ffffff'
            ctx.shadowBlur = (effect.radius || 10) * (effect.intensity || 1)
            ctx.shadowOffsetX = 0
            ctx.shadowOffsetY = 0
            break
          case 'brightness_contrast': {
            const b = 1 + (effect.brightness || 0) / 100
            const c = 1 + (effect.contrast || 0) / 100
            filters.push(`brightness(${b}) contrast(${c})`)
            break
          }
          case 'hue_saturation': {
            const h = effect.hue || 0
            const s = 1 + (effect.saturation || 0) / 100
            const l = 1 + (effect.lightness || 0) / 100
            filters.push(`hue-rotate(${h}deg) saturate(${s}) brightness(${l})`)
            break
          }
          case 'invert':
            filters.push('invert(1)')
            break
          case 'tint':
            // Approximate tint with sepia + hue-rotate
            filters.push(`sepia(${effect.amount || 1})`)
            break
        }
      }
      if (filters.length > 0) {
        ctx.filter = filters.join(' ')
      }
    }

    // Apply masks
    const masks = (layer as any).masks as any[] | undefined
    if (masks?.length) {
      for (const mask of masks) {
        const path = mask.path
        if (!path) continue

        ctx.beginPath()
        switch (path.type) {
          case 'rect':
            ctx.rect(
              (path.x || 0) - pos[0],
              (path.y || 0) - pos[1],
              path.width || 100,
              path.height || 100
            )
            break
          case 'ellipse':
            ctx.ellipse(
              (path.cx || 0) - pos[0],
              (path.cy || 0) - pos[1],
              path.rx || 50,
              path.ry || 50,
              0, 0, Math.PI * 2
            )
            break
          case 'path':
            if (path.points?.length) {
              ctx.moveTo(path.points[0][0], path.points[0][1])
              for (let i = 1; i < path.points.length; i++) {
                ctx.lineTo(path.points[i][0], path.points[i][1])
              }
              if (path.closed !== false) ctx.closePath()
            }
            break
        }

        // Apply mask mode
        if (mask.mode === 'subtract') {
          ctx.clip('evenodd')
        } else {
          ctx.clip()
        }
      }
    }

    switch (layer.type) {
      case 'solid': {
        const color = (layer as any).color || '#ffffff'
        ctx.fillStyle = parseHexColor(color)
        // If fill parent, draw full canvas; otherwise draw centered rect
        if ((layer as any).fill === 'parent') {
          ctx.translate(-pos[0], -pos[1])
          ctx.fillRect(0, 0, meta.width, meta.height)
        } else {
          ctx.fillRect(-meta.width / 2, -meta.height / 2, meta.width, meta.height)
        }
        break
      }
      case 'text': {
        const text = (layer as any).text || ''
        const font = (layer as any).font || {}
        const size = font.size || 48
        const weight = font.weight || 400
        const family = font.family || 'Inter'
        const color = font.color || '#ffffff'
        ctx.fillStyle = parseHexColor(color)
        ctx.font = `${weight} ${size}px ${family}, sans-serif`
        ctx.textAlign = 'center'
        ctx.textBaseline = 'middle'
        ctx.fillText(text, 0, 0)
        break
      }
      case 'shape': {
        const shape = (layer as any).shape
        if (!shape) break
        const fill = shape.fill ? parseHexColor(shape.fill) : null
        const strokeSpec = shape.stroke
        switch (shape.shape_type) {
          case 'rect': {
            const w = shape.width || 100
            const h = shape.height || 100
            const cr = shape.corner_radius || 0
            if (fill) {
              ctx.fillStyle = fill
              if (cr > 0) {
                roundRect(ctx, -w / 2, -h / 2, w, h, cr)
                ctx.fill()
              } else {
                ctx.fillRect(-w / 2, -h / 2, w, h)
              }
            }
            if (strokeSpec) {
              ctx.strokeStyle = parseHexColor(strokeSpec.color)
              ctx.lineWidth = strokeSpec.width
              ctx.strokeRect(-w / 2, -h / 2, w, h)
            }
            break
          }
          case 'ellipse': {
            const w = shape.width || 100
            const h = shape.height || 100
            ctx.beginPath()
            ctx.ellipse(0, 0, w / 2, h / 2, 0, 0, Math.PI * 2)
            if (fill) { ctx.fillStyle = fill; ctx.fill() }
            if (strokeSpec) { ctx.strokeStyle = parseHexColor(strokeSpec.color); ctx.lineWidth = strokeSpec.width; ctx.stroke() }
            break
          }
          case 'polygon': {
            if (shape.points?.length) {
              ctx.beginPath()
              ctx.moveTo(shape.points[0][0], shape.points[0][1])
              for (let i = 1; i < shape.points.length; i++) {
                ctx.lineTo(shape.points[i][0], shape.points[i][1])
              }
              ctx.closePath()
              if (fill) { ctx.fillStyle = fill; ctx.fill() }
              if (strokeSpec) { ctx.strokeStyle = parseHexColor(strokeSpec.color); ctx.lineWidth = strokeSpec.width; ctx.stroke() }
            }
            break
          }
          case 'line': {
            ctx.beginPath()
            ctx.moveTo(shape.x1 || 0, shape.y1 || 0)
            ctx.lineTo(shape.x2 || 100, shape.y2 || 0)
            if (strokeSpec) {
              ctx.strokeStyle = parseHexColor(strokeSpec.color)
              ctx.lineWidth = strokeSpec.width || 2
              ctx.stroke()
            }
            break
          }
        }
        break
      }
      case 'gradient': {
        const g = (layer as any).gradient
        if (!g) break
        let grad: CanvasGradient
        if (g.gradient_type === 'linear') {
          grad = ctx.createLinearGradient(
            g.start[0] * meta.width - pos[0], g.start[1] * meta.height - pos[1],
            g.end[0] * meta.width - pos[0], g.end[1] * meta.height - pos[1]
          )
        } else {
          grad = ctx.createRadialGradient(0, 0, 0, 0, 0, g.radius * Math.max(meta.width, meta.height))
        }
        for (const stop of g.colors || []) {
          grad.addColorStop(stop.offset, parseHexColor(stop.color))
        }
        ctx.fillStyle = grad
        ctx.fillRect(-meta.width / 2, -meta.height / 2, meta.width, meta.height)
        break
      }
      case 'image':
      case 'video': {
        const src = (layer as any).src
        if (src && imageCache.has(src)) {
          const img = imageCache.get(src)!
          const iw = img.naturalWidth
          const ih = img.naturalHeight
          ctx.drawImage(img, -iw / 2, -ih / 2, iw, ih)
        } else if (src) {
          // Image not loaded yet — show placeholder
          ctx.strokeStyle = '#669BBC'
          ctx.lineWidth = 2
          ctx.strokeRect(-50, -30, 100, 60)
          ctx.fillStyle = '#669BBC'
          ctx.font = '12px Inter, sans-serif'
          ctx.textAlign = 'center'
          ctx.textBaseline = 'middle'
          ctx.fillText(layer.type === 'video' ? '▶ loading...' : '◻ loading...', 0, 0)
          // Trigger async load
          loadImage(src).then(() => renderBrowserPreview())
        }
        break
      }
      case 'null':
        // Invisible — transform only
        break
    }

    // Reset effects
    ctx.filter = 'none'
    ctx.shadowColor = 'transparent'
    ctx.shadowBlur = 0
    ctx.shadowOffsetX = 0
    ctx.shadowOffsetY = 0
    ctx.globalCompositeOperation = 'source-over'

    ctx.restore()
  }

  // Draw frame info overlay
  ctx.save()
  ctx.fillStyle = 'rgba(0,0,0,0.5)'
  ctx.fillRect(0, meta.height - 24, 120, 24)
  ctx.fillStyle = '#F5F0E8'
  ctx.font = '11px JetBrains Mono, monospace'
  ctx.fillText(`Frame ${frame}`, 8, meta.height - 8)
  ctx.restore()
}

function roundRect(ctx: CanvasRenderingContext2D, x: number, y: number, w: number, h: number, r: number) {
  ctx.beginPath()
  ctx.moveTo(x + r, y)
  ctx.lineTo(x + w - r, y)
  ctx.quadraticCurveTo(x + w, y, x + w, y + r)
  ctx.lineTo(x + w, y + h - r)
  ctx.quadraticCurveTo(x + w, y + h, x + w - r, y + h)
  ctx.lineTo(x + r, y + h)
  ctx.quadraticCurveTo(x, y + h, x, y + h - r)
  ctx.lineTo(x, y + r)
  ctx.quadraticCurveTo(x, y, x + r, y)
  ctx.closePath()
}
</script>

<template>
  <div class="flex flex-col items-center justify-center bg-cosmos-deep p-4 gap-3">
    <div class="font-mono text-[10px] text-text-muted uppercase tracking-[0.2em]">
      Preview
      <span v-if="!isTauri" class="text-marble ml-2">(browser)</span>
    </div>

    <div class="relative bg-black rounded border border-cosmos-border shadow-2xl overflow-hidden"
         :style="{ aspectRatio: `${store.scene.meta.width}/${store.scene.meta.height}`, maxWidth: '100%', maxHeight: 'calc(100% - 40px)' }">

      <!-- Tauri rendered preview -->
      <img v-if="isTauri && store.previewImage" :src="store.previewImage" class="w-full h-full object-contain" />

      <!-- Browser canvas preview -->
      <canvas
        v-if="!isTauri"
        ref="canvasRef"
        :width="store.scene.meta.width"
        :height="store.scene.meta.height"
        class="w-full h-full object-contain"
      />

      <!-- Empty state (Tauri mode, no preview yet) -->
      <div v-if="isTauri && !store.previewImage"
           class="absolute inset-0 flex flex-col items-center justify-center gap-2">
        <div class="w-12 h-12 border border-cosmos-border rounded-full flex items-center justify-center animate-pulse">
          <div class="w-4 h-4 bg-crimson/50 rounded-full"></div>
        </div>
        <span class="font-mono text-xs text-text-muted uppercase tracking-widest">Rendering...</span>
      </div>

      <!-- Transform gizmos overlay -->
      <CanvasGizmos />
    </div>
  </div>
</template>
