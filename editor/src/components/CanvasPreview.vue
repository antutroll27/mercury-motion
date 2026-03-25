<script setup lang="ts">
import { ref, watch } from 'vue'
import { useSceneStore } from '../stores/scene'

const store = useSceneStore()
const canvasRef = ref<HTMLCanvasElement | null>(null)

// In Tauri, this would call render_frame IPC
// For now, show a placeholder with scene dimensions
watch(() => store.previewImage, (img) => {
  if (!img || !canvasRef.value) return
  const canvas = canvasRef.value
  const ctx = canvas.getContext('2d')
  if (!ctx) return
  const image = new Image()
  image.onload = () => {
    ctx.clearRect(0, 0, canvas.width, canvas.height)
    ctx.drawImage(image, 0, 0, canvas.width, canvas.height)
  }
  image.src = img
})
</script>

<template>
  <div class="flex flex-col items-center justify-center bg-cosmos-deep p-4 gap-3">
    <!-- Canvas Label -->
    <div class="font-mono text-[10px] text-text-muted uppercase tracking-[0.2em]">
      Preview
    </div>

    <!-- Canvas Container -->
    <div class="relative bg-black rounded border border-cosmos-border shadow-2xl overflow-hidden"
         :style="{ aspectRatio: `${store.scene.meta.width}/${store.scene.meta.height}`, maxWidth: '100%', maxHeight: 'calc(100% - 40px)' }">
      <canvas
        ref="canvasRef"
        :width="store.scene.meta.width"
        :height="store.scene.meta.height"
        class="w-full h-full object-contain"
      />

      <!-- Empty State -->
      <div v-if="!store.previewImage"
           class="absolute inset-0 flex flex-col items-center justify-center gap-2">
        <div class="w-16 h-16 border border-cosmos-border rounded-full flex items-center justify-center">
          <div class="w-6 h-6 border-2 border-text-muted rounded-sm rotate-45"></div>
        </div>
        <span class="font-mono text-xs text-text-muted uppercase tracking-widest">
          {{ store.scene.meta.width }} &times; {{ store.scene.meta.height }}
        </span>
      </div>
    </div>
  </div>
</template>
