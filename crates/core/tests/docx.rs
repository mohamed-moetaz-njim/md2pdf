//! DOCX renderer tests: a valid, deterministic OOXML package whose
//! document.xml carries the expected content.

use std::io::Read;

use md2pdf_core::render::OutputFormat;
use md2pdf_core::{Paper, RenderOptions, SecurityPolicy, Theme, convert};

fn opts() -> RenderOptions {
    RenderOptions {
        theme: Theme::Default,
        paper: Paper::A4,
        toc: false,
        title: None,
        layout: Default::default(),
        security: SecurityPolicy::strict("."),
    }
}

fn docx_bytes(md: &str, o: &RenderOptions) -> Vec<u8> {
    convert(md, o, OutputFormat::Docx).unwrap().bytes
}

fn zip_entry(bytes: &[u8], name: &str) -> String {
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes)).expect("valid zip");
    let mut file = archive.by_name(name).expect("entry present");
    let mut s = String::new();
    file.read_to_string(&mut s).unwrap();
    s
}

#[test]
fn renders_valid_docx_package() {
    let bytes = docx_bytes(
        "---\ntitle: Spec\nauthor: Ada\n---\n\n# Intro\n\nHello **world** and `code`.\n",
        &opts(),
    );
    assert_eq!(&bytes[..2], b"PK", "docx must be a zip");
    let doc = zip_entry(&bytes, "word/document.xml");
    assert!(doc.contains("Spec"), "title block: {doc}");
    assert!(doc.contains("Intro"), "heading text");
    assert!(doc.contains("Heading1"), "heading style");
    assert!(doc.contains("<w:b />"), "bold run for **world**");
    assert!(doc.contains("CodeChar"), "inline code style");
}

#[test]
fn docx_is_deterministic() {
    let md = "# T\n\n- a\n- b\n\n| x | y |\n|---|---|\n| 1 | 2 |\n\ntext[^a]\n\n[^a]: note\n";
    let a = docx_bytes(md, &opts());
    let b = docx_bytes(md, &opts());
    assert_eq!(a, b, "docx output must be byte-reproducible");
}

#[test]
fn docx_security_posture_matches() {
    let out = convert(
        "<div>raw</div>\n\n![alt](https://evil.test/x.png)\n",
        &opts(),
        OutputFormat::Docx,
    )
    .unwrap();
    let doc = zip_entry(&out.bytes, "word/document.xml");
    assert!(!doc.contains("<div>"), "raw HTML dropped");
    assert!(!doc.contains("evil.test"), "remote image denied");
    assert!(out.diagnostics.len() >= 2, "both drops reported");
}

#[test]
fn docx_structures_render() {
    let md = "> [!NOTE]\n> Callout.\n\nTerm\n: Details.\n\n1. one\n2. two\n\n> quoted\n";
    let bytes = docx_bytes(md, &opts());
    let doc = zip_entry(&bytes, "word/document.xml");
    assert!(doc.contains("Note"), "admonition title");
    assert!(
        doc.contains("Term") && doc.contains("Details."),
        "definition list"
    );
    assert!(doc.contains("w:numId"), "numbered list");
    assert!(doc.contains("Quote"), "blockquote style");
}

#[test]
fn docx_footnote_and_links() {
    let md = "See [docs](https://example.com)[^a]\n\n[^a]: the note body\n";
    let bytes = docx_bytes(md, &opts());
    let doc = zip_entry(&bytes, "word/document.xml");
    assert!(doc.contains("footnoteReference"), "footnote ref: {doc}");
    let notes = zip_entry(&bytes, "word/footnotes.xml");
    assert!(notes.contains("the note body"), "footnote body: {notes}");
    let rels = zip_entry(&bytes, "word/_rels/document.xml.rels");
    assert!(rels.contains("https://example.com"), "hyperlink rel");
}

#[test]
fn docx_header_footer_and_toc() {
    let mut o = opts();
    o.toc = true;
    o.layout.header = Some("{title} — internal".into());
    o.layout.footer = Some("by {author}".into());
    let bytes = docx_bytes("---\ntitle: Spec\nauthor: Ada\n---\n\n# H\n\nbody\n", &o);
    let header = zip_entry(&bytes, "word/header1.xml");
    assert!(header.contains("Spec — internal"), "header placeholders");
    let footer = zip_entry(&bytes, "word/footer1.xml");
    assert!(footer.contains("by Ada"), "footer placeholders");
    let doc = zip_entry(&bytes, "word/document.xml");
    assert!(
        doc.to_lowercase().contains("toc"),
        "table of contents field"
    );
}
