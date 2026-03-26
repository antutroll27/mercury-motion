use console::Style;
use mmot_core::diff::{DiffEntry, DiffResult};

/// Format a diff result for terminal output.
///
/// When `color` is true, additions are green, removals are red, and
/// property changes are yellow.
pub fn format_diff(result: &DiffResult, color: bool) -> String {
    if result.is_empty() {
        return if color {
            let dim = Style::new().dim();
            dim.apply_to("No changes detected.").to_string()
        } else {
            "No changes detected.".to_string()
        };
    }

    let add_style = if color {
        Style::new().green()
    } else {
        Style::new()
    };
    let rm_style = if color {
        Style::new().red()
    } else {
        Style::new()
    };
    let change_style = if color {
        Style::new().yellow()
    } else {
        Style::new()
    };
    let header_style = if color {
        Style::new().bold().cyan()
    } else {
        Style::new().bold()
    };

    let mut lines: Vec<String> = Vec::new();

    // ── Group entries ────────────────────────────────────────────────────────
    let mut meta_entries = Vec::new();
    let mut comp_entries = Vec::new();
    let mut layer_entries = Vec::new();

    for entry in &result.entries {
        match entry {
            DiffEntry::MetaChanged { .. } => meta_entries.push(entry),
            DiffEntry::CompositionAdded { .. } | DiffEntry::CompositionRemoved { .. } => {
                comp_entries.push(entry)
            }
            _ => layer_entries.push(entry),
        }
    }

    // ── Meta section ─────────────────────────────────────────────────────────
    if !meta_entries.is_empty() {
        lines.push(header_style.apply_to("Meta").to_string());
        for entry in &meta_entries {
            if let DiffEntry::MetaChanged { field, old, new } = entry {
                lines.push(format!(
                    "  {} {}: {} -> {}",
                    change_style.apply_to("~"),
                    field,
                    old,
                    new,
                ));
            }
        }
        lines.push(String::new());
    }

    // ── Compositions section ─────────────────────────────────────────────────
    if !comp_entries.is_empty() {
        lines.push(header_style.apply_to("Compositions").to_string());
        for entry in &comp_entries {
            match entry {
                DiffEntry::CompositionAdded { id } => {
                    lines.push(format!(
                        "  {} composition \"{}\"",
                        add_style.apply_to("+"),
                        id,
                    ));
                }
                DiffEntry::CompositionRemoved { id } => {
                    lines.push(format!(
                        "  {} composition \"{}\"",
                        rm_style.apply_to("-"),
                        id,
                    ));
                }
                _ => {}
            }
        }
        lines.push(String::new());
    }

    // ── Layers section ───────────────────────────────────────────────────────
    if !layer_entries.is_empty() {
        lines.push(header_style.apply_to("Layers").to_string());
        for entry in &layer_entries {
            match entry {
                DiffEntry::LayerAdded {
                    composition,
                    layer_id,
                    layer_type,
                    in_point,
                    out_point,
                } => {
                    lines.push(format!(
                        "  {} [{}/{}] ({}) frames {}-{}",
                        add_style.apply_to("+"),
                        composition,
                        layer_id,
                        layer_type,
                        in_point,
                        out_point,
                    ));
                }
                DiffEntry::LayerRemoved {
                    composition,
                    layer_id,
                    layer_type,
                } => {
                    lines.push(format!(
                        "  {} [{}/{}] ({})",
                        rm_style.apply_to("-"),
                        composition,
                        layer_id,
                        layer_type,
                    ));
                }
                DiffEntry::LayerPropertyChanged {
                    composition,
                    layer_id,
                    property,
                    old,
                    new,
                } => {
                    lines.push(format!(
                        "  {} [{}/{}] {}: {} -> {}",
                        change_style.apply_to("~"),
                        composition,
                        layer_id,
                        property,
                        old,
                        new,
                    ));
                }
                DiffEntry::EffectAdded {
                    composition,
                    layer_id,
                    effect_type,
                } => {
                    lines.push(format!(
                        "  {} [{}/{}] effect: {}",
                        add_style.apply_to("+"),
                        composition,
                        layer_id,
                        effect_type,
                    ));
                }
                DiffEntry::EffectRemoved {
                    composition,
                    layer_id,
                    effect_type,
                } => {
                    lines.push(format!(
                        "  {} [{}/{}] effect: {}",
                        rm_style.apply_to("-"),
                        composition,
                        layer_id,
                        effect_type,
                    ));
                }
                DiffEntry::MaskAdded {
                    composition,
                    layer_id,
                } => {
                    lines.push(format!(
                        "  {} [{}/{}] mask",
                        add_style.apply_to("+"),
                        composition,
                        layer_id,
                    ));
                }
                DiffEntry::MaskRemoved {
                    composition,
                    layer_id,
                } => {
                    lines.push(format!(
                        "  {} [{}/{}] mask",
                        rm_style.apply_to("-"),
                        composition,
                        layer_id,
                    ));
                }
                _ => {}
            }
        }
        lines.push(String::new());
    }

    // ── Summary ──────────────────────────────────────────────────────────────
    let additions = result
        .entries
        .iter()
        .filter(|e| {
            matches!(
                e,
                DiffEntry::CompositionAdded { .. }
                    | DiffEntry::LayerAdded { .. }
                    | DiffEntry::EffectAdded { .. }
                    | DiffEntry::MaskAdded { .. }
            )
        })
        .count();
    let removals = result
        .entries
        .iter()
        .filter(|e| {
            matches!(
                e,
                DiffEntry::CompositionRemoved { .. }
                    | DiffEntry::LayerRemoved { .. }
                    | DiffEntry::EffectRemoved { .. }
                    | DiffEntry::MaskRemoved { .. }
            )
        })
        .count();
    let changes = result
        .entries
        .iter()
        .filter(|e| {
            matches!(
                e,
                DiffEntry::MetaChanged { .. } | DiffEntry::LayerPropertyChanged { .. }
            )
        })
        .count();

    let summary = format!(
        "{} addition(s), {} removal(s), {} change(s)",
        additions, removals, changes
    );
    lines.push(if color {
        Style::new().dim().apply_to(summary).to_string()
    } else {
        summary
    });

    lines.join("\n")
}
