//! md2pdf-core — a renderer-agnostic Markdown documentation engine.
//!
//! The pipeline is three decoupled stages:
//!
//! ```text
//! Markdown ──parser──▶ Document (IR) ──Renderer──▶ bytes (PDF, Typst, …)
//! ```
//!
//! * [`parser`] is the only module that depends on comrak.
//! * [`render`] back-ends depend only on the [`ir`], never on the parser.
//! * [`security`] enforces deny-by-default handling of external assets.

pub mod config;
pub mod ir;
pub mod parser;
pub mod render;
pub mod security;
pub mod theme;

pub use config::Config;
pub use ir::Document;
pub use render::{Diagnostic, OutputFormat, Paper, RenderOptions, Rendered, Renderer};
pub use security::SecurityPolicy;
pub use theme::Theme;

/// The pinned Typst engine version, surfaced by `md2pdf doctor`.
pub const ENGINE_VERSION: &str = "typst 0.14.2";

/// Convenience pipeline: parse `markdown` and render it in one call.
pub fn convert(
    markdown: &str,
    opts: &RenderOptions,
    format: OutputFormat,
) -> anyhow::Result<Rendered> {
    let doc = parser::parse(markdown);
    render::for_format(format).render(&doc, opts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{Block, Inline};
    use std::path::PathBuf;

    fn opts(root: &str) -> RenderOptions {
        RenderOptions {
            theme: Theme::Default,
            paper: Paper::A4,
            toc: false,
            title: None,
            security: SecurityPolicy::strict(PathBuf::from(root)),
        }
    }

    #[test]
    fn parses_frontmatter_and_body() {
        let md = "---\ntitle: My Doc\nauthor: Ada\ncustom: yes\n---\n\n# Hello\n\nbody text\n";
        let doc = parser::parse(md);
        assert_eq!(doc.meta.title.as_deref(), Some("My Doc"));
        assert_eq!(doc.meta.author.as_deref(), Some("Ada"));
        assert_eq!(
            doc.meta.extra.get("custom").map(String::as_str),
            Some("yes")
        );
        assert!(matches!(
            doc.blocks.first(),
            Some(Block::Heading { level: 1, .. })
        ));
    }

    #[test]
    fn no_frontmatter_leaves_body_intact() {
        let doc = parser::parse("# Title\n\ntext");
        assert_eq!(doc.meta.title, None);
        assert_eq!(doc.first_heading_text().as_deref(), Some("Title"));
    }

    #[test]
    fn task_list_and_table_lower_to_ir() {
        let md = "- [x] done\n- [ ] todo\n\n| a | b |\n|---|---|\n| 1 | 2 |\n";
        let doc = parser::parse(md);
        assert!(doc.blocks.iter().any(|b| matches!(b, Block::List { .. })));
        assert!(doc.blocks.iter().any(|b| matches!(b, Block::Table { .. })));
    }

    #[test]
    fn footnote_is_inlined() {
        let doc = parser::parse("text[^a]\n\n[^a]: the note\n");
        let has_footnote = doc.blocks.iter().any(|b| match b {
            Block::Paragraph(inlines) => inlines.iter().any(|i| matches!(i, Inline::Footnote(_))),
            _ => false,
        });
        assert!(has_footnote);
    }

    #[test]
    fn security_denies_traversal_and_remote() {
        let policy = SecurityPolicy::strict("/tmp");
        assert!(matches!(
            policy.resolve_image("https://evil.test/x.png"),
            security::AssetDecision::Deny(_)
        ));
        assert!(matches!(
            policy.resolve_image("../../etc/passwd"),
            security::AssetDecision::Deny(_)
        ));
        assert!(matches!(
            policy.resolve_image("/etc/passwd"),
            security::AssetDecision::Deny(_)
        ));
    }

    #[test]
    fn security_fails_closed_when_root_cannot_be_canonicalized() {
        // A non-existent root cannot be canonicalised; asset access must deny
        // rather than fall back to a weaker comparison.
        let policy = SecurityPolicy::strict("/no/such/root/md2pdf-test");
        assert!(matches!(
            policy.resolve_image("logo.png"),
            security::AssetDecision::Deny(_)
        ));
    }

    #[test]
    fn loose_list_item_paragraphs_are_separated() {
        let md = "- first paragraph\n\n  second paragraph\n";
        let out = convert(md, &opts("."), OutputFormat::Typst).unwrap();
        let src = String::from_utf8(out.bytes).unwrap();
        assert!(
            src.contains("#parbreak()"),
            "multiple paragraphs in one list item must not run together: {src}"
        );
    }

    #[test]
    fn typst_source_render_is_deterministic() {
        let md = "# Title\n\nHello **world**.";
        let a = convert(md, &opts("."), OutputFormat::Typst).unwrap();
        let b = convert(md, &opts("."), OutputFormat::Typst).unwrap();
        assert_eq!(a.bytes, b.bytes, "Typst lowering must be reproducible");
    }
}
