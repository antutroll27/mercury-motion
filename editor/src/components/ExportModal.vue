<script setup lang="ts">
import { ref, computed } from 'vue'
import { useSceneStore } from '../stores/scene'

const emit = defineEmits<{ close: [] }>()
const store = useSceneStore()

type FormatId =
  | 'mp4' | 'webm' | 'gif' | 'mov'
  | 'lottie' | 'svg-anim' | 'css-anim' | 'rive'
  | 'mmot' | 'schema'
  | 'png' | 'svg'

interface FormatOption {
  id: FormatId
  name: string
  ext: string
  icon: string
  description: string
  available: boolean
  hint?: string
}

const videoFormats: FormatOption[] = [
  { id: 'mp4', name: 'MP4', ext: '.mp4', icon: '\u25B6', description: 'H.264/AV1 — web & social', available: false, hint: 'Requires CLI: mmot render' },
  { id: 'webm', name: 'WebM', ext: '.webm', icon: '\u25B6', description: 'VP9/AV1 — open, smaller', available: false, hint: 'Requires CLI: mmot render' },
  { id: 'gif', name: 'GIF', ext: '.gif', icon: '\u25B6', description: 'Animated, universal', available: false, hint: 'Requires CLI: mmot render' },
  { id: 'mov', name: 'MOV', ext: '.mov', icon: '\u25B6', description: 'ProRes — lossless quality', available: false, hint: 'Requires CLI: mmot render' },
]

const animationFormats: FormatOption[] = [
  { id: 'lottie', name: 'Lottie', ext: '.json', icon: '\u2728', description: 'Web/mobile animations', available: false, hint: 'Coming soon' },
  { id: 'svg-anim', name: 'SVG', ext: '.svg', icon: '\u2728', description: 'SMIL vector animation', available: false, hint: 'Coming soon' },
  { id: 'css-anim', name: 'CSS', ext: '.css', icon: '\u2728', description: 'Pure CSS keyframes', available: false, hint: 'Coming soon' },
  { id: 'rive', name: 'Rive', ext: '.riv', icon: '\u2728', description: 'Interactive runtime', available: false, hint: 'Coming soon' },
]

const projectFormats: FormatOption[] = [
  { id: 'mmot', name: '.mmot', ext: '.mmot.json', icon: '\u2B22', description: 'Native editable format', available: true },
  { id: 'schema', name: 'Schema', ext: '.schema.json', icon: '\u2B22', description: 'JSON Schema for validation', available: true },
]

const imageFormats: FormatOption[] = [
  { id: 'png', name: 'PNG', ext: '.png', icon: '\u25A3', description: 'Current frame as image', available: true },
  { id: 'svg', name: 'SVG', ext: '.svg', icon: '\u25A3', description: 'Current frame as vector', available: false, hint: 'Coming soon' },
]

const selectedFormat = ref<FormatId>('mp4')
const quality = ref(80)
const resWidth = ref(store.scene.meta.width)
const resHeight = ref(store.scene.meta.height)
const isExporting = ref(false)

const selectedOption = computed(() => {
  const all = [...videoFormats, ...animationFormats, ...projectFormats, ...imageFormats]
  return all.find(f => f.id === selectedFormat.value)
})

function selectFormat(id: FormatId) {
  selectedFormat.value = id
}

function closeOnBackdrop(e: MouseEvent) {
  if (e.target === e.currentTarget) {
    emit('close')
  }
}

function exportMmotJson() {
  const json = store.toJson()
  const blob = new Blob([json], { type: 'application/json' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `${store.scene.meta.name || 'scene'}.mmot.json`
  document.body.appendChild(a)
  a.click()
  document.body.removeChild(a)
  URL.revokeObjectURL(url)
}

function exportSchema() {
  const schema = {
    $schema: 'https://json-schema.org/draft/2020-12/schema',
    title: 'Mercury-Motion Scene',
    description: `Schema for ${store.scene.meta.name || 'scene'}`,
    type: 'object',
    properties: {
      version: { type: 'string' },
      meta: {
        type: 'object',
        properties: {
          name: { type: 'string' },
          width: { type: 'number' },
          height: { type: 'number' },
          fps: { type: 'number' },
          duration: { type: 'number' },
          root: { type: 'string' },
          background: { type: 'string' },
        },
        required: ['name', 'width', 'height', 'fps', 'duration', 'root'],
      },
      compositions: { type: 'object' },
      assets: { type: 'object' },
    },
    required: ['version', 'meta', 'compositions'],
  }
  const json = JSON.stringify(schema, null, 2)
  const blob = new Blob([json], { type: 'application/json' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `${store.scene.meta.name || 'scene'}.schema.json`
  document.body.appendChild(a)
  a.click()
  document.body.removeChild(a)
  URL.revokeObjectURL(url)
}

function exportPng() {
  const canvas = document.querySelector('canvas') as HTMLCanvasElement
  if (!canvas) {
    alert('No canvas found. Make sure the preview is visible.')
    return
  }
  const url = canvas.toDataURL('image/png')
  const a = document.createElement('a')
  a.href = url
  a.download = `${store.scene.meta.name || 'frame'}-${store.currentFrame}.png`
  document.body.appendChild(a)
  a.click()
  document.body.removeChild(a)
}

async function handleExport() {
  const fmt = selectedFormat.value
  isExporting.value = true

  try {
    if (fmt === 'mmot') {
      exportMmotJson()
    } else if (fmt === 'schema') {
      exportSchema()
    } else if (fmt === 'png') {
      exportPng()
    } else {
      // Unavailable format — should not reach here due to button disable
      return
    }
    emit('close')
  } catch (e) {
    console.error('Export failed:', e)
    alert(`Export failed: ${e}`)
  } finally {
    isExporting.value = false
  }
}
</script>

<template>
  <Teleport to="body">
    <div
      class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm"
      @mousedown="closeOnBackdrop"
    >
      <div
        class="w-[560px] max-h-[85vh] bg-cosmos-card border border-cosmos-border rounded-lg shadow-2xl flex flex-col overflow-hidden"
        @mousedown.stop
      >
        <!-- Header -->
        <div class="flex items-center justify-between px-6 py-4 border-b border-cosmos-border">
          <h2 class="font-serif text-lg tracking-wide text-varden">Export</h2>
          <button
            class="w-7 h-7 flex items-center justify-center text-text-muted hover:text-crimson transition-colors rounded hover:bg-cosmos-deep"
            @click="emit('close')"
          >
            <span class="text-lg leading-none">&times;</span>
          </button>
        </div>

        <!-- Body -->
        <div class="flex-1 overflow-y-auto px-6 py-5 space-y-5">

          <!-- Video Formats -->
          <div>
            <h3 class="font-mono text-[10px] uppercase tracking-[0.2em] text-text-muted mb-2">Video</h3>
            <div class="grid grid-cols-4 gap-2">
              <button
                v-for="fmt in videoFormats"
                :key="fmt.id"
                class="group relative flex flex-col items-center gap-1.5 px-3 py-3 rounded border transition-all text-center"
                :class="selectedFormat === fmt.id
                  ? 'border-crimson bg-crimson/10 text-varden'
                  : 'border-cosmos-border bg-cosmos-deep text-text-secondary hover:border-marble/40'"
                @click="selectFormat(fmt.id)"
              >
                <span class="text-base leading-none opacity-60">{{ fmt.icon }}</span>
                <span class="font-mono text-xs font-semibold tracking-wide">{{ fmt.name }}</span>
                <span class="text-[9px] text-text-muted leading-tight">{{ fmt.description }}</span>
                <span
                  v-if="!fmt.available"
                  class="absolute top-1 right-1 px-1 py-0.5 bg-cosmos-deep border border-cosmos-border rounded text-[7px] font-mono uppercase text-text-muted"
                >CLI</span>
              </button>
            </div>
          </div>

          <!-- Animation Formats -->
          <div>
            <h3 class="font-mono text-[10px] uppercase tracking-[0.2em] text-text-muted mb-2">Animation</h3>
            <div class="grid grid-cols-4 gap-2">
              <button
                v-for="fmt in animationFormats"
                :key="fmt.id"
                class="group relative flex flex-col items-center gap-1.5 px-3 py-3 rounded border transition-all text-center"
                :class="selectedFormat === fmt.id
                  ? 'border-crimson bg-crimson/10 text-varden'
                  : 'border-cosmos-border bg-cosmos-deep text-text-secondary hover:border-marble/40'"
                @click="selectFormat(fmt.id)"
              >
                <span class="text-base leading-none opacity-60">{{ fmt.icon }}</span>
                <span class="font-mono text-xs font-semibold tracking-wide">{{ fmt.name }}</span>
                <span class="text-[9px] text-text-muted leading-tight">{{ fmt.description }}</span>
                <span
                  v-if="!fmt.available"
                  class="absolute top-1 right-1 px-1 py-0.5 bg-cosmos-deep border border-cosmos-border rounded text-[7px] font-mono uppercase text-text-muted"
                >Soon</span>
              </button>
            </div>
          </div>

          <!-- Project Formats -->
          <div>
            <h3 class="font-mono text-[10px] uppercase tracking-[0.2em] text-text-muted mb-2">Project</h3>
            <div class="grid grid-cols-4 gap-2">
              <button
                v-for="fmt in projectFormats"
                :key="fmt.id"
                class="group flex flex-col items-center gap-1.5 px-3 py-3 rounded border transition-all text-center"
                :class="selectedFormat === fmt.id
                  ? 'border-crimson bg-crimson/10 text-varden'
                  : 'border-cosmos-border bg-cosmos-deep text-text-secondary hover:border-marble/40'"
                @click="selectFormat(fmt.id)"
              >
                <span class="text-base leading-none opacity-60">{{ fmt.icon }}</span>
                <span class="font-mono text-xs font-semibold tracking-wide">{{ fmt.name }}</span>
                <span class="text-[9px] text-text-muted leading-tight">{{ fmt.description }}</span>
              </button>
            </div>
          </div>

          <!-- Image Formats -->
          <div>
            <h3 class="font-mono text-[10px] uppercase tracking-[0.2em] text-text-muted mb-2">Still Image</h3>
            <div class="grid grid-cols-4 gap-2">
              <button
                v-for="fmt in imageFormats"
                :key="fmt.id"
                class="group relative flex flex-col items-center gap-1.5 px-3 py-3 rounded border transition-all text-center"
                :class="selectedFormat === fmt.id
                  ? 'border-crimson bg-crimson/10 text-varden'
                  : 'border-cosmos-border bg-cosmos-deep text-text-secondary hover:border-marble/40'"
                @click="selectFormat(fmt.id)"
              >
                <span class="text-base leading-none opacity-60">{{ fmt.icon }}</span>
                <span class="font-mono text-xs font-semibold tracking-wide">{{ fmt.name }}</span>
                <span class="text-[9px] text-text-muted leading-tight">{{ fmt.description }}</span>
                <span
                  v-if="!fmt.available"
                  class="absolute top-1 right-1 px-1 py-0.5 bg-cosmos-deep border border-cosmos-border rounded text-[7px] font-mono uppercase text-text-muted"
                >Soon</span>
              </button>
            </div>
          </div>

          <!-- Divider -->
          <div class="border-t border-cosmos-border"></div>

          <!-- Settings -->
          <div class="space-y-3">
            <!-- Quality Slider (for video/image formats) -->
            <div class="flex items-center gap-4">
              <label class="font-mono text-[10px] uppercase tracking-[0.2em] text-text-muted w-20 shrink-0">Quality</label>
              <input
                type="range"
                min="1"
                max="100"
                v-model.number="quality"
                class="flex-1 h-1 appearance-none bg-cosmos-border rounded-full accent-crimson cursor-pointer [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:w-3 [&::-webkit-slider-thumb]:h-3 [&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:bg-crimson [&::-webkit-slider-thumb]:cursor-pointer"
              />
              <span class="font-mono text-xs text-text-primary w-8 text-right tabular-nums">{{ quality }}</span>
            </div>

            <!-- Resolution -->
            <div class="flex items-center gap-4">
              <label class="font-mono text-[10px] uppercase tracking-[0.2em] text-text-muted w-20 shrink-0">Resolution</label>
              <div class="flex items-center gap-2">
                <input
                  type="number"
                  v-model.number="resWidth"
                  class="w-20 bg-cosmos-deep border border-cosmos-border rounded px-2 py-1 font-mono text-xs text-text-primary focus:border-crimson outline-none text-center tabular-nums"
                />
                <span class="font-mono text-xs text-text-muted">&times;</span>
                <input
                  type="number"
                  v-model.number="resHeight"
                  class="w-20 bg-cosmos-deep border border-cosmos-border rounded px-2 py-1 font-mono text-xs text-text-primary focus:border-crimson outline-none text-center tabular-nums"
                />
              </div>
            </div>
          </div>

          <!-- CLI Instructions (shown for unavailable formats) -->
          <div
            v-if="selectedOption && !selectedOption.available && selectedOption.hint"
            class="bg-cosmos-deep border border-cosmos-border rounded p-4"
          >
            <p class="font-mono text-[10px] uppercase tracking-[0.2em] text-text-muted mb-2">
              {{ selectedOption.hint }}
            </p>
            <template v-if="selectedOption.hint === 'Requires CLI: mmot render'">
              <p class="font-mono text-xs text-text-secondary mb-2">To render {{ selectedOption.name }}, install the CLI and run:</p>
              <pre class="font-mono text-[11px] text-marble bg-black/30 rounded p-3 overflow-x-auto leading-relaxed"><code>cargo install mmot
mmot render scene.mmot.json --output video{{ selectedOption.ext }}</code></pre>
            </template>
            <template v-else>
              <p class="font-mono text-xs text-text-secondary">
                {{ selectedOption.name }} export is under development and will be available in a future release.
              </p>
            </template>
          </div>
        </div>

        <!-- Footer -->
        <div class="px-6 py-4 border-t border-cosmos-border flex justify-end">
          <button
            class="px-6 py-2 bg-crimson text-varden text-xs font-mono uppercase tracking-widest rounded hover:bg-gochujang transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
            :disabled="isExporting || (selectedOption && !selectedOption.available)"
            @click="handleExport"
          >
            {{ isExporting ? 'Exporting...' : 'Export' }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>
