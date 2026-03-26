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
import { renderFrame } from './renderer.js';
export { renderFrame } from './renderer.js';
export { evaluateValue, evaluateVec2, evaluateColor } from './evaluator.js';
// ── Player Class ──────────────────────────────────────────────────────────────
export class MmotPlayer {
    canvas;
    ctx;
    controlsEl = null;
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    scene;
    frame = 0;
    playing = false;
    looping;
    fps;
    animationId = null;
    lastFrameTime = 0;
    container;
    options;
    // Control element references
    playBtn = null;
    scrubber = null;
    timecodeEl = null;
    frameCounterEl = null;
    constructor(selector, options) {
        // Resolve container
        if (typeof selector === 'string') {
            const el = document.querySelector(selector);
            if (!el)
                throw new Error(`MmotPlayer: no element found for selector "${selector}"`);
            this.container = el;
        }
        else {
            this.container = selector;
        }
        this.options = options;
        this.scene = options.scene;
        this.fps = options.fps || options.scene?.meta?.fps || 30;
        this.looping = options.loop ?? false;
        const width = options.width || options.scene?.meta?.width || 1920;
        const height = options.height || options.scene?.meta?.height || 1080;
        // Set up container styles
        this.container.style.position = this.container.style.position || 'relative';
        // Create canvas
        this.canvas = document.createElement('canvas');
        this.canvas.width = width;
        this.canvas.height = height;
        this.canvas.style.width = '100%';
        this.canvas.style.height = 'auto';
        this.canvas.style.display = 'block';
        this.canvas.style.background = options.background || '#000';
        this.canvas.style.borderRadius = options.controls !== false ? '4px 4px 0 0' : '4px';
        this.container.appendChild(this.canvas);
        const ctx = this.canvas.getContext('2d');
        if (!ctx)
            throw new Error('MmotPlayer: failed to get Canvas 2D context');
        this.ctx = ctx;
        // Controls
        if (options.controls !== false) {
            this.createControls();
        }
        // Initial render
        this.renderCurrentFrame();
        // Autoplay
        if (options.autoplay) {
            this.play();
        }
    }
    // ── Public Properties ───────────────────────────────────────────────────────
    /** Total number of frames in the scene. */
    get totalFrames() {
        return this.scene?.meta?.duration || 0;
    }
    /** Current frame number. */
    get currentFrame() {
        return this.frame;
    }
    /** Whether the player is currently playing. */
    get isPlaying() {
        return this.playing;
    }
    /** The underlying canvas element. */
    get canvasElement() {
        return this.canvas;
    }
    // ── Public Methods ──────────────────────────────────────────────────────────
    /** Start playback from the current frame. */
    play() {
        if (this.playing)
            return;
        // If at the end, restart from beginning
        if (this.frame >= this.totalFrames - 1) {
            this.frame = 0;
        }
        this.playing = true;
        this.lastFrameTime = performance.now();
        this.animationId = requestAnimationFrame((t) => this.tick(t));
        this.options.onPlay?.();
        this.updateControls();
    }
    /** Pause playback. */
    pause() {
        if (!this.playing)
            return;
        this.playing = false;
        if (this.animationId !== null) {
            cancelAnimationFrame(this.animationId);
            this.animationId = null;
        }
        this.options.onPause?.();
        this.updateControls();
    }
    /** Toggle play/pause. */
    toggle() {
        if (this.playing) {
            this.pause();
        }
        else {
            this.play();
        }
    }
    /** Seek to a specific frame. */
    seekTo(frame) {
        this.frame = Math.max(0, Math.min(frame, Math.max(0, this.totalFrames - 1)));
        this.renderCurrentFrame();
        this.options.onFrameChange?.(this.frame);
        this.updateControls();
    }
    /** Replace the scene data and re-render. */
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    setScene(scene) {
        this.scene = scene;
        this.fps = this.options.fps || scene?.meta?.fps || 30;
        this.frame = 0;
        // Update canvas dimensions if they differ
        const width = this.options.width || scene?.meta?.width || 1920;
        const height = this.options.height || scene?.meta?.height || 1080;
        if (this.canvas.width !== width || this.canvas.height !== height) {
            this.canvas.width = width;
            this.canvas.height = height;
        }
        // Update scrubber max
        if (this.scrubber) {
            this.scrubber.max = String(Math.max(0, this.totalFrames - 1));
        }
        this.renderCurrentFrame();
        this.updateControls();
    }
    /** Remove the player from the DOM and clean up. */
    destroy() {
        this.pause();
        if (this.controlsEl) {
            this.controlsEl.remove();
            this.controlsEl = null;
        }
        this.canvas.remove();
        this.playBtn = null;
        this.scrubber = null;
        this.timecodeEl = null;
        this.frameCounterEl = null;
    }
    // ── Private: Animation Loop ─────────────────────────────────────────────────
    tick(timestamp) {
        if (!this.playing)
            return;
        const elapsed = timestamp - this.lastFrameTime;
        const frameDuration = 1000 / this.fps;
        if (elapsed >= frameDuration) {
            // Advance frame (handle multiple frames if rendering is slow)
            const framesToAdvance = Math.floor(elapsed / frameDuration);
            this.lastFrameTime += framesToAdvance * frameDuration;
            this.frame += framesToAdvance;
            if (this.frame >= this.totalFrames) {
                if (this.looping) {
                    this.frame = this.frame % this.totalFrames;
                }
                else {
                    this.frame = Math.max(0, this.totalFrames - 1);
                    this.renderCurrentFrame();
                    this.updateControls();
                    this.pause();
                    this.options.onEnd?.();
                    return;
                }
            }
            this.renderCurrentFrame();
            this.options.onFrameChange?.(this.frame);
            this.updateControls();
        }
        this.animationId = requestAnimationFrame((t) => this.tick(t));
    }
    renderCurrentFrame() {
        renderFrame(this.ctx, this.scene, this.frame, this.canvas.width, this.canvas.height);
    }
    // ── Private: Controls ───────────────────────────────────────────────────────
    createControls() {
        this.controlsEl = document.createElement('div');
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
        ].join('; ');
        // Play/Pause button
        this.playBtn = document.createElement('button');
        this.playBtn.innerHTML = playIcon();
        this.playBtn.title = 'Play/Pause';
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
        ].join('; ');
        this.playBtn.addEventListener('mouseenter', () => {
            if (this.playBtn)
                this.playBtn.style.borderColor = '#c1121f';
        });
        this.playBtn.addEventListener('mouseleave', () => {
            if (this.playBtn)
                this.playBtn.style.borderColor = '#1a2a3a';
        });
        this.playBtn.addEventListener('click', () => this.toggle());
        this.controlsEl.appendChild(this.playBtn);
        // Frame counter
        this.frameCounterEl = document.createElement('span');
        this.frameCounterEl.style.cssText = [
            'color: #8a9aaa',
            'min-width: 32px',
            'text-align: center',
            'font-size: 10px',
            'flex-shrink: 0',
        ].join('; ');
        this.frameCounterEl.textContent = '0';
        this.controlsEl.appendChild(this.frameCounterEl);
        // Scrubber
        this.scrubber = document.createElement('input');
        this.scrubber.type = 'range';
        this.scrubber.min = '0';
        this.scrubber.max = String(Math.max(0, this.totalFrames - 1));
        this.scrubber.value = '0';
        this.scrubber.style.cssText = [
            'flex: 1',
            'accent-color: #c1121f',
            'cursor: pointer',
            'height: 4px',
        ].join('; ');
        this.scrubber.addEventListener('input', () => {
            this.seekTo(Number(this.scrubber.value));
        });
        // Pause while scrubbing for smoother interaction
        this.scrubber.addEventListener('mousedown', () => {
            if (this.playing) {
                this.pause();
                this.scrubber.dataset.wasPlaying = 'true';
            }
        });
        this.scrubber.addEventListener('mouseup', () => {
            if (this.scrubber.dataset.wasPlaying === 'true') {
                delete this.scrubber.dataset.wasPlaying;
                this.play();
            }
        });
        this.controlsEl.appendChild(this.scrubber);
        // Timecode display
        this.timecodeEl = document.createElement('span');
        this.timecodeEl.style.cssText = [
            'color: #5a7a90',
            'letter-spacing: 0.1em',
            'min-width: 68px',
            'text-align: right',
            'flex-shrink: 0',
        ].join('; ');
        this.timecodeEl.textContent = '00:00:00';
        this.controlsEl.appendChild(this.timecodeEl);
        this.container.appendChild(this.controlsEl);
    }
    updateControls() {
        if (!this.controlsEl)
            return;
        // Play/Pause icon
        if (this.playBtn) {
            this.playBtn.innerHTML = this.playing ? pauseIcon() : playIcon();
        }
        // Scrubber position
        if (this.scrubber) {
            this.scrubber.value = String(this.frame);
        }
        // Frame counter
        if (this.frameCounterEl) {
            this.frameCounterEl.textContent = String(this.frame);
        }
        // Timecode (MM:SS:FF)
        if (this.timecodeEl) {
            const totalSecs = this.frame / this.fps;
            const mins = Math.floor(totalSecs / 60);
            const secs = Math.floor(totalSecs % 60);
            const frames = this.frame % this.fps;
            this.timecodeEl.textContent = `${pad2(mins)}:${pad2(secs)}:${pad2(frames)}`;
        }
    }
}
// ── Helpers ───────────────────────────────────────────────────────────────────
function pad2(n) {
    return String(n).padStart(2, '0');
}
function playIcon() {
    return '<svg width="12" height="12" viewBox="0 0 12 12" fill="currentColor"><polygon points="2,0 12,6 2,12"/></svg>';
}
function pauseIcon() {
    return '<svg width="12" height="12" viewBox="0 0 12 12" fill="currentColor"><rect x="1" y="0" width="3.5" height="12"/><rect x="7.5" y="0" width="3.5" height="12"/></svg>';
}
//# sourceMappingURL=index.js.map