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
        #[arg(short, long, default_value = "output.ivf")]
        output: PathBuf,
        #[arg(short, long, default_value_t = 80)]
        quality: u8,
        #[arg(long)]
        concurrency: Option<usize>,
        #[arg(short, long)]
        verbose: bool,
    },
    /// Validate a .mmot.json file without rendering
    Validate { file: PathBuf },
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
            let json =
                std::fs::read_to_string(&file).with_context(|| format!("cannot read {}", file.display()))?;
            mmot_core::parser::parse(&json)?;
            println!("valid: {}", file.display());
            Ok(())
        }
        Commands::Render {
            file,
            output,
            quality,
            concurrency,
            verbose,
        } => {
            let json =
                std::fs::read_to_string(&file).with_context(|| format!("cannot read {}", file.display()))?;

            let progress: Option<mmot_core::pipeline::ProgressFn> = if verbose {
                Some(std::sync::Arc::new(|current, total| {
                    eprint!("\rRendering frame {current}/{total}");
                }))
            } else {
                None
            };

            let opts = mmot_core::pipeline::RenderOptions {
                output_path: output.clone(),
                format: mmot_core::pipeline::OutputFormat::Mp4,
                quality,
                frame_range: None,
                concurrency,
                backend: mmot_core::pipeline::RenderBackend::Cpu,
                include_audio: false,
            };

            mmot_core::pipeline::render_scene(&json, opts, progress)?;
            if verbose {
                eprintln!();
            }
            println!("rendered: {}", output.display());
            Ok(())
        }
    }
}
