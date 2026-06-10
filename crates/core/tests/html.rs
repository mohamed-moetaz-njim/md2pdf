//! HTML renderer tests: structure, escaping and the shared security posture.

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

fn html(md: &str, o: &RenderOptions) -> String {
    let out = convert(md, o, OutputFormat::Html).unwrap();
    String::from_utf8(out.bytes).unwrap()
}

#[test]
fn renders_standalone_page() {
    let src = html(
        "---\ntitle: Spec\nauthor: Ada\n---\n\n# Intro\n\nHello **world**.\n",
        &opts(),
    );
    assert!(src.starts_with("<!DOCTYPE html>"));
    assert!(src.contains("<title>Spec</title>"));
    assert!(src.contains("Ada"));
    assert!(src.contains("<h1 id=\"intro\">"));
    assert!(src.contains("<strong>world</strong>"));
    assert!(src.contains("</html>"));
}

#[test]
fn escapes_script_injection() {
    // A leading HTML tag makes the whole block raw HTML, which is dropped.
    let src = html("<script>alert(1)</script>\n", &opts());
    assert!(
        !src.contains("<script>"),
        "raw HTML must never pass through: {src}"
    );
    // Markup characters in ordinary text and inline code are entity-escaped.
    let src = html("a `<b>` tag and 1 < 2 & 3 > 2\n", &opts());
    assert!(src.contains("&lt;b&gt;"), "inline code is escaped: {src}");
    assert!(
        src.contains("1 &lt; 2 &amp; 3 &gt; 2"),
        "text is escaped: {src}"
    );
}

#[test]
fn raw_html_block_is_dropped_with_diagnostic() {
    let out = convert(
        "<div>block html</div>\n\ntext\n",
        &opts(),
        OutputFormat::Html,
    )
    .unwrap();
    let src = String::from_utf8(out.bytes).unwrap();
    assert!(!src.contains("<div>block html</div>"));
    assert!(
        out.diagnostics
            .iter()
            .any(|d| d.message.contains("raw HTML")),
        "dropping must be reported"
    );
}

#[test]
fn remote_images_are_denied() {
    let out = convert(
        "![alt text](https://evil.test/x.png)\n",
        &opts(),
        OutputFormat::Html,
    )
    .unwrap();
    let src = String::from_utf8(out.bytes).unwrap();
    assert!(
        !src.contains("evil.test"),
        "no remote refs in output: {src}"
    );
    assert!(src.contains("<em>alt text</em>"), "alt fallback: {src}");
    assert!(!out.diagnostics.is_empty());
}

#[test]
fn toc_links_match_heading_anchors() {
    let mut o = opts();
    o.toc = true;
    let src = html("# Alpha\n\n## Beta Gamma\n\n## Beta Gamma\n", &o);
    assert!(src.contains("href=\"#alpha\"") && src.contains("id=\"alpha\""));
    assert!(src.contains("href=\"#beta-gamma\"") && src.contains("id=\"beta-gamma\""));
    // Duplicate headings get unique anchors.
    assert!(src.contains("href=\"#beta-gamma-2\"") && src.contains("id=\"beta-gamma-2\""));
}

#[test]
fn footnotes_render_as_section() {
    let src = html("text[^a]\n\n[^a]: the note\n", &opts());
    assert!(src.contains("id=\"fnref1\""));
    assert!(src.contains("<section class=\"footnotes\">"));
    assert!(src.contains("the note"));
}

#[test]
fn admonition_and_table_render() {
    let md = "> [!TIP]\n> Try it.\n\n| a | b |\n|---|--:|\n| 1 | 2 |\n";
    let src = html(md, &opts());
    assert!(src.contains("class=\"admonition tip\""));
    assert!(src.contains("<th>a</th>"));
    assert!(src.contains("<td style=\"text-align: right\">2</td>"));
}

#[test]
fn html_lowering_is_deterministic() {
    let md = "# T\n\ntext[^a]\n\n[^a]: note\n\n- [x] done\n";
    let a = convert(md, &opts(), OutputFormat::Html).unwrap();
    let b = convert(md, &opts(), OutputFormat::Html).unwrap();
    assert_eq!(a.bytes, b.bytes);
}
