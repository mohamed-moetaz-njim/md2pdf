# Quickstart

From zero to a reproducible PDF in about a minute.

## 1. Install

```bash
cargo install md2pdf        # or a .deb/.rpm — see docs/INSTALL.md
```

## 2. Convert

```bash
md2pdf notes.md             # writes notes.pdf
```

That's it. Tables, code blocks, task lists and footnotes work out of the box.

## 3. Add metadata and a table of contents

```markdown
---
title: Engineering Handbook
author: Platform Team
date: 2026
---

# Introduction
...
```

```bash
md2pdf handbook.md --theme book --toc
```

## 4. Prove it's reproducible

```bash
md2pdf handbook.md -o a.pdf
md2pdf handbook.md -o b.pdf
sha256sum a.pdf b.pdf        # identical
```

## 5. Validate untrusted documents (no rendering)

```bash
md2pdf validate incoming.md  # reports denied assets, dropped HTML, stats
```

## 6. Wire it into CI

```yaml
- uses: mohamed-moetaz-njim/md2pdf@v0
  with:
    input: docs/handbook.md
    output: handbook.pdf
    theme: book
    toc: true
```

Or scaffold a whole project:

```bash
md2pdf init        # creates docs/ + a ready-to-use workflow
```

## Next

- [Example gallery](../examples/gallery/) — resume, invoice, API docs, whitepaper, …
- [Installation matrix](INSTALL.md)
- [Architecture](ARCHITECTURE.md) · [Security](../SECURITY.md) · [Benchmarks](BENCHMARKS.md)
