//! Property tests: for *any* input string, parsing must not panic and the
//! Typst lowering must be a pure function of the input.

use md2pdf_core::render::OutputFormat;
use md2pdf_core::{Paper, RenderOptions, SecurityPolicy, Theme, convert, parser};
use proptest::prelude::*;

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

proptest! {
    #[test]
    fn parse_never_panics(input in any::<String>()) {
        let _ = parser::parse(&input);
    }

    #[test]
    fn lowering_is_deterministic(input in any::<String>()) {
        let a = convert(&input, &opts(), OutputFormat::Typst).unwrap();
        let b = convert(&input, &opts(), OutputFormat::Typst).unwrap();
        prop_assert_eq!(a.bytes, b.bytes);
    }

    #[test]
    fn markdownish_inputs_parse(input in "[-#>*`|\\[\\]()!{}a-z0-9 \n]{0,400}") {
        // Strings biased towards Markdown syntax exercise far more parser
        // branches than uniform random text.
        let _ = convert(&input, &opts(), OutputFormat::Typst).unwrap();
    }
}
