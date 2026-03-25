<script setup lang="ts">
import { ref } from 'vue'
import { useSceneStore } from '../stores/scene'

const store = useSceneStore()
const assets = ref<{ name: string, path: string, type: string }[]>([])
const fileInputRef = ref<HTMLInputElement | null>(null)

function getMediaType(ext: string): string {
  if (['mp4', 'webm', 'mov', 'avi'].includes(ext)) return 'video'
  if (['mp3', 'wav', 'flac', 'ogg', 'aac'].includes(ext)) return 'audio'
  return 'image'
}

async function importMedia() {
  // Try Tauri dialog first, fall back to HTML file input
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
        // Tauri 2.0 returns string paths
        const pathStr = typeof p === 'string' ? p : (p as any).path ?? String(p)
        const name = pathStr.split(/[/\\]/).pop() || pathStr
        const ext = name.split('.').pop()?.toLowerCase() || ''
        assets.value.push({ name, path: pathStr, type: getMediaType(ext) })
      }
    }
    return
  } catch {
    // Not in Tauri — fall through to HTML input
  }

  // Browser fallback: use hidden file input
  fileInputRef.value?.click()
}

function handleFileInput(event: Event) {
  const input = event.target as HTMLInputElement
  if (!input.files) return
  for (const file of Array.from(input.files)) {
    const ext = file.name.split('.').pop()?.toLowerCase() || ''
    // In browser mode, use object URL (won't work with mmot render, but shows in UI)
    assets.value.push({
      name: file.name,
      path: URL.createObjectURL(file),
      type: getMediaType(ext),
    })
  }
  input.value = '' // reset for re-import
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

  if (asset.type === 'video') {
    layer.trim_start = 0
  }
  if (asset.type === 'audio') {
    layer.volume = 1.0
  }

  store.addLayer(layer)
  store.selectLayer(id)
}

function removeAsset(index: number) {
  assets.value.splice(index, 1)
}
</script>

<template>
  <div class="flex flex-col h-full">
    <!-- Hidden file input for browser fallback -->
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
        class="flex items-center gap-2 px-3 py-2 border-b border-cosmos-border/50 hover:bg-cosmos-deep/50 group"
      >
        <!-- Type Icon -->
        <span class="font-mono text-xs text-text-muted">
          {{ asset.type === 'image' ? '◻' : asset.type === 'video' ? '▶' : '♪' }}
        </span>

        <!-- File Name (click to add) -->
        <span
          class="font-sans text-xs text-text-primary truncate flex-1 cursor-pointer hover:text-crimson transition-colors"
          @click="addToTimeline(asset)"
          :title="'Click to add to timeline: ' + asset.path"
        >
          {{ asset.name }}
        </span>

        <!-- Type Badge -->
        <span class="font-mono text-[9px] text-text-muted uppercase">{{ asset.type }}</span>

        <!-- Remove Button -->
        <button
          class="opacity-0 group-hover:opacity-100 text-text-muted hover:text-crimson text-xs transition-opacity"
          @click="removeAsset(idx)"
        >
          ×
        </button>
      </div>

      <!-- Empty State -->
      <div v-if="assets.length === 0" class="flex flex-col items-center justify-center py-8 gap-3">
        <button
          class="w-16 h-16 rounded-lg border-2 border-dashed border-cosmos-border hover:border-crimson flex items-center justify-center transition-colors group"
          @click="importMedia"
        >
          <span class="text-text-muted group-hover:text-crimson text-2xl transition-colors">+</span>
        </button>
        <span class="font-mono text-[10px] text-text-muted uppercase tracking-widest">Import media</span>
        <span class="font-sans text-[10px] text-text-muted">Images, videos, audio files</span>
      </div>
    </div>
  </div>
</template>
