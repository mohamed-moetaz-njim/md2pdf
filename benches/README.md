# Benchmarks

End-to-end harness comparing md2pdf with other Markdown→PDF tools.

```bash
# Optional: install comparators and hyperfine
#   cargo install hyperfine
#   (pandoc, typst, md-to-pdf are picked up if present)

benches/run.sh 200      # synthetic doc with 200 sections
```

`run.sh` measures conversion wall-clock (via `hyperfine`) and peak RSS (via
`/usr/bin/time -v`). Missing tools are skipped, so partial runs are fine. `gen.sh`
produces the synthetic input (headings, paragraphs, code, tables, lists, footnotes).

## What we track

- **Startup / cold-start latency** — `md2pdf doctor` and tiny-doc conversion.
- **Throughput** — large synthetic documents (`gen.sh 1000`).
- **Image-heavy documents** — planned: a generator variant emitting many local
  images to stress the asset path.
- **Memory** — peak resident set size.

Numbers feed [docs/COMPARISON.md](../docs/COMPARISON.md). A `criterion` micro-bench
for `parser::parse` and Typst lowering is planned (see the roadmap) so parser
regressions are caught independently of the (dominant) Typst compile cost.
