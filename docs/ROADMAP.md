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
- [x] **Definition lists** — extend IR + parser + renderers.

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

- [x] **Golden tests** — the Typst lowering of `examples/sample.md` is checked in
      (`tests/golden/`), and CI renders twice and byte-compares the PDFs (which
      *are* stable by construction).
- [ ] **AST snapshot tests** — `insta` snapshots of `Document` for fixtures
      (partially covered by the golden lowering test).
- [x] **Property tests** — proptest over arbitrary inputs (`parse` must never
      panic); a `cargo-fuzz` target remains possible later.
- [x] **Malformed-input corpus** — unterminated tables, huge nesting, bad UTF-8,
      injection attempts (`tests/corpus.rs`).
- [x] **Benchmark suite** — `criterion` for parse + lower (`cargo bench -p
      md2pdf-core`); `hyperfine` end-to-end harness in `benches/`. CI trend
      publishing still open.

## v1.0 — Stability & ecosystem

- [x] Semver guard: `cargo-semver-checks` gates `md2pdf-core` API changes on PRs
      against the latest release. Full 1.0 API freeze still pending.
- [x] Man pages + shell completions (`clap_mangen`, `clap_complete`), shipped in
      release tarballs.
- [x] Versioned GitHub Action with prebuilt-binary install (`@v0.3.0`);
      Marketplace listing still pending.
- [ ] COPR + PPA automation from the release pipeline.
- [ ] Theme gallery and example showcase site.

## Maintainer automation (ongoing)

- [x] `release-plz` wired (inert until `CARGO_REGISTRY_TOKEN` is added); releases
      also cut from `release/v*` branches or workflow dispatch.
- [x] Dependabot for cargo deps and Actions; weekly `cargo-audit` and a
      `cargo-deny` license/source policy in CI.
- [x] Issue-form templates (bug/feature) with a security-advisory contact link.
- [ ] Benchmark regression gate on PRs.

## Tracking

Each unchecked box maps to a GitHub issue labelled by milestone. See
[CONTRIBUTING.md](../CONTRIBUTING.md) for how to pick one up.
