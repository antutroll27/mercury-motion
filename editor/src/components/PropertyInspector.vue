<script setup lang="ts">
import { computed, ref } from 'vue'
import { useSceneStore } from '../stores/scene'
// Components used in template below
// @ts-ignore TS6133 — vue-tsc does not detect template usage
import EffectsPanel from './EffectsPanel.vue'
// @ts-ignore TS6133 — vue-tsc does not detect template usage
import MaskEditor from './MaskEditor.vue'

const store = useSceneStore()
const fileInputRef = ref<HTMLInputElement | null>(null)

const layer = computed(() => store.selectedLayer)

const hasSource = computed(() => {
  if (!layer.value) return false
  return ['image', 'video', 'audio'].includes(layer.value.type)
})

const sourcePath = computed(() => {
  if (!layer.value) return ''
  return (layer.value as any).src || ''
})

const sourceFileName = computed(() => {
  const src = sourcePath.value
  if (!src) return 'No file selected'
  return src.split(/[/\\]/).pop() || src
})

async function browseFile() {
  if (!layer.value) return
  try {
    const { open } = await import('@tauri-apps/plugin-dialog')
    const filters = layer.value.type === 'audio'
      ? [{ name: 'Audio', extensions: ['mp3', 'wav', 'flac', 'ogg', 'aac'] }]
      : layer.value.type === 'video'
      ? [{ name: 'Video', extensions: ['mp4', 'webm', 'mov'] }]
      : [{ name: 'Images', extensions: ['png', 'jpg', 'jpeg', 'webp', 'gif'] }]
    const selected = await open({ multiple: false, filters })
    if (selected) {
      const path = typeof selected === 'string' ? selected : (selected as any).path ?? String(selected)
      store.updateLayerProperty(layer.value.id, 'src', path)
    }
    return
  } catch {
    // Not in Tauri — use HTML file input
  }
  fileInputRef.value?.click()
}

function handleFileInput(event: Event) {
  const input = event.target as HTMLInputElement
  if (!input.files?.length || !layer.value) return
  const file = input.files[0]
  const path = (file as any).path || URL.createObjectURL(file)
  store.updateLayerProperty(layer.value.id, 'src', path)
  input.value = ''
}

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

      <!-- Source File (for image/video/audio layers) -->
      <div v-if="hasSource" class="border-t border-cosmos-border pt-3">
        <h3 class="font-mono text-[11px] text-varden uppercase tracking-widest mb-3">Source</h3>
        <input
          ref="fileInputRef"
          type="file"
          :accept="layer.type === 'audio' ? 'audio/*' : layer.type === 'video' ? 'video/*' : 'image/*'"
          class="hidden"
          @change="handleFileInput"
        />
        <div class="flex items-center gap-2">
          <div class="flex-1 bg-cosmos-deep border border-cosmos-border rounded px-2 py-1.5 font-mono text-[11px] truncate"
               :class="sourcePath ? 'text-text-primary' : 'text-text-muted'">
            {{ sourceFileName }}
          </div>
          <button
            class="px-3 py-1.5 bg-cosmos-deep border border-cosmos-border rounded font-mono text-[10px] text-text-muted uppercase tracking-wider hover:border-crimson hover:text-crimson transition-colors"
            @click="browseFile"
          >
            Browse
          </button>
        </div>
        <div v-if="sourcePath" class="mt-1">
          <span class="font-mono text-[9px] text-text-muted break-all">{{ sourcePath }}</span>
        </div>
      </div>

      <!-- Color (for solid layers) -->
      <div v-if="layer.type === 'solid'" class="border-t border-cosmos-border pt-3">
        <h3 class="font-mono text-[11px] text-varden uppercase tracking-widest mb-3">Color</h3>
        <div class="flex items-center gap-2">
          <input
            type="color"
            :value="(layer as any).color || '#000000'"
            @input="store.updateLayerProperty(layer.id, 'color', ($event.target as HTMLInputElement).value)"
            class="w-8 h-8 rounded border border-cosmos-border cursor-pointer"
          />
          <input
            type="text"
            :value="(layer as any).color || '#000000'"
            @input="store.updateLayerProperty(layer.id, 'color', ($event.target as HTMLInputElement).value)"
            class="flex-1 bg-cosmos-deep border border-cosmos-border rounded px-2 py-1 font-mono text-xs text-text-primary focus:border-crimson outline-none"
          />
        </div>
      </div>

      <!-- Text Content (for text layers) -->
      <div v-if="layer.type === 'text'" class="border-t border-cosmos-border pt-3">
        <h3 class="font-mono text-[11px] text-varden uppercase tracking-widest mb-3">Text</h3>
        <textarea
          :value="(layer as any).text || ''"
          @input="store.updateLayerProperty(layer.id, 'text', ($event.target as HTMLTextAreaElement).value)"
          rows="3"
          class="w-full bg-cosmos-deep border border-cosmos-border rounded px-2 py-1 font-sans text-xs text-text-primary focus:border-crimson outline-none resize-none"
        />
        <div class="mt-2 flex gap-2">
          <div class="flex-1">
            <label class="font-mono text-[8px] text-text-muted uppercase">Font Size</label>
            <input
              type="number"
              :value="(layer as any).font?.size ?? 48"
              @input="store.updateLayerProperty(layer.id, 'font.size', Number(($event.target as HTMLInputElement).value))"
              class="w-full bg-cosmos-deep border border-cosmos-border rounded px-2 py-1 font-mono text-xs text-text-primary focus:border-crimson outline-none"
            />
          </div>
          <div class="flex-1">
            <label class="font-mono text-[8px] text-text-muted uppercase">Weight</label>
            <select
              :value="(layer as any).font?.weight ?? 400"
              @change="store.updateLayerProperty(layer.id, 'font.weight', Number(($event.target as HTMLSelectElement).value))"
              class="w-full bg-cosmos-deep border border-cosmos-border rounded px-2 py-1 font-mono text-xs text-text-primary focus:border-crimson outline-none"
            >
              <option :value="300">Light</option>
              <option :value="400">Regular</option>
              <option :value="500">Medium</option>
              <option :value="600">Semibold</option>
              <option :value="700">Bold</option>
              <option :value="900">Black</option>
            </select>
          </div>
        </div>
        <!-- Font Family -->
        <div class="mt-2">
          <label class="font-mono text-[8px] text-text-muted uppercase">Font Family</label>
          <select
            :value="(layer as any).font?.family ?? 'Inter'"
            @change="store.updateLayerProperty(layer.id, 'font.family', ($event.target as HTMLSelectElement).value)"
            class="w-full bg-cosmos-deep border border-cosmos-border rounded px-2 py-1 font-mono text-xs text-text-primary focus:border-crimson outline-none"
          >
            <option value="Inter">Inter</option>
            <option value="Arial">Arial</option>
            <option value="Helvetica">Helvetica</option>
            <option value="Georgia">Georgia</option>
            <option value="Times New Roman">Times New Roman</option>
            <option value="Courier New">Courier New</option>
            <option value="JetBrains Mono">JetBrains Mono</option>
            <option value="Playfair Display">Playfair Display</option>
          </select>
        </div>
        <div class="mt-2">
          <label class="font-mono text-[8px] text-text-muted uppercase">Color</label>
          <div class="flex items-center gap-2 mt-0.5">
            <input
              type="color"
              :value="(layer as any).font?.color || '#ffffff'"
              @input="store.updateLayerProperty(layer.id, 'font.color', ($event.target as HTMLInputElement).value)"
              class="w-6 h-6 rounded border border-cosmos-border cursor-pointer"
            />
            <input
              type="text"
              :value="(layer as any).font?.color || '#ffffff'"
              @input="store.updateLayerProperty(layer.id, 'font.color', ($event.target as HTMLInputElement).value)"
              class="flex-1 bg-cosmos-deep border border-cosmos-border rounded px-2 py-1 font-mono text-xs text-text-primary focus:border-crimson outline-none"
            />
          </div>
        </div>
        <!-- Text Alignment -->
        <div class="mt-2">
          <label class="font-mono text-[8px] text-text-muted uppercase">Alignment</label>
          <div class="flex gap-1 mt-1">
            <button
              v-for="align in ['left', 'center', 'right']"
              :key="align"
              class="flex-1 py-1 font-mono text-[10px] uppercase rounded border transition-colors"
              :class="(layer as any).align === align || (!((layer as any).align) && align === 'center')
                ? 'bg-crimson/20 border-crimson text-crimson'
                : 'border-cosmos-border text-text-muted hover:border-crimson hover:text-crimson'"
              @click="store.updateLayerProperty(layer.id, 'align', align)"
            >
              {{ align }}
            </button>
          </div>
        </div>
      </div>

      <!-- Shape Properties (for shape layers) -->
      <div v-if="layer.type === 'shape' && (layer as any).shape" class="border-t border-cosmos-border pt-3">
        <h3 class="font-mono text-[11px] text-varden uppercase tracking-widest mb-3">Shape</h3>

        <!-- Shape Type -->
        <div class="mb-2">
          <label class="font-mono text-[8px] text-text-muted uppercase">Type</label>
          <div class="font-mono text-xs text-marble uppercase">{{ (layer as any).shape.shape_type }}</div>
        </div>

        <!-- Width/Height for rect and ellipse -->
        <template v-if="['rect', 'ellipse'].includes((layer as any).shape.shape_type)">
          <div class="flex gap-2 mb-2">
            <div class="flex-1">
              <label class="font-mono text-[8px] text-text-muted uppercase">Width</label>
              <input
                type="number"
                :value="(layer as any).shape.width || 100"
                @input="store.updateLayerProperty(layer.id, 'shape.width', Number(($event.target as HTMLInputElement).value))"
                class="w-full bg-cosmos-deep border border-cosmos-border rounded px-2 py-1 font-mono text-xs text-text-primary focus:border-crimson outline-none"
              />
            </div>
            <div class="flex-1">
              <label class="font-mono text-[8px] text-text-muted uppercase">Height</label>
              <input
                type="number"
                :value="(layer as any).shape.height || 100"
                @input="store.updateLayerProperty(layer.id, 'shape.height', Number(($event.target as HTMLInputElement).value))"
                class="w-full bg-cosmos-deep border border-cosmos-border rounded px-2 py-1 font-mono text-xs text-text-primary focus:border-crimson outline-none"
              />
            </div>
          </div>
        </template>

        <!-- Corner Radius for rect -->
        <div v-if="(layer as any).shape.shape_type === 'rect'" class="mb-2">
          <div class="flex items-center justify-between mb-1">
            <label class="font-mono text-[8px] text-text-muted uppercase">Corner Radius</label>
            <input
              type="number"
              :value="(layer as any).shape.corner_radius || 0"
              @input="store.updateLayerProperty(layer.id, 'shape.corner_radius', Number(($event.target as HTMLInputElement).value))"
              class="w-14 bg-cosmos-deep border border-cosmos-border rounded px-1.5 py-0.5 font-mono text-[11px] text-text-primary focus:border-crimson outline-none text-right"
            />
          </div>
          <input
            type="range" min="0" max="100" step="1"
            :value="(layer as any).shape.corner_radius || 0"
            @input="store.updateLayerProperty(layer.id, 'shape.corner_radius', Number(($event.target as HTMLInputElement).value))"
            class="w-full accent-crimson h-1.5"
          />
        </div>

        <!-- Fill Color -->
        <div class="mb-2">
          <label class="font-mono text-[8px] text-text-muted uppercase">Fill</label>
          <div class="flex items-center gap-2 mt-0.5">
            <input
              type="color"
              :value="(layer as any).shape.fill || '#ffffff'"
              @input="store.updateLayerProperty(layer.id, 'shape.fill', ($event.target as HTMLInputElement).value)"
              class="w-6 h-6 rounded border border-cosmos-border cursor-pointer"
            />
            <input
              type="text"
              :value="(layer as any).shape.fill || '#ffffff'"
              @input="store.updateLayerProperty(layer.id, 'shape.fill', ($event.target as HTMLInputElement).value)"
              class="flex-1 bg-cosmos-deep border border-cosmos-border rounded px-2 py-1 font-mono text-xs text-text-primary focus:border-crimson outline-none"
            />
          </div>
        </div>

        <!-- Stroke -->
        <div class="mb-2">
          <label class="font-mono text-[8px] text-text-muted uppercase">Stroke Color</label>
          <div class="flex items-center gap-2 mt-0.5">
            <input
              type="color"
              :value="(layer as any).shape?.stroke?.color || '#000000'"
              @input="store.updateLayerProperty(layer.id, 'shape.stroke.color', ($event.target as HTMLInputElement).value)"
              class="w-6 h-6 rounded border border-cosmos-border cursor-pointer"
            />
            <input
              type="number"
              step="0.5"
              :value="(layer as any).shape?.stroke?.width || 0"
              @input="store.updateLayerProperty(layer.id, 'shape.stroke.width', Number(($event.target as HTMLInputElement).value))"
              class="w-14 bg-cosmos-deep border border-cosmos-border rounded px-1.5 py-0.5 font-mono text-[11px] text-text-primary focus:border-crimson outline-none text-right"
              placeholder="Width"
            />
          </div>
        </div>
      </div>

      <!-- Fill Mode (for solid/gradient layers) -->
      <div v-if="['solid', 'gradient'].includes(layer.type)" class="mb-2">
        <label class="font-mono text-[8px] text-text-muted uppercase block mb-1">Fill Mode</label>
        <button
          class="px-3 py-1 font-mono text-[10px] uppercase rounded border transition-colors"
          :class="(layer as any).fill === 'parent'
            ? 'bg-crimson/20 border-crimson text-crimson'
            : 'border-cosmos-border text-text-muted hover:border-crimson'"
          @click="store.updateLayerProperty(layer.id, 'fill', (layer as any).fill === 'parent' ? undefined : 'parent')"
        >
          {{ (layer as any).fill === 'parent' ? 'Fill Parent (ON)' : 'Fill Parent (OFF)' }}
        </button>
      </div>

      <!-- Transform Section -->
      <div class="border-t border-cosmos-border pt-3">
        <h3 class="font-mono text-[11px] text-varden uppercase tracking-widest mb-3">Transform</h3>

        <!-- Position X -->
        <div class="mb-2">
          <div class="flex items-center justify-between mb-1">
            <label class="font-mono text-[9px] text-text-muted uppercase tracking-[0.15em]">Position X</label>
            <input
              type="number"
              :value="Array.isArray(layer.transform.position) ? layer.transform.position[0] : 0"
              @input="updateTransform('position', Number(($event.target as HTMLInputElement).value), 0)"
              class="w-16 bg-cosmos-deep border border-cosmos-border rounded px-1.5 py-0.5 font-mono text-[11px] text-text-primary focus:border-crimson outline-none text-right tabular-nums"
            />
          </div>
          <input
            type="range"
            min="0"
            :max="store.scene.meta.width"
            :value="Array.isArray(layer.transform.position) ? layer.transform.position[0] : 0"
            @input="updateTransform('position', Number(($event.target as HTMLInputElement).value), 0)"
            class="w-full accent-crimson h-1.5"
          />
        </div>

        <!-- Position Y -->
        <div class="mb-3">
          <div class="flex items-center justify-between mb-1">
            <label class="font-mono text-[9px] text-text-muted uppercase tracking-[0.15em]">Position Y</label>
            <input
              type="number"
              :value="Array.isArray(layer.transform.position) ? layer.transform.position[1] : 0"
              @input="updateTransform('position', Number(($event.target as HTMLInputElement).value), 1)"
              class="w-16 bg-cosmos-deep border border-cosmos-border rounded px-1.5 py-0.5 font-mono text-[11px] text-text-primary focus:border-crimson outline-none text-right tabular-nums"
            />
          </div>
          <input
            type="range"
            min="0"
            :max="store.scene.meta.height"
            :value="Array.isArray(layer.transform.position) ? layer.transform.position[1] : 0"
            @input="updateTransform('position', Number(($event.target as HTMLInputElement).value), 1)"
            class="w-full accent-crimson h-1.5"
          />
        </div>

        <!-- Scale X -->
        <div class="mb-2">
          <div class="flex items-center justify-between mb-1">
            <label class="font-mono text-[9px] text-text-muted uppercase tracking-[0.15em]">Scale X</label>
            <input
              type="number"
              step="0.01"
              :value="layer.transform.scale?.[0] ?? 1"
              @input="updateTransform('scale', Number(($event.target as HTMLInputElement).value), 0)"
              class="w-16 bg-cosmos-deep border border-cosmos-border rounded px-1.5 py-0.5 font-mono text-[11px] text-text-primary focus:border-crimson outline-none text-right tabular-nums"
            />
          </div>
          <input
            type="range"
            min="0"
            max="5"
            step="0.01"
            :value="layer.transform.scale?.[0] ?? 1"
            @input="updateTransform('scale', Number(($event.target as HTMLInputElement).value), 0)"
            class="w-full accent-crimson h-1.5"
          />
        </div>

        <!-- Scale Y -->
        <div class="mb-3">
          <div class="flex items-center justify-between mb-1">
            <label class="font-mono text-[9px] text-text-muted uppercase tracking-[0.15em]">Scale Y</label>
            <input
              type="number"
              step="0.01"
              :value="layer.transform.scale?.[1] ?? 1"
              @input="updateTransform('scale', Number(($event.target as HTMLInputElement).value), 1)"
              class="w-16 bg-cosmos-deep border border-cosmos-border rounded px-1.5 py-0.5 font-mono text-[11px] text-text-primary focus:border-crimson outline-none text-right tabular-nums"
            />
          </div>
          <input
            type="range"
            min="0"
            max="5"
            step="0.01"
            :value="layer.transform.scale?.[1] ?? 1"
            @input="updateTransform('scale', Number(($event.target as HTMLInputElement).value), 1)"
            class="w-full accent-crimson h-1.5"
          />
        </div>

        <!-- Rotation -->
        <div class="mb-3">
          <div class="flex items-center justify-between mb-1">
            <label class="font-mono text-[9px] text-text-muted uppercase tracking-[0.15em]">Rotation</label>
            <div class="flex items-center gap-1">
              <input
                type="number"
                :value="layer.transform.rotation ?? 0"
                @input="updateTransform('rotation', Number(($event.target as HTMLInputElement).value))"
                class="w-16 bg-cosmos-deep border border-cosmos-border rounded px-1.5 py-0.5 font-mono text-[11px] text-text-primary focus:border-crimson outline-none text-right tabular-nums"
              />
              <span class="font-mono text-[9px] text-text-muted">deg</span>
            </div>
          </div>
          <input
            type="range"
            min="-360"
            max="360"
            step="1"
            :value="layer.transform.rotation ?? 0"
            @input="updateTransform('rotation', Number(($event.target as HTMLInputElement).value))"
            class="w-full accent-crimson h-1.5"
          />
        </div>

        <!-- Opacity -->
        <div class="mb-3">
          <div class="flex items-center justify-between mb-1">
            <label class="font-mono text-[9px] text-text-muted uppercase tracking-[0.15em]">Opacity</label>
            <span class="font-mono text-[11px] text-text-primary tabular-nums">
              {{ Math.round((layer.transform.opacity ?? 1) * 100) }}%
            </span>
          </div>
        </div>
      </div>

      <!-- Timing Section -->
      <div class="border-t border-cosmos-border pt-3">
        <h3 class="font-mono text-[11px] text-varden uppercase tracking-widest mb-3">Timing</h3>
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

      <!-- Blend Mode -->
      <div class="border-t border-cosmos-border pt-3">
        <h3 class="font-mono text-[11px] text-varden uppercase tracking-widest mb-3">Compositing</h3>
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

      <!-- Effects Panel -->
      <EffectsPanel />

      <!-- Mask Editor -->
      <MaskEditor />
    </div>
  </div>
</template>
