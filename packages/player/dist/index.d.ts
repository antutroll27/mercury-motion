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
export { renderFrame } from './renderer.js';
export { evaluateValue, evaluateVec2, evaluateColor } from './evaluator.js';
export interface PlayerOptions {
    /** The .mmot.json scene data (parsed JSON object). */
    scene: any;
    /** Canvas width in pixels. Falls back to scene.meta.width, then 1920. */
    width?: number;
    /** Canvas height in pixels. Falls back to scene.meta.height, then 1080. */
    height?: number;
    /** Playback framerate. Falls back to scene.meta.fps, then 30. */
    fps?: number;
    /** Start playing immediately. Default false. */
    autoplay?: boolean;
    /** Loop playback. Default false. */
    loop?: boolean;
    /** Show built-in controls (play/pause, scrubber, timecode). Default true. */
    controls?: boolean;
    /** Background color override for the player container. */
    background?: string;
    /** Called when the current frame changes. */
    onFrameChange?: (frame: number) => void;
    /** Called when playback starts. */
    onPlay?: () => void;
    /** Called when playback pauses. */
    onPause?: () => void;
    /** Called when playback reaches the end (and loop is false). */
    onEnd?: () => void;
}
export declare class MmotPlayer {
    private canvas;
    private ctx;
    private controlsEl;
    private scene;
    private frame;
    private playing;
    private looping;
    private fps;
    private animationId;
    private lastFrameTime;
    private container;
    private options;
    private playBtn;
    private scrubber;
    private timecodeEl;
    private frameCounterEl;
    constructor(selector: string | HTMLElement, options: PlayerOptions);
    /** Total number of frames in the scene. */
    get totalFrames(): number;
    /** Current frame number. */
    get currentFrame(): number;
    /** Whether the player is currently playing. */
    get isPlaying(): boolean;
    /** The underlying canvas element. */
    get canvasElement(): HTMLCanvasElement;
    /** Start playback from the current frame. */
    play(): void;
    /** Pause playback. */
    pause(): void;
    /** Toggle play/pause. */
    toggle(): void;
    /** Seek to a specific frame. */
    seekTo(frame: number): void;
    /** Replace the scene data and re-render. */
    setScene(scene: any): void;
    /** Remove the player from the DOM and clean up. */
    destroy(): void;
    private tick;
    private renderCurrentFrame;
    private createControls;
    private updateControls;
}
//# sourceMappingURL=index.d.ts.map