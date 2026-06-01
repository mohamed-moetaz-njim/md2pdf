#!/usr/bin/env bash
# Generate a synthetic Markdown document with N sections exercising headings,
# paragraphs, code blocks, tables, lists and footnotes. Prints to stdout.
set -euo pipefail
n="${1:-200}"

cat <<'EOF'
---
title: Benchmark Document
author: md2pdf benchmark harness
---

EOF

for i in $(seq 1 "$n"); do
  cat <<EOF
## Section $i

This is paragraph text for section $i with **bold**, *italic*, \`code\` and a
[link](https://example.com). Lorem ipsum dolor sit amet, consectetur adipiscing
elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.[^${i}]

[^${i}]: Footnote for section $i.

\`\`\`rust
fn section_${i}() -> u32 { ${i} }
\`\`\`

| Key | Value |
|:----|------:|
| id  | $i    |
| sq  | $((i * i)) |

- [ ] todo $i
- [x] done $i

EOF
done
