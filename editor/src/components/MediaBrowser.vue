<script setup lang="ts">
import { ref } from 'vue'
import { useSceneStore } from '../stores/scene'

const store = useSceneStore()
const assets = ref<{ name: string, path: string, type: string }[]>([])
const fileInputRef = ref<HTMLInputElement | null>(null)
const isDragOver = ref(false)

function getMediaType(ext: string): string {
  if (['mp4', 'webm', 'mov', 'avi'].includes(ext)) return 'video'
  if (['mp3', 'wav', 'flac', 'ogg', 'aac'].includes(ext)) return 'audio'
  return 'image'
}

async function importMedia() {
  try {
    const { open } = await import('@tauri-apps/plugin-dialog')
    const selected = await open({
      multiple: true,
      filters: [
        { name: 'Media', extensions: ['png', 'jpg', 'jpeg', 'webp', 'mp4', 'webm', 'mp3', 'wav', 'flac'] },
      ]
    })
    if (selected) {
      const paths = Array.isArray(selected) ? selected : [selected]
      for (const p of paths) {
        const pathStr = typeof p === 'string' ? p : (p as any).path ?? String(p)
        addAsset(pathStr)
      }
    }
    return
  } catch {
    // Not in Tauri
  }
  fileInputRef.value?.click()
}

function addAsset(pathStr: string) {
  const name = pathStr.split(/[/\\]/).pop() || pathStr
  const ext = name.split('.').pop()?.toLowerCase() || ''
  assets.value.push({ name, path: pathStr, type: getMediaType(ext) })
}

function handleFileInput(event: Event) {
  const input = event.target as HTMLInputElement
  if (!input.files) return
  for (const file of Array.from(input.files)) {
    const ext = file.name.split('.').pop()?.toLowerCase() || ''
    assets.value.push({
      name: file.name,
      path: URL.createObjectURL(file),
      type: getMediaType(ext),
    })
  }
  input.value = ''
}

// --- Drag from media browser (as source) ---
function handleDragStart(event: DragEvent, asset: { name: string, path: string, type: string }) {
  if (!event.dataTransfer) return
  event.dataTransfer.setData('application/mmot-media', JSON.stringify(asset))
  event.dataTransfer.effectAllowed = 'copy'
}

// --- Drop files from OS into media browser ---
function handleDrop(event: DragEvent) {
  event.preventDefault()
  isDragOver.value = false
  if (!event.dataTransfer?.files.length) return
  for (const file of Array.from(event.dataTransfer.files)) {
    const ext = file.name.split('.').pop()?.toLowerCase() || ''
    const validExts = ['png', 'jpg', 'jpeg', 'webp', 'mp4', 'webm', 'mp3', 'wav', 'flac', 'ogg', 'gif']
    if (!validExts.includes(ext)) continue
    assets.value.push({
      name: file.name,
      path: (file as any).path || URL.createObjectURL(file),
      type: getMediaType(ext),
    })
  }
}

function handleDragOver(event: DragEvent) {
  event.preventDefault()
  isDragOver.value = true
}

function handleDragLeave() {
  isDragOver.value = false
}

function addToTimeline(asset: { name: string, path: string, type: string }) {
  const id = `${asset.type}_${Date.now().toString(36)}`
  const w = store.scene.meta.width
  const h = store.scene.meta.height

  const layer: any = {
    id,
    type: asset.type,
    in: 0,
    out: store.totalFrames,
    transform: { position: [w / 2, h / 2] },
    src: asset.path,
  }

  if (asset.type === 'video') layer.trim_start = 0
  if (asset.type === 'audio') layer.volume = 1.0

  store.addLayer(layer)
  store.selectLayer(id)
}

function removeAsset(index: number) {
  assets.value.splice(index, 1)
}
</script>

<template>
  <div
    class="flex flex-col h-full"
    @drop="handleDrop"
    @dragover="handleDragOver"
    @dragleave="handleDragLeave"
  >
    <input
      ref="fileInputRef"
      type="file"
      multiple
      accept="image/*,video/*,audio/*"
      class="hidden"
      @change="handleFileInput"
    />

    <!-- Header -->
    <div class="h-10 flex items-center justify-between px-3 border-b border-cosmos-border">
      <span class="font-mono text-[10px] text-text-muted uppercase tracking-[0.2em]">Media</span>
      <button
        class="px-2 py-1 text-text-muted hover:text-crimson font-mono text-[10px] uppercase tracking-wider border border-cosmos-border rounded hover:border-crimson transition-colors"
        @click="importMedia"
      >
        + Import
      </button>
    </div>

    <!-- Asset List -->
    <div class="flex-1 overflow-y-auto">
      <div
        v-for="(asset, idx) in assets"
        :key="idx"
        class="flex items-center gap-2 px-3 py-2 border-b border-cosmos-border/50 hover:bg-cosmos-deep/50 group cursor-grab active:cursor-grabbing"
        draggable="true"
        @dragstart="handleDragStart($event, asset)"
      >
        <span class="font-mono text-xs text-text-muted">
          {{ asset.type === 'image' ? '◻' : asset.type === 'video' ? '▶' : '♪' }}
        </span>
        <span
          class="font-sans text-xs text-text-primary truncate flex-1 cursor-pointer hover:text-crimson transition-colors"
          @click="addToTimeline(asset)"
        >
          {{ asset.name }}
        </span>
        <span class="font-mono text-[9px] text-text-muted uppercase">{{ asset.type }}</span>
        <button
          class="opacity-0 group-hover:opacity-100 text-text-muted hover:text-crimson text-xs transition-opacity"
          @click.stop="removeAsset(idx)"
        >
          ×
        </button>
      </div>

      <!-- Empty / Drop State -->
      <div
        v-if="assets.length === 0"
        class="flex flex-col items-center justify-center py-8 gap-3 transition-colors"
        :class="isDragOver ? 'bg-crimson/10' : ''"
      >
        <button
          class="w-16 h-16 rounded-lg border-2 border-dashed flex items-center justify-center transition-colors group"
          :class="isDragOver ? 'border-crimson' : 'border-cosmos-border hover:border-crimson'"
          @click="importMedia"
        >
          <span class="text-2xl transition-colors" :class="isDragOver ? 'text-crimson' : 'text-text-muted group-hover:text-crimson'">+</span>
        </button>
        <span class="font-mono text-[10px] text-text-muted uppercase tracking-widest">
          {{ isDragOver ? 'Drop files here' : 'Import or drag media' }}
        </span>
      </div>
    </div>
  </div>
</template>
