//! The renderer-agnostic intermediate representation (IR).
//!
//! This is the contract that decouples parsing from rendering. The parser
//! ([`crate::parser`]) lowers Markdown into a [`Document`]; renderers
//! ([`crate::render`]) consume a [`Document`] and know nothing about comrak or
//! Typst. New back-ends (HTML, DOCX, ...) only need to understand these types.

use std::collections::BTreeMap;

/// A fully parsed document: metadata plus a tree of block-level content.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Document {
    pub meta: Meta,
    pub blocks: Vec<Block>,
}

/// Document metadata, sourced from YAML-style frontmatter and/or CLI flags.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Meta {
    pub title: Option<String>,
    pub author: Option<String>,
    pub date: Option<String>,
    pub subtitle: Option<String>,
    /// Any other `key: value` pairs found in the frontmatter, preserved in order.
    pub extra: BTreeMap<String, String>,
}

/// Column alignment for tables.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Align {
    None,
    Left,
    Center,
    Right,
}

/// Block-level content.
#[derive(Debug, Clone, PartialEq)]
pub enum Block {
    Heading {
        level: u8,
        content: Vec<Inline>,
    },
    Paragraph(Vec<Inline>),
    /// A fenced/indented code block. `lang` drives syntax highlighting; the
    /// reserved language `mermaid` is recognised by renderers that support it.
    CodeBlock {
        lang: Option<String>,
        code: String,
    },
    BlockQuote(Vec<Block>),
    List {
        ordered: bool,
        items: Vec<ListItem>,
    },
    Table {
        align: Vec<Align>,
        head: Vec<Vec<Inline>>,
        rows: Vec<Vec<Vec<Inline>>>,
    },
    ThematicBreak,
    /// A definition list: `term` lines followed by `: details` lines.
    DefinitionList(Vec<DefinitionItem>),
    /// A GitHub-style alert (`> [!NOTE]` …), rendered as a callout box.
    Admonition {
        kind: AdmonitionKind,
        /// Display title; defaults to the kind's name when not overridden.
        title: String,
        blocks: Vec<Block>,
    },
    /// Raw HTML, dropped by safe renderers (see the security model).
    RawHtml(String),
}

/// The five GitHub alert kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdmonitionKind {
    Note,
    Tip,
    Important,
    Warning,
    Caution,
}

/// A single list item; `task` is `Some(checked)` for GitHub task-list items.
#[derive(Debug, Clone, PartialEq)]
pub struct ListItem {
    pub task: Option<bool>,
    pub blocks: Vec<Block>,
}

/// One term/details pair of a [`Block::DefinitionList`].
#[derive(Debug, Clone, PartialEq)]
pub struct DefinitionItem {
    pub term: Vec<Inline>,
    pub details: Vec<Block>,
}

/// Inline (span-level) content.
#[derive(Debug, Clone, PartialEq)]
pub enum Inline {
    Text(String),
    Emph(Vec<Inline>),
    Strong(Vec<Inline>),
    Strikethrough(Vec<Inline>),
    Superscript(Vec<Inline>),
    Code(String),
    Link {
        href: String,
        content: Vec<Inline>,
    },
    Image {
        src: String,
        alt: String,
        /// Display width from `{width=…}` after the image, e.g. `50%`, `4cm`.
        /// Always a validated dimension (number + `%`/`cm`/`mm`/`in`/`pt`/`em`).
        width: Option<String>,
    },
    /// A footnote whose definition has been resolved and inlined.
    Footnote(Vec<Block>),
    SoftBreak,
    LineBreak,
}

impl Document {
    /// Resolve the effective title: explicit override, then frontmatter, then
    /// the first heading, then `None`.
    pub fn resolve_title(&self, override_title: Option<&str>) -> Option<String> {
        if let Some(t) = override_title {
            return Some(t.to_string());
        }
        if let Some(t) = &self.meta.title {
            return Some(t.clone());
        }
        self.first_heading_text()
    }

    /// Plain text of the first heading in document order.
    pub fn first_heading_text(&self) -> Option<String> {
        self.blocks.iter().find_map(|b| match b {
            Block::Heading { content, .. } => {
                let t = inline_text(content);
                (!t.trim().is_empty()).then_some(t)
            }
            _ => None,
        })
    }
}

/// Flatten inline content to plain text (used for titles, outlines, validation).
pub fn inline_text(inlines: &[Inline]) -> String {
    let mut s = String::new();
    for i in inlines {
        match i {
            Inline::Text(t) | Inline::Code(t) => s.push_str(t),
            Inline::Emph(c)
            | Inline::Strong(c)
            | Inline::Strikethrough(c)
            | Inline::Superscript(c)
            | Inline::Link { content: c, .. } => s.push_str(&inline_text(c)),
            Inline::Image { alt, .. } => s.push_str(alt),
            Inline::SoftBreak | Inline::LineBreak => s.push(' '),
            Inline::Footnote(_) => {}
        }
    }
    s
}
