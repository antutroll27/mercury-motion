<script setup lang="ts">
import { computed, ref } from 'vue'
import { useSceneStore } from '../stores/scene'

const props = defineProps<{
  layerId: string
  path: string
}>()

const store = useSceneStore()
const stripRef = ref<HTMLDivElement | null>(null)
const dragIdx = ref<number | null>(null)

const keyframes = computed(() => store.getKeyframes(props.layerId, props.path))
const isAnimated = computed(() => keyframes.value !== null)
const hasKfAtCurrentFrame = computed(() => store.hasKeyframeAtFrame(props.layerId, props.path, store.currentFrame))

function getCurrentValue(): any {
  const layer = store.selectedLayer
  if (!layer) return 0
  const keys = props.path.split('.')
  let obj: any = layer
  for (const k of keys) { obj = obj?.[k] }
  // If animated, get value at current frame
  if (Array.isArray(obj) && obj.length > 0 && typeof obj[0] === 'object' && 't' in obj[0]) {
    const kfs = obj
    const frame = store.currentFrame
    if (frame <= kfs[0].t) return kfs[0].v
    if (frame >= kfs[kfs.length - 1].t) return kfs[kfs.length - 1].v
    for (let i = 0; i < kfs.length - 1; i++) {
      if (frame >= kfs[i].t && frame <= kfs[i + 1].t) {
        const t = (frame - kfs[i].t) / (kfs[i + 1].t - kfs[i].t)
        const v0 = kfs[i].v
        const v1 = kfs[i + 1].v
        if (typeof v0 === 'number') return v0 + (v1 - v0) * t
        return v0
      }
    }
    return kfs[0].v
  }
  return typeof obj === 'number' ? obj : 0
}

function toggleKeyframe() {
  if (hasKfAtCurrentFrame.value) {
    store.removeKeyframe(props.layerId, props.path, store.currentFrame)
  } else {
    store.addKeyframe(props.layerId, props.path, store.currentFrame, getCurrentValue())
  }
}

function handleStripContext(e: MouseEvent) {
  e.preventDefault()
  if (!keyframes.value) return
  const frame = posToFrame(e)
  // Find nearest keyframe within 2 frames
  const nearest = keyframes.value.reduce((best, kf) => {
    return Math.abs(kf.t - frame) < Math.abs(best.t - frame) ? kf : best
  })
  if (Math.abs(nearest.t - frame) <= 2) {
    store.removeKeyframe(props.layerId, props.path, nearest.t)
  }
}

function posToFrame(e: MouseEvent): number {
  if (!stripRef.value) return 0
  const rect = stripRef.value.getBoundingClientRect()
  const x = (e.clientX - rect.left) / rect.width
  return Math.round(x * store.totalFrames)
}

function handleDragStart(e: MouseEvent, idx: number) {
  e.preventDefault()
  dragIdx.value = idx
  window.addEventListener('mousemove', handleDragMove)
  window.addEventListener('mouseup', handleDragEnd)
}

function handleDragMove(e: MouseEvent) {
  if (dragIdx.value === null || !keyframes.value) return
  const frame = posToFrame(e)
  const kf = keyframes.value[dragIdx.value]
  if (kf && frame >= 0 && frame < store.totalFrames) {
    kf.t = frame
    store.schedulePreview()
  }
}

function handleDragEnd() {
  dragIdx.value = null
  window.removeEventListener('mousemove', handleDragMove)
  window.removeEventListener('mouseup', handleDragEnd)
}
</script>

<template>
  <div class="flex items-center gap-1 mt-0.5">
    <!-- Keyframe toggle diamond -->
    <button
      class="w-4 h-4 flex items-center justify-center text-[10px] transition-colors"
      :class="hasKfAtCurrentFrame ? 'text-yellow-400' : isAnimated ? 'text-marble' : 'text-text-muted/40 hover:text-marble'"
      :title="hasKfAtCurrentFrame ? 'Remove keyframe' : 'Add keyframe at current frame'"
      @click="toggleKeyframe"
    >&#9670;</button>

    <!-- Mini timeline strip -->
    <div
      v-if="isAnimated && keyframes"
      ref="stripRef"
      class="flex-1 h-3 bg-cosmos-deep rounded relative cursor-pointer"
      @contextmenu.prevent="handleStripContext"
    >
      <!-- Playhead marker -->
      <div
        class="absolute top-0 w-px h-full bg-crimson/50"
        :style="{ left: `${(store.currentFrame / store.totalFrames) * 100}%` }"
      />

      <!-- Keyframe diamonds -->
      <div
        v-for="(kf, idx) in keyframes"
        :key="idx"
        class="absolute top-1/2 -translate-y-1/2 w-2 h-2 rotate-45 cursor-grab active:cursor-grabbing transition-colors"
        :class="kf.t === store.currentFrame ? 'bg-yellow-400' : 'bg-marble hover:bg-crimson'"
        :style="{ left: `calc(${(kf.t / store.totalFrames) * 100}% - 4px)` }"
        :title="`Frame ${kf.t}: ${typeof kf.v === 'number' ? kf.v.toFixed(2) : JSON.stringify(kf.v)}`"
        @mousedown="handleDragStart($event, idx)"
      />
    </div>
  </div>
</template>
