---
title: Feature Tour
subtitle: Everything new in md2pdf 0.3
author: The md2pdf project
date: 2026
---

# Page furniture and themes

This document is rendered with a **custom TOML theme** ([theme.toml](theme.toml))
that inherits from the `default` base and overrides the accent color, plus a
running `--header` built from the document title.

Custom theme
: A `.toml` file passed to `--theme`, starting from a built-in `base` and
  overriding individual fields — colors are validated before rendering.

Page header
: Set with `--header "{title}"`; the placeholder resolves from frontmatter.

# Callouts

> [!NOTE]
> GitHub-style alerts render as colored callout boxes in both the PDF and the
> HTML output.

> [!WARNING]
> All five kinds are supported: note, tip, important, warning and caution.

# The classics, still here

| Feature | Since | Notes |
|:--------|:-----:|------:|
| Tables | 0.1 | with column alignment |
| Footnotes | 0.1 | numbered[^pdf] |
| Task lists | 0.1 | see below |
| Definition lists | 0.3 | this document |
| Alerts | 0.3 | the callouts above |

- [x] Convert this file to PDF
- [x] Convert it to HTML with `--format html`
- [ ] Pipe it from stdin with `md2pdf -`

```bash
md2pdf feature-tour/source.md --theme theme.toml --toc --header "{title}"
```

[^pdf]: Footnotes render at the bottom of the page, like this one.
