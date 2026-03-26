/**
 * @mmot/react — Write React. Render natively.
 *
 * Remotion-compatible builder API that serializes to .mmot.json
 * for Mercury-Motion's native Rust renderer.
 *
 * @packageDocumentation
 */
// ---------------------------------------------------------------------------
// Internal: mmot.json serialization helpers
// ---------------------------------------------------------------------------
/**
 * Convert a transform config to the .mmot.json transform format.
 * Static values use direct literals; animated values use keyframe arrays.
 */
function serializeTransform(t) {
    const result = {};
    if (t?.position !== undefined) {
        result.position = isKeyframeArray(t.position)
            ? serializeKeyframes(t.position)
            : t.position;
    }
    if (t?.scale !== undefined) {
        result.scale = isKeyframeArray(t.scale)
            ? serializeKeyframes(t.scale)
            : t.scale;
    }
    if (t?.rotation !== undefined) {
        result.rotation = isKeyframeArray(t.rotation)
            ? serializeKeyframes(t.rotation)
            : t.rotation;
    }
    if (t?.opacity !== undefined) {
        result.opacity = isKeyframeArray(t.opacity)
            ? serializeKeyframes(t.opacity)
            : t.opacity;
    }
    return result;
}
/**
 * Detect whether a value is a Keyframe array vs. a static tuple/number.
 * Keyframe arrays are arrays of objects with a `frame` property.
 */
function isKeyframeArray(v) {
    return Array.isArray(v) && v.length > 0 && typeof v[0] === 'object' && v[0] !== null && 'frame' in v[0];
}
/** Serialize a Keyframe<T>[] to the .mmot.json `{t, v, easing?}` format. */
function serializeKeyframes(kfs) {
    return kfs.map(kf => {
        const entry = { t: kf.frame, v: kf.value };
        if (kf.easing !== undefined) {
            entry.easing = kf.easing;
        }
        return entry;
    });
}
// ---------------------------------------------------------------------------
// Scene Builder
// ---------------------------------------------------------------------------
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
export class Scene {
    config;
    layers = [];
    isSequence = false;
    sequenceTransition;
    constructor(config) {
        this.config = config;
    }
    /**
     * Add a solid color layer.
     *
     * @param id - Unique layer identifier.
     * @param opts - Layer options including color, timing, transform, and effects.
     */
    addSolid(id, opts) {
        this.layers.push({
            id,
            type: 'solid',
            in: opts.in ?? 0,
            out: opts.out ?? this.config.durationInFrames,
            color: opts.color,
            transform: serializeTransform(opts.transform),
            ...(opts.effects ? { effects: opts.effects } : {}),
            ...(opts.blendMode ? { blend_mode: opts.blendMode } : {}),
            ...(opts.fill ? { fill: opts.fill } : {}),
            ...(opts.parent ? { parent: opts.parent } : {}),
            ...(opts.masks ? { masks: opts.masks } : {}),
            ...(opts.adjustment ? { adjustment: opts.adjustment } : {}),
        });
        return this;
    }
    /**
     * Add a text layer.
     *
     * @param id - Unique layer identifier.
     * @param opts - Layer options including text content, font, alignment, and transform.
     */
    addText(id, opts) {
        const font = opts.font ?? { family: 'Arial' };
        this.layers.push({
            id,
            type: 'text',
            in: opts.in ?? 0,
            out: opts.out ?? this.config.durationInFrames,
            text: opts.text,
            font: {
                family: font.family,
                size: font.size ?? 32,
                weight: font.weight ?? 400,
                color: font.color ?? '#ffffff',
            },
            transform: serializeTransform(opts.transform),
            ...(opts.align ? { align: opts.align } : {}),
            ...(opts.effects ? { effects: opts.effects } : {}),
            ...(opts.blendMode ? { blend_mode: opts.blendMode } : {}),
            ...(opts.fill ? { fill: opts.fill } : {}),
        });
        return this;
    }
    /**
     * Add a shape layer (rect, ellipse, line, or polygon).
     *
     * @param id - Unique layer identifier.
     * @param opts - Layer options including shape spec, transform, and effects.
     */
    addShape(id, opts) {
        this.layers.push({
            id,
            type: 'shape',
            in: opts.in ?? 0,
            out: opts.out ?? this.config.durationInFrames,
            shape: opts.shape,
            transform: serializeTransform(opts.transform),
            ...(opts.effects ? { effects: opts.effects } : {}),
            ...(opts.blendMode ? { blend_mode: opts.blendMode } : {}),
            ...(opts.fill ? { fill: opts.fill } : {}),
            ...(opts.parent ? { parent: opts.parent } : {}),
            ...(opts.motionBlur ? { motion_blur: opts.motionBlur } : {}),
            ...(opts.pathAnimation ? {
                path_animation: {
                    points: opts.pathAnimation.points,
                    ...(opts.pathAnimation.autoOrient ? { auto_orient: opts.pathAnimation.autoOrient } : {}),
                }
            } : {}),
            ...(opts.trimPaths ? { trim_paths: opts.trimPaths } : {}),
        });
        return this;
    }
    /**
     * Add a gradient layer (linear or radial).
     *
     * @param id - Unique layer identifier.
     * @param opts - Layer options including gradient spec and transform.
     */
    addGradient(id, opts) {
        this.layers.push({
            id,
            type: 'gradient',
            in: opts.in ?? 0,
            out: opts.out ?? this.config.durationInFrames,
            gradient: opts.gradient,
            transform: serializeTransform(opts.transform),
            ...(opts.effects ? { effects: opts.effects } : {}),
            ...(opts.blendMode ? { blend_mode: opts.blendMode } : {}),
            ...(opts.fill ? { fill: opts.fill } : {}),
        });
        return this;
    }
    /**
     * Add an image layer.
     *
     * @param id - Unique layer identifier.
     * @param opts - Layer options including source path and transform.
     */
    addImage(id, opts) {
        this.layers.push({
            id,
            type: 'image',
            in: opts.in ?? 0,
            out: opts.out ?? this.config.durationInFrames,
            src: opts.src,
            transform: serializeTransform(opts.transform),
            ...(opts.effects ? { effects: opts.effects } : {}),
            ...(opts.blendMode ? { blend_mode: opts.blendMode } : {}),
            ...(opts.fill ? { fill: opts.fill } : {}),
        });
        return this;
    }
    /**
     * Add a video layer.
     *
     * @param id - Unique layer identifier.
     * @param opts - Layer options including source path, trim points, and transform.
     */
    addVideo(id, opts) {
        this.layers.push({
            id,
            type: 'video',
            in: opts.in ?? 0,
            out: opts.out ?? this.config.durationInFrames,
            src: opts.src,
            transform: serializeTransform(opts.transform),
            ...(opts.trimStart !== undefined ? { trim_start: opts.trimStart } : {}),
            ...(opts.trimEnd !== undefined ? { trim_end: opts.trimEnd } : {}),
            ...(opts.effects ? { effects: opts.effects } : {}),
            ...(opts.blendMode ? { blend_mode: opts.blendMode } : {}),
            ...(opts.fill ? { fill: opts.fill } : {}),
        });
        return this;
    }
    /**
     * Add an audio layer.
     *
     * @param id - Unique layer identifier.
     * @param opts - Layer options including source path and volume.
     */
    addAudio(id, opts) {
        this.layers.push({
            id,
            type: 'audio',
            in: opts.in ?? 0,
            out: opts.out ?? this.config.durationInFrames,
            src: opts.src,
            transform: {},
            ...(opts.volume !== undefined ? { volume: opts.volume } : {}),
        });
        return this;
    }
    /**
     * Add a null (invisible) layer, used for parenting transforms.
     *
     * @param id - Unique layer identifier.
     * @param opts - Layer options including transform and timing.
     */
    addNull(id, opts) {
        this.layers.push({
            id,
            type: 'null',
            in: opts?.in ?? 0,
            out: opts?.out ?? this.config.durationInFrames,
            transform: serializeTransform(opts?.transform),
        });
        return this;
    }
    /**
     * Add a composition reference layer.
     *
     * @param id - Unique layer identifier.
     * @param opts - Layer options including the target composition ID.
     */
    addComposition(id, opts) {
        this.layers.push({
            id,
            type: 'composition',
            in: opts.in ?? 0,
            out: opts.out ?? this.config.durationInFrames,
            composition_id: opts.compositionId,
            transform: serializeTransform(opts.transform),
            ...(opts.effects ? { effects: opts.effects } : {}),
        });
        return this;
    }
    /**
     * Enable sequence mode: layers play back-to-back instead of simultaneously.
     *
     * @param transition - Optional transition between consecutive layers.
     */
    setSequence(transition) {
        this.isSequence = true;
        this.sequenceTransition = transition;
        return this;
    }
    /**
     * Serialize the scene to a plain object matching the .mmot.json format.
     *
     * @returns The complete scene object ready for JSON.stringify.
     */
    toJSON() {
        const composition = {
            layers: this.layers,
        };
        if (this.isSequence) {
            composition.sequence = true;
            if (this.sequenceTransition) {
                composition.transition = this.sequenceTransition;
            }
        }
        return {
            version: '1.0',
            meta: {
                name: this.config.name ?? 'Untitled',
                width: this.config.width,
                height: this.config.height,
                fps: this.config.fps,
                duration: this.config.durationInFrames,
                background: this.config.background ?? '#000000',
                root: 'main',
            },
            compositions: {
                main: composition,
            },
        };
    }
    /**
     * Serialize the scene to a JSON string.
     *
     * @returns Formatted JSON string of the .mmot.json scene.
     */
    toString() {
        return JSON.stringify(this.toJSON(), null, 2);
    }
}
// ---------------------------------------------------------------------------
// Keyframe helpers
// ---------------------------------------------------------------------------
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
export function keyframes(kfs) {
    // Return the keyframes as-is; they'll be detected as animated
    // by isKeyframeArray and serialized by serializeKeyframes.
    return kfs;
}
// ---------------------------------------------------------------------------
// Remotion-compatible helpers
// ---------------------------------------------------------------------------
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
export function interpolate(_frame, inputRange, outputRange, options) {
    return [
        { frame: inputRange[0], value: outputRange[0], easing: options?.easing ?? 'ease_in_out' },
        { frame: inputRange[1], value: outputRange[1] },
    ];
}
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
export function spring(config) {
    const from = config?.from ?? 0;
    const to = config?.to ?? 1;
    const stiffness = config?.config?.stiffness ?? 170;
    const damping = config?.config?.damping ?? 26;
    const mass = config?.config?.mass ?? 1;
    return [
        { frame: 0, value: from, easing: { type: 'spring', stiffness, damping, mass } },
        { frame: config?.fps ?? 30, value: to },
    ];
}
/**
 * Easing presets matching Remotion's `Easing` object.
 *
 * @example
 * ```ts
 * interpolate(0, [0, 30], [0, 1], { easing: Easing.easeOut })
 * interpolate(0, [0, 30], [0, 1], { easing: Easing.bezier(0.42, 0, 0.58, 1) })
 * ```
 */
export const Easing = {
    /** Linear interpolation (no easing). */
    linear: 'linear',
    /** Ease in (slow start). */
    easeIn: 'ease_in',
    /** Ease out (slow end). */
    easeOut: 'ease_out',
    /** Ease in-out (slow start and end). */
    easeInOut: 'ease_in_out',
    /**
     * Custom cubic bezier curve.
     *
     * @param x1 - First control point X.
     * @param y1 - First control point Y.
     * @param x2 - Second control point X.
     * @param y2 - Second control point Y.
     */
    bezier: (x1, y1, x2, y2) => ({
        type: 'cubic_bezier', x1, y1, x2, y2
    }),
};
// ---------------------------------------------------------------------------
// Shape helpers
// ---------------------------------------------------------------------------
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
export function rect(width, height, opts) {
    return {
        shape_type: 'rect',
        width,
        height,
        ...(opts?.cornerRadius !== undefined ? { corner_radius: opts.cornerRadius } : {}),
        ...(opts?.fill ? { fill: opts.fill } : {}),
        ...(opts?.stroke ? { stroke: opts.stroke } : {}),
    };
}
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
export function ellipse(width, height, opts) {
    return {
        shape_type: 'ellipse',
        width,
        height,
        ...(opts?.fill ? { fill: opts.fill } : {}),
        ...(opts?.stroke ? { stroke: opts.stroke } : {}),
    };
}
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
export function line(x1, y1, x2, y2, stroke) {
    return { shape_type: 'line', x1, y1, x2, y2, stroke };
}
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
export function polygon(points, opts) {
    return {
        shape_type: 'polygon',
        points,
        ...(opts?.fill ? { fill: opts.fill } : {}),
        ...(opts?.stroke ? { stroke: opts.stroke } : {}),
    };
}
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
export function star(outerRadius, innerRadius, numPoints, opts) {
    const pts = [];
    for (let i = 0; i < numPoints * 2; i++) {
        const angle = (Math.PI * i) / numPoints - Math.PI / 2;
        const r = i % 2 === 0 ? outerRadius : innerRadius;
        pts.push([Math.cos(angle) * r, Math.sin(angle) * r]);
    }
    return {
        shape_type: 'polygon',
        points: pts,
        ...(opts?.fill ? { fill: opts.fill } : {}),
        ...(opts?.stroke ? { stroke: opts.stroke } : {}),
    };
}
// ---------------------------------------------------------------------------
// Gradient helpers
// ---------------------------------------------------------------------------
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
export function linearGradient(colors, start, end) {
    return {
        gradient_type: 'linear',
        colors,
        start: start ?? [0, 0],
        end: end ?? [1, 1],
    };
}
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
export function radialGradient(colors, center, radius) {
    return {
        gradient_type: 'radial',
        colors,
        center: center ?? [0.5, 0.5],
        radius: radius ?? 0.5,
    };
}
// ---------------------------------------------------------------------------
// Effects helpers
// ---------------------------------------------------------------------------
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
export const Effects = {
    /**
     * Gaussian blur effect.
     * @param radius - Blur radius in pixels.
     */
    blur: (radius) => ({ type: 'gaussian_blur', radius }),
    /**
     * Drop shadow effect.
     * @param opts - Shadow color, offset, blur, and opacity.
     */
    shadow: (opts) => ({
        type: 'drop_shadow',
        color: opts?.color ?? '#000000',
        offset_x: opts?.x ?? 0,
        offset_y: opts?.y ?? 4,
        blur: opts?.blur ?? 8,
        opacity: opts?.opacity ?? 0.5,
    }),
    /**
     * Glow effect emanating from bright areas.
     * @param opts - Glow color, radius, and intensity.
     */
    glow: (opts) => ({
        type: 'glow',
        color: opts?.color ?? '#ffffff',
        radius: opts?.radius ?? 10,
        intensity: opts?.intensity ?? 0.8,
    }),
    /**
     * Brightness and contrast adjustment.
     * @param brightness - Brightness offset (positive = brighter).
     * @param contrast - Contrast offset (positive = more contrast).
     */
    brightnessContrast: (brightness, contrast) => ({
        type: 'brightness_contrast', brightness, contrast,
    }),
    /**
     * Hue, saturation, and lightness adjustment.
     * @param hue - Hue rotation in degrees.
     * @param saturation - Saturation offset.
     * @param lightness - Lightness offset. Defaults to 0.
     */
    hueSaturation: (hue, saturation, lightness) => ({
        type: 'hue_saturation', hue, saturation, lightness: lightness ?? 0,
    }),
    /** Invert all color channels. */
    invert: () => ({ type: 'invert' }),
    /**
     * Tint the layer toward a target color.
     * @param color - Target tint color (hex).
     * @param amount - Tint strength (0-1). Defaults to 1.0.
     */
    tint: (color, amount) => ({
        type: 'tint', color, amount: amount ?? 1.0,
    }),
    /**
     * Fill the entire layer with a solid color overlay.
     * @param color - Fill color (hex).
     * @param opacity - Fill opacity (0-1). Defaults to 1.0.
     */
    fill: (color, opacity) => ({
        type: 'fill', color, opacity: opacity ?? 1.0,
    }),
};
