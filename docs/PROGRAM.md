# Program Readiness & Prioritization

This document scores every adoption/maintenance investment by **implementation
effort**, **adoption impact** and **OSS-program impact** (1 = low, 5 = high), and
records what is already done. It is the prioritization backbone for grant
applications and roadmap decisions.

Effort: S ≈ hours · M ≈ a day · L ≈ multiple days.

## Phase 1 — Distribution

| Item | Effort | Adoption | Program | Status |
|:-----|:------:|:--------:|:-------:|:-------|
| GitHub Releases (binary/.deb/.rpm/tarball/zip) | S | 5 | 4 | ✅ live — v0.3.0 published (linux/macOS/Windows, SHA256SUMS, provenance) |
| `.deb` packaging | S | 4 | 3 | ✅ verified locally + in CI |
| `.rpm` / COPR | M | 4 | 3 | ✅ rpm verified; COPR needs account setup |
| `cargo install` | S | 4 | 3 | ✅ `--git` works now; crates.io on first publish |
| Installation matrix | S | 3 | 2 | ✅ `docs/INSTALL.md` |
| Quickstart | S | 4 | 2 | ✅ `docs/QUICKSTART.md` |

## Phase 2 — Social proof

| Item | Effort | Adoption | Program | Status |
|:-----|:------:|:--------:|:-------:|:-------|
| Measured md2pdf numbers (size/mem/startup) | S | 4 | 4 | ✅ `docs/BENCHMARKS.md` |
| Reproducibility proof (byte-identical) | S | 5 | 5 | ✅ enforced in CI (render-twice byte gate, linux+macOS) |
| vs Pandoc (measured) | S | 5 | 4 | ✅ 3.5× faster, smaller, reproducible |
| vs Typst / mdBook / md-to-pdf | M | 3 | 3 | 🔧 harness ready; comparators not installed locally |
| Benchmark CI | S | 2 | 4 | ✅ `bench.yml` |

## Phase 3 — Real use cases

| Item | Effort | Adoption | Program | Status |
|:-----|:------:|:--------:|:-------:|:-------|
| 8 example documents + real PDFs | M | 5 | 4 | ✅ `examples/gallery/` |
| First-page screenshots | S | 4 | 2 | ✅ generated |
| Gallery CI (anti-drift) | S | 2 | 3 | ✅ `examples.yml` |
| GIF demo | S | 4 | 2 | ⬜ needs a screen recording (manual) |

## Phase 4 — Maintainer burden

| Item | Effort | Adoption | Program | Status |
|:-----|:------:|:--------:|:-------:|:-------|
| Issue / bug / feature templates | S | 2 | 4 | ✅ `.github/ISSUE_TEMPLATE/` |
| PR template (enforces architecture) | S | 2 | 4 | ✅ |
| Dependency automation | S | 1 | 4 | ✅ `dependabot.yml` |
| Release + changelog automation | M | 2 | 5 | ✅ `release-plz` (needs `CARGO_REGISTRY_TOKEN`) |
| CI: fmt + clippy + test + smoke + repro gate | S | 3 | 4 | ✅ `ci.yml` (linux/macOS/Windows + MSRV) |
| Security advisories CI | S | 1 | 4 | ✅ `audit.yml` (weekly cargo-audit) |

## Phase 5 — Ecosystem positioning

| Item | Effort | Adoption | Program | Status |
|:-----|:------:|:--------:|:-------:|:-------|
| Reposition as "reproducible secure documentation pipelines" | S | 5 | 5 | ✅ README + docs |
| Comparison matrix | S | 4 | 3 | ✅ `docs/COMPARISON.md` |

## Phase 6 — Program readiness

| Item | Effort | Adoption | Program | Status |
|:-----|:------:|:--------:|:-------:|:-------|
| OSS impact statement | S | 2 | 5 | ✅ `docs/IMPACT.md` |
| Maintainer statement + AI-leverage | S | 1 | 5 | ✅ `MAINTAINERS.md` |
| Roadmap | S | 2 | 4 | ✅ `docs/ROADMAP.md` |
| Contributor guide | S | 3 | 4 | ✅ `CONTRIBUTING.md` |

## Highest-leverage next actions (owner)

1. **Add `CARGO_REGISTRY_TOKEN`** and publish to crates.io (Adoption 4) —
   release v0.3.0 is already live with multi-platform binaries.
2. **Set up COPR** from the repo (Adoption 4).
3. **Record a GIF** for the README (Adoption 4).
4. **Submit to "Show HN" / r/rust** — the gallery and a published release are live (Adoption 5).

## What we are deliberately NOT doing (complexity without adoption)

Rejected or deferred because they add surface area without moving adoption,
security or reproducibility:

| Idea | Why rejected/deferred |
|:-----|:----------------------|
| Live-reload preview server | Large surface; doesn't serve the CI/reproducibility thesis |
| Plugin/scripting system | Security and maintenance cost outweigh demand; themes-as-data covers most needs |
| Bundled web GUI | Contradicts the single-binary, no-Chromium positioning |
| Arbitrary HTML pass-through | Breaks the security model on purpose |
| Many niche Markdown dialects | Maintenance tax; CommonMark + GFM covers the audience |
| DOCX before HTML | HTML proves the renderer abstraction more cheaply and has wider demand |
