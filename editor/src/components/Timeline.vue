<script setup lang="ts">
import { ref, computed } from 'vue'
import { useSceneStore } from '../stores/scene'

const store = useSceneStore()
const timelineRef = ref<HTMLDivElement | null>(null)
const isDragging = ref(false)

const scrubberPosition = computed(() => {
  if (store.totalFrames <= 0) return 0
  return (store.currentFrame / store.totalFrames) * 100
})

function handleTimelineClick(e: MouseEvent) {
  if (!timelineRef.value) return
  const rect = timelineRef.value.getBoundingClientRect()
  const x = (e.clientX - rect.left) / rect.width
  store.setFrame(Math.round(x * store.totalFrames))
}

function handleMouseDown(e: MouseEvent) {
  isDragging.value = true
  handleTimelineClick(e)
  window.addEventListener('mousemove', handleMouseMove)
  window.addEventListener('mouseup', handleMouseUp)
}

function handleMouseMove(e: MouseEvent) {
  if (!isDragging.value || !timelineRef.value) return
  const rect = timelineRef.value.getBoundingClientRect()
  const x = Math.max(0, Math.min(1, (e.clientX - rect.left) / rect.width))
  store.setFrame(Math.round(x * store.totalFrames))
}

function handleMouseUp() {
  isDragging.value = false
  window.removeEventListener('mousemove', handleMouseMove)
  window.removeEventListener('mouseup', handleMouseUp)
}

// Generate tick marks
const ticks = computed(() => {
  const marks: { frame: number; position: number; label: string; major: boolean }[] = []
  const fps = store.scene.meta.fps
  const total = store.totalFrames
  const step = fps // one tick per second
  for (let i = 0; i <= total; i += step) {
    marks.push({
      frame: i,
      position: (i / total) * 100,
      label: `${Math.floor(i / fps)}s`,
      major: true,
    })
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
        <span class="text-sm">{{ store.isPlaying ? '\u23F8' : '\u25B6' }}</span>
      </button>
      <button
        class="font-mono text-xs text-text-muted uppercase tracking-widest hover:text-crimson transition-colors"
        @click="store.setFrame(store.totalFrames - 1)"
      >
        &raquo;
      </button>

      <div class="flex-1" />

      <!-- Timecode Display -->
      <div class="font-mono text-2xl tracking-widest text-text-primary tabular-nums">
        {{ store.currentTimecode }}
      </div>
    </div>

    <!-- Timeline Ruler -->
    <div class="flex-1 relative px-4 py-2">
      <!-- Tick Marks -->
      <div class="h-6 relative mb-1">
        <template v-for="tick in ticks" :key="tick.frame">
          <div
            class="absolute top-0 h-full flex flex-col items-center"
            :style="{ left: `${tick.position}%` }"
          >
            <div class="w-px h-3 bg-cosmos-border"></div>
            <span class="font-mono text-[9px] text-text-muted mt-0.5">{{ tick.label }}</span>
          </div>
        </template>
      </div>

      <!-- Scrub Area -->
      <div
        ref="timelineRef"
        class="h-8 relative bg-cosmos-deep rounded cursor-pointer"
        @mousedown="handleMouseDown"
      >
        <!-- Layer Bars -->
        <template v-for="layer in store.layers" :key="layer.id">
          <div
            class="absolute h-5 rounded-sm top-1.5 cursor-pointer transition-colors"
            :class="[
              layer.id === store.selectedLayerId ? 'bg-crimson/80' : 'bg-marble/40 hover:bg-marble/60'
            ]"
            :style="{
              left: `${(layer.in / store.totalFrames) * 100}%`,
              width: `${((layer.out - layer.in) / store.totalFrames) * 100}%`,
            }"
            @click.stop="store.selectLayer(layer.id)"
          >
            <span class="font-mono text-[9px] text-text-primary truncate px-1 leading-5">
              {{ layer.id }}
            </span>
          </div>
        </template>

        <!-- Scrubber / Playhead -->
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
