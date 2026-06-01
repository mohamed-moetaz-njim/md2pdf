# Installation

md2pdf is a single self-contained binary. Pick the row that matches your platform.

## Installation matrix

| Platform | Method | Command | Status |
|:---------|:-------|:--------|:-------|
| Fedora / RHEL | `.rpm` | `sudo dnf install ./md2pdf-*.x86_64.rpm` | ✅ via Releases |
| Debian / Ubuntu | `.deb` | `sudo apt install ./md2pdf_*_amd64.deb` | ✅ via Releases |
| Any Linux | tarball | download `md2pdf-*-x86_64-linux.tar.gz`, put `md2pdf` on `$PATH` | ✅ via Releases |
| Any (Rust) | from git | `cargo install --git https://github.com/mohamed-moetaz-njim/md2pdf md2pdf` | ✅ available |
| Any (Rust) | crates.io | `cargo install md2pdf` | ⏳ after first crates.io publish |
| CI (GitHub Actions) | Action | `uses: mohamed-moetaz-njim/md2pdf@v0` | ⏳ after a `v0` tag is published |
| Fedora / RHEL | COPR | `sudo dnf copr enable mohamed-moetaz-njim/md2pdf && sudo dnf install md2pdf` | 🚧 planned, not yet available |

`.deb`, `.rpm` and tarballs are attached to each
[GitHub Release](https://github.com/mohamed-moetaz-njim/md2pdf/releases). Rows marked
⏳/🚧 are not usable yet — see [docs/PROGRAM.md](PROGRAM.md) for the activation steps.

## Verify

```bash
md2pdf --version
md2pdf doctor      # checks fonts + rendering in your environment
```

## Build from source

Requires a recent stable Rust (see `rust-version` in `Cargo.toml`). No system
libraries are needed — fonts and the rendering engine are vendored.

```bash
git clone https://github.com/mohamed-moetaz-njim/md2pdf
cd md2pdf
cargo build --release -p md2pdf
./target/release/md2pdf --help
```

## Uninstall

- rpm/dnf: `sudo dnf remove md2pdf`
- apt: `sudo apt remove md2pdf`
- cargo: `cargo uninstall md2pdf`
