# md2pdf

> Convert Markdown to PDF locally — no browser, no LaTeX, no Python. Just one binary.

[![CI](https://github.com/mohamed-moetaz-njim/md2pdf/actions/workflows/release.yml/badge.svg)](https://github.com/mohamed-moetaz-njim/md2pdf/actions)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Made with Rust](https://img.shields.io/badge/made%20with-rust-orange.svg)](https://www.rust-lang.org)

Most Markdown-to-PDF tools pull in a whole headless Chromium or a full LaTeX
distribution. **md2pdf** renders everything in-process with the
[Typst](https://typst.app) engine and ships its fonts inside the executable, so
the only thing you install is a single self-contained binary.

```console
$ md2pdf README.md
wrote README.pdf
```

## Features

- 📝 **CommonMark + GitHub flavor** — tables, task lists, strikethrough, footnotes, autolinks
- 🎨 **Themes** — a clean `default` and a classic `book` look
- 🧭 **Table of contents** — `--toc` builds one from your headings
- 📐 **Paper sizes** — `a4` or `letter`
- 🔤 **Fonts included** — embedded in the binary, nothing to install
- 🦀 **Single static binary** — no runtime dependencies at all

## Install

### Fedora / RHEL (COPR)

```bash
sudo dnf copr enable mohamed-moetaz-njim/md2pdf
sudo dnf install md2pdf
```

### Debian / Ubuntu (.deb)

Grab the latest `.deb` from the [releases page](https://github.com/mohamed-moetaz-njim/md2pdf/releases) and install it:

```bash
sudo apt install ./md2pdf_*_amd64.deb
```

### Any distro (.rpm)

```bash
sudo dnf install ./md2pdf-*.x86_64.rpm   # or: sudo rpm -i md2pdf-*.rpm
```

### From source

```bash
cargo install --git https://github.com/mohamed-moetaz-njim/md2pdf
```

## Usage

```
md2pdf <INPUT> [OPTIONS]

Arguments:
  <INPUT>  Markdown file to convert

Options:
  -o, --output <FILE>   Output PDF path (default: input with .pdf extension)
      --theme <THEME>   Visual theme [default: default] [possible values: default, book]
      --paper <PAPER>   Paper size [default: a4] [possible values: a4, letter]
      --toc             Add a table of contents built from the headings
      --title <TITLE>   Document title (default: first heading, then file name)
  -h, --help            Print help
  -V, --version         Print version
```

### Examples

```bash
# Simplest case — writes notes.pdf next to notes.md
md2pdf notes.md

# Pick an output path, the book theme and US Letter paper
md2pdf report.md -o build/report.pdf --theme book --paper letter

# Add a table of contents and a custom title
md2pdf handbook.md --toc --title "Engineering Handbook"
```

Try it on the bundled sample:

```bash
md2pdf examples/sample.md
```

## How it works

```
Markdown ──comrak──▶ AST ──▶ Typst markup ──typst──▶ PDF
```

1. [`comrak`](https://crates.io/crates/comrak) parses the Markdown into an AST.
2. A small renderer lowers that AST to [Typst](https://typst.app) markup, emitting
   every literal as a Typst string so nothing needs delicate markup escaping.
3. [`typst-as-lib`](https://crates.io/crates/typst-as-lib) compiles the markup to a
   `PagedDocument` using fonts embedded in the binary.
4. `typst-pdf` writes the final PDF.

No subprocesses, no temp HTML, no network access.

## Building distro packages

```bash
cargo install cargo-deb cargo-generate-rpm
cargo build --release
cargo deb            # -> target/debian/md2pdf_<version>_amd64.deb
cargo generate-rpm   # -> target/generate-rpm/md2pdf-<version>.x86_64.rpm
```

## License

[MIT](LICENSE) © Mohamed Moetaz Njim
