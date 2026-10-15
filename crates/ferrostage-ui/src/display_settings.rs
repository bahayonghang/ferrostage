//! Shared display-assignment draft. Numbers are view labels; only IDs become settings.
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
    ) -> impl IntoElement {
        let roles = policy::roles(&panel.monitors, &panel.effective);
        let windowed = panel.monitors.len() == 1
            && panel.effective.single_monitor_mode == SingleMonitorMode::WindowedAudience;
        let numbered = numbered(&panel.monitors);
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
        if panel.topology_valid {
            for (number, monitor) in &numbered {
                let usage = if roles.presenter.as_ref() == Some(&monitor.id) {
                    if windowed {
                        "displays-use-windowed"
                    } else {
                        "displays-app"
                    }
                } else if roles.audience.as_ref() == Some(&monitor.id) {
                    "displays-pdf"
                } else {
                    "displays-unused"
                };
                let label = format!("{number} · {}", monitor.name);
                let details = format!(
                    "{} × {}{} · {}",
                    monitor.native_size.0,
                    monitor.native_size.1,
                    if monitor.is_primary {
                        format!(" · {}", l.text("displays-primary"))
                    } else {
                        String::new()
                    },
                    l.text(usage)
                );
                let identity = if monitor.identity_confidence
                    == ferrostage_platform::IdentityConfidence::Degraded
                {
                    "displays-id-degraded"
                } else {
                    "displays-id-connection"
                };
                content =
                    content.child(
                        div()
                            .p(px(12.))
                            .rounded(px(theme::RADIUS))
                            .bg(rgb(p.soft))
                            .border_1()
                            .border_color(rgb(p.border))
                            .flex()
                            .flex_col()
                            .gap(px(4.))
                            .child(div().text_color(rgb(p.ink)).child(label))
                            .child(div().text_size(px(14.)).child(details))
                            .child(div().text_size(px(14.)).text_color(rgb(p.muted)).child(
                                format!(
                                    "{} · {}",
                                    l.text(identity),
                                    l.format(
                                        "displays-scale-source",
                                        &[("ratio", format!("{:.2}", monitor.scale_factor))]
                                    )
                                ),
                            )),
                    );
            }
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
                    .child(l.text("displays-identify-help")),
            );
        let selected = |label: String, is_selected: bool| {
            if is_selected {
                l.format("displays-selected", &[("label", label)])
            } else {
                label
            }
        };
        let mut apps = div().flex().flex_wrap().gap(px(8.));
        for (number, monitor) in &numbered {
            let active = panel.draft.presenter.as_ref() == Some(&monitor.id);
            apps = apps.child(view.button(
                format!("display-app-{number}"),
                selected(format!("{number} · {}", monitor.name), active),
                panel.editable,
                active,
                Intent::SelectPresenter(monitor.id.clone()),
                p,
            ));
        }
        let mut pdf = div().flex().flex_wrap().gap(px(8.)).child(view.button(
            "display-pdf-none".into(),
            selected(
                l.text("displays-no-play").into(),
                panel.draft.pdf == PdfTarget::None,
            ),
            panel.editable,
            panel.draft.pdf == PdfTarget::None,
            Intent::SelectPdf(PdfTarget::None),
            p,
        ));
        if numbered.len() == 1 {
            pdf = pdf.child(view.button(
                "display-pdf-windowed".into(),
                selected(
                    l.text("displays-windowed").into(),
                    panel.draft.pdf == PdfTarget::Windowed,
                ),
                panel.editable,
                panel.draft.pdf == PdfTarget::Windowed,
                Intent::SelectPdf(PdfTarget::Windowed),
                p,
            ));
        } else {
            for (number, monitor) in numbered {
                let target = PdfTarget::Display(monitor.id.clone());
                let active = panel.draft.pdf == target;
                pdf = pdf.child(view.button(
                    format!("display-pdf-{number}"),
                    selected(format!("{number} · {}", monitor.name), active),
                    panel.editable,
                    active,
                    Intent::SelectPdf(target),
                    p,
                ));
            }
        }
        content = content
            .child(div().text_color(rgb(p.ink)).child(l.text("displays-app")))
            .child(apps)
            .child(div().text_color(rgb(p.ink)).child(l.text("displays-pdf")))
            .child(pdf)
            .child(
                div()
                    .text_size(px(14.))
                    .text_color(rgb(p.muted))
                    .child(l.text("displays-draft-help")),
            );
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
        }
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
}
