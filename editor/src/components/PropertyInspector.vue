<script setup lang="ts">
import { computed } from 'vue'
import { useSceneStore } from '../stores/scene'

const store = useSceneStore()

const layer = computed(() => store.selectedLayer)

function updateTransform(prop: string, value: number, index?: number) {
  if (!layer.value) return
  if (index !== undefined) {
    const arr = [...(layer.value.transform as any)[prop]]
    arr[index] = value
    store.updateLayerProperty(layer.value.id, `transform.${prop}`, arr)
  } else {
    store.updateLayerProperty(layer.value.id, `transform.${prop}`, value)
  }
}
</script>

<template>
  <div class="flex flex-col bg-cosmos-card overflow-hidden">
    <!-- Header -->
    <div class="h-10 flex items-center px-3 border-b border-cosmos-border">
      <span class="font-mono text-[10px] text-text-muted uppercase tracking-[0.2em]">Properties</span>
    </div>

    <!-- No Selection -->
    <div v-if="!layer" class="flex-1 flex items-center justify-center">
      <span class="font-mono text-xs text-text-muted uppercase tracking-widest">No layer selected</span>
    </div>

    <!-- Layer Properties -->
    <div v-else class="flex-1 overflow-y-auto p-3 space-y-4">
      <!-- Layer ID -->
      <div>
        <label class="font-mono text-[9px] text-text-muted uppercase tracking-[0.15em] block mb-1">ID</label>
        <div class="font-mono text-sm text-text-primary">{{ layer.id }}</div>
      </div>

      <!-- Type -->
      <div>
        <label class="font-mono text-[9px] text-text-muted uppercase tracking-[0.15em] block mb-1">Type</label>
        <div class="font-mono text-sm text-marble uppercase">{{ layer.type }}</div>
      </div>

      <!-- Transform Section -->
      <div class="border-t border-cosmos-border pt-3">
        <h3 class="font-serif text-sm text-varden mb-3">Transform</h3>

        <!-- Position -->
        <div class="mb-3">
          <label class="font-mono text-[9px] text-text-muted uppercase tracking-[0.15em] block mb-1">Position</label>
          <div class="flex gap-2">
            <div class="flex-1">
              <span class="font-mono text-[8px] text-text-muted">X</span>
              <input
                type="number"
                :value="Array.isArray(layer.transform.position) ? layer.transform.position[0] : 0"
                @input="updateTransform('position', Number(($event.target as HTMLInputElement).value), 0)"
                class="w-full bg-cosmos-deep border border-cosmos-border rounded px-2 py-1 font-mono text-xs text-text-primary focus:border-crimson outline-none"
              />
            </div>
            <div class="flex-1">
              <span class="font-mono text-[8px] text-text-muted">Y</span>
              <input
                type="number"
                :value="Array.isArray(layer.transform.position) ? layer.transform.position[1] : 0"
                @input="updateTransform('position', Number(($event.target as HTMLInputElement).value), 1)"
                class="w-full bg-cosmos-deep border border-cosmos-border rounded px-2 py-1 font-mono text-xs text-text-primary focus:border-crimson outline-none"
              />
            </div>
          </div>
        </div>

        <!-- Scale -->
        <div class="mb-3">
          <label class="font-mono text-[9px] text-text-muted uppercase tracking-[0.15em] block mb-1">Scale</label>
          <div class="flex gap-2">
            <div class="flex-1">
              <span class="font-mono text-[8px] text-text-muted">X</span>
              <input
                type="number"
                step="0.1"
                :value="layer.transform.scale?.[0] ?? 1"
                @input="updateTransform('scale', Number(($event.target as HTMLInputElement).value), 0)"
                class="w-full bg-cosmos-deep border border-cosmos-border rounded px-2 py-1 font-mono text-xs text-text-primary focus:border-crimson outline-none"
              />
            </div>
            <div class="flex-1">
              <span class="font-mono text-[8px] text-text-muted">Y</span>
              <input
                type="number"
                step="0.1"
                :value="layer.transform.scale?.[1] ?? 1"
                @input="updateTransform('scale', Number(($event.target as HTMLInputElement).value), 1)"
                class="w-full bg-cosmos-deep border border-cosmos-border rounded px-2 py-1 font-mono text-xs text-text-primary focus:border-crimson outline-none"
              />
            </div>
          </div>
        </div>

        <!-- Rotation -->
        <div class="mb-3">
          <label class="font-mono text-[9px] text-text-muted uppercase tracking-[0.15em] block mb-1">Rotation</label>
          <input
            type="number"
            :value="layer.transform.rotation ?? 0"
            @input="updateTransform('rotation', Number(($event.target as HTMLInputElement).value))"
            class="w-full bg-cosmos-deep border border-cosmos-border rounded px-2 py-1 font-mono text-xs text-text-primary focus:border-crimson outline-none"
          />
        </div>

        <!-- Opacity -->
        <div class="mb-3">
          <label class="font-mono text-[9px] text-text-muted uppercase tracking-[0.15em] block mb-1">Opacity</label>
          <div class="flex items-center gap-2">
            <input
              type="range"
              min="0"
              max="1"
              step="0.01"
              :value="layer.transform.opacity ?? 1"
              @input="updateTransform('opacity', Number(($event.target as HTMLInputElement).value))"
              class="flex-1 accent-crimson"
            />
            <span class="font-mono text-xs text-text-primary w-10 text-right tabular-nums">
              {{ Math.round((layer.transform.opacity ?? 1) * 100) }}%
            </span>
          </div>
        </div>
      </div>

      <!-- Timing Section -->
      <div class="border-t border-cosmos-border pt-3">
        <h3 class="font-serif text-sm text-varden mb-3">Timing</h3>
        <div class="flex gap-2">
          <div class="flex-1">
            <label class="font-mono text-[9px] text-text-muted uppercase tracking-[0.15em] block mb-1">In</label>
            <div class="font-mono text-xs text-text-primary bg-cosmos-deep border border-cosmos-border rounded px-2 py-1">
              {{ layer.in }}
            </div>
          </div>
          <div class="flex-1">
            <label class="font-mono text-[9px] text-text-muted uppercase tracking-[0.15em] block mb-1">Out</label>
            <div class="font-mono text-xs text-text-primary bg-cosmos-deep border border-cosmos-border rounded px-2 py-1">
              {{ layer.out }}
            </div>
          </div>
        </div>
      </div>

      <!-- Blend Mode (if available) -->
      <div v-if="layer.blend_mode !== undefined" class="border-t border-cosmos-border pt-3">
        <h3 class="font-serif text-sm text-varden mb-3">Compositing</h3>
        <label class="font-mono text-[9px] text-text-muted uppercase tracking-[0.15em] block mb-1">Blend Mode</label>
        <select
          :value="layer.blend_mode || 'normal'"
          @change="store.updateLayerProperty(layer.id, 'blend_mode', ($event.target as HTMLSelectElement).value)"
          class="w-full bg-cosmos-deep border border-cosmos-border rounded px-2 py-1 font-mono text-xs text-text-primary focus:border-crimson outline-none"
        >
          <option v-for="mode in ['normal','multiply','screen','overlay','darken','lighten','color_dodge','color_burn','hard_light','soft_light','difference','exclusion','add']" :key="mode" :value="mode">
            {{ mode.replace(/_/g, ' ') }}
          </option>
        </select>
      </div>
    </div>
  </div>
</template>
