//! `md2pdf doctor` — verify the local environment can render documents.

use anyhow::Result;
use md2pdf_core::render::OutputFormat;
use md2pdf_core::{Config, Paper, RenderOptions, SecurityPolicy, Theme};

pub fn run() -> Result<()> {
    println!("md2pdf {}", env!("CARGO_PKG_VERSION"));
    println!("engine:   {}", md2pdf_core::ENGINE_VERSION);
    println!(
        "platform: {} / {}",
        std::env::consts::OS,
        std::env::consts::ARCH
    );
    println!("fonts:    embedded (Libertinus Serif, New Computer Modern, DejaVu Sans Mono)");

    let root = std::env::current_dir()?;
    match Config::load_from(&root) {
        Some(Ok(cfg)) => {
            let theme = cfg
                .document
                .as_ref()
                .and_then(|d| d.theme.as_deref())
                .unwrap_or("default");
            let paper = cfg
                .document
                .as_ref()
                .and_then(|d| d.paper.as_deref())
                .unwrap_or("a4");
            println!("config:   md2pdf.toml found (theme: {theme}, paper: {paper})");
        }
        Some(Err(e)) => {
            println!("config:   md2pdf.toml found but invalid: {e}");
        }
        None => {
            println!("config:   no md2pdf.toml (using defaults)");
        }
    }
    let opts = RenderOptions {
        theme: Theme::Default,
        paper: Paper::A4,
        toc: false,
        title: None,
        security: SecurityPolicy::strict(root),
    };

    print!("render:   ");
    match md2pdf_core::convert(
        "# md2pdf doctor\n\nIf you can read this in a PDF, rendering works.",
        &opts,
        OutputFormat::Pdf,
    ) {
        Ok(r) => {
            println!(
                "ok ({} byte PDF produced, embedded fonts loaded)",
                r.bytes.len()
            );
            Ok(())
        }
        Err(e) => {
            println!("FAILED");
            eprintln!("\nrender check failed: {e:?}");
            std::process::exit(1);
        }
    }
}
