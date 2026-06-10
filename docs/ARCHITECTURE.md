# Architecture

md2pdf is built around one principle: **parsing and rendering must not know about
each other.** They communicate only through a renderer-agnostic intermediate
representation (IR). This keeps the Markdown front-end and the output back-ends
independently evolvable and testable.

## Pipeline

```text
                 crates/core
 ┌───────────────────────────────────────────────────────────┐
 │  Markdown ─▶ parser ─▶ Document (ir) ─▶ Renderer ─▶ bytes  │
 │              (comrak)   security ▲        ├─ TypstPdf       │
 │                                  │        ├─ TypstSource    │
 │                          asset decisions  ├─ Html           │
 │                                           └─ Docx*          │
 └───────────────────────────────────────────────────────────┘
                 crates/cli  ── wires args ▶ pipeline ▶ file   (*planned)
```

## Crate & module layout

```text
crates/
  core/                     # the library — no CLI, no I/O policy
    src/
      ir.rs                 # Document, Block, Inline, Meta  (the contract)
      parser.rs             # Markdown → IR   (ONLY module that imports comrak)
      security.rs           # SecurityPolicy, AssetDecision (deny-by-default)
      theme.rs              # Theme, ThemeSpec (themes as data → inheritance)
      render/
        mod.rs              # Renderer trait, RenderOptions, OutputFormat, Paper
        typst/
          mod.rs            # TypstPdfRenderer, TypstSourceRenderer
          lower.rs          # IR → Typst markup (shared by both renderers)
        html/
          mod.rs            # HtmlRenderer (standalone page, theme as CSS)
      lib.rs                # convert() convenience + unit tests
  cli/                      # the binary
    src/
      args.rs               # clap definitions, *Arg → core enum mapping
      commands/             # convert, validate, doctor, init, theme
      main.rs               # dispatch
```

## The IR contract (`ir.rs`)

`Document { meta: Meta, blocks: Vec<Block> }` is the single boundary type.

- `Block`: `Heading`, `Paragraph`, `CodeBlock`, `BlockQuote`, `List`, `Table`,
  `DefinitionList`, `Admonition`, `ThematicBreak`, `RawHtml`.
- `Inline`: `Text`, `Emph`, `Strong`, `Strikethrough`, `Superscript`, `Code`,
  `Link`, `Image`, `Footnote`, `SoftBreak`, `LineBreak`.

Footnotes are **resolved during parsing** (definitions inlined at their reference
site) so renderers never chase cross-references. Raw HTML is preserved in the IR as
`Block::RawHtml` but dropped by safe renderers.

## SOLID mapping

- **Single responsibility** — `parser` only lowers Markdown; `lower.rs` only emits
  Typst; `security` only decides about assets.
- **Open/closed** — new output formats are *added* as `Renderer` impls; nothing in
  the parser or IR changes. New themes are *data* (`ThemeSpec`), not new code paths.
- **Liskov** — every `Renderer` honours the same `render(&Document, &RenderOptions)
  -> Rendered` contract; the CLI treats them interchangeably via `for_format`.
- **Interface segregation** — `Renderer` is a single, minimal trait; options are a
  plain struct, not a god-object.
- **Dependency inversion** — the CLI depends on the `Renderer` abstraction and
  `OutputFormat`, never on `typst` directly. Only `core` pulls the Typst crates.

## Extension points

### Add an output format (e.g. DOCX)

1. Add `OutputFormat::Docx` and its extension in `render/mod.rs`.
2. Create `render/docx/mod.rs` implementing `Renderer` over the IR.
3. Register it in `render::for_format`.

No change to `parser`, `ir`, or the CLI argument layer beyond exposing the flag.
`render/html/` is a worked example of a third back-end sharing zero code with
the PDF path except the IR.

### Add a theme

Return a new `ThemeSpec` from `Theme::spec` (or, once TOML themes land, deserialize
one and override a base spec's fields). Renderers consume `ThemeSpec` fields and
never branch on the theme name.

### Add a Markdown feature

Extend the IR if needed, map it in `parser.rs`, then handle it in each renderer.
The compiler's exhaustiveness checks make sure no back-end silently forgets it.

## Why Typst (not a browser or LaTeX)

Typst is a Rust library, so it links straight into the binary, compiles in
milliseconds, and lets us embed fonts — giving byte-reproducible PDFs with no
system dependencies. A `Document` could equally drive a `wkhtmltopdf`/Chromium or
LaTeX back-end, but those reintroduce the heavyweight runtime deps md2pdf exists to
avoid.
