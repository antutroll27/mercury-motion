<script setup lang="ts">
import { computed } from 'vue'
import { useSceneStore } from '../stores/scene'

const store = useSceneStore()
const layer = computed(() => store.selectedLayer)

const availableEffects = [
  { type: 'gaussian_blur', label: 'Gaussian Blur', defaults: { radius: 5 } },
  { type: 'drop_shadow', label: 'Drop Shadow', defaults: { color: '#000000', offset_x: 4, offset_y: 4, blur: 8, opacity: 0.6 } },
  { type: 'glow', label: 'Glow', defaults: { color: '#ffffff', radius: 10, intensity: 0.8 } },
  { type: 'brightness_contrast', label: 'Brightness / Contrast', defaults: { brightness: 0, contrast: 0 } },
  { type: 'hue_saturation', label: 'Hue / Saturation', defaults: { hue: 0, saturation: 0, lightness: 0 } },
  { type: 'invert', label: 'Invert', defaults: {} },
  { type: 'tint', label: 'Tint', defaults: { color: '#669BBC', amount: 0.5 } },
  { type: 'fill', label: 'Fill', defaults: { color: '#C1121F', opacity: 1.0 } },
]

function addEffect(effectDef: typeof availableEffects[0]) {
  if (!layer.value) return
  const effects = layer.value.effects ? [...layer.value.effects] : []
  effects.push({ type: effectDef.type, ...effectDef.defaults })
  store.updateLayerProperty(layer.value.id, 'effects', effects)
}

function removeEffect(index: number) {
  if (!layer.value || !layer.value.effects) return
  const effects = [...layer.value.effects]
  effects.splice(index, 1)
  store.updateLayerProperty(layer.value.id, 'effects', effects.length > 0 ? effects : undefined)
}

function updateEffect(index: number, key: string, value: any) {
  if (!layer.value || !layer.value.effects) return
  const effects = [...layer.value.effects]
  effects[index] = { ...effects[index], [key]: value }
  store.updateLayerProperty(layer.value.id, 'effects', effects)
}
</script>

<template>
  <div v-if="layer" class="border-t border-cosmos-border pt-3">
    <div class="flex items-center justify-between mb-3">
      <h3 class="font-serif text-sm text-varden">Effects</h3>
      <!-- Add Effect Dropdown -->
      <div class="relative group">
        <button class="w-6 h-6 flex items-center justify-center text-text-muted hover:text-crimson text-lg">+</button>
        <div class="hidden group-hover:block absolute right-0 top-6 z-50 bg-cosmos-card border border-cosmos-border rounded shadow-xl py-1 w-48">
          <button
            v-for="effect in availableEffects"
            :key="effect.type"
            class="w-full text-left px-3 py-1.5 font-mono text-xs text-text-primary hover:bg-crimson/20 hover:text-crimson transition-colors"
            @click="addEffect(effect)"
          >
            {{ effect.label }}
          </button>
        </div>
      </div>
    </div>

    <!-- Active Effects -->
    <div v-if="layer.effects && layer.effects.length > 0" class="space-y-2">
      <div
        v-for="(effect, idx) in layer.effects"
        :key="idx"
        class="bg-cosmos-deep border border-cosmos-border rounded p-2"
      >
        <div class="flex items-center justify-between mb-2">
          <span class="font-mono text-[10px] text-marble uppercase tracking-wider">
            {{ effect.type.replace(/_/g, ' ') }}
          </span>
          <button
            class="text-text-muted hover:text-crimson text-xs"
            @click="removeEffect(idx as number)"
          >x</button>
        </div>

        <!-- Effect Parameters -->
        <div class="space-y-1.5">
          <template v-for="(value, key) in effect" :key="key">
            <div v-if="key !== 'type'" class="flex items-center gap-2">
              <label class="font-mono text-[8px] text-text-muted uppercase w-16 truncate">{{ key }}</label>
              <input
                v-if="typeof value === 'number'"
                type="number"
                step="0.1"
                :value="value"
                @input="updateEffect(idx as number, String(key), Number(($event.target as HTMLInputElement).value))"
                class="flex-1 bg-cosmos-deep border border-cosmos-border rounded px-1.5 py-0.5 font-mono text-[11px] text-text-primary focus:border-crimson outline-none"
              />
              <input
                v-else-if="typeof value === 'string'"
                type="text"
                :value="value"
                @input="updateEffect(idx as number, String(key), ($event.target as HTMLInputElement).value)"
                class="flex-1 bg-cosmos-deep border border-cosmos-border rounded px-1.5 py-0.5 font-mono text-[11px] text-text-primary focus:border-crimson outline-none"
              />
            </div>
          </template>
        </div>
      </div>
    </div>

    <div v-else class="text-center py-4">
      <span class="font-mono text-[10px] text-text-muted uppercase tracking-widest">No effects</span>
    </div>
  </div>
</template>
