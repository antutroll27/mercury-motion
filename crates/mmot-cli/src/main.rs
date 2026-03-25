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
}

fn parse_prop(s: &str) -> std::result::Result<(String, String), String> {
    let pos = s
        .find('=')
        .ok_or_else(|| format!("invalid prop format: '{s}' (expected key=value)"))?;
    Ok((s[..pos].to_string(), s[pos + 1..].to_string()))
}

fn main() {
    tracing_subscriber::fmt::init();
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
