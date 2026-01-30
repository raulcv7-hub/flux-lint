use crate::core::config::AuditConfig;
use crate::core::rules::Smell;
use std::path::Path;

/// Trait that every language analyzer must implement.
pub trait AnalysisProvider: Send + Sync {
    /// Analyzes the source code and returns a list of smells.
    fn analyze(&self, path: &Path, code: &str, config: &AuditConfig) -> Vec<Smell>;
}

/// Factory to get the correct analyzer based on file extension.
pub fn get_analyzer(extension: &str) -> Option<Box<dyn AnalysisProvider>> {
    match extension {
        "rs" => Some(Box::new(super::rust_analyzer::RustAnalyzer::new())),
        _ => None,
    }
}
