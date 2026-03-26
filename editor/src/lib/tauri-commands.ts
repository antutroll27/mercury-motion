export interface SceneInfo {
  valid: boolean
  width?: number
  height?: number
  fps?: number
  duration_frames?: number
  duration_secs?: number
  composition_count?: number
  root_layer_count?: number
  error?: string
}

// Detect if running inside Tauri
const isTauri = typeof window !== 'undefined' && !!(window as any).__TAURI_INTERNALS__

async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (!isTauri) {
    throw new Error(`Tauri not available (browser mode) — ${cmd} skipped`)
  }
  const { invoke: tauriInvoke } = await import('@tauri-apps/api/core')
  return tauriInvoke<T>(cmd, args)
}

export async function validateScene(json: string): Promise<SceneInfo> {
  return invoke('validate_scene', { json })
}

export async function renderFrame(json: string, frame: number): Promise<string> {
  return invoke('render_frame', { json, frame })
}

export async function renderToFile(
  json: string,
  outputPath: string,
  format: string = 'mp4',
  quality: number = 80
): Promise<string> {
  return invoke('render_to_file', { json, outputPath, format, quality })
}

export async function setScene(json: string): Promise<void> {
  return invoke('set_scene', { json })
}

export async function getScene(): Promise<string | null> {
  return invoke('get_scene')
}

export async function getSchema(): Promise<string> {
  return invoke('get_schema')
}
