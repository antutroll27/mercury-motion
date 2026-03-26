//! Semantic diff engine for `.mmot.json` scenes.
//!
//! Compares two [`Scene`] values structurally — detecting added, removed, and
//! changed compositions, layers, effects, and masks — instead of doing a
//! line-level text diff.

use crate::schema::composition::{Layer, LayerContent};
use crate::schema::scene::Meta;
use crate::schema::{Compositions, Scene};

// ── Public types ────────────────────────────────────────────────────────────────

/// A single semantic change between two scenes.
#[derive(Debug, Clone)]
pub enum DiffEntry {
    /// A top-level metadata field changed (e.g. fps, width, background).
    MetaChanged {
        field: String,
        old: String,
        new: String,
    },
    /// A composition was added in the new scene.
    CompositionAdded { id: String },
    /// A composition was removed from the old scene.
    CompositionRemoved { id: String },
    /// A layer was added to a composition.
    LayerAdded {
        composition: String,
        layer_id: String,
        layer_type: String,
        in_point: u64,
        out_point: u64,
    },
    /// A layer was removed from a composition.
    LayerRemoved {
        composition: String,
        layer_id: String,
        layer_type: String,
    },
    /// A property on a shared layer changed.
    LayerPropertyChanged {
        composition: String,
        layer_id: String,
        property: String,
        old: String,
        new: String,
    },
    /// An effect was added to a layer.
    EffectAdded {
        composition: String,
        layer_id: String,
        effect_type: String,
    },
    /// An effect was removed from a layer.
    EffectRemoved {
        composition: String,
        layer_id: String,
        effect_type: String,
    },
    /// A mask was added to a layer.
    MaskAdded {
        composition: String,
        layer_id: String,
    },
    /// A mask was removed from a layer.
    MaskRemoved {
        composition: String,
        layer_id: String,
    },
}

/// Result of comparing two scenes.
pub struct DiffResult {
    pub entries: Vec<DiffEntry>,
}

impl DiffResult {
    /// Returns `true` when the two scenes are identical.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Returns `true` when differences were detected.
    pub fn has_changes(&self) -> bool {
        !self.entries.is_empty()
    }
}

// ── Public entry point ──────────────────────────────────────────────────────────

/// Compare two scenes semantically and return all detected changes.
pub fn diff(a: &Scene, b: &Scene) -> DiffResult {
    let mut entries = Vec::new();

    diff_meta(&a.meta, &b.meta, &mut entries);
    diff_compositions(&a.compositions, &b.compositions, &mut entries);

    DiffResult { entries }
}

// ── Helpers ─────────────────────────────────────────────────────────────────────

/// Serialize any `Serialize` value to a compact JSON string for comparison.
fn value_to_string(val: &impl serde::Serialize) -> String {
    serde_json::to_string(val).unwrap_or_else(|_| "?".into())
}

/// Convert a `Serialize` value to a `serde_json::Value` for deep comparison.
fn to_json_value(val: &impl serde::Serialize) -> serde_json::Value {
    serde_json::to_value(val).unwrap_or(serde_json::Value::Null)
}

/// Emit a `MetaChanged` entry if `old` and `new` differ (compared as strings).
fn check_meta_field(
    field: &str,
    old: &str,
    new: &str,
    entries: &mut Vec<DiffEntry>,
) {
    if old != new {
        entries.push(DiffEntry::MetaChanged {
            field: field.to_string(),
            old: old.to_string(),
            new: new.to_string(),
        });
    }
}

// ── Meta diff ───────────────────────────────────────────────────────────────────

fn diff_meta(a: &Meta, b: &Meta, entries: &mut Vec<DiffEntry>) {
    check_meta_field("name", &a.name, &b.name, entries);
    check_meta_field("width", &a.width.to_string(), &b.width.to_string(), entries);
    check_meta_field("height", &a.height.to_string(), &b.height.to_string(), entries);
    check_meta_field("fps", &a.fps.to_string(), &b.fps.to_string(), entries);
    check_meta_field(
        "duration",
        &a.duration.to_string(),
        &b.duration.to_string(),
        entries,
    );
    check_meta_field("background", &a.background, &b.background, entries);
    check_meta_field("root", &a.root, &b.root, entries);

    let safe_a = value_to_string(&a.safe_zone);
    let safe_b = value_to_string(&b.safe_zone);
    check_meta_field("safe_zone", &safe_a, &safe_b, entries);
}

// ── Composition diff ────────────────────────────────────────────────────────────

fn diff_compositions(a: &Compositions, b: &Compositions, entries: &mut Vec<DiffEntry>) {
    // Removed compositions
    for key in a.keys() {
        if !b.contains_key(key) {
            entries.push(DiffEntry::CompositionRemoved { id: key.clone() });
        }
    }

    // Added compositions
    for key in b.keys() {
        if !a.contains_key(key) {
            entries.push(DiffEntry::CompositionAdded { id: key.clone() });
        }
    }

    // Shared compositions — compare layers
    for (comp_id, comp_a) in a {
        if let Some(comp_b) = b.get(comp_id) {
            diff_layers(comp_id, &comp_a.layers, &comp_b.layers, entries);

            // Compare composition-level fields (sequence, transition)
            if comp_a.sequence != comp_b.sequence {
                entries.push(DiffEntry::MetaChanged {
                    field: format!("compositions.{comp_id}.sequence"),
                    old: comp_a.sequence.to_string(),
                    new: comp_b.sequence.to_string(),
                });
            }
            let trans_a = to_json_value(&comp_a.transition);
            let trans_b = to_json_value(&comp_b.transition);
            if trans_a != trans_b {
                entries.push(DiffEntry::MetaChanged {
                    field: format!("compositions.{comp_id}.transition"),
                    old: value_to_string(&comp_a.transition),
                    new: value_to_string(&comp_b.transition),
                });
            }
        }
    }
}

// ── Layer diff ──────────────────────────────────────────────────────────────────

/// Get a short type name for the layer content.
fn layer_type_name(content: &LayerContent) -> &'static str {
    match content {
        LayerContent::Solid { .. } => "solid",
        LayerContent::Image { .. } => "image",
        LayerContent::Video { .. } => "video",
        LayerContent::Text { .. } => "text",
        LayerContent::Audio { .. } => "audio",
        LayerContent::Lottie { .. } => "lottie",
        LayerContent::Composition { .. } => "composition",
        LayerContent::Shape { .. } => "shape",
        LayerContent::Gradient { .. } => "gradient",
        LayerContent::Null => "null",
    }
}

fn diff_layers(
    comp_id: &str,
    layers_a: &[Layer],
    layers_b: &[Layer],
    entries: &mut Vec<DiffEntry>,
) {
    // Build id -> layer maps
    let map_a: std::collections::HashMap<&str, &Layer> =
        layers_a.iter().map(|l| (l.id.as_str(), l)).collect();
    let map_b: std::collections::HashMap<&str, &Layer> =
        layers_b.iter().map(|l| (l.id.as_str(), l)).collect();

    // Removed layers
    for layer in layers_a {
        if !map_b.contains_key(layer.id.as_str()) {
            entries.push(DiffEntry::LayerRemoved {
                composition: comp_id.to_string(),
                layer_id: layer.id.clone(),
                layer_type: layer_type_name(&layer.content).to_string(),
            });
        }
    }

    // Added layers
    for layer in layers_b {
        if !map_a.contains_key(layer.id.as_str()) {
            entries.push(DiffEntry::LayerAdded {
                composition: comp_id.to_string(),
                layer_id: layer.id.clone(),
                layer_type: layer_type_name(&layer.content).to_string(),
                in_point: layer.in_point,
                out_point: layer.out_point,
            });
        }
    }

    // Shared layers — compare properties
    for layer_a in layers_a {
        if let Some(layer_b) = map_b.get(layer_a.id.as_str()) {
            diff_layer_properties(comp_id, layer_a, layer_b, entries);
        }
    }
}

fn diff_layer_properties(
    comp_id: &str,
    a: &Layer,
    b: &Layer,
    entries: &mut Vec<DiffEntry>,
) {
    let comp = comp_id.to_string();
    let lid = a.id.clone();

    // Timing
    if a.in_point != b.in_point {
        entries.push(DiffEntry::LayerPropertyChanged {
            composition: comp.clone(),
            layer_id: lid.clone(),
            property: "in".to_string(),
            old: a.in_point.to_string(),
            new: b.in_point.to_string(),
        });
    }
    if a.out_point != b.out_point {
        entries.push(DiffEntry::LayerPropertyChanged {
            composition: comp.clone(),
            layer_id: lid.clone(),
            property: "out".to_string(),
            old: a.out_point.to_string(),
            new: b.out_point.to_string(),
        });
    }

    // Transform (compare via JSON values)
    let transform_a = to_json_value(&a.transform);
    let transform_b = to_json_value(&b.transform);
    if transform_a != transform_b {
        // Drill into individual transform fields for better messages
        diff_transform_fields(comp_id, &lid, &a.transform, &b.transform, entries);
    }

    // Content type change
    let type_a = layer_type_name(&a.content);
    let type_b = layer_type_name(&b.content);
    if type_a != type_b {
        entries.push(DiffEntry::LayerPropertyChanged {
            composition: comp.clone(),
            layer_id: lid.clone(),
            property: "type".to_string(),
            old: type_a.to_string(),
            new: type_b.to_string(),
        });
    }

    // Content fields
    diff_content(comp_id, &lid, &a.content, &b.content, entries);

    // blend_mode
    let bm_a = to_json_value(&a.blend_mode);
    let bm_b = to_json_value(&b.blend_mode);
    if bm_a != bm_b {
        entries.push(DiffEntry::LayerPropertyChanged {
            composition: comp.clone(),
            layer_id: lid.clone(),
            property: "blend_mode".to_string(),
            old: value_to_string(&a.blend_mode),
            new: value_to_string(&b.blend_mode),
        });
    }

    // parent
    if a.parent != b.parent {
        entries.push(DiffEntry::LayerPropertyChanged {
            composition: comp.clone(),
            layer_id: lid.clone(),
            property: "parent".to_string(),
            old: a.parent.clone().unwrap_or_default(),
            new: b.parent.clone().unwrap_or_default(),
        });
    }

    // fill
    let fill_a = to_json_value(&a.fill);
    let fill_b = to_json_value(&b.fill);
    if fill_a != fill_b {
        entries.push(DiffEntry::LayerPropertyChanged {
            composition: comp.clone(),
            layer_id: lid.clone(),
            property: "fill".to_string(),
            old: value_to_string(&a.fill),
            new: value_to_string(&b.fill),
        });
    }

    // adjustment
    if a.adjustment != b.adjustment {
        entries.push(DiffEntry::LayerPropertyChanged {
            composition: comp.clone(),
            layer_id: lid.clone(),
            property: "adjustment".to_string(),
            old: a.adjustment.to_string(),
            new: b.adjustment.to_string(),
        });
    }

    // motion_blur
    if a.motion_blur != b.motion_blur {
        entries.push(DiffEntry::LayerPropertyChanged {
            composition: comp.clone(),
            layer_id: lid.clone(),
            property: "motion_blur".to_string(),
            old: a.motion_blur.to_string(),
            new: b.motion_blur.to_string(),
        });
    }

    // Effects
    diff_effects(comp_id, &lid, &a.effects, &b.effects, entries);

    // Masks
    diff_masks(comp_id, &lid, &a.masks, &b.masks, entries);

    // time_remap
    let tr_a = to_json_value(&a.time_remap);
    let tr_b = to_json_value(&b.time_remap);
    if tr_a != tr_b {
        entries.push(DiffEntry::LayerPropertyChanged {
            composition: comp.clone(),
            layer_id: lid.clone(),
            property: "time_remap".to_string(),
            old: value_to_string(&a.time_remap),
            new: value_to_string(&b.time_remap),
        });
    }

    // track_matte
    let tm_a = to_json_value(&a.track_matte);
    let tm_b = to_json_value(&b.track_matte);
    if tm_a != tm_b {
        entries.push(DiffEntry::LayerPropertyChanged {
            composition: comp.clone(),
            layer_id: lid.clone(),
            property: "track_matte".to_string(),
            old: value_to_string(&a.track_matte),
            new: value_to_string(&b.track_matte),
        });
    }

    // trim_paths
    let tp_a = to_json_value(&a.trim_paths);
    let tp_b = to_json_value(&b.trim_paths);
    if tp_a != tp_b {
        entries.push(DiffEntry::LayerPropertyChanged {
            composition: comp.clone(),
            layer_id: lid.clone(),
            property: "trim_paths".to_string(),
            old: value_to_string(&a.trim_paths),
            new: value_to_string(&b.trim_paths),
        });
    }

    // path_animation
    let pa_a = to_json_value(&a.path_animation);
    let pa_b = to_json_value(&b.path_animation);
    if pa_a != pa_b {
        entries.push(DiffEntry::LayerPropertyChanged {
            composition: comp,
            layer_id: lid,
            property: "path_animation".to_string(),
            old: value_to_string(&a.path_animation),
            new: value_to_string(&b.path_animation),
        });
    }
}

// ── Transform field diff ────────────────────────────────────────────────────────

fn diff_transform_fields(
    comp_id: &str,
    layer_id: &str,
    a: &crate::schema::Transform,
    b: &crate::schema::Transform,
    entries: &mut Vec<DiffEntry>,
) {
    let fields: &[(&str, serde_json::Value, serde_json::Value)] = &[
        ("transform.position", to_json_value(&a.position), to_json_value(&b.position)),
        ("transform.scale", to_json_value(&a.scale), to_json_value(&b.scale)),
        ("transform.opacity", to_json_value(&a.opacity), to_json_value(&b.opacity)),
        ("transform.rotation", to_json_value(&a.rotation), to_json_value(&b.rotation)),
    ];

    for (prop, val_a, val_b) in fields {
        if val_a != val_b {
            entries.push(DiffEntry::LayerPropertyChanged {
                composition: comp_id.to_string(),
                layer_id: layer_id.to_string(),
                property: prop.to_string(),
                old: val_a.to_string(),
                new: val_b.to_string(),
            });
        }
    }
}

// ── Content diff ────────────────────────────────────────────────────────────────

fn diff_content(
    comp_id: &str,
    layer_id: &str,
    a: &LayerContent,
    b: &LayerContent,
    entries: &mut Vec<DiffEntry>,
) {
    let comp = comp_id.to_string();
    let lid = layer_id.to_string();

    match (a, b) {
        (LayerContent::Solid { color: ca }, LayerContent::Solid { color: cb }) => {
            if ca != cb {
                entries.push(DiffEntry::LayerPropertyChanged {
                    composition: comp,
                    layer_id: lid,
                    property: "color".to_string(),
                    old: ca.clone(),
                    new: cb.clone(),
                });
            }
        }
        (
            LayerContent::Text {
                text: ta,
                font: fa,
                align: aa,
            },
            LayerContent::Text {
                text: tb,
                font: fb,
                align: ab,
            },
        ) => {
            if ta != tb {
                entries.push(DiffEntry::LayerPropertyChanged {
                    composition: comp.clone(),
                    layer_id: lid.clone(),
                    property: "text".to_string(),
                    old: ta.clone(),
                    new: tb.clone(),
                });
            }
            let font_a = to_json_value(fa);
            let font_b = to_json_value(fb);
            if font_a != font_b {
                entries.push(DiffEntry::LayerPropertyChanged {
                    composition: comp.clone(),
                    layer_id: lid.clone(),
                    property: "font".to_string(),
                    old: value_to_string(fa),
                    new: value_to_string(fb),
                });
            }
            let align_a = to_json_value(aa);
            let align_b = to_json_value(ab);
            if align_a != align_b {
                entries.push(DiffEntry::LayerPropertyChanged {
                    composition: comp,
                    layer_id: lid,
                    property: "align".to_string(),
                    old: value_to_string(aa),
                    new: value_to_string(ab),
                });
            }
        }
        (LayerContent::Image { src: sa }, LayerContent::Image { src: sb }) => {
            if sa != sb {
                entries.push(DiffEntry::LayerPropertyChanged {
                    composition: comp,
                    layer_id: lid,
                    property: "src".to_string(),
                    old: sa.clone(),
                    new: sb.clone(),
                });
            }
        }
        (
            LayerContent::Video {
                src: sa,
                trim_start: tsa,
                trim_end: tea,
            },
            LayerContent::Video {
                src: sb,
                trim_start: tsb,
                trim_end: teb,
            },
        ) => {
            if sa != sb {
                entries.push(DiffEntry::LayerPropertyChanged {
                    composition: comp.clone(),
                    layer_id: lid.clone(),
                    property: "src".to_string(),
                    old: sa.clone(),
                    new: sb.clone(),
                });
            }
            if (tsa - tsb).abs() > f64::EPSILON {
                entries.push(DiffEntry::LayerPropertyChanged {
                    composition: comp.clone(),
                    layer_id: lid.clone(),
                    property: "trim_start".to_string(),
                    old: tsa.to_string(),
                    new: tsb.to_string(),
                });
            }
            if tea != teb {
                entries.push(DiffEntry::LayerPropertyChanged {
                    composition: comp,
                    layer_id: lid,
                    property: "trim_end".to_string(),
                    old: value_to_string(tea),
                    new: value_to_string(teb),
                });
            }
        }
        (
            LayerContent::Audio { src: sa, volume: va },
            LayerContent::Audio { src: sb, volume: vb },
        ) => {
            if sa != sb {
                entries.push(DiffEntry::LayerPropertyChanged {
                    composition: comp.clone(),
                    layer_id: lid.clone(),
                    property: "src".to_string(),
                    old: sa.clone(),
                    new: sb.clone(),
                });
            }
            let vol_a = to_json_value(va);
            let vol_b = to_json_value(vb);
            if vol_a != vol_b {
                entries.push(DiffEntry::LayerPropertyChanged {
                    composition: comp,
                    layer_id: lid,
                    property: "volume".to_string(),
                    old: value_to_string(va),
                    new: value_to_string(vb),
                });
            }
        }
        (LayerContent::Lottie { src: sa }, LayerContent::Lottie { src: sb }) => {
            if sa != sb {
                entries.push(DiffEntry::LayerPropertyChanged {
                    composition: comp,
                    layer_id: lid,
                    property: "src".to_string(),
                    old: sa.clone(),
                    new: sb.clone(),
                });
            }
        }
        (LayerContent::Composition { id: ia }, LayerContent::Composition { id: ib }) => {
            if ia != ib {
                entries.push(DiffEntry::LayerPropertyChanged {
                    composition: comp,
                    layer_id: lid,
                    property: "composition_id".to_string(),
                    old: ia.clone(),
                    new: ib.clone(),
                });
            }
        }
        (LayerContent::Shape { shape: sa }, LayerContent::Shape { shape: sb }) => {
            let va = to_json_value(sa);
            let vb = to_json_value(sb);
            if va != vb {
                entries.push(DiffEntry::LayerPropertyChanged {
                    composition: comp,
                    layer_id: lid,
                    property: "shape".to_string(),
                    old: value_to_string(sa),
                    new: value_to_string(sb),
                });
            }
        }
        (LayerContent::Gradient { gradient: ga }, LayerContent::Gradient { gradient: gb }) => {
            let va = to_json_value(ga);
            let vb = to_json_value(gb);
            if va != vb {
                entries.push(DiffEntry::LayerPropertyChanged {
                    composition: comp,
                    layer_id: lid,
                    property: "gradient".to_string(),
                    old: value_to_string(ga),
                    new: value_to_string(gb),
                });
            }
        }
        (LayerContent::Null, LayerContent::Null) => {}
        // Different content types — already reported as a type change above
        _ => {}
    }
}

// ── Effects diff ────────────────────────────────────────────────────────────────

/// Get a short name for an effect variant.
fn effect_type_name(effect: &crate::schema::Effect) -> &'static str {
    match effect {
        crate::schema::Effect::GaussianBlur { .. } => "gaussian_blur",
        crate::schema::Effect::DropShadow { .. } => "drop_shadow",
        crate::schema::Effect::Glow { .. } => "glow",
        crate::schema::Effect::BrightnessContrast { .. } => "brightness_contrast",
        crate::schema::Effect::HueSaturation { .. } => "hue_saturation",
        crate::schema::Effect::Invert => "invert",
        crate::schema::Effect::Tint { .. } => "tint",
        crate::schema::Effect::Fill { .. } => "fill",
    }
}

fn diff_effects(
    comp_id: &str,
    layer_id: &str,
    a: &Option<Vec<crate::schema::Effect>>,
    b: &Option<Vec<crate::schema::Effect>>,
    entries: &mut Vec<DiffEntry>,
) {
    let empty = Vec::new();
    let effects_a = a.as_ref().unwrap_or(&empty);
    let effects_b = b.as_ref().unwrap_or(&empty);

    let len_a = effects_a.len();
    let len_b = effects_b.len();
    let shared = len_a.min(len_b);

    // Compare shared effects by index
    for i in 0..shared {
        let ea = &effects_a[i];
        let eb = &effects_b[i];
        let va = to_json_value(ea);
        let vb = to_json_value(eb);
        if va != vb {
            // Report as removed old + added new
            entries.push(DiffEntry::EffectRemoved {
                composition: comp_id.to_string(),
                layer_id: layer_id.to_string(),
                effect_type: effect_type_name(ea).to_string(),
            });
            entries.push(DiffEntry::EffectAdded {
                composition: comp_id.to_string(),
                layer_id: layer_id.to_string(),
                effect_type: effect_type_name(eb).to_string(),
            });
        }
    }

    // Removed effects (tail of a)
    for effect in effects_a.iter().skip(shared) {
        entries.push(DiffEntry::EffectRemoved {
            composition: comp_id.to_string(),
            layer_id: layer_id.to_string(),
            effect_type: effect_type_name(effect).to_string(),
        });
    }

    // Added effects (tail of b)
    for effect in effects_b.iter().skip(shared) {
        entries.push(DiffEntry::EffectAdded {
            composition: comp_id.to_string(),
            layer_id: layer_id.to_string(),
            effect_type: effect_type_name(effect).to_string(),
        });
    }
}

// ── Masks diff ──────────────────────────────────────────────────────────────────

fn diff_masks(
    comp_id: &str,
    layer_id: &str,
    a: &Option<Vec<crate::schema::Mask>>,
    b: &Option<Vec<crate::schema::Mask>>,
    entries: &mut Vec<DiffEntry>,
) {
    let count_a = a.as_ref().map_or(0, Vec::len);
    let count_b = b.as_ref().map_or(0, Vec::len);

    if count_b > count_a {
        for _ in 0..(count_b - count_a) {
            entries.push(DiffEntry::MaskAdded {
                composition: comp_id.to_string(),
                layer_id: layer_id.to_string(),
            });
        }
    } else if count_a > count_b {
        for _ in 0..(count_a - count_b) {
            entries.push(DiffEntry::MaskRemoved {
                composition: comp_id.to_string(),
                layer_id: layer_id.to_string(),
            });
        }
    }

    // Also detect changes in shared masks
    if let (Some(masks_a), Some(masks_b)) = (a, b) {
        let shared = count_a.min(count_b);
        for i in 0..shared {
            let va = to_json_value(&masks_a[i]);
            let vb = to_json_value(&masks_b[i]);
            if va != vb {
                // Report as remove + add for changed masks
                entries.push(DiffEntry::MaskRemoved {
                    composition: comp_id.to_string(),
                    layer_id: layer_id.to_string(),
                });
                entries.push(DiffEntry::MaskAdded {
                    composition: comp_id.to_string(),
                    layer_id: layer_id.to_string(),
                });
            }
        }
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;

    fn parse_scene(json: &str) -> Scene {
        parser::parse(json).unwrap()
    }

    fn minimal_json() -> String {
        r##"{
            "version": "1.0",
            "meta": { "name": "Test", "width": 1920, "height": 1080, "fps": 30, "duration": 60, "root": "main", "background": "#000000" },
            "compositions": {
                "main": {
                    "layers": [
                        { "id": "bg", "type": "solid", "in": 0, "out": 60, "color": "#1a1a2e", "transform": { "position": [960, 540] } }
                    ]
                }
            }
        }"##
        .to_string()
    }

    #[test]
    fn diff_identical_scenes_produces_empty_result() {
        let json = minimal_json();
        let a = parse_scene(&json);
        let b = parse_scene(&json);
        let result = diff(&a, &b);
        assert!(result.is_empty(), "identical scenes should produce no diff entries, got: {:?}", result.entries);
    }

    #[test]
    fn diff_meta_changes_detected() {
        let json_a = minimal_json();
        let json_b = r##"{
            "version": "1.0",
            "meta": { "name": "Test", "width": 1920, "height": 1080, "fps": 60, "duration": 60, "root": "main", "background": "#111111" },
            "compositions": {
                "main": {
                    "layers": [
                        { "id": "bg", "type": "solid", "in": 0, "out": 60, "color": "#1a1a2e", "transform": { "position": [960, 540] } }
                    ]
                }
            }
        }"##;
        let a = parse_scene(&json_a);
        let b = parse_scene(json_b);
        let result = diff(&a, &b);

        let meta_changes: Vec<_> = result
            .entries
            .iter()
            .filter(|e| matches!(e, DiffEntry::MetaChanged { .. }))
            .collect();
        assert!(meta_changes.len() >= 2, "expected at least fps and background changes");

        let fps_changed = meta_changes.iter().any(|e| {
            matches!(e, DiffEntry::MetaChanged { field, old, new }
                if field == "fps" && old == "30" && new == "60")
        });
        assert!(fps_changed, "fps change not detected");

        let bg_changed = meta_changes.iter().any(|e| {
            matches!(e, DiffEntry::MetaChanged { field, .. } if field == "background")
        });
        assert!(bg_changed, "background change not detected");
    }

    #[test]
    fn diff_layer_added_detected() {
        let json_a = minimal_json();
        let json_b = r##"{
            "version": "1.0",
            "meta": { "name": "Test", "width": 1920, "height": 1080, "fps": 30, "duration": 60, "root": "main", "background": "#000000" },
            "compositions": {
                "main": {
                    "layers": [
                        { "id": "bg", "type": "solid", "in": 0, "out": 60, "color": "#1a1a2e", "transform": { "position": [960, 540] } },
                        { "id": "sparkle", "type": "solid", "in": 10, "out": 50, "color": "#ffffff", "transform": { "position": [100, 100] } }
                    ]
                }
            }
        }"##;
        let a = parse_scene(&json_a);
        let b = parse_scene(json_b);
        let result = diff(&a, &b);

        let added = result.entries.iter().any(|e| {
            matches!(e, DiffEntry::LayerAdded { layer_id, layer_type, in_point, out_point, .. }
                if layer_id == "sparkle" && layer_type == "solid" && *in_point == 10 && *out_point == 50)
        });
        assert!(added, "new layer 'sparkle' not detected as added");
    }

    #[test]
    fn diff_layer_removed_detected() {
        let json_a = r##"{
            "version": "1.0",
            "meta": { "name": "Test", "width": 1920, "height": 1080, "fps": 30, "duration": 60, "root": "main", "background": "#000000" },
            "compositions": {
                "main": {
                    "layers": [
                        { "id": "bg", "type": "solid", "in": 0, "out": 60, "color": "#1a1a2e", "transform": { "position": [960, 540] } },
                        { "id": "overlay", "type": "solid", "in": 0, "out": 60, "color": "#ff0000", "transform": { "position": [960, 540] } }
                    ]
                }
            }
        }"##;
        let json_b = minimal_json();
        let a = parse_scene(json_a);
        let b = parse_scene(&json_b);
        let result = diff(&a, &b);

        let removed = result.entries.iter().any(|e| {
            matches!(e, DiffEntry::LayerRemoved { layer_id, .. } if layer_id == "overlay")
        });
        assert!(removed, "removed layer 'overlay' not detected");
    }

    #[test]
    fn diff_layer_property_change_detected() {
        let json_a = minimal_json();
        let json_b = r##"{
            "version": "1.0",
            "meta": { "name": "Test", "width": 1920, "height": 1080, "fps": 30, "duration": 60, "root": "main", "background": "#000000" },
            "compositions": {
                "main": {
                    "layers": [
                        { "id": "bg", "type": "solid", "in": 0, "out": 60, "color": "#ff0000", "transform": { "position": [960, 540] } }
                    ]
                }
            }
        }"##;
        let a = parse_scene(&json_a);
        let b = parse_scene(json_b);
        let result = diff(&a, &b);

        let color_changed = result.entries.iter().any(|e| {
            matches!(e, DiffEntry::LayerPropertyChanged { property, old, new, .. }
                if property == "color" && old == "#1a1a2e" && new == "#ff0000")
        });
        assert!(color_changed, "color property change not detected");
    }

    #[test]
    fn diff_effect_added_detected() {
        let json_a = minimal_json();
        let json_b = r##"{
            "version": "1.0",
            "meta": { "name": "Test", "width": 1920, "height": 1080, "fps": 30, "duration": 60, "root": "main", "background": "#000000" },
            "compositions": {
                "main": {
                    "layers": [
                        {
                            "id": "bg", "type": "solid", "in": 0, "out": 60, "color": "#1a1a2e",
                            "transform": { "position": [960, 540] },
                            "effects": [{ "type": "gaussian_blur", "radius": 5.0 }]
                        }
                    ]
                }
            }
        }"##;
        let a = parse_scene(&json_a);
        let b = parse_scene(json_b);
        let result = diff(&a, &b);

        let effect_added = result.entries.iter().any(|e| {
            matches!(e, DiffEntry::EffectAdded { effect_type, .. } if effect_type == "gaussian_blur")
        });
        assert!(effect_added, "added effect 'gaussian_blur' not detected");
    }

    #[test]
    fn diff_composition_added_and_removed() {
        let json_a = r##"{
            "version": "1.0",
            "meta": { "name": "Test", "width": 1920, "height": 1080, "fps": 30, "duration": 60, "root": "main", "background": "#000000" },
            "compositions": {
                "main": { "layers": [] },
                "intro": { "layers": [] }
            }
        }"##;
        let json_b = r##"{
            "version": "1.0",
            "meta": { "name": "Test", "width": 1920, "height": 1080, "fps": 30, "duration": 60, "root": "main", "background": "#000000" },
            "compositions": {
                "main": { "layers": [] },
                "outro": { "layers": [] }
            }
        }"##;
        let a = parse_scene(json_a);
        let b = parse_scene(json_b);
        let result = diff(&a, &b);

        let comp_removed = result.entries.iter().any(|e| {
            matches!(e, DiffEntry::CompositionRemoved { id } if id == "intro")
        });
        assert!(comp_removed, "removed composition 'intro' not detected");

        let comp_added = result.entries.iter().any(|e| {
            matches!(e, DiffEntry::CompositionAdded { id } if id == "outro")
        });
        assert!(comp_added, "added composition 'outro' not detected");
    }

    #[test]
    fn diff_fixture_files() {
        let json_a = include_str!("../../../tests/fixtures/valid/diff_base.mmot.json");
        let json_b = include_str!("../../../tests/fixtures/valid/diff_modified.mmot.json");
        let a = parse_scene(json_a);
        let b = parse_scene(json_b);
        let result = diff(&a, &b);

        assert!(result.has_changes(), "fixture diff should have changes");

        // fps changed 30 -> 60
        let fps_changed = result.entries.iter().any(|e| {
            matches!(e, DiffEntry::MetaChanged { field, .. } if field == "fps")
        });
        assert!(fps_changed, "fps change in fixtures not detected");

        // background changed
        let bg_changed = result.entries.iter().any(|e| {
            matches!(e, DiffEntry::MetaChanged { field, .. } if field == "background")
        });
        assert!(bg_changed, "background change in fixtures not detected");

        // title text changed
        let text_changed = result.entries.iter().any(|e| {
            matches!(e, DiffEntry::LayerPropertyChanged { property, .. } if property == "text")
        });
        assert!(text_changed, "text property change in fixtures not detected");

        // sparkle layer added
        let sparkle_added = result.entries.iter().any(|e| {
            matches!(e, DiffEntry::LayerAdded { layer_id, .. } if layer_id == "sparkle")
        });
        assert!(sparkle_added, "sparkle layer addition in fixtures not detected");

        // bg color changed
        let color_changed = result.entries.iter().any(|e| {
            matches!(e, DiffEntry::LayerPropertyChanged { layer_id, property, .. }
                if layer_id == "bg" && property == "color")
        });
        assert!(color_changed, "bg color change in fixtures not detected");
    }
}
