<script setup lang="ts">
import { ref, computed } from 'vue'
import { useSceneStore } from '../stores/scene'

const store = useSceneStore()
const timelineRef = ref<HTMLDivElement | null>(null)
const isScrubbing = ref(false)
const isDragOver = ref(false)
const dragLayerId = ref<string | null>(null)
const dragStartX = ref(0)
const dragOrigIn = ref(0)
const dragOrigOut = ref(0)

const scrubberPosition = computed(() => {
  if (store.totalFrames <= 0) return 0
  return (store.currentFrame / store.totalFrames) * 100
})

// --- Scrubber ---
function handleScrubStart(e: MouseEvent) {
  // Only scrub if clicking the background, not a layer bar
  isScrubbing.value = true
  scrubToPosition(e)
  window.addEventListener('mousemove', handleScrubMove)
  window.addEventListener('mouseup', handleScrubEnd)
}

function scrubToPosition(e: MouseEvent) {
  if (!timelineRef.value) return
  const rect = timelineRef.value.getBoundingClientRect()
  const x = Math.max(0, Math.min(1, (e.clientX - rect.left) / rect.width))
  store.setFrame(Math.round(x * store.totalFrames))
}

function handleScrubMove(e: MouseEvent) {
  if (!isScrubbing.value) return
  scrubToPosition(e)
}

function handleScrubEnd() {
  isScrubbing.value = false
  window.removeEventListener('mousemove', handleScrubMove)
  window.removeEventListener('mouseup', handleScrubEnd)
}

// --- Layer bar drag (move timing) ---
function handleLayerDragStart(e: MouseEvent, layer: any) {
  e.stopPropagation() // don't trigger scrubber
  dragLayerId.value = layer.id
  dragStartX.value = e.clientX
  dragOrigIn.value = layer.in
  dragOrigOut.value = layer.out
  window.addEventListener('mousemove', handleLayerDragMove)
  window.addEventListener('mouseup', handleLayerDragEnd)
}

function handleLayerDragMove(e: MouseEvent) {
  if (!dragLayerId.value || !timelineRef.value) return
  const rect = timelineRef.value.getBoundingClientRect()
  const deltaX = e.clientX - dragStartX.value
  const deltaFrames = Math.round((deltaX / rect.width) * store.totalFrames)
  const duration = dragOrigOut.value - dragOrigIn.value
  let newIn = dragOrigIn.value + deltaFrames
  newIn = Math.max(0, Math.min(newIn, store.totalFrames - duration))
  const newOut = newIn + duration
  store.updateLayerProperty(dragLayerId.value, 'in', newIn)
  store.updateLayerProperty(dragLayerId.value, 'out', newOut)
}

function handleLayerDragEnd() {
  dragLayerId.value = null
  window.removeEventListener('mousemove', handleLayerDragMove)
  window.removeEventListener('mouseup', handleLayerDragEnd)
}

// --- Drop media from media browser ---
function handleTimelineDrop(e: DragEvent) {
  e.preventDefault()
  isDragOver.value = false

  const mediaData = e.dataTransfer?.getData('application/mmot-media')
  if (!mediaData) return

  const asset = JSON.parse(mediaData)
  const id = `${asset.type}_${Date.now().toString(36)}`
  const w = store.scene.meta.width
  const h = store.scene.meta.height

  // Calculate drop frame position
  let dropFrame = 0
  if (timelineRef.value) {
    const rect = timelineRef.value.getBoundingClientRect()
    const x = Math.max(0, Math.min(1, (e.clientX - rect.left) / rect.width))
    dropFrame = Math.round(x * store.totalFrames)
  }

  const layer: any = {
    id, type: asset.type,
    in: dropFrame,
    out: Math.min(dropFrame + store.scene.meta.fps * 3, store.totalFrames), // 3 seconds default
    transform: { position: [w / 2, h / 2] },
    src: asset.path,
  }
  if (asset.type === 'video') layer.trim_start = 0
  if (asset.type === 'audio') layer.volume = 1.0

  store.addLayer(layer)
  store.selectLayer(id)
}

function handleTimelineDragOver(e: DragEvent) {
  e.preventDefault()
  if (e.dataTransfer) e.dataTransfer.dropEffect = 'copy'
  isDragOver.value = true
}

function handleTimelineDragLeave() {
  isDragOver.value = false
}

// --- Drop files from OS ---
function handleFileDrop(e: DragEvent) {
  if (e.dataTransfer?.getData('application/mmot-media')) return // handled above
  if (!e.dataTransfer?.files.length) return
  e.preventDefault()
  isDragOver.value = false

  for (const file of Array.from(e.dataTransfer.files)) {
    const ext = file.name.split('.').pop()?.toLowerCase() || ''
    const validExts = ['png', 'jpg', 'jpeg', 'webp', 'mp4', 'webm', 'mp3', 'wav', 'flac']
    if (!validExts.includes(ext)) continue
    const type = ['mp4', 'webm'].includes(ext) ? 'video' : ['mp3', 'wav', 'flac'].includes(ext) ? 'audio' : 'image'
    const id = `${type}_${Date.now().toString(36)}`
    const w = store.scene.meta.width
    const h = store.scene.meta.height
    const layer: any = {
      id, type, in: 0, out: store.totalFrames,
      transform: { position: [w / 2, h / 2] },
      src: (file as any).path || URL.createObjectURL(file),
    }
    if (type === 'video') layer.trim_start = 0
    if (type === 'audio') layer.volume = 1.0
    store.addLayer(layer)
    store.selectLayer(id)
  }
}

// --- Tick marks ---
const ticks = computed(() => {
  const marks: { frame: number; position: number; label: string }[] = []
  const fps = store.scene.meta.fps
  const total = store.totalFrames
  const step = fps
  for (let i = 0; i <= total; i += step) {
    marks.push({ frame: i, position: (i / total) * 100, label: `${Math.floor(i / fps)}s` })
  }
  return marks
})
</script>

<template>
  <div class="flex flex-col bg-cosmos-card overflow-hidden">
    <!-- Playback Controls -->
    <div class="h-10 flex items-center gap-3 px-4 border-b border-cosmos-border">
      <button
        class="font-mono text-xs text-text-muted uppercase tracking-widest hover:text-crimson transition-colors"
        @click="store.setFrame(0)"
      >
        &laquo;
      </button>
      <button
        class="w-8 h-8 flex items-center justify-center rounded-full border border-cosmos-border hover:border-crimson hover:text-crimson transition-colors"
        @click="store.togglePlayback()"
      >
        <span class="text-sm">{{ store.isPlaying ? '⏸' : '▶' }}</span>
      </button>
      <button
        class="font-mono text-xs text-text-muted uppercase tracking-widest hover:text-crimson transition-colors"
        @click="store.setFrame(store.totalFrames - 1)"
      >
        &raquo;
      </button>

      <div class="flex-1" />

      <div class="font-mono text-2xl tracking-widest text-text-primary tabular-nums">
        {{ store.currentTimecode }}
      </div>
    </div>

    <!-- Timeline Ruler + Layers -->
    <div class="flex-1 relative px-4 py-2">
      <!-- Tick Marks -->
      <div class="h-6 relative mb-1">
        <template v-for="tick in ticks" :key="tick.frame">
          <div class="absolute top-0 h-full flex flex-col items-center" :style="{ left: `${tick.position}%` }">
            <div class="w-px h-3 bg-cosmos-border"></div>
            <span class="font-mono text-[9px] text-text-muted mt-0.5">{{ tick.label }}</span>
          </div>
        </template>
      </div>

      <!-- Scrub / Drop Area -->
      <div
        ref="timelineRef"
        class="relative bg-cosmos-deep rounded transition-colors"
        :class="isDragOver ? 'ring-1 ring-crimson bg-crimson/5' : ''"
        :style="{ minHeight: Math.max(32, store.layers.length * 28 + 8) + 'px' }"
        @mousedown="handleScrubStart"
        @drop="handleTimelineDrop($event); handleFileDrop($event)"
        @dragover="handleTimelineDragOver"
        @dragleave="handleTimelineDragLeave"
      >
        <!-- Layer Bars (stacked vertically) -->
        <div
          v-for="(layer, idx) in store.layers"
          :key="layer.id"
          class="absolute h-5 rounded-sm cursor-grab active:cursor-grabbing transition-colors group"
          :class="[
            layer.id === store.selectedLayerId ? 'bg-crimson/80' : 'bg-marble/40 hover:bg-marble/60',
            dragLayerId === layer.id ? 'opacity-70' : ''
          ]"
          :style="{
            left: `${(layer.in / store.totalFrames) * 100}%`,
            width: `${((layer.out - layer.in) / store.totalFrames) * 100}%`,
            top: `${4 + idx * 28}px`,
          }"
          @click.stop="store.selectLayer(layer.id)"
          @mousedown.stop="handleLayerDragStart($event, layer)"
        >
          <span class="font-mono text-[9px] text-text-primary truncate px-1.5 leading-5 select-none">
            {{ layer.id }}
          </span>
        </div>

        <!-- Drop hint -->
        <div v-if="isDragOver && store.layers.length === 0" class="absolute inset-0 flex items-center justify-center">
          <span class="font-mono text-xs text-crimson uppercase tracking-widest">Drop media here</span>
        </div>

        <!-- Playhead -->
        <div
          class="absolute top-0 w-0.5 h-full bg-crimson z-10 pointer-events-none"
          :style="{ left: `${scrubberPosition}%` }"
        >
          <div class="absolute -top-1 -left-1.5 w-3.5 h-3 bg-crimson rounded-sm"></div>
        </div>
      </div>
    </div>
  </div>
</template>
