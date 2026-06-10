# OSS Impact Statement

*Prepared for open-source grant and program applications.*

## One-line summary

md2pdf makes **document generation reproducible and secure by default** — turning a
fragile, dependency-heavy CI step into a single pinned binary that produces
byte-identical, sandboxed PDFs from Markdown.

## The problem it solves

Generating PDFs from Markdown is ubiquitous (docs sites, reports, handbooks, release
artifacts) yet the common tooling is heavy and non-deterministic:

- **Pandoc** needs a multi-gigabyte TeX Live install and embeds timestamps, so output
  is not reproducible.
- **Chromium-based tools** (md-to-pdf, many "markdown-pdf" packages) ship a headless
  browser — a large, frequently-patched attack surface.
- None of them sandbox untrusted Markdown: remote images are fetched, local paths are
  read without bounds.

This blocks reproducible builds, inflates CI images, and creates supply-chain and
SSRF risk wherever documentation is built from untrusted or third-party content.

## What md2pdf delivers

- **Deterministic output** — identical `sha256` across repeated runs and across
  debug/release builds — enforced by a render-twice byte-compare gate in CI on Linux
  and macOS (the engine and fonts are pinned and no timestamps are embedded).
- **~3.5× faster** than Pandoc+LaTeX with **less memory**, and a **47 MB** self-contained
  binary vs **~1.7 GB** of toolchain — *locally measured on a single machine*, reproducible
  with the harness, not yet CI-published. (See [BENCHMARKS.md](BENCHMARKS.md).)
- **Deny-by-default security** — no network, no path traversal, bounded inputs, raw
  HTML dropped. (See [../SECURITY.md](../SECURITY.md).)

## Who benefits

| Audience | Benefit |
|:---------|:--------|
| **CI/CD & platform teams** | A tiny, pinnable binary; reproducible artifacts; no TeX/Chromium to provision or patch |
| **Reproducible-builds & supply-chain efforts** | Byte-stable PDFs that can be hashed and verified |
| **Security-sensitive orgs** | Safe rendering of untrusted Markdown (contracts, user submissions) |
| **Documentation authors** | `md2pdf file.md` just works; resumes, invoices, reports, handbooks |
| **Air-gapped / regulated environments** | Zero network dependency by construction |
| **The Rust & Typst ecosystems** | A reusable `md2pdf-core` library and a real-world Typst-as-a-library showcase |

## Why it is ecosystem infrastructure, not a toy

The renderer-agnostic core (`md2pdf-core`) is a library other tools can build on, with
a stable IR and pluggable back-ends. The project is positioned as the
**reproducible, secure documentation pipeline** primitive — a niche no existing tool
owns simultaneously.

## Sustainability

Maintenance is deliberately low-burden (see [PROGRAM.md](PROGRAM.md)): a small,
well-tested core; automated releases, changelogs and dependency updates; and a CI
suite that gates correctness, formatting, security defaults and reproducibility. This
is what lets a small maintainer team support a widely-used utility.
