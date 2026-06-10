//! Criterion micro-benchmarks for the two CPU-bound pipeline stages:
//! Markdown → IR parsing, and IR → Typst lowering.
//!
//! Run with: `cargo bench -p md2pdf-core`

use std::fmt::Write;

use criterion::{Criterion, criterion_group, criterion_main};
use md2pdf_core::render::OutputFormat;
use md2pdf_core::{Paper, RenderOptions, SecurityPolicy, Theme, convert, parser};

/// A synthetic document of `sections` sections exercising most constructs.
fn document(sections: usize) -> String {
    let mut md = String::from("---\ntitle: Benchmark\nauthor: criterion\n---\n\n");
    for i in 0..sections {
        let _ = write!(
            md,
            "## Section {i}\n\n\
             Some *styled* text with **bold**, `code` and a [link](https://example.com).\n\n\
             - item one\n- item two with ~~strikethrough~~\n- [x] a task\n\n\
             > [!NOTE]\n> Callout body {i}.\n\n\
             | col | value |\n|---|--:|\n| row | {i} |\n\n\
             ```rust\nfn section_{i}() {{ println!(\"{i}\"); }}\n```\n\n\
             Term {i}\n: Its definition.\n\n"
        );
    }
    md
}

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

fn benches(c: &mut Criterion) {
    let md = document(100);

    c.bench_function("parse_100_sections", |b| {
        b.iter(|| parser::parse(std::hint::black_box(&md)))
    });

    c.bench_function("lower_typst_100_sections", |b| {
        let o = opts();
        b.iter(|| convert(std::hint::black_box(&md), &o, OutputFormat::Typst).unwrap())
    });

    c.bench_function("lower_html_100_sections", |b| {
        let o = opts();
        b.iter(|| convert(std::hint::black_box(&md), &o, OutputFormat::Html).unwrap())
    });
}

criterion_group!(pipeline, benches);
criterion_main!(pipeline);
