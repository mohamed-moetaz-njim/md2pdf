#!/usr/bin/env bash
# End-to-end benchmark harness for md2pdf vs. other Markdown→PDF tools.
#
# Measures wall-clock conversion time with hyperfine and peak memory with
# /usr/bin/time. Tools that aren't installed are skipped, so partial results are
# fine. Regenerate docs/COMPARISON.md numbers from the output.
#
# Usage:
#   benches/run.sh [SIZE]      # SIZE = number of sections in the synthetic doc (default 200)
set -euo pipefail

here="$(cd "$(dirname "$0")" && pwd)"
size="${1:-200}"
work="$(mktemp -d)"
doc="$work/bench.md"

echo "Generating a ~${size}-section document at $doc"
bash "$here/gen.sh" "$size" >"$doc"

md2pdf_bin="${MD2PDF:-$here/../target/release/md2pdf}"
if [ ! -x "$md2pdf_bin" ]; then
  echo "Building release binary..."
  (cd "$here/.." && cargo build --release -p md2pdf >/dev/null)
fi

have() { command -v "$1" >/dev/null 2>&1; }

echo
echo "== Wall-clock (hyperfine) =="
if have hyperfine; then
  cmds=(--command-name md2pdf "$md2pdf_bin $doc -o $work/m.pdf")
  have pandoc      && cmds+=(--command-name pandoc      "pandoc $doc -o $work/p.pdf")
  have typst       && cmds+=(--command-name typst       "typst compile $doc $work/t.pdf || true")
  have md-to-pdf   && cmds+=(--command-name md-to-pdf   "md-to-pdf $doc")
  hyperfine --warmup 2 "${cmds[@]}" || true
else
  echo "hyperfine not installed; timing md2pdf only:"
  time "$md2pdf_bin" "$doc" -o "$work/m.pdf"
fi

echo
echo "== Peak memory (/usr/bin/time -v) =="
if [ -x /usr/bin/time ]; then
  /usr/bin/time -v "$md2pdf_bin" "$doc" -o "$work/m.pdf" 2>&1 | grep -E "Maximum resident|Elapsed" || true
fi

echo
echo "Artifacts in $work (remove when done)."
