//! Native window backdrop for the presenter; slide and audience pixels are independent.
use crate::theme::{self, Palette};
use gpui::{
    BoxShadow, Div, Hsla, Window, WindowBackgroundAppearance, div, linear_color_stop,
    linear_gradient, prelude::*, px, rgb, rgba,
};
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Surface {
    #[default]
    Disabled,
    Narrow,
    Reduced,
    Unsupported,
    Native,
}

impl Surface {
    fn resolve(requested: bool, width: f32, system: Self) -> Self {
        if !requested {
            Self::Disabled
        } else if width < 960. {
            Self::Narrow
        } else {
            system
        }
    }
}

#[derive(Default)]
pub(super) struct Material {
    surface: Surface,
    policy: Option<(Instant, Surface)>,
}

impl Material {
    pub(super) fn update(&mut self, requested: bool, window: &Window) {
        let width = f32::from(window.viewport_size().width);
        // The OS can change transparency without changing light/dark appearance.
        // Bound native policy queries independently of the presentation frame rate.
        if requested
            && width >= 960.
            && self
                .policy
                .is_none_or(|(at, _)| at.elapsed() >= Duration::from_secs(2))
        {
            let surface = match system_allows_blur() {
                Some(true) => Surface::Native,
                Some(false) => Surface::Reduced,
                None => Surface::Unsupported,
            };
            self.policy = Some((Instant::now(), surface));
        }
        let next = Surface::resolve(
            requested,
            width,
            self.policy.map_or(Surface::Unsupported, |(_, p)| p),
        );
        if (next == Surface::Native) != (self.surface == Surface::Native) {
            window.set_background_appearance(if next == Surface::Native {
                WindowBackgroundAppearance::Blurred
            } else {
                WindowBackgroundAppearance::Opaque
            });
        }
        self.surface = next;
    }

    pub(super) fn status_key(&self) -> &'static str {
        match self.surface {
            Surface::Disabled => "settings-glass-disabled",
            Surface::Narrow => "settings-glass-narrow",
            Surface::Reduced => "settings-glass-system",
            Surface::Unsupported => "settings-glass-unavailable",
            Surface::Native => "settings-glass-native",
        }
    }

    fn native(&self) -> bool {
        self.surface == Surface::Native
    }

    /// Full-bleed window field. Only the native surface lets the blurred desktop through.
    pub(super) fn field(&self, p: Palette) -> gpui::Background {
        let alpha = if self.native() { p.field_alpha } else { 1. };
        linear_gradient(
            160.,
            linear_color_stop(rgb(p.field_top).alpha(alpha), 0.),
            linear_color_stop(rgb(p.field_bottom).alpha(alpha), 1.),
        )
    }

    /// Panel layer color: translucent over the native field, otherwise solid.
    pub(super) fn layer(&self, p: Palette) -> Hsla {
        if self.native() {
            Hsla::from(rgb(p.layer)).alpha(p.layer_alpha)
        } else {
            rgb(p.layer_solid).into()
        }
    }

    /// One L1 panel: same hue as the field, bounded by its fill, never stroked.
    pub(super) fn panel(&self, p: Palette) -> Div {
        div().rounded(px(theme::PANEL_RADIUS)).bg(self.layer(p))
    }

    /// A status line on its own panel, led by a status-colored mark so the meaning
    /// never depends on the text color alone.
    pub(super) fn notice(&self, p: Palette, mark: u32) -> Div {
        self.panel(p)
            .flex_shrink_0()
            .min_h(px(theme::CONTROL))
            .px(px(12.))
            .py(px(4.))
            .flex()
            .items_center()
            .gap(px(8.))
            .child(
                div()
                    .size(px(8.))
                    .flex_shrink_0()
                    .rounded_full()
                    .bg(rgb(mark)),
            )
    }
}

/// L2 floating surface: the only layer with a hairline and a shadow.
pub(super) fn floating(p: Palette) -> Div {
    div()
        .rounded(px(theme::PANEL_RADIUS))
        .bg(rgb(p.raised))
        .border_1()
        .border_color(rgba(p.hairline))
        .shadow(vec![
            BoxShadow::new(px(0.), px(12.), rgba(p.shadow).into()).blur_radius(px(32.)),
        ])
}

#[cfg(target_os = "windows")]
fn system_allows_blur() -> Option<bool> {
    use windows::UI::ViewManagement::UISettings;
    // The locked GPUI Windows backend uses alpha-composited DirectComposition
    // windows. Its legacy opaque swap chain cannot show the desktop backdrop.
    if std::env::var("GPUI_DISABLE_DIRECT_COMPOSITION").is_ok_and(|v| v == "1" || v == "true") {
        return None;
    }
    UISettings::new()
        .and_then(|settings| settings.AdvancedEffectsEnabled())
        .ok()
}

#[cfg(target_os = "macos")]
fn system_allows_blur() -> Option<bool> {
    let workspace = objc2_app_kit::NSWorkspace::sharedWorkspace();
    Some(!workspace.accessibilityDisplayShouldReduceTransparency())
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn system_allows_blur() -> Option<bool> {
    // X11 only toggles alpha. GPUI doesn't expose whether the Wayland
    // compositor advertised its optional blur protocol.
    None
}

#[cfg(test)]
mod tests {
    use super::Surface;

    #[test]
    fn material_resolves_requested_width_and_system_policy_independently() {
        for system in [Surface::Reduced, Surface::Unsupported, Surface::Native] {
            assert_eq!(Surface::resolve(false, 1200., system), Surface::Disabled);
            assert_eq!(Surface::resolve(true, 959., system), Surface::Narrow);
            assert_eq!(Surface::resolve(true, 960., system), system);
        }
    }
}
