use super::languages::AnalysisProvider;
use crate::core::config::LintConfig;
use crate::core::rules::{Smell, SmellCategory};
use std::path::Path;
use tree_sitter::{Node, Parser};

pub struct RustAnalyzer;

impl RustAnalyzer {
    pub fn new() -> Self {
        Self
    }

    fn get_text<'a>(&self, node: Node, source: &'a str) -> &'a str {
        node.utf8_text(source.as_bytes()).unwrap_or("??")
    }

    fn check_bloat(
        &self,
        node: Node,
        config: &LintConfig,
        path: &Path,
        source: &str,
        smells: &mut Vec<Smell>,
    ) {
        let kind = node.kind();

        if kind == "function_item" || kind == "function_definition" {
            let start = node.start_position().row;
            let end = node.end_position().row;
            let length = end.saturating_sub(start);

            if length > config.thresholds.max_function_lines {
                let name = node
                    .child_by_field_name("name")
                    .map(|n| self.get_text(n, source))
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

            if let Some(params_node) = node.child_by_field_name("parameters") {
                let param_count = params_node.named_child_count();
                if param_count > config.thresholds.max_params {
                    let name = node
                        .child_by_field_name("name")
                        .map(|n| self.get_text(n, source))
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
    }

    fn check_complexity(
        &self,
        node: Node,
        _config: &LintConfig,
        path: &Path,
        _source: &str,
        smells: &mut Vec<Smell>,
    ) {
        let kind = node.kind();
        // Check for deep nesting
        if matches!(
            kind,
            "if_expression" | "for_expression" | "match_expression" | "while_expression"
        ) {
            let mut depth = 0;
            let mut ancestor = node.parent();

            while let Some(p) = ancestor {
                let p_kind = p.kind();
                if matches!(
                    p_kind,
                    "if_expression" | "for_expression" | "match_expression" | "while_expression"
                ) {
                    depth += 1;
                }
                ancestor = p.parent();
            }

            if depth > 4 {
                smells.push(Smell::new(
                    path.to_path_buf(),
                    node.start_position().row + 1,
                    SmellCategory::Complexity,
                    "deep_nesting",
                    &format!(
                        "Logic is deeply nested (Depth: {}). Consider refactoring.",
                        depth
                    ),
                ));
            }
        }
    }

    fn check_naming(
        &self,
        node: Node,
        _config: &LintConfig,
        path: &Path,
        source: &str,
        smells: &mut Vec<Smell>,
    ) {
        if node.kind() == "let_declaration" {
            if let Some(pattern) = node.child_by_field_name("pattern") {
                if pattern.kind() == "identifier" {
                    let name = self.get_text(pattern, source);
                    // Ignore common single-letter loop vars and underscores
                    if name.len() < 3
                        && name != "i"
                        && name != "j"
                        && name != "k"
                        && !name.starts_with('_')
                    {
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
    }

    fn check_hygiene(
        &self,
        node: Node,
        _config: &LintConfig,
        path: &Path,
        source: &str,
        smells: &mut Vec<Smell>,
    ) {
        if node.kind() == "line_comment" || node.kind() == "block_comment" {
            let text = self.get_text(node, source);
            if text.contains("TODO") || text.contains("FIXME") {
                smells.push(Smell::new(
                    path.to_path_buf(),
                    node.start_position().row + 1,
                    SmellCategory::Hygiene,
                    "todo_comment",
                    "Found technical debt anchor (TODO/FIXME).",
                ));
            }
        }
    }

    fn traverse(
        &self,
        node: Node,
        config: &LintConfig,
        path: &Path,
        source: &str,
        smells: &mut Vec<Smell>,
    ) {
        self.check_bloat(node, config, path, source, smells);
        self.check_complexity(node, config, path, source, smells);
        self.check_naming(node, config, path, source, smells);
        self.check_hygiene(node, config, path, source, smells);

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.traverse(child, config, path, source, smells);
        }
    }
}

impl AnalysisProvider for RustAnalyzer {
    fn analyze(&self, path: &Path, code: &str, config: &LintConfig) -> Vec<Smell> {
        let mut smells = Vec::new();
        let mut parser = Parser::new();

        let language = tree_sitter_rust::LANGUAGE;
        parser
            .set_language(&language.into())
            .expect("Error loading Rust grammar");

        let tree = match parser.parse(code, None) {
            Some(t) => t,
            None => return vec![],
        };

        self.traverse(tree.root_node(), config, path, code, &mut smells);
        smells
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_short_naming() {
        let code = "fn main() { let x = 5; let valid = 10; }";
        let analyzer = RustAnalyzer::new();
        let config = LintConfig::default();
        let smells = analyzer.analyze(&PathBuf::from("test.rs"), code, &config);

        assert!(smells.iter().any(|s| s.rule_id == "short_variable"));
    }

    #[test]
    fn test_todo_detection() {
        let code = "// TODO: Refactor this later";
        let analyzer = RustAnalyzer::new();
        let config = LintConfig::default();
        let smells = analyzer.analyze(&PathBuf::from("test.rs"), code, &config);

        assert!(smells.iter().any(|s| s.rule_id == "todo_comment"));
    }
}
