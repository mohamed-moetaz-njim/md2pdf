//! Golden test: the Typst lowering of `examples/sample.md` is checked in.
//!
//! Any intentional change to the lowering shows up as a reviewable diff.
//! Regenerate with:
//!
//! ```console
//! UPDATE_GOLDEN=1 cargo test -p md2pdf-core --test golden
//! ```

use std::path::PathBuf;

use md2pdf_core::render::OutputFormat;
use md2pdf_core::{Paper, RenderOptions, SecurityPolicy, Theme, convert};

#[test]
fn sample_lowering_matches_golden() {
    let sample = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/sample.md");
    let golden = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden/sample.typ");

    let markdown = std::fs::read_to_string(&sample).unwrap();
    let opts = RenderOptions {
        theme: Theme::Default,
        paper: Paper::A4,
        toc: true,
        title: None,
        layout: Default::default(),
        security: SecurityPolicy::strict(sample.parent().unwrap()),
    };
    let rendered = convert(&markdown, &opts, OutputFormat::Typst).unwrap();
    let actual = String::from_utf8(rendered.bytes).unwrap();

    if std::env::var_os("UPDATE_GOLDEN").is_some() {
        std::fs::write(&golden, &actual).unwrap();
        return;
    }

    let expected = std::fs::read_to_string(&golden)
        .expect("golden file missing — run with UPDATE_GOLDEN=1 to create it");
    assert_eq!(
        actual, expected,
        "Typst lowering changed; if intentional, regenerate with UPDATE_GOLDEN=1"
    );
}
