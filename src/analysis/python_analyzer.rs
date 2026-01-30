use super::languages::AnalysisProvider;
use super::rules::python::get_rules;
use crate::core::config::LintConfig;
use crate::core::rules::Smell;
use std::path::Path;
use tree_sitter::{Parser, Node};

pub struct PythonAnalyzer;

impl PythonAnalyzer {
    pub fn new() -> Self {
        Self
    }

    /// Recorre el AST recursivamente aplicando todas las reglas registradas a cada nodo.
    fn traverse(
        &self,
        node: Node,
        source: &str,
        path: &Path,
        config: &LintConfig,
        smells: &mut Vec<Smell>,
        rules: &[Box<dyn crate::analysis::rules::Rule>]
    ) {
        // 1. Ejecutar reglas en el nodo actual
        for rule in rules {
            if let Some(new_smells) = rule.check(node, source, path, config) {
                smells.extend(new_smells);
            }
        }

        // 2. Descenso recursivo
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.traverse(child, source, path, config, smells, rules);
        }
    }
}

impl AnalysisProvider for PythonAnalyzer {
    fn analyze(&self, path: &Path, code: &str, config: &LintConfig) -> Vec<Smell> {
        let mut smells = Vec::new();
        let mut parser = Parser::new();

        // Carga de gramática Python
        let language = tree_sitter_python::LANGUAGE;
        parser
            .set_language(&language.into())
            .expect("Error loading Python grammar");

        let tree = match parser.parse(code, None) {
            Some(t) => t,
            None => return vec![],
        };

        // Inyección de dependencias: Obtenemos las reglas dinámicamente
        let rules = get_rules();

        self.traverse(tree.root_node(), code, path, config, &mut smells, &rules);
        smells
    }
}