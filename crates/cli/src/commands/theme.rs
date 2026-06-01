//! `md2pdf theme` — inspect themes.

use anyhow::Result;
use md2pdf_core::Theme;

pub fn list() -> Result<()> {
    println!("Available themes:\n");
    for theme in Theme::ALL {
        println!("  {:<8}  {}", theme.name(), theme.description());
    }
    println!("\nUse with: md2pdf <file.md> --theme <name>");
    Ok(())
}
