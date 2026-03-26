import { Scene, keyframes, interpolate, spring, Easing, Effects, rect, ellipse, line, polygon, star, linearGradient, radialGradient } from './index.js';
import { test, describe } from 'node:test';
import assert from 'node:assert';
describe('@mmot/react', () => {
    // -------------------------------------------------------------------------
    // Scene builder
    // -------------------------------------------------------------------------
    test('Scene creates valid JSON structure', () => {
        const scene = new Scene({ width: 1920, height: 1080, fps: 30, durationInFrames: 90 });
        scene.addSolid('bg', { color: '#0a0a1a', fill: 'parent' });
        scene.addText('title', {
            text: 'Hello',
            font: { family: 'Inter', size: 72, weight: 700, color: '#ffffff' },
            transform: { position: [960, 540] },
        });
        const json = JSON.parse(scene.toString());
        assert.strictEqual(json.version, '1.0');
        assert.strictEqual(json.meta.width, 1920);
        assert.strictEqual(json.meta.height, 1080);
        assert.strictEqual(json.meta.fps, 30);
        assert.strictEqual(json.meta.duration, 90);
        assert.strictEqual(json.meta.root, 'main');
        assert.strictEqual(json.compositions.main.layers.length, 2);
    });
    test('Scene defaults are applied', () => {
        const scene = new Scene({ width: 640, height: 360, fps: 30, durationInFrames: 30 });
        scene.addSolid('s', { color: '#ff0000' });
        const json = JSON.parse(scene.toString());
        assert.strictEqual(json.meta.name, 'Untitled');
        assert.strictEqual(json.meta.background, '#000000');
        const layer = json.compositions.main.layers[0];
        assert.strictEqual(layer.in, 0);
        assert.strictEqual(layer.out, 30);
    });
    test('Scene with custom name and background', () => {
        const scene = new Scene({
            width: 100, height: 100, fps: 60, durationInFrames: 120,
            name: 'Test Scene', background: '#1a1a2e',
        });
        const json = JSON.parse(scene.toString());
        assert.strictEqual(json.meta.name, 'Test Scene');
        assert.strictEqual(json.meta.background, '#1a1a2e');
    });
    // -------------------------------------------------------------------------
    // Layer types
    // -------------------------------------------------------------------------
    test('addSolid creates solid layer', () => {
        const scene = new Scene({ width: 100, height: 100, fps: 30, durationInFrames: 30 });
        scene.addSolid('bg', {
            color: '#1a1a2e',
            fill: 'parent',
            blendMode: 'screen',
            effects: [Effects.blur(5)],
        });
        const layer = JSON.parse(scene.toString()).compositions.main.layers[0];
        assert.strictEqual(layer.type, 'solid');
        assert.strictEqual(layer.color, '#1a1a2e');
        assert.strictEqual(layer.fill, 'parent');
        assert.strictEqual(layer.blend_mode, 'screen');
        assert.strictEqual(layer.effects[0].type, 'gaussian_blur');
    });
    test('addText creates text layer with font', () => {
        const scene = new Scene({ width: 100, height: 100, fps: 30, durationInFrames: 30 });
        scene.addText('title', {
            text: 'Hello Mercury',
            font: { family: 'Arial', size: 48, weight: 700, color: '#ffffff' },
            align: 'center',
            transform: { position: [50, 50] },
        });
        const layer = JSON.parse(scene.toString()).compositions.main.layers[0];
        assert.strictEqual(layer.type, 'text');
        assert.strictEqual(layer.text, 'Hello Mercury');
        assert.strictEqual(layer.font.family, 'Arial');
        assert.strictEqual(layer.font.size, 48);
        assert.strictEqual(layer.font.weight, 700);
        assert.strictEqual(layer.align, 'center');
    });
    test('addText applies font defaults', () => {
        const scene = new Scene({ width: 100, height: 100, fps: 30, durationInFrames: 30 });
        scene.addText('t', { text: 'Hi' });
        const layer = JSON.parse(scene.toString()).compositions.main.layers[0];
        assert.strictEqual(layer.font.family, 'Arial');
        assert.strictEqual(layer.font.size, 32);
        assert.strictEqual(layer.font.weight, 400);
        assert.strictEqual(layer.font.color, '#ffffff');
    });
    test('addShape creates shape layer', () => {
        const scene = new Scene({ width: 100, height: 100, fps: 30, durationInFrames: 30 });
        scene.addShape('box', {
            shape: rect(80, 80, { cornerRadius: 10, fill: '#533483' }),
            parent: 'null_parent',
            blendMode: 'overlay',
            motionBlur: true,
            pathAnimation: { points: [[0, 0], [100, 100]], autoOrient: true },
        });
        const layer = JSON.parse(scene.toString()).compositions.main.layers[0];
        assert.strictEqual(layer.type, 'shape');
        assert.strictEqual(layer.shape.shape_type, 'rect');
        assert.strictEqual(layer.shape.width, 80);
        assert.strictEqual(layer.shape.corner_radius, 10);
        assert.strictEqual(layer.parent, 'null_parent');
        assert.strictEqual(layer.blend_mode, 'overlay');
        assert.strictEqual(layer.motion_blur, true);
        assert.deepStrictEqual(layer.path_animation.points, [[0, 0], [100, 100]]);
        assert.strictEqual(layer.path_animation.auto_orient, true);
    });
    test('addGradient creates linear gradient layer', () => {
        const scene = new Scene({ width: 100, height: 100, fps: 30, durationInFrames: 30 });
        scene.addGradient('grad', {
            gradient: linearGradient([{ offset: 0, color: '#ff0000' }, { offset: 1, color: '#0000ff' }], [0, 0], [1, 1]),
            fill: 'parent',
        });
        const layer = JSON.parse(scene.toString()).compositions.main.layers[0];
        assert.strictEqual(layer.type, 'gradient');
        assert.strictEqual(layer.gradient.gradient_type, 'linear');
        assert.deepStrictEqual(layer.gradient.start, [0, 0]);
        assert.deepStrictEqual(layer.gradient.end, [1, 1]);
        assert.strictEqual(layer.gradient.colors.length, 2);
        assert.strictEqual(layer.fill, 'parent');
    });
    test('addGradient creates radial gradient layer', () => {
        const scene = new Scene({ width: 100, height: 100, fps: 30, durationInFrames: 30 });
        scene.addGradient('grad', {
            gradient: radialGradient([{ offset: 0, color: '#ffffff' }, { offset: 1, color: '#000000' }], [0.5, 0.5], 0.8),
        });
        const layer = JSON.parse(scene.toString()).compositions.main.layers[0];
        assert.strictEqual(layer.gradient.gradient_type, 'radial');
        assert.deepStrictEqual(layer.gradient.center, [0.5, 0.5]);
        assert.strictEqual(layer.gradient.radius, 0.8);
    });
    test('addImage creates image layer', () => {
        const scene = new Scene({ width: 100, height: 100, fps: 30, durationInFrames: 30 });
        scene.addImage('logo', {
            src: 'assets/logo.png',
            transform: { position: [50, 50] },
        });
        const layer = JSON.parse(scene.toString()).compositions.main.layers[0];
        assert.strictEqual(layer.type, 'image');
        assert.strictEqual(layer.src, 'assets/logo.png');
    });
    test('addVideo creates video layer', () => {
        const scene = new Scene({ width: 100, height: 100, fps: 30, durationInFrames: 60 });
        scene.addVideo('clip', {
            src: 'footage/clip.mp4',
            trimStart: 1.5,
            trimEnd: 5.0,
            transform: { position: [50, 50] },
        });
        const layer = JSON.parse(scene.toString()).compositions.main.layers[0];
        assert.strictEqual(layer.type, 'video');
        assert.strictEqual(layer.src, 'footage/clip.mp4');
        assert.strictEqual(layer.trim_start, 1.5);
        assert.strictEqual(layer.trim_end, 5.0);
    });
    test('addAudio creates audio layer', () => {
        const scene = new Scene({ width: 100, height: 100, fps: 30, durationInFrames: 30 });
        scene.addAudio('music', { src: 'audio/track.mp3', volume: 0.8 });
        const layer = JSON.parse(scene.toString()).compositions.main.layers[0];
        assert.strictEqual(layer.type, 'audio');
        assert.strictEqual(layer.src, 'audio/track.mp3');
        assert.strictEqual(layer.volume, 0.8);
    });
    test('addNull creates null layer', () => {
        const scene = new Scene({ width: 100, height: 100, fps: 30, durationInFrames: 60 });
        scene.addNull('parent', {
            transform: {
                position: keyframes([
                    { frame: 0, value: [50, 50] },
                    { frame: 60, value: [80, 80] },
                ]),
            },
        });
        const layer = JSON.parse(scene.toString()).compositions.main.layers[0];
        assert.strictEqual(layer.type, 'null');
        assert.strictEqual(layer.transform.position.length, 2);
        assert.strictEqual(layer.transform.position[0].t, 0);
        assert.deepStrictEqual(layer.transform.position[0].v, [50, 50]);
    });
    test('addComposition creates composition reference layer', () => {
        const scene = new Scene({ width: 100, height: 100, fps: 30, durationInFrames: 30 });
        scene.addComposition('comp_ref', { compositionId: 'intro' });
        const layer = JSON.parse(scene.toString()).compositions.main.layers[0];
        assert.strictEqual(layer.type, 'composition');
        assert.strictEqual(layer.composition_id, 'intro');
    });
    // -------------------------------------------------------------------------
    // Sequence mode
    // -------------------------------------------------------------------------
    test('setSequence enables sequence mode', () => {
        const scene = new Scene({ width: 100, height: 100, fps: 30, durationInFrames: 60 });
        scene.setSequence({ type: 'crossfade', duration: 10 });
        scene.addSolid('red', { color: '#ff0000', out: 30 });
        scene.addSolid('blue', { color: '#0000ff', out: 30 });
        const json = JSON.parse(scene.toString());
        assert.strictEqual(json.compositions.main.sequence, true);
        assert.strictEqual(json.compositions.main.transition.type, 'crossfade');
        assert.strictEqual(json.compositions.main.transition.duration, 10);
    });
    // -------------------------------------------------------------------------
    // Transform serialization
    // -------------------------------------------------------------------------
    test('static transform values serialize directly', () => {
        const scene = new Scene({ width: 100, height: 100, fps: 30, durationInFrames: 30 });
        scene.addSolid('s', {
            color: '#ff0000',
            transform: {
                position: [50, 50],
                scale: [1, 1],
                rotation: 45,
                opacity: 0.8,
            },
        });
        const t = JSON.parse(scene.toString()).compositions.main.layers[0].transform;
        assert.deepStrictEqual(t.position, [50, 50]);
        assert.deepStrictEqual(t.scale, [1, 1]);
        assert.strictEqual(t.rotation, 45);
        assert.strictEqual(t.opacity, 0.8);
    });
    test('animated transform values serialize as keyframe arrays', () => {
        const scene = new Scene({ width: 100, height: 100, fps: 30, durationInFrames: 30 });
        scene.addSolid('s', {
            color: '#ff0000',
            transform: {
                opacity: [
                    { frame: 0, value: 0, easing: 'ease_in_out' },
                    { frame: 15, value: 1, easing: 'ease_in_out' },
                    { frame: 25, value: 1, easing: 'ease_out' },
                    { frame: 30, value: 0 },
                ],
            },
        });
        const t = JSON.parse(scene.toString()).compositions.main.layers[0].transform;
        assert.strictEqual(t.opacity.length, 4);
        assert.strictEqual(t.opacity[0].t, 0);
        assert.strictEqual(t.opacity[0].v, 0);
        assert.strictEqual(t.opacity[0].easing, 'ease_in_out');
        assert.strictEqual(t.opacity[3].t, 30);
        assert.strictEqual(t.opacity[3].v, 0);
        assert.strictEqual(t.opacity[3].easing, undefined);
    });
    test('spring easing serializes correctly', () => {
        const scene = new Scene({ width: 200, height: 200, fps: 30, durationInFrames: 60 });
        scene.addSolid('bouncer', {
            color: '#ff6600',
            transform: {
                position: [
                    { frame: 0, value: [100, 180], easing: { type: 'spring', mass: 1.0, stiffness: 170.0, damping: 26.0 } },
                    { frame: 60, value: [100, 100] },
                ],
            },
        });
        const t = JSON.parse(scene.toString()).compositions.main.layers[0].transform;
        assert.strictEqual(t.position[0].easing.type, 'spring');
        assert.strictEqual(t.position[0].easing.stiffness, 170);
        assert.strictEqual(t.position[0].easing.damping, 26);
        assert.strictEqual(t.position[0].easing.mass, 1);
    });
    // -------------------------------------------------------------------------
    // interpolate()
    // -------------------------------------------------------------------------
    test('interpolate creates two keyframes', () => {
        const kfs = interpolate(0, [0, 30], [0, 1]);
        assert.strictEqual(kfs.length, 2);
        assert.strictEqual(kfs[0].frame, 0);
        assert.strictEqual(kfs[0].value, 0);
        assert.strictEqual(kfs[0].easing, 'ease_in_out');
        assert.strictEqual(kfs[1].frame, 30);
        assert.strictEqual(kfs[1].value, 1);
        assert.strictEqual(kfs[1].easing, undefined);
    });
    test('interpolate uses custom easing', () => {
        const kfs = interpolate(0, [10, 50], [100, 200], { easing: Easing.easeOut });
        assert.strictEqual(kfs[0].easing, 'ease_out');
        assert.strictEqual(kfs[0].frame, 10);
        assert.strictEqual(kfs[0].value, 100);
        assert.strictEqual(kfs[1].frame, 50);
        assert.strictEqual(kfs[1].value, 200);
    });
    test('interpolate with bezier easing', () => {
        const kfs = interpolate(0, [0, 30], [0, 1], { easing: Easing.bezier(0.42, 0, 0.58, 1) });
        const easing = kfs[0].easing;
        assert.strictEqual(easing.type, 'cubic_bezier');
        assert.strictEqual(easing.x1, 0.42);
        assert.strictEqual(easing.y2, 1);
    });
    // -------------------------------------------------------------------------
    // spring()
    // -------------------------------------------------------------------------
    test('spring creates keyframes with spring easing', () => {
        const kfs = spring({ fps: 30 });
        assert.strictEqual(kfs.length, 2);
        assert.strictEqual(kfs[0].frame, 0);
        assert.strictEqual(kfs[0].value, 0);
        const easing = kfs[0].easing;
        assert.strictEqual(easing.type, 'spring');
        assert.strictEqual(easing.stiffness, 170);
        assert.strictEqual(easing.damping, 26);
        assert.strictEqual(easing.mass, 1);
        assert.strictEqual(kfs[1].frame, 30);
        assert.strictEqual(kfs[1].value, 1);
    });
    test('spring with custom config', () => {
        const kfs = spring({ fps: 60, from: 100, to: 200, config: { stiffness: 300, damping: 15, mass: 2 } });
        assert.strictEqual(kfs[0].value, 100);
        assert.strictEqual(kfs[1].value, 200);
        assert.strictEqual(kfs[1].frame, 60);
        const easing = kfs[0].easing;
        assert.strictEqual(easing.stiffness, 300);
        assert.strictEqual(easing.damping, 15);
        assert.strictEqual(easing.mass, 2);
    });
    test('spring defaults', () => {
        const kfs = spring();
        assert.strictEqual(kfs[0].value, 0);
        assert.strictEqual(kfs[1].value, 1);
        assert.strictEqual(kfs[1].frame, 30);
    });
    // -------------------------------------------------------------------------
    // Shape helpers
    // -------------------------------------------------------------------------
    test('rect() creates rect shape', () => {
        const s = rect(200, 100, { cornerRadius: 10, fill: '#ff0000', stroke: { color: '#000', width: 2 } });
        assert.strictEqual(s.shape_type, 'rect');
        if (s.shape_type !== 'rect')
            throw new Error('unreachable');
        assert.strictEqual(s.width, 200);
        assert.strictEqual(s.height, 100);
        assert.strictEqual(s.corner_radius, 10);
        assert.strictEqual(s.fill, '#ff0000');
        assert.strictEqual(s.stroke?.color, '#000');
        assert.strictEqual(s.stroke?.width, 2);
    });
    test('rect() without options', () => {
        const s = rect(50, 50);
        assert.strictEqual(s.shape_type, 'rect');
        if (s.shape_type !== 'rect')
            throw new Error('unreachable');
        assert.strictEqual(s.width, 50);
        assert.strictEqual(s.corner_radius, undefined);
        assert.strictEqual(s.fill, undefined);
    });
    test('ellipse() creates ellipse shape', () => {
        const s = ellipse(100, 80, { fill: '#00ff00' });
        assert.strictEqual(s.shape_type, 'ellipse');
        if (s.shape_type !== 'ellipse')
            throw new Error('unreachable');
        assert.strictEqual(s.width, 100);
        assert.strictEqual(s.height, 80);
        assert.strictEqual(s.fill, '#00ff00');
    });
    test('line() creates line shape', () => {
        const s = line(0, 0, 100, 50, { color: '#fff', width: 2 });
        assert.strictEqual(s.shape_type, 'line');
        if (s.shape_type !== 'line')
            throw new Error('unreachable');
        assert.strictEqual(s.x1, 0);
        assert.strictEqual(s.y1, 0);
        assert.strictEqual(s.x2, 100);
        assert.strictEqual(s.y2, 50);
        assert.strictEqual(s.stroke.color, '#fff');
    });
    test('polygon() creates polygon shape', () => {
        const s = polygon([[0, -50], [50, 50], [-50, 50]], { fill: '#ff0000' });
        assert.strictEqual(s.shape_type, 'polygon');
        if (s.shape_type !== 'polygon')
            throw new Error('unreachable');
        assert.strictEqual(s.points.length, 3);
        assert.deepStrictEqual(s.points[0], [0, -50]);
        assert.strictEqual(s.fill, '#ff0000');
    });
    test('star() generates correct polygon points', () => {
        const s = star(100, 50, 5, { fill: '#FFD700' });
        assert.strictEqual(s.shape_type, 'polygon');
        if (s.shape_type !== 'polygon')
            throw new Error('unreachable');
        assert.strictEqual(s.points.length, 10); // 5 outer + 5 inner
        assert.strictEqual(s.fill, '#FFD700');
        // First point should be at top (angle = -PI/2, radius = outer)
        const [x0, y0] = s.points[0];
        assert(Math.abs(x0) < 0.01, `Expected x0 near 0, got ${x0}`);
        assert(Math.abs(y0 - (-100)) < 0.01, `Expected y0 near -100, got ${y0}`);
    });
    test('star() with stroke', () => {
        const s = star(80, 35, 5, { fill: '#FFD700', stroke: { color: '#FFF8DC', width: 1.5 } });
        if (s.shape_type !== 'polygon')
            throw new Error('unreachable');
        assert.strictEqual(s.stroke?.color, '#FFF8DC');
        assert.strictEqual(s.stroke?.width, 1.5);
    });
    // -------------------------------------------------------------------------
    // Gradient helpers
    // -------------------------------------------------------------------------
    test('linearGradient() creates linear gradient config', () => {
        const g = linearGradient([{ offset: 0, color: '#ff0000' }, { offset: 1, color: '#0000ff' }]);
        assert.strictEqual(g.gradient_type, 'linear');
        if (g.gradient_type !== 'linear')
            throw new Error('unreachable');
        assert.deepStrictEqual(g.start, [0, 0]);
        assert.deepStrictEqual(g.end, [1, 1]);
        assert.strictEqual(g.colors.length, 2);
    });
    test('linearGradient() with custom start/end', () => {
        const g = linearGradient([{ offset: 0, color: '#000' }], [0, 0.5], [1, 0.5]);
        if (g.gradient_type !== 'linear')
            throw new Error('unreachable');
        assert.deepStrictEqual(g.start, [0, 0.5]);
        assert.deepStrictEqual(g.end, [1, 0.5]);
    });
    test('radialGradient() creates radial gradient config', () => {
        const g = radialGradient([{ offset: 0, color: '#fff' }, { offset: 1, color: '#000' }], [0.3, 0.7], 0.9);
        assert.strictEqual(g.gradient_type, 'radial');
        if (g.gradient_type !== 'radial')
            throw new Error('unreachable');
        assert.deepStrictEqual(g.center, [0.3, 0.7]);
        assert.strictEqual(g.radius, 0.9);
    });
    test('radialGradient() defaults', () => {
        const g = radialGradient([{ offset: 0, color: '#fff' }]);
        if (g.gradient_type !== 'radial')
            throw new Error('unreachable');
        assert.deepStrictEqual(g.center, [0.5, 0.5]);
        assert.strictEqual(g.radius, 0.5);
    });
    // -------------------------------------------------------------------------
    // Effects helpers
    // -------------------------------------------------------------------------
    test('Effects.blur', () => {
        const e = Effects.blur(5);
        assert.strictEqual(e.type, 'gaussian_blur');
        if (e.type !== 'gaussian_blur')
            throw new Error('unreachable');
        assert.strictEqual(e.radius, 5);
    });
    test('Effects.shadow with defaults', () => {
        const e = Effects.shadow();
        assert.strictEqual(e.type, 'drop_shadow');
        if (e.type !== 'drop_shadow')
            throw new Error('unreachable');
        assert.strictEqual(e.color, '#000000');
        assert.strictEqual(e.offset_x, 0);
        assert.strictEqual(e.offset_y, 4);
        assert.strictEqual(e.blur, 8);
        assert.strictEqual(e.opacity, 0.5);
    });
    test('Effects.shadow with custom values', () => {
        const e = Effects.shadow({ color: '#ff0000', x: 2, y: 6, blur: 10, opacity: 0.8 });
        if (e.type !== 'drop_shadow')
            throw new Error('unreachable');
        assert.strictEqual(e.color, '#ff0000');
        assert.strictEqual(e.offset_x, 2);
        assert.strictEqual(e.offset_y, 6);
        assert.strictEqual(e.blur, 10);
        assert.strictEqual(e.opacity, 0.8);
    });
    test('Effects.glow', () => {
        const e = Effects.glow({ color: '#e94560', radius: 15, intensity: 0.8 });
        assert.strictEqual(e.type, 'glow');
        if (e.type !== 'glow')
            throw new Error('unreachable');
        assert.strictEqual(e.color, '#e94560');
        assert.strictEqual(e.radius, 15);
        assert.strictEqual(e.intensity, 0.8);
    });
    test('Effects.glow defaults', () => {
        const e = Effects.glow();
        if (e.type !== 'glow')
            throw new Error('unreachable');
        assert.strictEqual(e.color, '#ffffff');
        assert.strictEqual(e.radius, 10);
        assert.strictEqual(e.intensity, 0.8);
    });
    test('Effects.brightnessContrast', () => {
        const e = Effects.brightnessContrast(5, 10);
        assert.strictEqual(e.type, 'brightness_contrast');
        if (e.type !== 'brightness_contrast')
            throw new Error('unreachable');
        assert.strictEqual(e.brightness, 5);
        assert.strictEqual(e.contrast, 10);
    });
    test('Effects.hueSaturation', () => {
        const e = Effects.hueSaturation(30, -10, 5);
        assert.strictEqual(e.type, 'hue_saturation');
        if (e.type !== 'hue_saturation')
            throw new Error('unreachable');
        assert.strictEqual(e.hue, 30);
        assert.strictEqual(e.saturation, -10);
        assert.strictEqual(e.lightness, 5);
    });
    test('Effects.hueSaturation lightness default', () => {
        const e = Effects.hueSaturation(0, 0);
        if (e.type !== 'hue_saturation')
            throw new Error('unreachable');
        assert.strictEqual(e.lightness, 0);
    });
    test('Effects.invert', () => {
        const e = Effects.invert();
        assert.strictEqual(e.type, 'invert');
    });
    test('Effects.tint', () => {
        const e = Effects.tint('#ff6600', 0.7);
        assert.strictEqual(e.type, 'tint');
        if (e.type !== 'tint')
            throw new Error('unreachable');
        assert.strictEqual(e.color, '#ff6600');
        assert.strictEqual(e.amount, 0.7);
    });
    test('Effects.tint default amount', () => {
        const e = Effects.tint('#ff0000');
        if (e.type !== 'tint')
            throw new Error('unreachable');
        assert.strictEqual(e.amount, 1.0);
    });
    test('Effects.fill', () => {
        const e = Effects.fill('#000000', 0.5);
        assert.strictEqual(e.type, 'fill');
        if (e.type !== 'fill')
            throw new Error('unreachable');
        assert.strictEqual(e.color, '#000000');
        assert.strictEqual(e.opacity, 0.5);
    });
    // -------------------------------------------------------------------------
    // keyframes() helper
    // -------------------------------------------------------------------------
    test('keyframes helper returns keyframes that serialize correctly', () => {
        const scene = new Scene({ width: 100, height: 100, fps: 30, durationInFrames: 30 });
        scene.addSolid('s', {
            color: '#ff0000',
            transform: {
                position: keyframes([
                    { frame: 0, value: [0, 0], easing: 'ease_out' },
                    { frame: 30, value: [100, 100] },
                ]),
            },
        });
        const t = JSON.parse(scene.toString()).compositions.main.layers[0].transform;
        assert.strictEqual(t.position[0].t, 0);
        assert.deepStrictEqual(t.position[0].v, [0, 0]);
        assert.strictEqual(t.position[0].easing, 'ease_out');
        assert.strictEqual(t.position[1].t, 30);
        assert.deepStrictEqual(t.position[1].v, [100, 100]);
        assert.strictEqual(t.position[1].easing, undefined);
    });
    // -------------------------------------------------------------------------
    // Easing presets
    // -------------------------------------------------------------------------
    test('Easing presets', () => {
        assert.strictEqual(Easing.linear, 'linear');
        assert.strictEqual(Easing.easeIn, 'ease_in');
        assert.strictEqual(Easing.easeOut, 'ease_out');
        assert.strictEqual(Easing.easeInOut, 'ease_in_out');
    });
    test('Easing.bezier', () => {
        const b = Easing.bezier(0.25, 0.1, 0.25, 1.0);
        assert.strictEqual(b.type, 'cubic_bezier');
        assert.strictEqual(b.x1, 0.25);
        assert.strictEqual(b.y1, 0.1);
        assert.strictEqual(b.x2, 0.25);
        assert.strictEqual(b.y2, 1.0);
    });
    // -------------------------------------------------------------------------
    // Fluent API (method chaining)
    // -------------------------------------------------------------------------
    test('Scene methods return this for chaining', () => {
        const scene = new Scene({ width: 100, height: 100, fps: 30, durationInFrames: 30 });
        const result = scene
            .addSolid('bg', { color: '#000' })
            .addText('t', { text: 'Hi' })
            .addShape('s', { shape: rect(10, 10) })
            .addNull('n');
        assert.strictEqual(result, scene);
        const json = JSON.parse(scene.toString());
        assert.strictEqual(json.compositions.main.layers.length, 4);
    });
    // -------------------------------------------------------------------------
    // Integration: matching existing fixture format
    // -------------------------------------------------------------------------
    test('output matches minimal.mmot.json structure', () => {
        const scene = new Scene({
            width: 640, height: 360, fps: 30, durationInFrames: 30,
            name: 'Minimal', background: '#000000',
        });
        scene.addSolid('bg', {
            color: '#1a1a2e',
            transform: {
                position: [320, 180],
                scale: [1, 1],
                opacity: 1.0,
                rotation: 0.0,
            },
        });
        const json = JSON.parse(scene.toString());
        assert.strictEqual(json.version, '1.0');
        assert.strictEqual(json.meta.name, 'Minimal');
        assert.strictEqual(json.meta.root, 'main');
        const layer = json.compositions.main.layers[0];
        assert.strictEqual(layer.id, 'bg');
        assert.strictEqual(layer.type, 'solid');
        assert.strictEqual(layer.color, '#1a1a2e');
        assert.deepStrictEqual(layer.transform.position, [320, 180]);
        assert.deepStrictEqual(layer.transform.scale, [1, 1]);
        assert.strictEqual(layer.transform.opacity, 1);
        assert.strictEqual(layer.transform.rotation, 0);
    });
    test('output matches text_fade.mmot.json structure', () => {
        const scene = new Scene({
            width: 640, height: 360, fps: 30, durationInFrames: 30,
            name: 'TextFade', background: '#000000',
        });
        scene.addText('title', {
            text: 'Hello Mercury',
            font: { family: 'Arial', size: 48, weight: 700, color: '#ffffff' },
            transform: {
                position: [320, 180],
                scale: [1, 1],
                opacity: [
                    { frame: 0, value: 0, easing: 'ease_in_out' },
                    { frame: 15, value: 1, easing: 'ease_in_out' },
                    { frame: 25, value: 1, easing: 'ease_out' },
                    { frame: 30, value: 0 },
                ],
                rotation: 0,
            },
        });
        const json = JSON.parse(scene.toString());
        const layer = json.compositions.main.layers[0];
        assert.strictEqual(layer.type, 'text');
        assert.strictEqual(layer.text, 'Hello Mercury');
        assert.strictEqual(layer.transform.opacity.length, 4);
        assert.strictEqual(layer.transform.opacity[0].t, 0);
        assert.strictEqual(layer.transform.opacity[0].v, 0);
        assert.strictEqual(layer.transform.opacity[0].easing, 'ease_in_out');
    });
    test('output matches sequence.mmot.json structure', () => {
        const scene = new Scene({
            width: 100, height: 100, fps: 30, durationInFrames: 60,
            name: 'SequenceTest', background: '#000000',
        });
        scene.setSequence({ type: 'crossfade', duration: 10 });
        scene.addSolid('red', {
            color: '#ff0000',
            out: 30,
            transform: { position: [50, 50], scale: [1, 1], opacity: 1, rotation: 0 },
        });
        scene.addSolid('blue', {
            color: '#0000ff',
            out: 30,
            transform: { position: [50, 50], scale: [1, 1], opacity: 1, rotation: 0 },
        });
        const json = JSON.parse(scene.toString());
        assert.strictEqual(json.compositions.main.sequence, true);
        assert.strictEqual(json.compositions.main.transition.type, 'crossfade');
        assert.strictEqual(json.compositions.main.layers.length, 2);
    });
    // -------------------------------------------------------------------------
    // Complete example scene
    // -------------------------------------------------------------------------
    test('complete example scene with all features', () => {
        const scene = new Scene({
            width: 1080, height: 1080, fps: 30, durationInFrames: 45,
            background: '#0a0a1a', name: 'Star Demo',
        });
        scene.addGradient('bg', {
            gradient: linearGradient([{ offset: 0, color: '#1a1a3e' }, { offset: 1, color: '#0a0a1a' }]),
            fill: 'parent',
        });
        scene.addShape('star', {
            shape: star(80, 35, 5, { fill: '#FFD700', stroke: { color: '#FFF8DC', width: 1.5 } }),
            transform: {
                position: keyframes([
                    { frame: 0, value: [200, 540], easing: 'ease_in_out' },
                    { frame: 45, value: [880, 540] },
                ]),
                rotation: keyframes([
                    { frame: 0, value: 0 },
                    { frame: 45, value: 360 },
                ]),
                scale: keyframes([
                    { frame: 0, value: [0, 0], easing: Easing.easeOut },
                    { frame: 8, value: [1, 1] },
                    { frame: 37, value: [1, 1], easing: Easing.easeIn },
                    { frame: 45, value: [0, 0] },
                ]),
            },
            effects: [Effects.shadow({ color: '#FFD700', blur: 15, opacity: 0.8 })],
        });
        const json = JSON.parse(scene.toString());
        assert.strictEqual(json.compositions.main.layers.length, 2);
        // Verify the star layer has animated position
        const starLayer = json.compositions.main.layers[1];
        assert(Array.isArray(starLayer.transform.position));
        assert.strictEqual(starLayer.transform.position[0].t, 0);
        assert.deepStrictEqual(starLayer.transform.position[0].v, [200, 540]);
        // Verify animated rotation
        assert(Array.isArray(starLayer.transform.rotation));
        assert.strictEqual(starLayer.transform.rotation[1].v, 360);
        // Verify animated scale
        assert(Array.isArray(starLayer.transform.scale));
        assert.strictEqual(starLayer.transform.scale.length, 4);
        assert.strictEqual(starLayer.transform.scale[0].easing, 'ease_out');
        assert.strictEqual(starLayer.transform.scale[2].easing, 'ease_in');
        // Verify effects
        assert.strictEqual(starLayer.effects.length, 1);
        assert.strictEqual(starLayer.effects[0].type, 'drop_shadow');
        assert.strictEqual(starLayer.effects[0].color, '#FFD700');
        // Verify shape
        assert.strictEqual(starLayer.shape.shape_type, 'polygon');
        assert.strictEqual(starLayer.shape.points.length, 10);
        assert.strictEqual(starLayer.shape.fill, '#FFD700');
        assert.strictEqual(starLayer.shape.stroke.color, '#FFF8DC');
    });
});
