# Changelog

All notable changes to this project are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- **Experimental DOCX renderer** (`--format docx` or a `.docx` output path):
  headings, lists, tables, code, blockquotes, admonitions, definition lists,
  footnotes, hyperlinks, images and page headers/footers, with deterministic
  output (paragraph/footnote ids are normalized).
- **Directory conversion**: `md2pdf docs/` converts every Markdown file under
  the tree; `-o <DIR>` mirrors the structure.
- `completions`/piped output no longer panic on closed pipes (SIGPIPE default).
- New gallery example (`feature-tour`) showcasing the 0.3 features.
- CI: `cargo-deny` license/source policy and a `cargo-semver-checks` API guard
  against the latest release.

## [0.3.0] - 2026-06-10

### Added
- **HTML renderer**: `--format html` (or a `.html` output path) emits a
  standalone page with the theme inlined as CSS, sharing the IR and the
  deny-by-default security posture with the PDF back-end.
- **Page furniture**: `[layout]` config and `--header`/`--footer`/
  `--no-page-numbers` flags, with `{title}`/`{author}`/`{date}` placeholders.
- **Custom TOML themes with inheritance**: `--theme file.toml` starts from a
  built-in `base` and overrides fields; `md2pdf theme create <name>` scaffolds
  one. Colors are validated as hex before reaching the renderer.
- **GitHub alerts** (`> [!NOTE]` …) render as colored callout boxes.
- **Definition lists** (`term` / `: details`) in all renderers.
- **Image sizing**: `![alt](src){width=50%}` (validated number + unit).
- **Paper sizes** `a5` and `legal`.
- **stdin/stdout piping**: `md2pdf -` reads stdin; `-o -` streams to stdout.
- `validate` accepts multiple files and gates CI with `--strict`.
- `completions <shell>` subcommand; release tarballs ship man page +
  bash/zsh/fish completions.
- Criterion micro-benchmarks (`cargo bench -p md2pdf-core`).
- CI: linux/macos/windows test matrix, MSRV job, weekly `cargo-audit`, and a
  byte-level reproducibility gate. Releases gain SHA256SUMS, build-provenance
  attestation and macOS/Windows binaries.

### Fixed
- Stale committed `Cargo.lock` that would have failed the next `--locked`
  release build; CI now passes `--locked` everywhere.
- `toc = true` in `md2pdf.toml` was silently overridden by the CLI default;
  `--no-toc` added for explicit override.
- Stack overflow on self-referential footnote definitions (cycle guard).
- Output bytes depended on the input's line endings; CRLF is normalized so
  conversion is reproducible across platforms.
- Successive paragraphs inside one list item no longer run together.

### Changed
- Declared MSRV raised to 1.89 (required by the Typst engine; 1.85 was never
  buildable).
- `ThemeSpec` fields are owned strings; `Theme` gained a `Custom` variant.

## [0.2.2] - 2026-06-05

### Added
- `md2pdf.toml` config file support with `[document]`, `[security]` and `[layout]`
  sections. Config auto-loads from the document root or via `--config <PATH>`.
- `serde` and `toml` dependencies enabling future TOML-based features (themes,
  header/footer configuration).
- `--config <PATH>` flag for explicit config path.
- `md2pdf init` now emits a sample `md2pdf.toml`.
- `md2pdf doctor` reports whether a config file was found.
- 10 unit tests for config parsing, merge precedence, validation and round-trip.

### Changed
- `--theme` and `--paper` CLI flags are now optional, allowing config file
  defaults to take effect when the flags are omitted.
- Precedence for all document options: CLI flags > config file > built-in defaults.

## [0.2.1] - 2026-06-01

A trustworthiness/maintenance release. No features, no architecture or benchmark
changes — only audit remediation.

### Security
- Asset resolution now **fails closed** when the document root cannot be
  canonicalised, instead of falling back to a weaker non-canonical comparison.

### Removed
- The `--allow-remote` flag and `SecurityPolicy::allow_remote` field. The flag was
  never enforced (remote resources are always denied), so it was misleading.

### Documentation
- Marked COPR, crates.io and GitHub Action install paths as planned/pending rather
  than implying they are available today.
- Labeled benchmark numbers as locally measured on a single machine (reproducible via
  the harness), not CI-published or certified.

### CI
- `release-plz` is now inert unless a crates.io token is configured, preventing a
  failing release run on the first push to `main`.

## [0.2.0] - 2026-06-01

### Added
- Renderer-agnostic intermediate representation (IR) decoupling Markdown parsing
  from output rendering.
- `md2pdf-core` library crate + `md2pdf` CLI crate (Cargo workspace).
- `Renderer` trait with `TypstPdfRenderer` and `TypstSourceRenderer`
  (`--format typst` emits `.typ`).
- Frontmatter support (`title`, `author`, `date`, `subtitle`, arbitrary metadata).
- Deny-by-default security policy: no remote resources, path-traversal protection,
  input/image size caps, raw HTML dropped.
- Subcommands: `convert`, `validate`, `doctor`, `init`, `theme list`.
- Data-driven theme model (`ThemeSpec`).
- Unit tests for parser and security; deterministic-lowering test.

### Changed
- Restructured the single-crate project into a workspace.

## [0.1.0] - 2026-06-01

### Added
- Initial release: Markdown → PDF via comrak + Typst, single self-contained binary
  with embedded fonts.
- `default` and `book` themes, `--toc`, `--paper`.
- `.deb`/`.rpm` packaging and GitHub Releases workflow.

[Unreleased]: https://github.com/mohamed-moetaz-njim/md2pdf/compare/v0.2.2...HEAD
[0.2.2]: https://github.com/mohamed-moetaz-njim/md2pdf/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/mohamed-moetaz-njim/md2pdf/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/mohamed-moetaz-njim/md2pdf/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/mohamed-moetaz-njim/md2pdf/releases/tag/v0.1.0
