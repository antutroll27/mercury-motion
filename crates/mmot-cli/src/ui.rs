use console::{style, StyledObject};

// Palette: Hunyadi Yellow #E09F3E, Auburn #9E2A2B, Dark Slate Gray #335C67, Vanilla #FFF3B0

pub fn gold<D: std::fmt::Display>(val: D) -> StyledObject<D> {
    style(val).color256(179)
}

pub fn auburn<D: std::fmt::Display>(val: D) -> StyledObject<D> {
    style(val).color256(124)
}

pub fn slate<D: std::fmt::Display>(val: D) -> StyledObject<D> {
    style(val).color256(66)
}

pub fn vanilla<D: std::fmt::Display>(val: D) -> StyledObject<D> {
    style(val).color256(229)
}

pub fn print_banner() {
    eprintln!();
    eprintln!("  {}", gold("███╗   ███╗███╗   ███╗ ██████╗ ████████╗").bold());
    eprintln!("  {}", gold("████╗ ████║████╗ ████║██╔═══██╗╚══██╔══╝").bold());
    eprintln!("  {}", gold("██╔████╔██║██╔████╔██║██║   ██║   ██║").bold());
    eprintln!("  {}", gold("██║╚██╔╝██║██║╚██╔╝██║██║   ██║   ██║").bold());
    eprintln!("  {}", gold("██║ ╚═╝ ██║██║ ╚═╝ ██║╚██████╔╝   ██║").bold());
    eprintln!("  {}", gold("╚═╝     ╚═╝╚═╝     ╚═╝ ╚═════╝    ╚═╝").bold());
    eprintln!();
    eprint!("  {}", vanilla("Mercury-Motion").bold());
    eprintln!("  {}", slate("•  Video Engine"));
    eprint!("  {}", slate("JSON → MP4/WebM/GIF"));
    eprint!("  {}  ", slate("•"));
    eprintln!("{}", vanilla("100× faster").bold());
    eprintln!();
    eprint!("  {}  ", slate("New here?"));
    eprint!("{}", gold("mmot help").bold());
    eprintln!("  {}", slate("to get started."));
    eprintln!();
}

pub fn print_step(icon: &str, label: &str, detail: &str) {
    eprintln!("  {} {} {}", gold(icon).bold(), vanilla(label).bold(), slate(detail));
}

pub fn print_error_step(icon: &str, label: &str, detail: &str) {
    eprintln!("  {} {} {}", auburn(icon).bold(), vanilla(label).bold(), auburn(detail));
}

pub fn print_summary_box(rows: &[(&str, String)]) {
    let max_key = rows.iter().map(|(k, _)| k.len()).max().unwrap_or(0);
    let max_val = rows.iter().map(|(_, v)| v.len()).max().unwrap_or(0);
    let inner_width = max_key + max_val + 5;
    let width = inner_width.max(30);
    eprintln!();
    eprintln!("  {}{}{}", slate("╭"), slate("─".repeat(width)), slate("╮"));
    for (key, val) in rows {
        let padding = width - key.len() - val.len() - 4;
        eprintln!("  {}  {}{}{}  {}", slate("│"), slate(*key), " ".repeat(padding), vanilla(val).bold(), slate("│"));
    }
    eprintln!("  {}{}{}", slate("╰"), slate("─".repeat(width)), slate("╯"));
    eprintln!();
}

pub fn print_section(title: &str) {
    eprintln!("  {}", gold(title).bold().underlined());
    eprintln!();
}
