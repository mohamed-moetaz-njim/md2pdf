//! Markdown → [`Document`] lowering. This is the only place comrak is used;
//! everything downstream operates on the renderer-agnostic [`crate::ir`].

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

use comrak::nodes::{AstNode, NodeValue, TableAlignment};
use comrak::{Arena, Options, parse_document};

use crate::ir::{AdmonitionKind, Align, Block, Document, Inline, ListItem, Meta};

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
    o.extension.alerts = true;
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

/// Footnote definitions plus the set of names currently being expanded,
/// used to break reference cycles (a footnote that references itself, or two
/// footnotes referencing each other, must not recurse forever).
struct Footnotes<'a> {
    defs: HashMap<String, Node<'a>>,
    expanding: RefCell<HashSet<String>>,
}

fn collect_footnotes<'a>(root: Node<'a>) -> Footnotes<'a> {
    let mut defs = HashMap::new();
    for node in root.descendants() {
        if let NodeValue::FootnoteDefinition(def) = &node.data.borrow().value {
            defs.insert(def.name.clone(), node);
        }
    }
    Footnotes {
        defs,
        expanding: RefCell::new(HashSet::new()),
    }
}

fn block<'a>(node: Node<'a>, fns: &Footnotes<'a>) -> Option<Block> {
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
        NodeValue::Alert(a) => Some(Block::Admonition {
            kind: match a.alert_type {
                comrak::nodes::AlertType::Note => AdmonitionKind::Note,
                comrak::nodes::AlertType::Tip => AdmonitionKind::Tip,
                comrak::nodes::AlertType::Important => AdmonitionKind::Important,
                comrak::nodes::AlertType::Warning => AdmonitionKind::Warning,
                comrak::nodes::AlertType::Caution => AdmonitionKind::Caution,
            },
            title: a
                .title
                .clone()
                .unwrap_or_else(|| a.alert_type.default_title().to_string()),
            blocks: node.children().filter_map(|n| block(n, fns)).collect(),
        }),
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

fn list_item<'a>(item: Node<'a>, fns: &Footnotes<'a>) -> ListItem {
    let task = match &item.data.borrow().value {
        NodeValue::TaskItem(t) => Some(t.symbol.is_some()),
        _ => None,
    };
    ListItem {
        task,
        blocks: item.children().filter_map(|n| block(n, fns)).collect(),
    }
}

fn table<'a>(node: Node<'a>, aligns: &[TableAlignment], fns: &Footnotes<'a>) -> Block {
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

fn inlines<'a>(node: Node<'a>, fns: &Footnotes<'a>) -> Vec<Inline> {
    let mut out = Vec::new();
    for child in node.children() {
        inline(child, fns, &mut out);
    }
    attach_image_attrs(&mut out);
    out.retain(|i| !matches!(i, Inline::Text(t) if t.is_empty()));
    out
}

/// Support `![alt](src){width=50%}`. The attribute block is not Markdown:
/// comrak surfaces it as a plain text node right after the image, so fold a
/// recognised `{…}` group into the preceding [`Inline::Image`].
fn attach_image_attrs(out: &mut [Inline]) {
    for i in 1..out.len() {
        let (head, tail) = out.split_at_mut(i);
        let Some(Inline::Image { width, .. }) = head.last_mut() else {
            continue;
        };
        let Inline::Text(t) = &mut tail[0] else {
            continue;
        };
        let Some(rest) = t.strip_prefix('{') else {
            continue;
        };
        let Some((attrs, after)) = rest.split_once('}') else {
            continue;
        };
        if let Some(w) = parse_width_attr(attrs) {
            *width = Some(w);
            *t = after.to_string();
        }
    }
}

fn parse_width_attr(attrs: &str) -> Option<String> {
    for part in attrs.split_whitespace() {
        if let Some(v) = part.strip_prefix("width=") {
            let v = v.trim_matches('"');
            if is_valid_dimension(v) {
                return Some(v.to_string());
            }
        }
    }
    None
}

/// A safe-to-interpolate dimension: a number followed by a known unit.
fn is_valid_dimension(v: &str) -> bool {
    ["%", "cm", "mm", "in", "pt", "em"].iter().any(|unit| {
        v.strip_suffix(unit).is_some_and(|n| {
            !n.is_empty() && n.parse::<f64>().is_ok_and(|x| x.is_finite() && x >= 0.0)
        })
    })
}

fn inline<'a>(node: Node<'a>, fns: &Footnotes<'a>, out: &mut Vec<Inline>) {
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
            width: None,
        }),
        NodeValue::FootnoteReference(r) => {
            if let Some(def) = fns.defs.get(&r.name) {
                // Guard against definition cycles: a reference seen while its
                // own definition is being expanded is dropped.
                if fns.expanding.borrow_mut().insert(r.name.clone()) {
                    let blocks = def.children().filter_map(|n| block(n, fns)).collect();
                    fns.expanding.borrow_mut().remove(&r.name);
                    out.push(Inline::Footnote(blocks));
                }
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
