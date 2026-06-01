//! Themes as data.
//!
//! A [`ThemeSpec`] is a flat bag of typographic knobs. Built-in themes are
//! constructed in code today; because a theme is plain data with an optional
//! `base`, user-defined themes (loaded from TOML) can later inherit from a
//! built-in by starting from its [`ThemeSpec`] and overriding fields. Renderers
//! consume a [`ThemeSpec`] and never branch on the theme name.

/// Built-in theme identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    Default,
    Book,
}

impl Theme {
    pub const ALL: [Theme; 2] = [Theme::Default, Theme::Book];

    pub fn name(self) -> &'static str {
        match self {
            Theme::Default => "default",
            Theme::Book => "book",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Theme::Default => "Clean serif body with blue heading accents",
            Theme::Book => "Classic book look set in New Computer Modern",
        }
    }

    pub fn from_name(name: &str) -> Option<Theme> {
        Theme::ALL.into_iter().find(|t| t.name() == name)
    }

    /// Resolve the concrete typographic spec for this theme.
    pub fn spec(self) -> ThemeSpec {
        match self {
            Theme::Default => ThemeSpec {
                name: "default",
                body_font: "Libertinus Serif",
                mono_font: "DejaVu Sans Mono",
                body_size_pt: 11.0,
                heading_color: "#13315c",
                link_color: "#1565c0",
                accent_color: "#13315c",
                code_fill: "#f4f6f8",
                code_stroke: Some("#e1e4e8"),
                margin_x_cm: 2.2,
                margin_y_cm: 2.4,
                justify: true,
                first_line_indent_em: 0.0,
            },
            Theme::Book => ThemeSpec {
                name: "book",
                body_font: "New Computer Modern",
                mono_font: "DejaVu Sans Mono",
                body_size_pt: 11.0,
                heading_color: "#5a1e1e",
                link_color: "#8a2b2b",
                accent_color: "#5a1e1e",
                code_fill: "#f5f5f5",
                code_stroke: None,
                margin_x_cm: 2.6,
                margin_y_cm: 2.8,
                justify: true,
                first_line_indent_em: 1.2,
            },
        }
    }
}

/// Concrete, renderer-facing description of a theme's typography.
#[derive(Debug, Clone)]
pub struct ThemeSpec {
    pub name: &'static str,
    pub body_font: &'static str,
    pub mono_font: &'static str,
    pub body_size_pt: f32,
    pub heading_color: &'static str,
    pub link_color: &'static str,
    pub accent_color: &'static str,
    pub code_fill: &'static str,
    pub code_stroke: Option<&'static str>,
    pub margin_x_cm: f32,
    pub margin_y_cm: f32,
    pub justify: bool,
    pub first_line_indent_em: f32,
}
