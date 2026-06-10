//! `md2pdf theme` — inspect and scaffold themes.

use anyhow::Result;
use md2pdf_core::Theme;
use md2pdf_core::theme::ThemeSpec;

pub fn list() -> Result<()> {
    println!("Available themes:\n");
    for theme in Theme::BUILTIN {
        println!("  {:<8}  {}", theme.name(), theme.description());
    }
    println!("\nUse with: md2pdf <file.md> --theme <name|file.toml>");
    println!("Create a custom theme with: md2pdf theme create <name>");
    Ok(())
}

pub fn create(name: &str) -> Result<()> {
    let path = std::path::PathBuf::from(format!("{name}.toml"));
    if path.exists() {
        anyhow::bail!("{} already exists", path.display());
    }
    std::fs::write(&path, ThemeSpec::sample())?;
    println!("wrote {}", path.display());
    println!("Use with: md2pdf <file.md> --theme {}", path.display());
    Ok(())
}
