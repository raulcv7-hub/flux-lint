use clap::Parser;
use std::path::Path;
use tracing::{info, error, Level};
use tracing_subscriber::FmtSubscriber;

// Import modules
mod core;
mod analysis;
mod reporting;

use core::config::AuditConfig;

/// Audit: High-performance static analysis tool for code quality.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Target directory to scan
    #[arg(short, long, default_value = ".")]
    path: String,

    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    /// Enforce strict mode
    #[arg(short, long)]
    strict: bool,
}

fn main() -> anyhow::Result<()> {
    // 1. Initialize Logger
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");

    // 2. Parse Arguments
    let args = Args::parse();
    let target_path = Path::new(&args.path);

    // 3. Load Configuration
    let config = if args.strict {
        info!("Strict mode enabled.");
        AuditConfig::strict()
    } else {
        AuditConfig::default()
    };

    info!("Starting Audit on: {}", args.path);

    // 4. File Walking
    info!("Scanning filesystem...");
    match analysis::walk_directory(target_path) {
        Ok(files) => {
            info!("Found {} files to analyze.", files.len());
            
            // 5. Analysis
            let smells = analysis::engine::run_analysis(&files, &config);

            // 6. Reporting
            reporting::print_report(&smells);
        }
        Err(e) => {
            error!("Failed to walk directory: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}