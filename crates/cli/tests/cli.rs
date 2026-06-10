use std::path::PathBuf;

use predicates::prelude::*;
use tempfile::TempDir;

/// Helper: build the `md2pdf` binary command.
fn cmd() -> assert_cmd::Command {
    assert_cmd::Command::cargo_bin("md2pdf").unwrap()
}

/// Path to the sample markdown file used in tests.
fn sample_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/sample.md")
        .canonicalize()
        .unwrap()
}

/// Simple markdown input for quick tests.
const HELLO_MD: &str = "# Hello\n\nTest paragraph.\n";

// ---------------------------------------------------------------------------
// convert
// ---------------------------------------------------------------------------

#[test]
fn convert_basic() {
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("out.pdf");

    cmd()
        .arg(sample_path())
        .arg("-o")
        .arg(&out)
        .assert()
        .success()
        .stderr(predicate::str::contains("wrote"));

    assert!(out.exists(), "output file must exist");
    let magic = std::fs::read(&out).unwrap();
    assert_eq!(&magic[..5], b"%PDF-", "output must be a PDF");
}

#[test]
fn convert_default_output_path() {
    // When -o is omitted, the output should be <input>.pdf.
    let dir = TempDir::new().unwrap();
    let input = dir.path().join("doc.md");
    std::fs::write(&input, HELLO_MD).unwrap();

    cmd().arg(&input).assert().success();

    let expected = dir.path().join("doc.pdf");
    assert!(expected.exists());
}

#[test]
fn convert_with_title() {
    let dir = TempDir::new().unwrap();
    let input = dir.path().join("doc.md");
    let out = dir.path().join("out.pdf");
    std::fs::write(&input, HELLO_MD).unwrap();

    cmd()
        .arg(&input)
        .arg("-o")
        .arg(&out)
        .arg("--title")
        .arg("Integration Test")
        .assert()
        .success();

    assert!(out.exists());
}

#[test]
fn convert_with_toc() {
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("toc.pdf");

    cmd()
        .arg(sample_path())
        .arg("-o")
        .arg(&out)
        .arg("--toc")
        .assert()
        .success();

    // With TOC the output should still be valid PDF (different size).
    assert!(out.exists());
    let magic = std::fs::read(&out).unwrap();
    assert_eq!(&magic[..5], b"%PDF-");
}

#[test]
fn convert_format_typst() {
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("out.typ");

    cmd()
        .arg(sample_path())
        .arg("-o")
        .arg(&out)
        .assert()
        .success();

    assert!(out.exists(), "output .typ file must exist");
    let content = std::fs::read_to_string(&out).unwrap();
    assert!(
        content.contains("#set page"),
        "Typst source must contain preamble"
    );
}

#[test]
fn convert_format_explicit() {
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("output");

    cmd()
        .arg(sample_path())
        .arg("-o")
        .arg(&out)
        .arg("--format")
        .arg("typst")
        .assert()
        .success()
        .stderr(predicate::str::contains("wrote"));

    // Explicit -o path with no extension is kept as-is when --format is given.
    assert!(out.exists(), "output file must exist at explicit path");
    let content = std::fs::read_to_string(&out).unwrap();
    assert!(content.contains("#set page"), "must produce Typst source");
}

#[test]
fn convert_theme_book() {
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("book.pdf");

    cmd()
        .arg(sample_path())
        .arg("-o")
        .arg(&out)
        .arg("--theme")
        .arg("book")
        .assert()
        .success();

    assert!(out.exists());
    let magic = std::fs::read(&out).unwrap();
    assert_eq!(&magic[..5], b"%PDF-");
}

#[test]
fn convert_paper_letter() {
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("letter.pdf");

    cmd()
        .arg(sample_path())
        .arg("-o")
        .arg(&out)
        .arg("--paper")
        .arg("letter")
        .assert()
        .success();

    assert!(out.exists());
    let magic = std::fs::read(&out).unwrap();
    assert_eq!(&magic[..5], b"%PDF-");
}

#[test]
fn convert_missing_input() {
    cmd()
        .arg("/tmp/nonexistent-md2pdf-test-file.md")
        .assert()
        .failure()
        .stderr(predicate::str::contains("could not open"));
}

#[test]
fn convert_no_input_shows_help() {
    cmd()
        .arg("convert")
        .assert()
        .failure()
        .stderr(predicate::str::contains("no input file given"));
}

// ---------------------------------------------------------------------------
// convert --config
// ---------------------------------------------------------------------------

#[test]
fn convert_with_config_flag() {
    let dir = TempDir::new().unwrap();
    let input = dir.path().join("doc.md");
    let config = dir.path().join("config.toml");
    let out = dir.path().join("out.pdf");

    std::fs::write(
        &config,
        r#"[document]
title = "Config Test"
paper = "letter"
"#,
    )
    .unwrap();
    std::fs::write(&input, HELLO_MD).unwrap();

    cmd()
        .arg(&input)
        .arg("-o")
        .arg(&out)
        .arg("--config")
        .arg(&config)
        .assert()
        .success();

    assert!(out.exists());
}

#[test]
fn convert_config_auto_detect() {
    let dir = TempDir::new().unwrap();
    let input = dir.path().join("doc.md");
    let config = dir.path().join("md2pdf.toml");
    let out = dir.path().join("out.pdf");

    std::fs::write(
        &config,
        r#"[document]
paper = "letter"
"#,
    )
    .unwrap();
    std::fs::write(&input, HELLO_MD).unwrap();

    cmd().arg(&input).arg("-o").arg(&out).assert().success();

    assert!(out.exists());
}

#[test]
fn convert_config_cli_overrides() {
    let dir = TempDir::new().unwrap();
    let input = dir.path().join("doc.md");
    let config = dir.path().join("md2pdf.toml");
    let out = dir.path().join("out.pdf");

    // Config sets theme=book, but CLI --theme default should win.
    std::fs::write(
        &config,
        r#"[document]
theme = "book"
"#,
    )
    .unwrap();
    std::fs::write(&input, HELLO_MD).unwrap();

    cmd()
        .arg(&input)
        .arg("-o")
        .arg(&out)
        .arg("--theme")
        .arg("default")
        .assert()
        .success();

    assert!(out.exists());
}

#[test]
fn convert_config_toc_applies() {
    // Regression: `toc = true` in md2pdf.toml must not be clobbered by the
    // CLI default when no --toc/--no-toc flag is passed.
    let dir = TempDir::new().unwrap();
    let input = dir.path().join("doc.md");
    let config = dir.path().join("md2pdf.toml");
    let out = dir.path().join("out.typ");

    std::fs::write(
        &config,
        r#"[document]
toc = true
"#,
    )
    .unwrap();
    std::fs::write(&input, HELLO_MD).unwrap();

    cmd().arg(&input).arg("-o").arg(&out).assert().success();

    let content = std::fs::read_to_string(&out).unwrap();
    assert!(
        content.contains("#outline"),
        "config toc = true must produce an outline"
    );
}

#[test]
fn convert_no_toc_overrides_config() {
    let dir = TempDir::new().unwrap();
    let input = dir.path().join("doc.md");
    let config = dir.path().join("md2pdf.toml");
    let out = dir.path().join("out.typ");

    std::fs::write(
        &config,
        r#"[document]
toc = true
"#,
    )
    .unwrap();
    std::fs::write(&input, HELLO_MD).unwrap();

    cmd()
        .arg(&input)
        .arg("-o")
        .arg(&out)
        .arg("--no-toc")
        .assert()
        .success();

    let content = std::fs::read_to_string(&out).unwrap();
    assert!(
        !content.contains("#outline"),
        "--no-toc must override config toc = true"
    );
}

#[test]
fn convert_header_footer_renders_pdf() {
    // End-to-end: the page-furniture Typst we generate must actually compile.
    let dir = TempDir::new().unwrap();
    let input = dir.path().join("doc.md");
    let out = dir.path().join("out.pdf");
    std::fs::write(
        &input,
        "---\ntitle: Spec\nauthor: Ada\n---\n\n# H\n\nbody\n",
    )
    .unwrap();

    cmd()
        .arg(&input)
        .arg("-o")
        .arg(&out)
        .arg("--header")
        .arg("{title} — confidential")
        .arg("--footer")
        .arg("by {author}")
        .assert()
        .success();

    let magic = std::fs::read(&out).unwrap();
    assert_eq!(&magic[..5], b"%PDF-");
}

#[test]
fn convert_no_page_numbers_renders_pdf() {
    let dir = TempDir::new().unwrap();
    let input = dir.path().join("doc.md");
    let out = dir.path().join("out.pdf");
    std::fs::write(&input, HELLO_MD).unwrap();

    cmd()
        .arg(&input)
        .arg("-o")
        .arg(&out)
        .arg("--no-page-numbers")
        .assert()
        .success();

    assert!(out.exists());
}

#[test]
fn convert_config_layout_applies() {
    let dir = TempDir::new().unwrap();
    let input = dir.path().join("doc.md");
    let config = dir.path().join("md2pdf.toml");
    let out = dir.path().join("out.typ");

    std::fs::write(
        &config,
        r#"[layout]
header = "Internal"
page_numbers = false
"#,
    )
    .unwrap();
    std::fs::write(&input, HELLO_MD).unwrap();

    cmd().arg(&input).arg("-o").arg(&out).assert().success();

    let content = std::fs::read_to_string(&out).unwrap();
    assert!(content.contains("Internal"), "config header must apply");
    assert!(
        !content.contains("numbering: \"1\""),
        "config page_numbers = false must apply"
    );
}

#[test]
fn convert_config_invalid_path() {
    cmd()
        .arg(sample_path())
        .arg("-o")
        .arg("/tmp/_md2pdf_test_invalid_config.pdf")
        .arg("--config")
        .arg("/nonexistent/md2pdf.toml")
        .assert()
        .failure()
        .stderr(predicate::str::contains("could not read config"));
}

#[test]
fn convert_image_width_lowered() {
    let dir = TempDir::new().unwrap();
    let input = dir.path().join("doc.md");
    let out = dir.path().join("out.typ");
    // Typst-source output only stats the file, so the content is irrelevant.
    std::fs::write(dir.path().join("logo.png"), b"png-bytes").unwrap();
    std::fs::write(&input, "# Doc\n\n![logo](logo.png){width=40%}\n").unwrap();

    cmd().arg(&input).arg("-o").arg(&out).assert().success();

    let content = std::fs::read_to_string(&out).unwrap();
    assert!(
        content.contains("width: 40%"),
        "image width must reach the Typst source: {content}"
    );
}

#[test]
fn convert_admonition_renders_pdf() {
    let dir = TempDir::new().unwrap();
    let input = dir.path().join("doc.md");
    let out = dir.path().join("out.pdf");
    std::fs::write(
        &input,
        "# Doc\n\n> [!TIP]\n> Use `--toc` for long documents.\n",
    )
    .unwrap();

    cmd().arg(&input).arg("-o").arg(&out).assert().success();

    let magic = std::fs::read(&out).unwrap();
    assert_eq!(&magic[..5], b"%PDF-");
}

// ---------------------------------------------------------------------------
// validate
// ---------------------------------------------------------------------------

#[test]
fn validate_ok() {
    cmd()
        .arg("validate")
        .arg(sample_path())
        .assert()
        .success()
        .stdout(predicate::str::contains("ok"));
}

#[test]
fn validate_multiple_files() {
    let dir = TempDir::new().unwrap();
    let a = dir.path().join("a.md");
    let b = dir.path().join("b.md");
    std::fs::write(&a, "# A\n\ntext\n").unwrap();
    std::fs::write(&b, "# B\n\ntext\n").unwrap();

    cmd()
        .arg("validate")
        .arg(&a)
        .arg(&b)
        .assert()
        .success()
        .stdout(predicate::str::contains("a.md"))
        .stdout(predicate::str::contains("b.md"));
}

#[test]
fn validate_strict_fails_on_warnings() {
    let dir = TempDir::new().unwrap();
    let input = dir.path().join("doc.md");
    // A remote image is denied by policy and produces a warning.
    std::fs::write(&input, "# Doc\n\n![x](https://example.com/x.png)\n").unwrap();

    // Without --strict, warnings do not fail the run.
    cmd().arg("validate").arg(&input).assert().success();

    cmd()
        .arg("validate")
        .arg("--strict")
        .arg(&input)
        .assert()
        .failure()
        .stderr(predicate::str::contains("strict"));
}

#[test]
fn validate_missing_input() {
    cmd()
        .arg("validate")
        .arg("/tmp/nonexistent-md2pdf-test-file.md")
        .assert()
        .failure()
        .stderr(predicate::str::contains("could not open"));
}

// ---------------------------------------------------------------------------
// doctor
// ---------------------------------------------------------------------------

#[test]
fn doctor_ok() {
    cmd()
        .arg("doctor")
        .assert()
        .success()
        .stdout(predicate::str::contains("md2pdf"))
        .stdout(predicate::str::contains("engine"))
        .stdout(predicate::str::contains("render"));
}

// ---------------------------------------------------------------------------
// init
// ---------------------------------------------------------------------------

#[test]
fn init_creates_files() {
    let dir = TempDir::new().unwrap();

    cmd()
        .arg("init")
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Scaffolded"));

    assert!(dir.path().join("md2pdf.toml").exists(), "md2pdf.toml");
    assert!(dir.path().join("docs/example.md").exists(), "example.md");
    assert!(
        dir.path().join(".github/workflows/docs.yml").exists(),
        "docs.yml"
    );
}

#[test]
fn init_idempotent() {
    let dir = TempDir::new().unwrap();

    // First run creates everything.
    cmd().arg("init").arg(dir.path()).assert().success();

    // Second run should complete without error (skips existing files).
    cmd()
        .arg("init")
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("skipped"));
}

#[test]
fn init_defaults_to_current_dir() {
    // Running `md2pdf init` without a path should create files in cwd.
    let dir = TempDir::new().unwrap();
    let cwd = std::env::current_dir().unwrap();
    std::env::set_current_dir(dir.path()).unwrap();

    cmd().arg("init").assert().success();

    assert!(dir.path().join("md2pdf.toml").exists());
    assert!(dir.path().join("docs/example.md").exists());

    std::env::set_current_dir(cwd).unwrap();
}

// ---------------------------------------------------------------------------
// theme
// ---------------------------------------------------------------------------

#[test]
fn theme_create_and_use() {
    let dir = TempDir::new().unwrap();
    let input = dir.path().join("doc.md");
    let out = dir.path().join("out.typ");
    let theme = dir.path().join("corporate.toml");

    std::fs::write(
        &theme,
        r##"base = "default"
heading_color = "#aa0000"
body_font = "New Computer Modern"
"##,
    )
    .unwrap();
    std::fs::write(&input, HELLO_MD).unwrap();

    cmd()
        .arg(&input)
        .arg("-o")
        .arg(&out)
        .arg("--theme")
        .arg(&theme)
        .assert()
        .success();

    let content = std::fs::read_to_string(&out).unwrap();
    assert!(content.contains("#aa0000"), "custom color: {content}");
    assert!(
        content.contains("New Computer Modern"),
        "custom font: {content}"
    );
}

#[test]
fn theme_custom_renders_pdf() {
    // A custom theme must also survive full Typst compilation.
    let dir = TempDir::new().unwrap();
    let input = dir.path().join("doc.md");
    let out = dir.path().join("out.pdf");
    let theme = dir.path().join("t.toml");
    std::fs::write(&theme, "base = \"book\"\nbody_size_pt = 12.0\n").unwrap();
    std::fs::write(&input, HELLO_MD).unwrap();

    cmd()
        .arg(&input)
        .arg("-o")
        .arg(&out)
        .arg("--theme")
        .arg(&theme)
        .assert()
        .success();

    let magic = std::fs::read(&out).unwrap();
    assert_eq!(&magic[..5], b"%PDF-");
}

#[test]
fn theme_unknown_fails() {
    cmd()
        .arg(sample_path())
        .arg("--theme")
        .arg("sparkly")
        .assert()
        .failure()
        .stderr(predicate::str::contains("unknown theme"));
}

#[test]
fn theme_create_subcommand() {
    let dir = TempDir::new().unwrap();
    let cwd = std::env::current_dir().unwrap();
    std::env::set_current_dir(dir.path()).unwrap();

    cmd()
        .arg("theme")
        .arg("create")
        .arg("mytheme")
        .assert()
        .success()
        .stdout(predicate::str::contains("mytheme.toml"));

    assert!(dir.path().join("mytheme.toml").exists());
    std::env::set_current_dir(cwd).unwrap();
}

#[test]
fn theme_list() {
    cmd()
        .arg("theme")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("default"))
        .stdout(predicate::str::contains("book"));
}

// ---------------------------------------------------------------------------
// no subcommand
// ---------------------------------------------------------------------------

#[test]
fn no_args_shows_help() {
    cmd()
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage"));
}
