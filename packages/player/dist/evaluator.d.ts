/**
 * Keyframe evaluator — port of mmot-core/src/evaluator to TypeScript.
 *
 * Evaluates animatable values (static or keyframed) at a given frame,
 * with full cubic-bezier easing and approximate spring support.
 */
export interface ScalarKeyframe {
    t: number;
    v: number;
    easing?: unknown;
}
export interface Vec2Keyframe {
    t: number;
    v: [number, number];
    easing?: unknown;
}
/**
 * Evaluate an animatable scalar value at the given frame.
 *
 * Accepts:
 * - A plain number (static value)
 * - An array of keyframes `[{ t, v, easing? }, ...]`
 */
export declare function evaluateValue(value: any, frame: number): number;
/**
 * Evaluate an animatable 2D vector value at the given frame.
 *
 * Accepts:
 * - A plain [x, y] array (static value)
 * - An array of keyframes `[{ t, v: [x, y], easing? }, ...]`
 */
export declare function evaluateVec2(value: any, frame: number): [number, number];
/**
 * Evaluate an animatable color value at the given frame.
 * Returns a CSS color string.
 *
 * Accepts:
 * - A plain string (static color)
 * - An array of keyframes `[{ t, v: "#hex", easing? }, ...]`
 */
export declare function evaluateColor(value: any, frame: number): string;
//# sourceMappingURL=evaluator.d.ts.map