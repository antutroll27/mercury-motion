<script setup lang="ts">
import { ref, nextTick } from 'vue'

const emit = defineEmits<{
  select: [easing: string]
}>()

const show = ref(false)

interface EasingPreset {
  id: string
  label: string
  // For linear: 2 points [start, end]
  // For cubic bezier: 4 points [start, cp1, cp2, end]
  points: [number, number][]
}

const presets: EasingPreset[] = [
  { id: 'linear', label: 'Linear', points: [[0, 0], [1, 1]] },
  { id: 'ease_in', label: 'Ease In', points: [[0, 0], [0.42, 0], [1, 1], [1, 1]] },
  { id: 'ease_out', label: 'Ease Out', points: [[0, 0], [0, 0], [0.58, 1], [1, 1]] },
  { id: 'ease_in_out', label: 'Ease In Out', points: [[0, 0], [0.42, 0], [0.58, 1], [1, 1]] },
  { id: 'ease_in_cubic', label: 'In Cubic', points: [[0, 0], [0.55, 0.055], [0.675, 0.19], [1, 1]] },
  { id: 'ease_out_cubic', label: 'Out Cubic', points: [[0, 0], [0.215, 0.61], [0.355, 1], [1, 1]] },
  { id: 'ease_in_back', label: 'In Back', points: [[0, 0], [0.6, -0.28], [0.735, 0.045], [1, 1]] },
  { id: 'ease_out_back', label: 'Out Back', points: [[0, 0], [0.175, 0.885], [0.32, 1.275], [1, 1]] },
]

function drawCurve(el: HTMLCanvasElement | null, preset: EasingPreset) {
  if (!el) return
  const ctx = el.getContext('2d')
  if (!ctx) return
  const w = el.width
  const h = el.height
  const pad = 4

  ctx.clearRect(0, 0, w, h)

  // Diagonal reference line
  ctx.strokeStyle = '#0A4060'
  ctx.lineWidth = 0.5
  ctx.beginPath()
  ctx.moveTo(pad, h - pad)
  ctx.lineTo(w - pad, pad)
  ctx.stroke()

  // Curve
  ctx.strokeStyle = '#C1121F'
  ctx.lineWidth = 1.5
  ctx.beginPath()

  const sx = (v: number) => pad + v * (w - pad * 2)
  const sy = (v: number) => (h - pad) - v * (h - pad * 2)

  if (preset.points.length === 2) {
    // Linear
    ctx.moveTo(sx(0), sy(0))
    ctx.lineTo(sx(1), sy(1))
  } else {
    // Cubic bezier
    const [, cp1, cp2] = preset.points
    ctx.moveTo(sx(0), sy(0))
    ctx.bezierCurveTo(
      sx(cp1[0]), sy(cp1[1]),
      sx(cp2[0]), sy(cp2[1]),
      sx(1), sy(1)
    )
  }
  ctx.stroke()
}

function selectEasing(id: string) {
  emit('select', id)
  show.value = false
}

function open() {
  show.value = true
  nextTick(() => {
    // Redraw canvases after DOM update
    const canvases = document.querySelectorAll('.easing-picker-canvas') as NodeListOf<HTMLCanvasElement>
    canvases.forEach((canvas, idx) => {
      if (presets[idx]) drawCurve(canvas, presets[idx])
    })
  })
}

function close() {
  show.value = false
}

defineExpose({ show, open, close })
</script>

<template>
  <div v-if="show" class="absolute z-50 bg-cosmos-card border border-cosmos-border rounded shadow-xl p-2 w-52">
    <div class="font-mono text-[9px] text-text-muted uppercase tracking-wider mb-2">Easing</div>
    <div class="grid grid-cols-2 gap-1.5">
      <button
        v-for="preset in presets"
        :key="preset.id"
        class="flex flex-col items-center gap-1 p-1.5 rounded border border-cosmos-border hover:border-crimson transition-colors group"
        @click="selectEasing(preset.id)"
      >
        <canvas
          class="easing-picker-canvas rounded"
          width="44"
          height="32"
          :ref="(el) => { if (el) drawCurve(el as HTMLCanvasElement, preset) }"
        />
        <span class="font-mono text-[8px] text-text-muted group-hover:text-crimson transition-colors">{{ preset.label }}</span>
      </button>
    </div>
  </div>
</template>
