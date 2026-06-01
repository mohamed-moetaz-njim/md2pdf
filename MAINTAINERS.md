# Maintainers

## Maintainer statement

md2pdf is maintained by Mohamed Moetaz Njim with the goal of being a dependable,
boring piece of infrastructure: small surface area, high test coverage, predictable
releases. We optimize for **long-term maintainability over feature count** and will
decline features that add complexity without improving adoption, security or
reproducibility.

## Current maintainers

| Maintainer | Role | Contact |
|:-----------|:-----|:--------|
| Mohamed Moetaz Njim ([@mohamed-moetaz-njim](https://github.com/mohamed-moetaz-njim)) | Lead | mohamedmoetaznjim@gmail.com |

## What maintenance actually involves

- **Triage** — issue templates route reports; a label scheme (`bug`, `enhancement`,
  `security`, `good first issue`) keeps the queue legible.
- **Reviews** — the PR template enforces the architecture boundary (parser ↔ renderer
  via the IR), formatting, lint and tests.
- **Releases** — `release-plz` proposes version bumps and changelog entries from
  conventional commits; merging ships `.deb`/`.rpm`/tarball + crates.io.
- **Dependencies** — Dependabot opens grouped weekly PRs; CI gates them.
- **Performance** — the benchmark workflow tracks speed/memory over time.

## How AI tooling reduces maintainer burden

This project is explicitly designed so that routine maintenance can be assisted by AI
coding tools, lowering the human cost of keeping it healthy:

- **Dependency PRs** — Dependabot opens them; an agent can read CI output, apply small
  fixups (e.g. an API rename in `parser.rs`) and summarize the diff for a human's
  final approval.
- **Issue triage** — agents can reproduce a reported bug from the template's minimal
  Markdown, label it, and draft a failing test.
- **Changelog & releases** — generated from conventional commits; an agent keeps
  commit messages consistent and drafts release notes.
- **Reviews** — the strict architectural invariant ("no comrak outside `parser.rs`, no
  typst outside `render/typst`") is mechanically checkable, so an agent can pre-screen
  PRs against it.

The human maintainer stays in the loop for judgement calls (API design, security
decisions, what *not* to build); AI handles the repetitive surface. This is what makes
a small-team, widely-used utility sustainable.

## Becoming a maintainer

Land a few quality PRs (see [CONTRIBUTING.md](CONTRIBUTING.md)), then ask. We add
maintainers who show good judgement on scope and care about the test suite.
