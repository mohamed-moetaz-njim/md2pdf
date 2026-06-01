# Security Policy

## Threat model

md2pdf is commonly pointed at **untrusted Markdown** in CI. We therefore treat the
input document and any assets it references as potentially hostile.

| Threat | Mitigation | Where |
|---|---|---|
| Remote resource fetch (SSRF, tracking, non-reproducibility) | `http(s)`/`//`/`data:`/`ftp` references are never loaded; the PDF engine is built without the `packages` feature, so it cannot reach the network | `security.rs::resolve_image`, no `packages`/`reqwest`/`ureq` features |
| Path traversal (`../../etc/passwd`, absolute paths) | Asset paths are canonicalised and must stay inside the document root | `security.rs::resolve_image` |
| Oversized input (memory exhaustion) | Markdown capped at 16 MiB, images at 32 MiB by default | `SecurityPolicy::max_input_bytes` / `max_image_bytes` |
| Malicious/raw HTML injection | Raw HTML blocks/inlines are dropped, never passed to a back-end | `parser.rs` / `lower.rs` (`RawHtml` diagnostic) |
| Malformed images | Files are stat-checked and handed to Typst's hardened decoders; failures degrade to alt text | `security.rs`, `lower.rs::emit_image` |
| Temp-file races | No temp files are written; rendering is fully in-memory | `render/typst` |
| Asset injection via symlinks | Canonicalisation resolves symlinks before the root check | `security.rs` |

Denied assets never abort the render: they degrade to alt text and a diagnostic, so
a hostile reference cannot break a pipeline but also cannot smuggle content in.

## Secure defaults

- Network access: **off**. (`--allow-remote` only marks references as permitted for
  future network-capable back-ends; the PDF back-end still never fetches.)
- Run `md2pdf validate <file>` to see exactly which assets would be denied and why.

## Supported versions

Until v1.0, only the latest release receives security fixes.

## Reporting a vulnerability

Please report privately via **GitHub Security Advisories**
("Report a vulnerability" on the Security tab) or email
mohamedmoetaznjim@gmail.com. We aim to acknowledge within 72 hours. Do not open a
public issue for undisclosed vulnerabilities.
