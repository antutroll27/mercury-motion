/**
 * @mmot/player — Embeddable web player for Mercury-Motion animations.
 *
 * Renders .mmot.json scenes in the browser using Canvas 2D.
 * Zero dependencies. Works in any browser.
 *
 * Usage:
 *   import { MmotPlayer } from '@mmot/player'
 *   const player = new MmotPlayer('#preview', { scene: myScene, autoplay: true })
 */

import { renderFrame } from './renderer.js'
export { renderFrame } from './renderer.js'
export { evaluateValue, evaluateVec2, evaluateColor } from './evaluator.js'

// ── Types ─────────────────────────────────────────────────────────────────────

export interface PlayerOptions {
  /** The .mmot.json scene data (parsed JSON object). */
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  scene: any
  /** Canvas width in pixels. Falls back to scene.meta.width, then 1920. */
  width?: number
  /** Canvas height in pixels. Falls back to scene.meta.height, then 1080. */
  height?: number
  /** Playback framerate. Falls back to scene.meta.fps, then 30. */
  fps?: number
  /** Start playing immediately. Default false. */
  autoplay?: boolean
  /** Loop playback. Default false. */
  loop?: boolean
  /** Show built-in controls (play/pause, scrubber, timecode). Default true. */
  controls?: boolean
  /** Background color override for the player container. */
  background?: string
  /** Called when the current frame changes. */
  onFrameChange?: (frame: number) => void
  /** Called when playback starts. */
  onPlay?: () => void
  /** Called when playback pauses. */
  onPause?: () => void
  /** Called when playback reaches the end (and loop is false). */
  onEnd?: () => void
}

// ── Player Class ──────────────────────────────────────────────────────────────

export class MmotPlayer {
  private canvas: HTMLCanvasElement
  private ctx: CanvasRenderingContext2D
  private controlsEl: HTMLDivElement | null = null
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  private scene: any
  private frame: number = 0
  private playing: boolean = false
  private looping: boolean
  private fps: number
  private animationId: number | null = null
  private lastFrameTime: number = 0
  private container: HTMLElement
  private options: PlayerOptions

  // Control element references
  private playBtn: HTMLButtonElement | null = null
  private scrubber: HTMLInputElement | null = null
  private timecodeEl: HTMLSpanElement | null = null
  private frameCounterEl: HTMLSpanElement | null = null

  constructor(selector: string | HTMLElement, options: PlayerOptions) {
    // Resolve container
    if (typeof selector === 'string') {
      const el = document.querySelector(selector)
      if (!el) throw new Error(`MmotPlayer: no element found for selector "${selector}"`)
      this.container = el as HTMLElement
    } else {
      this.container = selector
    }

    this.options = options
    this.scene = options.scene
    this.fps = options.fps || options.scene?.meta?.fps || 30
    this.looping = options.loop ?? false

    const width = options.width || options.scene?.meta?.width || 1920
    const height = options.height || options.scene?.meta?.height || 1080

    // Set up container styles
    this.container.style.position = this.container.style.position || 'relative'

    // Create canvas
    this.canvas = document.createElement('canvas')
    this.canvas.width = width
    this.canvas.height = height
    this.canvas.style.width = '100%'
    this.canvas.style.height = 'auto'
    this.canvas.style.display = 'block'
    this.canvas.style.background = options.background || '#000'
    this.canvas.style.borderRadius = options.controls !== false ? '4px 4px 0 0' : '4px'
    this.container.appendChild(this.canvas)

    const ctx = this.canvas.getContext('2d')
    if (!ctx) throw new Error('MmotPlayer: failed to get Canvas 2D context')
    this.ctx = ctx

    // Controls
    if (options.controls !== false) {
      this.createControls()
    }

    // Initial render
    this.renderCurrentFrame()

    // Autoplay
    if (options.autoplay) {
      this.play()
    }
  }

  // ── Public Properties ───────────────────────────────────────────────────────

  /** Total number of frames in the scene. */
  get totalFrames(): number {
    return this.scene?.meta?.duration || 0
  }

  /** Current frame number. */
  get currentFrame(): number {
    return this.frame
  }

  /** Whether the player is currently playing. */
  get isPlaying(): boolean {
    return this.playing
  }

  /** The underlying canvas element. */
  get canvasElement(): HTMLCanvasElement {
    return this.canvas
  }

  // ── Public Methods ──────────────────────────────────────────────────────────

  /** Start playback from the current frame. */
  play(): void {
    if (this.playing) return
    // If at the end, restart from beginning
    if (this.frame >= this.totalFrames - 1) {
      this.frame = 0
    }
    this.playing = true
    this.lastFrameTime = performance.now()
    this.animationId = requestAnimationFrame((t) => this.tick(t))
    this.options.onPlay?.()
    this.updateControls()
  }

  /** Pause playback. */
  pause(): void {
    if (!this.playing) return
    this.playing = false
    if (this.animationId !== null) {
      cancelAnimationFrame(this.animationId)
      this.animationId = null
    }
    this.options.onPause?.()
    this.updateControls()
  }

  /** Toggle play/pause. */
  toggle(): void {
    if (this.playing) {
      this.pause()
    } else {
      this.play()
    }
  }

  /** Seek to a specific frame. */
  seekTo(frame: number): void {
    this.frame = Math.max(0, Math.min(frame, Math.max(0, this.totalFrames - 1)))
    this.renderCurrentFrame()
    this.options.onFrameChange?.(this.frame)
    this.updateControls()
  }

  /** Replace the scene data and re-render. */
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  setScene(scene: any): void {
    this.scene = scene
    this.fps = this.options.fps || scene?.meta?.fps || 30
    this.frame = 0

    // Update canvas dimensions if they differ
    const width = this.options.width || scene?.meta?.width || 1920
    const height = this.options.height || scene?.meta?.height || 1080
    if (this.canvas.width !== width || this.canvas.height !== height) {
      this.canvas.width = width
      this.canvas.height = height
    }

    // Update scrubber max
    if (this.scrubber) {
      this.scrubber.max = String(Math.max(0, this.totalFrames - 1))
    }

    this.renderCurrentFrame()
    this.updateControls()
  }

  /** Remove the player from the DOM and clean up. */
  destroy(): void {
    this.pause()
    if (this.controlsEl) {
      this.controlsEl.remove()
      this.controlsEl = null
    }
    this.canvas.remove()
    this.playBtn = null
    this.scrubber = null
    this.timecodeEl = null
    this.frameCounterEl = null
  }

  // ── Private: Animation Loop ─────────────────────────────────────────────────

  private tick(timestamp: number): void {
    if (!this.playing) return

    const elapsed = timestamp - this.lastFrameTime
    const frameDuration = 1000 / this.fps

    if (elapsed >= frameDuration) {
      // Advance frame (handle multiple frames if rendering is slow)
      const framesToAdvance = Math.floor(elapsed / frameDuration)
      this.lastFrameTime += framesToAdvance * frameDuration
      this.frame += framesToAdvance

      if (this.frame >= this.totalFrames) {
        if (this.looping) {
          this.frame = this.frame % this.totalFrames
        } else {
          this.frame = Math.max(0, this.totalFrames - 1)
          this.renderCurrentFrame()
          this.updateControls()
          this.pause()
          this.options.onEnd?.()
          return
        }
      }

      this.renderCurrentFrame()
      this.options.onFrameChange?.(this.frame)
      this.updateControls()
    }

    this.animationId = requestAnimationFrame((t) => this.tick(t))
  }

  private renderCurrentFrame(): void {
    renderFrame(this.ctx, this.scene, this.frame, this.canvas.width, this.canvas.height)
  }

  // ── Private: Controls ───────────────────────────────────────────────────────

  private createControls(): void {
    this.controlsEl = document.createElement('div')
    this.controlsEl.style.cssText = [
      'display: flex',
      'align-items: center',
      'gap: 8px',
      'padding: 8px 12px',
      'background: #0a0a1a',
      'border-radius: 0 0 4px 4px',
      'font-family: "JetBrains Mono", "Fira Code", "Cascadia Code", monospace',
      'font-size: 11px',
      'color: #f5f0e8',
      'user-select: none',
    ].join('; ')

    // Play/Pause button
    this.playBtn = document.createElement('button')
    this.playBtn.innerHTML = playIcon()
    this.playBtn.title = 'Play/Pause'
    this.playBtn.style.cssText = [
      'background: none',
      'border: 1px solid #1a2a3a',
      'color: #f5f0e8',
      'width: 28px',
      'height: 28px',
      'border-radius: 50%',
      'cursor: pointer',
      'display: flex',
      'align-items: center',
      'justify-content: center',
      'padding: 0',
      'flex-shrink: 0',
      'transition: border-color 0.15s, background 0.15s',
    ].join('; ')
    this.playBtn.addEventListener('mouseenter', () => {
      if (this.playBtn) this.playBtn.style.borderColor = '#c1121f'
    })
    this.playBtn.addEventListener('mouseleave', () => {
      if (this.playBtn) this.playBtn.style.borderColor = '#1a2a3a'
    })
    this.playBtn.addEventListener('click', () => this.toggle())
    this.controlsEl.appendChild(this.playBtn)

    // Frame counter
    this.frameCounterEl = document.createElement('span')
    this.frameCounterEl.style.cssText = [
      'color: #8a9aaa',
      'min-width: 32px',
      'text-align: center',
      'font-size: 10px',
      'flex-shrink: 0',
    ].join('; ')
    this.frameCounterEl.textContent = '0'
    this.controlsEl.appendChild(this.frameCounterEl)

    // Scrubber
    this.scrubber = document.createElement('input')
    this.scrubber.type = 'range'
    this.scrubber.min = '0'
    this.scrubber.max = String(Math.max(0, this.totalFrames - 1))
    this.scrubber.value = '0'
    this.scrubber.style.cssText = [
      'flex: 1',
      'accent-color: #c1121f',
      'cursor: pointer',
      'height: 4px',
    ].join('; ')
    this.scrubber.addEventListener('input', () => {
      this.seekTo(Number(this.scrubber!.value))
    })
    // Pause while scrubbing for smoother interaction
    this.scrubber.addEventListener('mousedown', () => {
      if (this.playing) {
        this.pause()
        this.scrubber!.dataset.wasPlaying = 'true'
      }
    })
    this.scrubber.addEventListener('mouseup', () => {
      if (this.scrubber!.dataset.wasPlaying === 'true') {
        delete this.scrubber!.dataset.wasPlaying
        this.play()
      }
    })
    this.controlsEl.appendChild(this.scrubber)

    // Timecode display
    this.timecodeEl = document.createElement('span')
    this.timecodeEl.style.cssText = [
      'color: #5a7a90',
      'letter-spacing: 0.1em',
      'min-width: 68px',
      'text-align: right',
      'flex-shrink: 0',
    ].join('; ')
    this.timecodeEl.textContent = '00:00:00'
    this.controlsEl.appendChild(this.timecodeEl)

    this.container.appendChild(this.controlsEl)
  }

  private updateControls(): void {
    if (!this.controlsEl) return

    // Play/Pause icon
    if (this.playBtn) {
      this.playBtn.innerHTML = this.playing ? pauseIcon() : playIcon()
    }

    // Scrubber position
    if (this.scrubber) {
      this.scrubber.value = String(this.frame)
    }

    // Frame counter
    if (this.frameCounterEl) {
      this.frameCounterEl.textContent = String(this.frame)
    }

    // Timecode (MM:SS:FF)
    if (this.timecodeEl) {
      const totalSecs = this.frame / this.fps
      const mins = Math.floor(totalSecs / 60)
      const secs = Math.floor(totalSecs % 60)
      const frames = this.frame % this.fps
      this.timecodeEl.textContent = `${pad2(mins)}:${pad2(secs)}:${pad2(frames)}`
    }
  }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

function pad2(n: number): string {
  return String(n).padStart(2, '0')
}

function playIcon(): string {
  return '<svg width="12" height="12" viewBox="0 0 12 12" fill="currentColor"><polygon points="2,0 12,6 2,12"/></svg>'
}

function pauseIcon(): string {
  return '<svg width="12" height="12" viewBox="0 0 12 12" fill="currentColor"><rect x="1" y="0" width="3.5" height="12"/><rect x="7.5" y="0" width="3.5" height="12"/></svg>'
}
