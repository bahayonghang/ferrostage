//! The single presenter theme foundation from root DESIGN.md (warm-orange glass).
//! Never used by AudienceView.
use ferrostage_settings::Theme;

pub const CONTROL: f32 = 44.0;
pub const FOCUS_WIDTH: f32 = 2.0;
pub const FOCUS_GAP: f32 = 2.0;
pub const RADIUS: f32 = 16.0;
/// Panels and floating surfaces.
pub const PANEL_RADIUS: f32 = 16.0;
/// Inset wells and images inside a 4px-padded panel.
pub const WELL_RADIUS: f32 = 12.0;
pub const PILL_RADIUS: f32 = 999.0;
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

/// Presenter colors. Every value is opaque `0xRRGGBB` except the fields documented
/// as RGBA (`hairline`, `shadow`); `layer` is drawn at `layer_alpha` only when the
/// native window blur is active, otherwise panels use `layer_solid`.
#[derive(Clone, Copy, Debug)]
pub struct Palette {
    /// Window field gradient ends (160deg). Text never sits directly on the field.
    pub field_top: u32,
    pub field_bottom: u32,
    /// Field opacity over the native blurred window backdrop. High enough that an
    /// arbitrary desktop tints the field without replacing its warm hue.
    pub field_alpha: f32,
    /// Panel layer over the native blurred field.
    pub layer: u32,
    pub layer_alpha: f32,
    /// Panel color for every solid fallback surface.
    pub layer_solid: u32,
    /// Floating menus and dialogs.
    pub raised: u32,
    /// The selected option on a `control` track; always lighter than the track.
    pub segment: u32,
    /// Inset area inside a panel: note text, inputs and grouped settings.
    pub well: u32,
    /// Neutral control fill and its hover/pressed states, precomposited over `layer_solid`.
    pub control: u32,
    pub control_hover: u32,
    pub control_pressed: u32,
    /// Decorative separator, RGBA. Never a control boundary.
    pub hairline: u32,
    /// Boundary for inputs that need one; >=3:1 on `well` and `layer_solid`.
    pub border: u32,
    pub ink: u32,
    pub body: u32,
    pub muted: u32,
    /// Primary action fill and its text.
    pub accent: u32,
    pub on_accent: u32,
    pub accent_hover: u32,
    pub accent_pressed: u32,
    /// Orange text and links.
    pub accent_text: u32,
    /// Focus ring and selection indicator.
    pub focus: u32,
    /// Selected row or menu entry fill.
    pub accent_wash: u32,
    /// Status colors, for status text and marks only.
    pub success: u32,
    pub warning: u32,
    pub error: u32,
    pub error_wash: u32,
    /// The single floating-surface shadow, RGBA.
    pub shadow: u32,
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
    /// One option of a segmented control; the track is `control`.
    Segment,
    /// The session Stop action: error text on its own wash.
    Stop,
}

#[derive(Clone, Copy)]
pub struct ButtonStyle {
    pub fill: u32,
    pub ink: u32,
    pub hover: u32,
    pub pressed: u32,
    pub radius: f32,
    /// Non-color selected-state mark, drawn in a fixed slot.
    pub indicator: Option<u32>,
}

impl Palette {
    pub fn button(self, kind: ButtonKind, enabled: bool, selected: bool) -> ButtonStyle {
        let mut style = ButtonStyle {
            fill: self.control,
            ink: self.body,
            hover: self.control_hover,
            pressed: self.control_pressed,
            radius: PILL_RADIUS,
            indicator: None,
        };
        match kind {
            ButtonKind::Primary => {
                style.fill = self.accent;
                style.ink = self.on_accent;
                style.hover = self.accent_hover;
                style.pressed = self.accent_pressed;
                style.indicator = selected.then_some(self.on_accent);
            }
            ButtonKind::Stop => {
                style.fill = self.error_wash;
                style.ink = self.error;
                style.hover = over(self.ink, self.error_wash, 0.03);
                style.pressed = over(self.ink, self.error_wash, 0.06);
                style.indicator = selected.then_some(self.focus);
            }
            ButtonKind::Destructive => {
                style.ink = self.error;
                style.indicator = selected.then_some(self.focus);
            }
            // Black/white cover and drawing tools are options of segmented groups.
            ButtonKind::Segment | ButtonKind::Black | ButtonKind::White | ButtonKind::Tool
                if selected =>
            {
                style.fill = self.segment;
                style.ink = self.ink;
                style.hover = over(self.ink, self.segment, 0.03);
                style.pressed = over(self.ink, self.segment, 0.06);
                style.indicator = Some(self.focus);
            }
            _ if selected => {
                style.fill = self.accent_wash;
                style.ink = self.ink;
                style.indicator = Some(self.focus);
            }
            _ => {}
        }
        if !enabled {
            style.fill = self.control;
            style.ink = self.muted;
            style.indicator = selected.then_some(self.muted);
        }
        style
    }
}

/// Source-over composition of opaque `front` at `alpha` on opaque `back`.
pub fn over(front: u32, back: u32, alpha: f32) -> u32 {
    [16, 8, 0].into_iter().fold(0, |rgb, shift| {
        let a = ((front >> shift) & 255) as f32;
        let b = ((back >> shift) & 255) as f32;
        rgb | (((a * alpha + b * (1. - alpha)).round() as u32) << shift)
    })
}

pub const LIGHT: Palette = Palette {
    field_top: 0xF7F2EB,
    field_bottom: 0xEFE6DB,
    field_alpha: 0.88,
    layer: 0xFFFFFF,
    layer_alpha: 0.66,
    layer_solid: 0xFBF8F4,
    raised: 0xFFFFFF,
    segment: 0xFFFFFF,
    well: 0xF3EEE7,
    // Ink #1F1B17 at 0.06 / 0.10 / 0.14 over `layer_solid`.
    control: 0xEEEBE7,
    control_hover: 0xE5E2DE,
    control_pressed: 0xDCD9D5,
    hairline: 0x1F1B1714,
    border: 0x8C847A,
    ink: 0x1F1B17,
    body: 0x3D3833,
    muted: 0x5F5850,
    accent: 0xD97757,
    on_accent: 0x1F1712,
    accent_hover: 0xE0866A,
    accent_pressed: 0xC9694A,
    accent_text: 0xA3492A,
    focus: 0xA3492A,
    accent_wash: 0xF7E6DC,
    success: 0x2F6B45,
    warning: 0x8A5A12,
    error: 0xA63A32,
    error_wash: 0xF8E3DF,
    shadow: 0x1F1B171F,
};

pub const DARK: Palette = Palette {
    field_top: 0x1D1B19,
    field_bottom: 0x26221E,
    field_alpha: 0.82,
    layer: 0x2C2824,
    layer_alpha: 0.74,
    layer_solid: 0x2A2622,
    raised: 0x38332E,
    segment: 0x524D48,
    well: 0x221F1C,
    // White at 0.08 / 0.12 / 0.16 over `layer_solid`.
    control: 0x3B3734,
    control_hover: 0x44403D,
    control_pressed: 0x4C4945,
    hairline: 0xFFFFFF14,
    border: 0x8F877C,
    ink: 0xF5F1EB,
    body: 0xE2DBD2,
    muted: 0xC4BCB2,
    accent: 0xD97757,
    on_accent: 0x1F1712,
    accent_hover: 0xE0866A,
    accent_pressed: 0xC9694A,
    accent_text: 0xEBA98C,
    focus: 0xEBA98C,
    accent_wash: 0x3A2A22,
    success: 0x8DCEA0,
    warning: 0xE8B774,
    error: 0xFFA5A0,
    error_wash: 0x3B2523,
    shadow: 0x00000066,
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
    const KINDS: [ButtonKind; 9] = [
        ButtonKind::Primary,
        ButtonKind::Navigation,
        ButtonKind::Black,
        ButtonKind::White,
        ButtonKind::Tool,
        ButtonKind::Destructive,
        ButtonKind::Utility,
        ButtonKind::Segment,
        ButtonKind::Stop,
    ];
    /// Every opaque panel color text can land on, including the native composite of
    /// field and layer over the two desktop extremes behind the blurred window.
    fn panels(p: Palette) -> Vec<u32> {
        let mut panels = vec![p.layer_solid];
        for edge in [p.field_top, p.field_bottom] {
            for desktop in [0, 0xFFFFFF] {
                panels.push(over(
                    p.layer,
                    over(edge, desktop, p.field_alpha),
                    p.layer_alpha,
                ));
            }
        }
        panels
    }

    #[test]
    fn control_fills_are_the_documented_precomposites() {
        for (p, base, alphas) in [
            (LIGHT, LIGHT.ink, [0.06, 0.10, 0.14]),
            (DARK, 0xFFFFFF, [0.08, 0.12, 0.16]),
        ] {
            let fills = [p.control, p.control_hover, p.control_pressed];
            for (fill, alpha) in fills.into_iter().zip(alphas) {
                assert_eq!(fill, over(base, p.layer_solid, alpha), "{fill:06x}");
            }
        }
    }

    #[test]
    fn semantic_button_states_keep_text_focus_and_selection_readable() {
        for p in [LIGHT, DARK] {
            for kind in KINDS {
                for enabled in [false, true] {
                    for selected in [false, true] {
                        let style = p.button(kind, enabled, selected);
                        assert!(contrast(style.ink, style.fill) >= 4.5, "{kind:?}");
                        if enabled {
                            assert!(contrast(style.ink, style.hover) >= 4.5, "hover {kind:?}");
                            assert!(contrast(style.ink, style.pressed) >= 4.5, "press {kind:?}");
                        }
                        // The 2px ring sits outside a 2px gap on the panel; it must
                        // also read against every neutral fill it can touch.
                        if kind != ButtonKind::Primary {
                            assert!(contrast(p.focus, style.fill) >= 3., "focus {kind:?}");
                        }
                        // Selection always carries a non-color mark.
                        assert_eq!(style.indicator.is_some(), selected, "{kind:?}");
                        if let Some(mark) = style.indicator.filter(|_| enabled) {
                            for fill in [style.fill, style.hover, style.pressed] {
                                assert!(contrast(mark, fill) >= 3., "mark {kind:?} {fill:06x}");
                            }
                        }
                    }
                }
            }
            for panel in panels(p) {
                assert!(contrast(p.focus, panel) >= 3., "focus on {panel:06x}");
            }
            // The selected segment reads as lifted off its track in both variants.
            assert!(luminance(p.segment) > luminance(p.control));
            assert!(contrast(p.segment, p.control) >= 1.1, "segment lift");
        }
    }

    #[test]
    fn primary_action_is_warm_orange_with_dark_text() {
        for p in [LIGHT, DARK] {
            assert_eq!(p.accent, 0xD97757);
            for fill in [p.accent, p.accent_hover, p.accent_pressed] {
                assert!(contrast(p.on_accent, fill) >= 4.5, "{fill:06x}");
            }
            // White text on the warm orange is only about 3.1:1.
            assert!(contrast(0xFFFFFF, p.accent) < 4.5);
        }
    }

    #[test]
    fn used_text_and_boundary_pairs_meet_design_contrast() {
        for p in [LIGHT, DARK] {
            let mut surfaces = panels(p);
            surfaces.extend([p.raised, p.well, p.accent_wash, p.control]);
            for surface in surfaces {
                for text in [
                    p.ink,
                    p.body,
                    p.muted,
                    p.accent_text,
                    p.success,
                    p.warning,
                    p.error,
                ] {
                    assert!(contrast(text, surface) >= 4.5, "{text:06x}/{surface:06x}");
                }
            }
            for surface in [p.well, p.layer_solid, p.raised] {
                assert!(contrast(p.border, surface) >= 3., "border/{surface:06x}");
            }
            assert!(contrast(p.error, p.error_wash) >= 4.5);
            // The alert badge glyph, and the slider's value fill on its well.
            assert!(contrast(p.layer_solid, p.error) >= 4.5);
            assert!(contrast(p.focus, p.well) >= 3.);
        }

        assert_eq!(palette(Theme::System, true).layer_solid, DARK.layer_solid);
        assert_eq!(palette(Theme::System, false).layer_solid, LIGHT.layer_solid);
    }

    #[test]
    fn chrome_uses_one_warm_hue_without_green_or_purple() {
        fn hue_saturation(color: u32) -> (f64, f64) {
            let [r, g, b] = [color >> 16, color >> 8, color].map(|v| f64::from(v & 255) / 255.);
            let (max, min) = (r.max(g).max(b), r.min(g).min(b));
            let delta = max - min;
            if delta == 0. {
                return (0., 0.);
            }
            let lightness = (max + min) / 2.;
            let saturation = delta / (1. - (2. * lightness - 1.).abs());
            let hue = if max == r {
                60. * ((g - b) / delta).rem_euclid(6.)
            } else if max == g {
                60. * ((b - r) / delta + 2.)
            } else {
                60. * ((r - g) / delta + 4.)
            };
            (hue, saturation)
        }
        for p in [LIGHT, DARK] {
            // Status colors are excluded: they mark state text only.
            for color in [
                p.field_top,
                p.field_bottom,
                p.layer,
                p.layer_solid,
                p.raised,
                p.segment,
                p.well,
                p.control,
                p.control_hover,
                p.control_pressed,
                p.hairline >> 8,
                p.border,
                p.ink,
                p.body,
                p.muted,
                p.accent,
                p.on_accent,
                p.accent_hover,
                p.accent_pressed,
                p.accent_text,
                p.focus,
                p.accent_wash,
                p.error_wash,
            ] {
                let (hue, saturation) = hue_saturation(color);
                assert!(
                    !(90. ..=300.).contains(&hue) || saturation <= 0.25,
                    "{color:06x}: hue {hue:.0} saturation {saturation:.2}"
                );
            }
        }
    }

    #[test]
    fn timer_capsule_and_split_session_pairs_meet_design_contrast() {
        for p in [LIGHT, DARK] {
            // Capsule label, digits and warnings on the neutral control pill.
            for text in [p.muted, p.body, p.ink, p.warning, p.error] {
                assert!(contrast(text, p.control) >= 4.5, "{text:x}/{:x}", p.control);
            }
            // State marks are non-text UI: running dot, muted ring/square, warnings.
            for mark in [p.focus, p.muted, p.warning, p.error] {
                assert!(contrast(mark, p.control) >= 3.0, "{mark:x}");
            }
            // Restart segment: navigation face with a muted sublabel.
            let restart = p.button(ButtonKind::Navigation, true, false);
            for fill in [restart.fill, restart.hover, restart.pressed] {
                assert!(contrast(p.muted, fill) >= 4.5, "{fill:06x}");
            }
        }
    }
}
