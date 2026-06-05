//! `md2pdf convert` — the default action.

use std::path::Path;

use anyhow::{Context, Result};
use md2pdf_core::render::OutputFormat;
use md2pdf_core::{Config, Paper, RenderOptions, SecurityPolicy, Theme};

use crate::args::ConvertArgs;

pub fn run(args: ConvertArgs) -> Result<()> {
    let input = args
        .input
        .clone()
        .context("no input file given (try `md2pdf <file.md>` or `md2pdf --help`)")?;

    let root = super::doc_root(&input);

    // Layer 1: built-in defaults.
    let mut security = SecurityPolicy::strict(&root);
    let mut opts = RenderOptions {
        theme: Theme::Default,
        paper: Paper::A4,
        toc: false,
        title: None,
        security: security.clone(),
    };

    // Layer 2: config file (if present).
    let config = match &args.config {
        Some(path) => Some(Config::load(path)?),
        None => Config::load_from(&root).transpose()?,
    };
    if let Some(ref cfg) = config {
        cfg.apply_to_security(&mut security);
        cfg.apply_to_render_options(&mut opts);
        opts.security = security.clone();
        cfg.validate().unwrap_or_else(|e| {
            eprintln!("warning: config: {e}");
        });
    }

    // Layer 3: CLI flags (highest priority).
    if let Some(theme) = args.theme {
        opts.theme = theme.into();
    }
    if let Some(paper) = args.paper {
        opts.paper = paper.into();
    }

    // Apply remaining CLI overrides (always present when set).
    opts.toc = args.toc;
    if args.title.is_some() {
        opts.title = args.title.clone();
    }

    let markdown = super::read_input(&input, &security)?;
    opts.security = security;

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
