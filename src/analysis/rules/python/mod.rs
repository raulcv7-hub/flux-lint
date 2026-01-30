pub mod bloat;
pub mod complexity;
pub mod hygiene;
pub mod naming;

use crate::analysis::rules::Rule;

/// Devuelve todas las reglas activas para Python.
pub fn get_rules() -> Vec<Box<dyn Rule>> {
    vec![
        Box::new(bloat::PythonBloatRule),
        Box::new(complexity::PythonComplexityRule),
        Box::new(hygiene::PythonHygieneRule),
        Box::new(naming::PythonNamingRule),
    ]
}