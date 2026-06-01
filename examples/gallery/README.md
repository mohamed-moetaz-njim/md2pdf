# Example gallery

Real documents produced by md2pdf — every PDF here was generated from the Markdown
beside it by [`build.sh`](build.sh), with no manual editing. Previews are the first
page of each PDF.

> Regenerate everything: `examples/gallery/build.sh`

| Example | What it shows | Source | PDF | Preview |
|:--------|:--------------|:------:|:---:|:-------:|
| **Resume** | Headings, lists, a skills table, frontmatter byline | [md](resume/source.md) | [pdf](resume/resume.pdf) | [png](resume/resume.png) |
| **Invoice** | Aligned currency tables, totals block, hard line breaks | [md](invoice/source.md) | [pdf](invoice/invoice.pdf) | [png](invoice/invoice.png) |
| **Technical report** | TOC, timeline table, code, task list, footnotes, blockquote | [md](technical-report/source.md) | [pdf](technical-report/technical-report.pdf) | [png](technical-report/technical-report.png) |
| **API documentation** | TOC, HTTP/JSON/bash code, parameter & error tables | [md](api-documentation/source.md) | [pdf](api-documentation/api-documentation.pdf) | [png](api-documentation/api-documentation.png) |
| **Release notes** | Sectioned changelog, breaking-changes table | [md](release-notes/source.md) | [pdf](release-notes/release-notes.pdf) | [png](release-notes/release-notes.png) |
| **Whitepaper** | Book theme, TOC, properties table, footnotes, diagram | [md](whitepaper/source.md) | [pdf](whitepaper/whitepaper.pdf) | [png](whitepaper/whitepaper.png) |
| **Developer handbook** | Book theme, ordered lists, review table, runbook quote | [md](developer-handbook/source.md) | [pdf](developer-handbook/developer-handbook.pdf) | [png](developer-handbook/developer-handbook.png) |

## Reproduce any of these

```bash
md2pdf examples/gallery/invoice/source.md            # → invoice.pdf
md2pdf examples/gallery/whitepaper/source.md --theme book --toc
```

The `examples` CI workflow rebuilds this gallery on every change and uploads the
PDFs as artifacts, so the previews never drift from the sources.
