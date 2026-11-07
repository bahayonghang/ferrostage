//! The single presenter theme foundation from root DESIGN.md. Never used by AudienceView.
use ferrostage_settings::Theme;

pub const CONTROL: f32 = 44.0;
pub const FOCUS_WIDTH: f32 = 2.0;
pub const FOCUS_GAP: f32 = 2.0;
pub const RADIUS: f32 = 16.0;
pub const PANEL_RADIUS: f32 = 20.0;
pub const PILL_RADIUS: f32 = 999.0;
pub const FIELD_ALPHA: f32 = 0.75;
pub const SHEET_ALPHA: f32 = 0.78;
pub const SOLID_SHEET_ALPHA: f32 = 0.96;
pub const SHEET_SHADOW: u32 = 0x465A7814;
pub const SPACING: f32 = 16.0;
pub const HOME_WIDTH: f32 = 760.0;
pub const SESSION_CONTROL: f32 = 56.;
pub const SESSION_ICON: f32 = 26.;
pub const SESSION_WIDTH: f32 = 208.;
/// Minimum Restart segment width beside Resume at default text size.
pub const RESTART_WIDTH: f32 = 144.;
const PREFERRED_SANS: &str = "Figtree";
const SYSTEM_SANS: &str = ".SystemUIFont";
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
        let installed = if requested.is_empty() {
            None
        } else {
            available
                .iter()
                .find(|name| name.eq_ignore_ascii_case(requested))
        };
        // GPUI supplies sorted family names. Asking DirectWrite to resolve a
        // missing primary face refreshes the system collection for each new weight.
        let family = installed.map(String::as_str).unwrap_or_else(|| {
            if available
                .binary_search_by(|name| name.as_str().cmp(PREFERRED_SANS))
                .is_ok()
            {
                PREFERRED_SANS
            } else {
                SYSTEM_SANS
            }
        });
        Self {
            family: family.into(),
            missing: !requested.is_empty() && installed.is_none(),
            scale: settings.ui_font_size as f32 / 16.,
        }
    }

    pub fn size(&self, role: f32) -> f32 {
        role * self.scale
    }

    #[cfg(feature = "gpui")]
    pub fn font(&self) -> gpui::Font {
        static FALLBACKS: std::sync::LazyLock<gpui::FontFallbacks> =
            std::sync::LazyLock::new(|| {
                gpui::FontFallbacks::from_fonts(vec![
                    "Segoe UI".into(),
                    "PingFang SC".into(),
                    "Noto Sans SC".into(),
                    "Microsoft YaHei UI".into(),
                ])
            });
        gpui::Font {
            fallbacks: Some(FALLBACKS.clone()),
            ..gpui::font(self.family.clone())
        }
    }

    #[cfg(feature = "gpui")]
    pub fn numeric_font(&self) -> gpui::Font {
        static FEATURES: std::sync::LazyLock<gpui::FontFeatures> = std::sync::LazyLock::new(|| {
            gpui::FontFeatures(std::sync::Arc::new(vec![("tnum".into(), 1)]))
        });
        gpui::Font {
            features: FEATURES.clone(),
            ..self.font()
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Palette {
    pub field_mint: u32,
    pub field_lavender: u32,
    pub sheet: u32,
    pub sheet_edge: u32,
    pub canvas: u32,
    pub soft: u32,
    pub card: u32,
    pub ink: u32,
    pub body: u32,
    pub muted: u32,
    pub border: u32,
    pub focus: u32,
    pub output: u32,
    pub on_action: u32,
    action_hover: u32,
    action_pressed: u32,
    control_hover: u32,
    control_pressed: u32,
    pub success: u32,
    pub warning: u32,
    pub error: u32,
}

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
            fill: self.canvas,
            ink: self.body,
            border: self.border,
            hover: self.control_hover,
            pressed: self.control_pressed,
            radius: PILL_RADIUS,
        };
        match kind {
            ButtonKind::Primary => {
                style.fill = self.focus;
                style.ink = self.on_action;
                style.border = self.focus;
                style.hover = self.action_hover;
                style.pressed = self.action_pressed;
            }
            ButtonKind::Destructive => {
                style.ink = self.error;
                style.border = self.error;
            }
            _ if selected => {
                style.fill = self.soft;
                style.ink = self.focus;
                style.border = self.focus;
            }
            ButtonKind::Navigation => {
                style.fill = self.soft;
                style.ink = self.focus;
            }
            _ => {}
        }
        if !enabled {
            style.fill = self.canvas;
            style.ink = self.muted;
            style.border = self.border;
        }
        style
    }
}

pub const LIGHT: Palette = Palette {
    field_mint: 0xE7F6F1,
    field_lavender: 0xEDE7F8,
    sheet: 0xFFFFFF,
    sheet_edge: 0xFFFFFF,
    canvas: 0xFEFEFF,
    soft: 0xE7F7F0,
    card: 0xF3EEFC,
    ink: 0x24303A,
    body: 0x354650,
    muted: 0x5C6B76,
    border: 0x77878B,
    focus: 0x187A56,
    output: 0x6D4BD6,
    on_action: 0xFFFFFF,
    action_hover: 0x126747,
    action_pressed: 0x0E583D,
    control_hover: 0xEFF4F2,
    control_pressed: 0xE2F2EC,
    success: 0x187A56,
    warning: 0x875411,
    error: 0xA03242,
};

pub const DARK: Palette = Palette {
    field_mint: 0x142723,
    field_lavender: 0x242037,
    sheet: 0x172321,
    sheet_edge: 0xA3B9B1,
    canvas: 0x172321,
    soft: 0x213A32,
    card: 0x312C44,
    ink: 0xF3FAF7,
    body: 0xD9E6E1,
    muted: 0xB2C5BF,
    border: 0x859D94,
    focus: 0x83DFB7,
    output: 0xC6B4FF,
    on_action: 0x10271E,
    action_hover: 0xA4EACE,
    action_pressed: 0x70CEAA,
    control_hover: 0x26342F,
    control_pressed: 0x324037,
    success: 0x83DFB7,
    warning: 0xF1C27C,
    error: 0xFFABB6,
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
        assert_eq!(fallback.family, SYSTEM_SANS);
        assert_eq!(fallback.size(20.), 30.);
        assert_eq!(general.ui_font_family, "missing family");
        general.ui_font_family = "installed family".into();
        let installed = Typography::resolve(&general, &available);
        assert!(!installed.missing);
        assert_eq!(installed.family, "Installed Family");
        let with_figtree = vec![PREFERRED_SANS.into(), "Installed Family".into()];
        assert_eq!(
            Typography::resolve(&general, &with_figtree).family,
            "Installed Family"
        );
        general.ui_font_family = "missing family".into();
        let fallback = Typography::resolve(&general, &with_figtree);
        assert!(fallback.missing);
        assert_eq!(fallback.family, PREFERRED_SANS);
        general.ui_font_family.clear();
        let default = Typography::resolve(&general, &available);
        assert!(!default.missing);
        assert_eq!(default.family, SYSTEM_SANS);
        assert_eq!(
            Typography::resolve(&general, &with_figtree).family,
            PREFERRED_SANS
        );
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
                for text in [
                    p.ink, p.body, p.muted, p.success, p.output, p.warning, p.error,
                ] {
                    assert!(contrast(text, surface) >= 4.5, "{text:x}/{surface:x}");
                }
                for boundary in [p.border, p.focus] {
                    assert!(contrast(boundary, surface) >= 3.0);
                }
            }
        }

        assert_eq!(palette(Theme::System, true).canvas, DARK.canvas);
        assert_eq!(palette(Theme::System, false).canvas, LIGHT.canvas);
    }

    #[test]
    fn paper_over_each_field_edge_and_desktop_extreme_keeps_aa_contrast() {
        fn over(front: u32, back: u32, alpha: f32) -> u32 {
            [16, 8, 0].into_iter().fold(0, |rgb, shift| {
                let a = ((front >> shift) & 255) as f32;
                let b = ((back >> shift) & 255) as f32;
                rgb | (((a * alpha + b * (1. - alpha)).round() as u32) << shift)
            })
        }
        for p in [LIGHT, DARK] {
            for edge in [p.field_mint, p.field_lavender] {
                for desktop in [0, 0xffffff] {
                    let field = over(edge, desktop, FIELD_ALPHA);
                    for paper in [
                        over(p.sheet, field, SHEET_ALPHA),
                        over(p.sheet, edge, SOLID_SHEET_ALPHA),
                    ] {
                        for ink in [
                            p.ink, p.body, p.muted, p.output, p.success, p.warning, p.error,
                        ] {
                            assert!(contrast(ink, paper) >= 4.5, "{ink:06x} on {paper:06x}");
                        }
                        for boundary in [p.focus, p.border] {
                            assert!(
                                contrast(boundary, paper) >= 3.,
                                "{boundary:06x} on {paper:06x}"
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn timer_capsule_and_split_session_pairs_meet_design_contrast() {
        for p in [LIGHT, DARK] {
            // Capsule label, live/quiet digits and warnings on the output well.
            for text in [p.muted, p.body, p.output, p.warning, p.error] {
                assert!(contrast(text, p.card) >= 4.5, "{text:x}/{:x}", p.card);
            }
            // State marks are non-text UI: focus dot, muted ring/square, warnings.
            for mark in [p.focus, p.muted, p.warning, p.error] {
                assert!(contrast(mark, p.card) >= 3.0, "{mark:x}");
            }
            // Restart segment: navigation face with a muted sublabel.
            let restart = p.button(ButtonKind::Navigation, true, false);
            for fill in [restart.fill, restart.hover, restart.pressed] {
                assert!(contrast(p.muted, fill) >= 4.5);
            }
        }
    }
}
