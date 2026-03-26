use console::Style;
use mmot_core::accessibility::{AuditReport, Severity};

/// Format an [`AuditReport`] as a human-readable string with optional ANSI
/// colour codes.
pub fn format_report(report: &AuditReport, color: bool) -> String {
    let mut out = String::new();

    let critical_style = if color {
        Style::new().red().bold()
    } else {
        Style::new()
    };
    let warning_style = if color {
        Style::new().yellow()
    } else {
        Style::new()
    };
    let info_style = if color {
        Style::new().cyan()
    } else {
        Style::new()
    };

    for finding in &report.findings {
        let (icon, style) = match finding.severity {
            Severity::Critical => ("CRITICAL", &critical_style),
            Severity::Warning => ("WARNING", &warning_style),
            Severity::Info => ("INFO", &info_style),
        };
        out.push_str(&format!(
            "{} {} {}\n",
            style.apply_to(icon),
            finding.pointer,
            finding.message
        ));
        if let Some((start, end)) = finding.frame_range {
            out.push_str(&format!("  frames {}-{}\n", start, end));
        }
    }

    out.push_str(&format!(
        "\nSummary: {} critical, {} warnings, {} info\n",
        report.critical_count(),
        report.warning_count(),
        report.info_count(),
    ));

    out
}
