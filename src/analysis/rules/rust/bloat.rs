use crate::analysis::rules::Rule;
use crate::core::config::LintConfig;
use crate::core::rules::{Smell, SmellCategory};
use tree_sitter::Node;
use std::path::Path;

pub struct BloatRule;

impl Rule for BloatRule {
    fn name(&self) -> &str {
        "RustBloatRule"
    }

    fn check(
        &self,
        node: Node,
        source: &str,
        path: &Path,
        config: &LintConfig,
    ) -> Option<Vec<Smell>> {
        let kind = node.kind();
        let mut smells = Vec::new();

        if kind == "function_item" || kind == "function_definition" {
            // 1. Long Function
            let start = node.start_position().row;
            let end = node.end_position().row;
            let length = end.saturating_sub(start);

            if length > config.thresholds.max_function_lines {
                let name = node.child_by_field_name("name")
                    .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                    .unwrap_or("anonymous");

                smells.push(Smell::new(
                    path.to_path_buf(),
                    start + 1,
                    SmellCategory::Bloat,
                    "long_function",
                    &format!(
                        "Function '{}' is {} lines long (Limit: {}).",
                        name, length, config.thresholds.max_function_lines
                    ),
                ).with_context(format!("Lines: {}", length)));
            }

            // 2. Parameter Count
            if let Some(params_node) = node.child_by_field_name("parameters") {
                let param_count = params_node.named_child_count();
                if param_count > config.thresholds.max_params {
                    let name = node.child_by_field_name("name")
                        .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                        .unwrap_or("anonymous");

                    smells.push(Smell::new(
                        path.to_path_buf(),
                        start + 1,
                        SmellCategory::Bloat,
                        "too_many_params",
                        &format!(
                            "Function '{}' has {} parameters (Limit: {}).",
                            name, param_count, config.thresholds.max_params
                        ),
                    ).with_context(format!("Params: {}", param_count)));
                }
            }
        }

        if smells.is_empty() {
            None
        } else {
            Some(smells)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tree_sitter::Parser;
    use std::path::PathBuf;

    #[test]
    fn test_long_function() {
        let code = r#"
            fn huge() {
                // 1
                // 2
                // 3
                // 4
            }
        "#;
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_rust::LANGUAGE.into()).unwrap();
        let tree = parser.parse(code, None).unwrap();
        let root = tree.root_node();
        
        // Find the function node manually for the test
        let fn_node = root.child(0).unwrap(); // First child is the function

        let rule = BloatRule;
        let mut config = LintConfig::default();
        config.thresholds.max_function_lines = 3; // Strict

        let res = rule.check(fn_node, code, &PathBuf::from("test.rs"), &config);
        
        assert!(res.is_some());
        assert_eq!(res.unwrap()[0].rule_id, "long_function");
    }
}