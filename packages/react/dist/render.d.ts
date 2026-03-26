/**
 * @mmot/react/render — Execute the mmot CLI to render scenes to video.
 *
 * This module writes the scene to a temporary .mmot.json file and invokes
 * the `mmot render` CLI command to produce MP4, GIF, or WebM output.
 *
 * @packageDocumentation
 */
import { Scene } from './index.js';
/** Options for the render function. */
export interface RenderOptions {
    /** Output file path. Defaults to "output.mp4". */
    output?: string;
    /** Output format. Inferred from file extension if not specified. */
    format?: 'mp4' | 'gif' | 'webm';
    /** Encoding quality (0-100). Defaults to 80. */
    quality?: number;
    /** Print ffmpeg/encoder output to stderr. */
    verbose?: boolean;
    /** Path to the mmot binary. Defaults to "mmot" (resolved via PATH). */
    mmotBin?: string;
    /** Timeout in milliseconds. Defaults to 300000 (5 minutes). */
    timeout?: number;
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
export declare function render(scene: Scene, options?: RenderOptions): Promise<string>;
/**
 * Convenience function: render a scene to GIF.
 *
 * @param scene - The Scene object to render.
 * @param output - Output file path. Defaults to "output.gif".
 * @param options - Additional render options (format is set automatically).
 * @returns The output file path.
 */
export declare function renderGif(scene: Scene, output?: string, options?: Omit<RenderOptions, 'format'>): Promise<string>;
/**
 * Convenience function: render a scene to MP4.
 *
 * @param scene - The Scene object to render.
 * @param output - Output file path. Defaults to "output.mp4".
 * @param options - Additional render options (format is set automatically).
 * @returns The output file path.
 */
export declare function renderMp4(scene: Scene, output?: string, options?: Omit<RenderOptions, 'format'>): Promise<string>;
/**
 * Convenience function: render a scene to WebM.
 *
 * @param scene - The Scene object to render.
 * @param output - Output file path. Defaults to "output.webm".
 * @param options - Additional render options (format is set automatically).
 * @returns The output file path.
 */
export declare function renderWebm(scene: Scene, output?: string, options?: Omit<RenderOptions, 'format'>): Promise<string>;
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
export declare function exportJson(scene: Scene, outputPath: string): void;
