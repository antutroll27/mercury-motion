//! WCAG accessibility audit for `.mmot.json` scenes.
//!
//! Static analysis that detects seizure-triggering flash rates, insufficient
//! color contrast, small text, excessive motion intensity, and extreme
//! glow/brightness effects.

use crate::schema::{AnimatableValue, Effect, Layer, LayerContent, Scene};

// ── Public types ────────────────────────────────────────────────────────────────

/// Severity of an audit finding.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Info,
    Warning,
    Critical,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Info => write!(f, "INFO"),
            Self::Warning => write!(f, "WARNING"),
            Self::Critical => write!(f, "CRITICAL"),
        }
    }
}

/// Which audit rule produced a finding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuditRule {
    FlashRate,
    LargeAreaFlash,
    ColorContrast,
    TextSize,
    MotionIntensity,
    GlowIntensity,
    BrightnessExtreme,
}

/// A single audit finding with location and optional frame range.
#[derive(Debug, Clone)]
pub struct AuditFinding {
    pub severity: Severity,
    pub rule: AuditRule,
    pub message: String,
    /// JSON-Pointer-style path to the offending element.
    pub pointer: String,
    /// Optional (start_frame, end_frame) range where the violation occurs.
    pub frame_range: Option<(u64, u64)>,
}

/// Options controlling audit behaviour.
pub struct AuditOptions {
    /// WCAG contrast level to enforce.
    pub contrast_level: ContrastLevel,
    /// Rules to suppress (skip).
    pub suppress: Vec<AuditRule>,
}

impl Default for AuditOptions {
    fn default() -> Self {
        Self {
            contrast_level: ContrastLevel::AA,
            suppress: Vec::new(),
        }
    }
}

/// WCAG contrast conformance level.
#[derive(Debug, Clone)]
pub enum ContrastLevel {
    /// 4.5:1 minimum contrast ratio.
    AA,
    /// 7:1 minimum contrast ratio.
    AAA,
}

/// Aggregated audit report.
pub struct AuditReport {
    pub findings: Vec<AuditFinding>,
}

impl AuditReport {
    pub fn critical_count(&self) -> usize {
        self.findings
            .iter()
            .filter(|f| f.severity == Severity::Critical)
            .count()
    }

    pub fn warning_count(&self) -> usize {
        self.findings
            .iter()
            .filter(|f| f.severity == Severity::Warning)
            .count()
    }

    pub fn info_count(&self) -> usize {
        self.findings
            .iter()
            .filter(|f| f.severity == Severity::Info)
            .count()
    }

    /// Returns `true` when no findings of any severity were produced.
    pub fn is_clean(&self) -> bool {
        self.findings.is_empty()
    }
}

// ── Orchestrator ────────────────────────────────────────────────────────────────

/// Run all enabled accessibility checks on the scene and return a sorted report.
pub fn audit(scene: &Scene, opts: &AuditOptions) -> AuditReport {
    let mut findings = Vec::new();

    for (comp_id, comp) in &scene.compositions {
        for (idx, layer) in comp.layers.iter().enumerate() {
            let pointer = format!("/compositions/{}/layers[{}]", comp_id, idx);

            if !opts.suppress.contains(&AuditRule::FlashRate) {
                findings.extend(check_flash_rate(scene, &pointer, layer));
            }
            if !opts.suppress.contains(&AuditRule::ColorContrast) {
                findings.extend(check_color_contrast(
                    scene,
                    &pointer,
                    layer,
                    &opts.contrast_level,
                ));
            }
            if !opts.suppress.contains(&AuditRule::TextSize) {
                findings.extend(check_text_size(&pointer, layer));
            }
            if !opts.suppress.contains(&AuditRule::MotionIntensity) {
                findings.extend(check_motion_intensity(scene, &pointer, layer));
            }
            if !opts.suppress.contains(&AuditRule::GlowIntensity)
                || !opts.suppress.contains(&AuditRule::BrightnessExtreme)
            {
                findings.extend(check_glow_brightness(&pointer, layer, opts));
            }
        }
    }

    // Sort critical-first, then warning, then info.
    findings.sort_by(|a, b| a.severity.cmp(&b.severity).reverse());
    AuditReport { findings }
}

// ── Check: Flash rate ───────────────────────────────────────────────────────────

/// WCAG 2.3.1: Content must not flash more than 3 times per second.
///
/// We slide a 1-second window across the opacity keyframes and count how many
/// times opacity crosses the 0.5 threshold. If crossings > 6 (3 full on/off
/// cycles) the check emits a CRITICAL finding.
fn check_flash_rate(scene: &Scene, pointer: &str, layer: &Layer) -> Vec<AuditFinding> {
    let mut findings = Vec::new();

    let kfs = match &layer.transform.opacity {
        AnimatableValue::Animated(kfs) if kfs.len() >= 2 => kfs,
        _ => return findings,
    };

    let fps = scene.meta.fps;
    if fps <= 0.0 {
        return findings;
    }
    let one_second_frames = fps as u64;
    if one_second_frames == 0 {
        return findings;
    }

    let first_t = kfs.first().map(|k| k.t).unwrap_or(0);
    let last_t = kfs.last().map(|k| k.t).unwrap_or(0);

    // Slide a 1-second window from the first keyframe to the last.
    let mut window_start = first_t;
    while window_start + one_second_frames <= last_t {
        let window_end = window_start + one_second_frames;
        let crossings = count_threshold_crossings(kfs, 0.5, window_start, window_end);
        if crossings > 6 {
            findings.push(AuditFinding {
                severity: Severity::Critical,
                rule: AuditRule::FlashRate,
                message: format!(
                    "opacity crosses 0.5 threshold {} times in 1 second ({} flashes/sec exceeds WCAG 2.3.1 limit of 3)",
                    crossings,
                    crossings / 2,
                ),
                pointer: format!("{}/transform/opacity", pointer),
                frame_range: Some((window_start, window_end)),
            });
            // Only report once per layer — skip to the end of this window.
            break;
        }
        window_start += 1;
    }

    findings
}

/// Count how many times a sequence of keyframes crosses `threshold` within
/// `[start_frame, end_frame]`. Works by linearly interpolating between
/// keyframe pairs and checking side-of-threshold changes.
fn count_threshold_crossings(
    kfs: &[crate::schema::Keyframe<f64>],
    threshold: f64,
    start_frame: u64,
    end_frame: u64,
) -> usize {
    // Collect the values at every keyframe within the window, including
    // interpolated values at the window boundaries.
    let mut samples: Vec<f64> = Vec::new();

    for kf in kfs {
        if kf.t >= start_frame && kf.t <= end_frame {
            samples.push(kf.v);
        }
    }

    if samples.len() < 2 {
        return 0;
    }

    let mut crossings = 0usize;
    for window in samples.windows(2) {
        let prev_above = window[0] >= threshold;
        let curr_above = window[1] >= threshold;
        if prev_above != curr_above {
            crossings += 1;
        }
    }
    crossings
}

// ── Check: Color contrast ───────────────────────────────────────────────────────

/// WCAG 1.4.3 (AA) / 1.4.6 (AAA): Text must have sufficient contrast against
/// the background color.
fn check_color_contrast(
    scene: &Scene,
    pointer: &str,
    layer: &Layer,
    level: &ContrastLevel,
) -> Vec<AuditFinding> {
    let mut findings = Vec::new();

    let font = match &layer.content {
        LayerContent::Text { font, .. } => font,
        _ => return findings,
    };

    let fg = match parse_hex_color(&font.color) {
        Some(c) => c,
        None => return findings,
    };
    let bg = match parse_hex_color(&scene.meta.background) {
        Some(c) => c,
        None => return findings,
    };

    let ratio = contrast_ratio(fg, bg);

    let required = match level {
        ContrastLevel::AA => 4.5,
        ContrastLevel::AAA => 7.0,
    };
    let level_str = match level {
        ContrastLevel::AA => "AA",
        ContrastLevel::AAA => "AAA",
    };

    if ratio < required {
        findings.push(AuditFinding {
            severity: Severity::Warning,
            rule: AuditRule::ColorContrast,
            message: format!(
                "text color {} on background {} has contrast ratio {:.2}:1, below WCAG {} minimum of {:.1}:1",
                font.color, scene.meta.background, ratio, level_str, required,
            ),
            pointer: format!("{}/font/color", pointer),
            frame_range: None,
        });
    }

    findings
}

// ── Check: Text size ────────────────────────────────────────────────────────────

/// Small text (< 14px) may be hard to read in video content.
fn check_text_size(pointer: &str, layer: &Layer) -> Vec<AuditFinding> {
    let mut findings = Vec::new();

    let font = match &layer.content {
        LayerContent::Text { font, .. } => font,
        _ => return findings,
    };

    if font.size < 14.0 {
        findings.push(AuditFinding {
            severity: Severity::Info,
            rule: AuditRule::TextSize,
            message: format!(
                "text size {:.1}px is below 14px — may be hard to read in video",
                font.size,
            ),
            pointer: format!("{}/font/size", pointer),
            frame_range: None,
        });
    }

    findings
}

// ── Check: Motion intensity ─────────────────────────────────────────────────────

/// Excessive motion (position change > 50% of canvas diagonal per second)
/// can cause discomfort for vestibular-sensitive viewers.
fn check_motion_intensity(scene: &Scene, pointer: &str, layer: &Layer) -> Vec<AuditFinding> {
    let mut findings = Vec::new();

    let kfs = match &layer.transform.position {
        AnimatableValue::Animated(kfs) if kfs.len() >= 2 => kfs,
        _ => return findings,
    };

    let fps = scene.meta.fps;
    if fps <= 0.0 {
        return findings;
    }

    let diagonal = ((scene.meta.width as f64).powi(2) + (scene.meta.height as f64).powi(2)).sqrt();
    let threshold = diagonal * 0.5; // 50% of diagonal per second

    for window in kfs.windows(2) {
        let dt_frames = window[1].t.saturating_sub(window[0].t);
        if dt_frames == 0 {
            continue;
        }
        let dt_seconds = dt_frames as f64 / fps;

        let dx = window[1].v.x - window[0].v.x;
        let dy = window[1].v.y - window[0].v.y;
        let distance = (dx * dx + dy * dy).sqrt();
        let velocity = distance / dt_seconds; // pixels per second

        if velocity > threshold {
            findings.push(AuditFinding {
                severity: Severity::Warning,
                rule: AuditRule::MotionIntensity,
                message: format!(
                    "position velocity {:.0}px/s exceeds 50% of canvas diagonal ({:.0}px/s) — may cause motion sickness",
                    velocity, threshold,
                ),
                pointer: format!("{}/transform/position", pointer),
                frame_range: Some((window[0].t, window[1].t)),
            });
            // One finding per layer is enough.
            break;
        }
    }

    findings
}

// ── Check: Glow & brightness extremes ───────────────────────────────────────────

/// Flag extreme glow intensity or brightness values that may cause visual
/// discomfort.
fn check_glow_brightness(
    pointer: &str,
    layer: &Layer,
    opts: &AuditOptions,
) -> Vec<AuditFinding> {
    let mut findings = Vec::new();

    let effects = match &layer.effects {
        Some(effects) => effects,
        None => return findings,
    };

    for (i, effect) in effects.iter().enumerate() {
        match effect {
            Effect::Glow { intensity, .. }
                if *intensity > 2.0 && !opts.suppress.contains(&AuditRule::GlowIntensity) =>
            {
                findings.push(AuditFinding {
                    severity: Severity::Info,
                    rule: AuditRule::GlowIntensity,
                    message: format!(
                        "glow intensity {:.1} exceeds 2.0 — may cause visual discomfort",
                        intensity,
                    ),
                    pointer: format!("{}/effects[{}]", pointer, i),
                    frame_range: None,
                });
            }
            Effect::BrightnessContrast { brightness, .. }
                if (*brightness > 50.0 || *brightness < -50.0)
                    && !opts.suppress.contains(&AuditRule::BrightnessExtreme) =>
            {
                findings.push(AuditFinding {
                    severity: Severity::Info,
                    rule: AuditRule::BrightnessExtreme,
                    message: format!(
                        "brightness {:.1} is extreme (|value| > 50) — may cause visual discomfort",
                        brightness,
                    ),
                    pointer: format!("{}/effects[{}]", pointer, i),
                    frame_range: None,
                });
            }
            _ => {}
        }
    }

    findings
}

// ── Color math (WCAG) ───────────────────────────────────────────────────────────

/// Parse a hex color string (`#RRGGBB` or `#RGB`) into (R, G, B) in [0.0, 1.0].
fn parse_hex_color(hex: &str) -> Option<(f64, f64, f64)> {
    let hex = hex.strip_prefix('#')?;
    match hex.len() {
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some((r as f64 / 255.0, g as f64 / 255.0, b as f64 / 255.0))
        }
        3 => {
            let r = u8::from_str_radix(&hex[0..1], 16).ok()?;
            let g = u8::from_str_radix(&hex[1..2], 16).ok()?;
            let b = u8::from_str_radix(&hex[2..3], 16).ok()?;
            Some((
                (r * 17) as f64 / 255.0,
                (g * 17) as f64 / 255.0,
                (b * 17) as f64 / 255.0,
            ))
        }
        _ => None,
    }
}

/// Linearize an sRGB channel value (0.0..1.0) per the WCAG specification.
fn srgb_linearize(c: f64) -> f64 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// Compute WCAG relative luminance from linear sRGB components in [0.0, 1.0].
pub fn relative_luminance(r: f64, g: f64, b: f64) -> f64 {
    let r_lin = srgb_linearize(r);
    let g_lin = srgb_linearize(g);
    let b_lin = srgb_linearize(b);
    0.2126 * r_lin + 0.7152 * g_lin + 0.0722 * b_lin
}

/// Compute WCAG contrast ratio between two sRGB colors, each (r, g, b) in
/// [0.0, 1.0].
pub fn contrast_ratio(color1: (f64, f64, f64), color2: (f64, f64, f64)) -> f64 {
    let l1 = relative_luminance(color1.0, color1.1, color1.2);
    let l2 = relative_luminance(color2.0, color2.1, color2.2);
    let (lighter, darker) = if l1 > l2 { (l1, l2) } else { (l2, l1) };
    (lighter + 0.05) / (darker + 0.05)
}

// ── Tests ───────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{
        AnimatableValue, Keyframe, Layer, LayerContent, FontSpec, Transform, Vec2,
        Scene, Meta, Composition, Effect,
    };
    use crate::schema::easing::EasingValue;
    use std::collections::HashMap;

    // ── Helpers ──────────────────────────────────────────────────────────────

    fn make_scene(bg: &str, fps: f64, width: u32, height: u32, layers: Vec<Layer>) -> Scene {
        let mut compositions = HashMap::new();
        compositions.insert(
            "main".to_string(),
            Composition {
                layers,
                sequence: false,
                transition: None,
            },
        );
        Scene {
            version: "1.0".to_string(),
            meta: Meta {
                name: "test".to_string(),
                width,
                height,
                fps,
                duration: 90,
                background: bg.to_string(),
                root: "main".to_string(),
                safe_zone: None,
            },
            tokens: HashMap::new(),
            props: HashMap::new(),
            compositions,
            assets: Default::default(),
        }
    }

    fn default_transform() -> Transform {
        Transform {
            position: AnimatableValue::Static(Vec2 { x: 320.0, y: 180.0 }),
            scale: AnimatableValue::Static(Vec2 { x: 1.0, y: 1.0 }),
            opacity: AnimatableValue::Static(1.0),
            rotation: AnimatableValue::Static(0.0),
            opacity_modifiers: None,
            rotation_modifiers: None,
            position_modifiers: None,
            scale_modifiers: None,
        }
    }

    fn make_text_layer(id: &str, font_color: &str, font_size: f64) -> Layer {
        Layer {
            id: id.to_string(),
            in_point: 0,
            out_point: 30,
            transform: default_transform(),
            fill: None,
            blend_mode: None,
            parent: None,
            time_remap: None,
            masks: None,
            track_matte: None,
            adjustment: false,
            effects: None,
            motion_blur: false,
            trim_paths: None,
            path_animation: None,
            content: LayerContent::Text {
                text: "Hello".to_string(),
                font: FontSpec {
                    family: "Arial".to_string(),
                    size: font_size,
                    weight: 400,
                    color: font_color.to_string(),
                },
                align: Default::default(),
            },
        }
    }

    fn make_solid_layer_with_opacity(id: &str, opacity: AnimatableValue<f64>) -> Layer {
        Layer {
            id: id.to_string(),
            in_point: 0,
            out_point: 90,
            transform: Transform {
                position: AnimatableValue::Static(Vec2 { x: 320.0, y: 180.0 }),
                scale: AnimatableValue::Static(Vec2 { x: 1.0, y: 1.0 }),
                opacity,
                rotation: AnimatableValue::Static(0.0),
                opacity_modifiers: None,
                rotation_modifiers: None,
                position_modifiers: None,
                scale_modifiers: None,
            },
            fill: None,
            blend_mode: None,
            parent: None,
            time_remap: None,
            masks: None,
            track_matte: None,
            adjustment: false,
            effects: None,
            motion_blur: false,
            trim_paths: None,
            path_animation: None,
            content: LayerContent::Solid {
                color: "#ff0000".to_string(),
            },
        }
    }

    fn make_solid_layer_with_position(id: &str, position: AnimatableValue<Vec2>) -> Layer {
        Layer {
            id: id.to_string(),
            in_point: 0,
            out_point: 90,
            transform: Transform {
                position,
                scale: AnimatableValue::Static(Vec2 { x: 1.0, y: 1.0 }),
                opacity: AnimatableValue::Static(1.0),
                rotation: AnimatableValue::Static(0.0),
                opacity_modifiers: None,
                rotation_modifiers: None,
                position_modifiers: None,
                scale_modifiers: None,
            },
            fill: None,
            blend_mode: None,
            parent: None,
            time_remap: None,
            masks: None,
            track_matte: None,
            adjustment: false,
            effects: None,
            motion_blur: false,
            trim_paths: None,
            path_animation: None,
            content: LayerContent::Solid {
                color: "#ff0000".to_string(),
            },
        }
    }

    fn make_solid_layer_with_effects(id: &str, effects: Vec<Effect>) -> Layer {
        Layer {
            id: id.to_string(),
            in_point: 0,
            out_point: 30,
            transform: default_transform(),
            fill: None,
            blend_mode: None,
            parent: None,
            time_remap: None,
            masks: None,
            track_matte: None,
            adjustment: false,
            effects: Some(effects),
            motion_blur: false,
            trim_paths: None,
            path_animation: None,
            content: LayerContent::Solid {
                color: "#ff0000".to_string(),
            },
        }
    }

    fn kf(t: u64, v: f64) -> Keyframe<f64> {
        Keyframe {
            t,
            v,
            easing: EasingValue::linear(),
        }
    }

    fn kf_vec2(t: u64, x: f64, y: f64) -> Keyframe<Vec2> {
        Keyframe {
            t,
            v: Vec2 { x, y },
            easing: EasingValue::linear(),
        }
    }

    // ── Luminance & contrast tests ──────────────────────────────────────────

    #[test]
    fn relative_luminance_black() {
        let l = relative_luminance(0.0, 0.0, 0.0);
        assert!((l - 0.0).abs() < 1e-6);
    }

    #[test]
    fn relative_luminance_white() {
        let l = relative_luminance(1.0, 1.0, 1.0);
        assert!((l - 1.0).abs() < 1e-6);
    }

    #[test]
    fn contrast_ratio_black_white() {
        let ratio = contrast_ratio((0.0, 0.0, 0.0), (1.0, 1.0, 1.0));
        assert!((ratio - 21.0).abs() < 0.1);
    }

    #[test]
    fn contrast_ratio_same_color() {
        let ratio = contrast_ratio((0.5, 0.5, 0.5), (0.5, 0.5, 0.5));
        assert!((ratio - 1.0).abs() < 1e-6);
    }

    // ── Flash rate tests ────────────────────────────────────────────────────

    #[test]
    fn flash_rate_detects_rapid_cycling() {
        // At 30fps, create 10 on/off cycles within 30 frames (1 second).
        // That's opacity toggling every 3 frames = 10Hz, well above 3Hz.
        let mut keyframes = Vec::new();
        for i in 0..=10 {
            let t = i * 3;
            let v = if i % 2 == 0 { 1.0 } else { 0.0 };
            keyframes.push(kf(t, v));
        }

        let layer = make_solid_layer_with_opacity("flash", AnimatableValue::Animated(keyframes));
        let scene = make_scene("#000000", 30.0, 640, 360, vec![layer]);
        let report = audit(&scene, &AuditOptions::default());

        assert!(
            report.critical_count() > 0,
            "expected at least one CRITICAL finding for rapid flashing"
        );
        let flash_finding = report
            .findings
            .iter()
            .find(|f| f.rule == AuditRule::FlashRate);
        assert!(flash_finding.is_some(), "expected FlashRate finding");
    }

    #[test]
    fn flash_rate_ignores_slow_fade() {
        // A gentle fade: opacity goes from 0 to 1 over 30 frames at 30fps (1 second).
        // Only 1 crossing — well below threshold.
        let keyframes = vec![kf(0, 0.0), kf(15, 1.0), kf(30, 0.0)];
        let layer = make_solid_layer_with_opacity("fade", AnimatableValue::Animated(keyframes));
        let scene = make_scene("#000000", 30.0, 640, 360, vec![layer]);
        let report = audit(&scene, &AuditOptions::default());

        let flash_finding = report
            .findings
            .iter()
            .find(|f| f.rule == AuditRule::FlashRate);
        assert!(
            flash_finding.is_none(),
            "slow fade should not trigger flash rate warning"
        );
    }

    // ── Color contrast tests ────────────────────────────────────────────────

    #[test]
    fn color_contrast_fails_low_contrast() {
        // #888888 on #ffffff — computed ratio is about 3.54:1, fails AA (4.5:1).
        let layer = make_text_layer("gray_on_white", "#888888", 32.0);
        let scene = make_scene("#ffffff", 30.0, 640, 360, vec![layer]);
        let report = audit(&scene, &AuditOptions::default());

        let finding = report
            .findings
            .iter()
            .find(|f| f.rule == AuditRule::ColorContrast);
        assert!(
            finding.is_some(),
            "expected contrast warning for #888888 on #ffffff"
        );
    }

    #[test]
    fn color_contrast_passes_high_contrast() {
        // #000000 on #ffffff — contrast ratio ~21:1.
        let layer = make_text_layer("black_on_white", "#000000", 32.0);
        let scene = make_scene("#ffffff", 30.0, 640, 360, vec![layer]);
        let report = audit(&scene, &AuditOptions::default());

        let finding = report
            .findings
            .iter()
            .find(|f| f.rule == AuditRule::ColorContrast);
        assert!(
            finding.is_none(),
            "black on white should pass contrast check"
        );
    }

    // ── Text size tests ─────────────────────────────────────────────────────

    #[test]
    fn text_size_warns_small_text() {
        let layer = make_text_layer("tiny", "#ffffff", 10.0);
        let scene = make_scene("#000000", 30.0, 640, 360, vec![layer]);
        let report = audit(&scene, &AuditOptions::default());

        let finding = report
            .findings
            .iter()
            .find(|f| f.rule == AuditRule::TextSize);
        assert!(finding.is_some(), "expected TextSize info for 10px text");
    }

    #[test]
    fn text_size_ok_for_normal_text() {
        let layer = make_text_layer("normal", "#ffffff", 32.0);
        let scene = make_scene("#000000", 30.0, 640, 360, vec![layer]);
        let report = audit(&scene, &AuditOptions::default());

        let finding = report
            .findings
            .iter()
            .find(|f| f.rule == AuditRule::TextSize);
        assert!(finding.is_none(), "32px text should not trigger size warning");
    }

    // ── Motion intensity tests ──────────────────────────────────────────────

    #[test]
    fn motion_intensity_warns_fast_movement() {
        // Canvas 640x360 → diagonal ≈ 734px. 50% threshold ≈ 367px/s.
        // Move 600px in 15 frames at 30fps = 600 / 0.5 = 1200px/s → exceeds threshold.
        let keyframes = vec![
            kf_vec2(0, 20.0, 180.0),
            kf_vec2(15, 620.0, 180.0),
        ];
        let layer =
            make_solid_layer_with_position("fast", AnimatableValue::Animated(keyframes));
        let scene = make_scene("#000000", 30.0, 640, 360, vec![layer]);
        let report = audit(&scene, &AuditOptions::default());

        let finding = report
            .findings
            .iter()
            .find(|f| f.rule == AuditRule::MotionIntensity);
        assert!(
            finding.is_some(),
            "expected motion intensity warning for 1200px/s"
        );
    }

    #[test]
    fn motion_intensity_ok_for_slow_movement() {
        // Move 50px in 30 frames at 30fps = 50px/s — well below threshold.
        let keyframes = vec![
            kf_vec2(0, 300.0, 180.0),
            kf_vec2(30, 350.0, 180.0),
        ];
        let layer =
            make_solid_layer_with_position("slow", AnimatableValue::Animated(keyframes));
        let scene = make_scene("#000000", 30.0, 640, 360, vec![layer]);
        let report = audit(&scene, &AuditOptions::default());

        let finding = report
            .findings
            .iter()
            .find(|f| f.rule == AuditRule::MotionIntensity);
        assert!(
            finding.is_none(),
            "slow movement should not trigger motion intensity warning"
        );
    }

    // ── Glow & brightness tests ─────────────────────────────────────────────

    #[test]
    fn glow_intensity_warns_extreme() {
        let layer = make_solid_layer_with_effects(
            "glow",
            vec![Effect::Glow {
                color: "#ffffff".to_string(),
                radius: 10.0,
                intensity: 3.0,
            }],
        );
        let scene = make_scene("#000000", 30.0, 640, 360, vec![layer]);
        let report = audit(&scene, &AuditOptions::default());

        let finding = report
            .findings
            .iter()
            .find(|f| f.rule == AuditRule::GlowIntensity);
        assert!(
            finding.is_some(),
            "expected glow intensity info for intensity=3.0"
        );
    }

    #[test]
    fn brightness_extreme_warns() {
        let layer = make_solid_layer_with_effects(
            "bright",
            vec![Effect::BrightnessContrast {
                brightness: 75.0,
                contrast: 0.0,
            }],
        );
        let scene = make_scene("#000000", 30.0, 640, 360, vec![layer]);
        let report = audit(&scene, &AuditOptions::default());

        let finding = report
            .findings
            .iter()
            .find(|f| f.rule == AuditRule::BrightnessExtreme);
        assert!(
            finding.is_some(),
            "expected brightness extreme info for brightness=75"
        );
    }

    #[test]
    fn brightness_extreme_negative() {
        let layer = make_solid_layer_with_effects(
            "dark",
            vec![Effect::BrightnessContrast {
                brightness: -60.0,
                contrast: 0.0,
            }],
        );
        let scene = make_scene("#000000", 30.0, 640, 360, vec![layer]);
        let report = audit(&scene, &AuditOptions::default());

        let finding = report
            .findings
            .iter()
            .find(|f| f.rule == AuditRule::BrightnessExtreme);
        assert!(
            finding.is_some(),
            "expected brightness extreme info for brightness=-60"
        );
    }

    #[test]
    fn normal_effects_no_warnings() {
        let layer = make_solid_layer_with_effects(
            "normal",
            vec![
                Effect::Glow {
                    color: "#ffffff".to_string(),
                    radius: 5.0,
                    intensity: 1.0,
                },
                Effect::BrightnessContrast {
                    brightness: 20.0,
                    contrast: 10.0,
                },
            ],
        );
        let scene = make_scene("#000000", 30.0, 640, 360, vec![layer]);
        let report = audit(&scene, &AuditOptions::default());

        assert!(
            report.is_clean(),
            "normal effect values should not produce findings"
        );
    }

    // ── Integration test ────────────────────────────────────────────────────

    #[test]
    fn audit_integration_test() {
        let json = std::fs::read_to_string("../../tests/fixtures/valid/accessibility_flash.mmot.json")
            .expect("fixture file should exist");
        let scene = crate::parser::parse(&json).expect("fixture should parse");
        let report = audit(&scene, &AuditOptions::default());
        assert!(
            report.critical_count() > 0,
            "flash fixture should produce at least one CRITICAL finding"
        );
    }

    #[test]
    fn audit_contrast_fixture_test() {
        let json = std::fs::read_to_string("../../tests/fixtures/valid/accessibility_contrast.mmot.json")
            .expect("fixture file should exist");
        let scene = crate::parser::parse(&json).expect("fixture should parse");
        let report = audit(&scene, &AuditOptions::default());

        let contrast_finding = report
            .findings
            .iter()
            .find(|f| f.rule == AuditRule::ColorContrast);
        assert!(
            contrast_finding.is_some(),
            "contrast fixture should produce a ColorContrast warning"
        );
    }

    // ── Suppression test ────────────────────────────────────────────────────

    #[test]
    fn suppress_rules_works() {
        let layer = make_text_layer("tiny", "#888888", 10.0);
        let scene = make_scene("#ffffff", 30.0, 640, 360, vec![layer]);

        let opts = AuditOptions {
            contrast_level: ContrastLevel::AA,
            suppress: vec![AuditRule::ColorContrast, AuditRule::TextSize],
        };
        let report = audit(&scene, &opts);
        assert!(
            report.is_clean(),
            "suppressed rules should produce no findings"
        );
    }
}
