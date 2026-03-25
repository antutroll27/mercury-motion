use rustyline::completion::{Completer, Pair};
use rustyline::highlight::{CmdKind, Highlighter};
use rustyline::hint::{Hinter, HistoryHinter};
use rustyline::validate::Validator;
use rustyline::{Context, Helper};
use std::borrow::Cow;

const COMMANDS: &[&str] = &["render", "validate", "scan", "clear", "help", "quit", "exit"];

const RENDER_FLAGS: &[&str] = &[
    "--output", "--format", "--quality", "--prop",
    "--include-audio", "--concurrency", "--verbose",
    "-o", "-f", "-q", "-v",
];

pub struct MmotCompleter {
    hinter: HistoryHinter,
}

impl MmotCompleter {
    pub fn new() -> Self {
        Self { hinter: HistoryHinter {} }
    }
}

impl Completer for MmotCompleter {
    type Candidate = Pair;

    fn complete(&self, line: &str, pos: usize, _ctx: &Context<'_>) -> rustyline::Result<(usize, Vec<Pair>)> {
        let line_up_to = &line[..pos];
        let tokens: Vec<&str> = line_up_to.split_whitespace().collect();

        if tokens.is_empty() || (tokens.len() == 1 && !line_up_to.ends_with(' ')) {
            let prefix = tokens.first().copied().unwrap_or("");
            let start = pos - prefix.len();
            let matches: Vec<Pair> = COMMANDS.iter()
                .filter(|c| c.starts_with(prefix))
                .map(|c| Pair { display: c.to_string(), replacement: c.to_string() })
                .collect();
            return Ok((start, matches));
        }

        let cmd = tokens[0];
        let partial = if line_up_to.ends_with(' ') { "" } else { tokens.last().copied().unwrap_or("") };
        let start = pos - partial.len();

        if cmd == "render" && partial.starts_with('-') {
            let matches: Vec<Pair> = RENDER_FLAGS.iter()
                .filter(|f| f.starts_with(partial))
                .map(|f| Pair { display: f.to_string(), replacement: f.to_string() })
                .collect();
            return Ok((start, matches));
        }

        if cmd == "render" || cmd == "validate" {
            let matches: Vec<Pair> = std::fs::read_dir(".")
                .into_iter().flatten()
                .filter_map(|e| e.ok())
                .filter_map(|e| {
                    let name = e.file_name().to_string_lossy().to_string();
                    if name.ends_with(".mmot.json") && name.starts_with(partial) {
                        Some(Pair { display: name.clone(), replacement: name })
                    } else { None }
                })
                .collect();
            return Ok((start, matches));
        }

        Ok((pos, vec![]))
    }
}

impl Hinter for MmotCompleter {
    type Hint = String;
    fn hint(&self, line: &str, pos: usize, ctx: &Context<'_>) -> Option<String> {
        self.hinter.hint(line, pos, ctx)
    }
}

impl Highlighter for MmotCompleter {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        if let Some(space_idx) = line.find(' ') {
            let cmd = &line[..space_idx];
            if COMMANDS.contains(&cmd) {
                return Cow::Owned(format!("\x1b[38;5;179m{}\x1b[0m{}", cmd, &line[space_idx..]));
            }
        } else if COMMANDS.contains(&line) {
            return Cow::Owned(format!("\x1b[38;5;179m{}\x1b[0m", line));
        }
        Cow::Borrowed(line)
    }

    fn highlight_char(&self, _line: &str, _pos: usize, _forced: CmdKind) -> bool { true }

    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Cow::Owned(format!("\x1b[38;5;66m{}\x1b[0m", hint))
    }
}

impl Validator for MmotCompleter {}
impl Helper for MmotCompleter {}
