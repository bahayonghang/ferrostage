//! The single presenter theme foundation from root DESIGN.md. Never used by AudienceView.
use ferrostage_settings::Theme;

pub const CONTROL: f32 = 44.0;
pub const FOCUS_WIDTH: f32 = 2.0;
pub const FOCUS_GAP: f32 = 2.0;
pub const RADIUS: f32 = 8.0;
pub const PANEL_RADIUS: f32 = 12.0;
pub const SPACING: f32 = 16.0;
pub const HOME_WIDTH: f32 = 760.0;
pub const SESSION_CONTROL: f32 = 56.;
pub const SESSION_ICON: f32 = 26.;
pub const SESSION_WIDTH: f32 = 208.;
pub const SANS: &str = "Segoe UI";
pub const DISPLAY: &str = "Georgia";
pub const MONO: &str = "Consolas";
pub const SETTINGS_WIDTH: f32 = 960.;
pub const SETTINGS_DISPLAY_WIDTH: f32 = 1120.;
pub const SETTINGS_ROW: f32 = 56.;
pub const DISPLAY_PREVIEW_WIDTH: f32 = 420.;
pub const DISPLAY_PREVIEW_HEIGHT: f32 = 240.;

/// Resolved presenter typography. Requested preferences remain in Settings.
#[derive(Clone, Debug)]
pub struct Typography {
    pub family: String,
    pub missing: bool,
    scale: f32,
}

impl Typography {
    pub fn resolve(settings: &ferrostage_settings::GeneralSettings, available: &[String]) -> Self {
        let requested = settings.ui_font_family.trim();
        let installed = available
            .iter()
            .find(|name| name.eq_ignore_ascii_case(requested));
        Self {
            family: installed.cloned().unwrap_or_else(|| SANS.into()),
            missing: !requested.is_empty() && installed.is_none(),
            scale: settings.ui_font_size as f32 / 16.,
        }
    }

    pub fn size(&self, role: f32) -> f32 {
        role * self.scale
    }

    #[cfg(feature = "gpui")]
    pub fn font(&self) -> gpui::Font {
        gpui::Font {
            fallbacks: Some(gpui::FontFallbacks::from_fonts(vec![
                SANS.into(),
                "Microsoft YaHei UI".into(),
            ])),
            ..gpui::font(self.family.clone())
        }
    }
}

#[cfg(feature = "gpui")]
pub fn sans_font() -> gpui::Font {
    gpui::Font {
        fallbacks: Some(gpui::FontFallbacks::from_fonts(vec![
            "Microsoft YaHei UI".into(),
        ])),
        ..gpui::font(SANS)
    }
}

#[cfg(feature = "gpui")]
pub fn home_brand_font() -> gpui::Font {
    gpui::Font {
        weight: gpui::FontWeight::SEMIBOLD,
        ..gpui::font(DISPLAY)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Palette {
    pub canvas: u32,
    pub soft: u32,
    pub card: u32,
    pub ink: u32,
    pub body: u32,
    pub muted: u32,
    pub border: u32,
    pub focus: u32,
    pub success: u32,
    pub warning: u32,
    pub error: u32,
}

pub const PRIMARY: u32 = 0xA34F35;
pub const PRIMARY_HOVER: u32 = 0x93452F;
pub const PRIMARY_ACTIVE: u32 = 0x823C29;
pub const ON_PRIMARY: u32 = 0xFFFFFF;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonKind {
    Primary,
    Navigation,
    Black,
    White,
    Tool,
    Destructive,
    Utility,
}

#[derive(Clone, Copy)]
pub struct ButtonStyle {
    pub fill: u32,
    pub ink: u32,
    pub border: u32,
    pub hover: u32,
    pub pressed: u32,
    pub radius: f32,
}

impl Palette {
    pub fn button(self, kind: ButtonKind, enabled: bool, selected: bool) -> ButtonStyle {
        let mut style = ButtonStyle {
            fill: self.soft,
            ink: self.body,
            border: self.border,
            hover: self.card,
            pressed: self.card,
            radius: RADIUS,
        };
        match kind {
            ButtonKind::Primary => {
                style.fill = PRIMARY;
                style.ink = ON_PRIMARY;
                style.hover = PRIMARY_HOVER;
                style.pressed = PRIMARY_ACTIVE;
            }
            ButtonKind::Navigation => {
                style.fill = self.card;
                style.border = self.card;
                style.ink = self.ink;
                style.hover = self.soft;
                style.pressed = self.soft;
            }
            ButtonKind::Black | ButtonKind::White | ButtonKind::Tool => {
                style.border = self.soft;
                if selected {
                    style.fill = self.card;
                    style.ink = self.ink;
                    style.border = self.card;
                }
            }
            ButtonKind::Destructive => {
                style.ink = self.error;
                style.border = self.error;
                style.radius = 4.;
            }
            ButtonKind::Utility => {}
        }
        if !enabled {
            style.fill = self.card;
            style.ink = self.muted;
            style.border = self.border;
        }
        style
    }
}

pub const LIGHT: Palette = Palette {
    canvas: 0xFAF9F5,
    soft: 0xF5F0E8,
    card: 0xEFE9DE,
    ink: 0x141413,
    body: 0x3D3D3A,
    muted: 0x69665F,
    border: 0x858078,
    focus: 0xA34F35,
    success: 0x28613D,
    warning: 0x825315,
    error: 0xA63232,
};
pub const DARK: Palette = Palette {
    canvas: 0x181715,
    soft: 0x1F1E1B,
    card: 0x252320,
    ink: 0xFAF9F5,
    body: 0xE6DFD8,
    muted: 0xB9B3A8,
    border: 0x91897C,
    focus: 0xE5A38B,
    success: 0x8DCEA0,
    warning: 0xE8B774,
    error: 0xFFA5A0,
};

pub fn palette(theme: Theme, system_dark: bool) -> Palette {
    match theme {
        Theme::Dark => DARK,
        Theme::Light => LIGHT,
        Theme::System if system_dark => DARK,
        Theme::System => LIGHT,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn typography_retains_request_and_scales_roles_with_missing_fallback() {
        let mut general = ferrostage_settings::GeneralSettings {
            ui_font_family: "missing family".into(),
            ui_font_size: 24,
            ..Default::default()
        };
        let available = vec!["Installed Family".into()];
        let fallback = Typography::resolve(&general, &available);
        assert!(fallback.missing);
        assert_eq!(fallback.family, SANS);
        assert_eq!(fallback.size(20.), 30.);
        assert_eq!(general.ui_font_family, "missing family");
        general.ui_font_family = "installed family".into();
        let installed = Typography::resolve(&general, &available);
        assert!(!installed.missing);
        assert_eq!(installed.family, "Installed Family");
        general.ui_font_family.clear();
        assert!(!Typography::resolve(&general, &available).missing);
    }
    fn luminance(color: u32) -> f64 {
        let channels = [color >> 16, color >> 8, color].map(|v| {
            let c = f64::from(v & 255) / 255.;
            if c <= 0.04045 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        });
        channels[0] * 0.2126 + channels[1] * 0.7152 + channels[2] * 0.0722
    }
    fn contrast(a: u32, b: u32) -> f64 {
        let a = luminance(a);
        let b = luminance(b);
        (a.max(b) + 0.05) / (a.min(b) + 0.05)
    }
    #[test]
    fn semantic_button_states_keep_text_and_boundaries_readable() {
        for palette in [LIGHT, DARK] {
            for kind in [
                ButtonKind::Primary,
                ButtonKind::Navigation,
                ButtonKind::Black,
                ButtonKind::White,
                ButtonKind::Tool,
                ButtonKind::Destructive,
                ButtonKind::Utility,
            ] {
                for enabled in [false, true] {
                    for selected in [false, true] {
                        let style = palette.button(kind, enabled, selected);
                        assert!(contrast(style.ink, style.fill) >= 4.5, "{kind:?}");
                        if matches!(
                            kind,
                            ButtonKind::Primary | ButtonKind::Destructive | ButtonKind::Utility
                        ) || !enabled
                        {
                            assert!(contrast(style.border, palette.canvas) >= 3.);
                        }
                        assert!(
                            contrast(palette.focus, style.fill) >= 3.
                                || kind == ButtonKind::Primary
                        );
                        if enabled {
                            assert!(contrast(style.ink, style.hover) >= 4.5, "hover {kind:?}");
                            assert!(contrast(style.ink, style.pressed) >= 4.5, "press {kind:?}");
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn used_text_and_focus_pairs_meet_design_contrast() {
        for p in [LIGHT, DARK] {
            for surface in [p.canvas, p.soft, p.card] {
                for text in [p.ink, p.body, p.muted, p.success, p.warning, p.error] {
                    assert!(contrast(text, surface) >= 4.5, "{text:x}/{surface:x}");
                }
                for boundary in [p.border, p.focus] {
                    assert!(contrast(boundary, surface) >= 3.0);
                }
            }
        }
        for fill in [PRIMARY, PRIMARY_HOVER, PRIMARY_ACTIVE] {
            assert!(contrast(ON_PRIMARY, fill) >= 4.5);
        }
        assert_eq!(palette(Theme::System, true).canvas, DARK.canvas);
        assert_eq!(palette(Theme::System, false).canvas, LIGHT.canvas);
    }
}
