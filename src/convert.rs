//! Converts a parsed Markdown AST into Typst markup.
//!
//! Every literal piece of text is emitted as a Typst string (`#"..."`) and all
//! structure is expressed through Typst functions (`#strong`, `#heading`,
//! `#table`, ...). Going through strings means the only characters we ever have
//! to escape are the ones special inside a Typst string literal, which sidesteps
//! the much larger set of characters that are significant in Typst markup.

use std::collections::HashMap;
use std::fmt::Write;

use comrak::nodes::{AstNode, NodeValue, TableAlignment};

type Node<'a> = &'a AstNode<'a>;

/// Document-wide context gathered before rendering.
struct Ctx<'a> {
    /// Footnote definitions keyed by their label, resolved lazily at each
    /// reference site so the note text lands inline where Typst expects it.
    footnotes: HashMap<String, Node<'a>>,
}

/// Render the body of a document (everything below the preamble/title).
pub fn render_body<'a>(root: Node<'a>) -> String {
    let mut ctx = Ctx { footnotes: HashMap::new() };
    collect_footnotes(root, &mut ctx);

    let mut out = String::new();
    for child in root.children() {
        block(child, &mut out, &ctx);
    }
    out
}

/// Pull the text of the first heading, used as a fallback document title.
pub fn first_heading_text<'a>(root: Node<'a>) -> Option<String> {
    for node in root.descendants() {
        if matches!(node.data.borrow().value, NodeValue::Heading(_)) {
            let text = plain_text(node);
            if !text.trim().is_empty() {
                return Some(text);
            }
        }
    }
    None
}

fn collect_footnotes<'a>(root: Node<'a>, ctx: &mut Ctx<'a>) {
    for node in root.descendants() {
        if let NodeValue::FootnoteDefinition(def) = &node.data.borrow().value {
            ctx.footnotes.insert(def.name.clone(), node);
        }
    }
}

fn block<'a>(node: Node<'a>, out: &mut String, ctx: &Ctx<'a>) {
    let ast = node.data.borrow();
    match &ast.value {
        NodeValue::Paragraph => {
            inline_children(node, out, ctx);
            out.push_str("\n\n");
        }
        NodeValue::Heading(h) => {
            let _ = write!(out, "#heading(level: {})[", h.level);
            inline_children(node, out, ctx);
            out.push_str("]\n\n");
        }
        NodeValue::CodeBlock(code) => {
            let lang = code.info.split_whitespace().next().unwrap_or("");
            out.push_str("#raw(block: true");
            if !lang.is_empty() {
                let _ = write!(out, ", lang: \"{}\"", esc(lang));
            }
            let _ = write!(out, ", \"{}\")\n\n", esc(&code.literal));
        }
        NodeValue::BlockQuote => {
            out.push_str("#quote(block: true)[\n");
            for child in node.children() {
                block(child, out, ctx);
            }
            out.push_str("]\n\n");
        }
        NodeValue::List(_) => {
            list(node, out, ctx);
        }
        NodeValue::Table(table) => {
            render_table(node, &table.alignments, out, ctx);
        }
        NodeValue::ThematicBreak => {
            out.push_str("#line(length: 100%, stroke: 0.5pt + luma(180))\n#v(0.3em)\n\n");
        }
        // Definitions are rendered where they are referenced; raw HTML is dropped.
        NodeValue::FootnoteDefinition(_) | NodeValue::HtmlBlock(_) => {}
        // Anything else (documents, description lists, ...) we descend into.
        _ => {
            for child in node.children() {
                block(child, out, ctx);
            }
        }
    }
}

fn list<'a>(node: Node<'a>, out: &mut String, ctx: &Ctx<'a>) {
    let ast = node.data.borrow();
    let ordered = matches!(
        &ast.value,
        NodeValue::List(l) if l.list_type == comrak::nodes::ListType::Ordered
    );
    out.push_str(if ordered { "#enum(\n" } else { "#list(\n" });
    for item in node.children() {
        // A task-list checkbox renders as a leading symbol on the item.
        if let NodeValue::TaskItem(state) = &item.data.borrow().value {
            let mark = if state.symbol.is_some() { "☑ " } else { "☐ " };
            let _ = write!(out, "  [#\"{}\"", esc(mark));
            item_body(item, out, ctx);
            out.push_str("],\n");
        } else {
            out.push_str("  [");
            item_body(item, out, ctx);
            out.push_str("],\n");
        }
    }
    out.push_str(")\n\n");
}

/// Render the contents of a list item without the wrapping `[` `]`.
fn item_body<'a>(item: Node<'a>, out: &mut String, ctx: &Ctx<'a>) {
    for child in item.children() {
        match &child.data.borrow().value {
            // Keep simple items tight: emit the paragraph's inline run directly.
            NodeValue::Paragraph => inline_children(child, out, ctx),
            _ => block(child, out, ctx),
        }
    }
}

fn render_table<'a>(
    node: Node<'a>,
    alignments: &[TableAlignment],
    out: &mut String,
    ctx: &Ctx<'a>,
) {
    let rows: Vec<Node<'a>> = node.children().collect();
    let columns = rows
        .first()
        .map(|r| r.children().count())
        .unwrap_or(alignments.len().max(1));

    out.push_str("#table(\n");
    let _ = write!(out, "  columns: {},\n", columns);
    out.push_str("  align: (");
    for i in 0..columns {
        let a = alignments.get(i).copied().unwrap_or(TableAlignment::None);
        out.push_str(match a {
            TableAlignment::Center => "center",
            TableAlignment::Right => "right",
            TableAlignment::Left | TableAlignment::None => "left",
        });
        out.push_str(", ");
    }
    out.push_str("),\n");
    out.push_str("  stroke: 0.5pt + luma(190),\n");

    for row in rows {
        let header = matches!(&row.data.borrow().value, NodeValue::TableRow(true));
        if header {
            out.push_str("  table.header(");
        } else {
            out.push_str("  ");
        }
        for cell in row.children() {
            out.push('[');
            inline_children(cell, out, ctx);
            out.push_str("], ");
        }
        if header {
            out.push_str("),\n");
        } else {
            out.push('\n');
        }
    }
    out.push_str(")\n\n");
}

fn inline_children<'a>(node: Node<'a>, out: &mut String, ctx: &Ctx<'a>) {
    for child in node.children() {
        inline(child, out, ctx);
    }
}

fn inline<'a>(node: Node<'a>, out: &mut String, ctx: &Ctx<'a>) {
    let ast = node.data.borrow();
    match &ast.value {
        NodeValue::Text(t) => {
            let _ = write!(out, "#\"{}\"", esc(t));
        }
        NodeValue::SoftBreak => out.push_str("#\" \""),
        NodeValue::LineBreak => out.push_str("#linebreak()"),
        NodeValue::Code(code) => {
            let _ = write!(out, "#raw(\"{}\")", esc(&code.literal));
        }
        NodeValue::Emph => wrap(node, out, ctx, "#emph["),
        NodeValue::Strong => wrap(node, out, ctx, "#strong["),
        NodeValue::Strikethrough => wrap(node, out, ctx, "#strike["),
        NodeValue::Superscript => wrap(node, out, ctx, "#super["),
        NodeValue::Link(link) => {
            let _ = write!(out, "#link(\"{}\")[", esc(&link.url));
            inline_children(node, out, ctx);
            out.push(']');
        }
        NodeValue::Image(link) => image(node, &link.url, out, ctx),
        NodeValue::FootnoteReference(r) => {
            if let Some(def) = ctx.footnotes.get(&r.name) {
                out.push_str("#footnote[");
                for child in def.children() {
                    block(child, out, ctx);
                }
                out.push(']');
            }
        }
        // Raw inline HTML is skipped; unknown inlines fall back to their children.
        NodeValue::HtmlInline(_) => {}
        _ => inline_children(node, out, ctx),
    }
}

fn wrap<'a>(node: Node<'a>, out: &mut String, ctx: &Ctx<'a>, open: &str) {
    out.push_str(open);
    inline_children(node, out, ctx);
    out.push(']');
}

fn image<'a>(node: Node<'a>, url: &str, out: &mut String, ctx: &Ctx<'a>) {
    // Typst can only embed local files; for remote images fall back to alt text.
    if url.starts_with("http://") || url.starts_with("https://") {
        inline_children(node, out, ctx);
    } else {
        let _ = write!(out, "#image(\"{}\")", esc(url));
    }
}

/// Concatenate the plain text of a node's inline descendants.
fn plain_text<'a>(node: Node<'a>) -> String {
    let mut s = String::new();
    for d in node.descendants() {
        match &d.data.borrow().value {
            NodeValue::Text(t) => s.push_str(t),
            NodeValue::Code(c) => s.push_str(&c.literal),
            _ => {}
        }
    }
    s
}

/// Escape a string for inclusion inside a Typst `"..."` literal.
fn esc(s: &str) -> String {
    let mut o = String::with_capacity(s.len() + 2);
    for c in s.chars() {
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
