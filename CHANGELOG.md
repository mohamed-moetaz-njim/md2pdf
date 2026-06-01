# Changelog

All notable changes to this project are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

[Unreleased]: https://github.com/mohamed-moetaz-njim/md2pdf/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/mohamed-moetaz-njim/md2pdf/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/mohamed-moetaz-njim/md2pdf/releases/tag/v0.1.0
