use crate::analysis::rules::Rule;
use crate::core::config::LintConfig;
use crate::core::rules::{Smell, SmellCategory};
use tree_sitter::Node;
use std::path::Path;

pub struct ComplexityRule;

impl Rule for ComplexityRule {
    fn name(&self) -> &str {
        "RustComplexityRule"
    }

    fn check(
        &self,
        node: Node,
        _source: &str,
        path: &Path,
        _config: &LintConfig,
    ) -> Option<Vec<Smell>> {
        let kind = node.kind();
        if matches!(
            kind,
            "if_expression" | "for_expression" | "loop_expression" | "while_expression" | "match_expression"
        ) {
            let mut depth = 0;
            let mut ancestor = node.parent();

            while let Some(p) = ancestor {
                let p_kind = p.kind();
                if matches!(
                    p_kind,
                    "if_expression"
                        | "for_expression"
                        | "loop_expression"
                        | "while_expression"
                        | "match_expression"
                        | "function_item"
                ) {
                    depth += 1;
                }
                ancestor = p.parent();
            }

            if depth > 5 {
                return Some(vec![Smell::new(
                    path.to_path_buf(),
                    node.start_position().row + 1,
                    SmellCategory::Complexity,
                    "deep_nesting",
                    &format!(
                        "Code is deeply nested (Depth: {}). Consider extracting functions.",
                        depth
                    ),
                )]);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tree_sitter::Parser;
    use std::path::PathBuf;

    fn parse(code: &str) -> tree_sitter::Tree {
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_rust::LANGUAGE.into()).unwrap();
        parser.parse(code, None).unwrap()
    }

    #[test]
    fn test_rust_deep_nesting() {
        let code = r#"
            fn main() {                  // 0
                loop {                   // 1
                    if true {            // 2
                        match x {        // 3
                            Some(y) => {
                                for i in y { // 4
                                    if i > 0 { // 5
                                        while true { // 6 (BOOM)
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        "#;
        let tree = parse(code);
        let root = tree.root_node();
        
        let rule = ComplexityRule;
        let config = LintConfig::default();
        let path = PathBuf::from("test.rs");
        
        let mut found = false;
        
        // Simple walker for test
        fn walk(node: Node, rule: &ComplexityRule, code: &str, path: &Path, config: &LintConfig, found: &mut bool) {
            if let Some(_) = rule.check(node, code, path, config) {
                *found = true;
            }
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                walk(child, rule, code, path, config, found);
            }
        }

        walk(root, &rule, code, &path, &config, &mut found);
        assert!(found, "Should detect nested loop inside match inside if inside loop");
    }
}