use clap::Parser;
use std::path::Path;
use tracing::{info, error, Level};
use tracing_subscriber::FmtSubscriber;
use std::process::ExitCode;

mod core;
mod analysis;
mod reporting;

use core::config::LintConfig;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = ".")]
    path: String,

    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    #[arg(short, long)]
    strict: bool,

    #[arg(long)]
    json: bool,

    /// Fail the execution with exit code 1 if smells are found (Critical for CI)
    #[arg(long)]
    fail_on_error: bool,
}

// Cambiamos el retorno a ExitCode
fn main() -> ExitCode {
    let args = Args::parse();

    // 1. Logger Setup
    let log_level = if args.json { Level::WARN } else { Level::INFO };
    let subscriber = FmtSubscriber::builder()
        .with_max_level(log_level)
        .with_writer(std::io::stderr)
        .finish();
        
    if let Err(e) = tracing::subscriber::set_global_default(subscriber) {
        eprintln!("Logger init failed: {}", e);
        return ExitCode::FAILURE;
    }

    let target_path = Path::new(&args.path);

    // 2. Load Config
    let config = if args.strict {
        LintConfig::strict()
    } else {
        LintConfig::default()
    };

    if !args.json {
        info!("Starting lint on: {}", args.path);
    }

    // 3. Execution
    match analysis::walk_directory(target_path) {
        Ok(files) => {
            if !args.json {
                info!("Found {} files to analyze.", files.len());
            }
            
            let smells = analysis::engine::run_analysis(&files, &config);
            let smell_count = smells.len();

            // 4. Reporting
            if args.json {
                reporting::json::print_report(&smells);
            } else {
                reporting::console::print_report(&smells);
            }

            // 5. Exit Strategy
            if args.fail_on_error && smell_count > 0 {
                return ExitCode::FAILURE;
            }
            
            ExitCode::SUCCESS
        }
        Err(e) => {
            error!("Failed to walk directory: {}", e);
            ExitCode::FAILURE
        }
    }
}