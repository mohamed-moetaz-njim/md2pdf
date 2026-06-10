//! `md2pdf convert` — the default action.

use std::path::Path;

use anyhow::{Context, Result};
use md2pdf_core::render::OutputFormat;
use md2pdf_core::{Config, Paper, RenderOptions, SecurityPolicy, Theme};

use crate::args::ConvertArgs;

/// Resolve a theme argument: a built-in name, or a path to a .toml theme
/// (tried as given, then relative to the document root).
fn resolve_theme(name: &str, root: &Path) -> Result<md2pdf_core::Theme> {
    use md2pdf_core::Theme;
    if let Some(t) = Theme::from_name(name) {
        return Ok(t);
    }
    if name.ends_with(".toml") {
        let direct = Path::new(name);
        let path = if direct.exists() {
            direct.to_path_buf()
        } else {
            root.join(name)
        };
        return Theme::load(&path);
    }
    anyhow::bail!("unknown theme: {name} (built-ins: default, book; or a .toml file)")
}

pub fn run(args: ConvertArgs) -> Result<()> {
    let input = args
        .input
        .clone()
        .context("no input file given (try `md2pdf <file.md>` or `md2pdf --help`)")?;
    if input.as_os_str() != "-" && input.is_dir() {
        return run_dir(args, &input);
    }
    run_one(args)
}

/// Convert every Markdown file under `dir` (recursively, skipping dot-dirs).
///
/// With `-o <DIR>` outputs mirror the input tree under that directory;
/// otherwise each file converts alongside its source.
fn run_dir(args: ConvertArgs, dir: &Path) -> Result<()> {
    if args.output.as_ref().is_some_and(|o| o.as_os_str() == "-") {
        anyhow::bail!("cannot stream a directory conversion to stdout");
    }
    let mut files = Vec::new();
    collect_markdown(dir, &mut files)?;
    if files.is_empty() {
        anyhow::bail!("no Markdown files found under {}", dir.display());
    }
    let ext = args
        .format
        .map(OutputFormat::from)
        .unwrap_or(OutputFormat::Pdf)
        .extension();

    let mut failures = 0usize;
    for file in &files {
        let mut file_args = args.clone();
        file_args.input = Some(file.clone());
        file_args.output = match &args.output {
            Some(out_dir) => {
                let rel = file.strip_prefix(dir).expect("file is under dir");
                let target = out_dir.join(rel).with_extension(ext);
                if let Some(parent) = target.parent() {
                    std::fs::create_dir_all(parent)
                        .with_context(|| format!("could not create {}", parent.display()))?;
                }
                Some(target)
            }
            None => None,
        };
        if let Err(e) = run_one(file_args) {
            eprintln!("error: {}: {e:#}", file.display());
            failures += 1;
        }
    }
    if failures > 0 {
        anyhow::bail!("{failures} of {} file(s) failed", files.len());
    }
    eprintln!("converted {} file(s)", files.len());
    Ok(())
}

fn collect_markdown(dir: &Path, out: &mut Vec<std::path::PathBuf>) -> Result<()> {
    let mut entries: Vec<_> = std::fs::read_dir(dir)
        .with_context(|| format!("could not read {}", dir.display()))?
        .collect::<std::io::Result<_>>()?;
    // Sorted traversal keeps output and error ordering deterministic.
    entries.sort_by_key(|e| e.file_name());
    for entry in entries {
        let path = entry.path();
        let name = entry.file_name();
        if name.to_string_lossy().starts_with('.') {
            continue;
        }
        if path.is_dir() {
            collect_markdown(&path, out)?;
        } else if path
            .extension()
            .is_some_and(|e| e == "md" || e == "markdown")
        {
            out.push(path);
        }
    }
    Ok(())
}

fn run_one(args: ConvertArgs) -> Result<()> {
    let input = args
        .input
        .clone()
        .context("no input file given (try `md2pdf <file.md>` or `md2pdf --help`)")?;

    // `md2pdf -` reads Markdown from stdin (assets resolve against the cwd).
    let from_stdin = input.as_os_str() == "-";
    let root = if from_stdin {
        std::path::PathBuf::from(".")
    } else {
        super::doc_root(&input)
    };

    // Layer 1: built-in defaults.
    let mut security = SecurityPolicy::strict(&root);
    let mut opts = RenderOptions {
        theme: Theme::Default,
        paper: Paper::A4,
        toc: false,
        title: None,
        layout: Default::default(),
        security: security.clone(),
    };

    // Layer 2: config file (if present).
    let config = match &args.config {
        Some(path) => Some(Config::load(path)?),
        None => Config::load_from(&root).transpose()?,
    };
    if let Some(ref cfg) = config {
        cfg.apply_to_security(&mut security);
        cfg.apply_to_render_options(&mut opts);
        opts.security = security.clone();
        cfg.validate().unwrap_or_else(|e| {
            eprintln!("warning: config: {e}");
        });
    }

    // Custom (.toml) themes named in the config are resolved here, since
    // resolution needs the document root and may fail.
    if let Some(name) = config
        .as_ref()
        .and_then(|c| c.document.as_ref())
        .and_then(|d| d.theme.as_deref())
        && Theme::from_name(name).is_none()
        && name.ends_with(".toml")
    {
        opts.theme = resolve_theme(name, &root)?;
    }

    // Layer 3: CLI flags (highest priority).
    if let Some(theme) = &args.theme {
        opts.theme = resolve_theme(theme, &root)?;
    }
    if let Some(paper) = args.paper {
        opts.paper = paper.into();
    }

    // Only touch the config/default value when a flag was actually passed,
    // so `toc = true` in md2pdf.toml survives a plain `md2pdf doc.md`.
    if args.toc {
        opts.toc = true;
    } else if args.no_toc {
        opts.toc = false;
    }
    if args.title.is_some() {
        opts.title = args.title.clone();
    }
    if args.header.is_some() {
        opts.layout.header = args.header.clone();
    }
    if args.footer.is_some() {
        opts.layout.footer = args.footer.clone();
    }
    if args.page_numbers {
        opts.layout.page_numbers = true;
    } else if args.no_page_numbers {
        opts.layout.page_numbers = false;
    }

    let markdown = if from_stdin {
        super::read_stdin(&security)?
    } else {
        super::read_input(&input, &security)?
    };
    opts.security = security;

    let format = args
        .format
        .map(OutputFormat::from)
        .or_else(|| {
            args.output
                .as_deref()
                .and_then(Path::extension)
                .and_then(|e| OutputFormat::from_extension(&e.to_string_lossy()))
        })
        .unwrap_or(OutputFormat::Pdf);

    let rendered = md2pdf_core::convert(&markdown, &opts, format)?;

    for d in &rendered.diagnostics {
        eprintln!("warning: {}", d.message);
    }

    // `-o -` (or stdin input with no -o) streams the result to stdout.
    let to_stdout = match &args.output {
        Some(path) => path.as_os_str() == "-",
        None => from_stdin,
    };
    if to_stdout {
        use std::io::Write;
        std::io::stdout()
            .write_all(&rendered.bytes)
            .context("could not write to stdout")?;
        eprintln!("wrote stdout ({} bytes)", rendered.bytes.len());
        return Ok(());
    }

    let output = args
        .output
        .unwrap_or_else(|| input.with_extension(format.extension()));
    std::fs::write(&output, &rendered.bytes)
        .with_context(|| format!("could not write {}", output.display()))?;
    eprintln!(
        "wrote {} ({} bytes)",
        output.display(),
        rendered.bytes.len()
    );
    Ok(())
}
