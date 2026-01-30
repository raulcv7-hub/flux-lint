use anyhow::Result;
use ignore::WalkBuilder;
use std::path::{Path, PathBuf};
use tracing::{debug, warn};

// TODO: In the future, move this to the Configuration module or a Language Registry.
const SUPPORTED_EXTENSIONS: &[&str] = &[
    "rs", // Rust
    "py", // Python
    "js", "jsx", "ts", "tsx",  // JavaScript / TypeScript
    "go",   // Go
    "java", // Java
    "c", "cpp", "h", "hpp", // C/C++
    "cs",  // C#
    "php", // PHP
    "rb",  // Ruby
];

/// Scans a directory recursively and returns a list of relevant files.
pub fn walk_directory(path: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    // Configure the walker
    let walker = WalkBuilder::new(path)
        .standard_filters(true) 
        .hidden(true) 
        .follow_links(false) 
        .build();

    for result in walker {
        match result {
            Ok(entry) => {
                if is_supported_source_file(entry.path()) {
                    debug!("Collected file: {:?}", entry.path());
                    files.push(entry.path().to_path_buf());
                }
            }
            Err(err) => {
                warn!("Skipping entry due to error: {}", err);
            }
        }
    }

    Ok(files)
}

/// Helper puro para validar si un archivo debe ser analizado.
/// Extraído para reducir la complejidad ciclomática y longitud de walk_directory.
fn is_supported_source_file(path: &Path) -> bool {
    // 1. Must be a file
    if !path.is_file() {
        return false;
    }

    // 2. Check Extension
    path.extension()
        .and_then(|s| s.to_str())
        .map(|ext| {
            let ext_lower = ext.to_lowercase();
            SUPPORTED_EXTENSIONS.contains(&ext_lower.as_str())
        })
        .unwrap_or(false)
}