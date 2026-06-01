---
title: Orbit 4.2 Release Notes
author: The Orbit Team
date: 2026-06-01
---

# Highlights

Orbit 4.2 focuses on reproducible builds and a faster cold start. Upgrade with
`orbit self update`.

## ✨ New

- **Deterministic artifacts** — builds now produce byte-identical outputs given the
  same inputs, making cache hits and supply-chain verification reliable.
- **`orbit doctor`** — one command to validate your environment.
- Sub-100 ms cold start on typical projects.

## 🛠 Improvements

- Reduced binary size by 18%.
- Clearer error messages for misconfigured pipelines.
- Documentation is now generated and published as a PDF on every release.

## 🐞 Fixes

- Fixed a race in the asset cache under high parallelism.
- Correctly handle UTF-8 paths on Windows.

## ⚠️ Breaking changes

| Change                         | Migration                                  |
|:-------------------------------|:-------------------------------------------|
| `--legacy-cache` removed       | Remove the flag; the new cache is default  |
| Config key `net.fetch` renamed | Use `network.allow` instead                |

## Contributors

Thanks to everyone who contributed to this release. See the full changelog for the
complete list of merged pull requests.
