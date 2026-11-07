use super::*;
use crate::display_settings::WindowLocation;
use ferrostage_core::PlaybackIdentity;

pub(crate) struct ProjectionStatus {
    pub summary: String,
    pub location: String,
    pub restore: bool,
    pub failed: bool,
}

impl Shell {
    fn projection_page(&self, identity: PlaybackIdentity) -> String {
        let map = self.state.slide_map();
        let logical = map.and_then(|m| m.logical_for(identity.page));
        let group = logical.and_then(|logical| map?.groups().get(logical.0));
        let mut label = self.locale.format(
            "wb-page-compact",
            &[
                (
                    "slide",
                    logical.map_or(identity.page.0 + 1, |n| n.0 + 1).to_string(),
                ),
                (
                    "slides",
                    map.map_or(self.state.page_count(), |m| m.groups().len())
                        .to_string(),
                ),
            ],
        );
        if let Some(group) = group.filter(|group| group.len() > 1) {
            label.push_str(
                &self.locale.format(
                    "wb-step-compact",
                    &[
                        (
                            "step",
                            (group
                                .iter()
                                .position(|page| *page == identity.page)
                                .unwrap_or(0)
                                + 1)
                            .to_string(),
                        ),
                        ("steps", group.len().to_string()),
                    ],
                ),
            );
        }
        label
    }

    fn projection_monitor(&self, id: &MonitorId) -> String {
        display_settings::numbered(&self.monitors)
            .into_iter()
            .find(|(_, monitor)| &monitor.id == id)
            .map(|(number, monitor)| {
                format!(
                    "{} {}",
                    self.locale
                        .format("displays-screen-number", &[("number", number.to_string())]),
                    display_settings::screen_copy(self.locale, &self.monitors, monitor).title
                )
            })
            .unwrap_or_else(|| self.locale.text("displays-location-unknown").into())
    }

    pub(crate) fn projection_status(&self) -> Option<ProjectionStatus> {
        if self.state.stage() == SessionStage::Idle {
            return None;
        }
        let l = self.locale;
        if self.state.stage() != SessionStage::Presenting {
            let preparing = self
                .pending
                .as_ref()
                .is_some_and(|pending| matches!(pending.kind, TransactionKind::Start(_)));
            let target = policy::start_target(&self.monitors, &self.settings.settings.display)
                .map(|target| {
                    let monitor = self.projection_monitor(&target.monitor);
                    let mode = if target.shared {
                        l.text(if target.fullscreen {
                            "projection-shared-fullscreen"
                        } else {
                            "projection-shared-windowed"
                        })
                    } else {
                        l.text("projection-separate")
                    };
                    format!("{monitor} · {mode}")
                })
                .unwrap_or_else(|_| l.text("projection-target-unavailable").into());
            return Some(ProjectionStatus {
                summary: l
                    .text(if preparing {
                        "projection-preparing"
                    } else {
                        "projection-not-started"
                    })
                    .into(),
                location: l.format("projection-configured", &[("target", target)]),
                restore: false,
                failed: false,
            });
        }
        let settled = self.settled_playback();
        let selected = self.selected_playback();
        let failed = self.graphics_error.is_some()
            || self.reload.failed_worker
            || self
                .workbench
                .failed_surfaces()
                .contains(&crate::workbench::Surface::Audience);
        let mut parts = Vec::new();
        let blank = self.state.audience_snapshot().blank_mode;
        if let Some(blank) = blank {
            parts.push(
                l.text(match blank {
                    BlankMode::Black => "projection-black",
                    BlankMode::White => "projection-white",
                })
                .to_owned(),
            );
        }
        if failed {
            parts.push(l.text("projection-failed").into());
        }
        parts.push(settled.map_or_else(
            || l.text("projection-awaiting-draw").into(),
            |identity| {
                l.format(
                    "projection-drawn",
                    &[("page", self.projection_page(identity))],
                )
            },
        ));
        if let Some(identity) = selected.filter(|identity| Some(*identity) != settled) {
            parts.push(l.format(
                "projection-pending",
                &[("page", self.projection_page(identity))],
            ));
        }
        let actual = self.window_location(self.audience.as_ref(), true);
        let location = match &actual {
            WindowLocation::Monitor(id) => self.projection_monitor(id),
            _ => l.text("displays-location-unknown").into(),
        };
        let shared = matches!(actual, WindowLocation::Monitor(_))
            && actual == self.window_location(self.presenter.as_ref(), true);
        let location = if shared {
            let mode = if self.audience.as_ref().is_some_and(|a| a.fullscreen) {
                "projection-shared-fullscreen"
            } else {
                "projection-shared-windowed"
            };
            format!("{location} · {}", l.text(mode))
        } else {
            location
        };
        Some(ProjectionStatus {
            summary: parts.join(" · "),
            location: l.format("projection-location", &[("target", location)]),
            restore: blank.is_some(),
            failed,
        })
    }
}
