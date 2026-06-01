# Installation

md2pdf is a single self-contained binary. Pick the row that matches your platform.

## Installation matrix

| Platform | Method | Command |
|:---------|:-------|:--------|
| Fedora / RHEL | COPR | `sudo dnf copr enable mohamed-moetaz-njim/md2pdf && sudo dnf install md2pdf` |
| Fedora / RHEL | `.rpm` | `sudo dnf install ./md2pdf-*.x86_64.rpm` |
| Debian / Ubuntu | `.deb` | `sudo apt install ./md2pdf_*_amd64.deb` |
| Any Linux | tarball | download `md2pdf-*-x86_64-linux.tar.gz`, put `md2pdf` on `$PATH` |
| Any (Rust) | crates.io | `cargo install md2pdf` |
| Any (Rust) | from git | `cargo install --git https://github.com/mohamed-moetaz-njim/md2pdf md2pdf` |
| CI (GitHub Actions) | Action | `uses: mohamed-moetaz-njim/md2pdf@v0` |

`.deb`, `.rpm` and tarballs are attached to every
[GitHub Release](https://github.com/mohamed-moetaz-njim/md2pdf/releases).

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

- COPR/dnf: `sudo dnf remove md2pdf`
- apt: `sudo apt remove md2pdf`
- cargo: `cargo uninstall md2pdf`
