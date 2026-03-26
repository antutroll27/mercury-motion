<script setup lang="ts">
import { ref, computed, onBeforeUnmount } from 'vue'
import { useSceneStore } from '../stores/scene'
import EasingPicker from './EasingPicker.vue'

const store = useSceneStore()
const timelineRef = ref<HTMLDivElement | null>(null)
const isScrubbing = ref(false)
const isDragOver = ref(false)

// View mode toggle: bars (default) or keys (dope sheet)
const viewMode = ref<'bars' | 'keys'>('bars')

// Easing picker state
const easingPickerRef = ref<InstanceType<typeof EasingPicker> | null>(null)
const easingPickerPos = ref({ x: 0, y: 0 })
const easingPickerContext = ref<{ layerId: string; path: string; frame: number } | null>(null)

// Layer drag reorder
const dragFromIdx = ref<number | null>(null)
const dragOverIdx = ref<number | null>(null)

// Layer bar drag (move timing)
const dragLayerId = ref<string | null>(null)
const dragStartX = ref(0)
const dragOrigIn = ref(0)
const dragOrigOut = ref(0)

const TRACK_HEIGHT = 32
const KEYS_HEADER_HEIGHT = 24
const KEYS_PROP_HEIGHT = 20

// --- Dope sheet helpers ---

interface AnimatedProp {
  path: string
  label: string
  keyframes: { t: number; v: any }[] | null
}

function getAnimatedProps(layer: any): AnimatedProp[] {
  const props: AnimatedProp[] = []
  const t = layer.transform

  const checkProp = (val: any, path: string, label: string) => {
    if (Array.isArray(val) && val.length > 0 && typeof val[0] === 'object' && 't' in val[0]) {
      props.push({ path, label, keyframes: val })
    }
  }

  checkProp(t?.position, 'transform.position', 'Position')
  checkProp(t?.scale, 'transform.scale', 'Scale')
  checkProp(t?.rotation, 'transform.rotation', 'Rotation')
  checkProp(t?.opacity, 'transform.opacity', 'Opacity')

  // If no animated props, show placeholders so the dope sheet is not empty
  if (props.length === 0) {
    props.push({ path: 'transform.position', label: 'Position', keyframes: null })
    props.push({ path: 'transform.opacity', label: 'Opacity', keyframes: null })
  }

  return props
}

function getKeysRowTop(layerIdx: number, propIdx: number): number {
  let top = 0
  for (let i = 0; i < layerIdx; i++) {
    const layer = store.layers[i]
    top += KEYS_HEADER_HEIGHT + getAnimatedProps(layer).length * KEYS_PROP_HEIGHT
  }
  if (propIdx < 0) return top // Header row
  return top + KEYS_HEADER_HEIGHT + propIdx * KEYS_PROP_HEIGHT
}

const keysTotalHeight = computed(() => {
  let total = 0
  for (const layer of store.layers) {
    total += KEYS_HEADER_HEIGHT + getAnimatedProps(layer).length * KEYS_PROP_HEIGHT
  }
  return Math.max(TRACK_HEIGHT, total)
})

// --- Easing picker ---

function handleKeyframeContextMenu(e: MouseEvent, layerId: string, path: string, frame: number) {
  e.preventDefault()
  e.stopPropagation()
  easingPickerContext.value = { layerId, path, frame }
  easingPickerPos.value = { x: e.clientX, y: e.clientY }
  easingPickerRef.value?.open()
}

function handleEasingSelect(easing: string) {
  if (!easingPickerContext.value) return
  const { layerId, path, frame } = easingPickerContext.value
  // Find the keyframe and set its easing
  const layer = store.layers.find(l => l.id === layerId)
  if (!layer) return

  const keys = path.split('.')
  let obj: any = layer
  for (const k of keys) {
    obj = obj?.[k]
    if (obj == null) return
  }

  if (Array.isArray(obj)) {
    const kf = obj.find((k: any) => k.t === frame)
    if (kf) {
      kf.easing = easing
      store.schedulePreview()
    }
  }
  easingPickerContext.value = null
}

const scrubberPosition = computed(() => {
  if (store.totalFrames <= 0) return 0
  return (store.currentFrame / store.totalFrames) * 100
})

// --- Scrubber ---
function handleScrubStart(e: MouseEvent) {
  // Clean up any stale listeners before adding new ones
  window.removeEventListener('mousemove', handleScrubMove)
  window.removeEventListener('mouseup', handleScrubEnd)
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
function handleBarDragStart(e: MouseEvent, layer: any) {
  e.stopPropagation()
  // Clean up any stale listeners before adding new ones
  window.removeEventListener('mousemove', handleBarDragMove)
  window.removeEventListener('mouseup', handleBarDragEnd)
  dragLayerId.value = layer.id
  dragStartX.value = e.clientX
  dragOrigIn.value = layer.in
  dragOrigOut.value = layer.out
  window.addEventListener('mousemove', handleBarDragMove)
  window.addEventListener('mouseup', handleBarDragEnd)
}

function handleBarDragMove(e: MouseEvent) {
  if (!dragLayerId.value || !timelineRef.value) return
  const rect = timelineRef.value.getBoundingClientRect()
  const deltaX = e.clientX - dragStartX.value
  const deltaFrames = Math.round((deltaX / rect.width) * store.totalFrames)
  const duration = dragOrigOut.value - dragOrigIn.value
  let newIn = dragOrigIn.value + deltaFrames
  newIn = Math.max(0, Math.min(newIn, store.totalFrames - duration))
  store.updateLayerProperty(dragLayerId.value, 'in', newIn)
  store.updateLayerProperty(dragLayerId.value, 'out', newIn + duration)
}

function handleBarDragEnd() {
  dragLayerId.value = null
  window.removeEventListener('mousemove', handleBarDragMove)
  window.removeEventListener('mouseup', handleBarDragEnd)
}

// --- Track reorder (drag up/down) ---
function handleTrackDragStart(e: DragEvent, idx: number) {
  if (!e.dataTransfer) return
  dragFromIdx.value = idx
  e.dataTransfer.setData('text/plain', String(idx))
  e.dataTransfer.effectAllowed = 'move'
}

function handleTrackDragOver(e: DragEvent, idx: number) {
  e.preventDefault()
  if (e.dataTransfer) e.dataTransfer.dropEffect = 'move'
  dragOverIdx.value = idx
}

function handleTrackDrop(e: DragEvent, toIdx: number) {
  e.preventDefault()
  dragOverIdx.value = null
  if (dragFromIdx.value !== null && dragFromIdx.value !== toIdx) {
    store.reorderLayer(dragFromIdx.value, toIdx)
  }
  dragFromIdx.value = null
}

function handleTrackDragEnd() {
  dragFromIdx.value = null
  dragOverIdx.value = null
}

// --- Drop media from media browser ---
function handleTimelineDrop(e: DragEvent) {
  e.preventDefault()
  isDragOver.value = false
  const mediaData = e.dataTransfer?.getData('application/mmot-media')
  if (!mediaData) return
  let asset: any
  try { asset = JSON.parse(mediaData) } catch { return }
  const id = `${asset.type}_${Date.now().toString(36)}`
  let dropFrame = 0
  if (timelineRef.value) {
    const rect = timelineRef.value.getBoundingClientRect()
    const x = Math.max(0, Math.min(1, (e.clientX - rect.left) / rect.width))
    dropFrame = Math.round(x * store.totalFrames)
  }
  const layer: any = {
    id, type: asset.type,
    in: dropFrame,
    out: Math.min(dropFrame + store.scene.meta.fps * 3, store.totalFrames),
    transform: { position: [store.scene.meta.width / 2, store.scene.meta.height / 2] },
    src: asset.path,
  }
  if (asset.type === 'video') layer.trim_start = 0
  if (asset.type === 'audio') layer.volume = 1.0
  store.addLayer(layer)
  store.selectLayer(id)
}

// --- Tick marks ---
const ticks = computed(() => {
  const marks: { frame: number; position: number; label: string }[] = []
  const fps = store.scene.meta.fps
  const total = store.totalFrames
  if (total <= 0) return marks
  const step = fps
  for (let i = 0; i <= total; i += step) {
    marks.push({ frame: i, position: (i / total) * 100, label: `${Math.floor(i / fps)}s` })
  }
  return marks
})

const layerTypeIcons: Record<string, string> = {
  solid: '■', text: 'T', image: '◻', video: '▶', shape: '△',
  gradient: '◐', null: '◎', audio: '♪', composition: '⊞',
}

const layerColors: Record<string, string> = {
  solid: 'bg-marble/50',
  text: 'bg-purple-500/50',
  image: 'bg-emerald-500/50',
  video: 'bg-blue-500/50',
  audio: 'bg-amber-500/50',
  shape: 'bg-pink-500/50',
  gradient: 'bg-teal-500/50',
  null: 'bg-gray-500/30',
  composition: 'bg-indigo-500/50',
}

onBeforeUnmount(() => {
  window.removeEventListener('mousemove', handleScrubMove)
  window.removeEventListener('mouseup', handleScrubEnd)
  window.removeEventListener('mousemove', handleBarDragMove)
  window.removeEventListener('mouseup', handleBarDragEnd)
})
</script>

<template>
  <div class="flex flex-col bg-cosmos-card overflow-hidden">
    <!-- Playback Controls -->
    <div class="h-10 flex items-center gap-3 px-4 border-b border-cosmos-border">
      <button
        class="font-mono text-xs text-text-muted hover:text-crimson transition-colors"
        @click="store.setFrame(0)"
      >⟨⟨</button>
      <button
        class="w-8 h-8 flex items-center justify-center rounded-full border border-cosmos-border hover:border-crimson hover:text-crimson transition-colors"
        @click="store.togglePlayback()"
      >
        <span class="text-sm">{{ store.isPlaying ? '⏸' : '▶' }}</span>
      </button>
      <button
        class="font-mono text-xs text-text-muted hover:text-crimson transition-colors"
        @click="store.setFrame(store.totalFrames - 1)"
      >⟩⟩</button>

      <div class="flex gap-1 ml-3">
        <button
          class="px-2 py-0.5 font-mono text-[9px] uppercase tracking-wider rounded transition-colors"
          :class="viewMode === 'bars' ? 'bg-crimson/20 text-crimson' : 'text-text-muted hover:text-text-primary'"
          @click="viewMode = 'bars'"
        >Bars</button>
        <button
          class="px-2 py-0.5 font-mono text-[9px] uppercase tracking-wider rounded transition-colors"
          :class="viewMode === 'keys' ? 'bg-crimson/20 text-crimson' : 'text-text-muted hover:text-text-primary'"
          @click="viewMode = 'keys'"
        >Keys</button>
      </div>

      <div class="flex-1" />

      <div class="font-mono text-2xl tracking-widest text-text-primary tabular-nums">
        {{ store.currentTimecode }}
      </div>
    </div>

    <!-- Timeline Body: Track Headers + Track Area -->
    <div class="flex-1 flex overflow-hidden">
      <!-- Track Headers (left sidebar) -->
      <div class="w-44 border-r border-cosmos-border flex-shrink-0 overflow-y-auto">
        <!-- Ruler spacer -->
        <div class="h-6 border-b border-cosmos-border"></div>

        <!-- Track labels: Bars mode -->
        <template v-if="viewMode === 'bars'">
          <div
            v-for="(layer, idx) in store.layers"
            :key="layer.id"
            class="flex items-center gap-1.5 px-2 border-b border-cosmos-border/40 cursor-grab active:cursor-grabbing select-none"
            :style="{ height: TRACK_HEIGHT + 'px' }"
            :class="[
              layer.id === store.selectedLayerId ? 'bg-crimson/10' : 'hover:bg-cosmos-deep/50',
              dragOverIdx === idx ? 'border-t-2 border-t-crimson' : ''
            ]"
            draggable="true"
            @click="store.selectLayer(layer.id)"
            @dragstart="handleTrackDragStart($event, idx)"
            @dragover="handleTrackDragOver($event, idx)"
            @drop="handleTrackDrop($event, idx)"
            @dragend="handleTrackDragEnd"
          >
            <span class="font-mono text-[9px] text-text-muted/40">⋮⋮</span>
            <span class="font-mono text-[10px] w-3 text-center" :class="layer.id === store.selectedLayerId ? 'text-crimson' : 'text-text-muted'">
              {{ layerTypeIcons[layer.type] || '?' }}
            </span>
            <span class="font-mono text-[11px] text-text-primary truncate flex-1">
              {{ layer.id }}
            </span>
          </div>
        </template>

        <!-- Track labels: Keys (dope sheet) mode -->
        <template v-if="viewMode === 'keys'">
          <template v-for="(layer, _layerIdx) in store.layers" :key="'header-keys-' + layer.id">
            <div
              class="flex items-center gap-1.5 px-2 border-b border-cosmos-border/30 cursor-pointer select-none"
              :class="layer.id === store.selectedLayerId ? 'bg-crimson/10 text-crimson' : 'text-text-primary hover:bg-cosmos-deep/50'"
              :style="{ height: KEYS_HEADER_HEIGHT + 'px' }"
              @click="store.selectLayer(layer.id)"
            >
              <span class="font-mono text-[10px] w-3 text-center" :class="layer.id === store.selectedLayerId ? 'text-crimson' : 'text-text-muted'">
                {{ layerTypeIcons[layer.type] || '?' }}
              </span>
              <span class="font-mono text-[10px] truncate flex-1">{{ layer.id }}</span>
            </div>
            <div
              v-for="propInfo in getAnimatedProps(layer)"
              :key="'header-prop-' + layer.id + '-' + propInfo.path"
              class="flex items-center px-4 border-b border-cosmos-border/10 font-mono text-[9px] text-text-muted"
              :style="{ height: KEYS_PROP_HEIGHT + 'px' }"
            >
              {{ propInfo.label }}
            </div>
          </template>
        </template>

        <!-- Empty state -->
        <div v-if="store.layers.length === 0" class="flex items-center justify-center py-4">
          <span class="font-mono text-[9px] text-text-muted uppercase">No layers</span>
        </div>
      </div>

      <!-- Track Area (right, scrollable) -->
      <div class="flex-1 overflow-x-auto overflow-y-auto relative">
        <!-- Ruler (click/drag to scrub) -->
        <div
          ref="timelineRef"
          class="h-8 border-b border-cosmos-border sticky top-0 bg-cosmos-card z-20 cursor-pointer select-none"
          @mousedown="handleScrubStart"
        >
          <template v-for="tick in ticks" :key="tick.frame">
            <div class="absolute top-0 h-full flex flex-col items-center pointer-events-none" :style="{ left: `${tick.position}%` }">
              <div class="w-px h-3 bg-cosmos-border"></div>
              <span class="font-mono text-[9px] text-text-muted mt-0.5">{{ tick.label }}</span>
            </div>
          </template>

          <!-- Playhead on ruler -->
          <div
            class="absolute top-0 w-0.5 h-full bg-crimson z-30 pointer-events-none"
            :style="{ left: `${scrubberPosition}%` }"
          >
            <div class="absolute -bottom-1 -left-1.5 w-3.5 h-2 bg-crimson rounded-b-sm"></div>
          </div>
        </div>

        <!-- Tracks: Bars mode -->
        <div
          v-if="viewMode === 'bars'"
          class="relative"
          :class="isDragOver ? 'bg-crimson/5' : ''"
          :style="{ minHeight: Math.max(TRACK_HEIGHT, store.layers.length * TRACK_HEIGHT) + 'px' }"
          @drop="handleTimelineDrop"
          @dragover.prevent="isDragOver = true"
          @dragleave="isDragOver = false"
        >
          <!-- Track rows (alternating bg) -->
          <div
            v-for="(layer, idx) in store.layers"
            :key="'track-' + layer.id"
            class="absolute w-full border-b border-cosmos-border/20"
            :class="idx % 2 === 0 ? 'bg-cosmos-deep/30' : 'bg-cosmos-deep/10'"
            :style="{ top: idx * TRACK_HEIGHT + 'px', height: TRACK_HEIGHT + 'px' }"
          ></div>

          <!-- Layer bars -->
          <div
            v-for="(layer, idx) in store.layers"
            :key="'bar-' + layer.id"
            class="absolute rounded-[3px] cursor-grab active:cursor-grabbing transition-shadow flex items-center px-1.5 overflow-hidden border"
            :class="[
              layerColors[layer.type] || 'bg-marble/40',
              layer.id === store.selectedLayerId
                ? 'border-crimson shadow-[0_0_8px_rgba(193,18,31,0.3)]'
                : 'border-transparent hover:border-marble/60',
              dragLayerId === layer.id ? 'opacity-60' : ''
            ]"
            :style="{
              left: `${(layer.in / store.totalFrames) * 100}%`,
              width: `${Math.max(((layer.out - layer.in) / store.totalFrames) * 100, 1)}%`,
              top: (idx * TRACK_HEIGHT + 4) + 'px',
              height: (TRACK_HEIGHT - 8) + 'px',
            }"
            @click.stop="store.selectLayer(layer.id)"
            @mousedown.stop="handleBarDragStart($event, layer)"
          >
            <span class="font-mono text-[9px] text-white truncate select-none drop-shadow-sm">
              {{ layer.id }}
            </span>
          </div>

          <!-- Playhead -->
          <div
            class="absolute top-0 w-0.5 bg-crimson z-10 pointer-events-none"
            :style="{ left: `${scrubberPosition}%`, height: Math.max(TRACK_HEIGHT, store.layers.length * TRACK_HEIGHT) + 'px' }"
          >
            <div class="absolute -top-2 -left-1.5 w-3.5 h-5 bg-crimson rounded-sm"></div>
          </div>
        </div>

        <!-- Tracks: Keys (dope sheet) mode -->
        <div
          v-if="viewMode === 'keys'"
          class="relative"
          :class="isDragOver ? 'bg-crimson/5' : ''"
          :style="{ minHeight: keysTotalHeight + 'px' }"
          @drop="handleTimelineDrop"
          @dragover.prevent="isDragOver = true"
          @dragleave="isDragOver = false"
        >
          <template v-for="(layer, layerIdx) in store.layers" :key="'keys-' + layer.id">
            <!-- Layer header row -->
            <div
              class="absolute w-full border-b border-cosmos-border/30 flex items-center cursor-pointer"
              :class="layer.id === store.selectedLayerId ? 'bg-crimson/5' : ''"
              :style="{ top: getKeysRowTop(layerIdx, -1) + 'px', height: KEYS_HEADER_HEIGHT + 'px' }"
              @click="store.selectLayer(layer.id)"
            >
              <span class="font-mono text-[10px] text-text-primary ml-2 truncate">{{ layer.id }}</span>
            </div>

            <!-- Property sub-rows -->
            <template v-for="(propInfo, propIdx) in getAnimatedProps(layer)" :key="'prop-' + layer.id + '-' + propInfo.path">
              <div
                class="absolute w-full border-b border-cosmos-border/10"
                :class="propIdx % 2 === 0 ? 'bg-cosmos-deep/20' : ''"
                :style="{ top: getKeysRowTop(layerIdx, propIdx) + 'px', height: KEYS_PROP_HEIGHT + 'px' }"
              >
                <!-- Keyframe diamonds -->
                <template v-if="propInfo.keyframes">
                  <div
                    v-for="(kf, kfIdx) in propInfo.keyframes"
                    :key="kfIdx"
                    class="absolute top-1/2 -translate-y-1/2 w-2.5 h-2.5 rotate-45 cursor-pointer transition-colors"
                    :class="kf.t === store.currentFrame ? 'bg-yellow-400 shadow-[0_0_6px_rgba(250,204,21,0.5)]' : 'bg-marble/70 hover:bg-crimson'"
                    :style="{ left: `calc(${(kf.t / store.totalFrames) * 100}% - 5px)` }"
                    :title="`${propInfo.label} @ frame ${kf.t}`"
                    @click.stop="store.setFrame(kf.t)"
                    @contextmenu="handleKeyframeContextMenu($event, layer.id, propInfo.path, kf.t)"
                  />
                </template>

                <!-- Empty row indicator when no keyframes -->
                <template v-if="!propInfo.keyframes">
                  <div class="absolute inset-0 flex items-center justify-center">
                    <span class="font-mono text-[8px] text-text-muted/30 uppercase">no keys</span>
                  </div>
                </template>
              </div>
            </template>
          </template>

          <!-- Playhead -->
          <div
            class="absolute top-0 w-0.5 bg-crimson z-10 pointer-events-none"
            :style="{ left: `${scrubberPosition}%`, height: keysTotalHeight + 'px' }"
          >
            <div class="absolute -top-2 -left-1.5 w-3.5 h-5 bg-crimson rounded-sm"></div>
          </div>
        </div>

        <!-- Easing Curve Picker -->
        <EasingPicker
          ref="easingPickerRef"
          :style="{ left: easingPickerPos.x + 'px', top: easingPickerPos.y + 'px', position: 'fixed' }"
          @select="handleEasingSelect"
        />
      </div>
    </div>
  </div>
</template>
