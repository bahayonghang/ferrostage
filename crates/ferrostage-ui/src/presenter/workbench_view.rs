use super::*;
use crate::workbench::{ReadyFrame, Surface, Tool, WorkbenchLayout};
use ferrostage_core::{AudienceProjectionSnapshot, PresentationCommand as Command};
use gpui::{MouseButton, canvas};
use std::{cell::Cell, rc::Rc, time::Instant};

/// Visible presentation-timer state; the capsule shows it with a mark and label.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TimerPhase {
    /// Ready with nothing to resume.
    Fresh,
    Running,
    Paused,
    /// Ready after a stop that Resume can continue.
    Stopped,
}

pub(super) struct WorkbenchData {
    pub frames: Vec<(Surface, ReadyFrame)>,
    pub surfaces: Vec<Surface>,
    pub overlay: AudienceProjectionSnapshot,
    pub media: Vec<ferrostage_core::AudienceMediaFrame>,
    slots: Vec<crate::media_playback::SlotView>,
    pub(super) settings: ferrostage_settings::PresenterSettings,
    pointer: ferrostage_settings::PointerSettings,
    pub(super) tool: Tool,
    pub(super) grid_start: usize,
    pub steps: Vec<ferrostage_core::PhysicalPage>,
    pub progress: String,
    progress_compact: String,
    timer: String,
    timer_label: &'static str,
    phase: TimerPhase,
    /// Formatted frozen time Resume continues from, when the session can resume.
    pub(super) resumable: Option<String>,
    pub(super) pending_start: Option<crate::shell::StartTimer>,
    clock: Option<String>,
    warning: u8,
    paused: bool,
    timed: bool,
    pub(super) autoplay_paused: bool,
    pub budget: usize,
    debug: Option<String>,
    pub(super) writable: bool,
}
impl WorkbenchData {
    pub fn from_shell(shell: &Shell) -> Self {
        let mut data = Self::capture(shell);
        // Resume continues the frozen value the capsule already shows.
        data.resumable = (data.phase == TimerPhase::Stopped).then(|| data.timer.clone());
        data
    }
    fn capture(shell: &Shell) -> Self {
        let elapsed = shell.state.elapsed_at(Instant::now()).as_secs();
        let timer = &shell.settings.settings.timer;
        let target = u64::from(timer.target_minutes) * 60;
        let remaining = target.saturating_sub(elapsed);
        let seconds = if timer.mode == ferrostage_settings::TimerMode::CountUp {
            elapsed
        } else {
            target.abs_diff(elapsed)
        };
        let session = shell.workbench.document.as_ref();
        let page = shell.state.current_page();
        let (progress, progress_compact) = session
            .map(|s| {
                let map = &s.document.slide_map;
                let logical = map.logical_for(page).map_or(0, |l| l.0);
                let group = map.groups().get(logical);
                let step = group
                    .and_then(|g| g.iter().position(|p| *p == page))
                    .unwrap_or(0);
                let full = shell.locale.format(
                    "wb-progress",
                    &[
                        ("slide", (logical + 1).to_string()),
                        ("slides", map.groups().len().to_string()),
                        ("step", (step + 1).to_string()),
                        ("steps", group.map_or(1, Vec::len).to_string()),
                        ("page", (page.0 + 1).to_string()),
                        ("pages", s.document.page_count.to_string()),
                    ],
                );
                let mut compact = shell.locale.format(
                    "wb-page-compact",
                    &[
                        ("slide", (logical + 1).to_string()),
                        ("slides", map.groups().len().to_string()),
                    ],
                );
                if group.is_some_and(|group| group.len() > 1) {
                    compact.push_str(&shell.locale.format(
                        "wb-step-compact",
                        &[
                            ("step", (step + 1).to_string()),
                            ("steps", group.map_or(1, Vec::len).to_string()),
                        ],
                    ));
                }
                (full, compact)
            })
            .unwrap_or_default();
        let overlay = shell.state.audience_snapshot();
        Self {
            frames: shell
                .workbench
                .frames
                .iter()
                .filter(|(surface, _)| match surface {
                    Surface::Thumbnail(index) => {
                        shell.workbench.grid
                            && *index >= shell.workbench.grid_start
                            && *index
                                < shell.workbench.grid_start
                                    + shell.settings.settings.presenter.grid_visible_pages as usize
                    }
                    _ => true,
                })
                .cloned()
                .collect(),
            surfaces: shell
                .workbench
                .requested_surfaces()
                .into_iter()
                .filter(|surface| match surface {
                    Surface::Thumbnail(index) => {
                        shell.workbench.grid
                            && *index >= shell.workbench.grid_start
                            && *index
                                < shell.workbench.grid_start
                                    + shell.settings.settings.presenter.grid_visible_pages as usize
                    }
                    _ => true,
                })
                .collect(),
            overlay,
            media: shell
                .media
                .frames(&shell.state, session.map(|s| &s.document)),
            slots: shell
                .media
                .slots(&shell.state, session.map(|s| &s.document)),
            settings: shell.settings.settings.presenter.clone(),
            pointer: shell.settings.settings.pointer.clone(),
            tool: shell.workbench.tool,
            grid_start: shell.workbench.grid_start,
            steps: session
                .map(|s| s.document.slide_map.steps().to_vec())
                .unwrap_or_default(),
            progress,
            progress_compact,
            timer: format!(
                "{}{:02}:{:02}",
                if elapsed > target && timer.mode == ferrostage_settings::TimerMode::CountDown {
                    "−"
                } else {
                    ""
                },
                seconds / 60,
                seconds % 60
            ),
            timer_label: if timer.mode == ferrostage_settings::TimerMode::CountUp {
                "wb-elapsed"
            } else if elapsed > target {
                "wb-overtime"
            } else {
                "wb-remaining"
            },
            clock: shell
                .settings
                .settings
                .presenter
                .show_clock
                .then(|| chrono::Local::now().format("%H:%M:%S").to_string()),
            warning: if remaining <= u64::from(timer.warn_red_remaining_min) * 60 {
                2
            } else if remaining <= u64::from(timer.warn_yellow_remaining_min) * 60 {
                1
            } else {
                0
            },
            paused: shell.state.is_timer_paused(),
            phase: match shell.state.stage() {
                SessionStage::Presenting if shell.state.is_timer_paused() => TimerPhase::Paused,
                SessionStage::Presenting => TimerPhase::Running,
                _ if shell.resumable_elapsed().is_some() => TimerPhase::Stopped,
                _ => TimerPhase::Fresh,
            },
            resumable: None,
            pending_start: shell.pending_start(),
            timed: shell.has_authored_timing(),
            autoplay_paused: shell.autoplay_paused(),
            budget: shell.settings.settings.cache.l1_gpu_mib as usize * 1024 * 1024 / 2,
            debug: shell.settings.settings.general.debug_overlay.then(|| {
                format!(
                    "CPU frames {} · {} bytes · raster {:.2} ms",
                    shell.workbench.frames.len(),
                    shell.workbench.bytes(),
                    shell.workbench.render_ms
                )
            }),
            writable: shell.settings.write_policy == ferrostage_settings::WritePolicy::Writable,
        }
    }
}
impl Presenter {
    fn frame(&self, surface: Surface) -> Option<&ReadyFrame> {
        self.data
            .workbench
            .frames
            .iter()
            .find(|(s, _)| *s == surface)
            .map(|(_, f)| f)
    }
    pub(super) fn picture(&self, surface: Surface, interactive: bool) -> gpui::AnyElement {
        let frame = self
            .frame(surface)
            .map(|frame| frame.frame.clone())
            .or_else(|| self.textures.borrow().frame(surface));
        let Some(frame) = frame else {
            return div()
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .child(self.locale.text("wb-rendering"))
                .into_any_element();
        };
        let textures = self.textures.clone();
        let paint_ms = self.paint_ms.clone();
        let bounds = Rc::new(Cell::new(gpui::Bounds::default()));
        let paint_bounds = bounds.clone();
        let input_bounds = bounds.clone();
        let size_target = self.shell.clone();
        let overlay = interactive.then(|| self.data.workbench.overlay.clone());
        let media = if interactive {
            self.data.workbench.media.clone()
        } else {
            vec![]
        };
        let media_slots = self.data.workbench.slots.clone();
        let unavailable = self.locale.text("media-slot-unavailable");
        let placeholders = if interactive {
            media_slots
                .iter()
                .filter(|s| s.error.is_some())
                .map(|s| (s.rect, unavailable))
                .collect::<Vec<_>>()
        } else {
            vec![]
        };
        let target = self.shell.clone();
        let down_target = target.clone();
        let up_target = target.clone();
        let leave_target = target.clone();
        let down_bounds = bounds.clone();
        let move_bounds = bounds.clone();
        let tool = self.data.workbench.tool;
        let pointer = self.data.workbench.pointer.clone();
        let drawing = self.drawing.clone();
        let down_drawing = drawing.clone();
        let up_drawing = drawing.clone();
        let leave_drawing = drawing.clone();
        let width = frame.width();
        let height = frame.height();
        div()
            .id(SharedString::from(format!("surface-{surface:?}")))
            .debug_selector(|| format!("surface-{surface:?}"))
            .size_full()
            .when(!matches!(surface, Surface::Thumbnail(_)), |view| {
                view.min_h(px(70.))
            })
            .when_some(letterbox(surface), |view, fill| view.bg(rgb(fill)))
            .relative()
            .overflow_hidden()
            .child(
                canvas(
                    move |bounds, window, cx| {
                        if !matches!(surface, Surface::Thumbnail(_)) {
                            let scale = window.scale_factor();
                            let target = ferrostage_pdf::PixelSize {
                                width: (f32::from(bounds.size.width) * scale)
                                    .round()
                                    .clamp(1., 8192.) as u32,
                                height: (f32::from(bounds.size.height) * scale)
                                    .round()
                                    .clamp(1., 8192.)
                                    as u32,
                            };
                            actions::dispatch(
                                &size_target,
                                Intent::SurfaceSize(surface, target),
                                cx,
                            );
                        }
                        paint_bounds.set(crate::slide_surface::fit(bounds, width, height));
                    },
                    move |bounds, _, window, cx| {
                        let started = Instant::now();
                        let current = crate::slide_surface::composite(
                            &mut textures.borrow_mut(),
                            (surface, bounds),
                            overlay.as_ref(),
                            crate::slide_surface::MediaLayers {
                                frames: &media,
                                placeholders: &placeholders,
                            },
                            Surface::Media,
                            window,
                            cx,
                        );
                        if !current {
                            input_bounds.set(gpui::Bounds::default());
                        }
                        if surface == Surface::Current {
                            paint_ms.set(started.elapsed().as_secs_f64() * 1000.);
                        }
                    },
                )
                .size_full(),
            )
            .when(interactive, |view| {
                view.on_mouse_down(MouseButton::Left, move |event, _, cx| {
                    if let Some(point) =
                        crate::slide_surface::normalize(down_bounds.get(), event.position)
                    {
                        if tool == Tool::Navigate
                            && let Some(slot) = media_slots.iter().find(|s| {
                                point.0 >= s.rect.x
                                    && point.0 <= s.rect.x + s.rect.w
                                    && point.1 >= s.rect.y
                                    && point.1 <= s.rect.y + s.rect.h
                            })
                        {
                            actions::dispatch(&down_target, Intent::Media(slot.id), cx);
                            return;
                        }
                        down_drawing.set(true);
                        if matches!(tool, Tool::Pen | Tool::Highlighter) {
                            let mut color = pointer.pen_color_bgra;
                            if tool == Tool::Highlighter {
                                color[3] = (pointer.highlighter_alpha * 255.).round() as u8;
                            }
                            actions::dispatch(
                                &down_target,
                                Intent::Core(Command::BeginStroke {
                                    color,
                                    width: pointer.pen_width_norm as f32
                                        * if tool == Tool::Highlighter { 4. } else { 1. },
                                    is_highlighter: tool == Tool::Highlighter,
                                }),
                                cx,
                            );
                            actions::dispatch(
                                &down_target,
                                Intent::Core(Command::AppendStrokePoint(point)),
                                cx,
                            );
                        } else if tool == Tool::Eraser {
                            actions::dispatch(
                                &down_target,
                                Intent::Core(Command::EraseStrokeAt(point)),
                                cx,
                            );
                        }
                    }
                })
                .on_mouse_move(move |event, _, cx| {
                    let point = crate::slide_surface::normalize(move_bounds.get(), event.position);
                    if tool == Tool::Laser {
                        actions::dispatch(
                            &target,
                            Intent::Core(Command::UpdateLaserPointer(point)),
                            cx,
                        );
                    }
                    if drawing.get()
                        && event.pressed_button == Some(MouseButton::Left)
                        && let Some(point) = point
                    {
                        if matches!(tool, Tool::Pen | Tool::Highlighter) {
                            actions::dispatch(
                                &target,
                                Intent::Core(Command::AppendStrokePoint(point)),
                                cx,
                            );
                        } else if tool == Tool::Eraser {
                            actions::dispatch(
                                &target,
                                Intent::Core(Command::EraseStrokeAt(point)),
                                cx,
                            );
                        }
                    }
                })
                .on_mouse_up(MouseButton::Left, move |_, _, cx| {
                    up_drawing.set(false);
                    actions::dispatch(&up_target, Intent::Core(Command::EndStroke), cx);
                })
                .on_hover(move |hovered, _, cx| {
                    if !hovered {
                        leave_drawing.set(false);
                        actions::dispatch(
                            &leave_target,
                            Intent::Core(Command::UpdateLaserPointer(None)),
                            cx,
                        );
                        actions::dispatch(&leave_target, Intent::Core(Command::EndStroke), cx);
                    }
                })
            })
            .into_any_element()
    }
    #[cfg(all(test, feature = "test-support"))]
    pub(crate) fn timing_probe(&self) -> (&str, &'static str, bool) {
        let data = &self.data.workbench;
        (&data.timer, data.timer_label, data.paused)
    }
    pub(super) fn clock_readout(&self, p: Palette) -> Option<gpui::AnyElement> {
        self.data.workbench.clock.as_ref().map(|clock| {
            div()
                .id("presenter-clock")
                .debug_selector(|| "presenter-clock".into())
                .aria_label(self.locale.format("wb-clock", &[("time", clock.clone())]))
                .flex_shrink_0()
                .font(gpui::font(theme::MONO))
                .text_size(px(self.data.typography.size(14.)))
                .text_color(rgb(p.muted))
                .child(clock.clone())
                .into_any_element()
        })
    }
    /// Label key the capsule shows; state labels outrank threshold labels.
    fn capsule_label(data: &WorkbenchData) -> &'static str {
        let overtime = data.timer_label == "wb-overtime";
        match data.phase {
            TimerPhase::Paused => "wb-timer-paused",
            TimerPhase::Stopped => "wb-timer-stopped",
            TimerPhase::Running if !overtime && data.warning == 2 => "wb-timer-limit",
            TimerPhase::Running if !overtime && data.warning == 1 => "wb-timer-low",
            _ => data.timer_label,
        }
    }
    #[cfg(all(test, feature = "test-support"))]
    pub(crate) fn capsule_probe(
        &self,
    ) -> (
        TimerPhase,
        &'static str,
        Option<&str>,
        Option<crate::shell::StartTimer>,
    ) {
        let data = &self.data.workbench;
        (
            data.phase,
            Self::capsule_label(data),
            data.resumable.as_deref(),
            data.pending_start,
        )
    }
    /// 8px state mark: dot (running), ring (paused), square (stopped) or an
    /// empty reserved slot, so state never relies on colour alone.
    fn timer_mark(phase: TimerPhase, warning: u8, p: Palette) -> gpui::AnyElement {
        let mark = div().size(px(8.)).flex_shrink_0();
        match phase {
            TimerPhase::Running => mark
                .debug_selector(|| "timer-mark-running".into())
                .rounded_full()
                .bg(rgb(match warning {
                    2 => p.error,
                    1 => p.warning,
                    _ => p.focus,
                })),
            TimerPhase::Paused => mark
                .debug_selector(|| "timer-mark-paused".into())
                .rounded_full()
                .border(px(1.5))
                .border_color(rgb(p.muted)),
            TimerPhase::Stopped => mark
                .debug_selector(|| "timer-mark-stopped".into())
                .rounded(px(1.))
                .bg(rgb(p.muted)),
            TimerPhase::Fresh => mark.debug_selector(|| "timer-mark-fresh".into()),
        }
        .into_any_element()
    }
    pub(super) fn timer_panel(&self, p: Palette) -> gpui::AnyElement {
        let data = &self.data.workbench;
        let presenting = self.data.stage == SessionStage::Presenting;
        // A session that has not started cannot be running low.
        let warning = if data.phase == TimerPhase::Fresh {
            0
        } else {
            data.warning
        };
        let label = self.locale.text(Self::capsule_label(data));
        let mut accessible = format!("{label} {}", data.timer);
        if warning > 0 {
            accessible.push_str(". ");
            accessible.push_str(self.locale.text(if warning == 2 {
                "wb-timer-red"
            } else {
                "wb-timer-yellow"
            }));
        }
        div()
            .id("presenter-timer")
            .debug_selector(|| "presenter-timer".into())
            .aria_label(accessible)
            .flex_shrink_0()
            .min_h(px(theme::CONTROL))
            .pl(px(16.))
            .pr(px(if presenting { 4. } else { 16. }))
            .flex()
            .items_center()
            .gap(px(8.))
            .rounded(px(theme::PANEL_RADIUS))
            .bg(rgb(p.soft))
            .child(Self::timer_mark(data.phase, warning, p))
            .child(
                // Fixed slot: label changes never move the digits.
                div()
                    .id("presenter-timer-label")
                    .debug_selector(|| "presenter-timer-label".into())
                    .flex_shrink_0()
                    .min_w(px(self.ui_text_size(12.) * 6.5))
                    .flex()
                    .justify_end()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_size(px(self.ui_text_size(12.)))
                    .text_color(rgb(p.muted))
                    .child(label),
            )
            .child(
                div()
                    .id("presenter-timer-digits")
                    .debug_selector(|| "presenter-timer-digits".into())
                    // Seven monospace cells hold the longest signed value, −000:00.
                    .min_w(px(self.ui_text_size(30.) * 0.6 * 7.))
                    .flex_shrink_0()
                    .flex()
                    .justify_end()
                    .font(gpui::font(theme::MONO))
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_size(px(self.data.typography.size(30.)))
                    .text_color(rgb(match (warning, data.phase) {
                        (2, _) => p.error,
                        (1, _) => p.warning,
                        (_, TimerPhase::Running) => p.ink,
                        _ => p.body,
                    }))
                    .child(data.timer.clone()),
            )
            .when(presenting, |v| {
                v.child(div().w(px(1.)).h(px(20.)).flex_shrink_0().bg(rgb(p.border)))
                    .child(
                        self.button(
                            "timer-pause".into(),
                            self.locale
                                .text(if data.paused { "wb-resume" } else { "wb-pause" })
                                .into(),
                            true,
                            false,
                            Intent::Core(if data.paused {
                                Command::ResumeTimer
                            } else {
                                Command::PauseTimer
                            }),
                            p,
                        ),
                    )
                    .child(self.button(
                        "timer-reset".into(),
                        self.locale.text("wb-reset").into(),
                        true,
                        false,
                        Intent::Core(Command::ResetTimer),
                        p,
                    ))
            })
            .into_any_element()
    }
    pub(super) fn session_icon(stop: bool, ink: u32) -> gpui::AnyElement {
        canvas(
            |_, _, _| (),
            move |bounds, _, window, _| {
                let mut path = gpui::PathBuilder::fill();
                let points: &[(f32, f32)] = if stop {
                    &[(0.2, 0.2), (0.8, 0.2), (0.8, 0.8), (0.2, 0.8)]
                } else {
                    &[(0.24, 0.08), (0.90, 0.50), (0.24, 0.92)]
                };
                for (index, (x, y)) in points.iter().enumerate() {
                    let point = gpui::point(
                        bounds.origin.x + bounds.size.width * *x,
                        bounds.origin.y + bounds.size.height * *y,
                    );
                    if index == 0 {
                        path.move_to(point);
                    } else {
                        path.line_to(point);
                    }
                }
                path.close();
                if let Ok(path) = path.build() {
                    window.paint_path(path, rgb(ink));
                }
            },
        )
        .size(px(theme::SESSION_ICON))
        .flex_shrink_0()
        .into_any_element()
    }

    /// Drawn menu chevron; points up while its menu is open.
    pub(super) fn chevron(open: bool, ink: u32, selector: String) -> gpui::AnyElement {
        let icon = canvas(
            |_, _, _| (),
            move |bounds, _, window, _| {
                let (a, b) = if open { (0.68, 0.32) } else { (0.32, 0.68) };
                let points = [(0.15, a), (0.5, b), (0.85, a)];
                let mut path = gpui::PathBuilder::stroke(px(1.5));
                for (index, (x, y)) in points.iter().enumerate() {
                    let point = gpui::point(
                        bounds.origin.x + bounds.size.width * *x,
                        bounds.origin.y + bounds.size.height * *y,
                    );
                    if index == 0 {
                        path.move_to(point);
                    } else {
                        path.line_to(point);
                    }
                }
                if let Ok(path) = path.build() {
                    window.paint_path(path, rgb(ink));
                }
            },
        )
        .size(px(12.));
        div()
            .debug_selector(move || selector.clone())
            .size(px(12.))
            .flex_shrink_0()
            .child(icon)
            .into_any_element()
    }

    /// Drawn counter-clockwise arrow for Restart: a 270 degree polyline arc plus head.
    fn restart_icon(ink: u32) -> gpui::AnyElement {
        canvas(
            |_, _, _| (),
            move |bounds, _, window, _| {
                let at = |x: f32, y: f32| {
                    gpui::point(
                        bounds.origin.x + bounds.size.width * x,
                        bounds.origin.y + bounds.size.height * y,
                    )
                };
                let mut arc = gpui::PathBuilder::stroke(px(2.));
                // From the top, clockwise to the left side; the head points back.
                for step in 0..=18 {
                    let angle = -std::f32::consts::FRAC_PI_2
                        + std::f32::consts::PI * 1.5 * step as f32 / 18.;
                    let point = at(0.5 + 0.36 * angle.cos(), 0.5 + 0.36 * angle.sin());
                    if step == 0 {
                        arc.move_to(point);
                    } else {
                        arc.line_to(point);
                    }
                }
                if let Ok(path) = arc.build() {
                    window.paint_path(path, rgb(ink));
                }
                let mut head = gpui::PathBuilder::fill();
                head.move_to(at(0.5, 0.0));
                head.line_to(at(0.5, 0.28));
                head.line_to(at(0.28, 0.14));
                head.close();
                if let Ok(path) = head.build() {
                    window.paint_path(path, rgb(ink));
                }
            },
        )
        .size(px(20.))
        .flex_shrink_0()
        .into_any_element()
    }

    /// Ready after an interrupted session: joined Resume (primary) and Restart
    /// segments in place of the single Start button.
    fn session_split(&self, time: &str, p: Palette) -> gpui::AnyElement {
        use crate::shell::StartTimer;
        let l = self.locale;
        let enabled = self.data.start_reason.is_none() && !self.data.busy;
        let pending = self.data.workbench.pending_start;
        // Overlap the focus reservations so both rings stay fully visible.
        let reservation = theme::FOCUS_WIDTH + theme::FOCUS_GAP;
        let mut split = div()
            .id("session-start-split")
            .debug_selector(|| "session-start-split".into())
            .flex()
            .items_center()
            .flex_shrink_0();
        for (id, timer, kind) in [
            (
                "start-resume",
                StartTimer::Resume,
                theme::ButtonKind::Primary,
            ),
            (
                "start-restart",
                StartTimer::Restart,
                theme::ButtonKind::Navigation,
            ),
        ] {
            let resume = timer == StartTimer::Resume;
            let time_arg = [("time", time.to_string())];
            let (label, detail, accessible, intent, width) = if resume {
                (
                    l.text("resume").to_string(),
                    l.format("resume-detail", &time_arg),
                    l.format("resume-label", &time_arg),
                    Intent::Resume,
                    theme::SESSION_WIDTH,
                )
            } else {
                (
                    l.text("restart").to_string(),
                    l.text("restart-detail").to_string(),
                    l.text("restart-label").to_string(),
                    Intent::Start,
                    theme::RESTART_WIDTH,
                )
            };
            let label = if pending == Some(timer) {
                l.text("preparing").to_string()
            } else {
                label
            };
            let style = p.button(kind, enabled, false);
            let detail_ink = match (enabled, resume) {
                (true, true) => theme::ON_PRIMARY_DETAIL,
                (true, false) => p.muted,
                (false, _) => style.ink,
            };
            let radius = px(theme::RADIUS);
            let face = div()
                .id(SharedString::from(format!("{id}-face")))
                .min_h(px(theme::SESSION_CONTROL))
                .min_w(px(self.data.typography.size(width)))
                .px(px(12.))
                .flex()
                .items_center()
                .gap(px(8.))
                .relative()
                .bg(rgb(style.fill))
                .text_color(rgb(style.ink))
                .border_1()
                .border_color(rgb(style.border))
                .when(resume, |view| view.rounded_tl(radius).rounded_bl(radius))
                .when(!resume, |view| view.rounded_tr(radius).rounded_br(radius))
                .child(if resume {
                    Self::session_icon(false, style.ink)
                } else {
                    Self::restart_icon(style.ink)
                })
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .child(
                            div()
                                .font_weight(gpui::FontWeight::MEDIUM)
                                .text_size(px(self.data.typography.size(14.)))
                                .child(label),
                        )
                        .child(
                            div()
                                .id(SharedString::from(format!("{id}-detail")))
                                .debug_selector(move || format!("{id}-detail"))
                                .text_size(px(self.data.typography.size(12.)))
                                .text_color(rgb(detail_ink))
                                .child(detail),
                        ),
                )
                .when(enabled, |view| {
                    view.hover(move |state| state.bg(rgb(style.hover)))
                        .active(move |state| state.bg(rgb(style.pressed)).top(px(1.)))
                });
            split = split.child(
                self.activation_target(id.into(), accessible, enabled, intent, p)
                    .when(!resume, |view| view.ml(px(-reservation)))
                    .child(face),
            );
        }
        split.into_any_element()
    }

    pub(super) fn tools_panel(&self, p: Palette, window: &Window) -> gpui::AnyElement {
        let presenting = self.data.stage == SessionStage::Presenting;
        let has_steps = !self.data.workbench.steps.is_empty();
        let resumable = self
            .data
            .workbench
            .resumable
            .clone()
            .filter(|_| !presenting);
        let mut navigation = div()
            .id("session-navigation")
            .debug_selector(|| "session-navigation".into())
            .flex()
            .items_center()
            .flex_shrink_0()
            .gap(px(4.));
        navigation = if let Some(time) = resumable {
            navigation.child(self.session_split(&time, p))
        } else {
            navigation.child(
                self.button(
                    if presenting { "stop" } else { "start" }.into(),
                    self.locale
                        .text(if presenting {
                            "stop"
                        } else if self.data.busy {
                            "preparing"
                        } else {
                            "start"
                        })
                        .into(),
                    presenting || (self.data.start_reason.is_none() && !self.data.busy),
                    true,
                    if presenting {
                        Intent::Stop
                    } else {
                        Intent::Start
                    },
                    p,
                ),
            )
        };
        if !presenting {
            navigation = navigation.child(
                self.button(
                    "start-mode".into(),
                    self.locale
                        .text(super::start_mode::label(self.data.start_mode))
                        .into(),
                    !self.data.busy,
                    false,
                    Intent::StartModeMenu,
                    p,
                )
                .aria_label(
                    self.locale.format(
                        "start-mode-trigger",
                        &[(
                            "mode",
                            self.locale
                                .text(super::start_mode::label(self.data.start_mode))
                                .into(),
                        )],
                    ),
                ),
            );
        }
        if has_steps {
            for (id, label, intent) in [
                ("previous", "wb-previous", Intent::Previous),
                ("next", "wb-next", Intent::Next),
            ] {
                navigation = navigation.child(self.button(
                    id.into(),
                    self.locale.text(label).into(),
                    true,
                    false,
                    intent,
                    p,
                ));
            }
            let detail = self.data.workbench.progress.clone();
            let typography = self.data.typography.clone();
            navigation = navigation.child(
                self.activation_target(
                    "workbench-progress".into(),
                    self.locale
                        .format("wb-jump-detail", &[("progress", detail.clone())]),
                    true,
                    Intent::Jump,
                    p,
                )
                .track_focus(
                    &self
                        .progress_focus
                        .clone()
                        .tab_stop(self.data.browser.is_none()),
                )
                .when(!self.progress_focus.is_focused(window), |view| {
                    view.tooltip(move |_, cx| {
                        cx.new(|_| FilenameTooltip {
                            filename: detail.clone(),
                            palette: p,
                            typography: typography.clone(),
                        })
                        .into()
                    })
                })
                .child(
                    div()
                        .id("workbench-progress-face")
                        .min_h(px(theme::CONTROL))
                        .px(px(12.))
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .relative()
                        .rounded(px(theme::RADIUS))
                        .bg(rgb(p.soft))
                        .text_size(px(self.data.typography.size(14.)))
                        .text_color(rgb(p.body))
                        .hover(move |style| style.bg(rgb(p.card)))
                        .active(move |style| style.bg(rgb(p.card)).top(px(1.)))
                        .child(self.data.workbench.progress_compact.clone())
                        .child(
                            div()
                                .text_color(rgb(p.muted))
                                .child(self.locale.text("wb-jump-action")),
                        ),
                ),
            );
        }
        let mut strip = div()
            .id("workbench-controls")
            .debug_selector(|| "workbench-controls".into())
            .flex()
            .items_center()
            .flex_wrap()
            .gap(px(12.))
            .child(navigation)
            .when(self.data.workbench.timed, |view| {
                view.child(
                    self.button(
                        "automatic-playback".into(),
                        self.locale
                            .text(if self.data.workbench.autoplay_paused {
                                "wb-auto-resume"
                            } else {
                                "wb-auto-pause"
                            })
                            .into(),
                        presenting && !self.data.busy,
                        false,
                        Intent::ToggleAutoplay,
                        p,
                    ),
                )
            });
        if has_steps {
            let mut blanking = div()
                .flex()
                .items_center()
                .rounded(px(theme::PANEL_RADIUS))
                .bg(rgb(p.soft));
            for (id, label, intent) in [
                ("black", "wb-black", Intent::Black),
                ("white", "wb-white", Intent::White),
            ] {
                blanking = blanking.child(self.button(
                    id.into(),
                    self.locale.text(label).into(),
                    true,
                    false,
                    intent,
                    p,
                ));
            }
            let mut tools = div()
                .flex()
                .items_center()
                .flex_wrap()
                .rounded(px(theme::PANEL_RADIUS))
                .bg(rgb(p.soft));
            for (id, label, tool) in [
                ("laser", "wb-laser", Tool::Laser),
                ("pen", "wb-pen", Tool::Pen),
                ("highlighter", "wb-highlighter", Tool::Highlighter),
                ("eraser", "wb-eraser", Tool::Eraser),
            ] {
                if presenting && tool != Tool::Laser {
                    continue;
                }
                tools = tools.child(self.button(
                    id.into(),
                    self.locale.text(label).into(),
                    true,
                    self.data.workbench.tool == tool,
                    Intent::Tool(tool),
                    p,
                ));
            }
            if presenting {
                tools = tools.child(self.button(
                    "drawing-tools".into(),
                    self.locale.text("wb-drawing-tools").into(),
                    !self.data.busy,
                    false,
                    Intent::PresenterMenu(true),
                    p,
                ));
            }
            let secondary =
                div()
                    .flex()
                    .items_center()
                    .flex_wrap()
                    .gap(px(4.))
                    .when(!presenting, |view| {
                        view.child(self.button(
                            "clear-strokes".into(),
                            self.locale.text("wb-clear").into(),
                            true,
                            false,
                            Intent::Core(Command::ClearStrokes),
                            p,
                        ))
                    });
            strip = strip.child(blanking).child(tools).child(secondary);
        }
        div()
            .relative()
            .px(px(12.))
            .pb(px(8.))
            .flex_shrink_0()
            .flex()
            .flex_col()
            .gap(px(4.))
            .child(strip)
            .when_some(
                if presenting {
                    None
                } else {
                    self.data.start_reason.clone()
                },
                |view, reason| {
                    view.child(
                        div()
                            .id("start-reason")
                            .debug_selector(|| "start-reason".into())
                            .pl(px(4.))
                            .text_size(px(self.data.typography.size(12.)))
                            .text_color(rgb(p.muted))
                            .child(reason),
                    )
                },
            )
            .when(
                self.progress_focus.is_focused(window) && has_steps,
                |view| {
                    view.child(
                        div()
                            .id("progress-detail")
                            .debug_selector(|| "progress-detail".into())
                            .absolute()
                            .bottom(gpui::relative(1.))
                            .left(px(16.))
                            .max_w_full()
                            .p(px(12.))
                            .rounded(px(theme::RADIUS))
                            .bg(rgb(p.card))
                            .border_1()
                            .border_color(rgb(p.border))
                            .text_size(px(self.data.typography.size(12.)))
                            .text_color(rgb(p.muted))
                            .child(self.data.workbench.progress.clone()),
                    )
                },
            )
            .into_any_element()
    }
    /// One caption style for every workbench pane label.
    fn pane_caption(&self, key: &'static str, p: Palette) -> gpui::Div {
        div()
            .flex_shrink_0()
            .pb(px(4.))
            .font_weight(gpui::FontWeight::MEDIUM)
            .text_size(px(self.data.typography.size(12.)))
            .text_color(rgb(p.muted))
            .child(self.locale.text(key))
    }
    /// Notes-first divider: a 1px line inside an 8px hit target.
    fn hairline_split(
        &self,
        id: &'static str,
        axis: super::SplitAxis,
        can_drag: bool,
        p: Palette,
        cx: &mut Context<Self>,
    ) -> gpui::Stateful<Div> {
        let column = axis == super::SplitAxis::Column;
        let dragging = self.split_drag == Some((WorkbenchLayout::NotesFirst, axis));
        div()
            .id(id)
            .debug_selector(move || id.into())
            .group(id)
            .flex_shrink_0()
            .flex()
            .justify_center()
            .when(column, |view| view.w(px(8.)).h_full())
            .when(!column, |view| view.flex_col().h(px(8.)).w_full())
            .when(can_drag && column, |view| view.cursor_col_resize())
            .when(can_drag && !column, |view| view.cursor_row_resize())
            .child(
                div()
                    .when(column, |view| view.w(px(1.)).h_full())
                    .when(!column, |view| view.h(px(1.)).w_full())
                    .bg(rgb(if dragging { p.focus } else { p.border }))
                    .when(can_drag, |view| {
                        view.group_hover(id, move |style| style.bg(rgb(p.focus)))
                    }),
            )
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, _, _, cx| {
                    if can_drag {
                        this.split_drag = Some((WorkbenchLayout::NotesFirst, axis));
                        cx.notify();
                    }
                }),
            )
    }
    pub(super) fn workbench_panel(
        &mut self,
        p: Palette,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> gpui::AnyElement {
        let settings = &self.data.workbench.settings;
        let ratio = settings.next_slide_ratio as f32;
        let notes = settings.effective_notes_ratio() as f32;
        let notes_width = settings.effective_notes_first_notes_ratio() as f32;
        let current_share = settings.effective_notes_first_current_ratio() as f32;
        let stacked = window.viewport_size().width < px(800.);
        // Narrow windows always stack in the standard order.
        let layout = if stacked {
            WorkbenchLayout::Standard
        } else {
            WorkbenchLayout::for_mode(self.data.start_mode)
        };
        let mut notes_frames = self
            .data
            .workbench
            .surfaces
            .iter()
            .chain(
                self.data
                    .workbench
                    .frames
                    .iter()
                    .map(|(surface, _)| surface),
            )
            .filter_map(|surface| matches!(surface, Surface::Notes(_)).then_some(*surface))
            .collect::<Vec<_>>();
        notes_frames.sort_by_key(|surface| match surface {
            Surface::Notes(index) => *index,
            _ => 0,
        });
        notes_frames.dedup();
        let mut note_panel = div()
            .w_full()
            .flex_shrink_0()
            .flex()
            .flex_col()
            .gap(px(4.))
            .p(px(12.))
            .rounded(px(theme::RADIUS))
            .bg(rgb(p.card))
            .child(self.pane_caption("wb-notes", p));
        if notes_frames.is_empty() {
            note_panel = note_panel.child(
                div()
                    .text_color(rgb(p.muted))
                    .child(self.locale.text("wb-no-notes")),
            );
        }
        for surface in notes_frames {
            let aspect = self
                .frame(surface)
                .map(|frame| frame.frame.width() as f32 / frame.frame.height().max(1) as f32)
                .or_else(|| {
                    self.textures
                        .borrow()
                        .frame(surface)
                        .map(|frame| frame.width() as f32 / frame.height().max(1) as f32)
                })
                .unwrap_or(16. / 9.);
            note_panel = note_panel.child(
                div()
                    .w_full()
                    .flex_shrink_0()
                    .aspect_ratio(aspect)
                    .child(self.picture(surface, false)),
            );
        }
        let can_drag = self.data.workbench.writable && !self.data.busy;
        let split_bounds = Rc::new(Cell::new(gpui::Bounds::<gpui::Pixels>::default()));
        let next_bounds = Rc::new(Cell::new(gpui::Bounds::<gpui::Pixels>::default()));
        let split_measure = split_bounds.clone();
        let next_measure = next_bounds.clone();
        let content = div()
            .id("workbench-splits")
            .debug_selector(|| "workbench-splits".into())
            .relative()
            .flex_1()
            .min_h_0()
            .flex()
            .when(stacked, |view| view.flex_col())
            .when(layout == WorkbenchLayout::Standard, |view| view.gap(px(6.)))
            .on_mouse_move(
                cx.listener(move |this, event: &gpui::MouseMoveEvent, _, cx| {
                    if event.pressed_button != Some(MouseButton::Left) {
                        return;
                    }
                    let Some((layout, axis)) = this.split_drag else {
                        return;
                    };
                    let bounds = split_bounds.get();
                    let column = next_bounds.get();
                    let (mut primary, mut secondary) = layout.ratios(&this.data.workbench.settings);
                    match (layout, axis) {
                        (WorkbenchLayout::Standard, super::SplitAxis::Column) => {
                            primary = (1.
                                - (event.position.x - bounds.origin.x) / bounds.size.width)
                                .clamp(0.15, 0.65) as f64;
                        }
                        (WorkbenchLayout::Standard, super::SplitAxis::Row) => {
                            // The divider's center includes its 6px bar and
                            // adjacent 4px gaps; only pane heights own the ratio.
                            secondary = (1.
                                - ((event.position.y - column.origin.y - px(7.))
                                    / (column.size.height - px(14.)).max(px(1.))))
                            .clamp(0.55, 0.8) as f64;
                        }
                        (WorkbenchLayout::NotesFirst, super::SplitAxis::Column) => {
                            // Notes width is relative to the row; the 8px
                            // divider is centred 4px after it.
                            primary = ((event.position.x - bounds.origin.x - px(4.))
                                / bounds.size.width.max(px(1.)))
                            .clamp(0.4, 0.7) as f64;
                        }
                        (WorkbenchLayout::NotesFirst, super::SplitAxis::Row) => {
                            // Panes share the column height minus the 8px divider.
                            secondary = ((event.position.y - column.origin.y - px(4.))
                                / (column.size.height - px(8.)).max(px(1.)))
                            .clamp(0.5, 0.8) as f64;
                        }
                    }
                    actions::dispatch(
                        &this.shell,
                        Intent::Split {
                            layout,
                            primary,
                            secondary,
                            save: false,
                        },
                        cx,
                    );
                }),
            )
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, _, _, cx| this.finish_split(cx)),
            )
            .on_mouse_up_out(
                MouseButton::Left,
                cx.listener(|this, _, _, cx| this.finish_split(cx)),
            )
            .child(
                canvas(
                    move |bounds, _, _| split_measure.set(bounds),
                    |_, _, _, _| {},
                )
                .absolute()
                .size_full(),
            );
        let content = if layout == WorkbenchLayout::NotesFirst {
            content
                .child(
                    div()
                        .id("notes-panel")
                        .debug_selector(|| "notes-panel".into())
                        .w(gpui::relative(notes_width))
                        .h_full()
                        .flex_shrink_0()
                        .overflow_y_scroll()
                        .flex()
                        .flex_col()
                        .rounded(px(theme::RADIUS))
                        .bg(rgb(p.card))
                        .child(note_panel),
                )
                .child(self.hairline_split(
                    "column-split",
                    super::SplitAxis::Column,
                    can_drag,
                    p,
                    cx,
                ))
                .child(
                    div()
                        .id("previews-column")
                        .debug_selector(|| "previews-column".into())
                        .relative()
                        .flex_1()
                        .min_w_0()
                        .h_full()
                        .overflow_hidden()
                        .flex()
                        .flex_col()
                        .child(
                            canvas(
                                move |bounds, _, _| next_measure.set(bounds),
                                |_, _, _, _| {},
                            )
                            .absolute()
                            .size_full(),
                        )
                        .child(
                            div()
                                .id("current-pane")
                                .debug_selector(|| "current-pane".into())
                                .flex_grow(current_share)
                                .flex_shrink(1.)
                                .flex_basis(gpui::relative(0.))
                                .min_h_0()
                                .overflow_hidden()
                                .flex()
                                .flex_col()
                                .child(self.pane_caption("wb-current-slide", p))
                                .child(
                                    div()
                                        .flex_1()
                                        .min_h_0()
                                        .child(self.picture(Surface::Current, true)),
                                ),
                        )
                        .child(self.hairline_split(
                            "row-split",
                            super::SplitAxis::Row,
                            can_drag,
                            p,
                            cx,
                        ))
                        .child(
                            div()
                                .id("next-pane")
                                .debug_selector(|| "next-pane".into())
                                .flex_grow(1. - current_share)
                                .flex_shrink(1.)
                                .flex_basis(gpui::relative(0.))
                                .min_h_0()
                                .overflow_hidden()
                                .flex()
                                .flex_col()
                                .child(self.pane_caption("wb-next-slide", p))
                                .child(
                                    div()
                                        .flex_1()
                                        .min_h_0()
                                        .child(self.picture(Surface::Next, false)),
                                ),
                        ),
                )
        } else {
            content
                .child(
                    div()
                        .id("current-pane")
                        .debug_selector(|| "current-pane".into())
                        .when(!stacked, |view| view.w(gpui::relative(1. - ratio)).h_full())
                        .when(stacked, |view| view.w_full().flex_1().min_h_0())
                        .overflow_hidden()
                        .child(self.picture(Surface::Current, true)),
                )
                .when(!stacked, |view| {
                    view.child(
                        div()
                            .id("horizontal-split")
                            .debug_selector(|| "horizontal-split".into())
                            .w(px(6.))
                            .h_full()
                            .bg(rgb(p.border))
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |this, _, _, _| {
                                    if can_drag {
                                        this.split_drag = Some((
                                            WorkbenchLayout::Standard,
                                            super::SplitAxis::Column,
                                        ));
                                    }
                                }),
                            ),
                    )
                })
                .child(
                    div()
                        .id("next-notes-pane")
                        .debug_selector(|| "next-notes-pane".into())
                        .relative()
                        .flex_1()
                        .min_w_0()
                        .when(!stacked, |view| view.h_full())
                        .when(stacked, |view| view.w_full().flex_1().min_h_0())
                        .overflow_hidden()
                        .flex()
                        .flex_col()
                        .gap(px(4.))
                        .child(
                            canvas(
                                move |bounds, _, _| next_measure.set(bounds),
                                |_, _, _, _| {},
                            )
                            .absolute()
                            .size_full(),
                        )
                        .child(
                            div()
                                .id("next-pane")
                                .debug_selector(|| "next-pane".into())
                                .flex_grow((1. - notes).max(0.15))
                                .flex_shrink(1.)
                                .flex_basis(gpui::relative(0.))
                                .min_h_0()
                                .overflow_hidden()
                                .flex()
                                .flex_col()
                                .child(self.pane_caption("wb-next-slide", p))
                                .child(
                                    div()
                                        .flex_1()
                                        .min_h_0()
                                        .child(self.picture(Surface::Next, false)),
                                ),
                        )
                        .child(
                            div()
                                .id("vertical-split")
                                .debug_selector(|| "vertical-split".into())
                                .flex_shrink_0()
                                .h(px(6.))
                                .w_full()
                                .bg(rgb(p.border))
                                .on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener(move |this, _, _, _| {
                                        if can_drag {
                                            this.split_drag = Some((
                                                WorkbenchLayout::Standard,
                                                super::SplitAxis::Row,
                                            ));
                                        }
                                    }),
                                ),
                        )
                        .child(
                            div()
                                .id("notes-panel")
                                .debug_selector(|| "notes-panel".into())
                                .flex_grow(notes)
                                .flex_shrink(1.)
                                .flex_basis(gpui::relative(0.))
                                .min_h_0()
                                .overflow_y_scroll()
                                .flex()
                                .flex_col()
                                .rounded(px(theme::RADIUS))
                                .bg(rgb(p.card))
                                .child(note_panel),
                        ),
                )
        };
        let mut view = div()
            .id("workbench")
            .debug_selector(|| "workbench".into())
            .flex_1()
            .min_h_0()
            .w_full()
            .flex()
            .flex_col()
            .gap(px(8.))
            .child(content);
        for slot in &self.data.workbench.slots {
            let action = if slot.active && !slot.paused {
                self.locale.text("media-pause")
            } else {
                self.locale.text("media-play")
            };
            let title = format!(
                "{} {} · {} / {} s",
                action,
                slot.id + 1,
                slot.progress.0 / 1000,
                slot.progress.1 / 1000
            );
            let mut row = div().flex().items_center().gap(px(8.)).child(self.button(
                format!("media-slot-{}", slot.id),
                title,
                self.data.stage == ferrostage_core::SessionStage::Presenting
                    && self.data.workbench.overlay.blank_mode.is_none(),
                slot.active,
                Intent::Media(slot.id),
                p,
            ));
            if let Some(error) = slot.error {
                let key = match error {
                    ferrostage_media::MediaError::RuntimeUnavailable => "media-runtime-missing",
                    ferrostage_media::MediaError::SourceUnavailable => "media-source-missing",
                    ferrostage_media::MediaError::Limit => "media-limit",
                    _ => "media-failed",
                };
                row = row.child(self.locale.text(key));
            }
            view = view.child(row);
        }
        if let Some(debug) = &self.data.workbench.debug {
            view = view.child(format!(
                "{debug} · GPU managed {} bytes · UI paint {:.2} ms",
                self.textures.borrow().bytes(),
                self.paint_ms.get()
            ));
        }
        view.into_any_element()
    }
    fn finish_split(&mut self, cx: &mut Context<Self>) {
        if let Some((layout, _)) = self.split_drag.take() {
            let (primary, secondary) = layout.ratios(&self.data.workbench.settings);
            actions::dispatch(
                &self.shell,
                Intent::Split {
                    layout,
                    primary,
                    secondary,
                    save: true,
                },
                cx,
            );
        }
    }
}

#[cfg(all(test, feature = "test-support"))]
pub(crate) struct PreferencesProbe {
    pub layout: ferrostage_settings::PresenterSettings,
    pub pointer: ferrostage_settings::PointerSettings,
    pub timer: String,
    pub warning: u8,
    pub clock: bool,
    pub debug: bool,
}
#[cfg(all(test, feature = "test-support"))]
impl Presenter {
    pub(crate) fn preferences_probe(&self) -> PreferencesProbe {
        let data = &self.data.workbench;
        PreferencesProbe {
            layout: data.settings.clone(),
            pointer: data.pointer.clone(),
            timer: data.timer.clone(),
            warning: data.warning,
            clock: data.clock.is_some(),
            debug: data.debug.is_some(),
        }
    }
}
