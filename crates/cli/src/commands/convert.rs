//! `md2pdf convert` — the default action.

use std::path::Path;

use anyhow::{Context, Result};
use md2pdf_core::render::OutputFormat;
use md2pdf_core::{RenderOptions, SecurityPolicy};

use crate::args::ConvertArgs;

pub fn run(args: ConvertArgs) -> Result<()> {
    let input = args
        .input
        .clone()
        .context("no input file given (try `md2pdf <file.md>` or `md2pdf --help`)")?;

    let root = super::doc_root(&input);
    let security = SecurityPolicy::strict(&root);

    let markdown = super::read_input(&input, &security)?;

    let format = args
        .format
        .map(OutputFormat::from)
        .or_else(|| {
            args.output
                .as_deref()
                .and_then(Path::extension)
                .and_then(|e| OutputFormat::from_extension(&e.to_string_lossy()))
        })
        .unwrap_or(OutputFormat::Pdf);

    let opts = RenderOptions {
        theme: args.theme.into(),
        paper: args.paper.into(),
        toc: args.toc,
        title: args.title.clone(),
        security,
    };

    let rendered = md2pdf_core::convert(&markdown, &opts, format)?;

    let output = args
        .output
        .unwrap_or_else(|| input.with_extension(format.extension()));
    std::fs::write(&output, &rendered.bytes)
        .with_context(|| format!("could not write {}", output.display()))?;

    for d in &rendered.diagnostics {
        eprintln!("warning: {}", d.message);
    }
    eprintln!(
        "wrote {} ({} bytes)",
        output.display(),
        rendered.bytes.len()
    );
    Ok(())
}
