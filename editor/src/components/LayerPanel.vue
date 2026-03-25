<script setup lang="ts">
import { ref, computed } from 'vue'
import { useSceneStore } from '../stores/scene'
import AddLayerDialog from './AddLayerDialog.vue'

const store = useSceneStore()
const addLayerDialogRef = ref<InstanceType<typeof AddLayerDialog> | null>(null)
const dragFromIndex = ref<number | null>(null)
const dragOverIndex = ref<number | null>(null)

const layerTypeIcons: Record<string, string> = {
  solid: '■', text: 'T', image: '◻', video: '▶', shape: '△',
  gradient: '◐', null: '◎', audio: '♪', composition: '⊞',
}

// Reversed for display (top = front of stack)
const reversedLayers = computed(() => [...store.layers].reverse())

function openAddLayer() {
  if (addLayerDialogRef.value) {
    addLayerDialogRef.value.showDialog = true
  }
}

// --- Drag to reorder ---
function handleDragStart(event: DragEvent, displayIndex: number) {
  if (!event.dataTransfer) return
  // Convert display index (reversed) to actual store index
  const actualIndex = store.layers.length - 1 - displayIndex
  dragFromIndex.value = actualIndex
  event.dataTransfer.setData('application/mmot-layer-reorder', String(actualIndex))
  event.dataTransfer.effectAllowed = 'move'
}

function handleDragOver(event: DragEvent, displayIndex: number) {
  event.preventDefault()
  if (event.dataTransfer) event.dataTransfer.dropEffect = 'move'
  dragOverIndex.value = displayIndex
}

function handleDragLeave() {
  dragOverIndex.value = null
}

function handleDrop(event: DragEvent, displayIndex: number) {
  event.preventDefault()
  dragOverIndex.value = null

  // Check if it's a media drop from the media browser
  const mediaData = event.dataTransfer?.getData('application/mmot-media')
  if (mediaData) {
    const asset = JSON.parse(mediaData)
    const id = `${asset.type}_${Date.now().toString(36)}`
    const w = store.scene.meta.width
    const h = store.scene.meta.height
    const layer: any = {
      id, type: asset.type, in: 0, out: store.totalFrames,
      transform: { position: [w / 2, h / 2] }, src: asset.path,
    }
    if (asset.type === 'video') layer.trim_start = 0
    if (asset.type === 'audio') layer.volume = 1.0
    store.addLayer(layer)
    store.selectLayer(id)
    return
  }

  // Layer reorder
  const fromData = event.dataTransfer?.getData('application/mmot-layer-reorder')
  if (fromData === undefined || fromData === null || fromData === '') return
  const fromActual = parseInt(fromData)
  const toActual = store.layers.length - 1 - displayIndex
  if (fromActual !== toActual) {
    store.reorderLayer(fromActual, toActual)
  }
  dragFromIndex.value = null
}

function handleDragEnd() {
  dragFromIndex.value = null
  dragOverIndex.value = null
}

// --- Drop from OS files directly into layer panel ---
function handlePanelDrop(event: DragEvent) {
  event.preventDefault()
  if (!event.dataTransfer?.files.length) return
  for (const file of Array.from(event.dataTransfer.files)) {
    const ext = file.name.split('.').pop()?.toLowerCase() || ''
    const validExts = ['png', 'jpg', 'jpeg', 'webp', 'mp4', 'webm', 'mp3', 'wav', 'flac', 'ogg']
    if (!validExts.includes(ext)) continue
    const type = ['mp4', 'webm'].includes(ext) ? 'video' : ['mp3', 'wav', 'flac', 'ogg'].includes(ext) ? 'audio' : 'image'
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
</script>

<template>
  <div
    class="flex flex-col bg-cosmos-card overflow-hidden"
    @drop.prevent="handlePanelDrop"
    @dragover.prevent
  >
    <!-- Header -->
    <div class="h-10 flex items-center justify-between px-3 border-b border-cosmos-border">
      <span class="font-mono text-[10px] text-text-muted uppercase tracking-[0.2em]">Layers</span>
      <button
        class="w-6 h-6 flex items-center justify-center text-text-muted hover:text-crimson rounded transition-colors text-lg"
        @click="openAddLayer"
      >
        +
      </button>
    </div>

    <!-- Layer List -->
    <div class="flex-1 overflow-y-auto">
      <div
        v-for="(layer, displayIdx) in reversedLayers"
        :key="layer.id"
        class="flex items-center gap-2 px-3 py-2 cursor-grab active:cursor-grabbing border-b transition-colors"
        :class="[
          layer.id === store.selectedLayerId
            ? 'bg-crimson/10 border-l-2 border-l-crimson border-b-cosmos-border/50'
            : 'hover:bg-cosmos/50 border-l-2 border-l-transparent border-b-cosmos-border/50',
          dragOverIndex === displayIdx ? 'border-t-2 border-t-crimson' : ''
        ]"
        draggable="true"
        @click="store.selectLayer(layer.id)"
        @dragstart="handleDragStart($event, displayIdx)"
        @dragover="handleDragOver($event, displayIdx)"
        @dragleave="handleDragLeave"
        @drop="handleDrop($event, displayIdx)"
        @dragend="handleDragEnd"
      >
        <!-- Drag Handle -->
        <span class="font-mono text-[10px] text-text-muted/40 select-none">⋮⋮</span>

        <!-- Type Icon -->
        <span class="font-mono text-xs text-text-muted w-4 text-center">
          {{ layerTypeIcons[layer.type] || '?' }}
        </span>

        <!-- Layer Name -->
        <span class="font-sans text-xs text-text-primary truncate flex-1">
          {{ layer.id }}
        </span>

        <!-- Type Badge -->
        <span class="font-mono text-[9px] text-text-muted uppercase tracking-wider">
          {{ layer.type }}
        </span>

        <!-- Delete -->
        <button
          class="opacity-0 group-hover:opacity-100 text-text-muted hover:text-crimson text-xs"
          @click.stop="store.removeLayer(layer.id)"
        >
          ×
        </button>
      </div>

      <!-- Empty State -->
      <div v-if="store.layers.length === 0" class="flex flex-col items-center justify-center py-12 gap-2">
        <div class="w-10 h-10 rounded-full border border-dashed border-cosmos-border flex items-center justify-center">
          <span class="text-text-muted text-lg">+</span>
        </div>
        <span class="font-mono text-[10px] text-text-muted uppercase tracking-widest">Add or drop a layer</span>
      </div>
    </div>

    <AddLayerDialog ref="addLayerDialogRef" />
  </div>
</template>
