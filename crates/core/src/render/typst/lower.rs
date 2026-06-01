//! Lower the [`Document`] IR to Typst markup.
//!
//! Every literal is emitted as a Typst string (`#"..."`) and all structure is
//! expressed through Typst functions, so the only characters that ever need
//! escaping are those special inside a Typst string literal.

use std::fmt::Write;

use crate::ir::{Align, Block, Document, Inline, ListItem};
use crate::render::{Diagnostic, RenderOptions};
use crate::security::AssetDecision;
use crate::theme::ThemeSpec;

/// Produce a complete Typst document for `doc`.
pub fn lower(doc: &Document, opts: &RenderOptions, diags: &mut Vec<Diagnostic>) -> String {
    let spec = opts.theme.spec();
    let mut s = String::new();

    preamble(&mut s, &spec, opts);
    title_block(&mut s, doc, &spec, opts);

    if opts.toc {
        s.push_str("#outline(title: [Contents], indent: auto)\n#v(1.2em)\n\n");
    }

    let mut ctx = Ctx { opts, diags };
    for block in &doc.blocks {
        emit_block(block, &mut s, &mut ctx);
    }
    s
}

struct Ctx<'a> {
    opts: &'a RenderOptions,
    diags: &'a mut Vec<Diagnostic>,
}

fn preamble(s: &mut String, spec: &ThemeSpec, opts: &RenderOptions) {
    let _ = write!(
        s,
        "#set page(paper: \"{}\", margin: (x: {}cm, y: {}cm), numbering: \"1\")\n",
        opts.paper.typst_name(),
        spec.margin_x_cm,
        spec.margin_y_cm
    );
    let _ = write!(
        s,
        "#set text(font: \"{}\", size: {}pt, lang: \"en\")\n",
        spec.body_font, spec.body_size_pt
    );
    let _ = write!(
        s,
        "#set par(justify: {}, leading: 0.65em, first-line-indent: {}em)\n",
        spec.justify, spec.first_line_indent_em
    );
    let _ = write!(s, "#show heading: set text(fill: rgb(\"{}\"))\n", spec.heading_color);
    s.push_str("#show heading.where(level: 1): set text(size: 1.5em)\n");
    let _ = write!(s, "#show link: set text(fill: rgb(\"{}\"))\n", spec.link_color);
    let _ = write!(s, "#show raw: set text(font: \"{}\", size: 9.5pt)\n", spec.mono_font);
    match spec.code_stroke {
        Some(stroke) => {
            let _ = write!(
                s,
                "#show raw.where(block: true): block.with(fill: rgb(\"{}\"), inset: 10pt, radius: 4pt, width: 100%, stroke: 0.5pt + rgb(\"{}\"))\n",
                spec.code_fill, stroke
            );
        }
        None => {
            let _ = write!(
                s,
                "#show raw.where(block: true): block.with(fill: rgb(\"{}\"), inset: 10pt, radius: 2pt, width: 100%)\n",
                spec.code_fill
            );
        }
    }
    let _ = write!(
        s,
        "#show quote.where(block: true): set block(stroke: (left: 2pt + rgb(\"{}\")), inset: (left: 1em, y: 0.4em))\n\n",
        spec.accent_color
    );
}

fn title_block(s: &mut String, doc: &Document, spec: &ThemeSpec, opts: &RenderOptions) {
    let Some(title) = doc.resolve_title(opts.title.as_deref()) else {
        return;
    };
    if title.trim().is_empty() {
        return;
    }
    s.push_str("#align(center)[\n");
    let _ = write!(
        s,
        "  #text(size: 22pt, weight: \"bold\", fill: rgb(\"{}\"))[#\"{}\"]\n",
        spec.accent_color,
        esc(&title)
    );
    if let Some(sub) = &doc.meta.subtitle {
        let _ = write!(s, "  #v(0.2em)\n  #text(size: 13pt)[#\"{}\"]\n", esc(sub));
    }
    let byline: Vec<String> = [doc.meta.author.as_deref(), doc.meta.date.as_deref()]
        .into_iter()
        .flatten()
        .map(esc)
        .collect();
    if !byline.is_empty() {
        let _ = write!(
            s,
            "  #v(0.3em)\n  #text(size: 10pt, fill: luma(90))[#\"{}\"]\n",
            byline.join(" — ")
        );
    }
    s.push_str("  #v(0.3em)\n");
    let _ = write!(s, "  #line(length: 38%, stroke: 0.6pt + rgb(\"{}\"))\n", spec.accent_color);
    s.push_str("]\n#v(1.4em)\n\n");
}

fn emit_block(block: &Block, s: &mut String, ctx: &mut Ctx) {
    match block {
        Block::Heading { level, content } => {
            let _ = write!(s, "#heading(level: {level})[");
            emit_inlines(content, s, ctx);
            s.push_str("]\n\n");
        }
        Block::Paragraph(content) => {
            emit_inlines(content, s, ctx);
            s.push_str("\n\n");
        }
        Block::CodeBlock { lang, code } => emit_code(lang.as_deref(), code, s, ctx),
        Block::BlockQuote(blocks) => {
            s.push_str("#quote(block: true)[\n");
            for b in blocks {
                emit_block(b, s, ctx);
            }
            s.push_str("]\n\n");
        }
        Block::List { ordered, items } => emit_list(*ordered, items, s, ctx),
        Block::Table { align, head, rows } => emit_table(align, head, rows, s, ctx),
        Block::ThematicBreak => {
            s.push_str("#line(length: 100%, stroke: 0.5pt + luma(180))\n#v(0.3em)\n\n");
        }
        Block::RawHtml(_) => {
            // Dropped by policy; record it once so `validate` can report it.
            ctx.diags
                .push(Diagnostic::new("raw HTML block dropped (disabled for safety)"));
        }
    }
}

fn emit_code(lang: Option<&str>, code: &str, s: &mut String, ctx: &mut Ctx) {
    if lang == Some("mermaid") {
        // Real diagram rendering is on the roadmap; for now we preserve the
        // source verbatim and flag it rather than silently dropping it.
        ctx.diags
            .push(Diagnostic::new("mermaid diagram rendered as source (diagram support is planned)"));
    }
    s.push_str("#raw(block: true");
    if let Some(lang) = lang {
        let _ = write!(s, ", lang: \"{}\"", esc(lang));
    }
    let _ = write!(s, ", \"{}\")\n\n", esc(code));
}

fn emit_list(ordered: bool, items: &[ListItem], s: &mut String, ctx: &mut Ctx) {
    s.push_str(if ordered { "#enum(\n" } else { "#list(\n" });
    for item in items {
        s.push_str("  [");
        if let Some(checked) = item.task {
            let mark = if checked { "☑ " } else { "☐ " };
            let _ = write!(s, "#\"{}\"", esc(mark));
        }
        emit_item_body(&item.blocks, s, ctx);
        s.push_str("],\n");
    }
    s.push_str(")\n\n");
}

fn emit_item_body(blocks: &[Block], s: &mut String, ctx: &mut Ctx) {
    for b in blocks {
        match b {
            Block::Paragraph(content) => emit_inlines(content, s, ctx),
            other => emit_block(other, s, ctx),
        }
    }
}

fn emit_table(
    align: &[Align],
    head: &[Vec<Inline>],
    rows: &[Vec<Vec<Inline>>],
    s: &mut String,
    ctx: &mut Ctx,
) {
    let columns = head
        .len()
        .max(rows.first().map(|r| r.len()).unwrap_or(0))
        .max(align.len())
        .max(1);
    s.push_str("#table(\n");
    let _ = write!(s, "  columns: {columns},\n");
    s.push_str("  align: (");
    for i in 0..columns {
        let a = align.get(i).copied().unwrap_or(Align::None);
        s.push_str(match a {
            Align::Center => "center",
            Align::Right => "right",
            Align::Left | Align::None => "left",
        });
        s.push_str(", ");
    }
    s.push_str("),\n  stroke: 0.5pt + luma(190),\n");

    if !head.is_empty() {
        s.push_str("  table.header(");
        for cell in head {
            s.push('[');
            emit_inlines(cell, s, ctx);
            s.push_str("], ");
        }
        s.push_str("),\n");
    }
    for row in rows {
        s.push_str("  ");
        for cell in row {
            s.push('[');
            emit_inlines(cell, s, ctx);
            s.push_str("], ");
        }
        s.push('\n');
    }
    s.push_str(")\n\n");
}

fn emit_inlines(inlines: &[Inline], s: &mut String, ctx: &mut Ctx) {
    for inline in inlines {
        emit_inline(inline, s, ctx);
    }
}

fn emit_inline(inline: &Inline, s: &mut String, ctx: &mut Ctx) {
    match inline {
        Inline::Text(t) => {
            let _ = write!(s, "#\"{}\"", esc(t));
        }
        Inline::SoftBreak => s.push_str("#\" \""),
        Inline::LineBreak => s.push_str("#linebreak()"),
        Inline::Code(c) => {
            let _ = write!(s, "#raw(\"{}\")", esc(c));
        }
        Inline::Emph(c) => wrap("#emph[", c, s, ctx),
        Inline::Strong(c) => wrap("#strong[", c, s, ctx),
        Inline::Strikethrough(c) => wrap("#strike[", c, s, ctx),
        Inline::Superscript(c) => wrap("#super[", c, s, ctx),
        Inline::Link { href, content } => {
            let _ = write!(s, "#link(\"{}\")[", esc(href));
            emit_inlines(content, s, ctx);
            s.push(']');
        }
        Inline::Image { src, alt } => emit_image(src, alt, s, ctx),
        Inline::Footnote(blocks) => {
            s.push_str("#footnote[");
            for b in blocks {
                emit_block(b, s, ctx);
            }
            s.push(']');
        }
    }
}

fn wrap(open: &str, content: &[Inline], s: &mut String, ctx: &mut Ctx) {
    s.push_str(open);
    emit_inlines(content, s, ctx);
    s.push(']');
}

fn emit_image(src: &str, alt: &str, s: &mut String, ctx: &mut Ctx) {
    match ctx.opts.security.resolve_image(src) {
        AssetDecision::Allow(path) => {
            let _ = write!(s, "#image(\"{}\")", esc(&path));
        }
        AssetDecision::Deny(reason) => {
            ctx.diags.push(Diagnostic::new(reason));
            // Fall back to the alt text so the document still reads sensibly.
            let _ = write!(s, "#emph[#\"{}\"]", esc(alt));
        }
    }
}

/// Escape a string for inclusion inside a Typst `"..."` literal.
fn esc(input: &str) -> String {
    let mut o = String::with_capacity(input.len() + 2);
    for c in input.chars() {
        match c {
            '\\' => o.push_str("\\\\"),
            '"' => o.push_str("\\\""),
            '\n' => o.push_str("\\n"),
            '\r' => o.push_str("\\r"),
            '\t' => o.push_str("\\t"),
            _ => o.push(c),
        }
    }
    o
}
