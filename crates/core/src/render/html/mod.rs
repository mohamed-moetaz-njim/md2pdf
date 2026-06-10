//! The HTML back-end: lowers the [`Document`] IR to a single, self-contained
//! HTML page with the theme's typography inlined as CSS.
//!
//! It mirrors the Typst back-end's security posture: raw HTML from the source
//! document is dropped, remote images are never referenced, and local images
//! must pass the [`crate::security::SecurityPolicy`] checks.

use std::collections::BTreeMap;
use std::fmt::Write;

use crate::ir::{AdmonitionKind, Align, Block, Document, Inline, ListItem};
use crate::render::{Diagnostic, OutputFormat, RenderOptions, Rendered, Renderer};
use crate::security::AssetDecision;
use crate::theme::ThemeSpec;

/// Renders a [`Document`] to a standalone HTML page.
pub struct HtmlRenderer;

impl Renderer for HtmlRenderer {
    fn format(&self) -> OutputFormat {
        OutputFormat::Html
    }

    fn render(&self, doc: &Document, opts: &RenderOptions) -> anyhow::Result<Rendered> {
        let mut diagnostics = Vec::new();
        let html = lower(doc, opts, &mut diagnostics);
        Ok(Rendered {
            bytes: html.into_bytes(),
            diagnostics,
        })
    }
}

struct Ctx<'a> {
    opts: &'a RenderOptions,
    diags: &'a mut Vec<Diagnostic>,
    /// Footnote bodies in encounter order; rendered as a trailing section.
    footnotes: Vec<Vec<Block>>,
    /// Heading slug -> times seen, for unique anchors.
    slugs: BTreeMap<String, usize>,
}

pub fn lower(doc: &Document, opts: &RenderOptions, diags: &mut Vec<Diagnostic>) -> String {
    let spec = opts.theme.spec();
    let title = doc.resolve_title(opts.title.as_deref());

    let mut ctx = Ctx {
        opts,
        diags,
        footnotes: Vec::new(),
        slugs: BTreeMap::new(),
    };

    let mut body = String::new();
    title_block(&mut body, doc, title.as_deref());
    if opts.toc {
        toc(&mut body, doc);
    }
    for block in &doc.blocks {
        emit_block(block, &mut body, &mut ctx);
    }
    footnote_section(&mut body, &mut ctx);

    let mut s = String::with_capacity(body.len() + 2048);
    s.push_str("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n");
    s.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n");
    let _ = writeln!(
        s,
        "<title>{}</title>",
        esc(title.as_deref().unwrap_or("Document"))
    );
    s.push_str("<style>\n");
    s.push_str(&css(&spec));
    s.push_str("</style>\n</head>\n<body>\n<main>\n");
    s.push_str(&body);
    s.push_str("</main>\n</body>\n</html>\n");
    s
}

fn css(spec: &ThemeSpec) -> String {
    let justify = if spec.justify { "justify" } else { "left" };
    let code_border = match &spec.code_stroke {
        Some(c) => format!("border: 1px solid {c};"),
        None => String::new(),
    };
    format!(
        r#"body {{ margin: 0; background: #fff; color: #1a1a1a; }}
main {{ max-width: 46em; margin: 0 auto; padding: 3em 1.5em;
  font-family: "{body_font}", serif; font-size: {body_size}pt;
  line-height: 1.55; text-align: {justify}; }}
h1, h2, h3, h4, h5, h6 {{ color: {heading}; line-height: 1.25; text-align: left; }}
a {{ color: {link}; }}
header.title-block {{ text-align: center; margin-bottom: 2.5em; }}
header.title-block h1 {{ color: {accent}; margin-bottom: 0.2em; }}
header.title-block .subtitle {{ font-size: 1.1em; margin: 0.2em 0; }}
header.title-block .byline {{ color: #5a5a5a; font-size: 0.85em; }}
header.title-block hr {{ width: 38%; border: 0; border-top: 1px solid {accent}; }}
nav.toc {{ border: 1px solid #ddd; border-radius: 4px; padding: 0.8em 1.2em;
  margin-bottom: 2em; }}
nav.toc ul {{ list-style: none; padding-left: 0; margin: 0.3em 0; }}
nav.toc li {{ margin: 0.15em 0; }}
nav.toc li.lvl-2 {{ padding-left: 1.2em; }}
nav.toc li.lvl-3 {{ padding-left: 2.4em; }}
nav.toc li.lvl-4, nav.toc li.lvl-5, nav.toc li.lvl-6 {{ padding-left: 3.6em; }}
pre {{ background: {code_fill}; {code_border} border-radius: 4px;
  padding: 10px; overflow-x: auto; text-align: left; }}
code {{ font-family: "{mono_font}", monospace; font-size: 0.88em; }}
:not(pre) > code {{ background: {code_fill}; border-radius: 3px; padding: 0.1em 0.3em; }}
blockquote {{ border-left: 2px solid {accent}; margin-left: 0;
  padding: 0.4em 0 0.4em 1em; color: #444; }}
table {{ border-collapse: collapse; margin: 1em 0; }}
th, td {{ border: 1px solid #bbb; padding: 0.35em 0.7em; }}
th {{ background: #f5f6f7; }}
img {{ max-width: 100%; }}
hr {{ border: 0; border-top: 1px solid #b4b4b4; }}
.admonition {{ border-left: 3px solid; border-radius: 3px; padding: 10px 14px;
  margin: 1em 0; }}
.admonition-title {{ font-weight: bold; margin: 0 0 0.4em 0; }}
.admonition.note {{ border-color: #0969da; background: #f0f6ff; }}
.admonition.note .admonition-title {{ color: #0969da; }}
.admonition.tip {{ border-color: #1a7f37; background: #effaf1; }}
.admonition.tip .admonition-title {{ color: #1a7f37; }}
.admonition.important {{ border-color: #8250df; background: #f6f0fe; }}
.admonition.important .admonition-title {{ color: #8250df; }}
.admonition.warning {{ border-color: #9a6700; background: #fff8e5; }}
.admonition.warning .admonition-title {{ color: #9a6700; }}
.admonition.caution {{ border-color: #cf222e; background: #ffefef; }}
.admonition.caution .admonition-title {{ color: #cf222e; }}
section.footnotes {{ margin-top: 2.5em; border-top: 1px solid #ccc;
  font-size: 0.88em; }}
sup.footnote-ref a {{ text-decoration: none; }}
"#,
        body_font = spec.body_font,
        body_size = spec.body_size_pt,
        heading = spec.heading_color,
        link = spec.link_color,
        accent = spec.accent_color,
        code_fill = spec.code_fill,
        mono_font = spec.mono_font,
    )
}

fn title_block(s: &mut String, doc: &Document, title: Option<&str>) {
    let Some(title) = title else { return };
    if title.trim().is_empty() {
        return;
    }
    s.push_str("<header class=\"title-block\">\n");
    let _ = writeln!(s, "<h1>{}</h1>", esc(title));
    if let Some(sub) = &doc.meta.subtitle {
        let _ = writeln!(s, "<p class=\"subtitle\">{}</p>", esc(sub));
    }
    let byline: Vec<String> = [doc.meta.author.as_deref(), doc.meta.date.as_deref()]
        .into_iter()
        .flatten()
        .map(esc)
        .collect();
    if !byline.is_empty() {
        let _ = writeln!(s, "<p class=\"byline\">{}</p>", byline.join(" — "));
    }
    s.push_str("<hr>\n</header>\n");
}

fn toc(s: &mut String, doc: &Document) {
    // Build anchors with the same slug sequence emit_block will generate.
    let mut slugs: BTreeMap<String, usize> = BTreeMap::new();
    let entries: Vec<(u8, String, String)> = doc
        .blocks
        .iter()
        .filter_map(|b| match b {
            Block::Heading { level, content } => {
                let text = crate::ir::inline_text(content);
                let anchor = unique_slug(&text, &mut slugs);
                Some((*level, text, anchor))
            }
            _ => None,
        })
        .collect();
    if entries.is_empty() {
        return;
    }
    s.push_str("<nav class=\"toc\">\n<strong>Contents</strong>\n<ul>\n");
    for (level, text, anchor) in entries {
        let _ = writeln!(
            s,
            "<li class=\"lvl-{level}\"><a href=\"#{anchor}\">{}</a></li>",
            esc(&text)
        );
    }
    s.push_str("</ul>\n</nav>\n");
}

fn emit_block(block: &Block, s: &mut String, ctx: &mut Ctx) {
    match block {
        Block::Heading { level, content } => {
            let level = (*level).clamp(1, 6);
            let text = crate::ir::inline_text(content);
            let anchor = unique_slug(&text, &mut ctx.slugs);
            let _ = write!(s, "<h{level} id=\"{anchor}\">");
            emit_inlines(content, s, ctx);
            let _ = writeln!(s, "</h{level}>");
        }
        Block::Paragraph(content) => {
            s.push_str("<p>");
            emit_inlines(content, s, ctx);
            s.push_str("</p>\n");
        }
        Block::CodeBlock { lang, code } => {
            if lang.as_deref() == Some("mermaid") {
                ctx.diags.push(Diagnostic::new(
                    "mermaid diagram rendered as source (diagram support is planned)",
                ));
            }
            match lang {
                Some(lang) => {
                    let _ = write!(s, "<pre><code class=\"language-{}\">", esc(lang));
                }
                None => s.push_str("<pre><code>"),
            }
            s.push_str(&esc(code));
            s.push_str("</code></pre>\n");
        }
        Block::BlockQuote(blocks) => {
            s.push_str("<blockquote>\n");
            for b in blocks {
                emit_block(b, s, ctx);
            }
            s.push_str("</blockquote>\n");
        }
        Block::List { ordered, items } => emit_list(*ordered, items, s, ctx),
        Block::Table { align, head, rows } => emit_table(align, head, rows, s, ctx),
        Block::ThematicBreak => s.push_str("<hr>\n"),
        Block::Admonition {
            kind,
            title,
            blocks,
        } => {
            let class = match kind {
                AdmonitionKind::Note => "note",
                AdmonitionKind::Tip => "tip",
                AdmonitionKind::Important => "important",
                AdmonitionKind::Warning => "warning",
                AdmonitionKind::Caution => "caution",
            };
            let _ = writeln!(
                s,
                "<div class=\"admonition {class}\">\n<p class=\"admonition-title\">{}</p>",
                esc(title)
            );
            for b in blocks {
                emit_block(b, s, ctx);
            }
            s.push_str("</div>\n");
        }
        Block::RawHtml(_) => {
            ctx.diags.push(Diagnostic::new(
                "raw HTML block dropped (disabled for safety)",
            ));
        }
    }
}

fn emit_list(ordered: bool, items: &[ListItem], s: &mut String, ctx: &mut Ctx) {
    let tag = if ordered { "ol" } else { "ul" };
    let _ = writeln!(s, "<{tag}>");
    for item in items {
        s.push_str("<li>");
        if let Some(checked) = item.task {
            let checked = if checked { " checked" } else { "" };
            let _ = write!(s, "<input type=\"checkbox\" disabled{checked}> ");
        }
        for (i, b) in item.blocks.iter().enumerate() {
            match b {
                // Keep single-paragraph items tight.
                Block::Paragraph(content) if item.blocks.len() == 1 => {
                    emit_inlines(content, s, ctx)
                }
                other => {
                    if i > 0 {
                        s.push('\n');
                    }
                    emit_block(other, s, ctx);
                }
            }
        }
        s.push_str("</li>\n");
    }
    let _ = writeln!(s, "</{tag}>");
}

fn emit_table(
    align: &[Align],
    head: &[Vec<Inline>],
    rows: &[Vec<Vec<Inline>>],
    s: &mut String,
    ctx: &mut Ctx,
) {
    let style = |i: usize| match align.get(i).copied().unwrap_or(Align::None) {
        Align::Center => " style=\"text-align: center\"",
        Align::Right => " style=\"text-align: right\"",
        Align::Left | Align::None => "",
    };
    s.push_str("<table>\n");
    if !head.is_empty() {
        s.push_str("<thead>\n<tr>");
        for (i, cell) in head.iter().enumerate() {
            let _ = write!(s, "<th{}>", style(i));
            emit_inlines(cell, s, ctx);
            s.push_str("</th>");
        }
        s.push_str("</tr>\n</thead>\n");
    }
    s.push_str("<tbody>\n");
    for row in rows {
        s.push_str("<tr>");
        for (i, cell) in row.iter().enumerate() {
            let _ = write!(s, "<td{}>", style(i));
            emit_inlines(cell, s, ctx);
            s.push_str("</td>");
        }
        s.push_str("</tr>\n");
    }
    s.push_str("</tbody>\n</table>\n");
}

fn emit_inlines(inlines: &[Inline], s: &mut String, ctx: &mut Ctx) {
    for inline in inlines {
        emit_inline(inline, s, ctx);
    }
}

fn emit_inline(inline: &Inline, s: &mut String, ctx: &mut Ctx) {
    match inline {
        Inline::Text(t) => s.push_str(&esc(t)),
        Inline::SoftBreak => s.push('\n'),
        Inline::LineBreak => s.push_str("<br>\n"),
        Inline::Code(c) => {
            let _ = write!(s, "<code>{}</code>", esc(c));
        }
        Inline::Emph(c) => wrap("em", c, s, ctx),
        Inline::Strong(c) => wrap("strong", c, s, ctx),
        Inline::Strikethrough(c) => wrap("del", c, s, ctx),
        Inline::Superscript(c) => wrap("sup", c, s, ctx),
        Inline::Link { href, content } => {
            let _ = write!(s, "<a href=\"{}\">", esc(href));
            emit_inlines(content, s, ctx);
            s.push_str("</a>");
        }
        Inline::Image { src, alt, width } => emit_image(src, alt, width.as_deref(), s, ctx),
        Inline::Footnote(blocks) => {
            ctx.footnotes.push(blocks.clone());
            let n = ctx.footnotes.len();
            let _ = write!(
                s,
                "<sup class=\"footnote-ref\"><a href=\"#fn{n}\" id=\"fnref{n}\">[{n}]</a></sup>"
            );
        }
    }
}

fn wrap(tag: &str, content: &[Inline], s: &mut String, ctx: &mut Ctx) {
    let _ = write!(s, "<{tag}>");
    emit_inlines(content, s, ctx);
    let _ = write!(s, "</{tag}>");
}

fn emit_image(src: &str, alt: &str, width: Option<&str>, s: &mut String, ctx: &mut Ctx) {
    match ctx.opts.security.resolve_image(src) {
        AssetDecision::Allow(path) => {
            let _ = write!(s, "<img src=\"{}\" alt=\"{}\"", esc(&path), esc(alt));
            if let Some(w) = width {
                // Parser-validated dimension; CSS understands all of them but
                // bare `%` widths, which map to the width attribute instead.
                if let Some(pct) = w.strip_suffix('%') {
                    let _ = write!(s, " style=\"width: {pct}%\"");
                } else {
                    let _ = write!(s, " style=\"width: {w}\"");
                }
            }
            s.push('>');
        }
        AssetDecision::Deny(reason) => {
            ctx.diags.push(Diagnostic::new(reason));
            let _ = write!(s, "<em>{}</em>", esc(alt));
        }
    }
}

fn footnote_section(s: &mut String, ctx: &mut Ctx) {
    if ctx.footnotes.is_empty() {
        return;
    }
    s.push_str("<section class=\"footnotes\">\n<ol>\n");
    let footnotes = std::mem::take(&mut ctx.footnotes);
    for (i, blocks) in footnotes.iter().enumerate() {
        let _ = writeln!(s, "<li id=\"fn{}\">", i + 1);
        for b in blocks {
            emit_block(b, s, ctx);
        }
        s.push_str("</li>\n");
    }
    s.push_str("</ol>\n</section>\n");
}

/// Slugify heading text for anchors, keeping duplicates unique.
fn unique_slug(text: &str, seen: &mut BTreeMap<String, usize>) -> String {
    let mut slug: String = text
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect();
    while slug.contains("--") {
        slug = slug.replace("--", "-");
    }
    let slug = slug.trim_matches('-').to_string();
    let slug = if slug.is_empty() {
        "section".to_string()
    } else {
        slug
    };
    let n = seen.entry(slug.clone()).or_insert(0);
    *n += 1;
    if *n == 1 { slug } else { format!("{slug}-{n}") }
}

/// Escape text for HTML element and attribute content.
fn esc(input: &str) -> String {
    let mut o = String::with_capacity(input.len());
    for c in input.chars() {
        match c {
            '&' => o.push_str("&amp;"),
            '<' => o.push_str("&lt;"),
            '>' => o.push_str("&gt;"),
            '"' => o.push_str("&quot;"),
            '\'' => o.push_str("&#39;"),
            _ => o.push(c),
        }
    }
    o
}
