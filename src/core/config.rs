use serde::{Deserialize, Serialize};

/// Global configuration for the lint tool.
/// Defines thresholds and tolerances for the analysis engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LintConfig {
    pub thresholds: Thresholds,
    // Future: exclude_paths, active_rules, etc.
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thresholds {
    /// Max allowed complexity (Cognitive or Cyclomatic).
    pub max_complexity: u32,
    /// Max lines of code per function.
    pub max_function_lines: usize,
    /// Max parameters per function.
    pub max_params: usize,
    /// Max lines per file.
    pub max_file_lines: usize,
    /// Max files in a single directory (flat).
    pub max_dir_files: usize,
}

impl Default for LintConfig {
    fn default() -> Self {
        Self {
            thresholds: Thresholds::default(),
        }
    }
}

impl Default for Thresholds {
    /// Provides standard, sane defaults for a strict but fair analysis.
    fn default() -> Self {
        Self {
            max_complexity: 10,     // Sonar default is often 15, we aim for 10.
            max_function_lines: 40, // Fits on a standard monitor.
            max_params: 4,          // More than 4 is usually a Data Clump.
            max_file_lines: 300,    // Single Responsibility Principle limit.
            max_dir_files: 20,      // Prevents "God Packages".
        }
    }
}

impl LintConfig {
    /// Returns a strict profile for high-reliability systems.
    pub fn strict() -> Self {
        Self {
            thresholds: Thresholds {
                max_complexity: 5,
                max_function_lines: 25,
                max_params: 3,
                max_file_lines: 200,
                max_dir_files: 10,
            },
        }
    }

    /// Returns a lenient profile for legacy codebases.
    pub fn lenient() -> Self {
        Self {
            thresholds: Thresholds {
                max_complexity: 20,
                max_function_lines: 100,
                max_params: 8,
                max_file_lines: 500,
                max_dir_files: 50,
            },
        }
    }
}
