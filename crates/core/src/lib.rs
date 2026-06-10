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
pub use render::{Diagnostic, Layout, OutputFormat, Paper, RenderOptions, Rendered, Renderer};
pub use security::SecurityPolicy;
pub use theme::{Theme, ThemeSpec};

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
            layout: Default::default(),
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
    fn layout_header_footer_and_placeholders() {
        let md = "---\ntitle: Spec\nauthor: Ada\n---\n\n# Heading\n\nbody\n";
        let mut o = opts(".");
        o.layout = render::Layout {
            header: Some("{title} — internal".into()),
            footer: Some("by {author}".into()),
            page_numbers: true,
        };
        let out = convert(md, &o, OutputFormat::Typst).unwrap();
        let src = String::from_utf8(out.bytes).unwrap();
        assert!(src.contains("Spec — internal"), "header placeholder: {src}");
        assert!(src.contains("by Ada"), "footer placeholder: {src}");
        assert!(
            src.contains("counter(page).display"),
            "custom footer must still carry the page number: {src}"
        );
        assert!(
            !src.contains("numbering: \"1\""),
            "page numbering moves into the custom footer"
        );
    }

    #[test]
    fn layout_no_page_numbers() {
        let mut o = opts(".");
        o.layout.page_numbers = false;
        let out = convert("# T\n\nbody", &o, OutputFormat::Typst).unwrap();
        let src = String::from_utf8(out.bytes).unwrap();
        assert!(!src.contains("numbering: \"1\""));
        assert!(!src.contains("counter(page)"));
    }

    #[test]
    fn image_width_attribute_is_parsed() {
        let doc = parser::parse("![logo](logo.png){width=50%} tail text\n");
        let Some(Block::Paragraph(inlines)) = doc.blocks.first() else {
            panic!("expected paragraph");
        };
        assert!(matches!(
            &inlines[0],
            Inline::Image { width: Some(w), .. } if w == "50%"
        ));
        // The attribute block is consumed; the rest of the text survives.
        assert!(matches!(&inlines[1], Inline::Text(t) if t.contains("tail text")));
    }

    #[test]
    fn image_width_attribute_rejects_garbage() {
        let doc = parser::parse("![logo](logo.png){width=50%;rm -rf}\n");
        let Some(Block::Paragraph(inlines)) = doc.blocks.first() else {
            panic!("expected paragraph");
        };
        // Unparseable attr blocks are left as literal text, width stays None.
        assert!(matches!(&inlines[0], Inline::Image { width: None, .. }));
    }

    #[test]
    fn github_alert_becomes_admonition() {
        let md = "> [!WARNING]\n> Mind the gap.\n";
        let doc = parser::parse(md);
        assert!(matches!(
            doc.blocks.first(),
            Some(Block::Admonition {
                kind: crate::ir::AdmonitionKind::Warning,
                ..
            })
        ));
        let out = convert(md, &opts("."), OutputFormat::Typst).unwrap();
        let src = String::from_utf8(out.bytes).unwrap();
        assert!(src.contains("Warning"), "default title: {src}");
        assert!(src.contains("#block"), "callout box: {src}");
    }

    #[test]
    fn crlf_and_lf_inputs_render_identically() {
        let lf = "---\ntitle: X\n---\n\n# H\n\n```rust\nfn main() {}\n```\n\ntext\n";
        let crlf = lf.replace('\n', "\r\n");
        let a = convert(lf, &opts("."), OutputFormat::Typst).unwrap();
        let b = convert(&crlf, &opts("."), OutputFormat::Typst).unwrap();
        assert_eq!(
            a.bytes, b.bytes,
            "line-ending convention must not change the output bytes"
        );
    }

    #[test]
    fn definition_list_lowers_to_terms() {
        let md = "Apple\n: A fruit.\n\nRust\n: A language.\n: Also an oxide.\n";
        let doc = parser::parse(md);
        let Some(Block::DefinitionList(items)) = doc.blocks.first() else {
            panic!("expected definition list, got {:?}", doc.blocks.first());
        };
        assert_eq!(items.len(), 2);
        assert_eq!(crate::ir::inline_text(&items[0].term), "Apple");

        let out = convert(md, &opts("."), OutputFormat::Typst).unwrap();
        let src = String::from_utf8(out.bytes).unwrap();
        assert!(src.contains("#terms("), "typst terms: {src}");
        assert!(src.contains("terms.item("), "typst terms items: {src}");

        let out = convert(md, &opts("."), OutputFormat::Html).unwrap();
        let html = String::from_utf8(out.bytes).unwrap();
        assert!(html.contains("<dt>Apple</dt>"), "html dt: {html}");
        assert!(html.contains("<dd>"), "html dd: {html}");
    }

    #[test]
    fn typst_source_render_is_deterministic() {
        let md = "# Title\n\nHello **world**.";
        let a = convert(md, &opts("."), OutputFormat::Typst).unwrap();
        let b = convert(md, &opts("."), OutputFormat::Typst).unwrap();
        assert_eq!(a.bytes, b.bytes, "Typst lowering must be reproducible");
    }
}
