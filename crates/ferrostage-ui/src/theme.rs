//! The single presenter theme foundation from root DESIGN.md. Never used by AudienceView.
use ferrostage_settings::Theme;

pub const CONTROL: f32 = 44.0;
pub const FOCUS_WIDTH: f32 = 2.0;
pub const FOCUS_GAP: f32 = 2.0;
pub const RADIUS: f32 = 8.0;
pub const PANEL_RADIUS: f32 = 12.0;
pub const SPACING: f32 = 16.0;
pub const SANS: &str = "Segoe UI";
pub const DISPLAY: &str = "Georgia";
pub const MONO: &str = "Consolas";

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
pub fn display_font() -> gpui::Font {
    gpui::Font {
        fallbacks: Some(gpui::FontFallbacks::from_fonts(vec!["SimSun".into()])),
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
                style.fill = self.ink;
                style.ink = self.canvas;
                style.hover = self.body;
                style.pressed = self.body;
                style.radius = 4.;
            }
            ButtonKind::Black | ButtonKind::White => {
                style.fill = if kind == ButtonKind::Black {
                    0x000000
                } else {
                    0xFFFFFF
                };
                style.ink = if kind == ButtonKind::Black {
                    0xFFFFFF
                } else {
                    0x000000
                };
                style.hover = style.fill;
                style.pressed = style.fill;
                style.radius = 4.;
            }
            ButtonKind::Tool => {
                style.radius = CONTROL / 2.;
                if selected {
                    style.fill = self.card;
                    style.ink = self.ink;
                    style.border = self.focus;
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
                        assert!(contrast(style.border, palette.canvas) >= 3.);
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
