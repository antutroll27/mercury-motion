/**
 * @mmot/react/render — Execute the mmot CLI to render scenes to video.
 *
 * This module writes the scene to a temporary .mmot.json file and invokes
 * the `mmot render` CLI command to produce MP4, GIF, or WebM output.
 *
 * @packageDocumentation
 */

import { execSync } from 'child_process'
import { writeFileSync, mkdtempSync, unlinkSync, rmSync } from 'fs'
import { join } from 'path'
import { tmpdir } from 'os'
import { Scene } from './index.js'

/** Options for the render function. */
export interface RenderOptions {
  /** Output file path. Defaults to "output.mp4". */
  output?: string
  /** Output format. Inferred from file extension if not specified. */
  format?: 'mp4' | 'gif' | 'webm'
  /** Encoding quality (0-100). Defaults to 80. */
  quality?: number
  /** Print ffmpeg/encoder output to stderr. */
  verbose?: boolean
  /** Path to the mmot binary. Defaults to "mmot" (resolved via PATH). */
  mmotBin?: string
  /** Timeout in milliseconds. Defaults to 300000 (5 minutes). */
  timeout?: number
}

/**
 * Render a Scene to a video file using the mmot CLI.
 *
 * Writes the scene to a temporary .mmot.json file, invokes `mmot render`,
 * and returns the output file path on success.
 *
 * @param scene - The Scene object to render.
 * @param options - Render configuration.
 * @returns The absolute path to the rendered output file.
 * @throws If the mmot CLI is not found or rendering fails.
 *
 * @example
 * ```ts
 * import { Scene } from '@mmot/react'
 * import { render } from '@mmot/react/render'
 *
 * const scene = new Scene({ width: 1920, height: 1080, fps: 30, durationInFrames: 90 })
 * scene.addSolid('bg', { color: '#0a0a1a', fill: 'parent' })
 *
 * const outputPath = await render(scene, { output: 'my-video.mp4' })
 * console.log(`Rendered to ${outputPath}`)
 * ```
 */
export async function render(scene: Scene, options?: RenderOptions): Promise<string> {
  const output = options?.output ?? 'output.mp4'
  const format = options?.format ?? inferFormat(output)
  const quality = options?.quality ?? 80
  const bin = options?.mmotBin ?? 'mmot'
  const timeout = options?.timeout ?? 300_000

  // Write scene to temp file
  const tmpDir = mkdtempSync(join(tmpdir(), 'mmot-'))
  const tmpFile = join(tmpDir, 'scene.mmot.json')
  writeFileSync(tmpFile, scene.toString(), 'utf-8')

  try {
    const args = [
      'render',
      `"${tmpFile}"`,
      '--output', `"${output}"`,
      '--format', format,
      '--quality', String(quality),
    ]
    if (options?.verbose) {
      args.push('--verbose')
    }

    const cmd = `${bin} ${args.join(' ')}`
    execSync(cmd, {
      stdio: options?.verbose ? 'inherit' : 'pipe',
      timeout,
    })

    return output
  } finally {
    // Clean up temp files
    try { unlinkSync(tmpFile) } catch { /* ignore */ }
    try { rmSync(tmpDir, { recursive: true }) } catch { /* ignore */ }
  }
}

/**
 * Convenience function: render a scene to GIF.
 *
 * @param scene - The Scene object to render.
 * @param output - Output file path. Defaults to "output.gif".
 * @param options - Additional render options (format is set automatically).
 * @returns The output file path.
 */
export async function renderGif(
  scene: Scene,
  output?: string,
  options?: Omit<RenderOptions, 'format'>
): Promise<string> {
  return render(scene, { ...options, output: output ?? 'output.gif', format: 'gif' })
}

/**
 * Convenience function: render a scene to MP4.
 *
 * @param scene - The Scene object to render.
 * @param output - Output file path. Defaults to "output.mp4".
 * @param options - Additional render options (format is set automatically).
 * @returns The output file path.
 */
export async function renderMp4(
  scene: Scene,
  output?: string,
  options?: Omit<RenderOptions, 'format'>
): Promise<string> {
  return render(scene, { ...options, output: output ?? 'output.mp4', format: 'mp4' })
}

/**
 * Convenience function: render a scene to WebM.
 *
 * @param scene - The Scene object to render.
 * @param output - Output file path. Defaults to "output.webm".
 * @param options - Additional render options (format is set automatically).
 * @returns The output file path.
 */
export async function renderWebm(
  scene: Scene,
  output?: string,
  options?: Omit<RenderOptions, 'format'>
): Promise<string> {
  return render(scene, { ...options, output: output ?? 'output.webm', format: 'webm' })
}

/**
 * Export a scene to a .mmot.json file without rendering.
 *
 * Useful for debugging, piping to other tools, or manual rendering later.
 *
 * @param scene - The Scene object to export.
 * @param outputPath - Path to write the .mmot.json file.
 *
 * @example
 * ```ts
 * exportJson(scene, './my-scene.mmot.json')
 * // Then render manually: mmot render my-scene.mmot.json --output video.mp4
 * ```
 */
export function exportJson(scene: Scene, outputPath: string): void {
  writeFileSync(outputPath, scene.toString(), 'utf-8')
}

/** Infer output format from file extension. */
function inferFormat(path: string): 'mp4' | 'gif' | 'webm' {
  if (path.endsWith('.gif')) return 'gif'
  if (path.endsWith('.webm')) return 'webm'
  return 'mp4'
}
