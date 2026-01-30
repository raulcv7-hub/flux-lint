use crate::analysis::rules::Rule;
use crate::core::config::LintConfig;
use crate::core::rules::{Smell, SmellCategory};
use tree_sitter::Node;
use std::path::Path;

pub struct PythonNamingRule;

impl Rule for PythonNamingRule {
    fn name(&self) -> &str {
        "PythonNamingRule"
    }

    fn check(
        &self,
        node: Node,
        source: &str,
        path: &Path,
        _config: &LintConfig,
    ) -> Option<Vec<Smell>> {
        // x = 1  -> assignment(left: identifier)
        if node.kind() == "assignment" {
            if let Some(left) = node.child_by_field_name("left") {
                if left.kind() == "identifier" {
                    let name = left.utf8_text(source.as_bytes()).unwrap_or("?");
                    
                    // Ignore typical single-letter loop vars or coordinates
                    if name.len() < 3 && !["i", "j", "k", "x", "y", "z", "_"].contains(&name) {
                        return Some(vec![Smell::new(
                            path.to_path_buf(),
                            node.start_position().row + 1,
                            SmellCategory::Naming,
                            "short_variable",
                            &format!("Variable '{}' is too short. Use descriptive names.", name),
                        )]);
                    }
                }
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
        parser.set_language(&tree_sitter_python::LANGUAGE.into()).unwrap();
        parser.parse(code, None).unwrap()
    }

    #[test]
    fn test_short_variable_name() {
        let code = "d = 50"; // 'd' es malo
        let tree = parse(code);
        let assign_node = tree.root_node().child(0).unwrap(); // assignment

        let rule = PythonNamingRule;
        let config = LintConfig::default();

        let smells = rule.check(assign_node, code, &PathBuf::from("test.py"), &config).unwrap();
        assert_eq!(smells[0].rule_id, "short_variable");
    }

    #[test]
    fn test_allowed_short_name() {
        let code = "i = 0"; // 'i' es permitido
        let tree = parse(code);
        let assign_node = tree.root_node().child(0).unwrap();

        let rule = PythonNamingRule;
        let config = LintConfig::default();

        let res = rule.check(assign_node, code, &PathBuf::from("test.py"), &config);
        assert!(res.is_none());
    }
}