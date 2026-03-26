/**
 * @mmot/react — Write React. Render natively.
 *
 * Remotion-compatible builder API that serializes to .mmot.json
 * for Mercury-Motion's native Rust renderer.
 *
 * @packageDocumentation
 */
/** Configuration for a video composition. */
export interface CompositionConfig {
    /** Canvas width in pixels. */
    width: number;
    /** Canvas height in pixels. */
    height: number;
    /** Frames per second. */
    fps: number;
    /** Total duration in frames. */
    durationInFrames: number;
    /** Background color (hex). Defaults to "#000000". */
    background?: string;
    /** Human-readable name for the composition. */
    name?: string;
}
/** Font specification for text layers. */
export interface FontConfig {
    /** Font family name (e.g. "Inter", "Arial"). */
    family: string;
    /** Font size in pixels. Defaults to 32. */
    size?: number;
    /** Font weight (100-900). Defaults to 400. */
    weight?: number;
    /** Text color (hex). Defaults to "#ffffff". */
    color?: string;
}
/** Stroke specification for shapes. */
export interface StrokeConfig {
    /** Stroke color (hex). */
    color: string;
    /** Stroke width in pixels. */
    width: number;
}
/** A single keyframe for animation. */
export interface Keyframe<T> {
    /** Frame number (0-based). */
    frame: number;
    /** Value at this keyframe. */
    value: T;
    /** Easing curve from this keyframe to the next. Ignored on the final keyframe. */
    easing?: EasingValue;
}
/** Easing value — either a named preset or a structured easing definition. */
export type EasingValue = 'linear' | 'ease_in' | 'ease_out' | 'ease_in_out' | {
    type: 'cubic_bezier';
    x1: number;
    y1: number;
    x2: number;
    y2: number;
} | {
    type: 'spring';
    stiffness: number;
    damping: number;
    mass: number;
};
/** Transform properties for a layer. Each property can be static or animated. */
export interface TransformConfig {
    /** Position [x, y] in pixels, or keyframed. */
    position?: [number, number] | Keyframe<[number, number]>[];
    /** Scale [x, y] as multipliers (1.0 = 100%), or keyframed. */
    scale?: [number, number] | Keyframe<[number, number]>[];
    /** Rotation in degrees, or keyframed. */
    rotation?: number | Keyframe<number>[];
    /** Opacity from 0.0 to 1.0, or keyframed. */
    opacity?: number | Keyframe<number>[];
}
/** Shape specification for shape layers. */
export type ShapeConfig = {
    shape_type: 'rect';
    width: number;
    height: number;
    corner_radius?: number;
    fill?: string;
    stroke?: StrokeConfig;
} | {
    shape_type: 'ellipse';
    width: number;
    height: number;
    fill?: string;
    stroke?: StrokeConfig;
} | {
    shape_type: 'line';
    x1: number;
    y1: number;
    x2: number;
    y2: number;
    stroke: StrokeConfig;
} | {
    shape_type: 'polygon';
    points: [number, number][];
    fill?: string;
    stroke?: StrokeConfig;
};
/** Gradient color stop. */
export interface GradientStop {
    /** Position along the gradient (0.0 to 1.0). */
    offset: number;
    /** Color at this stop (hex). */
    color: string;
}
/** Gradient specification — linear or radial. */
export type GradientConfig = {
    gradient_type: 'linear';
    colors: GradientStop[];
    start: [number, number];
    end: [number, number];
} | {
    gradient_type: 'radial';
    colors: GradientStop[];
    center: [number, number];
    radius: number;
};
/** Transition between sequence layers. */
export type TransitionConfig = {
    type: 'crossfade';
    duration: number;
} | {
    type: 'wipe';
    duration: number;
    direction: 'left' | 'right' | 'up' | 'down';
} | {
    type: 'slide';
    duration: number;
    direction: 'left' | 'right' | 'up' | 'down';
};
/** Blend mode for compositing layers. */
export type BlendMode = 'normal' | 'multiply' | 'screen' | 'overlay' | 'darken' | 'lighten' | 'color_dodge' | 'color_burn' | 'hard_light' | 'soft_light' | 'difference' | 'exclusion' | 'add';
/** An effect applied to a layer. */
export type EffectConfig = {
    type: 'gaussian_blur';
    radius: number;
} | {
    type: 'drop_shadow';
    color: string;
    offset_x: number;
    offset_y: number;
    blur: number;
    opacity?: number;
} | {
    type: 'glow';
    color: string;
    radius: number;
    intensity?: number;
} | {
    type: 'brightness_contrast';
    brightness?: number;
    contrast?: number;
} | {
    type: 'hue_saturation';
    hue?: number;
    saturation?: number;
    lightness?: number;
} | {
    type: 'invert';
} | {
    type: 'tint';
    color: string;
    amount?: number;
} | {
    type: 'fill';
    color: string;
    opacity?: number;
};
/**
 * Builds a complete .mmot.json scene programmatically.
 *
 * @example
 * ```ts
 * const scene = new Scene({ width: 1920, height: 1080, fps: 30, durationInFrames: 90 })
 * scene.addSolid('bg', { color: '#0a0a1a', fill: 'parent' })
 * scene.addText('title', {
 *   text: 'Hello World',
 *   font: { family: 'Inter', size: 72, weight: 700, color: '#ffffff' },
 *   transform: { position: [960, 540], opacity: interpolate(0, [0, 30], [0, 1]) }
 * })
 * console.log(scene.toString())
 * ```
 */
export declare class Scene {
    private config;
    private layers;
    private isSequence;
    private sequenceTransition?;
    constructor(config: CompositionConfig);
    /**
     * Add a solid color layer.
     *
     * @param id - Unique layer identifier.
     * @param opts - Layer options including color, timing, transform, and effects.
     */
    addSolid(id: string, opts: {
        color: string;
        in?: number;
        out?: number;
        transform?: TransformConfig;
        effects?: EffectConfig[];
        blendMode?: BlendMode;
        fill?: 'parent';
        parent?: string;
        masks?: unknown[];
        adjustment?: boolean;
    }): this;
    /**
     * Add a text layer.
     *
     * @param id - Unique layer identifier.
     * @param opts - Layer options including text content, font, alignment, and transform.
     */
    addText(id: string, opts: {
        text: string;
        font?: FontConfig;
        align?: 'left' | 'center' | 'right';
        in?: number;
        out?: number;
        transform?: TransformConfig;
        effects?: EffectConfig[];
        blendMode?: BlendMode;
        fill?: 'parent';
    }): this;
    /**
     * Add a shape layer (rect, ellipse, line, or polygon).
     *
     * @param id - Unique layer identifier.
     * @param opts - Layer options including shape spec, transform, and effects.
     */
    addShape(id: string, opts: {
        shape: ShapeConfig;
        in?: number;
        out?: number;
        transform?: TransformConfig;
        effects?: EffectConfig[];
        blendMode?: BlendMode;
        fill?: 'parent';
        parent?: string;
        motionBlur?: boolean;
        pathAnimation?: {
            points: [number, number][];
            autoOrient?: boolean;
        };
        trimPaths?: {
            start?: number;
            end?: number;
            offset?: number;
        };
    }): this;
    /**
     * Add a gradient layer (linear or radial).
     *
     * @param id - Unique layer identifier.
     * @param opts - Layer options including gradient spec and transform.
     */
    addGradient(id: string, opts: {
        gradient: GradientConfig;
        in?: number;
        out?: number;
        transform?: TransformConfig;
        effects?: EffectConfig[];
        blendMode?: BlendMode;
        fill?: 'parent';
    }): this;
    /**
     * Add an image layer.
     *
     * @param id - Unique layer identifier.
     * @param opts - Layer options including source path and transform.
     */
    addImage(id: string, opts: {
        src: string;
        in?: number;
        out?: number;
        transform?: TransformConfig;
        effects?: EffectConfig[];
        blendMode?: BlendMode;
        fill?: 'parent';
    }): this;
    /**
     * Add a video layer.
     *
     * @param id - Unique layer identifier.
     * @param opts - Layer options including source path, trim points, and transform.
     */
    addVideo(id: string, opts: {
        src: string;
        in?: number;
        out?: number;
        trimStart?: number;
        trimEnd?: number;
        transform?: TransformConfig;
        effects?: EffectConfig[];
        blendMode?: BlendMode;
        fill?: 'parent';
    }): this;
    /**
     * Add an audio layer.
     *
     * @param id - Unique layer identifier.
     * @param opts - Layer options including source path and volume.
     */
    addAudio(id: string, opts: {
        src: string;
        in?: number;
        out?: number;
        volume?: number;
    }): this;
    /**
     * Add a null (invisible) layer, used for parenting transforms.
     *
     * @param id - Unique layer identifier.
     * @param opts - Layer options including transform and timing.
     */
    addNull(id: string, opts?: {
        in?: number;
        out?: number;
        transform?: TransformConfig;
    }): this;
    /**
     * Add a composition reference layer.
     *
     * @param id - Unique layer identifier.
     * @param opts - Layer options including the target composition ID.
     */
    addComposition(id: string, opts: {
        compositionId: string;
        in?: number;
        out?: number;
        transform?: TransformConfig;
        effects?: EffectConfig[];
    }): this;
    /**
     * Enable sequence mode: layers play back-to-back instead of simultaneously.
     *
     * @param transition - Optional transition between consecutive layers.
     */
    setSequence(transition?: TransitionConfig): this;
    /**
     * Serialize the scene to a plain object matching the .mmot.json format.
     *
     * @returns The complete scene object ready for JSON.stringify.
     */
    toJSON(): Record<string, unknown>;
    /**
     * Serialize the scene to a JSON string.
     *
     * @returns Formatted JSON string of the .mmot.json scene.
     */
    toString(): string;
}
/**
 * Convert an array of Keyframe objects to .mmot.json keyframe format.
 *
 * Use this when you need to pass animated values to a transform property.
 *
 * @param kfs - Array of keyframes with frame, value, and optional easing.
 * @returns The keyframes formatted for use in TransformConfig properties.
 *
 * @example
 * ```ts
 * scene.addShape('box', {
 *   shape: rect(100, 100),
 *   transform: {
 *     position: keyframes([
 *       { frame: 0, value: [0, 0], easing: 'ease_out' },
 *       { frame: 30, value: [960, 540] },
 *     ]),
 *   },
 * })
 * ```
 */
export declare function keyframes<T>(kfs: Keyframe<T>[]): Keyframe<T>[];
/**
 * Create keyframes that interpolate a value over a frame range.
 *
 * Matches Remotion's `interpolate()` API but returns keyframes for
 * .mmot.json serialization instead of computing a value per frame.
 *
 * @param _frame - Ignored in build mode (used for API compatibility with Remotion).
 * @param inputRange - [startFrame, endFrame] tuple.
 * @param outputRange - [startValue, endValue] tuple.
 * @param options - Optional easing configuration.
 * @returns Array of two keyframes spanning the interpolation.
 *
 * @example
 * ```ts
 * const opacity = interpolate(0, [0, 30], [0, 1])
 * // => [{ frame: 0, value: 0, easing: 'ease_in_out' }, { frame: 30, value: 1 }]
 * ```
 */
export declare function interpolate(_frame: number, inputRange: [number, number], outputRange: [number, number], options?: {
    easing?: EasingValue;
}): Keyframe<number>[];
/**
 * Create keyframes with spring physics easing.
 *
 * Matches Remotion's `spring()` API but returns keyframes for
 * .mmot.json serialization.
 *
 * @param config - Spring configuration including physics parameters.
 * @returns Array of two keyframes with spring easing from `from` to `to`.
 *
 * @example
 * ```ts
 * const scale = spring({ fps: 30, config: { stiffness: 170, damping: 26 } })
 * // => [{ frame: 0, value: 0, easing: { type: 'spring', ... } }, { frame: 30, value: 1 }]
 * ```
 */
export declare function spring(config?: {
    /** Ignored in build mode (Remotion API compat). */
    frame?: number;
    /** Frames per second — used as the end frame for the animation. Defaults to 30. */
    fps?: number;
    /** Start value. Defaults to 0. */
    from?: number;
    /** End value. Defaults to 1. */
    to?: number;
    /** Spring physics parameters. */
    config?: {
        stiffness?: number;
        damping?: number;
        mass?: number;
    };
}): Keyframe<number>[];
/**
 * Easing presets matching Remotion's `Easing` object.
 *
 * @example
 * ```ts
 * interpolate(0, [0, 30], [0, 1], { easing: Easing.easeOut })
 * interpolate(0, [0, 30], [0, 1], { easing: Easing.bezier(0.42, 0, 0.58, 1) })
 * ```
 */
export declare const Easing: {
    /** Linear interpolation (no easing). */
    readonly linear: "linear";
    /** Ease in (slow start). */
    readonly easeIn: "ease_in";
    /** Ease out (slow end). */
    readonly easeOut: "ease_out";
    /** Ease in-out (slow start and end). */
    readonly easeInOut: "ease_in_out";
    /**
     * Custom cubic bezier curve.
     *
     * @param x1 - First control point X.
     * @param y1 - First control point Y.
     * @param x2 - Second control point X.
     * @param y2 - Second control point Y.
     */
    readonly bezier: (x1: number, y1: number, x2: number, y2: number) => {
        type: "cubic_bezier";
        x1: number;
        y1: number;
        x2: number;
        y2: number;
    };
};
/**
 * Create a rectangle shape specification.
 *
 * @param width - Rectangle width in pixels.
 * @param height - Rectangle height in pixels.
 * @param opts - Optional fill color, corner radius, and stroke.
 *
 * @example
 * ```ts
 * scene.addShape('box', { shape: rect(200, 100, { fill: '#ff0000', cornerRadius: 10 }) })
 * ```
 */
export declare function rect(width: number, height: number, opts?: {
    cornerRadius?: number;
    fill?: string;
    stroke?: StrokeConfig;
}): ShapeConfig;
/**
 * Create an ellipse shape specification.
 *
 * @param width - Ellipse width (horizontal diameter) in pixels.
 * @param height - Ellipse height (vertical diameter) in pixels.
 * @param opts - Optional fill color and stroke.
 *
 * @example
 * ```ts
 * scene.addShape('circle', { shape: ellipse(100, 100, { fill: '#00ff00' }) })
 * ```
 */
export declare function ellipse(width: number, height: number, opts?: {
    fill?: string;
    stroke?: StrokeConfig;
}): ShapeConfig;
/**
 * Create a line shape specification.
 *
 * @param x1 - Start X coordinate.
 * @param y1 - Start Y coordinate.
 * @param x2 - End X coordinate.
 * @param y2 - End Y coordinate.
 * @param stroke - Stroke color and width.
 *
 * @example
 * ```ts
 * scene.addShape('divider', { shape: line(0, 0, 100, 0, { color: '#fff', width: 2 }) })
 * ```
 */
export declare function line(x1: number, y1: number, x2: number, y2: number, stroke: StrokeConfig): ShapeConfig;
/**
 * Create a polygon shape specification from a list of points.
 *
 * @param points - Array of [x, y] coordinate pairs.
 * @param opts - Optional fill color and stroke.
 *
 * @example
 * ```ts
 * scene.addShape('triangle', {
 *   shape: polygon([[0, -50], [50, 50], [-50, 50]], { fill: '#ff0000' })
 * })
 * ```
 */
export declare function polygon(points: [number, number][], opts?: {
    fill?: string;
    stroke?: StrokeConfig;
}): ShapeConfig;
/**
 * Generate a star polygon with alternating outer/inner radii.
 *
 * @param outerRadius - Radius of the outer points.
 * @param innerRadius - Radius of the inner points.
 * @param numPoints - Number of star points (e.g., 5 for a classic star).
 * @param opts - Optional fill color and stroke.
 * @returns A polygon ShapeConfig with the star's vertices.
 *
 * @example
 * ```ts
 * scene.addShape('star', {
 *   shape: star(100, 50, 5, { fill: '#FFD700' }),
 *   transform: { position: [960, 540] }
 * })
 * ```
 */
export declare function star(outerRadius: number, innerRadius: number, numPoints: number, opts?: {
    fill?: string;
    stroke?: StrokeConfig;
}): ShapeConfig;
/**
 * Create a linear gradient specification.
 *
 * @param colors - Array of color stops with offset (0-1) and hex color.
 * @param start - Gradient start point [x, y] in normalized coordinates. Defaults to [0, 0].
 * @param end - Gradient end point [x, y] in normalized coordinates. Defaults to [1, 1].
 *
 * @example
 * ```ts
 * scene.addGradient('bg', {
 *   gradient: linearGradient(
 *     [{ offset: 0, color: '#1a1a3e' }, { offset: 1, color: '#0a0a1a' }]
 *   ),
 *   fill: 'parent'
 * })
 * ```
 */
export declare function linearGradient(colors: GradientStop[], start?: [number, number], end?: [number, number]): GradientConfig;
/**
 * Create a radial gradient specification.
 *
 * @param colors - Array of color stops with offset (0-1) and hex color.
 * @param center - Gradient center point [x, y] in normalized coordinates. Defaults to [0.5, 0.5].
 * @param radius - Gradient radius in normalized coordinates. Defaults to 0.5.
 *
 * @example
 * ```ts
 * scene.addGradient('spotlight', {
 *   gradient: radialGradient(
 *     [{ offset: 0, color: '#ffffff' }, { offset: 1, color: '#000000' }],
 *     [0.5, 0.5], 0.8
 *   ),
 *   fill: 'parent'
 * })
 * ```
 */
export declare function radialGradient(colors: GradientStop[], center?: [number, number], radius?: number): GradientConfig;
/**
 * Pre-built effect constructors matching common After Effects / Remotion effects.
 *
 * @example
 * ```ts
 * scene.addShape('box', {
 *   shape: rect(200, 100, { fill: '#ff0000' }),
 *   effects: [
 *     Effects.blur(5),
 *     Effects.shadow({ color: '#000', blur: 10 }),
 *     Effects.glow({ color: '#ff0000', radius: 20 }),
 *   ]
 * })
 * ```
 */
export declare const Effects: {
    /**
     * Gaussian blur effect.
     * @param radius - Blur radius in pixels.
     */
    readonly blur: (radius: number) => EffectConfig;
    /**
     * Drop shadow effect.
     * @param opts - Shadow color, offset, blur, and opacity.
     */
    readonly shadow: (opts?: {
        color?: string;
        x?: number;
        y?: number;
        blur?: number;
        opacity?: number;
    }) => EffectConfig;
    /**
     * Glow effect emanating from bright areas.
     * @param opts - Glow color, radius, and intensity.
     */
    readonly glow: (opts?: {
        color?: string;
        radius?: number;
        intensity?: number;
    }) => EffectConfig;
    /**
     * Brightness and contrast adjustment.
     * @param brightness - Brightness offset (positive = brighter).
     * @param contrast - Contrast offset (positive = more contrast).
     */
    readonly brightnessContrast: (brightness: number, contrast: number) => EffectConfig;
    /**
     * Hue, saturation, and lightness adjustment.
     * @param hue - Hue rotation in degrees.
     * @param saturation - Saturation offset.
     * @param lightness - Lightness offset. Defaults to 0.
     */
    readonly hueSaturation: (hue: number, saturation: number, lightness?: number) => EffectConfig;
    /** Invert all color channels. */
    readonly invert: () => EffectConfig;
    /**
     * Tint the layer toward a target color.
     * @param color - Target tint color (hex).
     * @param amount - Tint strength (0-1). Defaults to 1.0.
     */
    readonly tint: (color: string, amount?: number) => EffectConfig;
    /**
     * Fill the entire layer with a solid color overlay.
     * @param color - Fill color (hex).
     * @param opacity - Fill opacity (0-1). Defaults to 1.0.
     */
    readonly fill: (color: string, opacity?: number) => EffectConfig;
};
