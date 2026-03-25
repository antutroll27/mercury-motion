use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use anyhow::Context;
use indicatif::{ProgressBar, ProgressStyle};
use rustyline::error::ReadlineError;
use rustyline::Editor;

use crate::completer::MmotCompleter;
use crate::ui;

/// Scan the current directory for .mmot.json files and print a styled table.
fn scan_scenes() {
    ui::print_section("Project Scanner");

    let entries: Vec<_> = std::fs::read_dir(".")
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_name()
                .to_string_lossy()
                .ends_with(".mmot.json")
        })
        .collect();

    if entries.is_empty() {
        ui::print_step("i", "No .mmot.json files", "found in current directory");
        return;
    }

    let total = entries.len();
    let show_count = total.min(8);

    eprintln!(
        "  {:<30} {:>8} {:>10} {:>12}",
        ui::slate("File").to_string(),
        ui::slate("Frames").to_string(),
        ui::slate("Duration").to_string(),
        ui::slate("Resolution").to_string(),
    );
    eprintln!("  {}", ui::slate("─".repeat(64)));

    for entry in entries.iter().take(show_count) {
        let name = entry.file_name().to_string_lossy().to_string();
        let display_name = if name.len() > 28 {
            format!("{}...", &name[..25])
        } else {
            name.clone()
        };

        match std::fs::read_to_string(entry.path()) {
            Ok(json) => match mmot_core::parser::parse(&json) {
                Ok(scene) => {
                    let m = &scene.meta;
                    let secs = m.duration as f64 / m.fps;
                    eprintln!(
                        "  {:<30} {:>8} {:>9.1}s {:>5}x{}",
                        ui::vanilla(&display_name),
                        ui::gold(m.duration),
                        secs,
                        m.width,
                        m.height,
                    );
                }
                Err(_) => {
                    eprintln!(
                        "  {:<30} {}",
                        ui::auburn(&display_name),
                        ui::auburn("(invalid)"),
                    );
                }
            },
            Err(_) => {
                eprintln!(
                    "  {:<30} {}",
                    ui::auburn(&display_name),
                    ui::auburn("(unreadable)"),
                );
            }
        }
    }

    if total > 8 {
        eprintln!();
        eprintln!("  {} ... and {} more", ui::slate("↓"), total - 8);
    }
    eprintln!();
}

/// Validate a .mmot.json file and print styled output.
fn cmd_validate(args: &[String]) -> anyhow::Result<()> {
    let file = args
        .first()
        .ok_or_else(|| anyhow::anyhow!("usage: validate <file.mmot.json>"))?;

    ui::print_step("●", "Validate", file);

    let json = std::fs::read_to_string(file)
        .with_context(|| format!("cannot read {file}"))?;

    let scene = mmot_core::parser::parse(&json)?;
    let m = &scene.meta;
    let secs = m.duration as f64 / m.fps;

    ui::print_step("✓", "Valid", &format!("{file} passed all checks"));
    ui::print_summary_box(&[
        ("Name", m.name.clone()),
        ("Resolution", format!("{}x{}", m.width, m.height)),
        ("FPS", format!("{}", m.fps)),
        ("Duration", format!("{} frames ({:.1}s)", m.duration, secs)),
        ("Background", m.background.clone()),
        ("Root comp", m.root.clone()),
    ]);
    Ok(())
}

/// Parse render flags with clap and execute the render pipeline.
fn cmd_render(args: &[String]) -> anyhow::Result<()> {
    // Build a clap Command to parse the render sub-arguments
    let cmd = clap::Command::new("render")
        .no_binary_name(true)
        .arg(clap::Arg::new("file").required(true))
        .arg(
            clap::Arg::new("output")
                .short('o')
                .long("output")
                .default_value("output.mp4"),
        )
        .arg(
            clap::Arg::new("format")
                .short('f')
                .long("format")
                .default_value("mp4"),
        )
        .arg(
            clap::Arg::new("quality")
                .short('q')
                .long("quality")
                .default_value("80"),
        )
        .arg(
            clap::Arg::new("prop")
                .long("prop")
                .action(clap::ArgAction::Append),
        )
        .arg(
            clap::Arg::new("include-audio")
                .long("include-audio")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(clap::Arg::new("concurrency").long("concurrency"))
        .arg(
            clap::Arg::new("verbose")
                .short('v')
                .long("verbose")
                .action(clap::ArgAction::SetTrue),
        );

    let matches = cmd
        .try_get_matches_from(args)
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    let file: &String = matches
        .get_one("file")
        .ok_or_else(|| anyhow::anyhow!("file argument is required"))?;
    let output: &String = matches
        .get_one("output")
        .ok_or_else(|| anyhow::anyhow!("output argument missing"))?;
    let format_str: &String = matches
        .get_one("format")
        .ok_or_else(|| anyhow::anyhow!("format argument missing"))?;
    let quality_str: &String = matches
        .get_one("quality")
        .ok_or_else(|| anyhow::anyhow!("quality argument missing"))?;
    let include_audio = matches.get_flag("include-audio");
    let verbose = matches.get_flag("verbose");

    let concurrency: Option<usize> = matches
        .get_one::<String>("concurrency")
        .map(|s| s.parse::<usize>())
        .transpose()
        .map_err(|_| anyhow::anyhow!("--concurrency must be a number"))?;

    let quality: u8 = quality_str
        .parse()
        .map_err(|_| anyhow::anyhow!("--quality must be 0-100"))?;

    let props: Vec<(String, String)> = matches
        .get_many::<String>("prop")
        .into_iter()
        .flatten()
        .map(|s| {
            let pos = s
                .find('=')
                .ok_or_else(|| anyhow::anyhow!("invalid prop format: '{s}' (expected key=value)"));
            pos.map(|p| (s[..p].to_string(), s[p + 1..].to_string()))
        })
        .collect::<anyhow::Result<Vec<_>>>()?;

    let cli_props: HashMap<String, String> = props.into_iter().collect();

    let format = match format_str.to_lowercase().as_str() {
        "mp4" => mmot_core::pipeline::OutputFormat::Mp4,
        "gif" => mmot_core::pipeline::OutputFormat::Gif,
        "webm" => mmot_core::pipeline::OutputFormat::Webm,
        other => anyhow::bail!("unsupported format: '{other}' (expected mp4, gif, or webm)"),
    };

    // Read and parse the file first to get metadata for display
    ui::print_step("●", "Render", file);

    let json = std::fs::read_to_string(file)
        .with_context(|| format!("cannot read {file}"))?;
    let scene = mmot_core::parser::parse(&json)?;
    let m = &scene.meta;
    let total_frames = m.duration;

    if verbose {
        ui::print_step("i", "Scene", &format!("{}  {}x{}  {} fps  {} frames",
            m.name, m.width, m.height, m.fps, m.duration));
    }

    // Set up indicatif progress bar
    let pb = ProgressBar::new(total_frames);
    let bar_style = ProgressStyle::default_bar()
        .template("  {spinner:.yellow} Rendering  [{bar:30.yellow/dim}]  {pos}/{len} frames  {elapsed_precise}")
        .expect("valid progress template")
        .progress_chars("█▓░");
    pb.set_style(bar_style);

    let pb_clone = pb.clone();
    let progress: Option<mmot_core::pipeline::ProgressFn> =
        Some(Arc::new(move |current, _total| {
            pb_clone.set_position(current);
        }));

    let output_path = PathBuf::from(output);
    let opts = mmot_core::pipeline::RenderOptions {
        output_path: output_path.clone(),
        format,
        quality,
        frame_range: None,
        concurrency,
        backend: mmot_core::pipeline::RenderBackend::Cpu,
        include_audio,
    };

    let start = Instant::now();
    mmot_core::pipeline::render_scene_with_props(&json, &cli_props, opts, progress)?;
    let elapsed = start.elapsed();

    pb.finish_and_clear();

    let fps_rate = if elapsed.as_secs_f64() > 0.0 {
        total_frames as f64 / elapsed.as_secs_f64()
    } else {
        0.0
    };

    ui::print_step("✓", "Done", &format!("{}", output_path.display()));
    ui::print_summary_box(&[
        ("Output", output_path.display().to_string()),
        ("Frames", format!("{total_frames}")),
        ("Time", format!("{:.2}s", elapsed.as_secs_f64())),
        ("Speed", format!("{fps_rate:.1} fps")),
        ("Quality", format!("{quality}")),
    ]);

    Ok(())
}

/// Print the help screen (delegates to the run_help function in main).
fn cmd_help() {
    crate::run_help();
}

/// The REPL prompt states.
enum PromptState {
    Fresh,
    Ok,
    Err,
}

fn make_prompt(state: &PromptState) -> String {
    match state {
        PromptState::Fresh => "\x1b[38;5;179mmmot \u{203a}\x1b[0m ".to_string(),
        PromptState::Ok => "\x1b[38;5;179mmmot \u{2713} \u{203a}\x1b[0m ".to_string(),
        PromptState::Err => "\x1b[38;5;179mmmot \u{2717} \u{203a}\x1b[0m ".to_string(),
    }
}

/// Main interactive REPL loop.
pub fn run_repl() {
    ui::print_banner();

    let helper = MmotCompleter::new();
    let config = rustyline::Config::builder()
        .auto_add_history(true)
        .build();
    let mut rl = Editor::with_config(config).expect("failed to create line editor");
    rl.set_helper(Some(helper));

    // Load history from ~/.mmot_history
    let history_path = dirs_next::home_dir().map(|h| h.join(".mmot_history"));
    if let Some(ref path) = history_path {
        let _ = rl.load_history(path);
    }

    let mut prompt_state = PromptState::Fresh;

    loop {
        let prompt = make_prompt(&prompt_state);
        match rl.readline(&prompt) {
            Ok(line) => {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }

                let tokens = match shell_words::split(trimmed) {
                    Ok(t) => t,
                    Err(e) => {
                        ui::print_error_step("✗", "Parse error", &format!("{e}"));
                        prompt_state = PromptState::Err;
                        continue;
                    }
                };

                if tokens.is_empty() {
                    continue;
                }

                let cmd = tokens[0].as_str();
                let args = &tokens[1..];

                match cmd {
                    "quit" | "exit" => break,
                    "clear" => {
                        // Clear terminal with ANSI escape
                        eprint!("\x1b[2J\x1b[H");
                        prompt_state = PromptState::Fresh;
                    }
                    "scan" => {
                        scan_scenes();
                        prompt_state = PromptState::Ok;
                    }
                    "help" => {
                        cmd_help();
                        prompt_state = PromptState::Ok;
                    }
                    "validate" => match cmd_validate(args) {
                        Ok(()) => {
                            prompt_state = PromptState::Ok;
                        }
                        Err(e) => {
                            ui::print_error_step("✗", "Validation failed", &format!("{e}"));
                            prompt_state = PromptState::Err;
                        }
                    },
                    "render" => match cmd_render(args) {
                        Ok(()) => {
                            prompt_state = PromptState::Ok;
                        }
                        Err(e) => {
                            ui::print_error_step("✗", "Render failed", &format!("{e}"));
                            prompt_state = PromptState::Err;
                        }
                    },
                    other => {
                        ui::print_error_step(
                            "?",
                            "Unknown command",
                            &format!("'{other}' — type help for available commands"),
                        );
                        prompt_state = PromptState::Err;
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                eprintln!("  (Type quit to exit)");
            }
            Err(ReadlineError::Eof) => break,
            Err(e) => {
                ui::print_error_step("✗", "Input error", &format!("{e}"));
                break;
            }
        }
    }

    // Save history
    if let Some(ref path) = history_path {
        let _ = rl.save_history(path);
    }

    eprintln!();
    ui::print_step("●", "Goodbye", "— happy rendering!");
    eprintln!();
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    #[test]
    fn tokenize_simple_render_command() {
        let tokens = shell_words::split("render scene.mmot.json --quality 90").unwrap();
        assert_eq!(tokens, vec!["render", "scene.mmot.json", "--quality", "90"]);
    }

    #[test]
    fn tokenize_quoted_prop() {
        let tokens =
            shell_words::split(r#"render scene.mmot.json --prop "title=Hello World""#).unwrap();
        assert_eq!(
            tokens,
            vec!["render", "scene.mmot.json", "--prop", "title=Hello World"]
        );
    }

    #[test]
    fn tokenize_empty_input() {
        let tokens = shell_words::split("").unwrap();
        assert!(tokens.is_empty());
    }

    #[test]
    fn scan_finds_mmot_files_in_temp_dir() {
        let dir = tempfile::TempDir::new().unwrap();

        let mmot_path = dir.path().join("scene.mmot.json");
        let mut f = std::fs::File::create(&mmot_path).unwrap();
        f.write_all(
            br##"{"meta":{"name":"Test","width":100,"height":100,"fps":30,"duration":10,"background":"#000","root":"main"},"compositions":{"main":{"layers":[]}}}"##,
        )
        .unwrap();
        drop(f);

        let txt_path = dir.path().join("notes.txt");
        std::fs::write(&txt_path, b"not a scene file").unwrap();

        let mmot_files: Vec<_> = std::fs::read_dir(dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_name()
                    .to_string_lossy()
                    .ends_with(".mmot.json")
            })
            .collect();

        assert_eq!(
            mmot_files.len(),
            1,
            "expected exactly 1 .mmot.json file, found {}",
            mmot_files.len()
        );
    }
}
