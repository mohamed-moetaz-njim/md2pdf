//! Subcommand implementations.

pub mod convert;
pub mod doctor;
pub mod init;
pub mod theme;
pub mod validate;

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use md2pdf_core::SecurityPolicy;

/// Read an input file, enforcing the policy's input-size cap.
pub(crate) fn read_input(path: &Path, policy: &SecurityPolicy) -> Result<String> {
    let meta = std::fs::metadata(path)
        .with_context(|| format!("could not open {}", path.display()))?;
    if meta.len() > policy.max_input_bytes {
        anyhow::bail!(
            "{} is {} bytes, over the {} byte limit",
            path.display(),
            meta.len(),
            policy.max_input_bytes
        );
    }
    std::fs::read_to_string(path).with_context(|| format!("could not read {}", path.display()))
}

/// The directory used as the security/asset root for a document.
pub(crate) fn doc_root(input: &Path) -> PathBuf {
    input
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."))
}
