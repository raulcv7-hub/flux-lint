use super::languages::AnalysisProvider;
use super::rules::rust::get_rules;
use crate::core::config::LintConfig;
use crate::core::rules::Smell;
use std::path::Path;
use tree_sitter::{Parser, Node};

pub struct RustAnalyzer;

impl RustAnalyzer {
    pub fn new() -> Self {
        Self
    }

    fn traverse(
        &self, 
        node: Node, 
        source: &str, 
        path: &Path, 
        config: &LintConfig, 
        smells: &mut Vec<Smell>,
        rules: &[Box<dyn crate::analysis::rules::Rule>]
    ) {
        // 1. Visitar el nodo actual con TODAS las reglas
        for rule in rules {
            if let Some(new_smells) = rule.check(node, source, path, config) {
                smells.extend(new_smells);
            }
        }

        // 2. Continuar recursividad
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.traverse(child, source, path, config, smells, rules);
        }
    }
}

impl AnalysisProvider for RustAnalyzer {
    fn analyze(&self, path: &Path, code: &str, config: &LintConfig) -> Vec<Smell> {
        let mut smells = Vec::new();
        let mut parser = Parser::new();
        
        let language = tree_sitter_rust::LANGUAGE;
        parser.set_language(&language.into())
            .expect("Error loading Rust grammar");

        let tree = match parser.parse(code, None) {
            Some(t) => t,
            None => return vec![],
        };

        // Cargamos las reglas dinámicamente
        let rules = get_rules();

        self.traverse(tree.root_node(), code, path, config, &mut smells, &rules);
        smells
    }
}