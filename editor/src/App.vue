<script setup lang="ts">
import { onMounted } from 'vue'
import { useSceneStore } from './stores/scene'
import TopBar from './components/TopBar.vue'
import LayerPanel from './components/LayerPanel.vue'
import CanvasPreview from './components/CanvasPreview.vue'
import Timeline from './components/Timeline.vue'
import PropertyInspector from './components/PropertyInspector.vue'

const store = useSceneStore()

onMounted(() => {
  // Start with default scene
  store.setFrame(0)
})
</script>

<template>
  <div class="h-screen w-screen flex flex-col bg-cosmos-deep text-text-primary font-sans overflow-hidden select-none">
    <!-- Top Bar -->
    <TopBar />

    <!-- Main Content -->
    <div class="flex-1 flex overflow-hidden">
      <!-- Left: Layer Panel -->
      <LayerPanel class="w-64 border-r border-cosmos-border" />

      <!-- Center: Canvas Preview -->
      <CanvasPreview class="flex-1" />

      <!-- Right: Property Inspector -->
      <PropertyInspector class="w-72 border-l border-cosmos-border" />
    </div>

    <!-- Bottom: Timeline -->
    <Timeline class="h-48 border-t border-cosmos-border" />
  </div>
</template>
