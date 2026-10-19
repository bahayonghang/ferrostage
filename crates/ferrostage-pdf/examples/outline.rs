//! Read-only metadata probe: cargo run -p ferrostage-pdf --example outline -- path.pdf
use std::{env, error::Error, fs};

fn main() -> Result<(), Box<dyn Error>> {
    let path = env::args_os().nth(1).ok_or("usage: outline path.pdf")?;
    let structure = ferrostage_pdf::extract_structure(&fs::read(path)?)?;
    eprintln!(
        "{} pages, {} outline entries, partial={}",
        structure.pages.len(),
        structure.outline.entries.len(),
        structure.outline.partial
    );
    for entry in structure.outline.entries {
        println!(
            "{}\t{}\t{}\t{}\t{}\t{:?}",
            entry.id,
            entry
                .parent
                .map_or_else(String::new, |parent| parent.to_string()),
            entry.depth,
            entry
                .page
                .map_or_else(String::new, |page| page.0.to_string()),
            entry
                .title
                .unwrap_or_default()
                .replace(['\t', '\r', '\n'], " "),
            entry.issue,
        );
    }
    Ok(())
}
