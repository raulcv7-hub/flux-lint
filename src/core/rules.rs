use serde::Serialize;
use std::fmt;
use std::path::PathBuf;

/// Categorías de olores de código alineadas con los requisitos.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum SmellCategory {
    Complexity, // Salud Mental (Bucles anidados, ceguera booleana)
    Bloat,      // Obesidad (Métodos largos, archivos gigantes)
    Hygiene,    // Higiene (Bloques vacíos, TODOs)
    Design,     // Diseño/OOP (Acoplamiento)
    Naming,     // Semántica (Nombres cortos o vagos)
    Unknown,
}

/// Representa una violación específica encontrada en el código.
#[derive(Debug, Clone, Serialize)]
pub struct Smell {
    /// Ruta del archivo donde se encontró.
    pub file_path: PathBuf,
    /// Número de línea (base 1).
    pub line: usize,
    /// Categoría del problema.
    pub category: SmellCategory,
    /// Identificador corto de la regla (ej: "long_function").
    pub rule_id: String,
    /// Explicación legible para humanos.
    pub message: String,
    /// Contexto extra (ej: "Found 8 args, limit is 4").
    pub context: Option<String>,
}

impl Smell {
    /// Constructor principal.
    pub fn new(
        file_path: PathBuf,
        line: usize,
        category: SmellCategory,
        rule_id: &str,
        message: &str,
    ) -> Self {
        Self {
            file_path,
            line,
            category,
            rule_id: rule_id.to_string(),
            message: message.to_string(),
            context: None,
        }
    }

    /// Builder pattern para añadir contexto adicional.
    pub fn with_context(mut self, context: String) -> Self {
        self.context = Some(context);
        self
    }
}

/// Implementación de Display para mostrar las categorías bonitas en consola.
impl fmt::Display for SmellCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            SmellCategory::Complexity => "COMPLEXITY",
            SmellCategory::Bloat => "BLOAT",
            SmellCategory::Hygiene => "HYGIENE",
            SmellCategory::Design => "DESIGN",
            SmellCategory::Naming => "NAMING",
            SmellCategory::Unknown => "UNKNOWN",
        };
        write!(f, "{}", s)
    }
}
