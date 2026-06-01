# md2pdf

> Turn Markdown into polished PDFs locally — no browser, no LaTeX, no Python. One static binary, reproducible by default.

[![CI](https://github.com/mohamed-moetaz-njim/md2pdf/actions/workflows/ci.yml/badge.svg)](https://github.com/mohamed-moetaz-njim/md2pdf/actions/workflows/ci.yml)
[![Release](https://github.com/mohamed-moetaz-njim/md2pdf/actions/workflows/release.yml/badge.svg)](https://github.com/mohamed-moetaz-njim/md2pdf/actions/workflows/release.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

md2pdf renders Markdown to PDF in-process with the [Typst](https://typst.app) engine
and ships its fonts inside the executable. The only thing you install is a single
self-contained binary — ideal for CI, air-gapped builds and reproducible docs.

```console
$ md2pdf README.md
wrote README.pdf (48 kB)
```

## Why md2pdf

| | md2pdf | Pandoc + LaTeX | Typst CLI | mdBook PDF | markdown-pdf (npm) |
|---|:---:|:---:|:---:|:---:|:---:|
| Single binary, no system deps | ✅ | ❌ (TeX Live) | ✅ | ⚠️ (needs Chromium) | ❌ (Node + Chromium) |
| Reads Markdown directly | ✅ | ✅ | ❌ (Typst syntax) | ✅ | ✅ |
| Fonts bundled (reproducible) | ✅ | ❌ | ❌ | ❌ | ❌ |
| Remote/network off by default | ✅ | ⚠️ | ⚠️ | ❌ | ❌ |
| Startup time | ms | seconds | ms | seconds | seconds |
| First-class CI action | ✅ | ⚠️ | ⚠️ | ⚠️ | ⚠️ |

See [docs/COMPARISON.md](docs/COMPARISON.md) for the full breakdown.

## Features

- 📝 **CommonMark + GitHub flavor** — tables, task lists, strikethrough, footnotes, autolinks
- 🧾 **Frontmatter** — `title`, `author`, `date`, `subtitle` and arbitrary metadata
- 🎨 **Themes** — `default` and `book`, with a data-driven theme model
- 🧭 **`--toc`** — table of contents from headings; `--paper a4|letter`
- 🔒 **Secure by default** — no remote fetches, path-traversal protection, size caps, raw HTML dropped
- 🧱 **Decoupled architecture** — pluggable renderers (PDF, Typst source today; HTML/DOCX planned)
- 🦀 **Single static binary** — fonts embedded, no runtime dependencies

## Install

### Fedora / RHEL (COPR)

```bash
sudo dnf copr enable mohamed-moetaz-njim/md2pdf
sudo dnf install md2pdf
```

### Debian / Ubuntu

```bash
sudo apt install ./md2pdf_*_amd64.deb   # from the Releases page
```

### From source

```bash
cargo install --git https://github.com/mohamed-moetaz-njim/md2pdf md2pdf
```

## Usage

```text
md2pdf <FILE>                      Convert (default action)
md2pdf convert <FILE> [options]    Convert explicitly
md2pdf validate <FILE>             Parse and lint without rendering
md2pdf doctor                      Check the local environment
md2pdf init [DIR]                  Scaffold a docs project + CI workflow
md2pdf theme list                  List built-in themes

Convert options:
  -o, --output <FILE>    Output path (default: input with .pdf)
      --format <FMT>     pdf | typst (default: from output extension, else pdf)
      --theme <THEME>    default | book
      --paper <PAPER>    a4 | letter
      --toc              Add a table of contents
      --title <TITLE>    Override the document title
      --allow-remote     Permit references to remote resources (off by default)
```

### Examples

```bash
md2pdf notes.md                                   # notes.pdf
md2pdf report.md -o out/report.pdf --theme book --paper letter
md2pdf handbook.md --toc --title "Engineering Handbook"
md2pdf spec.md --format typst -o spec.typ         # emit Typst source
md2pdf validate docs/*.md                         # CI linting
```

## Use in CI

```yaml
- uses: mohamed-moetaz-njim/md2pdf@v0
  with:
    input: docs/handbook.md
    output: handbook.pdf
    theme: book
    toc: true
```

`md2pdf init` scaffolds this workflow for you. More recipes (README→PDF, release
notes, whole `docs/` directories) live in [docs/](docs/).

## How it works

```text
Markdown ──parser──▶ Document (IR) ──Renderer──▶ bytes (PDF · Typst · HTML*)
            comrak      renderer-agnostic           *planned
```

Parsing and rendering are fully decoupled by an intermediate representation, so new
output formats are additive. See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

## Security

Untrusted Markdown is handled deny-by-default: no network access, no path traversal,
bounded inputs, raw HTML dropped. See [SECURITY.md](SECURITY.md).

## Contributing

Issues and PRs welcome — start with [CONTRIBUTING.md](CONTRIBUTING.md) and the
[roadmap](docs/ROADMAP.md). Adding a renderer is a great first contribution.

## License

[MIT](LICENSE) © Mohamed Moetaz Njim
