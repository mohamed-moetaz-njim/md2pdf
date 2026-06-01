//! `md2pdf validate` — parse and lint a document without rendering a PDF.

use std::path::Path;

use anyhow::Result;
use md2pdf_core::ir::{Block, Inline};
use md2pdf_core::render::OutputFormat;
use md2pdf_core::{Document, Paper, RenderOptions, SecurityPolicy, Theme};

pub fn run(input: &Path) -> Result<()> {
    let root = super::doc_root(input);
    let security = SecurityPolicy::strict(&root);
    let markdown = super::read_input(input, &security)?;

    let doc = md2pdf_core::parser::parse(&markdown);
    let stats = Stats::of(&doc);

    // Lower to Typst source (cheap, no compilation) to collect diagnostics
    // such as denied assets, dropped HTML and unrendered diagrams.
    let opts = RenderOptions {
        theme: Theme::Default,
        paper: Paper::A4,
        toc: false,
        title: None,
        security,
    };
    let rendered = md2pdf_core::render::for_format(OutputFormat::Typst).render(&doc, &opts)?;

    println!("{}", input.display());
    println!(
        "  {} headings, {} paragraphs, {} code blocks, {} tables, {} images, {} footnotes",
        stats.headings, stats.paragraphs, stats.code, stats.tables, stats.images, stats.footnotes
    );

    if rendered.diagnostics.is_empty() {
        println!("  ok — no problems found");
    } else {
        println!("  {} warning(s):", rendered.diagnostics.len());
        for d in &rendered.diagnostics {
            println!("    - {}", d.message);
        }
    }
    Ok(())
}

#[derive(Default)]
struct Stats {
    headings: usize,
    paragraphs: usize,
    code: usize,
    tables: usize,
    images: usize,
    footnotes: usize,
}

impl Stats {
    fn of(doc: &Document) -> Stats {
        let mut s = Stats::default();
        s.walk_blocks(&doc.blocks);
        s
    }

    fn walk_blocks(&mut self, blocks: &[Block]) {
        for b in blocks {
            match b {
                Block::Heading { content, .. } => {
                    self.headings += 1;
                    self.walk_inlines(content);
                }
                Block::Paragraph(c) => {
                    self.paragraphs += 1;
                    self.walk_inlines(c);
                }
                Block::CodeBlock { .. } => self.code += 1,
                Block::Table { head, rows, .. } => {
                    self.tables += 1;
                    for cell in head {
                        self.walk_inlines(cell);
                    }
                    for row in rows {
                        for cell in row {
                            self.walk_inlines(cell);
                        }
                    }
                }
                Block::BlockQuote(blocks) => self.walk_blocks(blocks),
                Block::List { items, .. } => {
                    for item in items {
                        self.walk_blocks(&item.blocks);
                    }
                }
                Block::ThematicBreak | Block::RawHtml(_) => {}
            }
        }
    }

    fn walk_inlines(&mut self, inlines: &[Inline]) {
        for i in inlines {
            match i {
                Inline::Image { .. } => self.images += 1,
                Inline::Footnote(blocks) => {
                    self.footnotes += 1;
                    self.walk_blocks(blocks);
                }
                Inline::Emph(c)
                | Inline::Strong(c)
                | Inline::Strikethrough(c)
                | Inline::Superscript(c)
                | Inline::Link { content: c, .. } => self.walk_inlines(c),
                _ => {}
            }
        }
    }
}
