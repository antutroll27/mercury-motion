<script setup lang="ts">
import { ref, onMounted, watch, nextTick } from 'vue'
import { useSceneStore } from '../stores/scene'

const store = useSceneStore()
const canvasRef = ref<HTMLCanvasElement | null>(null)
const isTauri = ref(false)

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
    nextTick(() => renderBrowserPreview())
  }
}, { deep: false })

function parseHexColor(hex: string): string {
  return hex.startsWith('#') ? hex : `#${hex}`
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

    // Apply transform
    const pos = Array.isArray(layer.transform.position) ? layer.transform.position : [0, 0]
    const scale = layer.transform.scale || [1, 1]
    const rotation = layer.transform.rotation || 0
    const opacity = layer.transform.opacity ?? 1

    ctx.globalAlpha = typeof opacity === 'number' ? opacity : 1

    ctx.translate(pos[0], pos[1])
    ctx.rotate((rotation * Math.PI) / 180)
    ctx.scale(scale[0], scale[1])

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
      case 'null':
        // Invisible — transform only
        break
    }

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
    </div>
  </div>
</template>
