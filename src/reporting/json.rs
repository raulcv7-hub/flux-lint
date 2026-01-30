use crate::core::rules::{Smell, SmellCategory};
use serde::Serialize;
use std::collections::HashMap;

/// Estructura raíz del reporte JSON.
#[derive(Serialize)]
struct JsonReport<'a> {
    summary: Summary,
    smells: &'a [Smell],
}

/// Resumen estadístico para facilitar el parseo en CI.
#[derive(Serialize)]
struct Summary {
    total_smells: usize,
    by_category: HashMap<SmellCategory, usize>,
}

pub fn print_report(smells: &[Smell]) {
    // 1. Calcular Estadísticas
    let mut by_category = HashMap::new();

    // Inicializar a 0 para que siempre aparezcan todas las categorías (opcional, pero limpio)
    by_category.insert(SmellCategory::Complexity, 0);
    by_category.insert(SmellCategory::Bloat, 0);
    by_category.insert(SmellCategory::Hygiene, 0);
    by_category.insert(SmellCategory::Naming, 0);
    by_category.insert(SmellCategory::Design, 0);

    for smell in smells {
        *by_category.entry(smell.category).or_insert(0) += 1;
    }

    let report = JsonReport {
        summary: Summary {
            total_smells: smells.len(),
            by_category,
        },
        smells,
    };

    // 2. Serializar a String
    match serde_json::to_string_pretty(&report) {
        Ok(json) => println!("{}", json),
        Err(e) => eprintln!("Failed to generate JSON report: {}", e),
    }
}
