//! Typst preambles for the available visual themes plus document assembly.
//!
//! `assemble` stitches a theme preamble, an optional title block, an optional
//! table of contents and the rendered body into a single Typst source string
//! that the engine compiles in one pass.

use std::fmt::Write;

use clap::ValueEnum;

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum Theme {
    /// Clean sans-serif headings with blue accents (default).
    Default,
    /// Classic book look set in New Computer Modern.
    Book,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum Paper {
    A4,
    Letter,
}

impl Paper {
    fn typst_name(self) -> &'static str {
        match self {
            Paper::A4 => "a4",
            Paper::Letter => "us-letter",
        }
    }
}

/// Build the full Typst source for a document.
pub fn assemble(title: &str, body: &str, theme: Theme, paper: Paper, toc: bool) -> String {
    let mut s = String::new();
    s.push_str(&preamble(theme, paper));

    if !title.trim().is_empty() {
        title_block(&mut s, title, theme);
    }

    if toc {
        s.push_str("#outline(title: [Contents], indent: auto)\n");
        s.push_str("#v(1.2em)\n\n");
    }

    s.push_str(body);
    s
}

fn preamble(theme: Theme, paper: Paper) -> String {
    let mut s = String::new();
    let paper = paper.typst_name();
    match theme {
        Theme::Default => {
            let _ = write!(
                s,
                "#set page(paper: \"{paper}\", margin: (x: 2.2cm, y: 2.4cm), numbering: \"1\")\n"
            );
            s.push_str("#set text(font: \"Libertinus Serif\", size: 11pt, lang: \"en\")\n");
            s.push_str("#set par(justify: true, leading: 0.62em)\n");
            s.push_str("#show heading: set text(fill: rgb(\"#13315c\"))\n");
            s.push_str("#show heading.where(level: 1): set text(size: 1.5em)\n");
            s.push_str("#show link: set text(fill: rgb(\"#1565c0\"))\n");
            s.push_str("#show raw: set text(font: \"DejaVu Sans Mono\", size: 9.5pt)\n");
            s.push_str(
                "#show raw.where(block: true): block.with(fill: rgb(\"#f4f6f8\"), inset: 10pt, radius: 4pt, width: 100%, stroke: 0.5pt + rgb(\"#e1e4e8\"))\n",
            );
            s.push_str(
                "#show quote.where(block: true): set block(stroke: (left: 2pt + rgb(\"#1565c0\")), inset: (left: 1em, y: 0.4em))\n",
            );
        }
        Theme::Book => {
            let _ = write!(
                s,
                "#set page(paper: \"{paper}\", margin: (x: 2.6cm, y: 2.8cm), numbering: \"1\")\n"
            );
            s.push_str("#set text(font: \"New Computer Modern\", size: 11pt, lang: \"en\")\n");
            s.push_str("#set par(justify: true, leading: 0.7em, first-line-indent: 1.2em)\n");
            s.push_str("#show heading: set text(fill: rgb(\"#5a1e1e\"))\n");
            s.push_str("#show heading.where(level: 1): set text(size: 1.6em)\n");
            s.push_str("#show link: set text(fill: rgb(\"#8a2b2b\"))\n");
            s.push_str("#show raw: set text(font: \"DejaVu Sans Mono\", size: 9.5pt)\n");
            s.push_str(
                "#show raw.where(block: true): block.with(fill: luma(245), inset: 10pt, radius: 2pt, width: 100%)\n",
            );
            s.push_str(
                "#show quote.where(block: true): set block(stroke: (left: 2pt + luma(160)), inset: (left: 1em, y: 0.4em))\n",
            );
        }
    }
    s.push('\n');
    s
}

fn title_block(s: &mut String, title: &str, theme: Theme) {
    let accent = match theme {
        Theme::Default => "#13315c",
        Theme::Book => "#5a1e1e",
    };
    s.push_str("#align(center)[\n");
    let _ = write!(
        s,
        "  #text(size: 22pt, weight: \"bold\", fill: rgb(\"{accent}\"))[#\"{}\"]\n",
        esc_str(title)
    );
    s.push_str("  #v(0.3em)\n");
    let _ = write!(s, "  #line(length: 38%, stroke: 0.6pt + rgb(\"{accent}\"))\n");
    s.push_str("]\n");
    s.push_str("#v(1.4em)\n\n");
}

/// Escape a string for a Typst `"..."` literal (mirrors `convert::esc`).
fn esc_str(s: &str) -> String {
    let mut o = String::with_capacity(s.len() + 2);
    for c in s.chars() {
        match c {
            '\\' => o.push_str("\\\\"),
            '"' => o.push_str("\\\""),
            '\n' => o.push_str("\\n"),
            '\r' => o.push_str("\\r"),
            '\t' => o.push_str("\\t"),
            _ => o.push(c),
        }
    }
    o
}
