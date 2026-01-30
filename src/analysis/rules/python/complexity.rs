use crate::analysis::rules::Rule;
use crate::core::config::LintConfig;
use crate::core::rules::{Smell, SmellCategory};
use tree_sitter::Node;
use std::path::Path;

pub struct PythonComplexityRule;

impl Rule for PythonComplexityRule {
    fn name(&self) -> &str {
        "PythonComplexityRule"
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
            "if_statement" | "for_statement" | "while_statement" | "try_statement"
        ) {
            let mut depth = 0;
            let mut ancestor = node.parent();

            while let Some(p) = ancestor {
                let p_kind = p.kind();
                if matches!(
                    p_kind,
                    "if_statement"
                        | "for_statement"
                        | "while_statement"
                        | "try_statement"
                        | "function_definition"
                ) {
                    depth += 1;
                }
                ancestor = p.parent();
            }

            // Python tends to be flatter, so depth > 4 is definitely messy
            if depth > 4 {
                return Some(vec![Smell::new(
                    path.to_path_buf(),
                    node.start_position().row + 1,
                    SmellCategory::Complexity,
                    "deep_nesting",
                    &format!(
                        "Logic is deeply nested (Depth: {}). Flatten your code.",
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
        parser.set_language(&tree_sitter_python::LANGUAGE.into()).unwrap();
        parser.parse(code, None).unwrap()
    }

    #[test]
    fn test_deep_nesting() {
        let code = r#"
def matrix():                  # depth 0 (func)
    for i in range(10):        # depth 1
        if i > 5:              # depth 2
            while True:        # depth 3
                try:           # depth 4
                    if x:      # depth 5 (BOOM)
                        pass
                except:
                    pass
"#;
        let tree = parse(code);
        let root = tree.root_node();
        
        // Necesitamos navegar hasta el 'if x' más profundo.
        // Estructura aproximada: func -> body -> for -> body -> if -> body -> while -> body -> try -> body -> if
        // Para el test, podemos recorrer el árbol y buscar donde salte la regla.
        
        let rule = PythonComplexityRule;
        let config = LintConfig::default();
        let path = PathBuf::from("test.py");
        
        let mut found_smell = false;
        
        // Función auxiliar recursiva para simular el walker
        fn walk(node: Node, rule: &PythonComplexityRule, code: &str, path: &Path, config: &LintConfig, found: &mut bool) {
            if let Some(_) = rule.check(node, code, path, config) {
                *found = true;
            }
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                walk(child, rule, code, path, config, found);
            }
        }

        walk(root, &rule, code, &path, &config, &mut found_smell);
        assert!(found_smell, "Should define deep nesting smell");
    }
}