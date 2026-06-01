#!/usr/bin/env bash
# Regenerate every gallery PDF (and a first-page PNG preview, if pdftoppm is
# available) from its Markdown source. Run from anywhere.
set -euo pipefail

here="$(cd "$(dirname "$0")" && pwd)"
root="$here/../.."
bin="${MD2PDF:-$root/target/release/md2pdf}"

if [ ! -x "$bin" ]; then
  echo "Building release binary..."
  (cd "$root" && cargo build --release -p md2pdf >/dev/null)
fi

# name:theme:flags
examples=(
  "resume:default:"
  "invoice:default:"
  "technical-report:book:--toc"
  "api-documentation:default:--toc"
  "release-notes:default:"
  "whitepaper:book:--toc"
  "developer-handbook:book:--toc"
)

for spec in "${examples[@]}"; do
  IFS=: read -r name theme flags <<<"$spec"
  dir="$here/$name"
  echo "→ $name (theme: $theme ${flags:-})"
  "$bin" "$dir/source.md" -o "$dir/$name.pdf" --theme "$theme" $flags
  if command -v pdftoppm >/dev/null 2>&1; then
    pdftoppm -png -r 110 -f 1 -l 1 -singlefile "$dir/$name.pdf" "$dir/$name"
  fi
done

echo "Done. PDFs and previews written under $here/<name>/."
