# Roadmap

This roadmap turns the product brief into milestones with concrete, code-level
tasks. Checked items are implemented on the `feat/v0.2-architecture` line.

## v0.2 — Architecture & secure foundation ✅ (current)

The decoupling and security work that everything else depends on.

- [x] Cargo workspace: `md2pdf-core` (library) + `md2pdf` (CLI)
- [x] Renderer-agnostic IR (`ir.rs`) decoupling comrak from Typst
- [x] `parser.rs` (Markdown → IR) with frontmatter + footnote resolution
- [x] `Renderer` trait + `TypstPdfRenderer` + `TypstSourceRenderer`
- [x] `ThemeSpec` data model (foundation for theme inheritance)
- [x] `SecurityPolicy`: no remote, path-traversal guard, size caps, HTML dropped
- [x] Subcommands: `convert`, `validate`, `doctor`, `init`, `theme list`
- [x] Unit tests for parser + security; deterministic-lowering test

## v0.3 — Layout & content features

- [x] **Headers/footers** — `Block`-free page furniture driven by `Meta`
      (`render/typst/lower.rs`: `#set page(header: …, footer: …)`); flags
      `--header`, `--footer`, `--no-page-numbers`; `{title}`/`{author}`/`{date}`
      placeholders.
- [x] **Config file** — `md2pdf.toml` (theme, paper, security, header/footer);
      add `toml` + `serde`; load in `cli`, merge under CLI flags. `init` emits it.
- [x] **TOML themes + inheritance** — `theme.rs`: TOML overrides over a built-in
      `base`, `Theme::load(path)`, `theme create <name>`, `--theme file.toml`.
- [ ] **Local asset bundling** — copy/validate referenced assets into an output
      bundle; `--bundle` for self-contained artifacts.
- [x] **Image sizing** — width/attributes via `{width=}` syntax in IR `Image`.
- [x] **Admonitions** — GitHub alerts (`> [!NOTE]` …) as colored callouts.
- [ ] **Definition lists** — extend IR + parser + renderers.

## v0.4 — Diagrams & highlighting

- [ ] **Mermaid** — detect `lang == "mermaid"`; render via an embeddable engine or
      a sandboxed sidecar, off the network. IR already carries the fenced source;
      `lower.rs` has the hook + diagnostic today.
- [ ] **Themed syntax highlighting** — expose Typst raw-theme selection per
      `ThemeSpec`; ship a couple of code colour schemes.

## v0.5 — Renderers

- [x] **HTML renderer** — `render/html/`, `OutputFormat::Html`; reuse IR. Proves the
      abstraction a third time and unlocks web previews.
- [ ] **DOCX renderer (experimental)** — `render/docx/` via an OOXML writer.

## v0.6 — Testing & performance

- [ ] **Golden PDF tests** — render fixtures, compare normalized page text +
      structural hash (PDF bytes aren't stable, so compare extracted text + layout
      metrics, not raw bytes). `tests/golden/`.
- [ ] **AST snapshot tests** — `insta` snapshots of `Document` for fixtures.
- [ ] **Fuzzing** — `cargo-fuzz` target over `parser::parse` (must never panic).
- [ ] **Malformed-input corpus** — unterminated tables, huge nesting, bad UTF-8.
- [ ] **Benchmark suite** — `criterion` for parse + lower; `hyperfine` end-to-end vs
      Pandoc/Typst/mdBook; track startup, memory (`/usr/bin/time -v`), large docs,
      image-heavy docs. CI publishes a trend.

## v1.0 — Stability & ecosystem

- [ ] Stabilise the `md2pdf-core` public API; semver guarantees; `cargo public-api`.
- [ ] Man pages + shell completions (`clap_mangen`, `clap_complete`).
- [ ] Published, versioned GitHub Action with prebuilt-binary install.
- [ ] COPR + PPA automation from the release pipeline.
- [ ] Theme gallery and example showcase site.

## Maintainer automation (ongoing)

- [ ] `release-plz` for changelog + version + tag + crates.io publish.
- [ ] Dependabot/Renovate for deps and Actions.
- [ ] Issue-form templates + triage labels + stale-bot.
- [ ] Benchmark regression gate on PRs.

## Tracking

Each unchecked box maps to a GitHub issue labelled by milestone. See
[CONTRIBUTING.md](../CONTRIBUTING.md) for how to pick one up.
