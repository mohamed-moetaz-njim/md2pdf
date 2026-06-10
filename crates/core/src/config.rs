use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::render::{Paper, RenderOptions};
use crate::security::SecurityPolicy;
use crate::theme::Theme;

/// Top-level md2pdf.toml configuration file.
///
/// All sections and fields are optional. Values here serve as defaults;
/// CLI flags override them.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default)]
    pub document: Option<DocumentConfig>,
    #[serde(default)]
    pub security: Option<SecurityConfig>,
    #[serde(default)]
    pub layout: Option<LayoutConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentConfig {
    pub theme: Option<String>,
    pub paper: Option<String>,
    pub toc: Option<bool>,
    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SecurityConfig {
    pub max_input_bytes: Option<u64>,
    pub max_image_bytes: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LayoutConfig {
    pub header: Option<String>,
    pub footer: Option<String>,
    pub page_numbers: Option<bool>,
}

impl Config {
    /// Load a config from the given path.
    pub fn load(path: &Path) -> anyhow::Result<Config> {
        let contents = std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("could not read config {}: {e}", path.display()))?;
        let config: Config = toml::from_str(&contents)
            .map_err(|e| anyhow::anyhow!("invalid config {}: {e}", path.display()))?;
        Ok(config)
    }

    /// Try to load `md2pdf.toml` from `dir`, returning `None` if it does not exist.
    pub fn load_from(dir: &Path) -> Option<anyhow::Result<Config>> {
        let path = dir.join("md2pdf.toml");
        if path.exists() {
            Some(Config::load(&path))
        } else {
            None
        }
    }

    /// Apply document-level config to render options.
    ///
    /// Only overrides fields that are `Some` in the config; `None` fields
    /// leave the existing value intact. Call this before applying CLI flags.
    pub fn apply_to_render_options(&self, opts: &mut RenderOptions) {
        if let Some(doc) = &self.document {
            if let Some(theme) = &doc.theme
                && let Some(t) = Theme::from_name(theme)
            {
                opts.theme = t;
            }
            if let Some(paper) = &doc.paper {
                opts.paper = Paper::from_name(paper).unwrap_or(opts.paper);
            }
            if let Some(toc) = doc.toc {
                opts.toc = toc;
            }
            if doc.title.is_some() {
                opts.title = doc.title.clone();
            }
        }
        if let Some(layout) = &self.layout {
            if layout.header.is_some() {
                opts.layout.header = layout.header.clone();
            }
            if layout.footer.is_some() {
                opts.layout.footer = layout.footer.clone();
            }
            if let Some(pn) = layout.page_numbers {
                opts.layout.page_numbers = pn;
            }
        }
    }

    /// Apply security config to a security policy.
    pub fn apply_to_security(&self, policy: &mut SecurityPolicy) {
        let Some(sec) = &self.security else {
            return;
        };
        if let Some(v) = sec.max_input_bytes {
            policy.max_input_bytes = v;
        }
        if let Some(v) = sec.max_image_bytes {
            policy.max_image_bytes = v;
        }
    }

    /// Validate the config, returning an error for invalid values.
    pub fn validate(&self) -> anyhow::Result<()> {
        if let Some(doc) = &self.document {
            // Custom themes are file paths, resolved (and validated)
            // against the document root by the CLI.
            if let Some(theme) = &doc.theme
                && Theme::from_name(theme).is_none()
                && !theme.ends_with(".toml")
            {
                anyhow::bail!("unknown theme: {theme}");
            }
            if let Some(paper) = &doc.paper
                && Paper::from_name(paper).is_none()
            {
                anyhow::bail!("unknown paper size: {paper}");
            }
        }
        Ok(())
    }

    /// Emit a sample `md2pdf.toml` with all fields commented out.
    pub fn sample() -> &'static str {
        r##"# md2pdf configuration file.
# Place this file in your project root to set document defaults.
# CLI flags override values in this file.

# [document]
# theme = "default"
# paper = "a4"
# toc = false
# title = "My Document"

# [security]
# max_input_bytes = 16777216
# max_image_bytes = 33554432

# Header and footer accept {title}, {author} and {date} placeholders,
# resolved from the document's frontmatter.
# [layout]
# header = "{title}"
# footer = "{author}"
# page_numbers = true
"##
    }
}

impl Paper {
    /// Look up a paper size by its CLI/config name.
    pub fn from_name(name: &str) -> Option<Paper> {
        match name {
            "a4" => Some(Paper::A4),
            "a5" => Some(Paper::A5),
            "letter" => Some(Paper::Letter),
            "legal" => Some(Paper::Legal),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::Paper;
    use crate::theme::Theme;

    #[test]
    fn config_sample_is_valid_toml() {
        let contents = Config::sample();
        let config: Config = toml::from_str(contents).expect("sample config must parse");
        // All fields are commented out, so everything should be None.
        assert!(config.document.is_none());
        assert!(config.security.is_none());
        assert!(config.layout.is_none());
    }

    #[test]
    fn config_round_trip() {
        let toml_str = r#"
[document]
theme = "book"
paper = "letter"
toc = true
title = "Test Document"

[security]
max_input_bytes = 8192
max_image_bytes = 65536

[layout]
header = "Confidential"
footer = "Page"
page_numbers = true
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(
            config.document.as_ref().unwrap().theme.as_deref(),
            Some("book")
        );
        assert_eq!(
            config.document.as_ref().unwrap().paper.as_deref(),
            Some("letter")
        );
        assert_eq!(config.document.as_ref().unwrap().toc, Some(true));
        assert_eq!(
            config.document.as_ref().unwrap().title.as_deref(),
            Some("Test Document")
        );
        assert_eq!(
            config.security.as_ref().unwrap().max_input_bytes,
            Some(8192)
        );
        assert_eq!(
            config.security.as_ref().unwrap().max_image_bytes,
            Some(65536)
        );
        assert_eq!(
            config.layout.as_ref().unwrap().header.as_deref(),
            Some("Confidential")
        );
        assert_eq!(config.layout.as_ref().unwrap().page_numbers, Some(true));
    }

    #[test]
    fn config_denies_unknown_fields() {
        let toml_str = r#"[document]
unknown = "value"
"#;
        let result: Result<Config, toml::de::Error> = toml::from_str(toml_str);
        assert!(result.is_err());
    }

    #[test]
    fn validate_theme() {
        let config = Config {
            document: Some(DocumentConfig {
                theme: Some("nonexistent".into()),
                paper: None,
                toc: None,
                title: None,
            }),
            security: None,
            layout: None,
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn validate_paper() {
        let config = Config {
            document: Some(DocumentConfig {
                theme: None,
                paper: Some("huge".into()),
                toc: None,
                title: None,
            }),
            security: None,
            layout: None,
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn validate_ok() {
        let config = Config {
            document: Some(DocumentConfig {
                theme: Some("book".into()),
                paper: Some("a4".into()),
                toc: Some(true),
                title: None,
            }),
            security: None,
            layout: None,
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn apply_to_render_options_overrides_some() {
        let mut opts = RenderOptions {
            theme: Theme::Default,
            paper: Paper::A4,
            toc: false,
            title: None,
            layout: Default::default(),
            security: SecurityPolicy::strict("."),
        };
        let config = Config {
            document: Some(DocumentConfig {
                theme: Some("book".into()),
                paper: Some("letter".into()),
                toc: Some(true),
                title: Some("Config Title".into()),
            }),
            security: None,
            layout: None,
        };
        config.apply_to_render_options(&mut opts);
        assert_eq!(opts.theme, Theme::Book);
        assert_eq!(opts.paper, Paper::Letter);
        assert!(opts.toc);
        assert_eq!(opts.title.as_deref(), Some("Config Title"));
    }

    #[test]
    fn apply_to_render_options_ignores_none() {
        let mut opts = RenderOptions {
            theme: Theme::Default,
            paper: Paper::A4,
            toc: false,
            title: Some("CLI Title".into()),
            layout: Default::default(),
            security: SecurityPolicy::strict("."),
        };
        let config = Config {
            document: Some(DocumentConfig {
                theme: None,
                paper: None,
                toc: None,
                title: None,
            }),
            security: None,
            layout: None,
        };
        config.apply_to_render_options(&mut opts);
        // No config fields set; opts unchanged.
        assert_eq!(opts.theme, Theme::Default);
        assert_eq!(opts.paper, Paper::A4);
        assert!(!opts.toc);
        assert_eq!(opts.title.as_deref(), Some("CLI Title"));
    }

    #[test]
    fn apply_to_security_overrides() {
        let mut policy = SecurityPolicy::strict(".");
        assert_eq!(policy.max_input_bytes, 16 * 1024 * 1024);
        let config = Config {
            document: None,
            security: Some(SecurityConfig {
                max_input_bytes: Some(4096),
                max_image_bytes: Some(8192),
            }),
            layout: None,
        };
        config.apply_to_security(&mut policy);
        assert_eq!(policy.max_input_bytes, 4096);
        assert_eq!(policy.max_image_bytes, 8192);
    }

    #[test]
    fn empty_config_is_valid() {
        let config = Config {
            document: None,
            security: None,
            layout: None,
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn paper_from_name() {
        assert_eq!(Paper::from_name("a4"), Some(Paper::A4));
        assert_eq!(Paper::from_name("a5"), Some(Paper::A5));
        assert_eq!(Paper::from_name("letter"), Some(Paper::Letter));
        assert_eq!(Paper::from_name("legal"), Some(Paper::Legal));
        assert_eq!(Paper::from_name("tabloid"), None);
    }
}
