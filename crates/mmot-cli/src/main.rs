mod audit_format;
mod completer;
mod diff_format;
mod repl;
mod ui;

use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::Context;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "mmot",
    version = "0.1.0",
    about = "Mercury-Motion programmatic video renderer"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Render a .mmot.json file to video
    Render {
        file: PathBuf,
        #[arg(short, long, default_value = "output.mp4")]
        output: PathBuf,
        #[arg(short, long, default_value_t = 80)]
        quality: u8,
        #[arg(long)]
        concurrency: Option<usize>,
        /// Set a prop value: --prop key=value (repeatable)
        #[arg(long = "prop", value_parser = parse_prop)]
        props: Vec<(String, String)>,
        #[arg(short, long)]
        verbose: bool,
        /// Output format: mp4, gif, webm
        #[arg(short, long, default_value = "mp4")]
        format: String,
        /// Include audio tracks in output
        #[arg(long)]
        include_audio: bool,
    },
    /// Validate a .mmot.json file without rendering
    Validate { file: PathBuf },
    /// Render a single frame to PNG
    Frame {
        file: PathBuf,
        #[arg(short, long, default_value = "frame.png")]
        output: PathBuf,
        #[arg(short = 'n', long, default_value_t = 0)]
        frame: u64,
        /// Set a prop value: --prop key=value (repeatable)
        #[arg(long = "prop", value_parser = parse_prop)]
        props: Vec<(String, String)>,
    },
    /// Export to multiple aspect ratios (YouTube, Instagram, TikTok, etc.)
    ExportAll {
        /// Input .mmot.json file
        file: PathBuf,
        /// Output directory
        #[arg(short = 'd', long, default_value = "./exports")]
        output_dir: PathBuf,
        /// Profiles to export (comma-separated, or "all")
        #[arg(short, long, default_value = "all")]
        profiles: String,
        /// Output format: mp4, gif, webm
        #[arg(short, long, default_value = "mp4")]
        format: String,
        /// Quality (1-100)
        #[arg(short, long, default_value_t = 80)]
        quality: u8,
        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },
    /// Compare two .mmot.json files and show semantic differences
    Diff {
        /// First .mmot.json file (base)
        file_a: PathBuf,
        /// Second .mmot.json file (changed)
        file_b: PathBuf,
        /// Disable color output
        #[arg(long)]
        no_color: bool,
    },
    /// Run WCAG accessibility audit on a .mmot.json file
    Audit {
        /// Input .mmot.json file
        file: PathBuf,
        /// WCAG contrast level: aa or aaa
        #[arg(long, default_value = "aa")]
        level: String,
        /// Disable color output
        #[arg(long)]
        no_color: bool,
    },
    /// Enter interactive REPL mode
    Interactive,
}

fn parse_prop(s: &str) -> std::result::Result<(String, String), String> {
    let pos = s
        .find('=')
        .ok_or_else(|| format!("invalid prop format: '{s}' (expected key=value)"))?;
    Ok((s[..pos].to_string(), s[pos + 1..].to_string()))
}

/// Print styled help listing all REPL commands.
pub fn run_help() {
    ui::print_section("Commands");
    eprintln!(
        "  {:<20} {}",
        ui::gold("render <file>"),
        ui::slate("Render a .mmot.json file to video")
    );
    eprintln!(
        "  {:<20} {}",
        ui::gold("validate <file>"),
        ui::slate("Validate a .mmot.json file without rendering")
    );
    eprintln!(
        "  {:<20} {}",
        ui::gold("scan"),
        ui::slate("Scan current directory for .mmot.json files")
    );
    eprintln!(
        "  {:<20} {}",
        ui::gold("clear"),
        ui::slate("Clear the terminal screen")
    );
    eprintln!(
        "  {:<20} {}",
        ui::gold("help"),
        ui::slate("Show this help message")
    );
    eprintln!(
        "  {:<20} {}",
        ui::gold("quit / exit"),
        ui::slate("Exit the REPL")
    );
    eprintln!();
    ui::print_section("Render Flags");
    eprintln!(
        "  {:<20} {}",
        ui::gold("-o, --output"),
        ui::slate("Output file path (default: output.mp4)")
    );
    eprintln!(
        "  {:<20} {}",
        ui::gold("-f, --format"),
        ui::slate("Output format: mp4, gif, webm")
    );
    eprintln!(
        "  {:<20} {}",
        ui::gold("-q, --quality"),
        ui::slate("Encode quality 0-100 (default: 80)")
    );
    eprintln!(
        "  {:<20} {}",
        ui::gold("--prop key=val"),
        ui::slate("Set a template prop (repeatable)")
    );
    eprintln!(
        "  {:<20} {}",
        ui::gold("--include-audio"),
        ui::slate("Include audio tracks in output")
    );
    eprintln!(
        "  {:<20} {}",
        ui::gold("--concurrency N"),
        ui::slate("Number of parallel render threads")
    );
    eprintln!(
        "  {:<20} {}",
        ui::gold("-v, --verbose"),
        ui::slate("Show detailed scene info during render")
    );
    eprintln!();
}

fn main() {
    tracing_subscriber::fmt::init();

    // Intercept bare `mmot` (no args) and `mmot interactive` before clap parsing
    let args: Vec<String> = std::env::args().collect();
    if args.len() <= 1 {
        repl::run_repl();
        std::process::exit(0);
    }
    if args.len() == 2 && args[1] == "interactive" {
        repl::run_repl();
        std::process::exit(0);
    }

    let cli = Cli::parse();
    let exit_code = match run(cli) {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("{e}");
            match e.downcast_ref::<mmot_core::error::MmotError>() {
                Some(mmot_core::error::MmotError::Parse { .. }) => 2,
                Some(mmot_core::error::MmotError::AssetNotFound { .. }) => 3,
                _ => 1,
            }
        }
    };
    std::process::exit(exit_code);
}

fn run(cli: Cli) -> anyhow::Result<()> {
    match cli.command {
        Commands::Validate { file } => {
            let json = std::fs::read_to_string(&file)
                .with_context(|| format!("cannot read {}", file.display()))?;
            mmot_core::parser::parse(&json)?;
            println!("valid: {}", file.display());
            Ok(())
        }
        Commands::Frame {
            file,
            output,
            frame,
            props,
        } => {
            let json = std::fs::read_to_string(&file)
                .with_context(|| format!("cannot read {}", file.display()))?;
            let cli_props: HashMap<String, String> = props.into_iter().collect();
            let (width, height, rgba) =
                mmot_core::pipeline::render_single_frame_with_props(&json, &cli_props, frame)?;
            let image = image::RgbaImage::from_raw(width, height, rgba)
                .ok_or_else(|| anyhow::anyhow!("failed to create image from RGBA buffer"))?;
            image
                .save(&output)
                .with_context(|| format!("failed to write {}", output.display()))?;
            println!("rendered frame: {}", output.display());
            Ok(())
        }
        Commands::Diff {
            file_a,
            file_b,
            no_color,
        } => {
            let json_a = std::fs::read_to_string(&file_a)
                .with_context(|| format!("cannot read {}", file_a.display()))?;
            let json_b = std::fs::read_to_string(&file_b)
                .with_context(|| format!("cannot read {}", file_b.display()))?;

            let scene_a = mmot_core::parser::parse(&json_a)
                .with_context(|| format!("failed to parse {}", file_a.display()))?;
            let scene_b = mmot_core::parser::parse(&json_b)
                .with_context(|| format!("failed to parse {}", file_b.display()))?;

            let result = mmot_core::diff::diff(&scene_a, &scene_b);
            let output = diff_format::format_diff(&result, !no_color);
            println!("{output}");

            if result.has_changes() {
                std::process::exit(1);
            }
            Ok(())
        }
        Commands::Audit {
            file,
            level,
            no_color,
        } => {
            let json = std::fs::read_to_string(&file)
                .with_context(|| format!("cannot read {}", file.display()))?;
            let scene = mmot_core::parser::parse(&json)
                .with_context(|| format!("failed to parse {}", file.display()))?;

            let contrast_level = match level.to_lowercase().as_str() {
                "aa" => mmot_core::accessibility::ContrastLevel::AA,
                "aaa" => mmot_core::accessibility::ContrastLevel::AAA,
                other => {
                    anyhow::bail!("unsupported contrast level: '{other}' (expected aa or aaa)")
                }
            };

            let opts = mmot_core::accessibility::AuditOptions {
                contrast_level,
                suppress: Vec::new(),
            };

            let report = mmot_core::accessibility::audit(&scene, &opts);
            let output = audit_format::format_report(&report, !no_color);
            print!("{output}");

            if report.critical_count() > 0 {
                std::process::exit(1);
            }
            Ok(())
        }
        Commands::Interactive => {
            repl::run_repl();
            Ok(())
        }
        Commands::ExportAll {
            file,
            output_dir,
            profiles,
            format,
            quality,
            verbose,
        } => {
            let json = std::fs::read_to_string(&file)
                .with_context(|| format!("cannot read {}", file.display()))?;

            let format = match format.to_lowercase().as_str() {
                "mp4" => mmot_core::pipeline::OutputFormat::Mp4,
                "gif" => mmot_core::pipeline::OutputFormat::Gif,
                "webm" => mmot_core::pipeline::OutputFormat::Webm,
                other => {
                    anyhow::bail!("unsupported format: '{other}' (expected mp4, gif, or webm)")
                }
            };

            let all_profiles = mmot_core::export::builtin_profiles();
            let selected: Vec<mmot_core::export::ExportProfile> = if profiles == "all" {
                all_profiles
            } else {
                let names: Vec<&str> = profiles.split(',').map(|s| s.trim()).collect();
                let mut selected = Vec::new();
                for name in &names {
                    match all_profiles.iter().find(|p| p.name == *name) {
                        Some(p) => selected.push(p.clone()),
                        None => {
                            let available: Vec<&str> = all_profiles.iter().map(|p| p.name.as_str()).collect();
                            anyhow::bail!(
                                "unknown profile '{}'. Available: {}",
                                name,
                                available.join(", ")
                            );
                        }
                    }
                }
                selected
            };

            if verbose {
                eprintln!("Exporting {} profiles to {}", selected.len(), output_dir.display());
                for p in &selected {
                    eprintln!("  {} ({}x{})", p.name, p.width, p.height);
                }
            }

            let progress: Option<mmot_core::pipeline::ProgressFn> = if verbose {
                Some(std::sync::Arc::new(|current, total| {
                    eprint!("\rExporting profile {current}/{total}");
                }))
            } else {
                None
            };

            let opts = mmot_core::export::ExportOptions {
                output_dir: output_dir.clone(),
                profiles: selected,
                quality,
                concurrency: None,
                format,
            };

            let results = mmot_core::export::export_all(&json, opts, progress)?;
            if verbose {
                eprintln!();
            }
            for r in &results {
                println!("  {} ({}x{}) -> {}", r.profile_name, r.width, r.height, r.output_path.display());
            }
            println!("exported {} profiles to {}", results.len(), output_dir.display());
            Ok(())
        }
        Commands::Render {
            file,
            output,
            quality,
            concurrency,
            props,
            verbose,
            format,
            include_audio,
        } => {
            let json = std::fs::read_to_string(&file)
                .with_context(|| format!("cannot read {}", file.display()))?;

            let cli_props: HashMap<String, String> = props.into_iter().collect();

            let format = match format.to_lowercase().as_str() {
                "mp4" => mmot_core::pipeline::OutputFormat::Mp4,
                "gif" => mmot_core::pipeline::OutputFormat::Gif,
                "webm" => mmot_core::pipeline::OutputFormat::Webm,
                other => {
                    anyhow::bail!("unsupported format: '{other}' (expected mp4, gif, or webm)")
                }
            };

            let progress: Option<mmot_core::pipeline::ProgressFn> = if verbose {
                Some(std::sync::Arc::new(|current, total| {
                    eprint!("\rRendering frame {current}/{total}");
                }))
            } else {
                None
            };

            let opts = mmot_core::pipeline::RenderOptions {
                output_path: output.clone(),
                format,
                quality,
                frame_range: None,
                concurrency,
                backend: mmot_core::pipeline::RenderBackend::Cpu,
                include_audio,
            };

            mmot_core::pipeline::render_scene_with_props(&json, &cli_props, opts, progress)?;
            if verbose {
                eprintln!();
            }
            println!("rendered: {}", output.display());
            Ok(())
        }
    }
}
