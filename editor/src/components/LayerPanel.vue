<script setup lang="ts">
import { ref } from 'vue'
import { useSceneStore } from '../stores/scene'
import AddLayerDialog from './AddLayerDialog.vue'

const store = useSceneStore()

const addLayerDialogRef = ref<InstanceType<typeof AddLayerDialog> | null>(null)

const layerTypeIcons: Record<string, string> = {
  solid: '\u25A0',
  text: 'T',
  image: '\u25FB',
  video: '\u25B6',
  shape: '\u25B3',
  gradient: '\u25D0',
  null: '\u25CE',
  audio: '\u266A',
  composition: '\u229E',
}

function openAddLayer() {
  if (addLayerDialogRef.value) {
    addLayerDialogRef.value.showDialog = true
  }
}
</script>

<template>
  <div class="flex flex-col bg-cosmos-card overflow-hidden">
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
        v-for="layer in [...store.layers].reverse()"
        :key="layer.id"
        class="flex items-center gap-2 px-3 py-2 cursor-pointer border-b border-cosmos-border/50 transition-colors"
        :class="[
          layer.id === store.selectedLayerId
            ? 'bg-crimson/10 border-l-2 border-l-crimson'
            : 'hover:bg-cosmos/50 border-l-2 border-l-transparent'
        ]"
        @click="store.selectLayer(layer.id)"
      >
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
      </div>

      <!-- Empty State -->
      <div v-if="store.layers.length === 0" class="flex flex-col items-center justify-center py-12 gap-2">
        <div class="w-10 h-10 rounded-full border border-dashed border-cosmos-border flex items-center justify-center">
          <span class="text-text-muted text-lg">+</span>
        </div>
        <span class="font-mono text-[10px] text-text-muted uppercase tracking-widest">Add a layer</span>
      </div>
    </div>

    <AddLayerDialog ref="addLayerDialogRef" />
  </div>
</template>
