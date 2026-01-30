use crate::core::config::LintConfig;
use crate::core::rules::Smell;
use std::path::Path;
use tree_sitter::Node;

// Sub-módulos para que sean accesibles desde fuera.
pub mod python;
pub mod rust;

/// El contrato que debe cumplir cualquier regla de análisis individual.
/// T se refiere al tipo de lenguaje si quisiéramos ser estrictos,
/// pero tree-sitter usa nodos genéricos, así que simplificamos.
pub trait Rule: Send + Sync {
    /// Nombre único de la regla (para logs o debugging).
    fn name(&self) -> &str;

    /// Método principal. Recibe un nodo y decide si hay un problema.
    /// Retorna un Option para eficiencia (la mayoría de nodos no tienen problemas).
    fn check(
        &self,
        node: Node,
        source: &str,
        path: &Path,
        config: &LintConfig,
    ) -> Option<Vec<Smell>>;
}
