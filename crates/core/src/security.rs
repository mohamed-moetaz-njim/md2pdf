//! Security policy and threat-model enforcement.
//!
//! md2pdf is frequently pointed at untrusted Markdown in CI. The defaults are
//! therefore deny-by-default:
//!
//! * **No remote resources.** `http(s)` images are never fetched. The Typst
//!   engine is built without the `packages` feature, so it cannot reach the
//!   network either.
//! * **No path traversal.** Local image paths are canonicalised and must stay
//!   inside the document root; `../../etc/passwd` and absolute paths are denied.
//! * **Bounded inputs.** Oversized Markdown and oversized images are rejected
//!   rather than buffered without limit.
//! * **Raw HTML is dropped**, never passed through to a back-end.

use std::path::{Path, PathBuf};

const DEFAULT_MAX_IMAGE_BYTES: u64 = 32 * 1024 * 1024;
const DEFAULT_MAX_INPUT_BYTES: u64 = 16 * 1024 * 1024;

/// The active security policy for a render.
#[derive(Debug, Clone)]
pub struct SecurityPolicy {
    /// Directory that local assets must resolve within.
    pub root: PathBuf,
    /// Allow `http(s)` resources to be referenced (still never fetched by the
    /// PDF back-end; reserved for future network-capable back-ends).
    pub allow_remote: bool,
    /// Maximum size of an embeddable image, in bytes.
    pub max_image_bytes: u64,
    /// Maximum size of the input Markdown, in bytes.
    pub max_input_bytes: u64,
}

/// What to do with a referenced image.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssetDecision {
    /// Safe to embed; carries the (root-relative) path to hand to the back-end.
    Allow(String),
    /// Rejected; carries a human-readable reason for diagnostics.
    Deny(String),
}

impl SecurityPolicy {
    /// The strict, recommended default policy rooted at `root`.
    pub fn strict(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            allow_remote: false,
            max_image_bytes: DEFAULT_MAX_IMAGE_BYTES,
            max_input_bytes: DEFAULT_MAX_INPUT_BYTES,
        }
    }

    /// Decide whether an image reference may be embedded.
    pub fn resolve_image(&self, src: &str) -> AssetDecision {
        if is_remote(src) {
            return AssetDecision::Deny(format!(
                "remote image not loaded (remote resources are disabled): {src}"
            ));
        }
        if Path::new(src).is_absolute() {
            return AssetDecision::Deny(format!("absolute image path denied: {src}"));
        }

        let candidate = self.root.join(src);
        let canonical = match candidate.canonicalize() {
            Ok(p) => p,
            Err(_) => return AssetDecision::Deny(format!("image not found: {src}")),
        };
        let root = self
            .root
            .canonicalize()
            .unwrap_or_else(|_| self.root.clone());
        if !canonical.starts_with(&root) {
            return AssetDecision::Deny(format!(
                "path traversal outside document root denied: {src}"
            ));
        }
        match std::fs::metadata(&canonical) {
            Ok(m) if m.len() > self.max_image_bytes => AssetDecision::Deny(format!(
                "image exceeds {} byte limit: {src}",
                self.max_image_bytes
            )),
            Ok(_) => AssetDecision::Allow(src.to_string()),
            Err(e) => AssetDecision::Deny(format!("cannot stat image {src}: {e}")),
        }
    }
}

fn is_remote(src: &str) -> bool {
    let lower = src.trim_start().to_ascii_lowercase();
    lower.starts_with("http://")
        || lower.starts_with("https://")
        || lower.starts_with("//")
        || lower.starts_with("ftp://")
        || lower.starts_with("data:")
}
