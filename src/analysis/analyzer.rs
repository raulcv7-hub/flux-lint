use super::languages::AnalysisProvider;
use crate::analysis::rules::Rule;
use crate::core::config::LintConfig;
use crate::core::rules::Smell;
use std::path::Path;
use tree_sitter::{Language, Node, Parser};

/// Un analizador genérico capaz de procesar cualquier lenguaje soportado por Tree-sitter
/// siempre que se le suministren las reglas y la gramática correspondientes.
pub struct GenericAnalyzer {
    language: Language,
    rules: Vec<Box<dyn Rule>>,
}

impl GenericAnalyzer {
    pub fn new(language: Language, rules: Vec<Box<dyn Rule>>) -> Self {
        Self { language, rules }
    }

    fn traverse(
        &self,
        node: Node,
        source: &str,
        path: &Path,
        config: &LintConfig,
        smells: &mut Vec<Smell>,
    ) {
        // 1. Ejecutar reglas en el nodo actual
        for rule in &self.rules {
            if let Some(new_smells) = rule.check(node, source, path, config) {
                smells.extend(new_smells);
            }
        }

        // 2. Descenso recursivo
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.traverse(child, source, path, config, smells);
        }
    }
}

impl AnalysisProvider for GenericAnalyzer {
    fn analyze(&self, path: &Path, code: &str, config: &LintConfig) -> Vec<Smell> {
        let mut smells = Vec::new();
        let mut parser = Parser::new();

        parser
            .set_language(&self.language)
            .expect("Error loading grammar in GenericAnalyzer");

        let tree = match parser.parse(code, None) {
            Some(t) => t,
            None => return vec![],
        };

        self.traverse(tree.root_node(), code, path, config, &mut smells);
        smells
    }
}
