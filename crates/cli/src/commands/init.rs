//! `md2pdf init` — scaffold a documentation project.

use std::path::Path;

use anyhow::{Context, Result};

const EXAMPLE_MD: &str = "---\ntitle: Project Documentation\nauthor: Your Name\ndate: 2026\n---\n\n# Introduction\n\nWrite your documentation in Markdown and run `md2pdf docs/example.md` to get a\nPDF. Tables, code blocks, task lists and footnotes all work out of the box.\n\n## Next steps\n\n- [ ] Replace this with your content\n- [ ] Push and let the workflow build the PDF on every release\n";

const WORKFLOW_YML: &str = "name: docs\n\non:\n  push:\n    branches: [main]\n\njobs:\n  pdf:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@v4\n      - uses: mohamed-moetaz-njim/md2pdf@v0\n        with:\n          input: docs/example.md\n          output: docs/example.pdf\n      - uses: actions/upload-artifact@v4\n        with:\n          name: docs-pdf\n          path: docs/example.pdf\n";

pub fn run(dir: &Path) -> Result<()> {
    let docs = dir.join("docs");
    let workflows = dir.join(".github/workflows");
    std::fs::create_dir_all(&docs).with_context(|| format!("could not create {}", docs.display()))?;
    std::fs::create_dir_all(&workflows)
        .with_context(|| format!("could not create {}", workflows.display()))?;

    write_if_absent(&docs.join("example.md"), EXAMPLE_MD)?;
    write_if_absent(&workflows.join("docs.yml"), WORKFLOW_YML)?;

    println!("Scaffolded a documentation project in {}", dir.display());
    println!("  docs/example.md            sample document");
    println!("  .github/workflows/docs.yml build the PDF in CI");
    println!("\nTry it now:  md2pdf docs/example.md");
    Ok(())
}

fn write_if_absent(path: &Path, contents: &str) -> Result<()> {
    if path.exists() {
        println!("  skipped {} (already exists)", path.display());
        return Ok(());
    }
    std::fs::write(path, contents).with_context(|| format!("could not write {}", path.display()))?;
    println!("  created {}", path.display());
    Ok(())
}
