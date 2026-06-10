//! CLI argument definitions and mapping to the core enums.

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};
use md2pdf_core::render::{OutputFormat, Paper};

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
    /// Convert a Markdown file to PDF (or Typst, HTML, DOCX).
    Convert(ConvertArgs),
    /// Parse documents and report problems without rendering.
    Validate {
        /// Markdown files to validate.
        #[arg(required = true)]
        inputs: Vec<PathBuf>,

        /// Exit with an error if any warnings are found (for CI gating).
        #[arg(long)]
        strict: bool,
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
    /// Generate shell completions for md2pdf.
    Completions {
        /// Target shell.
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },
    /// Print the man page (troff format) to stdout.
    #[command(hide = true)]
    Man,
}

#[derive(Subcommand, Debug)]
pub enum ThemeCommand {
    /// List the available built-in themes.
    List,
    /// Write a starter custom theme file (<NAME>.toml).
    Create {
        /// Theme name; the file is written as <NAME>.toml.
        name: String,
    },
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

    /// Visual theme: "default", "book", or a path to a custom .toml theme.
    #[arg(long)]
    pub theme: Option<String>,

    /// Paper size (default: "a4").
    #[arg(long, value_enum)]
    pub paper: Option<PaperArg>,

    /// Add a table of contents built from the headings.
    #[arg(long, overrides_with = "no_toc")]
    pub toc: bool,

    /// Disable the table of contents (overrides the config file).
    #[arg(long, overrides_with = "toc")]
    pub no_toc: bool,

    /// Document title (default: frontmatter title, then first heading).
    #[arg(long)]
    pub title: Option<String>,

    /// Running header text ({title}, {author}, {date} placeholders).
    #[arg(long)]
    pub header: Option<String>,

    /// Running footer text ({title}, {author}, {date} placeholders).
    #[arg(long)]
    pub footer: Option<String>,

    /// Show page numbers (default: on).
    #[arg(long, overrides_with = "no_page_numbers")]
    pub page_numbers: bool,

    /// Hide page numbers.
    #[arg(long, overrides_with = "page_numbers")]
    pub no_page_numbers: bool,

    /// Path to an md2pdf.toml config file.
    #[arg(long)]
    pub config: Option<PathBuf>,
}

#[derive(ValueEnum, Debug, Clone, Copy)]
pub enum PaperArg {
    A4,
    A5,
    Letter,
    Legal,
}

impl From<PaperArg> for Paper {
    fn from(value: PaperArg) -> Self {
        match value {
            PaperArg::A4 => Paper::A4,
            PaperArg::A5 => Paper::A5,
            PaperArg::Letter => Paper::Letter,
            PaperArg::Legal => Paper::Legal,
        }
    }
}

#[derive(ValueEnum, Debug, Clone, Copy)]
pub enum FormatArg {
    Pdf,
    Typst,
    Html,
    Docx,
}

impl From<FormatArg> for OutputFormat {
    fn from(value: FormatArg) -> Self {
        match value {
            FormatArg::Pdf => OutputFormat::Pdf,
            FormatArg::Typst => OutputFormat::Typst,
            FormatArg::Html => OutputFormat::Html,
            FormatArg::Docx => OutputFormat::Docx,
        }
    }
}
