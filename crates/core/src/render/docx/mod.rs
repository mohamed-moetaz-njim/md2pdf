//! The experimental DOCX back-end: lowers the [`Document`] IR to an OOXML
//! word-processing document via `docx-rs`.
//!
//! Output is deterministic: `docx-rs` writes fixed zip timestamps and epoch
//! document properties, so the reproducibility guarantee holds here too.
//! Same security posture as the other back-ends — raw HTML is dropped,
//! remote images are denied, local images must pass the policy.
//!
//! Known limitations (reported as diagnostics where it matters): superscript
//! renders as plain text, `{width=…}` image sizing is ignored, and custom
//! page-number styling is fixed.

use docx_rs::{
    AbstractNumbering, AlignmentType, BreakType, Docx, Footer, Footnote, Header, Hyperlink,
    HyperlinkType, IndentLevel, Level, LevelJc, LevelText, NumberFormat, Numbering, NumberingId,
    Paragraph, Pic, Run, RunFonts, SpecialIndentType, Start, Style, StyleType, Table, TableCell,
    TableOfContents, TableRow,
};

use crate::ir::{AdmonitionKind, Block, Document, Inline, ListItem, Meta};
use crate::render::{Diagnostic, OutputFormat, RenderOptions, Rendered, Renderer};
use crate::security::AssetDecision;
use crate::theme::ThemeSpec;

const BULLET_NUM_ID: usize = 1;
const DECIMAL_NUM_ID: usize = 2;
/// Twips of indentation per nesting level (720 = 0.5").
const INDENT_STEP: i32 = 720;

/// Renders a [`Document`] to a `.docx` file (experimental).
pub struct DocxRenderer;

impl Renderer for DocxRenderer {
    fn format(&self) -> OutputFormat {
        OutputFormat::Docx
    }

    fn render(&self, doc: &Document, opts: &RenderOptions) -> anyhow::Result<Rendered> {
        let mut diagnostics = Vec::new();
        let spec = opts.theme.spec();

        let mut docx = with_styles(Docx::new(), &spec);
        docx = with_numberings(docx);
        docx = with_page_furniture(docx, &doc.meta, opts);

        docx = title_block(docx, doc, opts);
        if opts.toc {
            docx = docx.add_table_of_contents(
                TableOfContents::new()
                    .heading_styles_range(1, 6)
                    .alias("Contents")
                    .auto(),
            );
        }

        let mut ctx = Ctx {
            opts,
            diags: &mut diagnostics,
            footnote_seq: 0,
        };
        for block in &doc.blocks {
            docx = emit_block(docx, block, 0, &mut ctx);
        }

        let mut xml = docx.build();
        // docx-rs assigns paragraph ids from a process-global counter, which
        // breaks byte-reproducibility across renders; renumber them.
        let mut next_id = 1usize;
        normalize_para_ids(&mut xml.document, &mut next_id);
        for part in xml
            .headers
            .iter_mut()
            .chain(xml.footers.iter_mut())
            .chain(std::iter::once(&mut xml.footnotes))
        {
            normalize_para_ids(part, &mut next_id);
        }

        let mut buf = std::io::Cursor::new(Vec::new());
        xml.pack(&mut buf)
            .map_err(|e| anyhow::anyhow!("could not pack docx: {e}"))?;
        Ok(Rendered {
            bytes: buf.into_inner(),
            diagnostics,
        })
    }
}

/// Rewrite every `w14:paraId="xxxxxxxx"` with a sequential value so output
/// does not depend on how many documents this process rendered before.
fn normalize_para_ids(xml: &mut [u8], next_id: &mut usize) {
    const NEEDLE: &[u8] = b"w14:paraId=\"";
    let mut i = 0;
    while i + NEEDLE.len() + 8 <= xml.len() {
        if &xml[i..i + NEEDLE.len()] == NEEDLE {
            let start = i + NEEDLE.len();
            let id = format!("{:08x}", *next_id);
            xml[start..start + 8].copy_from_slice(id.as_bytes());
            *next_id += 1;
            i = start + 8;
        } else {
            i += 1;
        }
    }
}

struct Ctx<'a> {
    opts: &'a RenderOptions,
    diags: &'a mut Vec<Diagnostic>,
    /// Sequential footnote ids; docx-rs's own come from a process-global
    /// counter, which would break byte-reproducibility.
    footnote_seq: usize,
}

/// docx colors are hex without the leading `#`.
fn color(c: &str) -> String {
    c.trim_start_matches('#').to_string()
}

fn with_styles(docx: Docx, spec: &ThemeSpec) -> Docx {
    // docx run sizes are half-points.
    let body = (spec.body_size_pt * 2.0).round() as usize;
    let heading = color(&spec.heading_color);
    let mut d = docx
        .add_style(
            Style::new("Title", StyleType::Paragraph)
                .name("Title")
                .size(body + 22)
                .bold()
                .color(color(&spec.accent_color))
                .align(AlignmentType::Center),
        )
        .add_style(
            Style::new("Subtitle", StyleType::Paragraph)
                .name("Subtitle")
                .size(body + 4)
                .align(AlignmentType::Center),
        )
        .add_style(
            Style::new("Byline", StyleType::Paragraph)
                .name("Byline")
                .size(body.saturating_sub(2))
                .color("5a5a5a")
                .align(AlignmentType::Center),
        )
        .add_style(
            Style::new("Code", StyleType::Paragraph)
                .name("Code")
                .size(body.saturating_sub(3))
                .fonts(RunFonts::new().ascii(&spec.mono_font)),
        )
        .add_style(
            Style::new("CodeChar", StyleType::Character)
                .name("Code Char")
                .fonts(RunFonts::new().ascii(&spec.mono_font)),
        )
        .add_style(
            Style::new("Quote", StyleType::Paragraph)
                .name("Quote")
                .italic()
                .color("444444"),
        );
    // Heading1..Heading6, shrinking sizes.
    for level in 1..=6usize {
        let extra = [14usize, 8, 4, 2, 0, 0][level - 1];
        d = d.add_style(
            Style::new(format!("Heading{level}"), StyleType::Paragraph)
                .name(format!("Heading {level}"))
                .size(body + extra)
                .bold()
                .color(heading.clone()),
        );
    }
    d
}

fn with_numberings(docx: Docx) -> Docx {
    let mut bullet = AbstractNumbering::new(BULLET_NUM_ID);
    let mut decimal = AbstractNumbering::new(DECIMAL_NUM_ID);
    for level in 0..9usize {
        let indent = (level as i32 + 1) * INDENT_STEP;
        bullet = bullet.add_level(
            Level::new(
                level,
                Start::new(1),
                NumberFormat::new("bullet"),
                LevelText::new("•"),
                LevelJc::new("left"),
            )
            .indent(
                Some(indent),
                Some(SpecialIndentType::Hanging(320)),
                None,
                None,
            ),
        );
        decimal = decimal.add_level(
            Level::new(
                level,
                Start::new(1),
                NumberFormat::new("decimal"),
                LevelText::new(format!("%{}.", level + 1)),
                LevelJc::new("left"),
            )
            .indent(
                Some(indent),
                Some(SpecialIndentType::Hanging(320)),
                None,
                None,
            ),
        );
    }
    docx.add_abstract_numbering(bullet)
        .add_numbering(Numbering::new(BULLET_NUM_ID, BULLET_NUM_ID))
        .add_abstract_numbering(decimal)
        .add_numbering(Numbering::new(DECIMAL_NUM_ID, DECIMAL_NUM_ID))
}

fn with_page_furniture(docx: Docx, meta: &Meta, opts: &RenderOptions) -> Docx {
    let layout = &opts.layout;
    let mut d = docx;
    if let Some(text) = &layout.header {
        let text = substitute_meta(text, meta);
        d = d.header(
            Header::new().add_paragraph(
                Paragraph::new()
                    .align(AlignmentType::Center)
                    .add_run(Run::new().add_text(text).size(18).color("787878")),
            ),
        );
    }
    if let Some(text) = &layout.footer {
        let text = substitute_meta(text, meta);
        d = d.footer(Footer::new().add_paragraph(
            Paragraph::new().add_run(Run::new().add_text(text).size(18).color("787878")),
        ));
    }
    d
}

/// Resolve `{title}`, `{author}` and `{date}` placeholders against metadata.
fn substitute_meta(template: &str, meta: &Meta) -> String {
    template
        .replace("{title}", meta.title.as_deref().unwrap_or(""))
        .replace("{author}", meta.author.as_deref().unwrap_or(""))
        .replace("{date}", meta.date.as_deref().unwrap_or(""))
}

fn title_block(docx: Docx, doc: &Document, opts: &RenderOptions) -> Docx {
    let Some(title) = doc.resolve_title(opts.title.as_deref()) else {
        return docx;
    };
    if title.trim().is_empty() {
        return docx;
    }
    let mut d = docx.add_paragraph(
        Paragraph::new()
            .style("Title")
            .add_run(Run::new().add_text(title)),
    );
    if let Some(sub) = &doc.meta.subtitle {
        d = d.add_paragraph(
            Paragraph::new()
                .style("Subtitle")
                .add_run(Run::new().add_text(sub.clone())),
        );
    }
    let byline: Vec<&str> = [doc.meta.author.as_deref(), doc.meta.date.as_deref()]
        .into_iter()
        .flatten()
        .collect();
    if !byline.is_empty() {
        d = d.add_paragraph(
            Paragraph::new()
                .style("Byline")
                .add_run(Run::new().add_text(byline.join(" — "))),
        );
    }
    d.add_paragraph(Paragraph::new())
}

fn emit_block(docx: Docx, block: &Block, indent: i32, ctx: &mut Ctx) -> Docx {
    match block {
        Block::Heading { level, content } => {
            let level = (*level).clamp(1, 6);
            let p = add_inlines(
                Paragraph::new().style(&format!("Heading{level}")),
                content,
                Fmt::default(),
                ctx,
            );
            docx.add_paragraph(indented(p, indent))
        }
        Block::Paragraph(content) => {
            let p = add_inlines(Paragraph::new(), content, Fmt::default(), ctx);
            docx.add_paragraph(indented(p, indent))
        }
        Block::CodeBlock { lang, code } => {
            if lang.as_deref() == Some("mermaid") {
                ctx.diags.push(Diagnostic::new(
                    "mermaid diagram rendered as source (diagram support is planned)",
                ));
            }
            let mut run = Run::new();
            for (i, line) in code.trim_end_matches('\n').lines().enumerate() {
                if i > 0 {
                    run = run.add_break(BreakType::TextWrapping);
                }
                run = run.add_text(line);
            }
            docx.add_paragraph(indented(
                Paragraph::new().style("Code").add_run(run),
                indent,
            ))
        }
        Block::BlockQuote(blocks) => {
            let mut d = docx;
            for b in blocks {
                d = emit_quotedish(d, b, indent + INDENT_STEP, "Quote", ctx);
            }
            d
        }
        Block::List { ordered, items } => emit_list(docx, *ordered, items, indent, 0, ctx),
        Block::Table { head, rows, .. } => emit_table(docx, head, rows, ctx),
        Block::ThematicBreak => docx.add_paragraph(
            Paragraph::new()
                .align(AlignmentType::Center)
                .add_run(Run::new().add_text("⸻").color("b4b4b4")),
        ),
        Block::DefinitionList(items) => {
            let mut d = docx;
            for item in items {
                let term = add_inlines(Paragraph::new(), &item.term, Fmt::bold(), ctx);
                d = d.add_paragraph(indented(term, indent));
                for b in &item.details {
                    d = emit_block(d, b, indent + INDENT_STEP, ctx);
                }
            }
            d
        }
        Block::Admonition {
            kind,
            title,
            blocks,
        } => {
            let accent = match kind {
                AdmonitionKind::Note => "0969da",
                AdmonitionKind::Tip => "1a7f37",
                AdmonitionKind::Important => "8250df",
                AdmonitionKind::Warning => "9a6700",
                AdmonitionKind::Caution => "cf222e",
            };
            let mut d = docx.add_paragraph(indented(
                Paragraph::new().add_run(Run::new().add_text(title.clone()).bold().color(accent)),
                indent + INDENT_STEP,
            ));
            for b in blocks {
                d = emit_block(d, b, indent + INDENT_STEP, ctx);
            }
            d
        }
        Block::RawHtml(_) => {
            ctx.diags.push(Diagnostic::new(
                "raw HTML block dropped (disabled for safety)",
            ));
            docx
        }
    }
}

/// Emit a block with a paragraph style applied where possible (blockquotes).
fn emit_quotedish(docx: Docx, block: &Block, indent: i32, style: &str, ctx: &mut Ctx) -> Docx {
    match block {
        Block::Paragraph(content) => {
            let p = add_inlines(Paragraph::new().style(style), content, Fmt::default(), ctx);
            docx.add_paragraph(indented(p, indent))
        }
        other => emit_block(docx, other, indent, ctx),
    }
}

fn emit_list(
    docx: Docx,
    ordered: bool,
    items: &[ListItem],
    indent: i32,
    level: usize,
    ctx: &mut Ctx,
) -> Docx {
    let num_id = if ordered {
        DECIMAL_NUM_ID
    } else {
        BULLET_NUM_ID
    };
    let mut d = docx;
    for item in items {
        let mut first_paragraph_done = false;
        for b in &item.blocks {
            match b {
                Block::Paragraph(content) if !first_paragraph_done => {
                    first_paragraph_done = true;
                    let mut p = Paragraph::new()
                        .numbering(NumberingId::new(num_id), IndentLevel::new(level.min(8)));
                    if let Some(checked) = item.task {
                        let mark = if checked { "☑ " } else { "☐ " };
                        p = p.add_run(Run::new().add_text(mark));
                    }
                    d = d.add_paragraph(add_inlines(p, content, Fmt::default(), ctx));
                }
                Block::List {
                    ordered: nested_ordered,
                    items: nested_items,
                } => {
                    d = emit_list(d, *nested_ordered, nested_items, indent, level + 1, ctx);
                }
                other => {
                    d = emit_block(d, other, indent + INDENT_STEP * (level as i32 + 1), ctx);
                }
            }
        }
    }
    d
}

fn emit_table(docx: Docx, head: &[Vec<Inline>], rows: &[Vec<Vec<Inline>>], ctx: &mut Ctx) -> Docx {
    let mut table_rows = Vec::new();
    if !head.is_empty() {
        let cells = head
            .iter()
            .map(|cell| {
                TableCell::new().add_paragraph(add_inlines(
                    Paragraph::new(),
                    cell,
                    Fmt::bold(),
                    ctx,
                ))
            })
            .collect();
        table_rows.push(TableRow::new(cells));
    }
    for row in rows {
        let cells = row
            .iter()
            .map(|cell| {
                TableCell::new().add_paragraph(add_inlines(
                    Paragraph::new(),
                    cell,
                    Fmt::default(),
                    ctx,
                ))
            })
            .collect();
        table_rows.push(TableRow::new(cells));
    }
    docx.add_table(Table::new(table_rows))
        .add_paragraph(Paragraph::new())
}

fn indented(p: Paragraph, indent: i32) -> Paragraph {
    if indent > 0 {
        p.indent(Some(indent), None, None, None)
    } else {
        p
    }
}

/// Inline formatting state, accumulated while descending nested inlines.
#[derive(Clone, Copy, Default)]
struct Fmt {
    bold: bool,
    italic: bool,
    strike: bool,
    code: bool,
}

impl Fmt {
    fn bold() -> Fmt {
        Fmt {
            bold: true,
            ..Fmt::default()
        }
    }
}

fn styled_run(text: &str, fmt: Fmt) -> Run {
    let mut r = Run::new().add_text(text);
    if fmt.bold {
        r = r.bold();
    }
    if fmt.italic {
        r = r.italic();
    }
    if fmt.strike {
        r = r.strike();
    }
    if fmt.code {
        r = r.style("CodeChar");
    }
    r
}

fn add_inlines(mut p: Paragraph, inlines: &[Inline], fmt: Fmt, ctx: &mut Ctx) -> Paragraph {
    for inline in inlines {
        p = add_inline(p, inline, fmt, ctx);
    }
    p
}

fn add_inline(p: Paragraph, inline: &Inline, fmt: Fmt, ctx: &mut Ctx) -> Paragraph {
    match inline {
        Inline::Text(t) => p.add_run(styled_run(t, fmt)),
        Inline::SoftBreak => p.add_run(styled_run(" ", fmt)),
        Inline::LineBreak => p.add_run(Run::new().add_break(BreakType::TextWrapping)),
        Inline::Code(c) => p.add_run(styled_run(c, Fmt { code: true, ..fmt })),
        Inline::Emph(c) => add_inlines(
            p,
            c,
            Fmt {
                italic: true,
                ..fmt
            },
            ctx,
        ),
        Inline::Strong(c) => add_inlines(p, c, Fmt { bold: true, ..fmt }, ctx),
        Inline::Strikethrough(c) => add_inlines(
            p,
            c,
            Fmt {
                strike: true,
                ..fmt
            },
            ctx,
        ),
        // Superscript formatting is not exposed by the writer; plain text.
        Inline::Superscript(c) => add_inlines(p, c, fmt, ctx),
        Inline::Link { href, content } => {
            let text = crate::ir::inline_text(content);
            p.add_hyperlink(
                Hyperlink::new(href, HyperlinkType::External).add_run(
                    styled_run(&text, fmt).color(color(&ctx.opts.theme.spec().link_color)),
                ),
            )
        }
        Inline::Image { src, alt, width } => match ctx.opts.security.resolve_image(src) {
            AssetDecision::Allow(path) => {
                let full = ctx.opts.security.root.join(&path);
                match std::fs::read(&full) {
                    Ok(bytes) => {
                        if width.is_some() {
                            ctx.diags.push(Diagnostic::new(format!(
                                "image width ignored in docx output: {src}"
                            )));
                        }
                        p.add_run(Run::new().add_image(Pic::new(&bytes)))
                    }
                    Err(e) => {
                        ctx.diags
                            .push(Diagnostic::new(format!("cannot read image {src}: {e}")));
                        p.add_run(styled_run(alt, Fmt::default()))
                    }
                }
            }
            AssetDecision::Deny(reason) => {
                ctx.diags.push(Diagnostic::new(reason));
                p.add_run(styled_run(
                    alt,
                    Fmt {
                        italic: true,
                        ..Fmt::default()
                    },
                ))
            }
        },
        Inline::Footnote(blocks) => {
            ctx.footnote_seq += 1;
            let mut note = Footnote {
                id: ctx.footnote_seq,
                ..Footnote::default()
            };
            // Footnote bodies are flattened to text paragraphs.
            for b in blocks {
                note =
                    note.add_content(Paragraph::new().add_run(Run::new().add_text(block_text(b))));
            }
            p.add_run(Run::new().add_footnote_reference(note))
        }
    }
}

/// Flatten a block to plain text (footnote bodies).
fn block_text(block: &Block) -> String {
    match block {
        Block::Paragraph(c) | Block::Heading { content: c, .. } => crate::ir::inline_text(c),
        Block::CodeBlock { code, .. } => code.clone(),
        Block::BlockQuote(blocks) | Block::Admonition { blocks, .. } => {
            blocks.iter().map(block_text).collect::<Vec<_>>().join(" ")
        }
        Block::List { items, .. } => items
            .iter()
            .flat_map(|i| i.blocks.iter().map(block_text))
            .collect::<Vec<_>>()
            .join(" "),
        Block::DefinitionList(items) => items
            .iter()
            .map(|i| crate::ir::inline_text(&i.term))
            .collect::<Vec<_>>()
            .join(" "),
        Block::Table { .. } | Block::ThematicBreak | Block::RawHtml(_) => String::new(),
    }
}
