/**
 * Canvas 2D renderer for .mmot.json scenes.
 *
 * Renders a scene at a given frame to a CanvasRenderingContext2D.
 * Supports: solid, text, shape, gradient, null layers, transforms,
 * easing, effects, masks, and blend modes.
 */
import { evaluateValue, evaluateVec2 } from './evaluator.js';
// ── Public API ────────────────────────────────────────────────────────────────
/**
 * Render a single frame of a .mmot.json scene to a Canvas 2D context.
 */
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function renderFrame(ctx, scene, frame, width, height) {
    ctx.clearRect(0, 0, width, height);
    // Background
    ctx.fillStyle = parseHexColor(scene.meta?.background || '#000000');
    ctx.fillRect(0, 0, width, height);
    const rootId = scene.meta?.root || 'main';
    const comp = scene.compositions?.[rootId];
    if (!comp?.layers)
        return;
    for (const layer of comp.layers) {
        const inPoint = layer.in ?? 0;
        const outPoint = layer.out ?? Infinity;
        if (frame < inPoint || frame >= outPoint)
            continue;
        renderLayer(ctx, layer, frame, width, height);
    }
}
// ── Layer Rendering ───────────────────────────────────────────────────────────
// eslint-disable-next-line @typescript-eslint/no-explicit-any
function renderLayer(ctx, layer, frame, canvasWidth, canvasHeight) {
    ctx.save();
    // Transform evaluation
    const pos = evaluateVec2(layer.transform?.position, frame);
    const scale = evaluateVec2(layer.transform?.scale ?? [1, 1], frame);
    const rotation = evaluateValue(layer.transform?.rotation ?? 0, frame);
    const opacity = evaluateValue(layer.transform?.opacity ?? 1, frame);
    ctx.globalAlpha = opacity;
    // Blend mode
    if (layer.blend_mode) {
        ctx.globalCompositeOperation = mapBlendMode(layer.blend_mode);
    }
    // Pre-drawing effects (shadow, blur)
    applyPreDrawEffects(ctx, layer.effects);
    // Fill-parent mode: skip position translate, render at full canvas
    const isFillParent = layer.fill === 'parent';
    if (isFillParent) {
        ctx.scale(scale[0], scale[1]);
        ctx.rotate((rotation * Math.PI) / 180);
    }
    else {
        ctx.translate(pos[0], pos[1]);
        ctx.scale(scale[0], scale[1]);
        ctx.rotate((rotation * Math.PI) / 180);
    }
    // Masks (clip paths)
    if (layer.masks && Array.isArray(layer.masks) && layer.masks.length > 0) {
        applyMasks(ctx, layer.masks);
    }
    // Draw content by type
    switch (layer.type) {
        case 'solid':
            drawSolid(ctx, layer, canvasWidth, canvasHeight, isFillParent, pos);
            break;
        case 'text':
            drawText(ctx, layer);
            break;
        case 'shape':
            drawShape(ctx, layer);
            break;
        case 'gradient':
            drawGradient(ctx, layer, pos, canvasWidth, canvasHeight, isFillParent);
            break;
        case 'null':
            // Transform-only layer, nothing to draw
            break;
    }
    // Reset effects
    ctx.filter = 'none';
    ctx.shadowColor = 'transparent';
    ctx.shadowBlur = 0;
    ctx.shadowOffsetX = 0;
    ctx.shadowOffsetY = 0;
    ctx.restore();
}
// ── Drawing Functions ─────────────────────────────────────────────────────────
// eslint-disable-next-line @typescript-eslint/no-explicit-any
function drawSolid(ctx, layer, canvasWidth, canvasHeight, isFillParent, pos) {
    const color = layer.color || '#ffffff';
    ctx.fillStyle = parseHexColor(color);
    if (isFillParent) {
        ctx.fillRect(0, 0, canvasWidth, canvasHeight);
    }
    else {
        // Draw centered around the position
        ctx.fillRect(-canvasWidth / 2, -canvasHeight / 2, canvasWidth, canvasHeight);
    }
}
// eslint-disable-next-line @typescript-eslint/no-explicit-any
function drawText(ctx, layer) {
    const text = layer.text || '';
    const font = layer.font || {};
    const size = font.size || 48;
    const weight = font.weight || 400;
    const family = font.family || 'Inter';
    const color = font.color || '#ffffff';
    const align = layer.align || 'center';
    ctx.fillStyle = parseHexColor(color);
    ctx.font = `${weight} ${size}px ${family}, sans-serif`;
    // Map alignment
    switch (align) {
        case 'left':
            ctx.textAlign = 'left';
            break;
        case 'right':
            ctx.textAlign = 'right';
            break;
        case 'center':
        default:
            ctx.textAlign = 'center';
            break;
    }
    ctx.textBaseline = 'middle';
    // Handle multi-line text
    const lines = text.split('\n');
    if (lines.length === 1) {
        ctx.fillText(text, 0, 0);
    }
    else {
        const lineHeight = size * 1.2;
        const totalHeight = (lines.length - 1) * lineHeight;
        const startY = -totalHeight / 2;
        for (let i = 0; i < lines.length; i++) {
            ctx.fillText(lines[i], 0, startY + i * lineHeight);
        }
    }
}
// eslint-disable-next-line @typescript-eslint/no-explicit-any
function drawShape(ctx, layer) {
    const shape = layer.shape;
    if (!shape)
        return;
    const fill = shape.fill ? parseHexColor(shape.fill) : null;
    const strokeSpec = shape.stroke;
    switch (shape.shape_type) {
        case 'rect': {
            const w = shape.width || 100;
            const h = shape.height || 100;
            const cr = shape.corner_radius || 0;
            if (fill) {
                ctx.fillStyle = fill;
                if (cr > 0) {
                    drawRoundRect(ctx, -w / 2, -h / 2, w, h, cr);
                    ctx.fill();
                }
                else {
                    ctx.fillRect(-w / 2, -h / 2, w, h);
                }
            }
            if (strokeSpec) {
                ctx.strokeStyle = parseHexColor(strokeSpec.color);
                ctx.lineWidth = strokeSpec.width;
                if (cr > 0) {
                    drawRoundRect(ctx, -w / 2, -h / 2, w, h, cr);
                    ctx.stroke();
                }
                else {
                    ctx.strokeRect(-w / 2, -h / 2, w, h);
                }
            }
            break;
        }
        case 'ellipse': {
            const w = shape.width || 100;
            const h = shape.height || 100;
            ctx.beginPath();
            ctx.ellipse(0, 0, w / 2, h / 2, 0, 0, Math.PI * 2);
            if (fill) {
                ctx.fillStyle = fill;
                ctx.fill();
            }
            if (strokeSpec) {
                ctx.strokeStyle = parseHexColor(strokeSpec.color);
                ctx.lineWidth = strokeSpec.width;
                ctx.stroke();
            }
            break;
        }
        case 'line': {
            const x1 = shape.x1 ?? 0;
            const y1 = shape.y1 ?? 0;
            const x2 = shape.x2 ?? 100;
            const y2 = shape.y2 ?? 100;
            if (strokeSpec) {
                ctx.beginPath();
                ctx.moveTo(x1, y1);
                ctx.lineTo(x2, y2);
                ctx.strokeStyle = parseHexColor(strokeSpec.color);
                ctx.lineWidth = strokeSpec.width;
                ctx.stroke();
            }
            break;
        }
        case 'polygon': {
            const points = shape.points;
            if (!points || points.length < 2)
                break;
            ctx.beginPath();
            ctx.moveTo(points[0][0], points[0][1]);
            for (let i = 1; i < points.length; i++) {
                ctx.lineTo(points[i][0], points[i][1]);
            }
            ctx.closePath();
            if (fill) {
                ctx.fillStyle = fill;
                ctx.fill();
            }
            if (strokeSpec) {
                ctx.strokeStyle = parseHexColor(strokeSpec.color);
                ctx.lineWidth = strokeSpec.width;
                ctx.stroke();
            }
            break;
        }
    }
}
// eslint-disable-next-line @typescript-eslint/no-explicit-any
function drawGradient(ctx, layer, pos, canvasWidth, canvasHeight, isFillParent) {
    const g = layer.gradient;
    if (!g)
        return;
    let grad;
    if (g.gradient_type === 'linear') {
        const start = g.start || [0, 0];
        const end = g.end || [1, 1];
        if (isFillParent) {
            grad = ctx.createLinearGradient(start[0] * canvasWidth, start[1] * canvasHeight, end[0] * canvasWidth, end[1] * canvasHeight);
        }
        else {
            // Gradient coordinates are relative to canvas dimensions, offset by position
            grad = ctx.createLinearGradient(start[0] * canvasWidth - pos[0], start[1] * canvasHeight - pos[1], end[0] * canvasWidth - pos[0], end[1] * canvasHeight - pos[1]);
        }
    }
    else {
        // Radial gradient
        const center = g.center || [0.5, 0.5];
        const radius = (g.radius || 0.5) * Math.max(canvasWidth, canvasHeight);
        if (isFillParent) {
            grad = ctx.createRadialGradient(center[0] * canvasWidth, center[1] * canvasHeight, 0, center[0] * canvasWidth, center[1] * canvasHeight, radius);
        }
        else {
            grad = ctx.createRadialGradient(0, 0, 0, 0, 0, radius);
        }
    }
    const colors = g.colors || [];
    for (const stop of colors) {
        grad.addColorStop(stop.offset, parseHexColor(stop.color));
    }
    ctx.fillStyle = grad;
    if (isFillParent) {
        ctx.fillRect(0, 0, canvasWidth, canvasHeight);
    }
    else {
        ctx.fillRect(-canvasWidth / 2, -canvasHeight / 2, canvasWidth, canvasHeight);
    }
}
// ── Round Rect Helper ─────────────────────────────────────────────────────────
function drawRoundRect(ctx, x, y, w, h, r) {
    const radius = Math.min(r, w / 2, h / 2);
    ctx.beginPath();
    ctx.moveTo(x + radius, y);
    ctx.lineTo(x + w - radius, y);
    ctx.quadraticCurveTo(x + w, y, x + w, y + radius);
    ctx.lineTo(x + w, y + h - radius);
    ctx.quadraticCurveTo(x + w, y + h, x + w - radius, y + h);
    ctx.lineTo(x + radius, y + h);
    ctx.quadraticCurveTo(x, y + h, x, y + h - radius);
    ctx.lineTo(x, y + radius);
    ctx.quadraticCurveTo(x, y, x + radius, y);
    ctx.closePath();
}
// ── Blend Mode Mapping ────────────────────────────────────────────────────────
function mapBlendMode(mode) {
    const map = {
        normal: 'source-over',
        multiply: 'multiply',
        screen: 'screen',
        overlay: 'overlay',
        darken: 'darken',
        lighten: 'lighten',
        color_dodge: 'color-dodge',
        color_burn: 'color-burn',
        hard_light: 'hard-light',
        soft_light: 'soft-light',
        difference: 'difference',
        exclusion: 'exclusion',
        add: 'lighter',
    };
    return map[mode] || 'source-over';
}
// ── Effects ───────────────────────────────────────────────────────────────────
// eslint-disable-next-line @typescript-eslint/no-explicit-any
function applyPreDrawEffects(ctx, effects) {
    if (!effects || !Array.isArray(effects))
        return;
    const filters = [];
    for (const effect of effects) {
        switch (effect.type) {
            case 'drop_shadow':
                ctx.shadowColor = effect.color || '#000000';
                ctx.shadowBlur = effect.blur || 8;
                ctx.shadowOffsetX = effect.offset_x || 0;
                ctx.shadowOffsetY = effect.offset_y || 0;
                break;
            case 'gaussian_blur':
                filters.push(`blur(${effect.radius || 0}px)`);
                break;
            case 'brightness_contrast': {
                // Canvas filter: brightness is 0-2 range (1 = normal), contrast likewise
                const brightness = 1 + (effect.brightness || 0) / 100;
                const contrast = 1 + (effect.contrast || 0) / 100;
                filters.push(`brightness(${brightness})`);
                filters.push(`contrast(${contrast})`);
                break;
            }
            case 'hue_saturation': {
                const hue = effect.hue || 0;
                const saturation = 1 + (effect.saturation || 0) / 100;
                const lightness = 1 + (effect.lightness || 0) / 100;
                filters.push(`hue-rotate(${hue}deg)`);
                filters.push(`saturate(${saturation})`);
                filters.push(`brightness(${lightness})`);
                break;
            }
            case 'invert':
                filters.push('invert(1)');
                break;
            case 'glow':
                // Approximate glow with drop shadow (no offset)
                ctx.shadowColor = effect.color || '#ffffff';
                ctx.shadowBlur = effect.radius || 10;
                ctx.shadowOffsetX = 0;
                ctx.shadowOffsetY = 0;
                break;
        }
    }
    if (filters.length > 0) {
        ctx.filter = filters.join(' ');
    }
}
// ── Masks ─────────────────────────────────────────────────────────────────────
// eslint-disable-next-line @typescript-eslint/no-explicit-any
function applyMasks(ctx, masks) {
    for (const mask of masks) {
        const path = mask.path;
        if (!path)
            continue;
        ctx.beginPath();
        switch (path.type) {
            case 'rect': {
                const cr = path.corner_radius || 0;
                if (cr > 0) {
                    drawRoundRect(ctx, path.x || 0, path.y || 0, path.width || 0, path.height || 0, cr);
                }
                else {
                    ctx.rect(path.x || 0, path.y || 0, path.width || 0, path.height || 0);
                }
                break;
            }
            case 'ellipse': {
                const cx = path.cx || 0;
                const cy = path.cy || 0;
                const rx = path.rx || 0;
                const ry = path.ry || 0;
                ctx.ellipse(cx, cy, rx, ry, 0, 0, Math.PI * 2);
                break;
            }
            case 'path': {
                const points = path.points;
                if (points && points.length > 0) {
                    ctx.moveTo(points[0][0], points[0][1]);
                    for (let i = 1; i < points.length; i++) {
                        ctx.lineTo(points[i][0], points[i][1]);
                    }
                    if (path.closed) {
                        ctx.closePath();
                    }
                }
                break;
            }
        }
        // Mask mode determines clip behavior
        // Canvas 2D only supports intersect-style clipping, so subtract/difference
        // are best-effort (clip with evenodd).
        const mode = mask.mode || 'add';
        if (mode === 'subtract' || mode === 'difference') {
            ctx.clip('evenodd');
        }
        else {
            ctx.clip();
        }
    }
}
// ── Color Parsing ─────────────────────────────────────────────────────────────
function parseHexColor(hex) {
    if (!hex)
        return '#000000';
    return hex.startsWith('#') ? hex : `#${hex}`;
}
//# sourceMappingURL=renderer.js.map