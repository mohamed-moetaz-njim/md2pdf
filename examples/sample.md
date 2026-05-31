# md2pdf

A small, fast, **local** Markdown to PDF converter. No browser, no LaTeX, no
Python — just a single binary that turns this file into a clean PDF.

## Why

Most Markdown-to-PDF tools drag in a whole headless browser or a full LaTeX
distribution. `md2pdf` compiles everything in-process with [Typst][typst] and
ships the fonts inside the binary, so the only thing you install is one
executable.

[typst]: https://typst.app

## Features

- CommonMark plus GitHub extensions
- Tables, task lists and footnotes[^1]
- Syntax-friendly code blocks
- Two themes and an optional table of contents

[^1]: Footnotes render at the bottom of the page, just like this one.

### Inline formatting

You get **bold**, *italic*, ~~strikethrough~~ and `inline code`, plus
[links](https://example.com) that stay clickable in the PDF.

### A table

| Feature      | Supported | Notes                 |
|:-------------|:---------:|----------------------:|
| Tables       | yes       | with column alignment |
| Code blocks  | yes       | monospace, boxed      |
| Footnotes    | yes       | numbered              |

### A code block

```rust
fn main() {
    println!("rendered by md2pdf");
}
```

### A task list

- [x] Parse Markdown
- [x] Emit Typst
- [ ] Conquer the world

> Blockquotes are supported too — handy for callouts and citations.

---

That's it. Run `md2pdf examples/sample.md` and open the result.
