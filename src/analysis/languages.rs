use crate::core::config::LintConfig;
use crate::core::rules::Smell;
use std::path::Path;
use super::analyzer::GenericAnalyzer;

/// Trait that every language analyzer must implement.
pub trait AnalysisProvider: Send + Sync {
    fn analyze(&self, path: &Path, code: &str, config: &LintConfig) -> Vec<Smell>;
}

/// Factory to get the correct analyzer based on file extension.
pub fn get_analyzer(extension: &str) -> Option<Box<dyn AnalysisProvider>> {
    match extension {
        "rs" => {
            let lang = tree_sitter_rust::LANGUAGE;
            let rules = super::rules::rust::get_rules();
            Some(Box::new(GenericAnalyzer::new(lang.into(), rules)))
        },
        "py" => {
            let lang = tree_sitter_python::LANGUAGE;
            let rules = super::rules::python::get_rules();
            Some(Box::new(GenericAnalyzer::new(lang.into(), rules)))
        },
        _ => None,
    }
}