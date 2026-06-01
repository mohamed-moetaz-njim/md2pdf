---
title: Engineering Handbook
subtitle: How we build and ship software
author: Platform Team
date: 2026
---

# Welcome

This handbook describes how we work. It is the single source of truth for our
engineering practices and is published as a PDF on every change.

# Principles

1. **Reproducibility first.** If a build isn't deterministic, it isn't done.
2. **Secure by default.** The safe path is the easy path.
3. **Write it down.** Decisions live in version control, not in chat.

# Development workflow

## Branching

We use short-lived feature branches off `main` and squash-merge via pull request.

## Code review

| Check        | Owner     | Blocking |
|:-------------|:----------|:--------:|
| Tests pass   | CI        | yes      |
| Lint clean   | CI        | yes      |
| Design sound | Reviewer  | yes      |
| Docs updated | Author    | no       |

## Definition of done

- [x] Tests added and passing
- [x] Lint and format clean
- [x] Documentation updated
- [ ] Changelog entry added
- [ ] Reviewed and approved

# On-call

> Page early, page often. It is always acceptable to escalate.

The primary on-call owns triage; the secondary owns communication. Runbooks live in
`docs/runbooks/` and are kept current as part of the definition of done.

# Tooling

```bash
# bootstrap a new service
platform init service --name billing
# run the full local check suite
platform check
```

# Further reading

See the architecture decision records (ADRs) for the reasoning behind our
infrastructure choices.
