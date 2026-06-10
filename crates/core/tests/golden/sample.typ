#set page(paper: "a4", margin: (x: 2.2cm, y: 2.4cm), numbering: "1")
#set text(font: "Libertinus Serif", size: 11pt, lang: "en")
#set par(justify: true, leading: 0.65em, first-line-indent: 0em)
#show heading: set text(fill: rgb("#13315c"))
#show heading.where(level: 1): set text(size: 1.5em)
#show link: set text(fill: rgb("#1565c0"))
#show raw: set text(font: "DejaVu Sans Mono", size: 9.5pt)
#show raw.where(block: true): block.with(fill: rgb("#f4f6f8"), inset: 10pt, radius: 4pt, width: 100%, stroke: 0.5pt + rgb("#e1e4e8"))
#show quote.where(block: true): set block(stroke: (left: 2pt + rgb("#13315c")), inset: (left: 1em, y: 0.4em))

#align(center)[
  #text(size: 22pt, weight: "bold", fill: rgb("#13315c"))[#"md2pdf"]
  #v(0.3em)
  #line(length: 38%, stroke: 0.6pt + rgb("#13315c"))
]
#v(1.4em)

#outline(title: [Contents], indent: auto)
#v(1.2em)

#heading(level: 1)[#"md2pdf"]

#"A small, fast, "#strong[#"local"]#" Markdown to PDF converter. No browser, no LaTeX, no"#" "#"Python — just a single binary that turns this file into a clean PDF."

#heading(level: 2)[#"Why"]

#"Most Markdown-to-PDF tools drag in a whole headless browser or a full LaTeX"#" "#"distribution. "#raw("md2pdf")#" compiles everything in-process with "#link("https://typst.app")[#"Typst"]#" and"#" "#"ships the fonts inside the binary, so the only thing you install is one"#" "#"executable."

#heading(level: 2)[#"Features"]

#list(
  [#"CommonMark plus GitHub extensions"],
  [#"Tables, task lists and footnotes"#footnote[#"Footnotes render at the bottom of the page, just like this one."

]],
  [#"Syntax-friendly code blocks"],
  [#"Two themes and an optional table of contents"],
)

#heading(level: 3)[#"Inline formatting"]

#"You get "#strong[#"bold"]#", "#emph[#"italic"]#", "#strike[#"strikethrough"]#" and "#raw("inline code")#", plus"#" "#link("https://example.com")[#"links"]#" that stay clickable in the PDF."

#heading(level: 3)[#"A table"]

#table(
  columns: 3,
  align: (left, center, right, ),
  stroke: 0.5pt + luma(190),
  table.header([#"Feature"], [#"Supported"], [#"Notes"], ),
  [#"Tables"], [#"yes"], [#"with column alignment"], 
  [#"Code blocks"], [#"yes"], [#"monospace, boxed"], 
  [#"Footnotes"], [#"yes"], [#"numbered"], 
)

#heading(level: 3)[#"A code block"]

#raw(block: true, lang: "rust", "fn main() {\n    println!(\"rendered by md2pdf\");\n}\n")

#heading(level: 3)[#"A task list"]

#list(
  [#"☑ "#"Parse Markdown"],
  [#"☑ "#"Emit Typst"],
  [#"☐ "#"Conquer the world"],
)

#quote(block: true)[
#"Blockquotes are supported too — handy for callouts and citations."

]

#line(length: 100%, stroke: 0.5pt + luma(180))
#v(0.3em)

#"That's it. Run "#raw("md2pdf examples/sample.md")#" and open the result."

