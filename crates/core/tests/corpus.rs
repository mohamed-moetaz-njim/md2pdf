//! Parser robustness: hostile or malformed inputs must never panic, and the
//! Typst lowering must stay deterministic for every one of them.

use md2pdf_core::render::OutputFormat;
use md2pdf_core::{Paper, RenderOptions, SecurityPolicy, Theme, convert};

fn opts() -> RenderOptions {
    RenderOptions {
        theme: Theme::Default,
        paper: Paper::A4,
        toc: true,
        title: None,
        layout: Default::default(),
        security: SecurityPolicy::strict("."),
    }
}

const CORPUS: &[&str] = &[
    // Empty / trivial
    "",
    "\n",
    "\u{feff}",
    // Unterminated structures
    "```rust\nfn main() {",
    "| a | b |\n|---|",
    "| a | b |\n|---|---|---|---|\n| 1 |",
    "[link](",
    "![img](",
    "**bold _mixed*",
    "> quote\n> > deeper\n> > > ```",
    // Frontmatter edge cases
    "---\n",
    "---\ntitle: x",
    "---\n---\n",
    "---\n: : :\nnot yaml at all\n---\nbody",
    "----\nnot frontmatter\n----\n",
    // Footnote oddities
    "[^a]\n",
    "[^a]: orphan definition\n",
    "self[^a]\n\n[^a]: refers[^a] to itself\n",
    // Task lists and alerts
    "- [x]\n- [ \n- [z] x\n",
    "> [!NOTE]\n",
    "> [!BOGUS]\n> text\n",
    // Image attribute blocks
    "![a](b.png){width=}\n",
    "![a](b.png){width=-5%}\n",
    "![a](b.png){width=1e999%}\n",
    "![a](b.png){{{{\n",
    // Unicode stress
    "# \u{0000}\u{0001}\u{0007}\n",
    "Z\u{0301}a\u{0308}l\u{0327}g\u{0301}o\u{0308} **t\u{0335}e\u{0336}x\u{0337}t**",
    "\u{202e}right-to-left override\u{202c}\n",
    "🏳️‍🌈🏴‍☠️👨‍👩‍👧‍👦 ## not a heading\n",
    // Typst-injection attempts (must come out as inert string content)
    "#eval(\"1+1\")\n",
    "\"; #import \"evil.typ\"\n",
    "#raw(read(\"/etc/passwd\"))\n",
    "`#eval` and ```#import``` fences\n",
];

#[test]
fn corpus_never_panics_and_is_deterministic() {
    for (i, case) in CORPUS.iter().enumerate() {
        let a = convert(case, &opts(), OutputFormat::Typst)
            .unwrap_or_else(|e| panic!("case {i} failed to render: {e}"));
        let b = convert(case, &opts(), OutputFormat::Typst).unwrap();
        assert_eq!(a.bytes, b.bytes, "case {i} must lower deterministically");
    }
}

#[test]
fn deep_blockquote_nesting() {
    let md = format!("{}boom", "> ".repeat(300));
    let _ = convert(&md, &opts(), OutputFormat::Typst).unwrap();
}

#[test]
fn deep_list_nesting() {
    let mut md = String::new();
    for depth in 0..200 {
        md.push_str(&"  ".repeat(depth));
        md.push_str("- item\n");
    }
    let _ = convert(&md, &opts(), OutputFormat::Typst).unwrap();
}

#[test]
fn pathological_emphasis() {
    // Classic CommonMark quadratic-blowup shape; must terminate and not panic.
    let md = format!("{}x{}", "*a **a ".repeat(500), " a** a*".repeat(500));
    let _ = convert(&md, &opts(), OutputFormat::Typst).unwrap();
}

#[test]
fn huge_table_row() {
    let md = format!("|{}\n|{}\n", "x|".repeat(1000), "-|".repeat(1000));
    let _ = convert(&md, &opts(), OutputFormat::Typst).unwrap();
}

#[test]
fn injection_attempts_are_quoted() {
    // Body text must reach Typst only inside string literals: the document
    // can never call Typst functions.
    let md = "#eval(\"2+2\") and #import \"x.typ\"\n";
    let out = convert(md, &opts(), OutputFormat::Typst).unwrap();
    let src = String::from_utf8(out.bytes).unwrap();
    assert!(
        src.contains(r##"#"#eval(\"2+2\") and #import \"x.typ\"""##),
        "user #eval/#import must be inert, escaped string content: {src}"
    );
}
