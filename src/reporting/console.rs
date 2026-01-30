use crate::core::rules::Smell;

/// Renders the list of smells to stdout using nice tables/colors.
pub fn print_report(smells: &[Smell]) {
    if smells.is_empty() {
        println!("No smells found! Great job.");
    } else {
        println!("Found {} smells.", smells.len());
        // TODO: Implement ComfyTable rendering
    }
}