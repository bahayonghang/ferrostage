//! Fullscreen is part of Start's placement transaction, not a saved preference.
use super::*;

pub(super) struct Restore {
    window: WindowId,
    monitor: MonitorId,
    display: u64,
    bounds: Bounds<Pixels>,
    fullscreen: bool,
    maximized: bool,
    deadline: Option<Instant>,
    exit_requested: bool,
}

impl Shell {
    pub(super) fn restoring_presenter(&self) -> bool {
        self.presenter_restore
            .as_ref()
            .is_some_and(|r| r.deadline.is_some())
    }

    pub(super) fn enter_presenter_fullscreen(
        &mut self,
        cx: &mut Context<Self>,
    ) -> Result<(), String> {
        let presenter = self
            .presenter
            .as_mut()
            .ok_or_else(|| self.locale.text("confirming").to_owned())?;
        let mapping = self.mapped.as_ref().map_err(|e| e.localized(self.locale))?;
        let restore = presenter
            .handle
            .update(cx, |_, window, cx| {
                let display = window.display(cx).map(|d| u64::from(d.id()));
                // A manually moved private window cannot authorize entering the PDF display.
                if display != mapping.get(&presenter.monitor).copied() {
                    return None;
                }
                let restore = Restore {
                    window: presenter.handle.window_id(),
                    monitor: presenter.monitor.clone(),
                    display: presenter.display,
                    bounds: window.bounds(),
                    fullscreen: window.is_fullscreen(),
                    maximized: window.is_maximized(),
                    deadline: None,
                    exit_requested: false,
                };
                if !restore.fullscreen {
                    presenter.bounds_observed.set(false);
                    window.toggle_fullscreen();
                }
                Some(restore)
            })
            .map_err(|e| e.to_string())?
            .ok_or_else(|| ShellError::PlacementTimeout.localized(self.locale))?;
        presenter.fullscreen = true;
        self.presenter_restore = Some(restore);
        Ok(())
    }

    pub(super) fn start_presenter_confirmed(&self, cx: &mut App) -> bool {
        // Single-screen Start does not toggle the presenter; its user-chosen
        // bounds (for example maximized) are not part of the placement.
        self.presenter
            .as_ref()
            .is_some_and(|p| self.presenter_restore.is_none() || Self::confirmed(p, cx))
    }

    pub(super) fn restore_presenter(&mut self, cx: &mut Context<Self>) {
        let Some(restore) = self.presenter_restore.as_mut() else {
            return;
        };
        if restore.deadline.is_some() {
            return;
        }
        let Some(presenter) = self
            .presenter
            .as_mut()
            .filter(|p| p.handle.window_id() == restore.window)
        else {
            // A confirmed role migration retired the original window. Its native
            // bounds must never be applied to the newly assigned presenter.
            self.presenter_restore = None;
            return;
        };
        let safe = self.topology_valid
            && self
                .mapped
                .as_ref()
                .is_ok_and(|m| m.get(&restore.monitor) == Some(&restore.display));
        if !safe {
            self.presenter_restore = None;
            self.recover_windowed = true;
            return;
        }
        restore.deadline = Some(Instant::now() + CONFIRM_TIMEOUT);
        presenter.fullscreen = restore.fullscreen;
        // GPUI's native fullscreen exit restores the OS placement, including
        // maximized state and the normal rectangle. Do not recreate the view.
        self.poll_presenter_restore(cx);
    }

    pub(super) fn poll_presenter_restore(&mut self, cx: &mut Context<Self>) {
        let Some(restore) = self
            .presenter_restore
            .as_mut()
            .filter(|r| r.deadline.is_some())
        else {
            return;
        };
        let Some(presenter) = self
            .presenter
            .as_ref()
            .filter(|p| p.handle.window_id() == restore.window)
        else {
            self.presenter_restore = None;
            return;
        };
        let confirmed = presenter
            .handle
            .update(cx, |_, window, cx| {
                // Windows queues the toggle. Stop can arrive before entry has
                // executed: wait to observe entry, then request exit exactly
                // once, rather than accepting the pre-entry windowed flag.
                if !restore.fullscreen && window.is_fullscreen() && !restore.exit_requested {
                    restore.exit_requested = true;
                    window.toggle_fullscreen();
                }
                (restore.fullscreen || restore.exit_requested)
                    && window.is_fullscreen() == restore.fullscreen
                    && window.is_maximized() == restore.maximized
                    && window
                        .display(cx)
                        .is_some_and(|d| u64::from(d.id()) == restore.display)
                    && (restore.fullscreen
                        || restore.maximized
                        || rect(window.bounds())
                            .iter()
                            .zip(rect(restore.bounds))
                            .all(|(a, b)| (a - b).abs() <= 2.))
            })
            .unwrap_or(false);
        if confirmed
            || restore
                .deadline
                .is_some_and(|deadline| Instant::now() > deadline)
        {
            Self::log_placement(
                presenter,
                if confirmed {
                    "presenter restored"
                } else {
                    "presenter restore timeout"
                },
                cx,
            );
            if !confirmed {
                self.error = Some(ShellError::PlacementTimeout.localized(self.locale));
                self.recover_windowed = true;
            }
            self.presenter_restore = None;
            if !confirmed {
                if let Some(presenter) = self.presenter.as_mut() {
                    presenter.fullscreen = presenter
                        .handle
                        .update(cx, |_, window, _| window.is_fullscreen())
                        .unwrap_or(true);
                }
                self.recover_presenter(cx);
            }
            cx.notify();
        }
    }
}
