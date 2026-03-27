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
    /// Visual regression test — compare rendered frames against golden references
    Test {
        /// .mmot.json file to test
        file: PathBuf,
        /// Update golden references instead of comparing
        #[arg(long)]
        update: bool,
        /// Golden directory (default: tests/golden/{scene_name}/)
        #[arg(long)]
        golden_dir: Option<PathBuf>,
        /// Frames to test (comma-separated, e.g. "0,15,29")
        #[arg(long)]
        frames: Option<String>,
        /// Pixel difference tolerance percentage (default: 0.1)
        #[arg(long, default_value_t = 0.1)]
        tolerance: f64,
    },
    /// Batch-render videos from a template + CSV/JSON data source
    Batch {
        /// Template .mmot.json file with ${variable} placeholders
        file: PathBuf,
        /// Data source file (CSV or JSON array)
        #[arg(long)]
        data: PathBuf,
        /// Output directory for rendered videos
        #[arg(short = 'd', long, default_value = "./batch-output")]
        output_dir: PathBuf,
        /// Output format: mp4, gif, webm
        #[arg(short, long, default_value = "mp4")]
        format: String,
        /// Encode quality (1-100)
        #[arg(short, long, default_value_t = 80)]
        quality: u8,
        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
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
        Commands::Test {
            file,
            update,
            golden_dir,
            frames,
            tolerance,
        } => {
            let json = std::fs::read_to_string(&file)
                .with_context(|| format!("cannot read {}", file.display()))?;

            let info = mmot_core::pipeline::get_scene_info(&json)
                .with_context(|| format!("failed to parse {}", file.display()))?;

            // Determine golden directory
            let golden_dir = golden_dir.unwrap_or_else(|| {
                let stem = file
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown");
                // Strip .mmot suffix if present (file is foo.mmot.json -> stem is foo.mmot)
                let clean_stem = stem.strip_suffix(".mmot").unwrap_or(stem);
                PathBuf::from("tests/golden").join(clean_stem)
            });

            // Determine frames to test
            let frame_list: Vec<u64> = if let Some(ref spec) = frames {
                spec.split(',')
                    .map(|s| {
                        s.trim()
                            .parse::<u64>()
                            .map_err(|_| anyhow::anyhow!("invalid frame number: '{}'", s.trim()))
                    })
                    .collect::<anyhow::Result<Vec<_>>>()?
            } else {
                mmot_core::visual_test::default_frames(info.duration_frames)
            };

            if update {
                let count =
                    mmot_core::visual_test::update_goldens(&json, &golden_dir, &frame_list)
                        .with_context(|| "failed to generate golden frames")?;
                println!(
                    "generated {count} golden frame(s) in {}",
                    golden_dir.display()
                );
            } else {
                let result = mmot_core::visual_test::run_visual_test(
                    &json,
                    &file,
                    &golden_dir,
                    &frame_list,
                    tolerance,
                )
                .with_context(|| "visual test failed")?;

                if result.passed() {
                    println!(
                        "PASS ({} frame(s) match)",
                        result.frames_passed
                    );
                } else {
                    println!(
                        "FAIL ({}/{} frame(s) passed)",
                        result.frames_passed, result.frames_tested
                    );
                    for f in &result.failures {
                        println!(
                            "  frame {} differs: {:.2}% pixel difference",
                            f.frame, f.pixel_diff_percent
                        );
                        if f.actual_path.as_os_str().is_empty() {
                            println!("    golden not found: {}", f.golden_path.display());
                        } else {
                            println!("    golden:  {}", f.golden_path.display());
                            println!("    actual:  {}", f.actual_path.display());
                        }
                    }
                    std::process::exit(1);
                }
            }
            Ok(())
        }
        Commands::Batch {
            file,
            data,
            output_dir,
            format,
            quality,
            verbose,
        } => {
            let template_json = std::fs::read_to_string(&file)
                .with_context(|| format!("cannot read template {}", file.display()))?;

            let format = match format.to_lowercase().as_str() {
                "mp4" => mmot_core::pipeline::OutputFormat::Mp4,
                "gif" => mmot_core::pipeline::OutputFormat::Gif,
                "webm" => mmot_core::pipeline::OutputFormat::Webm,
                other => {
                    anyhow::bail!("unsupported format: '{other}' (expected mp4, gif, or webm)")
                }
            };

            // Detect data format by extension
            let ext = data
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();
            let data_rows = match ext.as_str() {
                "csv" => mmot_core::batch::parse_csv(&data)
                    .with_context(|| format!("failed to parse CSV {}", data.display()))?,
                "json" => mmot_core::batch::parse_json_data(&data)
                    .with_context(|| format!("failed to parse JSON data {}", data.display()))?,
                other => {
                    anyhow::bail!(
                        "unsupported data format: '.{other}' (expected .csv or .json)"
                    )
                }
            };

            if verbose {
                eprintln!(
                    "Batch: {} rows from {}, rendering to {}",
                    data_rows.len(),
                    data.display(),
                    output_dir.display()
                );
            }

            let progress: Option<Box<dyn Fn(usize, usize) + Send + Sync>> = if verbose {
                Some(Box::new(|current, total| {
                    eprint!("\rRendering {current}/{total}");
                }))
            } else {
                None
            };

            let opts = mmot_core::batch::BatchOptions {
                template_json,
                output_dir: output_dir.clone(),
                format,
                quality,
                concurrency: None,
            };

            let result = mmot_core::batch::render_batch(opts, &data_rows, progress)
                .with_context(|| "batch render failed")?;

            if verbose {
                eprintln!();
            }
            println!(
                "batch complete: {}/{} rendered to {}",
                result.rendered,
                result.total,
                output_dir.display()
            );
            for (idx, err) in &result.failed {
                eprintln!("  row {idx} failed: {err}");
            }
            if !result.failed.is_empty() {
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
