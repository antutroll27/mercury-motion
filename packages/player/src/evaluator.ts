/**
 * Keyframe evaluator — port of mmot-core/src/evaluator to TypeScript.
 *
 * Evaluates animatable values (static or keyframed) at a given frame,
 * with full cubic-bezier easing and approximate spring support.
 */

// ── Cubic Bezier ──────────────────────────────────────────────────────────────

/**
 * Evaluate a single component of a cubic Bezier curve at parameter t.
 */
function bezierComponent(p1: number, p2: number, t: number): number {
  const c1 = 3 * p1
  const c2 = 3 * (p2 - p1) - c1
  const c3 = 1 - c1 - c2
  return ((c3 * t + c2) * t + c1) * t
}

/**
 * Compute the slope of a cubic Bezier component at parameter t.
 */
function bezierSlope(p1: number, p2: number, t: number): number {
  const c1 = 3 * p1
  const c2 = 3 * (p2 - p1) - c1
  const c3 = 1 - c1 - c2
  return (3 * c3 * t + 2 * c2) * t + c1
}

/**
 * Newton's method (8 iterations) to solve for the Bezier parameter
 * given an x value. Matches Rust evaluator/easing.rs.
 */
function solveT(x1: number, x2: number, x: number): number {
  let t = x
  for (let i = 0; i < 8; i++) {
    const xEst = bezierComponent(x1, x2, t) - x
    const slope = bezierSlope(x1, x2, t)
    if (Math.abs(slope) < 1e-6) break
    t -= xEst / slope
    t = Math.max(0, Math.min(1, t))
  }
  return t
}

/**
 * CSS-style cubic Bezier: solve for y given x = t.
 */
function cubicBezier(t: number, x1: number, y1: number, x2: number, y2: number): number {
  const s = solveT(x1, x2, t)
  return bezierComponent(y1, y2, s)
}

// ── Spring ────────────────────────────────────────────────────────────────────

/**
 * Compute the settling time for a spring (time to decay below 0.1% of target).
 */
function springSettlingTime(mass: number, stiffness: number, damping: number): number {
  const omega = Math.sqrt(stiffness / mass)
  const zeta = damping / (2 * Math.sqrt(stiffness * mass))
  const settle = zeta > 0.001 ? 6.9 / (zeta * omega) : 10.0
  return Math.min(settle, 10.0)
}

/**
 * Spring easing — full physics simulation matching Rust evaluator.
 */
function spring(mass: number, stiffness: number, damping: number, t: number): number {
  if (t <= 0) return 0
  if (t >= 1) return 1

  const omega = Math.sqrt(stiffness / mass)
  const zeta = damping / (2 * Math.sqrt(stiffness * mass))
  const settleTime = springSettlingTime(mass, stiffness, damping)
  const simT = t * settleTime

  if (zeta >= 1.0) {
    // Critically damped or overdamped
    const decay = Math.exp(-omega * zeta * simT)
    return 1.0 - decay * (1.0 + omega * zeta * simT)
  } else {
    // Underdamped — oscillation with decay
    const omegaD = omega * Math.sqrt(1.0 - zeta * zeta)
    const decay = Math.exp(-zeta * omega * simT)
    return 1.0 - decay * ((zeta * omega / omegaD) * Math.sin(omegaD * simT) + Math.cos(omegaD * simT))
  }
}

// ── Easing Presets ────────────────────────────────────────────────────────────

const EASING_PRESETS: Record<string, [number, number, number, number]> = {
  linear: [0, 0, 1, 1],
  ease_in: [0.42, 0, 1, 1],
  ease_out: [0, 0, 0.58, 1],
  ease_in_out: [0.42, 0, 0.58, 1],
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
function applyEasing(t: number, easing: any): number {
  if (!easing || easing === 'linear') return t

  if (typeof easing === 'string' && EASING_PRESETS[easing]) {
    const [x1, y1, x2, y2] = EASING_PRESETS[easing]
    return cubicBezier(t, x1, y1, x2, y2)
  }

  if (typeof easing === 'object') {
    if (easing.type === 'cubic_bezier') {
      return cubicBezier(t, easing.x1, easing.y1, easing.x2, easing.y2)
    }
    if (easing.type === 'spring') {
      const mass = easing.mass ?? 1.0
      const stiffness = easing.stiffness ?? 170.0
      const damping = easing.damping ?? 26.0
      return spring(mass, stiffness, damping, t)
    }
    // Named easing object: { "type": "ease_in" } etc.
    if (typeof easing.type === 'string' && EASING_PRESETS[easing.type]) {
      const [x1, y1, x2, y2] = EASING_PRESETS[easing.type]
      return cubicBezier(t, x1, y1, x2, y2)
    }
  }

  return t
}

// ── Keyframe Evaluation ───────────────────────────────────────────────────────

export interface ScalarKeyframe {
  t: number
  v: number
  easing?: unknown
}

export interface Vec2Keyframe {
  t: number
  v: [number, number]
  easing?: unknown
}

/**
 * Evaluate an animatable scalar value at the given frame.
 *
 * Accepts:
 * - A plain number (static value)
 * - An array of keyframes `[{ t, v, easing? }, ...]`
 */
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function evaluateValue(value: any, frame: number): number {
  if (typeof value === 'number') return value
  if (value == null) return 0

  // Keyframe array
  if (Array.isArray(value) && value.length > 0 && typeof value[0] === 'object' && 't' in value[0]) {
    const kfs = value as ScalarKeyframe[]
    if (kfs.length === 1) return kfs[0].v
    if (frame <= kfs[0].t) return kfs[0].v
    if (frame >= kfs[kfs.length - 1].t) return kfs[kfs.length - 1].v

    // Find the segment: last keyframe whose t <= frame
    let i = 0
    while (i < kfs.length - 1 && kfs[i + 1].t <= frame) i++

    const from = kfs[i]
    const to = kfs[i + 1]

    if (to.t === from.t) return to.v

    const rawT = (frame - from.t) / (to.t - from.t)
    const t = applyEasing(Math.max(0, Math.min(1, rawT)), from.easing)
    return from.v + (to.v - from.v) * t
  }

  return typeof value === 'number' ? value : 0
}

/**
 * Evaluate an animatable 2D vector value at the given frame.
 *
 * Accepts:
 * - A plain [x, y] array (static value)
 * - An array of keyframes `[{ t, v: [x, y], easing? }, ...]`
 */
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function evaluateVec2(value: any, frame: number): [number, number] {
  if (value == null) return [0, 0]

  // Static [x, y]
  if (Array.isArray(value) && value.length === 2 && typeof value[0] === 'number') {
    return value as [number, number]
  }

  // Keyframe array
  if (Array.isArray(value) && value.length > 0 && typeof value[0] === 'object' && 't' in value[0]) {
    const kfs = value as Vec2Keyframe[]
    if (kfs.length === 1) return kfs[0].v
    if (frame <= kfs[0].t) return kfs[0].v
    if (frame >= kfs[kfs.length - 1].t) return kfs[kfs.length - 1].v

    let i = 0
    while (i < kfs.length - 1 && kfs[i + 1].t <= frame) i++

    const from = kfs[i]
    const to = kfs[i + 1]

    if (to.t === from.t) return to.v

    const rawT = (frame - from.t) / (to.t - from.t)
    const t = applyEasing(Math.max(0, Math.min(1, rawT)), from.easing)
    return [
      from.v[0] + (to.v[0] - from.v[0]) * t,
      from.v[1] + (to.v[1] - from.v[1]) * t,
    ]
  }

  return [0, 0]
}

/**
 * Evaluate an animatable color value at the given frame.
 * Returns a CSS color string.
 *
 * Accepts:
 * - A plain string (static color)
 * - An array of keyframes `[{ t, v: "#hex", easing? }, ...]`
 */
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function evaluateColor(value: any, frame: number): string {
  if (typeof value === 'string') return value
  if (value == null) return '#000000'

  if (Array.isArray(value) && value.length > 0 && typeof value[0] === 'object' && 't' in value[0]) {
    const kfs = value as { t: number; v: string; easing?: unknown }[]
    if (kfs.length === 1) return kfs[0].v
    if (frame <= kfs[0].t) return kfs[0].v
    if (frame >= kfs[kfs.length - 1].t) return kfs[kfs.length - 1].v

    let i = 0
    while (i < kfs.length - 1 && kfs[i + 1].t <= frame) i++

    const from = kfs[i]
    const to = kfs[i + 1]

    if (to.t === from.t) return to.v

    const rawT = (frame - from.t) / (to.t - from.t)
    const t = applyEasing(Math.max(0, Math.min(1, rawT)), from.easing)

    // Interpolate hex colors in RGB space
    const fromRgb = parseHex(from.v)
    const toRgb = parseHex(to.v)
    const r = Math.round(fromRgb[0] + (toRgb[0] - fromRgb[0]) * t)
    const g = Math.round(fromRgb[1] + (toRgb[1] - fromRgb[1]) * t)
    const b = Math.round(fromRgb[2] + (toRgb[2] - fromRgb[2]) * t)
    return `#${hex2(r)}${hex2(g)}${hex2(b)}`
  }

  return typeof value === 'string' ? value : '#000000'
}

function parseHex(hex: string): [number, number, number] {
  const h = hex.replace('#', '')
  if (h.length < 6) return [0, 0, 0]
  return [
    parseInt(h.slice(0, 2), 16),
    parseInt(h.slice(2, 4), 16),
    parseInt(h.slice(4, 6), 16),
  ]
}

function hex2(n: number): string {
  return Math.max(0, Math.min(255, n)).toString(16).padStart(2, '0')
}
