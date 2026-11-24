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
/// Non-interactive wall clock and presentation time surfaces.
pub const READOUT_RADIUS: f32 = 6.;
pub const READOUT_TOP_ALPHA: f32 = 0.32;
pub const READOUT_BASE_ALPHA: f32 = 0.04;
pub const PILL_RADIUS: f32 = 999.0;
pub const SPACING: f32 = 16.0;
pub const SESSION_CONTROL: f32 = 56.;
pub const SESSION_ICON: f32 = 26.;
pub const SESSION_WIDTH: f32 = 208.;
/// Minimum Restart segment width beside Resume at default text size.
pub const RESTART_WIDTH: f32 = 144.;
const PREFERRED_SANS: &str = "Figtree";
pub(crate) const SYSTEM_SANS: &str = ".SystemUIFont";
pub const SETTINGS_WIDTH: f32 = 960.;
pub const SETTINGS_DISPLAY_WIDTH: f32 = 1120.;
pub const SETTINGS_ROW: f32 = 56.;
pub const DISPLAY_PREVIEW_WIDTH: f32 = 420.;
pub const DISPLAY_PREVIEW_HEIGHT: f32 = 240.;
/// Integrated window titlebar height and caption control button width.
pub const TITLEBAR_HEIGHT: f32 = 32.0;
pub const TITLEBAR_BUTTON_WIDTH: f32 = 46.0;
/// Leading clearance and coordinates for macOS native traffic lights.
pub const MACOS_TRAFFIC_LIGHT_INSET: f32 = 78.0;
pub const MACOS_TRAFFIC_LIGHT_X: f32 = 12.0;
pub const MACOS_TRAFFIC_LIGHT_Y: f32 = 10.0;

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

/// Translucent color token (`0xRRGGBB` + `alpha`). Keeps fill and alpha paired so
/// compositing and GPUI color conversion cannot drift apart.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tint {
    pub rgb: u32,
    pub alpha: f32,
}

impl Tint {
    pub const TRANSPARENT: Self = Self {
        rgb: 0x000000,
        alpha: 0.0,
    };

    pub const fn new(rgb: u32, alpha: f32) -> Self {
        Self { rgb, alpha }
    }

    pub const fn opaque(rgb: u32) -> Self {
        Self { rgb, alpha: 1.0 }
    }

    pub fn composite_over(self, back: u32) -> u32 {
        if self.alpha <= 0.0 {
            back
        } else if self.alpha >= 1.0 {
            self.rgb
        } else {
            over(self.rgb, back, self.alpha)
        }
    }

    pub fn resolve(self, sheet_solid: u32, glass: bool) -> Self {
        if glass {
            self
        } else {
            Self::opaque(self.composite_over(sheet_solid))
        }
    }

    #[cfg(feature = "gpui")]
    pub fn color(self) -> gpui::Rgba {
        gpui::rgb(self.rgb).alpha(self.alpha)
    }
}

/// Presenter colors. Opaque values use `0xRRGGBB`; translucent layers, overlays,
/// wells, controls, hairlines and titlebar hover/press overlays use [`Tint`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Palette {
    /// Full-bleed window field gradient ends (160deg). Besides the integrated
    /// titlebar (`titlebar-*`), the Home top header row and Settings title/Close
    /// header row float directly on this warm field using `ink` and
    /// `Utility`/`Primary` button fills; all other presenter content stays on L1/L2 surfaces.
    pub field_top: u32,
    pub field_bottom: u32,
    /// Up to three soft ambient blobs painted onto the L0 field when airy glass is active.
    pub blobs: [Tint; 3],
    /// L1 glass paper fill, solid fallback, 1px border edge, top inner highlight, and drop shadow.
    pub sheet: Tint,
    pub sheet_solid: u32,
    pub sheet_edge: Tint,
    pub sheet_highlight: Tint,
    pub sheet_shadow: Tint,
    /// L2 floating overlay fill, solid fallback, border edge, top inner highlight, and drop shadow.
    pub overlay: Tint,
    pub overlay_solid: u32,
    pub overlay_edge: Tint,
    pub overlay_highlight: Tint,
    pub overlay_shadow: Tint,
    /// Inset well inside a panel: note text, inputs and grouped settings.
    pub well: Tint,
    /// Neutral control fill and its hover/pressed states.
    pub control: Tint,
    pub control_hover: Tint,
    pub control_pressed: Tint,
    /// The selected option on a `control` track; always brighter than the track.
    pub segment: u32,
    /// Raised inline surface (e.g. list rows / diagram tiles).
    pub raised: u32,
    /// Decorative separator. Never a control boundary.
    pub hairline: Tint,
    /// Boundary for inputs that need one; >=3:1 on `well` and `sheet`.
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
    /// Integrated titlebar text, icons, and hover/pressed control fills.
    pub titlebar_ink: u32,
    pub titlebar_inactive_ink: u32,
    pub titlebar_icon: u32,
    pub titlebar_inactive_icon: u32,
    pub titlebar_control_hover: Tint,
    pub titlebar_control_pressed: Tint,
    /// Exclusive danger-red hover/pressed fills and icon foreground for the titlebar close button.
    pub titlebar_close_hover: u32,
    pub titlebar_close_pressed: u32,
    pub titlebar_on_close: u32,
    pub is_dark: bool,
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ButtonStyle {
    pub fill: Tint,
    pub ink: u32,
    pub hover: Tint,
    pub pressed: Tint,
    pub radius: f32,
    /// Non-color selected-state mark, drawn in a fixed slot.
    pub indicator: Option<u32>,
}

impl Palette {
    pub fn button(
        self,
        kind: ButtonKind,
        enabled: bool,
        selected: bool,
        glass: bool,
    ) -> ButtonStyle {
        let is_segmented = matches!(
            kind,
            ButtonKind::Segment | ButtonKind::Black | ButtonKind::White | ButtonKind::Tool
        );
        let mut style = ButtonStyle {
            fill: if is_segmented && !selected {
                Tint::TRANSPARENT
            } else {
                self.control.resolve(self.sheet_solid, glass)
            },
            ink: self.body,
            hover: self.control_hover.resolve(self.sheet_solid, glass),
            pressed: self.control_pressed.resolve(self.sheet_solid, glass),
            radius: PILL_RADIUS,
            indicator: None,
        };
        match kind {
            ButtonKind::Primary => {
                style.fill = Tint::opaque(self.accent);
                style.ink = self.on_accent;
                style.hover = Tint::opaque(self.accent_hover);
                style.pressed = Tint::opaque(self.accent_pressed);
                style.indicator = selected.then_some(self.on_accent);
            }
            ButtonKind::Stop => {
                style.fill = Tint::opaque(self.error_wash);
                style.ink = self.error;
                style.hover = Tint::opaque(over(self.ink, self.error_wash, 0.03));
                style.pressed = Tint::opaque(over(self.ink, self.error_wash, 0.06));
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
                style.fill = Tint::opaque(self.segment);
                style.ink = self.ink;
                style.hover = Tint::opaque(over(self.ink, self.segment, 0.03));
                style.pressed = Tint::opaque(over(self.ink, self.segment, 0.06));
                style.indicator = Some(self.focus);
            }
            _ if selected => {
                style.fill = Tint::opaque(self.accent_wash);
                style.ink = self.ink;
                style.indicator = Some(self.focus);
            }
            _ => {}
        }
        if !enabled {
            style.fill = if is_segmented {
                Tint::TRANSPARENT
            } else {
                self.control.resolve(self.sheet_solid, glass)
            };
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
    field_top: 0xFBF6F0,
    field_bottom: 0xF1E4D6,
    blobs: [
        Tint::new(0xF7D7BE, 0.65),
        Tint::new(0xF1CEB6, 0.65),
        Tint::new(0xF6E0C8, 0.65),
    ],
    sheet: Tint::new(0xFFFDFB, 0.66),
    sheet_solid: 0xFFFDF9,
    sheet_edge: Tint::new(0xFFFFFF, 0.78),
    sheet_highlight: Tint::new(0xFFFFFF, 0.65),
    sheet_shadow: Tint::new(0x4A3222, 0.14),
    overlay: Tint::new(0xFFFDFB, 0.96),
    overlay_solid: 0xFFFFFF,
    overlay_edge: Tint::new(0x1F1B17, 0.10),
    overlay_highlight: Tint::new(0xFFFFFF, 0.72),
    overlay_shadow: Tint::new(0x4A3222, 0.22),
    well: Tint::new(0x1F1B17, 0.045),
    control: Tint::new(0x1F1B17, 0.075),
    control_hover: Tint::new(0x1F1B17, 0.115),
    control_pressed: Tint::new(0x1F1B17, 0.155),
    segment: 0xFFFFFF,
    raised: 0xFFFFFF,
    hairline: Tint::new(0x1F1B17, 0.08),
    border: 0x7F776E,
    ink: 0x1F1B17,
    body: 0x3D3833,
    muted: 0x524B44,
    accent: 0xD97757,
    on_accent: 0x1F1712,
    accent_hover: 0xE0866A,
    accent_pressed: 0xC9694A,
    accent_text: 0x8A3B1E,
    focus: 0x8A3B1E,
    accent_wash: 0xF7E6DC,
    success: 0x215534,
    warning: 0x6B430A,
    error: 0x872A22,
    error_wash: 0xF8E3DF,
    titlebar_ink: 0x1F1B17,
    titlebar_inactive_ink: 0x524B44,
    titlebar_icon: 0x1F1B17,
    titlebar_inactive_icon: 0x524B44,
    titlebar_control_hover: Tint::new(0x1F1B17, 0.08),
    titlebar_control_pressed: Tint::new(0x1F1B17, 0.14),
    titlebar_close_hover: 0xC42B1C,
    titlebar_close_pressed: 0xA62317,
    titlebar_on_close: 0xFFFDF9,
    is_dark: false,
};

pub const DARK: Palette = Palette {
    field_top: 0x14110E,
    field_bottom: 0x1A1511,
    blobs: [
        Tint::new(0x2E1F15, 0.65),
        Tint::new(0x281B13, 0.65),
        Tint::new(0x251A13, 0.65),
    ],
    sheet: Tint::new(0x2D2722, 0.74),
    sheet_solid: 0x2B2521,
    sheet_edge: Tint::new(0xFFF4E8, 0.12),
    sheet_highlight: Tint::new(0xFFF4E8, 0.12),
    sheet_shadow: Tint::new(0x000000, 0.45),
    overlay: Tint::new(0x332D28, 0.96),
    overlay_solid: 0x332D28,
    overlay_edge: Tint::new(0xFFF4E8, 0.14),
    overlay_highlight: Tint::new(0xFFF4E8, 0.12),
    overlay_shadow: Tint::new(0x000000, 0.60),
    well: Tint::new(0x0A0806, 0.34),
    control: Tint::new(0xF5F1EB, 0.08),
    control_hover: Tint::new(0xF5F1EB, 0.12),
    control_pressed: Tint::new(0xF5F1EB, 0.16),
    segment: 0x544E48,
    raised: 0x332D28,
    hairline: Tint::new(0xFFFFFF, 0.08),
    border: 0x958C81,
    ink: 0xF5F1EB,
    body: 0xE2DBD2,
    muted: 0xC8C0B6,
    accent: 0xD97757,
    on_accent: 0x1F1712,
    accent_hover: 0xE0866A,
    accent_pressed: 0xC9694A,
    accent_text: 0xF0B296,
    focus: 0xF0B296,
    accent_wash: 0x3A2A22,
    success: 0x8DCEA0,
    warning: 0xE8B774,
    error: 0xFFACA7,
    error_wash: 0x3B2523,
    titlebar_ink: 0xF5F1EB,
    titlebar_inactive_ink: 0xACA49A,
    titlebar_icon: 0xF5F1EB,
    titlebar_inactive_icon: 0xACA49A,
    titlebar_control_hover: Tint::new(0xFFFFFF, 0.09),
    titlebar_control_pressed: Tint::new(0xFFFFFF, 0.15),
    titlebar_close_hover: 0xC42B1C,
    titlebar_close_pressed: 0xA62317,
    titlebar_on_close: 0xFFFDF9,
    is_dark: true,
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
    /// Every field color across gradient endpoints and blob peaks.
    fn fields(p: Palette) -> Vec<u32> {
        let mut fields = vec![p.field_top, p.field_bottom];
        for end in [p.field_top, p.field_bottom] {
            for blob in p.blobs {
                fields.push(blob.composite_over(end));
            }
        }
        fields
    }

    /// Every opaque L1 panel color text can land on (solid fallback + glass sheet over field/blobs).
    fn sheets(p: Palette) -> Vec<u32> {
        let mut out = vec![p.sheet_solid];
        for field in fields(p) {
            out.push(p.sheet.composite_over(field));
        }
        out
    }

    /// Every opaque L2 overlay color text can land on.
    fn overlays(p: Palette) -> Vec<u32> {
        let mut out = vec![p.overlay_solid, p.raised];
        for field in fields(p) {
            out.push(p.overlay.composite_over(field));
        }
        out
    }

    #[test]
    fn airy_glass_sheet_and_wells_avoid_brightness_inversion() {
        for p in [LIGHT, DARK] {
            let glass_sheets: Vec<u32> = fields(p)
                .into_iter()
                .map(|field| p.sheet.composite_over(field))
                .collect();
            let glass_wells: Vec<u32> = glass_sheets
                .iter()
                .map(|&sheet| p.well.composite_over(sheet))
                .collect();

            if p.is_dark {
                let max_field_lum = fields(p).into_iter().map(luminance).fold(0.0_f64, f64::max);
                let sheet_list = sheets(p);
                let min_sheet_lum = sheet_list
                    .iter()
                    .copied()
                    .map(luminance)
                    .fold(f64::INFINITY, f64::min);
                let max_well_lum = sheet_list
                    .iter()
                    .map(|&sheet| luminance(p.well.composite_over(sheet)))
                    .fold(0.0_f64, f64::max);
                assert!(
                    max_field_lum < min_sheet_lum,
                    "dark field/blob ceiling ({max_field_lum:.4}) must stay darker than L1 sheet floor ({min_sheet_lum:.4})"
                );
                assert!(
                    max_well_lum < min_sheet_lum,
                    "dark well ceiling ({max_well_lum:.4}) must sink below L1 sheet floor ({min_sheet_lum:.4})"
                );
                for &sheet in &sheet_list {
                    let well = p.well.composite_over(sheet);
                    let ctrl = p.control.composite_over(sheet);
                    let hover = p.control_hover.composite_over(sheet);
                    let pressed = p.control_pressed.composite_over(sheet);
                    assert!(
                        luminance(well) < luminance(sheet),
                        "dark well must sink darker than sheet"
                    );
                    assert!(
                        luminance(sheet) < luminance(ctrl),
                        "dark ctrl brighter than sheet"
                    );
                    assert!(luminance(ctrl) < luminance(hover));
                    assert!(luminance(hover) < luminance(pressed));
                    assert!(luminance(p.segment) > luminance(ctrl), "dark segment lift");
                    assert!(contrast(p.segment, ctrl) >= 1.1, "dark segment contrast");
                }
            } else {
                let min_sheet_lum = glass_sheets
                    .iter()
                    .map(|&c| luminance(c))
                    .fold(f64::INFINITY, f64::min);
                let max_well_lum = glass_wells
                    .iter()
                    .map(|&c| luminance(c))
                    .fold(0.0_f64, f64::max);
                assert!(
                    min_sheet_lum > max_well_lum,
                    "light glass sheet floor ({min_sheet_lum:.4}) must exceed glass well ceiling ({max_well_lum:.4})"
                );
                for &sheet in &sheets(p) {
                    let well = p.well.composite_over(sheet);
                    let ctrl = p.control.composite_over(sheet);
                    let hover = p.control_hover.composite_over(sheet);
                    let pressed = p.control_pressed.composite_over(sheet);
                    assert!(
                        luminance(sheet) > luminance(well),
                        "light well darker than sheet"
                    );
                    assert!(
                        luminance(well) > luminance(ctrl),
                        "light ctrl darker than well"
                    );
                    assert!(luminance(ctrl) > luminance(hover));
                    assert!(luminance(hover) > luminance(pressed));
                    assert!(luminance(p.segment) > luminance(ctrl), "light segment lift");
                    assert!(contrast(p.segment, ctrl) >= 1.1, "light segment contrast");
                }
            }
        }
    }

    #[test]
    fn button_solid_fallback_precomposites_and_segments_stay_transparent() {
        for p in [LIGHT, DARK] {
            for kind in [
                ButtonKind::Navigation,
                ButtonKind::Utility,
                ButtonKind::Destructive,
            ] {
                let solid = p.button(kind, true, false, false);
                assert_eq!(
                    solid.fill,
                    Tint::opaque(p.control.composite_over(p.sheet_solid))
                );
                assert_eq!(
                    solid.hover,
                    Tint::opaque(p.control_hover.composite_over(p.sheet_solid))
                );
                assert_eq!(
                    solid.pressed,
                    Tint::opaque(p.control_pressed.composite_over(p.sheet_solid))
                );
                let glass = p.button(kind, true, false, true);
                assert_eq!(glass.fill, p.control);
                assert_eq!(glass.hover, p.control_hover);
                assert_eq!(glass.pressed, p.control_pressed);
            }
            for kind in [
                ButtonKind::Segment,
                ButtonKind::Black,
                ButtonKind::White,
                ButtonKind::Tool,
            ] {
                for glass in [false, true] {
                    for enabled in [false, true] {
                        let unselected = p.button(kind, enabled, false, glass);
                        assert_eq!(
                            unselected.fill,
                            Tint::TRANSPARENT,
                            "{kind:?} unselected must be transparent"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn semantic_button_states_keep_text_focus_and_selection_readable() {
        for p in [LIGHT, DARK] {
            let panel_surfaces = sheets(p);
            for kind in KINDS {
                let is_segmented = matches!(
                    kind,
                    ButtonKind::Segment | ButtonKind::Black | ButtonKind::White | ButtonKind::Tool
                );
                for enabled in [false, true] {
                    for selected in [false, true] {
                        for glass in [false, true] {
                            let style = p.button(kind, enabled, selected, glass);
                            for &sheet in &panel_surfaces {
                                let base = if is_segmented {
                                    p.control.composite_over(sheet)
                                } else {
                                    sheet
                                };
                                let fill_bg = style.fill.composite_over(base);
                                let hover_bg = style.hover.composite_over(base);
                                let pressed_bg = style.pressed.composite_over(base);
                                assert!(
                                    contrast(style.ink, fill_bg) >= 4.5,
                                    "{kind:?} enabled={enabled} selected={selected} glass={glass} on {fill_bg:06x}"
                                );
                                if enabled {
                                    assert!(
                                        contrast(style.ink, hover_bg) >= 4.5,
                                        "hover {kind:?} on {hover_bg:06x}"
                                    );
                                    assert!(
                                        contrast(style.ink, pressed_bg) >= 4.5,
                                        "press {kind:?} on {pressed_bg:06x}"
                                    );
                                }
                                if kind != ButtonKind::Primary {
                                    assert!(
                                        contrast(p.focus, fill_bg) >= 3.0,
                                        "focus {kind:?} on {fill_bg:06x}"
                                    );
                                }
                                assert_eq!(style.indicator.is_some(), selected, "{kind:?}");
                                if let Some(mark) = style.indicator.filter(|_| enabled) {
                                    for bg in [fill_bg, hover_bg, pressed_bg] {
                                        assert!(
                                            contrast(mark, bg) >= 3.0,
                                            "mark {kind:?} {bg:06x}"
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
            for &sheet in &panel_surfaces {
                assert!(contrast(p.focus, sheet) >= 3.0, "focus on {sheet:06x}");
            }
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
            let sheet_list = sheets(p);
            let overlay_list = overlays(p);
            let mut bases = Vec::new();
            bases.extend_from_slice(&sheet_list);
            bases.extend_from_slice(&overlay_list);

            let mut surfaces = sheet_list.clone();
            surfaces.extend_from_slice(&overlay_list);
            surfaces.push(p.accent_wash);
            for &sheet in &sheet_list {
                surfaces.push(p.well.composite_over(sheet));
                surfaces.push(p.control.composite_over(sheet));
                surfaces.push(p.control_hover.composite_over(sheet));
                surfaces.push(p.control_pressed.composite_over(sheet));
            }
            for &overlay in &overlay_list {
                surfaces.push(p.well.composite_over(overlay));
                surfaces.push(p.control.composite_over(overlay));
                surfaces.push(p.control_hover.composite_over(overlay));
                surfaces.push(p.control_pressed.composite_over(overlay));
            }
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
            for &base in &bases {
                let well = p.well.composite_over(base);
                assert!(contrast(p.border, base) >= 3.0, "border/base {base:06x}");
                assert!(contrast(p.border, well) >= 3.0, "border/well {well:06x}");
                assert!(contrast(p.focus, well) >= 3.0, "focus/well {well:06x}");
            }
            assert!(contrast(p.error, p.error_wash) >= 4.5);
            assert!(contrast(p.sheet_solid, p.error) >= 4.5);
        }

        assert!(palette(Theme::System, true).is_dark);
        assert!(!palette(Theme::System, false).is_dark);
        assert_eq!(palette(Theme::System, true).sheet_solid, DARK.sheet_solid);
        assert_eq!(palette(Theme::System, false).sheet_solid, LIGHT.sheet_solid);
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
            for color in [
                p.field_top,
                p.field_bottom,
                p.blobs[0].rgb,
                p.blobs[1].rgb,
                p.blobs[2].rgb,
                p.sheet.rgb,
                p.sheet_solid,
                p.sheet_edge.rgb,
                p.sheet_highlight.rgb,
                p.overlay.rgb,
                p.overlay_solid,
                p.overlay_edge.rgb,
                p.overlay_highlight.rgb,
                p.well.rgb,
                p.control.rgb,
                p.control_hover.rgb,
                p.control_pressed.rgb,
                p.raised,
                p.segment,
                p.hairline.rgb,
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
                p.titlebar_ink,
                p.titlebar_inactive_ink,
                p.titlebar_icon,
                p.titlebar_inactive_icon,
                p.titlebar_control_hover.rgb,
                p.titlebar_control_pressed.rgb,
                p.titlebar_on_close,
            ] {
                let (hue, saturation) = hue_saturation(color);
                assert!(
                    !(90. ..=300.).contains(&hue) || saturation <= 0.25,
                    "{color:06x}: hue {hue:.0} saturation {saturation:.2}"
                );
            }
            for close_red in [p.titlebar_close_hover, p.titlebar_close_pressed] {
                let (hue, saturation) = hue_saturation(close_red);
                assert!(
                    (hue <= 20. || hue >= 340.) && saturation >= 0.5,
                    "close red {close_red:06x}: hue {hue:.0} saturation {saturation:.2}"
                );
                for kind in KINDS {
                    for enabled in [false, true] {
                        for selected in [false, true] {
                            for glass in [false, true] {
                                let style = p.button(kind, enabled, selected, glass);
                                assert_ne!(style.fill.rgb, close_red, "{kind:?} fill");
                                assert_ne!(style.hover.rgb, close_red, "{kind:?} hover");
                                assert_ne!(style.pressed.rgb, close_red, "{kind:?} pressed");
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn integrated_titlebar_states_meet_contrast_and_focus_distinction() {
        for p in [LIGHT, DARK] {
            let field_list = fields(p);
            for bg in field_list {
                assert!(
                    contrast(p.titlebar_ink, bg) >= 4.5,
                    "titlebar_ink on {bg:06x}"
                );
                assert!(
                    contrast(p.titlebar_icon, bg) >= 4.5,
                    "titlebar_icon on {bg:06x}"
                );
                assert!(
                    contrast(p.titlebar_inactive_ink, bg) >= 4.5,
                    "titlebar_inactive_ink on {bg:06x}"
                );
                assert!(
                    contrast(p.titlebar_inactive_icon, bg) >= 4.5,
                    "titlebar_inactive_icon on {bg:06x}"
                );
                let hover_bg = p.titlebar_control_hover.composite_over(bg);
                let press_bg = p.titlebar_control_pressed.composite_over(bg);
                assert!(contrast(p.titlebar_icon, hover_bg) >= 4.5);
                assert!(contrast(p.titlebar_icon, press_bg) >= 4.5);
                assert!(contrast(p.titlebar_inactive_icon, hover_bg) >= 3.0);
            }
            assert!(contrast(p.titlebar_on_close, p.titlebar_close_hover) >= 4.5);
            assert!(contrast(p.titlebar_on_close, p.titlebar_close_pressed) >= 4.5);

            assert_ne!(p.titlebar_ink, p.titlebar_inactive_ink);
            assert_ne!(p.titlebar_icon, p.titlebar_inactive_icon);
            assert!(contrast(p.titlebar_ink, p.titlebar_inactive_ink) >= 1.8);
            if p.is_dark {
                assert!(luminance(p.titlebar_ink) > luminance(p.titlebar_inactive_ink));
            } else {
                assert!(luminance(p.titlebar_ink) < luminance(p.titlebar_inactive_ink));
            }
        }
        assert_eq!(palette(Theme::System, true).titlebar_ink, DARK.titlebar_ink);
        assert_eq!(
            palette(Theme::System, false).titlebar_ink,
            LIGHT.titlebar_ink
        );
    }

    #[test]
    fn timer_readouts_and_split_session_pairs_meet_design_contrast() {
        for p in [LIGHT, DARK] {
            let sheet_list = sheets(p);
            let mut surfaces = vec![p.raised, p.sheet_solid];
            for &sheet in &sheet_list {
                surfaces.extend([
                    p.well.composite_over(sheet),
                    over(p.raised, sheet, READOUT_TOP_ALPHA),
                    over(p.sheet_solid, sheet, READOUT_BASE_ALPHA),
                ]);
            }
            for bg in surfaces {
                for text in [p.muted, p.accent_text, p.ink, p.warning, p.error] {
                    assert!(contrast(text, bg) >= 4.5, "{text:x}/{bg:x}");
                }
                for mark in [p.focus, p.muted, p.warning, p.error] {
                    assert!(contrast(mark, bg) >= 3.0, "{mark:x}/{bg:x}");
                }
            }
            for glass in [false, true] {
                let restart = p.button(ButtonKind::Navigation, true, false, glass);
                for &sheet in &sheet_list {
                    for fill in [restart.fill, restart.hover, restart.pressed] {
                        let bg = fill.composite_over(sheet);
                        assert!(contrast(p.muted, bg) >= 4.5, "{bg:06x}");
                    }
                }
            }
        }
    }

    #[test]
    fn field_header_text_and_buttons_meet_contrast_across_field_and_blobs() {
        for p in [LIGHT, DARK] {
            for field in fields(p) {
                assert!(contrast(p.ink, field) >= 4.5, "ink on field {field:06x}");
                assert!(
                    contrast(p.focus, field) >= 3.0,
                    "focus on field {field:06x}"
                );
                for kind in [ButtonKind::Utility, ButtonKind::Primary] {
                    for enabled in [false, true] {
                        for glass in [false, true] {
                            let style = p.button(kind, enabled, false, glass);
                            let fill_bg = style.fill.composite_over(field);
                            assert!(
                                contrast(style.ink, fill_bg) >= 4.5,
                                "{kind:?} enabled={enabled} glass={glass} on field {field:06x} -> {fill_bg:06x}"
                            );
                            if enabled {
                                let hover_bg = style.hover.composite_over(field);
                                let pressed_bg = style.pressed.composite_over(field);
                                assert!(
                                    contrast(style.ink, hover_bg) >= 4.5,
                                    "hover {kind:?} glass={glass} on field {field:06x} -> {hover_bg:06x}"
                                );
                                assert!(
                                    contrast(style.ink, pressed_bg) >= 4.5,
                                    "press {kind:?} glass={glass} on field {field:06x} -> {pressed_bg:06x}"
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn tint_resolve_keeps_alpha_in_glass_and_precomposites_in_solid() {
        for p in [LIGHT, DARK] {
            assert_eq!(p.control.resolve(p.sheet_solid, true), p.control);
            assert_eq!(
                p.control.resolve(p.sheet_solid, false),
                Tint::opaque(p.control.composite_over(p.sheet_solid))
            );
            assert_eq!(p.well.resolve(p.sheet_solid, true), p.well);
            assert_eq!(
                p.well.resolve(p.sheet_solid, false),
                Tint::opaque(p.well.composite_over(p.sheet_solid))
            );
        }
    }
}
