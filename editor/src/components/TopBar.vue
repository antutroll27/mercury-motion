<script setup lang="ts">
import { ref } from 'vue'
import { useSceneStore } from '../stores/scene'
import { renderToFile } from '../lib/tauri-commands'

const store = useSceneStore()
const isExporting = ref(false)

async function handleExport() {
  isExporting.value = true
  try {
    let outputPath = 'output.mp4'

    // Try to open a save dialog via Tauri dialog plugin
    try {
      const { invoke } = await import('@tauri-apps/api/core')
      const path = await invoke<string | null>('plugin:dialog|save', {
        filters: [
          { name: 'MP4', extensions: ['mp4'] },
          { name: 'GIF', extensions: ['gif'] },
          { name: 'WebM', extensions: ['webm'] },
        ]
      })
      if (path) outputPath = path
    } catch {
      // TODO: dialog plugin not available, using default path
      console.warn('Save dialog not available, exporting to default path:', outputPath)
    }

    const format = outputPath.endsWith('.gif') ? 'gif' : outputPath.endsWith('.webm') ? 'webm' : 'mp4'
    await renderToFile(store.toJson(), outputPath, format, 80)
    alert(`Exported to ${outputPath}`)
  } catch (e) {
    console.error('Export failed:', e)
    alert(`Export failed: ${e}`)
  } finally {
    isExporting.value = false
  }
}
</script>

<template>
  <header class="h-12 bg-cosmos-card border-b border-cosmos-border flex items-center px-4 gap-6">
    <!-- Logo -->
    <div class="flex items-center gap-2">
      <div class="w-2 h-2 rounded-full bg-crimson"></div>
      <span class="font-serif text-sm tracking-wide text-varden">Mercury Motion</span>
    </div>

    <!-- Timecode -->
    <div class="font-mono text-lg tracking-widest text-text-primary tabular-nums">
      {{ store.currentTimecode }}
    </div>

    <!-- Frame Counter -->
    <div class="font-mono text-xs text-text-muted uppercase tracking-widest">
      Frame {{ store.currentFrame }} / {{ store.totalFrames }}
    </div>

    <div class="flex-1" />

    <!-- Resolution -->
    <div class="font-mono text-xs text-text-muted uppercase tracking-widest">
      {{ store.scene.meta.width }}&times;{{ store.scene.meta.height }}
    </div>

    <!-- FPS Selector -->
    <select
      :value="store.scene.meta.fps"
      @change="store.scene.meta.fps = Number(($event.target as HTMLSelectElement).value); store.schedulePreview()"
      class="bg-cosmos-deep border border-cosmos-border rounded px-2 py-1 font-mono text-xs text-text-primary focus:border-crimson outline-none cursor-pointer"
    >
      <option :value="23.976">23.976</option>
      <option :value="24">24</option>
      <option :value="25">25</option>
      <option :value="30">30</option>
      <option :value="60">60</option>
      <option :value="90">90</option>
      <option :value="120">120</option>
    </select>
    <span class="font-mono text-xs text-text-muted uppercase">fps</span>

    <!-- Export Button -->
    <button
      class="px-4 py-1.5 bg-crimson text-varden text-xs font-mono uppercase tracking-widest rounded hover:bg-gochujang transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
      :disabled="isExporting"
      @click="handleExport"
    >
      {{ isExporting ? 'Exporting...' : 'Export' }}
    </button>
  </header>
</template>
