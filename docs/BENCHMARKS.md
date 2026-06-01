# Benchmarks

Real measurements, not marketing. Everything here is reproducible with the harness
in [`benches/`](../benches); please open a PR if your numbers differ.

## Environment

| | |
|---|---|
| CPU | AMD Ryzen 5 5600H (12 threads) |
| OS / kernel | Fedora Linux, kernel 6.19 (x86_64) |
| md2pdf | 0.2.0 (release build, stripped) |
| Pandoc | 3.1.11.1 + pdfTeX (TeX Live 2023) |
| Workload | `benches/gen.sh 100` → a 2,005-line document (headings, paragraphs, code, tables, task lists, footnotes) |
| Method | 3 runs each, `/usr/bin/time`; lowest-variance run reported |

## Speed and memory

| Tool | Wall time | Peak memory | Output |
|:-----|----------:|------------:|-------:|
| **md2pdf** | **0.45 s** | **84 MB** | 31 pages |
| Pandoc + pdfTeX | 1.58 s | 124 MB | 37 pages |

md2pdf is **~3.5× faster** and uses **~30% less memory** on this workload, with no
external process spawned. Cold start on a trivial document is **~10 ms / 28 MB**.

## Reproducibility

The same input rendered twice, bytes compared:

| Tool | Identical bytes across runs? |
|:-----|:----------------------------:|
| **md2pdf** | ✅ yes (`sha256` matches) |
| Pandoc + pdfTeX | ❌ no (embeds build timestamps) |

This is md2pdf's defining property: deterministic output makes PDFs cacheable and
verifiable in a supply chain. Verify it yourself:

```bash
md2pdf doc.md -o a.pdf && md2pdf doc.md -o b.pdf && sha256sum a.pdf b.pdf
```

## Install footprint

| Tool | What you install |
|:-----|:-----------------|
| **md2pdf** | one 47 MB binary, fonts embedded, links only glibc |
| Pandoc (PDF) | 192 MB binary **+ ~1.5 GB TeX Live** |

## Comparators not yet measured here

`typst`, `mdBook` and `md-to-pdf` were not installed on the reference machine, so they
are omitted rather than guessed. The harness picks them up automatically when present:

```bash
cargo install hyperfine        # nicer statistics (optional)
benches/run.sh 100             # runs whatever comparators are installed
```

Expected shape from their designs: the Typst CLI is comparable in speed/size to
md2pdf but requires Typst-syntax input (not Markdown); mdBook and md-to-pdf rely on a
headless Chromium and so carry a much larger install and slower cold start. We will
publish measured numbers once they are added to the benchmark CI.

## Caveats

- Page-count differs because default margins/typography differ; both render the same
  content faithfully.
- Single-machine numbers. CI will track these over time (see the roadmap) so trends,
  not absolute values, are the signal.
