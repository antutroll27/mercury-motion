<script setup lang="ts">
import { ref } from 'vue'
import { useSceneStore } from '../stores/scene'
import ExportModal from './ExportModal.vue'

const store = useSceneStore()
const showExportModal = ref(false)
const fileInputRef = ref<HTMLInputElement | null>(null)

function openFile() {
  fileInputRef.value?.click()
}

function handleFileLoad(e: Event) {
  const input = e.target as HTMLInputElement
  if (!input.files?.length) return
  const file = input.files[0]
  const reader = new FileReader()
  reader.onload = () => {
    try {
      const json = reader.result as string
      JSON.parse(json) // validate it's JSON
      store.fromJson(json)
      store.setFrame(0)
    } catch (err) {
      alert(`Invalid .mmot.json file: ${err}`)
    }
  }
  reader.readAsText(file)
  input.value = '' // reset so same file can be re-loaded
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

    <!-- Open File -->
    <input
      ref="fileInputRef"
      type="file"
      accept=".mmot.json,.json"
      class="hidden"
      @change="handleFileLoad"
    />
    <button
      class="px-3 py-1.5 bg-cosmos-deep border border-cosmos-border text-text-muted text-xs font-mono uppercase tracking-widest rounded hover:border-crimson hover:text-crimson transition-colors"
      @click="openFile"
    >
      Open
    </button>

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
      class="px-4 py-1.5 bg-crimson text-varden text-xs font-mono uppercase tracking-widest rounded hover:bg-gochujang transition-colors"
      @click="showExportModal = true"
    >
      Export
    </button>

    <!-- Export Modal -->
    <ExportModal v-if="showExportModal" @close="showExportModal = false" />
  </header>
</template>
