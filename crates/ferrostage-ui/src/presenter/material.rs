//! Self-contained airy frosted glass material engine for the presenter.
//! Windows remain `Opaque`; L0 ambient blobs, L1 glass sheets, translucent wells,
//! and L2 overlays are painted with GPUI primitives independently of OS blur.
use crate::theme::{self, Palette, Tint};
use gpui::{
    Background, BoxShadow, Div, Image, ImageFormat, ObjectFit, Rgba, Window,
    WindowBackgroundAppearance, div, img, linear_color_stop, linear_gradient, point, prelude::*,
    px, relative, rgb,
};
use std::sync::Arc;
use std::time::{Duration, Instant};

const BLOB_FADE: f32 = 0.70;
const BLOB_GEOMETRY: [(f32, f32, f32, f32); 3] = [
    (0.12, 0.06, 42.0 * 16.0, 32.0 * 16.0),
    (0.92, 0.96, 40.0 * 16.0, 32.0 * 16.0),
    (0.82, 0.10, 30.0 * 16.0, 24.0 * 16.0),
];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Surface {
    #[default]
    Disabled,
    Reduced,
    AiryGlass,
}

impl Surface {
    fn resolve(requested: bool, system_allows: bool) -> Self {
        if !requested {
            Self::Disabled
        } else if system_allows {
            Self::AiryGlass
        } else {
            Self::Reduced
        }
    }

    const fn status_key(self) -> &'static str {
        match self {
            Self::Disabled => "settings-glass-disabled",
            Self::Reduced => "settings-glass-system",
            Self::AiryGlass => "settings-glass-airy",
        }
    }
}

#[derive(Default)]
pub(super) struct Material {
    surface: Surface,
    policy: Option<(Instant, bool)>,
    opaque_ensured: bool,
    blob_cache: Option<(bool, [Arc<Image>; 3])>,
}

impl Material {
    pub(super) fn update(&mut self, requested: bool, window: &Window) {
        self.update_with(requested, system_allows_glass, |appearance| {
            window.set_background_appearance(appearance)
        });
    }

    fn update_with<P, A>(&mut self, requested: bool, mut probe_system: P, mut apply_appearance: A)
    where
        P: FnMut() -> bool,
        A: FnMut(WindowBackgroundAppearance),
    {
        if !self.opaque_ensured {
            apply_appearance(WindowBackgroundAppearance::Opaque);
            self.opaque_ensured = true;
        }
        // The OS can change reduced transparency or high contrast without firing
        // appearance changes. Bound native queries independently of frame rate.
        if requested
            && self
                .policy
                .is_none_or(|(at, _)| at.elapsed() >= Duration::from_secs(2))
        {
            self.policy = Some((Instant::now(), probe_system()));
        }
        let system_allows = self.policy.is_some_and(|(_, allowed)| allowed);
        self.surface = Surface::resolve(requested, system_allows);
    }

    pub(super) fn status_key(&self) -> &'static str {
        self.surface.status_key()
    }

    pub(super) fn is_glass(&self) -> bool {
        self.surface == Surface::AiryGlass
    }

    fn ensure_blobs(&mut self, p: Palette) -> &[Arc<Image>; 3] {
        let needs_rebuild = self
            .blob_cache
            .as_ref()
            .is_none_or(|(is_dark, _)| *is_dark != p.is_dark);
        if needs_rebuild {
            let blobs = [
                blob_svg(p.blobs[0], BLOB_FADE),
                blob_svg(p.blobs[1], BLOB_FADE),
                blob_svg(p.blobs[2], BLOB_FADE),
            ];
            self.blob_cache = Some((p.is_dark, blobs));
        }
        &self.blob_cache.as_ref().expect("blob cache populated").1
    }

    /// Full-bleed opaque 160deg window field gradient.
    pub(super) fn field(&self, p: Palette) -> Background {
        linear_gradient(
            160.,
            linear_color_stop(rgb(p.field_top), 0.),
            linear_color_stop(rgb(p.field_bottom), 1.),
        )
    }

    /// L0 full-bleed backdrop: warm gradient plus up to 3 cached SVG ambient blobs
    /// when airy glass is active, or pure solid gradient in solid fallback.
    pub(super) fn field_backdrop(&mut self, p: Palette) -> Div {
        let field = self.field(p);
        let glass = self.is_glass();
        let mut backdrop = div().absolute().inset_0().overflow_hidden().bg(field);
        if glass {
            let blobs = self.ensure_blobs(p);
            for (image, (x, y, rx, ry)) in blobs.iter().zip(BLOB_GEOMETRY) {
                backdrop = backdrop.child(
                    div()
                        .absolute()
                        .size(px(0.0))
                        .left(relative(x))
                        .top(relative(y))
                        .child(
                            img(image.clone())
                                .absolute()
                                .left(px(-rx))
                                .top(px(-ry))
                                .w(px(2.0 * rx))
                                .h(px(2.0 * ry))
                                .object_fit(ObjectFit::Fill),
                        ),
                );
            }
        }
        backdrop
    }

    /// One L1 glass paper surface: 1px edge, 1px top inner highlight, and a
    /// clipped bottom negative-spread shadow. Callers must NOT attach
    /// `overflow_hidden()` directly to this outer element.
    pub(super) fn panel(&self, p: Palette) -> Div {
        let (fill, edge) = if self.is_glass() {
            (p.sheet.color(), p.sheet_edge.color())
        } else {
            (rgb(p.sheet_solid), p.hairline.color())
        };
        div()
            .rounded(px(theme::PANEL_RADIUS))
            .bg(fill)
            .border_1()
            .border_color(edge)
            .shadow(vec![top_highlight(p.sheet_highlight)])
            .child(ground_shadow(
                soft_shadow(p.sheet_shadow, 12.0, 40.0, -16.0),
                theme::PANEL_RADIUS,
            ))
    }

    /// Alias of [`Self::panel`] matching `design-atlas` paper terminology.
    #[allow(dead_code)]
    pub(super) fn paper(&self, p: Palette) -> Div {
        self.panel(p)
    }

    /// L2 floating overlay: high-opacity overlay fill, 1px edge, top highlight,
    /// and a deeper clipped bottom drop shadow.
    pub(super) fn floating(&self, p: Palette) -> Div {
        floating_surface(p, self.is_glass())
    }

    pub(super) fn well_bg(&self, p: Palette) -> Rgba {
        well_bg_for(p, self.is_glass())
    }

    pub(super) fn control_bg(&self, p: Palette) -> Rgba {
        resolve_tint(p.control, p.sheet_solid, self.is_glass())
    }

    pub(super) fn control_hover_bg(&self, p: Palette) -> Rgba {
        resolve_tint(p.control_hover, p.sheet_solid, self.is_glass())
    }

    pub(super) fn control_pressed_bg(&self, p: Palette) -> Rgba {
        resolve_tint(p.control_pressed, p.sheet_solid, self.is_glass())
    }

    pub(super) fn hairline_color(&self, p: Palette) -> Rgba {
        p.hairline.color()
    }

    /// Shallow highlight on the shared header glass, above the inset wall clock.
    pub(super) fn readout(&self, p: Palette) -> Background {
        let (top, base) = if self.is_glass() {
            (theme::READOUT_TOP_ALPHA, theme::READOUT_BASE_ALPHA)
        } else {
            (1., 1.)
        };
        linear_gradient(
            180.,
            linear_color_stop(rgb(p.raised).alpha(top), 0.),
            linear_color_stop(rgb(p.sheet_solid).alpha(base), 1.),
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

    #[cfg(test)]
    fn expire_policy_for_test(&mut self) {
        if let Some((_, allowed)) = self.policy {
            self.policy = Some((Instant::now() - Duration::from_secs(3), allowed));
        }
    }

    #[cfg(test)]
    fn cached_blobs(&self) -> Option<&[Arc<Image>; 3]> {
        self.blob_cache.as_ref().map(|(_, blobs)| blobs)
    }

    #[cfg(all(test, feature = "test-support"))]
    pub(super) fn blob_ids(&self) -> Option<[u64; 3]> {
        self.blob_cache
            .as_ref()
            .map(|(_, blobs)| [blobs[0].id(), blobs[1].id(), blobs[2].id()])
    }
}

pub(super) fn well_bg_for(p: Palette, glass: bool) -> Rgba {
    resolve_tint(p.well, p.sheet_solid, glass)
}

pub(super) fn floating_surface(p: Palette, glass: bool) -> Div {
    let (fill, edge) = if glass {
        (p.overlay.color(), p.overlay_edge.color())
    } else {
        (rgb(p.overlay_solid), p.hairline.color())
    };
    div()
        .rounded(px(theme::PANEL_RADIUS))
        .bg(fill)
        .border_1()
        .border_color(edge)
        .shadow(vec![top_highlight(p.overlay_highlight)])
        .child(ground_shadow(
            soft_shadow(p.overlay_shadow, 24.0, 64.0, -24.0),
            theme::PANEL_RADIUS,
        ))
}

fn resolve_tint(tint: Tint, sheet_solid: u32, glass: bool) -> Rgba {
    if glass {
        tint.color()
    } else {
        rgb(tint.composite_over(sheet_solid))
    }
}

fn blob_svg(tint: Tint, edge: f32) -> Arc<Image> {
    let svg = format!(
        concat!(
            r##"<svg xmlns="http://www.w3.org/2000/svg" width="256" height="256">"##,
            r##"<defs><radialGradient id="g">"##,
            r##"<stop offset="0" stop-color="#{rgb:06X}" stop-opacity="{alpha:.2}"/>"##,
            r##"<stop offset="{edge:.2}" stop-color="#{rgb:06X}" stop-opacity="0"/>"##,
            r##"</radialGradient></defs>"##,
            r##"<rect width="256" height="256" fill="url(#g)"/></svg>"##,
        ),
        rgb = tint.rgb,
        alpha = tint.alpha,
        edge = edge,
    );
    Arc::new(Image::from_bytes(ImageFormat::Svg, svg.into_bytes()))
}

/// CSS `0 <y>px <blur>px <spread>px <colour>`. GPUI's blur_radius is the Gaussian sigma.
fn soft_shadow(color: Tint, y: f32, css_blur: f32, spread: f32) -> BoxShadow {
    BoxShadow {
        color: color.color().into(),
        offset: point(px(0.0), px(y)),
        blur_radius: px(css_blur / 2.0),
        spread_radius: px(spread),
        inset: false,
    }
}

/// CSS `inset 0 1px 0 <colour>`.
fn top_highlight(color: Tint) -> BoxShadow {
    BoxShadow {
        color: color.color().into(),
        offset: point(px(0.0), px(1.0)),
        blur_radius: px(0.0),
        spread_radius: px(0.0),
        inset: true,
    }
}

/// Keeps only the portion of a negative-spread outer shadow below the bottom edge
/// so the shadow never paints behind and darkens a translucent sheet.
fn ground_shadow(shadow: BoxShadow, radius: f32) -> Div {
    let reach = f32::from(shadow.offset.y) + 3.0 * f32::from(shadow.blur_radius);
    let drop = (1.0 - f32::from(shadow.offset.y) - f32::from(shadow.spread_radius)).max(0.0);
    div()
        .absolute()
        .left(px(-reach - 1.0))
        .right(px(-reach - 1.0))
        .top(relative(1.0))
        .mt(px(1.0))
        .h(px(reach))
        .overflow_hidden()
        .child(
            div()
                .absolute()
                .left(px(reach))
                .right(px(reach))
                .top(px(drop - 2.0 * reach))
                .h(px(2.0 * reach))
                .rounded(px(radius))
                .shadow(vec![shadow]),
        )
}

#[cfg(target_os = "windows")]
fn system_allows_glass() -> bool {
    use windows::UI::ViewManagement::{AccessibilitySettings, UISettings};
    let transparency = UISettings::new()
        .and_then(|settings| settings.AdvancedEffectsEnabled())
        .unwrap_or(true);
    let high_contrast = AccessibilitySettings::new()
        .and_then(|settings| settings.HighContrast())
        .unwrap_or(false);
    transparency && !high_contrast
}

#[cfg(target_os = "macos")]
fn system_allows_glass() -> bool {
    let workspace = objc2_app_kit::NSWorkspace::sharedWorkspace();
    !workspace.accessibilityDisplayShouldReduceTransparency()
        && !workspace.accessibilityDisplayShouldIncreaseContrast()
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn system_allows_glass() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::{Material, Surface};
    use crate::theme::{DARK, LIGHT};
    use gpui::WindowBackgroundAppearance;
    use std::sync::Arc;

    #[test]
    fn surface_resolves_requested_and_system_policy_without_width_or_os_fallbacks() {
        for system_allows in [false, true] {
            let disabled = Surface::resolve(false, system_allows);
            assert_eq!(disabled, Surface::Disabled);
            assert_eq!(disabled.status_key(), "settings-glass-disabled");
        }

        let airy = Surface::resolve(true, true);
        assert_eq!(airy, Surface::AiryGlass);
        assert_eq!(airy.status_key(), "settings-glass-airy");

        let reduced = Surface::resolve(true, false);
        assert_eq!(reduced, Surface::Reduced);
        assert_eq!(reduced.status_key(), "settings-glass-system");
    }

    #[test]
    fn material_keeps_window_opaque_throttles_probes_and_restores_airy_glass() {
        let mut material = Material::default();
        let mut probe_calls = 0usize;
        let mut appearances = Vec::new();
        let mut system_allows = true;

        // First glass update probes once and ensures Opaque once (never Blurred).
        material.update_with(
            true,
            || {
                probe_calls += 1;
                system_allows
            },
            |appearance| appearances.push(appearance),
        );
        assert!(material.is_glass());
        assert_eq!(material.status_key(), "settings-glass-airy");
        assert_eq!(probe_calls, 1);
        assert_eq!(appearances, vec![WindowBackgroundAppearance::Opaque]);

        // Immediate repeat stays cached within 2s and emits no duplicate appearance update.
        material.update_with(
            true,
            || {
                probe_calls += 1;
                system_allows
            },
            |appearance| appearances.push(appearance),
        );
        assert!(material.is_glass());
        assert_eq!(probe_calls, 1);
        assert_eq!(appearances, vec![WindowBackgroundAppearance::Opaque]);

        // Turning glass off switches to Disabled while keeping Opaque and preserving preference path.
        material.update_with(
            false,
            || {
                probe_calls += 1;
                system_allows
            },
            |appearance| appearances.push(appearance),
        );
        assert!(!material.is_glass());
        assert_eq!(material.status_key(), "settings-glass-disabled");
        assert_eq!(probe_calls, 1);

        // Re-enabling restores AiryGlass within the 2s cache window and never emits Blurred.
        material.update_with(
            true,
            || {
                probe_calls += 1;
                system_allows
            },
            |appearance| appearances.push(appearance),
        );
        assert!(material.is_glass());
        assert_eq!(material.status_key(), "settings-glass-airy");
        assert_eq!(appearances, vec![WindowBackgroundAppearance::Opaque]);

        // System reduced transparency or high contrast falls back to solid and recovers when restored.
        let mut reduced = Material::default();
        let mut reduced_appearances = Vec::new();
        system_allows = false;
        reduced.update_with(
            true,
            || system_allows,
            |appearance| reduced_appearances.push(appearance),
        );
        assert!(!reduced.is_glass());
        assert_eq!(reduced.status_key(), "settings-glass-system");
        assert_eq!(
            reduced_appearances,
            vec![WindowBackgroundAppearance::Opaque]
        );

        // Expire policy cache and restore OS transparency -> AiryGlass resumes automatically.
        reduced.expire_policy_for_test();
        system_allows = true;
        reduced.update_with(
            true,
            || system_allows,
            |appearance| reduced_appearances.push(appearance),
        );
        assert!(reduced.is_glass());
        assert_eq!(reduced.status_key(), "settings-glass-airy");
        assert_eq!(
            reduced_appearances,
            vec![WindowBackgroundAppearance::Opaque]
        );
    }

    #[test]
    fn blob_images_cache_per_theme_variant_and_never_rebuild_on_repeated_frames() {
        let mut material = Material::default();
        material.update_with(true, || true, |_| {});

        let _ = material.field_backdrop(LIGHT);
        let light_first = material.cached_blobs().expect("light blobs cached").clone();
        let _ = material.field_backdrop(LIGHT);
        let light_second = material
            .cached_blobs()
            .expect("light blobs still cached")
            .clone();
        for i in 0..3 {
            assert!(Arc::ptr_eq(&light_first[i], &light_second[i]));
            assert_eq!(light_first[i].format(), gpui::ImageFormat::Svg);
            let svg = std::str::from_utf8(light_first[i].bytes()).expect("valid svg");
            assert!(svg.contains("<radialGradient"));
            assert!(svg.contains("0.70"));
        }

        // Switching to DARK refreshes the cached blobs once, then reuses them across frames.
        let _ = material.field_backdrop(DARK);
        let dark_first = material.cached_blobs().expect("dark blobs cached").clone();
        assert!(!Arc::ptr_eq(&light_first[0], &dark_first[0]));
        let _ = material.field_backdrop(DARK);
        let dark_second = material.cached_blobs().expect("dark blobs reused").clone();
        for i in 0..3 {
            assert!(Arc::ptr_eq(&dark_first[i], &dark_second[i]));
        }
    }

    #[test]
    fn solid_fallback_precomposites_wells_and_controls_over_sheet_solid() {
        let mut material = Material::default();
        material.update_with(false, || true, |_| {});
        assert!(!material.is_glass());

        for p in [LIGHT, DARK] {
            assert_eq!(
                material.well_bg(p),
                gpui::rgb(p.well.composite_over(p.sheet_solid))
            );
            assert_eq!(
                material.control_bg(p),
                gpui::rgb(p.control.composite_over(p.sheet_solid))
            );
            assert_eq!(
                material.control_hover_bg(p),
                gpui::rgb(p.control_hover.composite_over(p.sheet_solid))
            );
            assert_eq!(
                material.control_pressed_bg(p),
                gpui::rgb(p.control_pressed.composite_over(p.sheet_solid))
            );
            assert_eq!(material.hairline_color(p), p.hairline.color());
        }

        material.update_with(true, || true, |_| {});
        assert!(material.is_glass());
        for p in [LIGHT, DARK] {
            assert_eq!(material.well_bg(p), p.well.color());
            assert_eq!(material.control_bg(p), p.control.color());
            assert_eq!(material.control_hover_bg(p), p.control_hover.color());
            assert_eq!(material.control_pressed_bg(p), p.control_pressed.color());
        }
    }
}
