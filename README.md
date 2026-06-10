# md2pdf

> **Reproducible, secure documentation pipelines.** Turn Markdown into byte-identical PDFs from a single binary — no Chromium, no TeX Live, no network.

[![CI](https://github.com/mohamed-moetaz-njim/md2pdf/actions/workflows/ci.yml/badge.svg)](https://github.com/mohamed-moetaz-njim/md2pdf/actions/workflows/ci.yml)
[![Release](https://github.com/mohamed-moetaz-njim/md2pdf/actions/workflows/release.yml/badge.svg)](https://github.com/mohamed-moetaz-njim/md2pdf/actions/workflows/release.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

md2pdf treats document generation as a **build step**: deterministic, hermetic and
sandboxed. It compiles Markdown to PDF in-process with the [Typst](https://typst.app)
engine and embeds its fonts in the executable, so the same input produces the same
bytes on every machine — ideal for CI, air-gapped builds and supply-chain verification.

```console
$ md2pdf handbook.md && md2pdf handbook.md && sha256sum handbook.pdf
# identical hash every run — reproducible by construction
```

### Locally measured against Pandoc + LaTeX (100-section doc)

> Numbers below were measured on a single machine (AMD Ryzen 5 5600H) and are
> reproducible with [`benches/run.sh`](benches/run.sh) — they are **not** yet
> CI-published or independently certified. Treat them as indicative.

| | md2pdf | Pandoc + pdfTeX |
|---|:---:|:---:|
| Wall time | **0.45 s** | 1.58 s |
| Peak memory | **84 MB** | 124 MB |
| Reproducible bytes | **✅ yes** | ❌ no |
| Install footprint | **47 MB binary** | 192 MB + ~1.5 GB TeX Live |

Full methodology and the comparison matrix: [docs/BENCHMARKS.md](docs/BENCHMARKS.md) ·
[docs/COMPARISON.md](docs/COMPARISON.md). See real output in the
[example gallery](examples/gallery/) (resume, invoice, API docs, whitepaper, …).

## Why it matters

- **Deterministic output** — same input, same bytes; PDFs become cacheable, verifiable artifacts.
- **Secure by default** — no network, no path traversal, bounded inputs, raw HTML dropped.
- **CI-friendly** — one pinned binary, a [GitHub Action](action.yml), millisecond cold start.
- **No Chromium, no TeX Live** — nothing to provision, nothing to keep patched.

## Features

- 📝 **CommonMark + GitHub flavor** — tables, task lists, strikethrough, footnotes, autolinks
- 🧾 **Frontmatter** — `title`, `author`, `date`, `subtitle` and arbitrary metadata
- 🎨 **Themes** — `default` and `book`, with a data-driven theme model
- 🧭 **`--toc`** — table of contents from headings; `--paper a4|letter`
- 🔒 **Secure by default** — no remote fetches, path-traversal protection, size caps, raw HTML dropped
- 🧱 **Decoupled architecture** — pluggable renderers (PDF, Typst source today; HTML/DOCX planned)
- 🦀 **Single static binary** — fonts embedded, no runtime dependencies

## Install

### Fedora / RHEL

```bash
sudo dnf install ./md2pdf-*.x86_64.rpm   # from the Releases page
```

> A COPR repository (`dnf copr enable …`) is **planned but not yet available**.

### Debian / Ubuntu

```bash
sudo apt install ./md2pdf_*_amd64.deb   # from the Releases page
```

### From source

```bash
cargo install --git https://github.com/mohamed-moetaz-njim/md2pdf md2pdf
```

Full [installation matrix](docs/INSTALL.md) (`.deb`, `.rpm`, tarball, Action; COPR planned) ·
[60-second quickstart](docs/QUICKSTART.md).

## Usage

```text
md2pdf <FILE>                      Convert (default action)
md2pdf convert <FILE> [options]    Convert explicitly
md2pdf validate <FILES...>         Parse and lint without rendering (--strict to gate CI)
md2pdf doctor                      Check the local environment
md2pdf init [DIR]                  Scaffold a docs project + CI workflow
md2pdf theme list                  List built-in themes

Convert options:
  -o, --output <FILE>    Output path (default: input with .pdf)
      --format <FMT>     pdf | typst (default: from output extension, else pdf)
      --theme <THEME>    default | book
      --paper <PAPER>    a4 | letter
      --toc / --no-toc   Enable or disable the table of contents
      --title <TITLE>    Override the document title
      --header <TEXT>    Running page header ({title}, {author}, {date})
      --footer <TEXT>    Running page footer ({title}, {author}, {date})
      --no-page-numbers  Hide page numbers
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

## Documentation

| | |
|:--|:--|
| [Quickstart](docs/QUICKSTART.md) · [Install](docs/INSTALL.md) | get running |
| [Architecture](docs/ARCHITECTURE.md) | how the IR + renderers fit together |
| [Security](SECURITY.md) | threat model and secure defaults |
| [Benchmarks](docs/BENCHMARKS.md) · [Comparison](docs/COMPARISON.md) | measured numbers |
| [Example gallery](examples/gallery/) | seven real documents |
| [Roadmap](docs/ROADMAP.md) · [Program](docs/PROGRAM.md) · [Impact](docs/IMPACT.md) | direction & sustainability |

## Contributing

Issues and PRs welcome — start with [CONTRIBUTING.md](CONTRIBUTING.md) and the
[roadmap](docs/ROADMAP.md). Adding a renderer is a great first contribution.
Maintained per [MAINTAINERS.md](MAINTAINERS.md).

## License

[MIT](LICENSE) © Mohamed Moetaz Njim
