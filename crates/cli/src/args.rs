//! CLI argument definitions and mapping to the core enums.

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};
use md2pdf_core::render::{OutputFormat, Paper};
use md2pdf_core::theme::Theme;

#[derive(Parser, Debug)]
#[command(
    name = "md2pdf",
    version,
    about = "Convert Markdown to PDF locally, with no external dependencies.",
    long_about = None,
    args_conflicts_with_subcommands = true,
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,

    /// Convert a Markdown file (default action; `md2pdf file.md`).
    #[command(flatten)]
    pub convert: ConvertArgs,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Convert a Markdown file to PDF (or Typst source).
    Convert(ConvertArgs),
    /// Parse a document and report problems without rendering.
    Validate {
        /// Markdown file to validate.
        input: PathBuf,
    },
    /// Check that the local environment can render documents.
    Doctor,
    /// Scaffold a documentation project in the given directory.
    Init {
        /// Target directory.
        #[arg(default_value = ".")]
        dir: PathBuf,
    },
    /// Inspect and manage themes.
    #[command(subcommand)]
    Theme(ThemeCommand),
}

#[derive(Subcommand, Debug)]
pub enum ThemeCommand {
    /// List the available built-in themes.
    List,
}

#[derive(Args, Debug, Clone)]
pub struct ConvertArgs {
    /// Markdown file to convert.
    pub input: Option<PathBuf>,

    /// Output path (default: input with the format's extension).
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Output format (default: inferred from output extension, else pdf).
    #[arg(long, value_enum)]
    pub format: Option<FormatArg>,

    /// Visual theme.
    #[arg(long, value_enum, default_value_t = ThemeArg::Default)]
    pub theme: ThemeArg,

    /// Paper size.
    #[arg(long, value_enum, default_value_t = PaperArg::A4)]
    pub paper: PaperArg,

    /// Add a table of contents built from the headings.
    #[arg(long)]
    pub toc: bool,

    /// Document title (default: frontmatter title, then first heading).
    #[arg(long)]
    pub title: Option<String>,
}

#[derive(ValueEnum, Debug, Clone, Copy)]
pub enum ThemeArg {
    Default,
    Book,
}

impl From<ThemeArg> for Theme {
    fn from(value: ThemeArg) -> Self {
        match value {
            ThemeArg::Default => Theme::Default,
            ThemeArg::Book => Theme::Book,
        }
    }
}

#[derive(ValueEnum, Debug, Clone, Copy)]
pub enum PaperArg {
    A4,
    Letter,
}

impl From<PaperArg> for Paper {
    fn from(value: PaperArg) -> Self {
        match value {
            PaperArg::A4 => Paper::A4,
            PaperArg::Letter => Paper::Letter,
        }
    }
}

#[derive(ValueEnum, Debug, Clone, Copy)]
pub enum FormatArg {
    Pdf,
    Typst,
}

impl From<FormatArg> for OutputFormat {
    fn from(value: FormatArg) -> Self {
        match value {
            FormatArg::Pdf => OutputFormat::Pdf,
            FormatArg::Typst => OutputFormat::Typst,
        }
    }
}
