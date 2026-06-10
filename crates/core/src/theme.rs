//! Themes as data.
//!
//! A [`ThemeSpec`] is a flat bag of typographic knobs. Built-in themes are
//! constructed in code; custom themes are loaded from TOML files that start
//! from a built-in `base` and override individual fields. Renderers consume a
//! [`ThemeSpec`] and never branch on the theme name.

use std::path::Path;

use serde::Deserialize;

/// A visual theme: one of the built-ins, or a custom spec loaded from TOML.
#[derive(Debug, Clone, PartialEq)]
pub enum Theme {
    Default,
    Book,
    Custom(Box<ThemeSpec>),
}

impl Theme {
    /// The built-in themes.
    pub const BUILTIN: [Theme; 2] = [Theme::Default, Theme::Book];

    pub fn name(&self) -> &str {
        match self {
            Theme::Default => "default",
            Theme::Book => "book",
            Theme::Custom(spec) => &spec.name,
        }
    }

    pub fn description(&self) -> &str {
        match self {
            Theme::Default => "Clean serif body with blue heading accents",
            Theme::Book => "Classic book look set in New Computer Modern",
            Theme::Custom(_) => "Custom theme loaded from TOML",
        }
    }

    /// Look up a built-in theme by name.
    pub fn from_name(name: &str) -> Option<Theme> {
        Theme::BUILTIN.into_iter().find(|t| t.name() == name)
    }

    /// Load a custom theme from a TOML file.
    pub fn load(path: &Path) -> anyhow::Result<Theme> {
        let contents = std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("could not read theme {}: {e}", path.display()))?;
        let name = path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "custom".to_string());
        let spec = ThemeSpec::from_toml(&contents, &name)
            .map_err(|e| anyhow::anyhow!("invalid theme {}: {e}", path.display()))?;
        Ok(Theme::Custom(Box::new(spec)))
    }

    /// Resolve the concrete typographic spec for this theme.
    pub fn spec(&self) -> ThemeSpec {
        match self {
            Theme::Default => ThemeSpec {
                name: "default".into(),
                body_font: "Libertinus Serif".into(),
                mono_font: "DejaVu Sans Mono".into(),
                body_size_pt: 11.0,
                heading_color: "#13315c".into(),
                link_color: "#1565c0".into(),
                accent_color: "#13315c".into(),
                code_fill: "#f4f6f8".into(),
                code_stroke: Some("#e1e4e8".into()),
                margin_x_cm: 2.2,
                margin_y_cm: 2.4,
                justify: true,
                first_line_indent_em: 0.0,
            },
            Theme::Book => ThemeSpec {
                name: "book".into(),
                body_font: "New Computer Modern".into(),
                mono_font: "DejaVu Sans Mono".into(),
                body_size_pt: 11.0,
                heading_color: "#5a1e1e".into(),
                link_color: "#8a2b2b".into(),
                accent_color: "#5a1e1e".into(),
                code_fill: "#f5f5f5".into(),
                code_stroke: None,
                margin_x_cm: 2.6,
                margin_y_cm: 2.8,
                justify: true,
                first_line_indent_em: 1.2,
            },
            Theme::Custom(spec) => (**spec).clone(),
        }
    }
}

/// Concrete, renderer-facing description of a theme's typography.
#[derive(Debug, Clone, PartialEq)]
pub struct ThemeSpec {
    pub name: String,
    pub body_font: String,
    pub mono_font: String,
    pub body_size_pt: f32,
    pub heading_color: String,
    pub link_color: String,
    pub accent_color: String,
    pub code_fill: String,
    pub code_stroke: Option<String>,
    pub margin_x_cm: f32,
    pub margin_y_cm: f32,
    pub justify: bool,
    pub first_line_indent_em: f32,
}

/// The on-disk TOML form of a theme: a built-in `base` plus field overrides.
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct ThemeOverrides {
    base: Option<String>,
    body_font: Option<String>,
    mono_font: Option<String>,
    body_size_pt: Option<f32>,
    heading_color: Option<String>,
    link_color: Option<String>,
    accent_color: Option<String>,
    code_fill: Option<String>,
    code_stroke: Option<String>,
    margin_x_cm: Option<f32>,
    margin_y_cm: Option<f32>,
    justify: Option<bool>,
    first_line_indent_em: Option<f32>,
}

impl ThemeSpec {
    /// Parse a theme from TOML, inheriting from its `base` (default: "default").
    pub fn from_toml(contents: &str, name: &str) -> anyhow::Result<ThemeSpec> {
        let o: ThemeOverrides = toml::from_str(contents)?;
        let base_name = o.base.as_deref().unwrap_or("default");
        let base = Theme::from_name(base_name)
            .ok_or_else(|| anyhow::anyhow!("unknown base theme: {base_name}"))?;
        let mut spec = base.spec();
        spec.name = name.to_string();

        if let Some(v) = o.body_font {
            spec.body_font = v;
        }
        if let Some(v) = o.mono_font {
            spec.mono_font = v;
        }
        if let Some(v) = o.body_size_pt {
            spec.body_size_pt = v;
        }
        if let Some(v) = o.heading_color {
            spec.heading_color = validated_color(v)?;
        }
        if let Some(v) = o.link_color {
            spec.link_color = validated_color(v)?;
        }
        if let Some(v) = o.accent_color {
            spec.accent_color = validated_color(v)?;
        }
        if let Some(v) = o.code_fill {
            spec.code_fill = validated_color(v)?;
        }
        if let Some(v) = o.code_stroke {
            spec.code_stroke = Some(validated_color(v)?);
        }
        if let Some(v) = o.margin_x_cm {
            spec.margin_x_cm = v;
        }
        if let Some(v) = o.margin_y_cm {
            spec.margin_y_cm = v;
        }
        if let Some(v) = o.justify {
            spec.justify = v;
        }
        if let Some(v) = o.first_line_indent_em {
            spec.first_line_indent_em = v;
        }
        Ok(spec)
    }

    /// A starter theme file for `md2pdf theme create`.
    pub fn sample() -> &'static str {
        r##"# md2pdf custom theme. All fields are optional and override the base.
base = "default"   # or "book"

# body_font = "Libertinus Serif"
# mono_font = "DejaVu Sans Mono"
# body_size_pt = 11.0
# heading_color = "#13315c"
# link_color = "#1565c0"
# accent_color = "#13315c"
# code_fill = "#f4f6f8"
# code_stroke = "#e1e4e8"
# margin_x_cm = 2.2
# margin_y_cm = 2.4
# justify = true
# first_line_indent_em = 0.0
"##
    }
}

/// Colors are interpolated into Typst `rgb("…")` calls; accept only hex forms.
fn validated_color(v: String) -> anyhow::Result<String> {
    let hex = v.strip_prefix('#').unwrap_or(&v);
    let ok = matches!(hex.len(), 3 | 4 | 6 | 8) && hex.chars().all(|c| c.is_ascii_hexdigit());
    if !ok {
        anyhow::bail!("invalid color {v:?} (expected hex like \"#1a2b3c\")");
    }
    Ok(if v.starts_with('#') {
        v
    } else {
        format!("#{v}")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn custom_theme_inherits_from_base() {
        let spec = ThemeSpec::from_toml(
            "base = \"book\"\nheading_color = \"#222222\"\nbody_size_pt = 12.5\n",
            "mytheme",
        )
        .unwrap();
        assert_eq!(spec.name, "mytheme");
        assert_eq!(spec.heading_color, "#222222");
        assert_eq!(spec.body_size_pt, 12.5);
        // Untouched fields come from the book base.
        assert_eq!(spec.body_font, "New Computer Modern");
        assert_eq!(spec.first_line_indent_em, 1.2);
    }

    #[test]
    fn theme_defaults_to_default_base() {
        let spec = ThemeSpec::from_toml("", "empty").unwrap();
        assert_eq!(spec.body_font, Theme::Default.spec().body_font);
    }

    #[test]
    fn theme_rejects_unknown_base_and_fields() {
        assert!(ThemeSpec::from_toml("base = \"nope\"", "t").is_err());
        assert!(ThemeSpec::from_toml("not_a_field = 1", "t").is_err());
    }

    #[test]
    fn theme_rejects_color_injection() {
        let res = ThemeSpec::from_toml("heading_color = '\") #eval(...) //'", "t");
        assert!(res.is_err(), "non-hex colors must be rejected");
    }

    #[test]
    fn sample_theme_parses() {
        let spec = ThemeSpec::from_toml(ThemeSpec::sample(), "sample").unwrap();
        assert_eq!(spec.body_font, "Libertinus Serif");
    }
}
