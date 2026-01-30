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
    fn test_short_var() {
        let code = "fn main() { let d = 10; }";
        let tree = parse(code);
        // let_declaration es hijo del block del function_item
        // Para simplificar, buscamos en el árbol
        let mut cursor = tree.walk();
        let root = tree.root_node();
        
        let mut target_node = None;
        
        // Manual find for 'let d = 10;' which is 'let_declaration'
        // root -> function_item -> block -> let_declaration
        let func = root.child(0).unwrap();
        let block = func.child_by_field_name("body").unwrap();
        for child in block.children(&mut cursor) {
            if child.kind() == "let_declaration" {
                target_node = Some(child);
                break;
            }
        }
        
        let rule = NamingRule;
        let config = LintConfig::default();
        let smells = rule.check(target_node.unwrap(), code, &PathBuf::from("t.rs"), &config).unwrap();
        assert_eq!(smells[0].rule_id, "short_variable");
    }
}