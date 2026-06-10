//! Pluggable rendering back-ends.
//!
//! A [`Renderer`] turns a [`Document`] plus [`RenderOptions`] into bytes. The
//! trait is intentionally output-format-agnostic so that PDF, Typst source,
//! HTML and (future) DOCX back-ends are interchangeable. The CLI selects a
//! back-end by [`OutputFormat`] and never reaches into renderer internals.

pub mod docx;
pub mod html;
pub mod typst;

use crate::ir::Document;
use crate::security::SecurityPolicy;
use crate::theme::Theme;

/// Output formats md2pdf can emit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Pdf,
    /// The generated Typst source (useful for debugging and as a power-user
    /// escape hatch into the wider Typst toolchain).
    Typst,
    /// A standalone HTML page with the theme inlined as CSS.
    Html,
    /// An OOXML word-processing document (experimental).
    Docx,
}

impl OutputFormat {
    pub fn from_extension(ext: &str) -> Option<OutputFormat> {
        match ext.to_ascii_lowercase().as_str() {
            "pdf" => Some(OutputFormat::Pdf),
            "typ" | "typst" => Some(OutputFormat::Typst),
            "html" | "htm" => Some(OutputFormat::Html),
            "docx" => Some(OutputFormat::Docx),
            _ => None,
        }
    }

    pub fn extension(self) -> &'static str {
        match self {
            OutputFormat::Pdf => "pdf",
            OutputFormat::Typst => "typ",
            OutputFormat::Html => "html",
            OutputFormat::Docx => "docx",
        }
    }
}

/// Paper sizes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Paper {
    A4,
    A5,
    Letter,
    Legal,
}

impl Paper {
    pub fn typst_name(self) -> &'static str {
        match self {
            Paper::A4 => "a4",
            Paper::A5 => "a5",
            Paper::Letter => "us-letter",
            Paper::Legal => "us-legal",
        }
    }
}

/// Page furniture: running header/footer text and page numbering.
///
/// Header and footer strings may contain `{title}`, `{author}` and `{date}`
/// placeholders, resolved from the document metadata at render time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Layout {
    pub header: Option<String>,
    pub footer: Option<String>,
    pub page_numbers: bool,
}

impl Default for Layout {
    fn default() -> Self {
        Self {
            header: None,
            footer: None,
            page_numbers: true,
        }
    }
}

/// Everything a renderer needs beyond the document itself.
#[derive(Debug, Clone)]
pub struct RenderOptions {
    pub theme: Theme,
    pub paper: Paper,
    pub toc: bool,
    pub title: Option<String>,
    pub layout: Layout,
    pub security: SecurityPolicy,
}

/// A non-fatal note produced while rendering (e.g. a skipped asset).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub message: String,
}

impl Diagnostic {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

/// The output of a render: bytes plus any diagnostics worth surfacing.
pub struct Rendered {
    pub bytes: Vec<u8>,
    pub diagnostics: Vec<Diagnostic>,
}

/// A rendering back-end.
pub trait Renderer {
    /// The format this renderer emits.
    fn format(&self) -> OutputFormat;

    /// Render `doc` to bytes.
    fn render(&self, doc: &Document, opts: &RenderOptions) -> anyhow::Result<Rendered>;
}

/// Construct the default renderer for a given output format.
pub fn for_format(format: OutputFormat) -> Box<dyn Renderer> {
    match format {
        OutputFormat::Pdf => Box::new(typst::TypstPdfRenderer),
        OutputFormat::Typst => Box::new(typst::TypstSourceRenderer),
        OutputFormat::Html => Box::new(html::HtmlRenderer),
        OutputFormat::Docx => Box::new(docx::DocxRenderer),
    }
}
