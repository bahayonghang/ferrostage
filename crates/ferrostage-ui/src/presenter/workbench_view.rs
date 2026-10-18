use super::*;
use crate::workbench::{ReadyFrame, Surface, Tool};
use ferrostage_core::{AudienceProjectionSnapshot, PresentationCommand as Command};
use gpui::{MouseButton, canvas};
use std::{cell::Cell, rc::Rc, time::Instant};

pub(super) struct WorkbenchData {
    pub frames: Vec<(Surface, ReadyFrame)>,
    pub surfaces: Vec<Surface>,
    pub overlay: AudienceProjectionSnapshot,
    pub media: Vec<ferrostage_core::AudienceMediaFrame>,
    slots: Vec<crate::media_playback::SlotView>,
    pub(super) settings: ferrostage_settings::PresenterSettings,
    pointer: ferrostage_settings::PointerSettings,
    tool: Tool,
    grid: bool,
    pub(super) grid_start: usize,
    pub steps: Vec<ferrostage_core::PhysicalPage>,
    pub progress: String,
    progress_compact: String,
    timer: String,
    clock: Option<String>,
    warning: u8,
    paused: bool,
    pub budget: usize,
    debug: Option<String>,
    pub(super) writable: bool,
}
impl WorkbenchData {
    pub fn from_shell(shell: &Shell) -> Self {
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
            grid: shell.workbench.grid,
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
            .bg(rgb(0x000000))
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
    pub(super) fn timer_panel(&self, p: Palette) -> gpui::AnyElement {
        let data = &self.data.workbench;
        div()
            .id("presenter-timer")
            .debug_selector(|| "presenter-timer".into())
            .flex()
            .items_center()
            .gap(px(12.))
            .child(
                div()
                    .font(gpui::font(theme::MONO))
                    .text_size(px(self.data.typography.size(24.)))
                    .text_color(rgb(match data.warning {
                        2 => p.error,
                        1 => p.warning,
                        _ => p.ink,
                    }))
                    .child(data.timer.clone()),
            )
            .when(data.warning > 0, |view| {
                view.child(self.locale.text(if data.warning == 2 {
                    "wb-timer-red"
                } else {
                    "wb-timer-yellow"
                }))
            })
            .when_some(data.clock.clone(), |v, clock| {
                v.child(
                    div()
                        .font(gpui::font(theme::MONO))
                        .text_size(px(self.data.typography.size(14.)))
                        .text_color(rgb(p.muted))
                        .child(clock),
                )
            })
            .when(self.data.stage == SessionStage::Presenting, |v| {
                v.child(
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

    pub(super) fn tools_panel(&self, p: Palette, window: &Window) -> gpui::AnyElement {
        let presenting = self.data.stage == SessionStage::Presenting;
        let has_steps = !self.data.workbench.steps.is_empty();
        let mut navigation = div()
            .id("session-navigation")
            .debug_selector(|| "session-navigation".into())
            .flex()
            .items_center()
            .flex_shrink_0()
            .gap(px(4.))
            .child(
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
            );
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
            .child(navigation);
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
                ("navigate", "wb-navigate", Tool::Navigate),
                ("laser", "wb-laser", Tool::Laser),
                ("pen", "wb-pen", Tool::Pen),
                ("highlighter", "wb-highlighter", Tool::Highlighter),
                ("eraser", "wb-eraser", Tool::Eraser),
            ] {
                tools = tools.child(self.button(
                    id.into(),
                    self.locale.text(label).into(),
                    true,
                    self.data.workbench.tool == tool,
                    Intent::Tool(tool),
                    p,
                ));
            }
            let secondary = div()
                .flex()
                .items_center()
                .flex_wrap()
                .gap(px(4.))
                .child(self.button(
                    "clear-strokes".into(),
                    self.locale.text("wb-clear").into(),
                    true,
                    false,
                    Intent::Core(Command::ClearStrokes),
                    p,
                ))
                .child(self.button(
                    "grid".into(),
                    self.locale.text("wb-grid").into(),
                    true,
                    self.data.workbench.grid,
                    Intent::Grid,
                    p,
                ))
                .child(self.button(
                    "document-options".into(),
                    format!(
                        "{} {}",
                        self.locale.text("wb-document"),
                        if self.data.layout_menu.is_some() {
                            "⌃"
                        } else {
                            "⌄"
                        }
                    ),
                    !self.data.busy,
                    false,
                    Intent::Document,
                    p,
                ));
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
    pub(super) fn workbench_panel(
        &mut self,
        p: Palette,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> gpui::AnyElement {
        let ratio = self.data.workbench.settings.next_slide_ratio as f32;
        let notes = self.data.workbench.settings.effective_notes_ratio() as f32;
        let stacked = window.viewport_size().width < px(800.);
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
        let scale = self.data.workbench.settings.notes_font_scale as f32;
        let mut note_panel = div()
            .id("notes-panel")
            .debug_selector(|| "notes-panel".into())
            .flex_grow(notes.max(0.15))
            .flex_shrink(1.)
            .flex_basis(gpui::relative(0.))
            .min_h_0()
            .overflow_scroll()
            .flex()
            .flex_col()
            .gap(px(4.))
            .p(px(12.))
            .rounded(px(theme::RADIUS))
            .bg(rgb(p.card))
            .child(
                div()
                    .flex_shrink_0()
                    .text_size(px(self.data.typography.size(12.)))
                    .text_color(rgb(p.muted))
                    .child(self.locale.text("wb-notes")),
            );
        if notes_frames.is_empty() {
            note_panel = note_panel.child(
                div()
                    .text_color(rgb(p.muted))
                    .child(self.locale.text("wb-no-notes")),
            );
        }
        for surface in notes_frames {
            note_panel = note_panel.child(
                div()
                    .w_full()
                    .flex_grow(1.)
                    .flex_basis(gpui::relative(0.))
                    .min_h(px(80. * scale))
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
            .gap(px(6.))
            .on_mouse_move(
                cx.listener(move |this, event: &gpui::MouseMoveEvent, _, cx| {
                    if event.pressed_button != Some(MouseButton::Left) {
                        return;
                    }
                    if let Some(vertical) = this.split_drag {
                        let bounds = split_bounds.get();
                        let right = next_bounds.get();
                        let next = if !vertical {
                            (1. - (event.position.x - bounds.origin.x) / bounds.size.width)
                                .clamp(0.15, 0.65) as f64
                        } else {
                            this.data.workbench.settings.next_slide_ratio
                        };
                        let notes = if vertical {
                            ((event.position.y - right.origin.y) / right.size.height)
                                .clamp(0.55, 0.8) as f64
                        } else {
                            this.data.workbench.settings.notes_ratio
                        };
                        actions::dispatch(
                            &this.shell,
                            Intent::Split {
                                next,
                                notes,
                                save: false,
                            },
                            cx,
                        );
                    }
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
            )
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
                                    this.split_drag = Some(false);
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
                    .child(note_panel)
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
                                        this.split_drag = Some(true);
                                    }
                                }),
                            ),
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
                            .child(
                                div()
                                    .flex_shrink_0()
                                    .text_size(px(self.data.typography.size(12.)))
                                    .text_color(rgb(p.muted))
                                    .child(self.locale.text("wb-next-slide")),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .min_h_0()
                                    .child(self.picture(Surface::Next, false)),
                            ),
                    ),
            );
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
        if self.split_drag.take().is_some() {
            actions::dispatch(
                &self.shell,
                Intent::Split {
                    next: self.data.workbench.settings.next_slide_ratio,
                    notes: self.data.workbench.settings.notes_ratio,
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
