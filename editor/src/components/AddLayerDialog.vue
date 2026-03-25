<script setup lang="ts">
import { ref } from 'vue'
import { useSceneStore } from '../stores/scene'

const store = useSceneStore()
const showDialog = ref(false)

const layerTypes = [
  { type: 'solid', label: 'Solid Color', icon: '\u25A0', desc: 'Flat color fill' },
  { type: 'text', label: 'Text', icon: 'T', desc: 'Text with font styling' },
  { type: 'shape', label: 'Shape', icon: '\u25B3', desc: 'Rectangle, ellipse, polygon, line' },
  { type: 'gradient', label: 'Gradient', icon: '\u25D0', desc: 'Linear or radial gradient' },
  { type: 'image', label: 'Image', icon: '\u25FB', desc: 'PNG, JPEG, WebP' },
  { type: 'video', label: 'Video', icon: '\u25B6', desc: 'MP4, WebM video clip' },
  { type: 'audio', label: 'Audio', icon: '\u266A', desc: 'MP3, WAV, FLAC audio' },
  { type: 'null', label: 'Null Object', icon: '\u25CE', desc: 'Transform-only (for parenting)' },
  { type: 'composition', label: 'Composition', icon: '\u229E', desc: 'Nested composition' },
]

function addLayer(typeDef: typeof layerTypes[0]) {
  const id = `${typeDef.type}_${Date.now().toString(36)}`
  const w = store.scene.meta.width
  const h = store.scene.meta.height

  const base: any = {
    id,
    type: typeDef.type,
    in: 0,
    out: store.totalFrames,
    transform: { position: [w / 2, h / 2] },
  }

  switch (typeDef.type) {
    case 'solid':
      base.color = '#669BBC'
      break
    case 'text':
      base.text = 'New Text'
      base.font = { family: 'Inter', size: 48, weight: 400, color: '#F5F0E8' }
      base.align = 'center'
      break
    case 'shape':
      base.shape = { shape_type: 'rect', width: 200, height: 200, corner_radius: 8, fill: '#C1121F' }
      break
    case 'gradient':
      base.gradient = {
        gradient_type: 'linear',
        start: [0, 0], end: [1, 1],
        colors: [
          { offset: 0, color: '#003049' },
          { offset: 1, color: '#669BBC' },
        ]
      }
      break
    case 'image':
      base.src = ''
      break
    case 'video':
      base.src = ''
      base.trim_start = 0
      break
    case 'audio':
      base.src = ''
      base.volume = 1.0
      break
    case 'null':
      break
    case 'composition':
      base.composition_id = 'main'
      break
  }

  store.addLayer(base)
  store.selectLayer(id)
  showDialog.value = false
}

defineExpose({ showDialog })
</script>

<template>
  <!-- Trigger is handled by parent via ref -->
  <Teleport to="body">
    <div v-if="showDialog" class="fixed inset-0 z-50 flex items-center justify-center bg-black/60" @click.self="showDialog = false">
      <div class="bg-cosmos-card border border-cosmos-border rounded-lg shadow-2xl w-96 overflow-hidden">
        <!-- Header -->
        <div class="px-4 py-3 border-b border-cosmos-border flex items-center justify-between">
          <h2 class="font-serif text-base text-varden">Add Layer</h2>
          <button class="text-text-muted hover:text-crimson" @click="showDialog = false">x</button>
        </div>

        <!-- Layer Types Grid -->
        <div class="p-3 grid grid-cols-3 gap-2">
          <button
            v-for="lt in layerTypes"
            :key="lt.type"
            class="flex flex-col items-center gap-1.5 p-3 rounded border border-cosmos-border hover:border-crimson hover:bg-crimson/10 transition-colors group"
            @click="addLayer(lt)"
          >
            <span class="text-2xl text-text-muted group-hover:text-crimson transition-colors">{{ lt.icon }}</span>
            <span class="font-mono text-[10px] text-text-primary uppercase tracking-wider">{{ lt.label }}</span>
            <span class="font-sans text-[9px] text-text-muted text-center leading-tight">{{ lt.desc }}</span>
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>
