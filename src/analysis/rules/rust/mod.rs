pub mod bloat;
pub mod complexity;
pub mod hygiene;
pub mod naming;

use crate::analysis::rules::Rule;

/// Devuelve todas las reglas activas para Rust.
pub fn get_rules() -> Vec<Box<dyn Rule>> {
    vec![
        Box::new(bloat::BloatRule),
        Box::new(naming::NamingRule),
        Box::new(complexity::ComplexityRule),
        Box::new(hygiene::HygieneRule),
    ]
}
