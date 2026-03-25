<script setup lang="ts">
import { onMounted, watch } from 'vue'
import { useSceneStore } from '../stores/scene'

const store = useSceneStore()

onMounted(() => {
  store.requestPreview()
})

watch(() => store.currentFrame, () => {
  store.requestPreview()
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
      <!-- Rendered Preview -->
      <img v-if="store.previewImage" :src="store.previewImage" class="w-full h-full object-contain" />

      <!-- Empty State -->
      <div v-else class="absolute inset-0 flex flex-col items-center justify-center gap-2">
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
