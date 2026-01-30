use clap::Parser;
use std::path::Path;
use tracing::{info, error, Level};
use tracing_subscriber::FmtSubscriber;

mod core;
mod analysis;
mod reporting;

use core::config::AuditConfig;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Target directory to scan
    #[arg(short, long, default_value = ".")]
    path: String,

    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    /// Enforce strict mode (lower tolerance)
    #[arg(short, long)]
    strict: bool,

    /// Output results in JSON format (ideal for CI/CD)
    #[arg(long)]
    json: bool,
}

fn main() -> anyhow::Result<()> {
    // 1. Initialize Logger
    let args = Args::parse();

    let log_level = if args.json { Level::WARN } else { Level::INFO };
    
    let subscriber = FmtSubscriber::builder()
        .with_max_level(log_level)
        .with_writer(std::io::stderr)
        .finish();
        
    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");

    let target_path = Path::new(&args.path);

    // 2. Load Config
    let config = if args.strict {
        AuditConfig::strict()
    } else {
        AuditConfig::default()
    };

    if !args.json {
        info!("Starting Audit on: {}", args.path);
        info!("Scanning filesystem...");
    }

    // 3. Execution
    match analysis::walk_directory(target_path) {
        Ok(files) => {
            if !args.json {
                info!("Found {} files to analyze.", files.len());
            }
            
            let smells = analysis::engine::run_analysis(&files, &config);

            // 4. Reporting Strategy
            if args.json {
                reporting::json::print_report(&smells);
            } else {
                reporting::console::print_report(&smells);
            }
        }
        Err(e) => {
            error!("Failed to walk directory: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}