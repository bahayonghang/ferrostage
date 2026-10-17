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

#[cfg(feature = "gpui")]
#[derive(Clone)]
pub(crate) struct DisplayPanel {
    pub monitors: Vec<MonitorInfo>,
    pub effective: DisplaySettings,
    pub draft: DisplayDraft,
    pub problem: Option<String>,
    pub operation_error: Option<String>,
    pub topology_valid: bool,
    pub editable: bool,
    pub can_identify: bool,
}

#[cfg(feature = "gpui")]
pub(crate) mod view {
    use super::*;
    use crate::{
        actions::Intent,
        locale::Locale,
        presenter::Presenter,
        theme::{self, Palette},
    };
    use gpui::{IntoElement, div, prelude::*, px, rgb};

    pub(crate) fn panel(
        view: &Presenter,
        panel: &DisplayPanel,
        l: Locale,
        p: Palette,
        detailed: bool,
    ) -> impl IntoElement {
        let roles = policy::roles(&panel.monitors, &panel.effective);
        let numbered = numbered(&panel.monitors);
        let role_label = |text: String, active: bool| {
            if active {
                l.format("displays-role-picked", &[("label", text)])
            } else {
                text
            }
        };
        let mut content = div()
            .id("display-panel-content")
            .flex_1()
            .min_h_0()
            .overflow_y_scroll()
            .flex()
            .flex_col()
            .gap(px(12.));
        content = content
            .child(
                div()
                    .text_size(px(20.))
                    .text_color(rgb(p.ink))
                    .child(l.text("displays")),
            )
            .child(div().child(if panel.topology_valid {
                l.format(
                    "displays-count",
                    &[("count", panel.monitors.len().to_string())],
                )
            } else {
                l.text("displays-unavailable").into()
            }))
            .child(
                div()
                    .text_size(px(14.))
                    .text_color(rgb(p.muted))
                    .child(l.text("displays-number-help")),
            );
        if panel.topology_valid && detailed {
            let source = match roles.source {
                ferrostage_platform::ResolutionSource::NoMonitors => "displays-source-none",
                ferrostage_platform::ResolutionSource::SingleMonitor => "displays-source-single",
                ferrostage_platform::ResolutionSource::Heuristic => "displays-source-heuristic",
                ferrostage_platform::ResolutionSource::Saved => "displays-source-saved",
            };
            content = content.child(div().text_size(px(14.)).child(l.text(source)));
            for (index, warning) in roles.warnings.iter().enumerate() {
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
            let mut diagram = div()
                .id("display-topology")
                .w_full()
                .aspect_ratio(topology_aspect(&panel.monitors))
                .flex_shrink_0()
                .relative()
                .bg(rgb(p.card))
                .rounded(px(theme::RADIUS));
            for (number, x, y, w, h) in topology_rectangles(&panel.monitors) {
                diagram = diagram.child(
                    div()
                        .absolute()
                        .left(gpui::relative(x))
                        .top(gpui::relative(y))
                        .w(gpui::relative(w))
                        .h(gpui::relative(h))
                        .border_1()
                        .border_color(rgb(p.border))
                        .bg(rgb(p.soft))
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(number.to_string()),
                );
            }
            content = content.child(diagram);
        }
        content = content
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap(px(8.))
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
                        true,
                        false,
                        Intent::RefreshDisplays,
                        p,
                    )),
            )
            .child(
                div()
                    .text_size(px(14.))
                    .text_color(rgb(p.muted))
                    .whitespace_normal()
                    .child(l.text("displays-identify-help")),
            );
        if panel.topology_valid {
            let none = panel.draft.pdf == PdfTarget::None;
            content = content.child(view.button(
                "display-pdf-none".into(),
                role_label(l.text("displays-no-play").into(), none),
                panel.editable,
                none,
                Intent::SelectPdf(PdfTarget::None),
                p,
            ));
            for (number, monitor) in &numbered {
                let copy = screen_copy(l, &panel.monitors, monitor);
                let presenter_here = panel.draft.presenter.as_ref() == Some(&monitor.id);
                let (pdf_id, pdf_label, pdf_active, pdf_intent) = if numbered.len() == 1 {
                    (
                        "display-pdf-windowed".to_owned(),
                        l.text("displays-role-windowed").to_owned(),
                        panel.draft.pdf == PdfTarget::Windowed,
                        Intent::SelectPdf(PdfTarget::Windowed),
                    )
                } else {
                    let target = PdfTarget::Display(monitor.id.clone());
                    let active = panel.draft.pdf == target;
                    (
                        format!("display-pdf-{number}"),
                        l.text("displays-role-pdf").to_owned(),
                        active,
                        Intent::SelectPdf(target),
                    )
                };
                let identity = if monitor.identity_confidence
                    == ferrostage_platform::IdentityConfidence::Degraded
                {
                    "displays-id-degraded"
                } else {
                    "displays-id-connection"
                };
                content = content.child(
                    div()
                        .id(gpui::SharedString::from(format!("display-card-{number}")))
                        .w_full()
                        .min_w_0()
                        .p(px(12.))
                        .rounded(px(theme::RADIUS))
                        .bg(rgb(p.soft))
                        .border_1()
                        .border_color(rgb(if presenter_here || pdf_active {
                            p.focus
                        } else {
                            p.border
                        }))
                        .flex()
                        .flex_col()
                        .gap(px(8.))
                        .child(
                            div().flex().gap(px(12.)).min_w_0().child(
                                div().flex().flex_1().min_w_0().gap(px(12.)).child(
                                    div()
                                        .id(gpui::SharedString::from(format!(
                                            "display-index-{number}"
                                        )))
                                        .flex_shrink_0()
                                        .w(px(theme::CONTROL))
                                        .h(px(theme::CONTROL))
                                        .rounded(px(theme::RADIUS))
                                        .border_1()
                                        .border_color(rgb(p.border))
                                        .bg(rgb(p.card))
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .text_size(px(20.))
                                        .text_color(rgb(p.ink))
                                        .child(number.to_string()),
                                ).child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .flex_1()
                                        .min_w_0()
                                        .justify_center()
                                        .gap(px(2.))
                                        .child(
                                            div()
                                                .text_size(px(16.))
                                                .text_color(rgb(p.ink))
                                                .whitespace_normal()
                                                .child(copy.title.clone()),
                                        )
                                        .when(!copy.meta.is_empty(), |column| {
                                            column.child(
                                                div()
                                                    .text_size(px(14.))
                                                    .text_color(rgb(p.muted))
                                                    .whitespace_normal()
                                                    .child(copy.meta.clone()),
                                            )
                                        }),
                                ),
                            ),
                        )
                        .when(detailed, |card| {
                            card.child(
                                div()
                                    .text_size(px(14.))
                                    .text_color(rgb(p.muted))
                                    .whitespace_normal()
                                    .child(l.text(match monitor.identity_source {
                                        ferrostage_platform::IdentitySource::DeviceInterfacePath => {
                                            "displays-id-interface"
                                        }
                                        ferrostage_platform::IdentitySource::NameAndResolution => {
                                            "displays-id-name"
                                        }
                                    })),
                            )
                            .child(
                                div()
                                    .text_size(px(14.))
                                    .text_color(rgb(p.muted))
                                    .whitespace_normal()
                                    .child(format!(
                                        "{} · {}",
                                        l.text(identity),
                                        l.format(
                                            "displays-scale-source",
                                            &[("ratio", format!("{:.2}", monitor.scale_factor))]
                                        )
                                    )),
                            )
                        })
                        .child(
                            div().flex().flex_wrap().gap(px(8.)).child(view.button(
                                format!("display-app-{number}"),
                                role_label(l.text("displays-role-app").into(), presenter_here),
                                panel.editable,
                                presenter_here,
                                Intent::SelectPresenter(monitor.id.clone()),
                                p,
                            )).child(view.button(
                                pdf_id,
                                role_label(pdf_label, pdf_active),
                                panel.editable,
                                pdf_active,
                                pdf_intent,
                                p,
                            )),
                        ),
                );
            }
            content = content.child(
                div()
                    .text_size(px(14.))
                    .text_color(rgb(p.muted))
                    .whitespace_normal()
                    .child(l.text("displays-draft-help")),
            );
        }
        if let Some(problem) = &panel.problem {
            content = content.child(
                div()
                    .id("display-validation")
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
        div()
            .size_full()
            .p(px(16.))
            .flex()
            .flex_col()
            .gap(px(12.))
            .child(content)
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap(px(8.))
                    .border_t_1()
                    .border_color(rgb(p.border))
                    .pt(px(8.))
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
                        true,
                        false,
                        Intent::Escape,
                        p,
                    )),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferrostage_platform::{IdentityConfidence, IdentitySource, ScaleSource};

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
