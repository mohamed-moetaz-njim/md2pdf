//! md2pdf — convert Markdown to PDF locally with no external runtime
//! dependencies. Markdown is parsed with comrak, lowered to Typst markup and
//! compiled to a PDF in-process, using fonts embedded in the binary.

mod convert;
mod theme;

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;
use comrak::{Arena, Options, parse_document};
use typst::layout::PagedDocument;
use typst_as_lib::TypstEngine;
use typst_as_lib::typst_kit_options::TypstKitFontOptions;

use theme::{Paper, Theme};

#[derive(Parser, Debug)]
#[command(
    name = "md2pdf",
    version,
    about = "Convert Markdown to a PDF locally, with no external dependencies.",
    long_about = None,
)]
struct Cli {
    /// Markdown file to convert.
    input: PathBuf,

    /// Output PDF path (defaults to the input file with a .pdf extension).
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Visual theme.
    #[arg(long, value_enum, default_value_t = Theme::Default)]
    theme: Theme,

    /// Paper size.
    #[arg(long, value_enum, default_value_t = Paper::A4)]
    paper: Paper,

    /// Add a table of contents built from the headings.
    #[arg(long)]
    toc: bool,

    /// Document title (defaults to the first heading, then the file name).
    #[arg(long)]
    title: Option<String>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let markdown = std::fs::read_to_string(&cli.input)
        .with_context(|| format!("could not read {}", cli.input.display()))?;

    let mut options = Options::default();
    options.extension.table = true;
    options.extension.strikethrough = true;
    options.extension.tasklist = true;
    options.extension.footnotes = true;
    options.extension.autolink = true;
    options.extension.superscript = true;

    let arena = Arena::new();
    let root = parse_document(&arena, &markdown, &options);

    let title = cli
        .title
        .clone()
        .or_else(|| convert::first_heading_text(root))
        .unwrap_or_else(|| {
            cli.input
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_default()
        });

    let body = convert::render_body(root);
    let source = theme::assemble(&title, &body, cli.theme, cli.paper, cli.toc);

    // Resolve image paths relative to the Markdown file's directory.
    let root_dir = cli
        .input
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));

    let engine = TypstEngine::builder()
        .main_file(source)
        .with_file_system_resolver(root_dir)
        .search_fonts_with(
            TypstKitFontOptions::default()
                .include_system_fonts(false)
                .include_embedded_fonts(true),
        )
        .build();

    let doc: PagedDocument = engine
        .compile()
        .output
        .map_err(|e| anyhow::anyhow!("typst compilation failed: {e}"))?;

    let pdf = typst_pdf::pdf(&doc, &typst_pdf::PdfOptions::default())
        .map_err(|errors| anyhow::anyhow!("could not generate PDF: {errors:?}"))?;

    let output = cli
        .output
        .unwrap_or_else(|| cli.input.with_extension("pdf"));
    std::fs::write(&output, pdf)
        .with_context(|| format!("could not write {}", output.display()))?;

    eprintln!("wrote {}", output.display());
    Ok(())
}
