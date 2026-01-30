use super::languages::AnalysisProvider;
use crate::core::config::LintConfig;
use crate::core::rules::{Smell, SmellCategory};
use std::path::Path;
use tree_sitter::{Parser, Node};

pub struct PythonAnalyzer;

impl PythonAnalyzer {
    pub fn new() -> Self {
        Self
    }

    fn get_text<'a>(&self, node: Node, source: &'a str) -> &'a str {
        node.utf8_text(source.as_bytes()).unwrap_or("??")
    }

    /// Rule: Bloat (Function length, Params)
    fn check_bloat(&self, node: Node, config: &LintConfig, path: &Path, source: &str, smells: &mut Vec<Smell>) {
        if node.kind() == "function_definition" {
            // 1. Function Length
            let start = node.start_position().row;
            let end = node.end_position().row;
            let length = end.saturating_sub(start);

            if length > config.thresholds.max_function_lines {
                let name = node.child_by_field_name("name")
                    .map(|n| self.get_text(n, source))
                    .unwrap_or("anonymous");

                smells.push(Smell::new(
                    path.to_path_buf(),
                    start + 1,
                    SmellCategory::Bloat,
                    "long_function",
                    &format!("Function '{}' is {} lines long (Limit: {}).", name, length, config.thresholds.max_function_lines),
                ).with_context(format!("Lines: {}", length)));
            }

            // 2. Parameter Count
            if let Some(params_node) = node.child_by_field_name("parameters") {
                let mut param_count = 0;
                let mut cursor = params_node.walk();
                for child in params_node.children(&mut cursor) {
                    if child.kind() == "identifier" || child.kind() == "typed_parameter" || child.kind() == "default_parameter" {
                        let p_name = self.get_text(child, source);
                        if p_name != "self" && p_name != "cls" {
                            param_count += 1;
                        }
                    }
                }

                if param_count > config.thresholds.max_params {
                     let name = node.child_by_field_name("name")
                        .map(|n| self.get_text(n, source))
                        .unwrap_or("anonymous");

                    smells.push(Smell::new(
                        path.to_path_buf(),
                        start + 1,
                        SmellCategory::Bloat,
                        "too_many_params",
                        &format!("Function '{}' has {} parameters (Limit: {}).", name, param_count, config.thresholds.max_params),
                    ).with_context(format!("Params: {}", param_count)));
                }
            }
        }
    }

    /// Rule: Complexity (Deep Nesting)
    fn check_complexity(&self, node: Node, _config: &LintConfig, path: &Path, _source: &str, smells: &mut Vec<Smell>) {
        let kind = node.kind();
        // Python logical structures
        if matches!(kind, "if_statement" | "for_statement" | "while_statement" | "try_statement") {
            let mut depth = 0;
            let mut ancestor = node.parent();
            
            while let Some(p) = ancestor {
                let p_kind = p.kind();
                if matches!(p_kind, "if_statement" | "for_statement" | "while_statement" | "try_statement" | "function_definition") {
                    depth += 1;
                }
                ancestor = p.parent();
            }

            // Python tends to be flatter, so depth > 4 is definitely messy
            if depth > 4 {
                smells.push(Smell::new(
                    path.to_path_buf(),
                    node.start_position().row + 1,
                    SmellCategory::Complexity,
                    "deep_nesting",
                    &format!("Logic is deeply nested (Depth: {}). Flatten your code.", depth),
                ));
            }
        }
    }

    /// Rule: Naming (Short Variables)
    fn check_naming(&self, node: Node, _config: &LintConfig, path: &Path, source: &str, smells: &mut Vec<Smell>) {
        // x = 1  -> assignment(left: identifier)
        if node.kind() == "assignment" {
            if let Some(left) = node.child_by_field_name("left") {
                if left.kind() == "identifier" {
                    let name = self.get_text(left, source);
                    // Ignore typical single-letter loop vars or coordinates
                    if name.len() < 3 && !["i", "j", "k", "x", "y", "z", "_"].contains(&name) {
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

    /// Rule: Hygiene (TODOs)
    fn check_hygiene(&self, node: Node, _config: &LintConfig, path: &Path, source: &str, smells: &mut Vec<Smell>) {
        if node.kind() == "comment" {
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

    fn traverse(&self, node: Node, config: &LintConfig, path: &Path, source: &str, smells: &mut Vec<Smell>) {
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

impl AnalysisProvider for PythonAnalyzer {
    fn analyze(&self, path: &Path, code: &str, config: &LintConfig) -> Vec<Smell> {
        let mut smells = Vec::new();
        let mut parser = Parser::new();
        
        // Carga dinámica del lenguaje Python v0.23
        let language = tree_sitter_python::LANGUAGE;
        parser.set_language(&language.into())
            .expect("Error loading Python grammar");

        let tree = match parser.parse(code, None) {
            Some(t) => t,
            None => return vec![],
        };

        self.traverse(tree.root_node(), config, path, code, &mut smells);
        smells
    }
}