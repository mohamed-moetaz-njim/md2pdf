# Changelog

All notable changes to this project are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
