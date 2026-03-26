<script setup lang="ts">
import { onMounted, onBeforeUnmount, ref } from 'vue'
import { useSceneStore } from './stores/scene'
import TopBar from './components/TopBar.vue'
import LayerPanel from './components/LayerPanel.vue'
import MediaBrowser from './components/MediaBrowser.vue'
import CanvasPreview from './components/CanvasPreview.vue'
import Timeline from './components/Timeline.vue'
import PropertyInspector from './components/PropertyInspector.vue'

const store = useSceneStore()
const leftTab = ref<'layers' | 'media'>('layers')

function handleKeydown(e: KeyboardEvent) {
  // Ctrl+Z / Cmd+Z = Undo
  if ((e.ctrlKey || e.metaKey) && e.key === 'z' && !e.shiftKey) {
    e.preventDefault()
    store.undo()
  }
  // Ctrl+Shift+Z / Cmd+Shift+Z = Redo
  if ((e.ctrlKey || e.metaKey) && e.key === 'z' && e.shiftKey) {
    e.preventDefault()
    store.redo()
  }
  // Ctrl+Y = Redo (Windows)
  if ((e.ctrlKey || e.metaKey) && e.key === 'y') {
    e.preventDefault()
    store.redo()
  }
  // Space = Play/Pause
  if (e.key === ' ' && !(e.target instanceof HTMLInputElement) && !(e.target instanceof HTMLTextAreaElement) && !(e.target instanceof HTMLSelectElement)) {
    e.preventDefault()
    store.togglePlayback()
  }
  // Delete/Backspace = Delete selected layer
  if ((e.key === 'Delete' || e.key === 'Backspace') && store.selectedLayerId && !(e.target instanceof HTMLInputElement) && !(e.target instanceof HTMLTextAreaElement)) {
    e.preventDefault()
    store.removeLayer(store.selectedLayerId)
  }
}

onMounted(() => {
  store.setFrame(0)
  window.addEventListener('keydown', handleKeydown)
})

onBeforeUnmount(() => {
  window.removeEventListener('keydown', handleKeydown)
})
</script>

<template>
  <div class="h-screen w-screen flex flex-col bg-cosmos-deep text-text-primary font-sans overflow-hidden select-none">
    <!-- Top Bar -->
    <TopBar />

    <!-- Main Content -->
    <div class="flex-1 flex overflow-hidden">
      <!-- Left: Layer Panel / Media Browser -->
      <div class="w-64 border-r border-cosmos-border flex flex-col overflow-hidden">
        <!-- Tab Switcher -->
        <div class="flex border-b border-cosmos-border">
          <button
            class="flex-1 py-2 font-mono text-[10px] uppercase tracking-[0.15em] transition-colors"
            :class="leftTab === 'layers' ? 'text-crimson border-b-2 border-crimson' : 'text-text-muted hover:text-text-primary'"
            @click="leftTab = 'layers'"
          >
            Layers
          </button>
          <button
            class="flex-1 py-2 font-mono text-[10px] uppercase tracking-[0.15em] transition-colors"
            :class="leftTab === 'media' ? 'text-crimson border-b-2 border-crimson' : 'text-text-muted hover:text-text-primary'"
            @click="leftTab = 'media'"
          >
            Media
          </button>
        </div>

        <LayerPanel v-show="leftTab === 'layers'" class="flex-1" />
        <MediaBrowser v-show="leftTab === 'media'" class="flex-1" />
      </div>

      <!-- Center: Canvas Preview -->
      <CanvasPreview class="flex-1" />

      <!-- Right: Property Inspector -->
      <PropertyInspector class="w-72 border-l border-cosmos-border" />
    </div>

    <!-- Bottom: Timeline -->
    <Timeline class="h-48 border-t border-cosmos-border" />
  </div>
</template>
