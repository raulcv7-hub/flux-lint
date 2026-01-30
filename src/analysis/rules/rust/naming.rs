use crate::analysis::rules::Rule;
use crate::core::config::LintConfig;
use crate::core::rules::{Smell, SmellCategory};
use tree_sitter::Node;
use std::path::Path;

pub struct NamingRule;

impl Rule for NamingRule {
    fn name(&self) -> &str {
        "RustNamingRule"
    }

    fn check(
        &self,
        node: Node,
        source: &str,
        path: &Path,
        _config: &LintConfig,
    ) -> Option<Vec<Smell>> {
        let kind = node.kind();
        let mut smells = Vec::new();

        if kind == "let_declaration" {
            if let Some(pattern) = node.child_by_field_name("pattern") {
                if pattern.kind() == "identifier" {
                    let name = pattern.utf8_text(source.as_bytes()).unwrap_or("?");
                    
                    if name.len() < 3 && !is_allowed_short_name(name) && !name.starts_with('_') {
                         smells.push(Smell::new(
                            path.to_path_buf(),
                            node.start_position().row + 1,
                            SmellCategory::Naming,
                            "short_variable",
                            &format!("Variable '{}' is too short. Use descriptive names.", name),
                        ));
                    }
                }
            }
        }
        
        if smells.is_empty() { None } else { Some(smells) }
    }
}

fn is_allowed_short_name(name: &str) -> bool {
    matches!(name, "i" | "j" | "k" | "x" | "y" | "z" | "id" | "ok" | "tx" | "rx")
}