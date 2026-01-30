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
        let code = "d = 50";
        let tree = parse(code);
        
        // Estructura: module -> expression_statement -> assignment
        let root = tree.root_node();
        let stmt = root.child(0).unwrap(); 
        
        // Verificamos si es un wrap (expression_statement) y bajamos un nivel
        let assign_node = if stmt.kind() == "expression_statement" {
            stmt.child(0).unwrap()
        } else {
            stmt
        };

        let rule = PythonNamingRule;
        let config = LintConfig::default();

        let smells = rule.check(assign_node, code, &PathBuf::from("test.py"), &config).expect("Should find a smell");
        
        assert_eq!(smells[0].rule_id, "short_variable");
    }

    #[test]
    fn test_allowed_short_name() {
        let code = "i = 0";
        let tree = parse(code);
        
        let root = tree.root_node();
        let stmt = root.child(0).unwrap();
        let assign_node = stmt.child(0).unwrap();

        let rule = PythonNamingRule;
        let config = LintConfig::default();

        let res = rule.check(assign_node, code, &PathBuf::from("test.py"), &config);
        assert!(res.is_none(), "Variable 'i' should be allowed");
    }
}