//! Markdown → [`Document`] lowering. This is the only place comrak is used;
//! everything downstream operates on the renderer-agnostic [`crate::ir`].

use std::collections::HashMap;

use comrak::nodes::{AstNode, NodeValue, TableAlignment};
use comrak::{Arena, Options, parse_document};

use crate::ir::{Align, Block, Document, Inline, ListItem, Meta};

type Node<'a> = &'a AstNode<'a>;

/// Parse a Markdown string (optionally with a leading YAML frontmatter block)
/// into a [`Document`].
pub fn parse(input: &str) -> Document {
    let (meta, body) = split_frontmatter(input);

    let arena = Arena::new();
    let root = parse_document(&arena, body, &options());

    let footnotes = collect_footnotes(root);
    let blocks = root
        .children()
        .filter_map(|n| block(n, &footnotes))
        .collect();

    Document { meta, blocks }
}

fn options() -> Options<'static> {
    let mut o = Options::default();
    o.extension.table = true;
    o.extension.strikethrough = true;
    o.extension.tasklist = true;
    o.extension.footnotes = true;
    o.extension.autolink = true;
    o.extension.superscript = true;
    o
}

/// Split a leading `---\n ... \n---` frontmatter block from the body.
///
/// We keep this deliberately small (flat `key: value` pairs) to avoid pulling in
/// a full YAML engine; nested structures are out of scope and ignored.
fn split_frontmatter(input: &str) -> (Meta, &str) {
    let mut meta = Meta::default();
    let trimmed = input.strip_prefix('\u{feff}').unwrap_or(input);
    let Some(rest) = trimmed
        .strip_prefix("---\n")
        .or_else(|| trimmed.strip_prefix("---\r\n"))
    else {
        return (meta, input);
    };
    // Find the closing fence at the start of a line.
    let Some(end) = find_fence(rest) else {
        return (meta, input);
    };
    let (front, after) = rest.split_at(end);
    for line in front.lines() {
        let line = line.trim_end();
        if line.trim().is_empty() {
            continue;
        }
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        let key = key.trim().to_ascii_lowercase();
        let value = unquote(value.trim());
        match key.as_str() {
            "title" => meta.title = Some(value),
            "author" | "authors" => meta.author = Some(value),
            "date" => meta.date = Some(value),
            "subtitle" => meta.subtitle = Some(value),
            _ => {
                meta.extra.insert(key, value);
            }
        }
    }
    // Skip past the closing fence line.
    let body = after
        .trim_start_matches("---")
        .trim_start_matches(['\r', '\n']);
    (meta, body)
}

fn find_fence(s: &str) -> Option<usize> {
    let mut offset = 0;
    for line in s.split_inclusive('\n') {
        if line.trim_end() == "---" {
            return Some(offset);
        }
        offset += line.len();
    }
    None
}

fn unquote(s: &str) -> String {
    let bytes = s.as_bytes();
    if bytes.len() >= 2
        && ((bytes[0] == b'"' && bytes[bytes.len() - 1] == b'"')
            || (bytes[0] == b'\'' && bytes[bytes.len() - 1] == b'\''))
    {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}

fn collect_footnotes<'a>(root: Node<'a>) -> HashMap<String, Node<'a>> {
    let mut map = HashMap::new();
    for node in root.descendants() {
        if let NodeValue::FootnoteDefinition(def) = &node.data.borrow().value {
            map.insert(def.name.clone(), node);
        }
    }
    map
}

fn block<'a>(node: Node<'a>, fns: &HashMap<String, Node<'a>>) -> Option<Block> {
    let value = node.data.borrow().value.clone();
    match value {
        NodeValue::Paragraph => Some(Block::Paragraph(inlines(node, fns))),
        NodeValue::Heading(h) => Some(Block::Heading {
            level: h.level,
            content: inlines(node, fns),
        }),
        NodeValue::CodeBlock(c) => {
            let lang = c.info.split_whitespace().next().filter(|s| !s.is_empty());
            Some(Block::CodeBlock {
                lang: lang.map(str::to_string),
                code: c.literal,
            })
        }
        NodeValue::BlockQuote => Some(Block::BlockQuote(
            node.children().filter_map(|n| block(n, fns)).collect(),
        )),
        NodeValue::List(l) => Some(Block::List {
            ordered: l.list_type == comrak::nodes::ListType::Ordered,
            items: node.children().map(|item| list_item(item, fns)).collect(),
        }),
        NodeValue::Table(t) => Some(table(node, &t.alignments, fns)),
        NodeValue::ThematicBreak => Some(Block::ThematicBreak),
        NodeValue::HtmlBlock(h) => Some(Block::RawHtml(h.literal)),
        // Footnote definitions are inlined at their references; descend into
        // any other container we don't model explicitly.
        NodeValue::FootnoteDefinition(_) => None,
        _ => {
            let mut children: Vec<Block> = node.children().filter_map(|n| block(n, fns)).collect();
            match children.len() {
                0 => None,
                1 => children.pop(),
                _ => Some(Block::BlockQuote(children)), // safe structural fallback
            }
        }
    }
}

fn list_item<'a>(item: Node<'a>, fns: &HashMap<String, Node<'a>>) -> ListItem {
    let task = match &item.data.borrow().value {
        NodeValue::TaskItem(t) => Some(t.symbol.is_some()),
        _ => None,
    };
    ListItem {
        task,
        blocks: item.children().filter_map(|n| block(n, fns)).collect(),
    }
}

fn table<'a>(node: Node<'a>, aligns: &[TableAlignment], fns: &HashMap<String, Node<'a>>) -> Block {
    let align = aligns
        .iter()
        .map(|a| match a {
            TableAlignment::Left => Align::Left,
            TableAlignment::Center => Align::Center,
            TableAlignment::Right => Align::Right,
            TableAlignment::None => Align::None,
        })
        .collect();

    let mut head = Vec::new();
    let mut rows = Vec::new();
    for row in node.children() {
        let is_header = matches!(&row.data.borrow().value, NodeValue::TableRow(true));
        let cells: Vec<Vec<Inline>> = row.children().map(|c| inlines(c, fns)).collect();
        if is_header {
            head = cells;
        } else {
            rows.push(cells);
        }
    }
    Block::Table { align, head, rows }
}

fn inlines<'a>(node: Node<'a>, fns: &HashMap<String, Node<'a>>) -> Vec<Inline> {
    let mut out = Vec::new();
    for child in node.children() {
        inline(child, fns, &mut out);
    }
    out
}

fn inline<'a>(node: Node<'a>, fns: &HashMap<String, Node<'a>>, out: &mut Vec<Inline>) {
    let value = node.data.borrow().value.clone();
    match value {
        NodeValue::Text(t) => out.push(Inline::Text(t.to_string())),
        NodeValue::SoftBreak => out.push(Inline::SoftBreak),
        NodeValue::LineBreak => out.push(Inline::LineBreak),
        NodeValue::Code(c) => out.push(Inline::Code(c.literal)),
        NodeValue::Emph => out.push(Inline::Emph(inlines(node, fns))),
        NodeValue::Strong => out.push(Inline::Strong(inlines(node, fns))),
        NodeValue::Strikethrough => out.push(Inline::Strikethrough(inlines(node, fns))),
        NodeValue::Superscript => out.push(Inline::Superscript(inlines(node, fns))),
        NodeValue::Link(l) => out.push(Inline::Link {
            href: l.url,
            content: inlines(node, fns),
        }),
        NodeValue::Image(l) => out.push(Inline::Image {
            src: l.url,
            alt: crate::ir::inline_text(&inlines(node, fns)),
        }),
        NodeValue::FootnoteReference(r) => {
            if let Some(def) = fns.get(&r.name) {
                let blocks = def.children().filter_map(|n| block(n, fns)).collect();
                out.push(Inline::Footnote(blocks));
            }
        }
        NodeValue::HtmlInline(_) => {}
        // Unknown inline containers: flatten their children.
        _ => {
            for child in node.children() {
                inline(child, fns, out);
            }
        }
    }
}
