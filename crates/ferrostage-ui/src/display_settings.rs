//! Shared display-assignment draft. Numbers are view labels; only IDs become settings.
#[cfg(any(feature = "gpui", test))]
use crate::locale::Locale;
use crate::policy;
use ferrostage_core::{MonitorId, MonitorRole};
use ferrostage_platform::MonitorInfo;
use ferrostage_settings::{DisplaySettings, MonitorBinding, SingleMonitorMode};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PdfTarget {
    None,
    Windowed,
    Display(MonitorId),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DisplayDraft {
    pub revision: u64,
    pub presenter: Option<MonitorId>,
    pub pdf: PdfTarget,
}

impl DisplayDraft {
    pub fn new(revision: u64, monitors: &[MonitorInfo], settings: &DisplaySettings) -> Self {
        let roles = policy::roles(monitors, settings);
        Self {
            revision,
            presenter: roles.presenter,
            pdf: match policy::audience_target(monitors, settings) {
                Ok((_, false)) if monitors.len() == 1 => PdfTarget::Windowed,
                Ok((id, _)) => PdfTarget::Display(id),
                Err(_) => PdfTarget::None,
            },
        }
    }

    /// Return a locale key for an invalid draft; never reinterpret a missing ID.
    pub fn validation_error(
        &self,
        revision: u64,
        monitors: &[MonitorInfo],
    ) -> Option<&'static str> {
        if self.revision != revision {
            return Some("displays-stale");
        }
        let unique = |id: &MonitorId| monitors.iter().filter(|m| &m.id == id).count() == 1;
        if self.presenter.as_ref().is_none_or(|id| !unique(id)) {
            return Some("displays-target-missing");
        }
        match &self.pdf {
            PdfTarget::Display(id) if !unique(id) => Some("displays-target-missing"),
            PdfTarget::Display(id) if self.presenter.as_ref() == Some(id) => {
                Some("displays-conflict")
            }
            PdfTarget::Windowed if monitors.len() != 1 => Some("displays-windowed-single"),
            _ => None,
        }
    }

    pub fn settings(
        &self,
        revision: u64,
        monitors: &[MonitorInfo],
        current: &DisplaySettings,
    ) -> Result<DisplaySettings, &'static str> {
        if let Some(error) = self.validation_error(revision, monitors) {
            return Err(error);
        }
        let mut next = current.clone();
        next.single_monitor_mode = if self.pdf == PdfTarget::Windowed {
            SingleMonitorMode::WindowedAudience
        } else {
            SingleMonitorMode::PresenterOnly
        };
        // Explicit None bindings suppress the unsaved-role heuristic. Replace
        // old offline bindings too, so reconnection cannot revive an old role.
        next.monitors = numbered(monitors)
            .into_iter()
            .map(|(_, monitor)| MonitorBinding {
                id: monitor.id.0.clone(),
                friendly_name: monitor.name.clone(),
                role: if self.presenter.as_ref() == Some(&monitor.id) {
                    MonitorRole::Presenter
                } else if self.pdf == PdfTarget::Display(monitor.id.clone()) {
                    MonitorRole::Audience
                } else {
                    MonitorRole::None
                },
                source_occurrence: None,
            })
            .collect();
        Ok(next)
    }
}

/// Shared by the list, selectors and Identify. Never used as persisted identity.
pub fn numbered(monitors: &[MonitorInfo]) -> Vec<(usize, &MonitorInfo)> {
    let mut sorted = monitors.iter().collect::<Vec<_>>();
    sorted.sort_by(|a, b| a.id.cmp(&b.id));
    sorted
        .into_iter()
        .enumerate()
        .map(|(index, monitor)| (index + 1, monitor))
        .collect()
}

#[cfg(any(feature = "gpui", test))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ScreenCopy {
    pub title: String,
    pub meta: String,
    pub identify: String,
}

/// One label for the card, the role controls, and Identify.
#[cfg(any(feature = "gpui", test))]
pub(crate) fn screen_copy(
    locale: Locale,
    monitors: &[MonitorInfo],
    monitor: &MonitorInfo,
) -> ScreenCopy {
    let resolution = format!("{}×{}", monitor.native_size.0, monitor.native_size.1);
    let portrait = is_portrait(monitor);
    let model = distinct_model(&monitor.name);
    let title = if let Some(model) = model {
        if monitor.built_in {
            format!("{} {model}", locale.text("displays-builtin"))
        } else {
            model.to_owned()
        }
    } else if monitor.built_in {
        locale.text("displays-builtin").to_owned()
    } else if portrait {
        locale.text("displays-portrait").to_owned()
    } else {
        resolution.clone()
    };
    let mut meta = Vec::new();
    if title != resolution {
        meta.push(resolution);
    }
    if portrait && title != locale.text("displays-portrait") {
        meta.push(locale.text("displays-portrait").to_owned());
    }
    if monitor.is_primary {
        meta.push(locale.text("displays-primary").to_owned());
    }
    if let Some(key) = placement_key(monitors, monitor) {
        meta.push(locale.text(key).to_owned());
    }
    let meta = meta.join(" · ");
    let identify = if meta.is_empty() {
        title.clone()
    } else {
        format!("{title} · {meta}")
    };
    ScreenCopy {
        title,
        meta,
        identify,
    }
}

#[cfg(any(feature = "gpui", test))]
fn is_portrait(monitor: &MonitorInfo) -> bool {
    matches!(monitor.rotation_degrees, 90 | 270) || monitor.native_size.1 > monitor.native_size.0
}

#[cfg(any(feature = "gpui", test))]
fn distinct_model(name: &str) -> Option<&str> {
    let trimmed = name.trim();
    let lower = trimmed.to_ascii_lowercase();
    if trimmed.is_empty()
        || lower == "generic pnp monitor"
        || lower == "generic monitor"
        || lower == "generic non-pnp monitor"
        || lower.starts_with("generic pnp monitor")
        || lower.starts_with("generic monitor")
    {
        None
    } else {
        Some(trimmed)
    }
}

#[cfg(any(feature = "gpui", test))]
fn placement_key(monitors: &[MonitorInfo], monitor: &MonitorInfo) -> Option<&'static str> {
    if monitors.len() < 2 {
        return None;
    }
    let center = |item: &MonitorInfo| {
        (
            f64::from(item.position.0) + f64::from(item.native_size.0) / 2.0,
            f64::from(item.position.1) + f64::from(item.native_size.1) / 2.0,
        )
    };
    let (x, y) = center(monitor);
    let centers = monitors.iter().map(center).collect::<Vec<_>>();
    let (min_x, max_x, min_y, max_y) = centers.iter().fold(
        (f64::MAX, f64::MIN, f64::MAX, f64::MIN),
        |(min_x, max_x, min_y, max_y), (cx, cy)| {
            (
                min_x.min(*cx),
                max_x.max(*cx),
                min_y.min(*cy),
                max_y.max(*cy),
            )
        },
    );
    let span_x = max_x - min_x;
    let span_y = max_y - min_y;
    if span_x < 1.0 && span_y < 1.0 {
        return None;
    }
    let edge = |value: f64, min: f64, max: f64, low: &'static str, high: &'static str| {
        if (value - min).abs() <= 1.0 {
            Some(low)
        } else if (value - max).abs() <= 1.0 {
            Some(high)
        } else if monitor.is_primary {
            None
        } else {
            Some("displays-center")
        }
    };
    if span_x >= span_y {
        edge(x, min_x, max_x, "displays-left", "displays-right")
    } else {
        edge(y, min_y, max_y, "displays-above", "displays-below")
    }
}

pub fn topology_aspect(monitors: &[MonitorInfo]) -> f32 {
    let min_x = monitors
        .iter()
        .map(|m| i64::from(m.position.0))
        .min()
        .unwrap_or(0);
    let min_y = monitors
        .iter()
        .map(|m| i64::from(m.position.1))
        .min()
        .unwrap_or(0);
    let max_x = monitors
        .iter()
        .map(|m| i64::from(m.position.0) + i64::from(m.native_size.0))
        .max()
        .unwrap_or(1);
    let max_y = monitors
        .iter()
        .map(|m| i64::from(m.position.1) + i64::from(m.native_size.1))
        .max()
        .unwrap_or(1);
    (max_x - min_x).max(1) as f32 / (max_y - min_y).max(1) as f32
}

/// Normalized physical desktop rectangles; negative origins and gaps are retained.
/// Returned coordinates use a shared physical scale; the view uses the desktop aspect ratio.
pub fn topology_rectangles(monitors: &[MonitorInfo]) -> Vec<(usize, f32, f32, f32, f32)> {
    let Some(min_x) = monitors.iter().map(|m| i64::from(m.position.0)).min() else {
        return Vec::new();
    };
    let min_y = monitors
        .iter()
        .map(|m| i64::from(m.position.1))
        .min()
        .unwrap_or(0);
    let max_x = monitors
        .iter()
        .map(|m| i64::from(m.position.0) + i64::from(m.native_size.0))
        .max()
        .unwrap_or(min_x + 1);
    let max_y = monitors
        .iter()
        .map(|m| i64::from(m.position.1) + i64::from(m.native_size.1))
        .max()
        .unwrap_or(min_y + 1);
    let w = (max_x - min_x).max(1) as f32;
    let h = (max_y - min_y).max(1) as f32;
    numbered(monitors)
        .into_iter()
        .map(|(n, m)| {
            (
                n,
                (i64::from(m.position.0) - min_x) as f32 / w,
                (i64::from(m.position.1) - min_y) as f32 / h,
                m.native_size.0 as f32 / w,
                m.native_size.1 as f32 / h,
            )
        })
        .collect()
}

/// Presenter-only observations. Requested allocation is never an actual location.
#[cfg(any(feature = "gpui", test))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum WindowLocation {
    NotRunning,
    Unknown,
    Monitor(MonitorId),
}

#[cfg(any(feature = "gpui", test))]
pub(crate) fn observed_location(
    running: bool,
    display: Option<u64>,
    valid: bool,
    monitors: &[MonitorInfo],
    mapping: Option<&std::collections::BTreeMap<MonitorId, u64>>,
) -> WindowLocation {
    if !running {
        return WindowLocation::NotRunning;
    }
    if valid && let (Some(display), Some(mapping)) = (display, mapping) {
        let mut matches = mapping.iter().filter(|(_, value)| **value == display);
        if let Some((id, _)) = matches.next()
            && matches.next().is_none()
            && monitors.iter().filter(|m| &m.id == id).count() == 1
        {
            return WindowLocation::Monitor(id.clone());
        }
    }
    WindowLocation::Unknown
}

#[cfg(any(feature = "gpui", test))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RoleProjection {
    pub app: WindowLocation,
    pub pdf: WindowLocation,
    pub configured: DisplayDraft,
    pub pending: bool,
}

#[cfg(any(feature = "gpui", test))]
impl RoleProjection {
    pub fn new(
        app: WindowLocation,
        pdf: WindowLocation,
        configured: DisplayDraft,
        draft: &DisplayDraft,
    ) -> Self {
        let pending = draft.presenter != configured.presenter || draft.pdf != configured.pdf;
        Self {
            app,
            pdf,
            configured,
            pending,
        }
    }

    /// These role keys are consumed by both the schematic and its corresponding row.
    pub fn markers(&self, id: &MonitorId, draft: &DisplayDraft) -> Vec<(&'static str, bool)> {
        let mut markers = Vec::new();
        if self.app == WindowLocation::Monitor(id.clone()) {
            markers.push(("displays-actual-app", true));
        }
        if self.pdf == WindowLocation::Monitor(id.clone()) {
            markers.push(("displays-actual-pdf", true));
        }
        if self.configured.presenter.as_ref() == Some(id)
            && self.app != WindowLocation::Monitor(id.clone())
        {
            markers.push(("displays-configured-app", false));
        }
        let configured_pdf = match &self.configured.pdf {
            PdfTarget::Display(target) => target == id,
            PdfTarget::Windowed => self.configured.presenter.as_ref() == Some(id),
            PdfTarget::None => false,
        };
        if configured_pdf && self.pdf != WindowLocation::Monitor(id.clone()) {
            markers.push((
                if self.pdf == WindowLocation::NotRunning {
                    "displays-configured-pdf-idle"
                } else {
                    "displays-configured-pdf"
                },
                false,
            ));
        }
        if self.pending {
            if draft.presenter != self.configured.presenter && draft.presenter.as_ref() == Some(id)
            {
                markers.push(("displays-pending-app", false));
            }
            let draft_pdf = match &draft.pdf {
                PdfTarget::Display(target) => target == id,
                PdfTarget::Windowed => draft.presenter.as_ref() == Some(id),
                PdfTarget::None => false,
            };
            if draft_pdf
                && (draft.pdf != self.configured.pdf
                    || draft.presenter != self.configured.presenter
                        && draft.pdf == PdfTarget::Windowed)
            {
                markers.push(("displays-pending-pdf", false));
            }
        }
        if markers.is_empty() {
            markers.push(("displays-unused", false));
        }
        markers
    }
}

#[cfg(feature = "gpui")]
#[derive(Clone)]
pub(crate) struct DisplayPanel {
    pub monitors: Vec<MonitorInfo>,
    pub effective: DisplaySettings,
    pub draft: DisplayDraft,
    pub problem: Option<String>,
    pub operation_error: Option<String>,
    pub monitor_error: Option<String>,
    pub topology_valid: bool,
    pub editable: bool,
    pub can_identify: bool,
    pub busy: bool,
    pub roles: RoleProjection,
    pub details: std::collections::BTreeSet<MonitorId>,
}

#[cfg(feature = "gpui")]
pub(crate) mod view {
    use super::*;
    use crate::{
        actions::Intent,
        presenter::Presenter,
        theme::{self, Palette},
    };
    use gpui::{IntoElement, div, prelude::*, px, rgb};

    fn location_text(location: &WindowLocation, monitors: &[MonitorInfo], l: Locale) -> String {
        match location {
            WindowLocation::NotRunning => l.text("displays-not-running").into(),
            WindowLocation::Unknown => l.text("displays-location-unknown").into(),
            WindowLocation::Monitor(id) => numbered(monitors)
                .into_iter()
                .find(|(_, m)| &m.id == id)
                .map(|(number, _)| {
                    l.format("displays-screen-number", &[("number", number.to_string())])
                })
                .unwrap_or_else(|| l.text("displays-location-unknown").into()),
        }
    }
    fn markers(
        view: &Presenter,
        panel: &DisplayPanel,
        id: &MonitorId,
        l: Locale,
        p: Palette,
    ) -> gpui::Div {
        let mut line = div()
            .flex()
            .flex_wrap()
            .gap(px(4.))
            .text_size(px(view.ui_text_size(12.)));
        for (key, actual) in panel.roles.markers(id, &panel.draft) {
            line = line.child(
                div()
                    .px(px(4.))
                    .py(px(2.))
                    .rounded(px(4.))
                    .border_1()
                    .border_color(rgb(if key.starts_with("displays-pending-") {
                        p.focus
                    } else if actual {
                        p.card
                    } else {
                        p.canvas
                    }))
                    .bg(rgb(if actual { p.card } else { p.canvas }))
                    .text_color(rgb(if actual { p.ink } else { p.muted }))
                    .child(l.text(key)),
            );
        }
        line
    }
    fn body(
        view: &Presenter,
        panel: &DisplayPanel,
        l: Locale,
        p: Palette,
        width: f32,
    ) -> gpui::Div {
        let mut content = div().flex().flex_col().gap(px(8.)).min_w_0();
        content = content
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .gap(px(8.))
                    .child(
                        div()
                            .id("display-detection-status")
                            .debug_selector(|| "display-detection-status".into())
                            .flex_1()
                            .min_w_0()
                            .whitespace_normal()
                            .when(panel.monitor_error.is_some(), |status| {
                                status
                                    .role(gpui::Role::Alert)
                                    .text_size(px(view.ui_text_size(12.)))
                                    .text_color(rgb(p.error))
                            })
                            .child(if let Some(error) = &panel.monitor_error {
                                error.clone()
                            } else if panel.topology_valid {
                                l.format(
                                    "displays-count",
                                    &[("count", panel.monitors.len().to_string())],
                                )
                            } else {
                                l.text("displays-unavailable").into()
                            }),
                    )
                    .child(view.button(
                        "identify-displays".into(),
                        l.text("displays-identify").into(),
                        panel.can_identify,
                        false,
                        Intent::Identify,
                        p,
                    ))
                    .child(view.button(
                        "refresh-displays".into(),
                        l.text("displays-refresh").into(),
                        !panel.busy,
                        false,
                        Intent::RefreshDisplays,
                        p,
                    )),
            )
            .child(
                div()
                    .text_size(px(view.ui_text_size(12.)))
                    .text_color(rgb(p.muted))
                    .whitespace_normal()
                    .child(format!(
                        "{} {}",
                        l.text("displays-number-help"),
                        l.text("displays-identify-help")
                    )),
            );
        if !panel.topology_valid {
            let mut recovery = div()
                .id("display-recovery")
                .debug_selector(|| "display-recovery".into())
                .flex()
                .flex_col()
                .gap(px(4.))
                .text_size(px(view.ui_text_size(12.)))
                .text_color(rgb(p.muted))
                .whitespace_normal();
            for key in [
                "displays-recovery-wait",
                "displays-recovery-windows",
                "displays-recovery-apply",
            ] {
                recovery = recovery.child(div().child(l.text(key)));
            }
            content = content.child(recovery);
        }
        if panel.topology_valid {
            for (index, warning) in policy::roles(&panel.monitors, &panel.effective)
                .warnings
                .iter()
                .enumerate()
            {
                let key = match warning {
                    ferrostage_platform::TopologyWarning::NoMonitors => "displays-empty",
                    ferrostage_platform::TopologyWarning::SavedAudienceMissing(_) => {
                        "displays-warning-audience-missing"
                    }
                    ferrostage_platform::TopologyWarning::PresenterMissingFallbackPrimary => {
                        "displays-warning-presenter-missing"
                    }
                    _ => "displays-warning-duplicate",
                };
                content = content.child(
                    div()
                        .id(gpui::SharedString::from(format!(
                            "topology-warning-{index}"
                        )))
                        .role(gpui::Role::Alert)
                        .text_color(rgb(p.warning))
                        .child(l.text(key)),
                );
            }
            let side_by_side = width >= 760.;
            let preview_width = if side_by_side {
                (width * 0.4).min(theme::DISPLAY_PREVIEW_WIDTH)
            } else {
                width.clamp(1., 720.)
            };
            let preview_height = theme::DISPLAY_PREVIEW_HEIGHT;
            let aspect = topology_aspect(&panel.monitors);
            let fit_width = (preview_width - 16.)
                .min((preview_height - 16.) * aspect)
                .max(1.);
            let fit_height = fit_width / aspect;
            let mut diagram = div()
                .id("display-topology")
                .debug_selector(|| "display-topology".into())
                .relative()
                .w(px(preview_width))
                .h(px(preview_height))
                .flex_shrink_0()
                .bg(rgb(p.soft))
                .rounded(px(theme::RADIUS));
            let mut callouts = Vec::new();
            for (number, x, y, w, h) in topology_rectangles(&panel.monitors) {
                let monitor = numbered(&panel.monitors)[number - 1].1;
                let rect_w = fit_width * w;
                let rect_h = fit_height * h;
                let roles = panel.roles.markers(&monitor.id, &panel.draft);
                let actual = roles.iter().any(|(_, actual)| *actual);
                let mut screen = div()
                    .id(gpui::SharedString::from(format!(
                        "display-diagram-{number}"
                    )))
                    .absolute()
                    .left(px((preview_width - fit_width) / 2. + fit_width * x))
                    .top(px((preview_height - fit_height) / 2. + fit_height * y))
                    .w(px(rect_w))
                    .h(px(rect_h))
                    .border_1()
                    .border_color(rgb(if actual { p.focus } else { p.border }))
                    .bg(rgb(p.card))
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .gap(px(2.))
                    .text_color(rgb(p.ink))
                    .text_size(px(view.ui_text_size(16.)))
                    .aria_label(format!(
                        "{number}: {}",
                        roles
                            .iter()
                            .map(|(key, _)| l.text(key))
                            .collect::<Vec<_>>()
                            .join(", ")
                    ))
                    .child(number.to_string());
                // Small physical rectangles use the number-linked callouts below.
                if rect_w >= view.ui_text_size(100.)
                    && rect_h >= view.ui_text_size(30. + roles.len() as f32 * 18.)
                {
                    for (key, _) in &roles {
                        let key = match *key {
                            "displays-actual-app" => "displays-diagram-app",
                            "displays-actual-pdf" => "displays-diagram-pdf",
                            "displays-configured-app" => "displays-diagram-app-target",
                            "displays-configured-pdf" | "displays-configured-pdf-idle" => {
                                "displays-diagram-pdf-target"
                            }
                            "displays-pending-app" => "displays-diagram-app-pending",
                            "displays-pending-pdf" => "displays-diagram-pdf-pending",
                            _ => "displays-unused",
                        };
                        screen = screen.child(
                            div()
                                .text_size(px(view.ui_text_size(12.)))
                                .child(l.text(key)),
                        );
                    }
                } else {
                    callouts.push(number);
                }
                diagram = diagram.child(screen);
            }
            let mut overview = div()
                .flex()
                .flex_col()
                .gap(px(8.))
                .flex_shrink_0()
                .when(side_by_side, |v| v.w(px(preview_width)))
                .child(diagram);
            for (number, monitor) in numbered(&panel.monitors) {
                if !callouts.contains(&number) {
                    continue;
                }
                overview = overview.child(
                    div()
                        .id(gpui::SharedString::from(format!(
                            "display-callout-{number}"
                        )))
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .child(number.to_string())
                        .child(markers(view, panel, &monitor.id, l, p)),
                );
            }
            overview = overview.child(
                div()
                    .id("display-role-summary")
                    .debug_selector(|| "display-role-summary".into())
                    .flex()
                    .flex_col()
                    .gap(px(4.))
                    .text_size(px(view.ui_text_size(12.)))
                    .text_color(rgb(p.muted))
                    .child(format!(
                        "{}: {}",
                        l.text("displays-role-app"),
                        location_text(&panel.roles.app, &panel.monitors, l)
                    ))
                    .child(format!(
                        "{}: {}",
                        l.text("displays-role-pdf"),
                        if panel.roles.pdf == WindowLocation::NotRunning
                            && panel.roles.configured.pdf == PdfTarget::None
                        {
                            l.text("displays-disabled").into()
                        } else {
                            location_text(&panel.roles.pdf, &panel.monitors, l)
                        }
                    )),
            );
            let mut rows = div().flex().flex_col().min_w_0().flex_1();
            for (number, monitor) in numbered(&panel.monitors) {
                let copy = screen_copy(l, &panel.monitors, monitor);
                let app_selected = panel.draft.presenter.as_ref() == Some(&monitor.id);
                let (pdf_id, pdf_label, target) = if panel.monitors.len() == 1 {
                    (
                        "display-pdf-windowed".to_owned(),
                        "displays-role-windowed",
                        PdfTarget::Windowed,
                    )
                } else {
                    (
                        format!("display-pdf-{number}"),
                        "displays-role-pdf",
                        PdfTarget::Display(monitor.id.clone()),
                    )
                };
                let selected = panel.draft.pdf == target;
                let picked = |key, active| {
                    if active {
                        l.format("displays-role-picked", &[("label", l.text(key).into())])
                    } else {
                        l.text(key).into()
                    }
                };
                let details = panel.details.contains(&monitor.id);
                let mut row = div()
                    .id(gpui::SharedString::from(format!("display-card-{number}")))
                    .debug_selector(move || format!("display-card-{number}"))
                    .min_w_0()
                    .border_b_1()
                    .border_color(rgb(p.border))
                    .py(px(8.))
                    .flex()
                    .flex_col()
                    .gap(px(4.))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(8.))
                            .child(
                                div()
                                    .id(gpui::SharedString::from(format!("display-index-{number}")))
                                    .w(px(theme::CONTROL))
                                    .h(px(theme::CONTROL))
                                    .flex_shrink_0()
                                    .rounded(px(theme::RADIUS))
                                    .bg(rgb(p.soft))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .child(number.to_string()),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .flex()
                                    .flex_col()
                                    .gap(px(2.))
                                    .child(
                                        div()
                                            .text_size(px(view.ui_text_size(14.)))
                                            .whitespace_normal()
                                            .child(copy.title),
                                    )
                                    .child(
                                        div()
                                            .text_size(px(view.ui_text_size(12.)))
                                            .text_color(rgb(p.muted))
                                            .whitespace_normal()
                                            .child(copy.meta),
                                    ),
                            )
                            .child(
                                view.button(
                                    format!("display-details-{number}"),
                                    l.text(if details {
                                        "displays-details-hide"
                                    } else {
                                        "displays-details"
                                    })
                                    .into(),
                                    true,
                                    false,
                                    Intent::ToggleDisplayDetails(monitor.id.clone()),
                                    p,
                                ),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_wrap()
                            .items_center()
                            .justify_between()
                            .gap(px(4.))
                            .child(markers(view, panel, &monitor.id, l, p))
                            .child(
                                div()
                                    .flex()
                                    .flex_wrap()
                                    .gap(px(4.))
                                    .child(view.button(
                                        format!("display-app-{number}"),
                                        picked("displays-role-app", app_selected),
                                        panel.editable,
                                        false,
                                        Intent::SelectPresenter(monitor.id.clone()),
                                        p,
                                    ))
                                    .child(view.button(
                                        pdf_id,
                                        picked(pdf_label, selected),
                                        panel.editable,
                                        false,
                                        Intent::SelectPdf(target),
                                        p,
                                    )),
                            ),
                    );
                if monitor.identity_confidence == ferrostage_platform::IdentityConfidence::Degraded
                {
                    row = row.child(
                        div()
                            .id(gpui::SharedString::from(format!(
                                "display-degraded-{number}"
                            )))
                            .role(gpui::Role::Alert)
                            .text_size(px(view.ui_text_size(12.)))
                            .text_color(rgb(p.warning))
                            .child(l.text("displays-id-degraded")),
                    );
                }
                if details {
                    row = row.child(
                        div()
                            .id(gpui::SharedString::from(format!(
                                "display-diagnostics-{number}"
                            )))
                            .debug_selector(move || format!("display-diagnostics-{number}"))
                            .text_size(px(view.ui_text_size(12.)))
                            .text_color(rgb(p.muted))
                            .whitespace_normal()
                            .child(l.text(match monitor.identity_source {
                                ferrostage_platform::IdentitySource::DeviceInterfacePath => {
                                    "displays-id-interface"
                                }
                                ferrostage_platform::IdentitySource::NameAndResolution => {
                                    "displays-id-name"
                                }
                            }))
                            .child(l.format(
                                "displays-scale-source",
                                &[("ratio", format!("{:.2}", monitor.scale_factor))],
                            )),
                    );
                }
                rows = rows.child(row);
            }
            content = content.child(
                div()
                    .flex()
                    .gap(px(16.))
                    .min_w_0()
                    .when(!side_by_side, |v| v.flex_col())
                    .child(overview)
                    .child(rows),
            );
        }
        if let Some(problem) = &panel.problem {
            content = content.child(
                div()
                    .id("display-validation")
                    .debug_selector(|| "display-validation".into())
                    .role(gpui::Role::Alert)
                    .text_color(rgb(p.warning))
                    .child(problem.clone()),
            );
        }
        if let Some(error) = &panel.operation_error {
            content = content.child(
                div()
                    .id("display-operation-error")
                    .role(gpui::Role::Alert)
                    .text_color(rgb(p.error))
                    .child(error.clone()),
            );
        }
        content
    }
    fn actions(view: &Presenter, panel: &DisplayPanel, l: Locale, p: Palette) -> gpui::Div {
        let none_label = if panel.draft.pdf == PdfTarget::None {
            l.format(
                "displays-role-picked",
                &[("label", l.text("displays-no-play").into())],
            )
        } else {
            l.text("displays-no-play").into()
        };
        div()
            .flex()
            .flex_col()
            .flex_shrink_0()
            .gap(px(4.))
            .border_t_1()
            .border_color(rgb(p.border))
            .pt(px(8.))
            .child(
                div()
                    .text_size(px(view.ui_text_size(12.)))
                    .text_color(rgb(if panel.roles.pending {
                        p.warning
                    } else {
                        p.muted
                    }))
                    .whitespace_normal()
                    .child(l.text(if panel.roles.pending {
                        "displays-pending-help"
                    } else {
                        "displays-draft-help"
                    })),
            )
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .gap(px(8.))
                    .child(view.button(
                        "display-pdf-none".into(),
                        none_label,
                        panel.editable,
                        false,
                        Intent::SelectPdf(PdfTarget::None),
                        p,
                    ))
                    .child(view.button(
                        "apply-displays".into(),
                        l.text("displays-apply").into(),
                        panel.editable && panel.problem.is_none(),
                        true,
                        Intent::ApplyDisplays,
                        p,
                    ))
                    .child(view.button(
                        "cancel-displays".into(),
                        l.text("displays-cancel").into(),
                        !panel.busy,
                        false,
                        Intent::CancelDisplayDraft,
                        p,
                    )),
            )
    }
    pub(crate) fn panel(
        view: &Presenter,
        panel: &DisplayPanel,
        l: Locale,
        p: Palette,
        embedded: bool,
        width: f32,
    ) -> gpui::AnyElement {
        let body = body(view, panel, l, p, width);
        if embedded {
            div()
                .flex()
                .flex_col()
                .gap(px(12.))
                .child(body)
                .child(actions(view, panel, l, p))
                .into_any_element()
        } else {
            div()
                .size_full()
                .p(px(16.))
                .flex()
                .flex_col()
                .gap(px(12.))
                .child(
                    div()
                        .text_size(px(view.ui_text_size(20.)))
                        .child(l.text("displays")),
                )
                .child(
                    div()
                        .id("display-panel-content")
                        .debug_selector(|| "display-panel-content".into())
                        .flex_1()
                        .min_h_0()
                        .overflow_y_scroll()
                        .child(body),
                )
                .child(actions(view, panel, l, p))
                .into_any_element()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferrostage_platform::{IdentityConfidence, IdentitySource, ScaleSource};

    #[test]
    fn native_observation_requires_current_unique_mapping_and_running_owner() {
        use std::collections::BTreeMap;
        let screens = vec![monitor("a"), monitor("b")];
        let mut mapping = BTreeMap::from([(MonitorId("a".into()), 4), (MonitorId("b".into()), 8)]);
        assert_eq!(
            observed_location(true, Some(8), true, &screens, Some(&mapping)),
            WindowLocation::Monitor(MonitorId("b".into()))
        );
        assert_eq!(
            observed_location(false, Some(8), true, &screens, Some(&mapping)),
            WindowLocation::NotRunning
        );
        for (display, valid, mapping) in [
            (None, true, Some(&mapping)),
            (Some(8), false, Some(&mapping)),
            (Some(99), true, Some(&mapping)),
            (Some(8), true, None),
        ] {
            assert_eq!(
                observed_location(true, display, valid, &screens, mapping),
                WindowLocation::Unknown
            );
        }
        mapping.insert(MonitorId("a".into()), 8);
        assert_eq!(
            observed_location(true, Some(8), true, &screens, Some(&mapping)),
            WindowLocation::Unknown
        );
        let duplicate = vec![monitor("b"), monitor("b")];
        mapping.remove(&MonitorId("a".into()));
        assert_eq!(
            observed_location(true, Some(8), true, &duplicate, Some(&mapping)),
            WindowLocation::Unknown
        );
    }

    #[test]
    fn projection_keeps_actual_configured_and_pending_roles_separate() {
        let a = MonitorId("a".into());
        let b = MonitorId("b".into());
        let c = MonitorId("c".into());
        let configured = DisplayDraft {
            revision: 4,
            presenter: Some(a.clone()),
            pdf: PdfTarget::Display(b.clone()),
        };
        let mut draft = configured.clone();
        draft.pdf = PdfTarget::Display(c.clone());
        let projection = RoleProjection::new(
            WindowLocation::Monitor(a.clone()),
            WindowLocation::NotRunning,
            configured.clone(),
            &draft,
        );
        assert!(projection.pending);
        assert_eq!(
            projection.markers(&a, &draft),
            [("displays-actual-app", true)]
        );
        assert_eq!(
            projection.markers(&b, &draft),
            [("displays-configured-pdf-idle", false)]
        );
        assert_eq!(
            projection.markers(&c, &draft),
            [("displays-pending-pdf", false)]
        );
        // An OS drag changes observation, never the configured or pending allocation.
        let moved = RoleProjection::new(
            WindowLocation::Monitor(c.clone()),
            WindowLocation::Monitor(b.clone()),
            configured.clone(),
            &draft,
        );
        assert_eq!(
            moved.markers(&a, &draft),
            [("displays-configured-app", false)]
        );
        assert_eq!(moved.markers(&b, &draft), [("displays-actual-pdf", true)]);
        assert_eq!(
            moved.markers(&c, &draft),
            [
                ("displays-actual-app", true),
                ("displays-pending-pdf", false)
            ]
        );
        let cancelled = RoleProjection::new(moved.app, moved.pdf, configured.clone(), &configured);
        assert!(!cancelled.pending);
        assert_eq!(
            cancelled.markers(&c, &configured),
            [("displays-actual-app", true)]
        );
    }

    #[test]
    fn single_screen_windowed_pdf_can_share_actual_app_without_inventing_an_audience() {
        let a = MonitorId("a".into());
        let configured = DisplayDraft {
            revision: 0,
            presenter: Some(a.clone()),
            pdf: PdfTarget::Windowed,
        };
        let ready = RoleProjection::new(
            WindowLocation::Monitor(a.clone()),
            WindowLocation::NotRunning,
            configured.clone(),
            &configured,
        );
        assert_eq!(
            ready.markers(&a, &configured),
            [
                ("displays-actual-app", true),
                ("displays-configured-pdf-idle", false)
            ]
        );
        let playing = RoleProjection::new(
            ready.app,
            WindowLocation::Monitor(a.clone()),
            configured.clone(),
            &configured,
        );
        assert_eq!(
            playing.markers(&a, &configured),
            [("displays-actual-app", true), ("displays-actual-pdf", true)]
        );
        assert_eq!(
            playing.markers(&MonitorId("unused".into()), &configured),
            [("displays-unused", false)]
        );
    }

    fn monitor(id: &str) -> MonitorInfo {
        MonitorInfo {
            id: MonitorId(id.into()),
            name: format!("Screen {id}"),
            connection_name: id.into(),
            native_size: (1920, 1080),
            position: (0, 0),
            scale_factor: 1.,
            scale_source: ScaleSource::GdiDeviceCapsRatio,
            rotation_degrees: 0,
            is_primary: id == "a",
            identity_source: IdentitySource::DeviceInterfacePath,
            identity_confidence: IdentityConfidence::ConnectionScoped,
            built_in: false,
        }
    }
    #[test]
    fn topology_diagram_preserves_negative_coordinates_and_physical_aspect() {
        let mut left = monitor("a");
        left.position = (-1920, 0);
        let mut upper = monitor("b");
        upper.position = (0, -1080);
        upper.native_size = (1080, 1920);
        let aspect = topology_aspect(&[left.clone(), upper.clone()]);
        let rectangles = topology_rectangles(&[upper, left]);
        assert_eq!(rectangles[0].1, 0.);
        assert_eq!(rectangles[1].2, 0.);
        assert!((rectangles[0].3 / rectangles[0].4 * aspect - 1920. / 1080.).abs() < 0.001);
        assert!((rectangles[1].3 / rectangles[1].4 * aspect - 1080. / 1920.).abs() < 0.001);
    }
    #[test]
    fn numbering_is_shared_stable_and_not_serialized() {
        let screens = vec![monitor("c"), monitor("a"), monitor("b")];
        assert_eq!(
            numbered(&screens)
                .iter()
                .map(|(n, m)| (*n, m.id.0.as_str()))
                .collect::<Vec<_>>(),
            [(1, "a"), (2, "b"), (3, "c")]
        );
        let draft = DisplayDraft {
            revision: 7,
            presenter: Some(MonitorId("b".into())),
            pdf: PdfTarget::Display(MonitorId("c".into())),
        };
        let next = draft
            .settings(7, &screens, &DisplaySettings::default())
            .unwrap();
        assert_eq!(policy::roles(&screens, &next).presenter, draft.presenter);
        assert_eq!(
            policy::roles(&screens, &next).audience,
            Some(MonitorId("c".into()))
        );
        assert_eq!(next.monitors[0].role, MonitorRole::None);
        assert!(!toml::to_string(&next).unwrap().contains("revision"));
    }
    #[test]
    fn stale_missing_ambiguous_and_conflicting_destinations_are_rejected() {
        let screens = vec![monitor("a"), monitor("b")];
        let mut draft = DisplayDraft::new(2, &screens, &DisplaySettings::default());
        assert_eq!(draft.validation_error(3, &screens), Some("displays-stale"));
        draft.pdf = PdfTarget::Display(MonitorId("a".into()));
        assert_eq!(
            draft.validation_error(2, &screens),
            Some("displays-conflict")
        );
        draft.pdf = PdfTarget::Display(MonitorId("gone".into()));
        assert_eq!(
            draft.validation_error(2, &screens),
            Some("displays-target-missing")
        );
        draft.pdf = PdfTarget::Windowed;
        assert_eq!(
            draft.validation_error(2, &screens),
            Some("displays-windowed-single")
        );
        assert_eq!(
            draft.validation_error(2, &[monitor("a"), monitor("a")]),
            Some("displays-target-missing")
        );
    }
    #[test]
    fn single_screen_choices_roundtrip_without_duplicate_role_or_inherited_mode() {
        let screens = vec![monitor("a")];
        let mut draft = DisplayDraft::new(1, &screens, &DisplaySettings::default());
        draft.pdf = PdfTarget::Windowed;
        let windowed = draft
            .settings(1, &screens, &DisplaySettings::default())
            .unwrap();
        assert_eq!(windowed.monitors.len(), 1);
        assert_eq!(
            policy::audience_target(&screens, &windowed).unwrap(),
            (MonitorId("a".into()), false)
        );
        draft.pdf = PdfTarget::None;
        let none = draft.settings(1, &screens, &windowed).unwrap();
        let settings = ferrostage_settings::Settings {
            display: none,
            ..Default::default()
        };
        let roundtrip =
            ferrostage_settings::from_toml(&toml::to_string(&settings).unwrap()).unwrap();
        assert!(policy::audience_target(&screens, &roundtrip.settings.display).is_err());
        assert_eq!(
            roundtrip.settings.display.single_monitor_mode,
            SingleMonitorMode::PresenterOnly
        );
    }
    fn placed(
        id: &str,
        name: &str,
        built_in: bool,
        size: (u32, u32),
        position: (i32, i32),
        rotation: u16,
        primary: bool,
    ) -> MonitorInfo {
        let mut monitor = monitor(id);
        monitor.name = name.into();
        monitor.built_in = built_in;
        monitor.native_size = size;
        monitor.position = position;
        monitor.rotation_degrees = rotation;
        monitor.is_primary = primary;
        monitor
    }
    #[test]
    fn identical_driver_names_become_distinct_cards() {
        let screens = vec![
            placed("bkx", "BKX270", false, (2560, 1440), (0, 0), 0, true),
            placed(
                "boe",
                "NE160QDM-NM9",
                true,
                (2560, 1600),
                (-2560, -169),
                0,
                false,
            ),
            placed(
                "rtk",
                "Generic PnP Monitor",
                false,
                (1600, 2560),
                (2560, -1142),
                270,
                false,
            ),
        ];
        let zh = crate::locale::Locale::new("zh-CN");
        let copies = screens
            .iter()
            .map(|monitor| screen_copy(zh, &screens, monitor).identify)
            .collect::<Vec<_>>();
        assert_eq!(copies.len(), 3);
        assert!(
            copies
                .iter()
                .all(|copy| copies.iter().filter(|other| *other == copy).count() == 1)
        );
        assert_eq!(copies[0], "BKX270 · 2560×1440 · 主屏");
        assert_eq!(copies[1], "内置屏 NE160QDM-NM9 · 2560×1600 · 左侧");
        assert_eq!(copies[2], "竖屏 · 1600×2560 · 右侧");
        let en = crate::locale::Locale::new("en-US");
        let english = screens
            .iter()
            .map(|monitor| screen_copy(en, &screens, monitor).identify)
            .collect::<Vec<_>>();
        assert_eq!(english[0], "BKX270 · 2560×1440 · Primary");
        assert_eq!(english[1], "Built-in NE160QDM-NM9 · 2560×1600 · Left");
        assert_eq!(english[2], "Portrait · 1600×2560 · Right");
        let twins = vec![
            placed(
                "a",
                "Generic PnP Monitor",
                false,
                (1920, 1080),
                (0, 0),
                0,
                true,
            ),
            placed(
                "b",
                "Generic PnP Monitor",
                false,
                (1920, 1080),
                (1920, 0),
                0,
                false,
            ),
        ];
        let left = screen_copy(zh, &twins, &twins[0]).identify;
        let right = screen_copy(zh, &twins, &twins[1]).identify;
        assert_ne!(left, right);
        assert!(left.contains("主屏"));
        assert!(right.contains("右侧"));
        assert!(!left.contains("Generic"));
        assert!(!right.contains("Generic"));
    }
}
