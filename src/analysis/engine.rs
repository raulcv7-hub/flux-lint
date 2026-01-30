use super::languages::get_analyzer;
use crate::core::config::LintConfig;
use crate::core::rules::Smell;
use rayon::prelude::*;
use std::fs;
use std::path::PathBuf;
use tracing::{debug, warn};

/// Orchestrates the analysis process in parallel.
pub fn run_analysis(files: &[PathBuf], config: &LintConfig) -> Vec<Smell> {
    files
        .par_iter()
        .flat_map(|path| {
            // 1. Identify Language
            let ext = path
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_lowercase();

            // 2. Get Analyzer
            if let Some(analyzer) = get_analyzer(&ext) {
                // 3. Read File (IO)
                match fs::read_to_string(path) {
                    Ok(code) => {
                        debug!("Analyzing: {:?}", path);
                        // 4. Analyze
                        analyzer.analyze(path, &code, config)
                    }
                    Err(e) => {
                        warn!("Could not read file {:?}: {}", path, e);
                        vec![]
                    }
                }
            } else {
                vec![]
            }
        })
        .collect() // Gather all smells from all threads into a single Vec
}
