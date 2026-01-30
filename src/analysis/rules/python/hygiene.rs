use crate::analysis::rules::Rule;
use crate::core::config::LintConfig;
use crate::core::rules::{Smell, SmellCategory};
use std::path::Path;
use tree_sitter::Node;

pub struct PythonHygieneRule;

impl Rule for PythonHygieneRule {
    fn name(&self) -> &str {
        "PythonHygieneRule"
    }

    fn check(
        &self,
        node: Node,
        source: &str,
        path: &Path,
        _config: &LintConfig,
    ) -> Option<Vec<Smell>> {
        if node.kind() == "comment" {
            let text = node.utf8_text(source.as_bytes()).unwrap_or("");
            if text.contains("TODO") || text.contains("FIXME") {
                return Some(vec![Smell::new(
                    path.to_path_buf(),
                    node.start_position().row + 1,
                    SmellCategory::Hygiene,
                    "todo_comment",
                    "Found technical debt anchor (TODO/FIXME).",
                )]);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tree_sitter::Parser;

    fn parse(code: &str) -> tree_sitter::Tree {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_python::LANGUAGE.into())
            .unwrap();
        parser.parse(code, None).unwrap()
    }

    #[test]
    fn test_todo_detection() {
        let code = "# TODO: Fix this later";
        let tree = parse(code);
        let comment_node = tree.root_node().child(0).unwrap();

        let rule = PythonHygieneRule;
        let config = LintConfig::default();

        let smells = rule
            .check(comment_node, code, &PathBuf::from("test.py"), &config)
            .unwrap();
        assert_eq!(smells[0].rule_id, "todo_comment");
    }
}
