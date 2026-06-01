# Changelog

All notable changes to this project are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

[Unreleased]: https://github.com/mohamed-moetaz-njim/md2pdf/compare/v0.2.1...HEAD
[0.2.1]: https://github.com/mohamed-moetaz-njim/md2pdf/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/mohamed-moetaz-njim/md2pdf/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/mohamed-moetaz-njim/md2pdf/releases/tag/v0.1.0
