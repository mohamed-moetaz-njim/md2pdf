# Comparison with other Markdown → PDF tools

This is an honest assessment, including where md2pdf is *not* the right tool.

## Feature matrix

| Capability | md2pdf | Pandoc (+LaTeX) | Typst CLI | mdBook (PDF) | markdown-pdf (npm) |
|---|:---:|:---:|:---:|:---:|:---:|
| Install footprint | 1 binary (~47 MB) | TeX Live (GBs) | 1 binary | Rust + headless Chromium | Node + Chromium |
| Input | Markdown | Markdown (+22 formats) | Typst markup | Markdown book | Markdown |
| Fonts bundled | ✅ | ❌ | ❌ (system/local) | ❌ | ❌ |
| Reproducible output | ✅ | ⚠️ (TeX/env-sensitive) | ⚠️ (font-dependent) | ❌ | ❌ |
| Network off by default | ✅ | ⚠️ | ⚠️ | ❌ | ❌ |
| Path-traversal guard | ✅ | ❌ | n/a | ❌ | ❌ |
| Tables / task lists / footnotes | ✅ | ✅ | ✅ (manual) | ✅ | ⚠️ |
| Themes (data-driven) | ✅ | ⚠️ (templates) | ✅ (code) | ⚠️ | ❌ |
| Syntax highlighting | ✅ | ✅ | ✅ | ✅ | ⚠️ |
| Mermaid diagrams | 🚧 planned | ⚠️ (filters) | ⚠️ (pkg) | ⚠️ | ⚠️ |
| Cold-start latency | milliseconds | seconds | milliseconds | seconds | seconds |
| First-class GitHub Action | ✅ | community | community | community | community |
| Multi-format output | PDF, Typst (HTML/DOCX planned) | everything | PDF/SVG/PNG | HTML/PDF | PDF |

Legend: ✅ yes · ⚠️ partial/with setup · ❌ no · 🚧 in progress

## When to use something else

- **You need 20 input/output formats or citations/CSL** → Pandoc is unmatched.
- **You author in Typst already** → use the Typst CLI directly; md2pdf targets
  Markdown authors who don't want to learn Typst.
- **You publish a browsable multi-page book site** → mdBook is purpose-built; md2pdf
  produces a single PDF, not an HTML book.

## Where md2pdf wins

- **CI and reproducibility.** One pinned binary, fonts inside it, network disabled →
  the same bytes on every machine. No `apt-get install texlive-full`.
- **Security posture.** Deny-by-default asset handling for untrusted Markdown.
- **Developer experience.** `md2pdf file.md` just works; `doctor`/`validate`/`init`
  cover the rest of the workflow.

## Reproducing these numbers

Latency and size claims are measured by the harness in [`benches/`](../benches).
Run `benches/run.sh` to regenerate the table inputs on your machine; please open a PR
if any column is inaccurate for your environment.
