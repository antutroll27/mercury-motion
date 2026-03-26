<script setup lang="ts">
import { ref, computed, watch, onMounted, onBeforeUnmount } from 'vue'
import { useSceneStore } from '../stores/scene'

const store = useSceneStore()
const overlayRef = ref<HTMLCanvasElement | null>(null)

// Interaction state
type DragMode = 'none' | 'move' | 'scale-tl' | 'scale-tr' | 'scale-bl' | 'scale-br' | 'scale-t' | 'scale-b' | 'scale-l' | 'scale-r' | 'rotate'

const dragMode = ref<DragMode>('none')
const dragStartX = ref(0)
const dragStartY = ref(0)
const dragOrigPos = ref<[number, number]>([0, 0])
const dragOrigScale = ref<[number, number]>([1, 1])
const dragOrigRotation = ref(0)
const undoPushed = ref(false)

// Canvas-to-scene coordinate conversion
const canvasRect = ref<DOMRect | null>(null)

function sceneToCanvas(sx: number, sy: number): [number, number] {
  if (!overlayRef.value || !canvasRect.value) return [0, 0]
  const cw = overlayRef.value.width
  const ch = overlayRef.value.height
  const sw = store.scene.meta.width
  const sh = store.scene.meta.height
  return [(sx / sw) * cw, (sy / sh) * ch]
}

const layer = computed(() => store.selectedLayer)

const layerBounds = computed(() => {
  if (!layer.value) return null
  const pos = Array.isArray(layer.value.transform.position)
    ? layer.value.transform.position
    : [0, 0]
  const scale = layer.value.transform.scale || [1, 1]

  let w = 200
  let h = 200
  const content = layer.value as any

  if (content.shape) {
    w = content.shape.width || 200
    h = content.shape.height || 200
  } else if (content.font) {
    w = (content.font.size || 48) * (content.text?.length || 5) * 0.6
    h = (content.font.size || 48) * 1.2
  } else if (content.fill === 'parent') {
    w = store.scene.meta.width
    h = store.scene.meta.height
    return { x: 0, y: 0, w, h, cx: w / 2, cy: h / 2, scale }
  }

  return {
    x: pos[0] - (w * scale[0]) / 2,
    y: pos[1] - (h * scale[1]) / 2,
    w: w * scale[0],
    h: h * scale[1],
    cx: pos[0],
    cy: pos[1],
    scale,
  }
})

const HANDLE_SIZE = 8
const HANDLE_COLOR = '#c1121f'
const HANDLE_FILL = '#0a0a1a'
const SELECTION_COLOR = '#c1121f'

function syncCanvasSize() {
  const canvas = overlayRef.value
  if (!canvas) return
  const parent = canvas.parentElement
  if (parent) {
    const rect = parent.getBoundingClientRect()
    canvas.width = rect.width
    canvas.height = rect.height
    canvasRect.value = rect
  }
}

function drawGizmos() {
  const canvas = overlayRef.value
  if (!canvas) return
  const ctx = canvas.getContext('2d')
  if (!ctx) return

  syncCanvasSize()
  ctx.clearRect(0, 0, canvas.width, canvas.height)

  if (!layerBounds.value || !layer.value) return
  const b = layerBounds.value

  // Convert to canvas coordinates
  const [x1, y1] = sceneToCanvas(b.x, b.y)
  const [x2, y2] = sceneToCanvas(b.x + b.w, b.y + b.h)
  const w = x2 - x1
  const h = y2 - y1

  // Selection rectangle (dashed)
  ctx.strokeStyle = SELECTION_COLOR
  ctx.lineWidth = 1.5
  ctx.setLineDash([4, 4])
  ctx.strokeRect(x1, y1, w, h)
  ctx.setLineDash([])

  // 8 resize handles: 4 corners + 4 edges
  const handles = [
    { id: 'scale-tl', x: x1, y: y1 },
    { id: 'scale-tr', x: x2, y: y1 },
    { id: 'scale-bl', x: x1, y: y2 },
    { id: 'scale-br', x: x2, y: y2 },
    { id: 'scale-t', x: (x1 + x2) / 2, y: y1 },
    { id: 'scale-b', x: (x1 + x2) / 2, y: y2 },
    { id: 'scale-l', x: x1, y: (y1 + y2) / 2 },
    { id: 'scale-r', x: x2, y: (y1 + y2) / 2 },
  ]

  for (const handle of handles) {
    ctx.fillStyle = HANDLE_FILL
    ctx.strokeStyle = HANDLE_COLOR
    ctx.lineWidth = 2
    ctx.fillRect(
      handle.x - HANDLE_SIZE / 2,
      handle.y - HANDLE_SIZE / 2,
      HANDLE_SIZE,
      HANDLE_SIZE,
    )
    ctx.strokeRect(
      handle.x - HANDLE_SIZE / 2,
      handle.y - HANDLE_SIZE / 2,
      HANDLE_SIZE,
      HANDLE_SIZE,
    )
  }

  // Center crosshair
  const [cx, cy] = sceneToCanvas(b.cx, b.cy)
  ctx.strokeStyle = SELECTION_COLOR
  ctx.lineWidth = 1
  ctx.beginPath()
  ctx.moveTo(cx - 8, cy)
  ctx.lineTo(cx + 8, cy)
  ctx.moveTo(cx, cy - 8)
  ctx.lineTo(cx, cy + 8)
  ctx.stroke()
}

function getHandleAt(mx: number, my: number): string | null {
  if (!layerBounds.value) return null
  const b = layerBounds.value
  const [x1, y1] = sceneToCanvas(b.x, b.y)
  const [x2, y2] = sceneToCanvas(b.x + b.w, b.y + b.h)

  const handles = [
    { id: 'scale-tl', x: x1, y: y1 },
    { id: 'scale-tr', x: x2, y: y1 },
    { id: 'scale-bl', x: x1, y: y2 },
    { id: 'scale-br', x: x2, y: y2 },
    { id: 'scale-t', x: (x1 + x2) / 2, y: y1 },
    { id: 'scale-b', x: (x1 + x2) / 2, y: y2 },
    { id: 'scale-l', x: x1, y: (y1 + y2) / 2 },
    { id: 'scale-r', x: x2, y: (y1 + y2) / 2 },
  ]

  for (const handle of handles) {
    if (Math.abs(mx - handle.x) < HANDLE_SIZE && Math.abs(my - handle.y) < HANDLE_SIZE) {
      return handle.id
    }
  }

  // Check if inside bounds (move)
  if (mx >= x1 && mx <= x2 && my >= y1 && my <= y2) return 'move'

  return null
}

function handleMouseDown(e: MouseEvent) {
  if (!overlayRef.value || !layer.value) return
  const rect = overlayRef.value.getBoundingClientRect()
  const mx = e.clientX - rect.left
  const my = e.clientY - rect.top

  const handle = getHandleAt(mx, my)
  if (!handle) return

  e.preventDefault()
  e.stopPropagation()

  dragMode.value = handle as DragMode
  dragStartX.value = e.clientX
  dragStartY.value = e.clientY
  undoPushed.value = false

  const pos = Array.isArray(layer.value.transform.position)
    ? layer.value.transform.position
    : [0, 0]
  dragOrigPos.value = [pos[0], pos[1]]
  dragOrigScale.value = [...(layer.value.transform.scale || [1, 1])] as [number, number]
  dragOrigRotation.value = layer.value.transform.rotation || 0

  // Clean up stale listeners before adding
  window.removeEventListener('mousemove', handleMouseMove)
  window.removeEventListener('mouseup', handleMouseUp)
  window.addEventListener('mousemove', handleMouseMove)
  window.addEventListener('mouseup', handleMouseUp)
}

/**
 * Set a layer property without pushing undo for every mousemove.
 * Undo is pushed once on the first change in a drag gesture.
 */
function setLayerProp(path: string, value: any) {
  const l = layer.value
  if (!l) return
  if (!undoPushed.value) {
    undoPushed.value = true
    // Push undo once at the start of the drag via the store
    store.updateLayerProperty(l.id, path, value)
    return
  }
  // Subsequent updates: set directly without pushing undo
  const keys = path.split('.')
  let obj: any = l
  for (let i = 0; i < keys.length - 1; i++) {
    obj = obj[keys[i]]
    if (obj == null || typeof obj !== 'object') return
  }
  obj[keys[keys.length - 1]] = value
}

function handleMouseMove(e: MouseEvent) {
  if (!layer.value || dragMode.value === 'none') return

  const sw = store.scene.meta.width || 1
  const sh = store.scene.meta.height || 1
  const cw = overlayRef.value?.width || sw || 1
  const ch = overlayRef.value?.height || sh || 1

  const dx = (e.clientX - dragStartX.value) * (sw / cw)
  const dy = (e.clientY - dragStartY.value) * (sh / ch)

  if (dragMode.value === 'move') {
    const newX = Math.round(dragOrigPos.value[0] + dx)
    const newY = Math.round(dragOrigPos.value[1] + dy)
    setLayerProp('transform.position', [newX, newY])
  } else if (dragMode.value.startsWith('scale-')) {
    const origW = (layer.value as any).shape?.width || 200
    const origH = (layer.value as any).shape?.height || 200
    let sx = dragOrigScale.value[0]
    let sy = dragOrigScale.value[1]

    const mode = dragMode.value

    // Horizontal scaling
    if (mode === 'scale-r' || mode === 'scale-tr' || mode === 'scale-br') {
      sx = dragOrigScale.value[0] + (dx / origW) * 2
    } else if (mode === 'scale-l' || mode === 'scale-tl' || mode === 'scale-bl') {
      sx = dragOrigScale.value[0] - (dx / origW) * 2
    }

    // Vertical scaling
    if (mode === 'scale-b' || mode === 'scale-bl' || mode === 'scale-br') {
      sy = dragOrigScale.value[1] + (dy / origH) * 2
    } else if (mode === 'scale-t' || mode === 'scale-tl' || mode === 'scale-tr') {
      sy = dragOrigScale.value[1] - (dy / origH) * 2
    }

    // Shift key: proportional scaling on corner handles
    if (['scale-tl', 'scale-tr', 'scale-bl', 'scale-br'].includes(mode)) {
      if (e.shiftKey) {
        const avg = (sx + sy) / 2
        sx = avg
        sy = avg
      }
    }

    sx = Math.max(0.01, sx)
    sy = Math.max(0.01, sy)
    setLayerProp('transform.scale', [
      Math.round(sx * 100) / 100,
      Math.round(sy * 100) / 100,
    ])
  }

  drawGizmos()
}

function handleMouseUp() {
  dragMode.value = 'none'
  window.removeEventListener('mousemove', handleMouseMove)
  window.removeEventListener('mouseup', handleMouseUp)
  // Trigger a preview refresh after the drag completes
  store.schedulePreview()
}

// Update cursor based on handle hover
function handleOverlayMouseMove(e: MouseEvent) {
  if (!overlayRef.value || dragMode.value !== 'none') return
  const rect = overlayRef.value.getBoundingClientRect()
  const mx = e.clientX - rect.left
  const my = e.clientY - rect.top

  const handle = getHandleAt(mx, my)
  const cursors: Record<string, string> = {
    move: 'move',
    'scale-tl': 'nwse-resize',
    'scale-br': 'nwse-resize',
    'scale-tr': 'nesw-resize',
    'scale-bl': 'nesw-resize',
    'scale-t': 'ns-resize',
    'scale-b': 'ns-resize',
    'scale-l': 'ew-resize',
    'scale-r': 'ew-resize',
  }
  overlayRef.value.style.cursor = handle ? cursors[handle] || 'default' : 'default'
}

// Redraw on selection or property changes
watch([layer, () => store.currentFrame], () => drawGizmos(), { deep: true })

onMounted(() => {
  drawGizmos()
  window.addEventListener('resize', drawGizmos)
})

onBeforeUnmount(() => {
  window.removeEventListener('resize', drawGizmos)
  window.removeEventListener('mousemove', handleMouseMove)
  window.removeEventListener('mouseup', handleMouseUp)
})
</script>

<template>
  <canvas
    ref="overlayRef"
    class="absolute inset-0 z-10"
    @mousedown="handleMouseDown"
    @mousemove="handleOverlayMouseMove"
  />
</template>
