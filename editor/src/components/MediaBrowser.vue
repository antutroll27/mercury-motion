<script setup lang="ts">
import { ref } from 'vue'
import { useSceneStore } from '../stores/scene'

const store = useSceneStore()
const assets = ref<{ name: string, path: string, type: string }[]>([])

async function importMedia() {
  try {
    // Try Tauri file dialog
    const { open } = await import('@tauri-apps/plugin-dialog')
    const selected = await open({
      multiple: true,
      filters: [
        { name: 'Media', extensions: ['png', 'jpg', 'jpeg', 'webp', 'mp4', 'webm', 'mp3', 'wav', 'flac'] },
        { name: 'Images', extensions: ['png', 'jpg', 'jpeg', 'webp'] },
        { name: 'Video', extensions: ['mp4', 'webm'] },
        { name: 'Audio', extensions: ['mp3', 'wav', 'flac', 'ogg'] },
      ]
    })
    if (selected) {
      const paths = Array.isArray(selected) ? selected : [selected]
      for (const path of paths) {
        const name = path.split(/[/\\]/).pop() || path
        const ext = name.split('.').pop()?.toLowerCase() || ''
        const type = ['mp4', 'webm'].includes(ext) ? 'video' :
                     ['mp3', 'wav', 'flac', 'ogg'].includes(ext) ? 'audio' : 'image'
        assets.value.push({ name, path, type })
      }
    }
  } catch {
    console.warn('File dialog not available — running in browser mode')
  }
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
</script>

<template>
  <div class="flex flex-col">
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
        v-for="asset in assets"
        :key="asset.path"
        class="flex items-center gap-2 px-3 py-2 border-b border-cosmos-border/50 hover:bg-cosmos-deep/50 cursor-pointer"
        @click="addToTimeline(asset)"
      >
        <span class="font-mono text-xs text-text-muted">
          {{ asset.type === 'image' ? '\u25FB' : asset.type === 'video' ? '\u25B6' : '\u266A' }}
        </span>
        <span class="font-sans text-xs text-text-primary truncate flex-1">{{ asset.name }}</span>
        <span class="font-mono text-[9px] text-text-muted uppercase">{{ asset.type }}</span>
      </div>

      <!-- Empty State -->
      <div v-if="assets.length === 0" class="flex flex-col items-center justify-center py-8 gap-2">
        <div class="w-10 h-10 rounded border border-dashed border-cosmos-border flex items-center justify-center">
          <span class="text-text-muted text-sm">^</span>
        </div>
        <span class="font-mono text-[10px] text-text-muted uppercase tracking-widest">Import media</span>
        <span class="font-sans text-[10px] text-text-muted">Images, videos, audio</span>
      </div>
    </div>
  </div>
</template>
