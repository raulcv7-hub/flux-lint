use crate::core::rules::{Smell, SmellCategory};
use colored::Colorize;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Attribute, Cell, Color, ContentArrangement, Table};

/// Renders the list of smells to stdout.
pub fn print_report(smells: &[Smell]) {
    println!("\n{}", "--- lint REPORT ---".bold().underline());

    if smells.is_empty() {
        println!(
            "\n{}",
            "+++ No smells found. Clean architecture +++".green().bold()
        );
        return;
    }

    // 1. Summary by Category
    let mut complexity = 0;
    let mut bloat = 0;
    let mut hygiene = 0;
    let mut naming = 0;
    let mut design = 0;

    for s in smells {
        match s.category {
            SmellCategory::Complexity => complexity += 1,
            SmellCategory::Bloat => bloat += 1,
            SmellCategory::Hygiene => hygiene += 1,
            SmellCategory::Naming => naming += 1,
            SmellCategory::Design => design += 1,
            _ => {}
        }
    }

    println!("\n📊 Summary:");
    println!("  • Complexity: {}", complexity.to_string().red());
    println!("  • Bloat:      {}", bloat.to_string().yellow());
    println!("  • Hygiene:    {}", hygiene.to_string().blue());
    println!("  • Naming:     {}", naming.to_string().cyan());
    println!("  • Design:     {}", design.to_string().magenta());
    println!("  • Total:      {}\n", smells.len().to_string().bold());

    // 2. Detailed Table
    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_content_arrangement(ContentArrangement::Dynamic);
    table.set_header(vec![
        Cell::new("Category").add_attribute(Attribute::Bold),
        Cell::new("File").add_attribute(Attribute::Bold),
        Cell::new("Line").add_attribute(Attribute::Bold),
        Cell::new("Issue").add_attribute(Attribute::Bold),
    ]);

    for smell in smells {
        let category_color = match smell.category {
            SmellCategory::Complexity => Color::Red,
            SmellCategory::Bloat => Color::Yellow,
            SmellCategory::Hygiene => Color::Blue,
            SmellCategory::Naming => Color::Cyan,
            SmellCategory::Design => Color::Magenta,
            _ => Color::White,
        };

        table.add_row(vec![
            Cell::new(format!("{}", smell.category)).fg(category_color),
            Cell::new(smell.file_path.display()).fg(Color::White),
            Cell::new(smell.line).fg(Color::DarkGrey),
            Cell::new(&smell.message),
        ]);
    }

    println!("{}", table);
}
