use crate::analysis::rules::Rule;
use crate::core::config::LintConfig;
use crate::core::rules::{Smell, SmellCategory};
use tree_sitter::Node;
use std::path::Path;

pub struct PythonBloatRule;

impl Rule for PythonBloatRule {
    fn name(&self) -> &str {
        "PythonBloatRule"
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

        if kind == "function_definition" {
            // 1. Function Length
            let start = node.start_position().row;
            let end = node.end_position().row;
            let length = end.saturating_sub(start);

            if length > config.thresholds.max_function_lines {
                let name = node
                    .child_by_field_name("name")
                    .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                    .unwrap_or("anonymous");

                smells.push(
                    Smell::new(
                        path.to_path_buf(),
                        start + 1,
                        SmellCategory::Bloat,
                        "long_function",
                        &format!(
                            "Function '{}' is {} lines long (Limit: {}).",
                            name, length, config.thresholds.max_function_lines
                        ),
                    )
                    .with_context(format!("Lines: {}", length)),
                );
            }

            // 2. Parameter Count
            if let Some(params_node) = node.child_by_field_name("parameters") {
                let mut param_count = 0;
                let mut cursor = params_node.walk();
                for child in params_node.children(&mut cursor) {
                    let k = child.kind();
                    if k == "identifier" || k == "typed_parameter" || k == "default_parameter" {
                        let p_name = child.utf8_text(source.as_bytes()).unwrap_or("?");
                        if p_name != "self" && p_name != "cls" {
                            param_count += 1;
                        }
                    }
                }

                if param_count > config.thresholds.max_params {
                    let name = node
                        .child_by_field_name("name")
                        .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                        .unwrap_or("anonymous");

                    smells.push(
                        Smell::new(
                            path.to_path_buf(),
                            start + 1,
                            SmellCategory::Bloat,
                            "too_many_params",
                            &format!(
                                "Function '{}' has {} parameters (Limit: {}).",
                                name, param_count, config.thresholds.max_params
                            ),
                        )
                        .with_context(format!("Params: {}", param_count)),
                    );
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

    fn parse(code: &str) -> tree_sitter::Tree {
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_python::LANGUAGE.into()).unwrap();
        parser.parse(code, None).unwrap()
    }

    #[test]
    fn test_long_function() {
        let code = r#"
def big_func():
    print(1)
    print(2)
    print(3)
    print(4)
    print(5)
"#;
        let tree = parse(code);
        // Buscamos el nodo 'function_definition' (es el primer hijo del root 'module')
        let fn_node = tree.root_node().child(0).unwrap();
        
        let rule = PythonBloatRule;
        let mut config = LintConfig::default();
        config.thresholds.max_function_lines = 3; // Umbral estricto para test

        let smells = rule.check(fn_node, code, &PathBuf::from("test.py"), &config).unwrap();
        assert_eq!(smells.len(), 1);
        assert_eq!(smells[0].rule_id, "long_function");
    }

    #[test]
    fn test_too_many_params() {
        // 'self' no debería contar, así que tenemos a,b,c,d,e (5 params)
        let code = "def complex_func(self, a, b, c, d, e): pass";
        let tree = parse(code);
        let fn_node = tree.root_node().child(0).unwrap();

        let rule = PythonBloatRule;
        let mut config = LintConfig::default();
        config.thresholds.max_params = 4;

        let smells = rule.check(fn_node, code, &PathBuf::from("test.py"), &config).unwrap();
        assert_eq!(smells.len(), 1);
        assert_eq!(smells[0].rule_id, "too_many_params");
        assert!(smells[0].context.as_ref().unwrap().contains("Params: 5"));
    }
}