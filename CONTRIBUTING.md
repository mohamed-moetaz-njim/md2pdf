# Contributing to md2pdf

Thanks for helping out! This project aims to be a friendly, well-architected place
to contribute.

## Getting started

```bash
git clone https://github.com/mohamed-moetaz-njim/md2pdf
cd md2pdf
cargo build
cargo test
cargo run -p md2pdf -- examples/sample.md -o /tmp/out.pdf
```

You need a recent stable Rust (see `rust-version` in `Cargo.toml`). No system
libraries are required — fonts and the rendering engine are vendored.

## Project layout

- `crates/core` — the library: IR, parser, renderers, themes, security.
- `crates/cli` — the `md2pdf` binary and subcommands.

Read [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) first — the golden rule is that
**the parser and renderers communicate only through the IR** (`crates/core/src/ir.rs`).

## Good first issues

- **Add a theme** — return a new `ThemeSpec` from `Theme::spec`.
- **Add a renderer** — implement `Renderer` over the IR (HTML is a great target;
  `TypstSourceRenderer` is a minimal worked example).
- **Add a Markdown feature** — extend the IR, map it in `parser.rs`, handle it in the
  renderers (the compiler will tell you which back-ends still need it).

## Before you open a PR

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

- Keep parsing and rendering decoupled — no `comrak` types outside `parser.rs`, no
  `typst` types outside `render/typst`.
- Add tests next to what you change. Parser/security tests are fast; renderer changes
  should at least keep `validate` and a sample render working.
- Commits: short, imperative, lower-case (e.g. `add html renderer`). One logical
  change per commit.

## Reporting bugs

Open an issue with the input Markdown (or a minimal repro), the command you ran, and
what you expected. `md2pdf doctor` output helps for environment issues.

## Code of conduct

This project follows the [Contributor Covenant](CODE_OF_CONDUCT.md).
