<!-- Thanks for contributing! Keep the parser and renderers decoupled via the IR. -->

## What this does

Brief description and the issue it closes (e.g. `Closes #12`).

## Checklist

- [ ] `cargo fmt --all --check` passes
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes
- [ ] `cargo test` passes
- [ ] No `comrak` types outside `parser.rs`; no `typst` types outside `render/typst`
- [ ] Tests added/updated
- [ ] CHANGELOG.md updated under `[Unreleased]` (for user-facing changes)
