//! Native window backdrop for the presenter; slide and audience pixels are independent.
use crate::theme::{self, Palette};
use gpui::{
    Background, BoxShadow, Div, Window, WindowBackgroundAppearance, div, linear_color_stop,
    linear_gradient, prelude::*, px, rgb, rgba,
};
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Surface {
    #[default]
    Disabled,
    LegacyWindows,
    Narrow,
    Reduced,
    Unsupported,
    Native,
}

impl Surface {
    fn resolve(requested: bool, width: f32, windows_blur_supported: bool, system: Self) -> Self {
        if !requested {
            Self::Disabled
        } else if !windows_blur_supported {
            Self::LegacyWindows
        } else if width < 960. {
            Self::Narrow
        } else {
            system
        }
    }

    const fn status_key(self) -> &'static str {
        match self {
            Self::Disabled => "settings-glass-disabled",
            Self::LegacyWindows => "settings-glass-win10",
            Self::Narrow => "settings-glass-narrow",
            Self::Reduced => "settings-glass-system",
            Self::Unsupported => "settings-glass-unavailable",
            Self::Native => "settings-glass-native",
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
        self.update_with(
            requested,
            f32::from(window.viewport_size().width),
            ferrostage_platform::host_supports_windows_native_blur(),
            system_allows_blur,
            |appearance| window.set_background_appearance(appearance),
        );
    }

    fn update_with<P, A>(
        &mut self,
        requested: bool,
        width: f32,
        windows_blur_supported: bool,
        mut probe_system: P,
        mut apply_appearance: A,
    ) where
        P: FnMut() -> Option<bool>,
        A: FnMut(WindowBackgroundAppearance),
    {
        // The OS can change transparency without changing light/dark appearance.
        // Bound native policy queries independently of the presentation frame rate.
        if requested
            && windows_blur_supported
            && width >= 960.
            && self
                .policy
                .is_none_or(|(at, _)| at.elapsed() >= Duration::from_secs(2))
        {
            let surface = match probe_system() {
                Some(true) => Surface::Native,
                Some(false) => Surface::Reduced,
                None => Surface::Unsupported,
            };
            self.policy = Some((Instant::now(), surface));
        }
        let next = Surface::resolve(
            requested,
            width,
            windows_blur_supported,
            self.policy.map_or(Surface::Unsupported, |(_, p)| p),
        );
        if (next == Surface::Native) != (self.surface == Surface::Native) {
            apply_appearance(if next == Surface::Native {
                WindowBackgroundAppearance::Blurred
            } else {
                WindowBackgroundAppearance::Opaque
            });
        }
        self.surface = next;
    }

    pub(super) fn status_key(&self) -> &'static str {
        self.surface.status_key()
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

    /// Lift the upper part of each panel with one fill, including in solid mode.
    pub(super) fn layer(&self, p: Palette) -> Background {
        let (top, base, alpha) = if self.native() {
            (p.layer_top, p.layer, p.layer_alpha)
        } else {
            (p.layer_solid_top, p.layer_solid, 1.)
        };
        linear_gradient(
            180.,
            linear_color_stop(rgb(top).alpha(alpha), 0.),
            linear_color_stop(rgb(base).alpha(alpha), 0.55),
        )
    }

    /// One L1 panel: same hue as the field, bounded by its fill, never stroked.
    pub(super) fn panel(&self, p: Palette) -> Div {
        div().rounded(px(theme::PANEL_RADIUS)).bg(self.layer(p))
    }

    /// Shallow highlight on the shared header glass, above the inset wall clock.
    pub(super) fn readout(&self, p: Palette) -> gpui::Background {
        let (top, base) = if self.native() {
            (theme::READOUT_TOP_ALPHA, theme::READOUT_BASE_ALPHA)
        } else {
            (1., 1.)
        };
        linear_gradient(
            180.,
            linear_color_stop(rgb(p.raised).alpha(top), 0.),
            linear_color_stop(rgb(p.layer_solid_top).alpha(base), 1.),
        )
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
    use super::{Material, Surface};
    use gpui::WindowBackgroundAppearance;

    #[test]
    fn material_resolves_requested_width_windows_build_and_system_policy() {
        for system in [Surface::Reduced, Surface::Unsupported, Surface::Native] {
            for windows_blur_supported in [false, true] {
                for width in [959., 960., 1200.] {
                    let disabled = Surface::resolve(false, width, windows_blur_supported, system);
                    assert_eq!(disabled, Surface::Disabled);
                    assert_eq!(disabled.status_key(), "settings-glass-disabled");
                }
            }

            for width in [959., 960., 1200.] {
                let win10 = Surface::resolve(true, width, false, system);
                assert_eq!(win10, Surface::LegacyWindows);
                assert_eq!(win10.status_key(), "settings-glass-win10");
            }

            let narrow = Surface::resolve(true, 959., true, system);
            assert_eq!(narrow, Surface::Narrow);
            assert_eq!(narrow.status_key(), "settings-glass-narrow");

            for width in [960., 1200.] {
                assert_eq!(Surface::resolve(true, width, true, system), system);
            }
        }

        assert_eq!(Surface::Reduced.status_key(), "settings-glass-system");
        assert_eq!(
            Surface::Unsupported.status_key(),
            "settings-glass-unavailable"
        );
        assert_eq!(Surface::Native.status_key(), "settings-glass-native");
    }

    #[test]
    fn legacy_windows_skips_system_probe_and_keeps_opaque_surface() {
        let mut material = Material::default();
        let mut probe_calls = 0usize;
        let mut appearances = Vec::new();

        for width in [1200., 959., 1200.] {
            material.update_with(
                true,
                width,
                false,
                || {
                    probe_calls += 1;
                    Some(true)
                },
                |appearance| appearances.push(appearance),
            );
            assert!(!material.native());
            assert_eq!(material.status_key(), "settings-glass-win10");
        }

        material.update_with(
            false,
            1200.,
            false,
            || {
                probe_calls += 1;
                Some(true)
            },
            |appearance| appearances.push(appearance),
        );
        assert!(!material.native());
        assert_eq!(material.status_key(), "settings-glass-disabled");

        material.update_with(
            true,
            1200.,
            false,
            || {
                probe_calls += 1;
                Some(true)
            },
            |appearance| appearances.push(appearance),
        );
        assert!(!material.native());
        assert_eq!(material.status_key(), "settings-glass-win10");
        assert_eq!(probe_calls, 0);
        assert!(appearances.is_empty());
    }

    #[test]
    fn win11_probes_on_demand_and_toggles_window_background_only_on_native_transitions() {
        let mut material = Material::default();
        let mut probe_calls = 0usize;
        let mut appearances = Vec::new();

        // First wide update probes once and requests Blurred once.
        material.update_with(
            true,
            1200.,
            true,
            || {
                probe_calls += 1;
                Some(true)
            },
            |appearance| appearances.push(appearance),
        );
        assert!(material.native());
        assert_eq!(material.status_key(), "settings-glass-native");
        assert_eq!(probe_calls, 1);
        assert_eq!(appearances, vec![WindowBackgroundAppearance::Blurred]);

        // Immediate repeat stays cached within 2s and emits no duplicate appearance update.
        material.update_with(
            true,
            1200.,
            true,
            || {
                probe_calls += 1;
                Some(true)
            },
            |appearance| appearances.push(appearance),
        );
        assert!(material.native());
        assert_eq!(probe_calls, 1);
        assert_eq!(appearances, vec![WindowBackgroundAppearance::Blurred]);

        // Narrow window falls back to Opaque once and stays quiet on repeats.
        material.update_with(
            true,
            959.,
            true,
            || {
                probe_calls += 1;
                Some(true)
            },
            |appearance| appearances.push(appearance),
        );
        material.update_with(
            true,
            959.,
            true,
            || {
                probe_calls += 1;
                Some(true)
            },
            |appearance| appearances.push(appearance),
        );
        assert!(!material.native());
        assert_eq!(material.status_key(), "settings-glass-narrow");
        assert_eq!(probe_calls, 1);
        assert_eq!(
            appearances,
            vec![
                WindowBackgroundAppearance::Blurred,
                WindowBackgroundAppearance::Opaque,
            ]
        );

        // Widening back reuses the cached policy within 2s and restores Blurred once.
        material.update_with(
            true,
            960.,
            true,
            || {
                probe_calls += 1;
                Some(true)
            },
            |appearance| appearances.push(appearance),
        );
        assert!(material.native());
        assert_eq!(probe_calls, 1);
        assert_eq!(
            appearances,
            vec![
                WindowBackgroundAppearance::Blurred,
                WindowBackgroundAppearance::Opaque,
                WindowBackgroundAppearance::Blurred,
            ]
        );

        // Turning glass off restores Opaque once without re-probing.
        material.update_with(
            false,
            960.,
            true,
            || {
                probe_calls += 1;
                Some(true)
            },
            |appearance| appearances.push(appearance),
        );
        assert!(!material.native());
        assert_eq!(material.status_key(), "settings-glass-disabled");
        assert_eq!(probe_calls, 1);
        assert_eq!(
            appearances,
            vec![
                WindowBackgroundAppearance::Blurred,
                WindowBackgroundAppearance::Opaque,
                WindowBackgroundAppearance::Blurred,
                WindowBackgroundAppearance::Opaque,
            ]
        );

        // Reduced transparency and unsupported backends keep the window opaque.
        for (system_reply, expected_key) in [
            (Some(false), "settings-glass-system"),
            (None, "settings-glass-unavailable"),
        ] {
            let mut fallback = Material::default();
            let mut fallback_probes = 0usize;
            let mut fallback_appearances = Vec::new();
            fallback.update_with(
                true,
                1200.,
                true,
                || {
                    fallback_probes += 1;
                    system_reply
                },
                |appearance| fallback_appearances.push(appearance),
            );
            assert!(!fallback.native());
            assert_eq!(fallback.status_key(), expected_key);
            assert_eq!(fallback_probes, 1);
            assert!(fallback_appearances.is_empty());
        }
    }
}
