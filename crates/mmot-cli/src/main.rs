mod completer;
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
        Commands::Interactive => {
            repl::run_repl();
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
                other => anyhow::bail!("unsupported format: '{other}' (expected mp4, gif, or webm)"),
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
