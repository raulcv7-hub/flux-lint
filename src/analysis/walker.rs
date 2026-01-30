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
///
/// Features:
/// - Respects `.gitignore` and `.ignore` files.
/// - Skips hidden files/directories (like `.git/`).
/// - Filters by supported source code extensions.
pub fn walk_directory(path: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    // Configure the walker
    let walker = WalkBuilder::new(path)
        .standard_filters(true) // Enables .gitignore, .ignore, global ignores, etc.
        .hidden(true) // Skip hidden files
        .follow_links(false) // Security: Don't follow symlinks to avoid cycles/external paths
        .build();

    for result in walker {
        match result {
            Ok(entry) => {
                let path = entry.path();

                // 1. Must be a file (not a directory or symlink pointing to nowhere)
                if !path.is_file() {
                    continue;
                }

                // 2. Check Extension
                if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                    let ext_lower = ext.to_lowercase();
                    if SUPPORTED_EXTENSIONS.contains(&ext_lower.as_str()) {
                        debug!("Collected file: {:?}", path);
                        files.push(path.to_path_buf());
                    }
                }
            }
            Err(err) => {
                // Log permission errors or IO errors but do not crash the pipeline
                warn!("Skipping entry due to error: {}", err);
            }
        }
    }

    Ok(files)
}
