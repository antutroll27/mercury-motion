import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { renderFrame } from '../lib/tauri-commands'

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
  const dirty = ref(false)

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
    dirty.value = true
    schedulePreview()
  }

  // --- Preview rendering ---

  async function requestPreview() {
    try {
      const json = toJson()
      const dataUrl = await renderFrame(json, currentFrame.value)
      previewImage.value = dataUrl
      dirty.value = false
    } catch (e) {
      console.error('Preview render failed:', e)
    }
  }

  let previewTimeout: ReturnType<typeof setTimeout> | null = null

  function schedulePreview() {
    dirty.value = true
    if (previewTimeout) clearTimeout(previewTimeout)
    previewTimeout = setTimeout(() => requestPreview(), 300)
  }

  // --- Playback ---

  let playInterval: ReturnType<typeof setInterval> | null = null

  function play() {
    if (isPlaying.value) return
    isPlaying.value = true
    const fps = scene.value.meta.fps
    playInterval = setInterval(() => {
      if (currentFrame.value >= totalFrames.value - 1) {
        currentFrame.value = 0 // loop
      } else {
        currentFrame.value++
      }
      requestPreview()
    }, 1000 / fps)
  }

  function pause() {
    isPlaying.value = false
    if (playInterval) {
      clearInterval(playInterval)
      playInterval = null
    }
  }

  function togglePlayback() {
    if (isPlaying.value) pause()
    else play()
  }

  // --- Scene mutations ---

  function addLayer(layer: Layer) {
    rootComposition.value.layers.push(layer)
    schedulePreview()
  }

  function removeLayer(id: string) {
    const comp = rootComposition.value
    comp.layers = comp.layers.filter(l => l.id !== id)
    if (selectedLayerId.value === id) selectedLayerId.value = null
    schedulePreview()
  }

  function reorderLayer(fromIndex: number, toIndex: number) {
    const comp = rootComposition.value
    if (fromIndex < 0 || fromIndex >= comp.layers.length) return
    if (toIndex < 0 || toIndex >= comp.layers.length) return
    const [moved] = comp.layers.splice(fromIndex, 1)
    comp.layers.splice(toIndex, 0, moved)
    schedulePreview()
  }

  function selectLayer(id: string | null) {
    selectedLayerId.value = id
  }

  function setFrame(frame: number) {
    currentFrame.value = Math.max(0, Math.min(frame, totalFrames.value - 1))
    schedulePreview()
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
    schedulePreview()
  }

  // --- File I/O ---

  async function saveToFile(path: string) {
    try {
      // Use Tauri invoke directly to avoid requiring @tauri-apps/plugin-fs
      const { invoke } = await import('@tauri-apps/api/core')
      await invoke('plugin:fs|write_text_file', { path, contents: toJson() })
    } catch (e) {
      console.error('Save failed:', e)
      throw e
    }
  }

  async function loadFromFile(path: string) {
    try {
      // Use Tauri invoke directly to avoid requiring @tauri-apps/plugin-fs
      const { invoke } = await import('@tauri-apps/api/core')
      const json = await invoke<string>('plugin:fs|read_text_file', { path })
      fromJson(json)
    } catch (e) {
      console.error('Load failed:', e)
      throw e
    }
  }

  return {
    scene, currentFrame, selectedLayerId, isPlaying, previewImage, dirty,
    rootComposition, layers, selectedLayer, totalFrames, currentTimecode,
    toJson, fromJson,
    requestPreview, schedulePreview,
    play, pause, togglePlayback,
    addLayer, removeLayer, reorderLayer, selectLayer, setFrame, updateLayerProperty,
    saveToFile, loadFromFile,
  }
})
