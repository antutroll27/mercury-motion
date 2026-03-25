<script setup lang="ts">
import { computed } from 'vue'
import { useSceneStore } from '../stores/scene'

const store = useSceneStore()
const layer = computed(() => store.selectedLayer)

const maskTypes = [
  { type: 'rect', label: 'Rectangle' },
  { type: 'ellipse', label: 'Ellipse' },
  { type: 'path', label: 'Freeform Path' },
]

function addMask(type: string) {
  if (!layer.value) return
  const w = store.scene.meta.width
  const h = store.scene.meta.height
  const masks = layer.value.masks ? [...layer.value.masks] : []

  const mask: any = { mode: 'add', feather: 0, opacity: 1.0, inverted: false }
  switch (type) {
    case 'rect':
      mask.path = { type: 'rect', x: w * 0.1, y: h * 0.1, width: w * 0.8, height: h * 0.8, corner_radius: 0 }
      break
    case 'ellipse':
      mask.path = { type: 'ellipse', cx: w / 2, cy: h / 2, rx: w * 0.3, ry: h * 0.3 }
      break
    case 'path':
      mask.path = { type: 'path', points: [[w*0.2, h*0.2], [w*0.8, h*0.2], [w*0.8, h*0.8], [w*0.2, h*0.8]], closed: true }
      break
  }
  masks.push(mask)
  store.updateLayerProperty(layer.value.id, 'masks', masks)
}

function removeMask(index: number) {
  if (!layer.value || !layer.value.masks) return
  const masks = [...layer.value.masks]
  masks.splice(index, 1)
  store.updateLayerProperty(layer.value.id, 'masks', masks.length > 0 ? masks : undefined)
}

function updateMask(index: number, key: string, value: any) {
  if (!layer.value || !layer.value.masks) return
  const masks = [...layer.value.masks]
  masks[index] = { ...masks[index], [key]: value }
  store.updateLayerProperty(layer.value.id, 'masks', masks)
}
</script>

<template>
  <div v-if="layer" class="border-t border-cosmos-border pt-3">
    <div class="flex items-center justify-between mb-3">
      <h3 class="font-serif text-sm text-varden">Masks</h3>
      <div class="relative group">
        <button class="w-6 h-6 flex items-center justify-center text-text-muted hover:text-crimson text-lg">+</button>
        <div class="hidden group-hover:block absolute right-0 top-6 z-50 bg-cosmos-card border border-cosmos-border rounded shadow-xl py-1 w-40">
          <button
            v-for="mt in maskTypes"
            :key="mt.type"
            class="w-full text-left px-3 py-1.5 font-mono text-xs text-text-primary hover:bg-crimson/20 hover:text-crimson"
            @click="addMask(mt.type)"
          >
            {{ mt.label }}
          </button>
        </div>
      </div>
    </div>

    <div v-if="layer.masks && layer.masks.length > 0" class="space-y-2">
      <div
        v-for="(mask, idx) in layer.masks"
        :key="idx"
        class="bg-cosmos-deep border border-cosmos-border rounded p-2"
      >
        <div class="flex items-center justify-between mb-2">
          <span class="font-mono text-[10px] text-marble uppercase tracking-wider">
            {{ mask.path?.type || 'mask' }}
          </span>
          <button class="text-text-muted hover:text-crimson text-xs" @click="removeMask(idx as number)">x</button>
        </div>

        <!-- Mode -->
        <div class="flex items-center gap-2 mb-1.5">
          <label class="font-mono text-[8px] text-text-muted uppercase w-16">Mode</label>
          <select
            :value="mask.mode || 'add'"
            @change="updateMask(idx as number, 'mode', ($event.target as HTMLSelectElement).value)"
            class="flex-1 bg-cosmos-deep border border-cosmos-border rounded px-1.5 py-0.5 font-mono text-[11px] text-text-primary focus:border-crimson outline-none"
          >
            <option value="add">Add</option>
            <option value="subtract">Subtract</option>
            <option value="intersect">Intersect</option>
            <option value="difference">Difference</option>
          </select>
        </div>

        <!-- Feather -->
        <div class="flex items-center gap-2 mb-1.5">
          <label class="font-mono text-[8px] text-text-muted uppercase w-16">Feather</label>
          <input
            type="number"
            step="0.5"
            :value="mask.feather || 0"
            @input="updateMask(idx as number, 'feather', Number(($event.target as HTMLInputElement).value))"
            class="flex-1 bg-cosmos-deep border border-cosmos-border rounded px-1.5 py-0.5 font-mono text-[11px] text-text-primary focus:border-crimson outline-none"
          />
        </div>

        <!-- Opacity -->
        <div class="flex items-center gap-2">
          <label class="font-mono text-[8px] text-text-muted uppercase w-16">Opacity</label>
          <input
            type="range" min="0" max="1" step="0.01"
            :value="mask.opacity ?? 1"
            @input="updateMask(idx as number, 'opacity', Number(($event.target as HTMLInputElement).value))"
            class="flex-1 accent-crimson"
          />
          <span class="font-mono text-[10px] text-text-primary w-8 text-right">{{ Math.round((mask.opacity ?? 1) * 100) }}%</span>
        </div>
      </div>
    </div>
    <div v-else class="text-center py-3">
      <span class="font-mono text-[10px] text-text-muted uppercase tracking-widest">No masks</span>
    </div>
  </div>
</template>
