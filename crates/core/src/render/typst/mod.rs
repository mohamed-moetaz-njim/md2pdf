//! The Typst back-end: lowers the [`Document`] IR to Typst markup and either
//! returns that markup ([`TypstSourceRenderer`]) or compiles it to a PDF
//! ([`TypstPdfRenderer`]). Both share the [`lower`] function, so PDF and Typst
//! output can never drift apart.

mod lower;

use std::path::PathBuf;

use typst::layout::PagedDocument;
use typst_as_lib::TypstEngine;
use typst_as_lib::typst_kit_options::TypstKitFontOptions;

use crate::ir::Document;
use crate::render::{Diagnostic, OutputFormat, RenderOptions, Rendered, Renderer};

pub use lower::lower;

/// Renders a [`Document`] to a PDF, fully in-process with embedded fonts.
pub struct TypstPdfRenderer;

impl Renderer for TypstPdfRenderer {
    fn format(&self) -> OutputFormat {
        OutputFormat::Pdf
    }

    fn render(&self, doc: &Document, opts: &RenderOptions) -> anyhow::Result<Rendered> {
        let mut diagnostics: Vec<Diagnostic> = Vec::new();
        let source = lower(doc, opts, &mut diagnostics);

        let root: PathBuf = opts.security.root.clone();
        let engine = TypstEngine::builder()
            .main_file(source)
            .with_file_system_resolver(root)
            .search_fonts_with(
                TypstKitFontOptions::default()
                    .include_system_fonts(false)
                    .include_embedded_fonts(true),
            )
            .build();

        let document: PagedDocument = engine
            .compile()
            .output
            .map_err(|e| anyhow::anyhow!("typst compilation failed: {e}"))?;

        let bytes = typst_pdf::pdf(&document, &typst_pdf::PdfOptions::default())
            .map_err(|errors| anyhow::anyhow!("could not generate PDF: {errors:?}"))?;

        Ok(Rendered { bytes, diagnostics })
    }
}

/// Renders a [`Document`] to Typst source (`.typ`).
pub struct TypstSourceRenderer;

impl Renderer for TypstSourceRenderer {
    fn format(&self) -> OutputFormat {
        OutputFormat::Typst
    }

    fn render(&self, doc: &Document, opts: &RenderOptions) -> anyhow::Result<Rendered> {
        let mut diagnostics: Vec<Diagnostic> = Vec::new();
        let source = lower(doc, opts, &mut diagnostics);
        Ok(Rendered {
            bytes: source.into_bytes(),
            diagnostics,
        })
    }
}
