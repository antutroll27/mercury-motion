import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

export interface Layer {
  id: string
  type: string
  in: number
  out: number
  transform: {
    position: [number, number]
    scale?: [number, number]
    rotation?: number
    opacity?: number
  }
  [key: string]: any
}

export interface Composition {
  layers: Layer[]
  sequence?: boolean
}

export interface SceneMeta {
  name: string
  width: number
  height: number
  fps: number
  duration: number
  root: string
  background: string
}

export interface Scene {
  version: string
  meta: SceneMeta
  compositions: Record<string, Composition>
  assets: { fonts: any[] }
}

export const useSceneStore = defineStore('scene', () => {
  const scene = ref<Scene>({
    version: '1.0',
    meta: {
      name: 'Untitled',
      width: 1920,
      height: 1080,
      fps: 30,
      duration: 90,
      root: 'main',
      background: '#003049',
    },
    compositions: {
      main: { layers: [] },
    },
    assets: { fonts: [] },
  })

  const currentFrame = ref(0)
  const selectedLayerId = ref<string | null>(null)
  const isPlaying = ref(false)
  const previewImage = ref<string | null>(null)

  const rootComposition = computed(() => scene.value.compositions[scene.value.meta.root])
  const layers = computed(() => rootComposition.value?.layers ?? [])
  const selectedLayer = computed(() =>
    layers.value.find(l => l.id === selectedLayerId.value) ?? null
  )
  const totalFrames = computed(() => scene.value.meta.duration)
  const currentTimecode = computed(() => {
    const fps = scene.value.meta.fps
    const totalSeconds = currentFrame.value / fps
    const mins = Math.floor(totalSeconds / 60)
    const secs = Math.floor(totalSeconds % 60)
    const frames = currentFrame.value % fps
    return `${String(mins).padStart(2, '0')}:${String(secs).padStart(2, '0')}:${String(frames).padStart(2, '0')}`
  })

  function toJson(): string {
    return JSON.stringify(scene.value, null, 2)
  }

  function fromJson(json: string) {
    scene.value = JSON.parse(json)
  }

  function addLayer(layer: Layer) {
    rootComposition.value.layers.push(layer)
  }

  function removeLayer(id: string) {
    const comp = rootComposition.value
    comp.layers = comp.layers.filter(l => l.id !== id)
    if (selectedLayerId.value === id) selectedLayerId.value = null
  }

  function selectLayer(id: string | null) {
    selectedLayerId.value = id
  }

  function setFrame(frame: number) {
    currentFrame.value = Math.max(0, Math.min(frame, totalFrames.value - 1))
  }

  function updateLayerProperty(layerId: string, path: string, value: any) {
    const layer = layers.value.find(l => l.id === layerId)
    if (!layer) return
    const keys = path.split('.')
    let obj: any = layer
    for (let i = 0; i < keys.length - 1; i++) {
      obj = obj[keys[i]]
    }
    obj[keys[keys.length - 1]] = value
  }

  return {
    scene, currentFrame, selectedLayerId, isPlaying, previewImage,
    rootComposition, layers, selectedLayer, totalFrames, currentTimecode,
    toJson, fromJson, addLayer, removeLayer, selectLayer, setFrame, updateLayerProperty,
  }
})
