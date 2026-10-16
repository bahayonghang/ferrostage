mod settings_view;
mod workbench_view;
use crate::{
    actions::{self, Intent},
    shell::{Modal, Shell},
    theme::{self, Palette},
};
use ferrostage_core::SessionStage;
use gpui::{
    AccessibleAction, Context, Div, ExternalPaths, FocusHandle, Render, Role, SharedString,
    Stateful, WeakEntity, Window, WindowAppearance, div, prelude::*, px, rgb,
};

/// Drawing data, captured before open_window synchronously draws its first
/// frame. Rendering never leases the controller that may be opening this view.
pub(crate) struct PresenterData {
    theme: ferrostage_settings::Theme,
    stage: SessionStage,
    filename: String,
    modal: Option<Modal>,
    displays: Option<crate::display_settings::DisplayPanel>,
    error: Option<String>,
    warnings: Vec<String>,
    start_reason: Option<String>,
    loading: bool,
    busy: bool,
    can_open: bool,
    dirty: bool,
    save_error: Option<String>,
    pages: usize,
    current: usize,
    preview: Option<usize>,
    recent: Vec<String>,
    workbench: workbench_view::WorkbenchData,
    preferences: settings_view::SettingsData,
    staging: Option<crate::shell::reload::PresenterStaging>,
}

impl PresenterData {
    pub(crate) fn from_shell(shell: &Shell) -> Self {
        Self {
            theme: shell.settings.settings.general.theme,
            staging: shell.presenter_staging(),
            stage: shell.state.stage(),
            filename: shell
                .path
                .as_ref()
                .and_then(|path| path.file_name())
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_else(|| "FerroStage".into()),
            modal: shell.modal.clone(),
            displays: shell.display_panel(),
            error: shell
                .graphics_error
                .as_ref()
                .map(|error| shell.locale.error("wb-upload-error", error))
                .or_else(|| shell.error.clone()),
            warnings: shell.warnings.clone(),
            start_reason: shell.start_reason(),
            loading: shell.loading,
            busy: shell.busy(),
            can_open: shell.import_allowed() && !shell.busy(),
            dirty: shell.settings.dirty,
            save_error: shell.settings.save_error.as_ref().map(ToString::to_string),
            pages: shell.state.page_count(),
            current: shell.state.current_page().0 + 1,
            preview: shell.state.jump_preview().map(|page| page.0 + 1),
            workbench: workbench_view::WorkbenchData::from_shell(shell),
            preferences: settings_view::SettingsData::from_shell(shell),
            recent: shell
                .settings
                .settings
                .general
                .recent_files
                .iter()
                .take(3)
                .cloned()
                .collect(),
        }
    }
}

pub(crate) struct Presenter {
    shell: WeakEntity<Shell>,
    data: PresenterData,
    locale: crate::locale::Locale,
    focus: FocusHandle,
    modal_focus: FocusHandle,
    return_focus: Option<FocusHandle>,
    had_modal: bool,
    textures: std::rc::Rc<
        std::cell::RefCell<crate::texture_cache::TextureCache<crate::workbench::Surface>>,
    >,
    split_drag: Option<bool>,
    settings_drag: std::rc::Rc<std::cell::Cell<Option<&'static str>>>,
    drawing: std::rc::Rc<std::cell::Cell<bool>>,
    paint_ms: std::rc::Rc<std::cell::Cell<f64>>,
}

struct FilenameTooltip {
    filename: String,
    palette: Palette,
}

impl Render for FilenameTooltip {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .max_w(px(560.))
            .p(px(12.))
            .rounded(px(theme::RADIUS))
            .border_1()
            .border_color(rgb(self.palette.border))
            .bg(rgb(self.palette.card))
            .text_color(rgb(self.palette.ink))
            .font(theme::sans_font())
            .text_size(px(14.))
            .whitespace_normal()
            .child(self.filename.clone())
    }
}

impl Presenter {
    pub(crate) fn staged(&self, attempt: u64) -> bool {
        self.textures.borrow().staged(attempt)
    }
    pub(crate) fn promote(&mut self, attempt: u64, cx: &mut gpui::App) -> bool {
        self.textures.borrow_mut().promote(attempt, cx)
    }
    pub(crate) fn cancel_staging(&mut self, cx: &mut gpui::App) {
        self.textures.borrow_mut().cancel_stage(cx);
    }
    #[cfg(all(test, feature = "test-support"))]
    pub(crate) fn seed_budget_image(
        &mut self,
        frame: ferrostage_core::BgraFrame,
        budget: usize,
        window: &mut Window,
        cx: &mut gpui::App,
    ) {
        let mut cache = self.textures.borrow_mut();
        cache.sync(
            &[(crate::workbench::Surface::Current, frame)],
            &[crate::workbench::Surface::Current],
            budget,
            window,
            cx,
        );
        assert!(cache.paint(
            crate::workbench::Surface::Current,
            None,
            window,
            cx,
            |_, _, _, _| Ok(())
        ));
    }
    #[cfg(all(test, feature = "test-support"))]
    pub(crate) fn effective_texture_budget(&self) -> usize {
        self.textures.borrow().effective_budget()
    }
    pub(crate) fn accepts_texture_budget(&self, bytes: usize) -> bool {
        self.textures.borrow().accepts_budget(bytes)
    }
    pub(crate) fn apply_texture_budget(&mut self, bytes: usize) -> Result<(), String> {
        self.textures.borrow_mut().apply_budget(bytes)
    }
    pub fn new(
        shell: WeakEntity<Shell>,
        locale: crate::locale::Locale,
        data: PresenterData,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        if let Some(model) = shell.upgrade() {
            cx.observe(&model, |presenter, model, cx| {
                // GPUI delivers entity notifications after the controller lease
                // ends. Keep reads here, outside all synchronous draw paths.
                presenter.data = PresenterData::from_shell(model.read(cx));
                cx.notify();
            })
            .detach();
        }
        cx.on_release(|presenter, cx| presenter.textures.borrow_mut().release(cx))
            .detach();
        let focus = cx.focus_handle();
        window.focus(&focus, cx);
        Self {
            shell,
            data,
            locale,
            focus,
            modal_focus: cx.focus_handle(),
            return_focus: None,
            had_modal: false,
            textures: Default::default(),
            split_drag: None,
            settings_drag: Default::default(),
            drawing: Default::default(),
            paint_ms: Default::default(),
        }
    }

    #[cfg(all(test, feature = "test-support"))]
    pub(crate) fn rendered_document(&self) -> (&str, usize, usize) {
        (&self.data.filename, self.data.current, self.data.pages)
    }

    pub(crate) fn button(
        &self,
        id: String,
        label: String,
        enabled: bool,
        primary: bool,
        intent: Intent,
        palette: Palette,
    ) -> Stateful<Div> {
        let target = self.shell.clone();
        let keyboard = target.clone();
        let a11y = target.clone();
        let click = intent.clone();
        let accessible = intent.clone();
        let fill = if !enabled {
            palette.card
        } else if primary {
            theme::PRIMARY
        } else {
            palette.soft
        };
        let ink = if !enabled {
            palette.muted
        } else if primary {
            theme::ON_PRIMARY
        } else {
            palette.body
        };
        div()
            .id(SharedString::from(id.clone()))
            .debug_selector(|| id.clone())
            .role(Role::Button)
            .key_context("FerroStageButton")
            .max_w_full()
            .min_w_0()
            .accessibility_id(SharedString::from(id.clone()))
            .aria_label(if enabled {
                label.clone()
            } else {
                self.locale
                    .format("disabled-label", &[("label", label.clone())])
            })
            .tab_index(0)
            .tab_stop(enabled)
            .border(px(theme::FOCUS_WIDTH))
            .border_color(rgb(palette.canvas))
            .rounded(px(theme::RADIUS + theme::FOCUS_GAP))
            .p(px(theme::FOCUS_GAP))
            .focus(|style| style.border_color(rgb(palette.focus)))
            .child(
                div()
                    .id(SharedString::from(format!("{id}-face")))
                    .min_h(px(theme::CONTROL))
                    .px(px(theme::SPACING))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded(px(theme::RADIUS))
                    .bg(rgb(fill))
                    .text_color(rgb(ink))
                    .border_1()
                    .border_color(rgb(palette.border))
                    .min_w_0()
                    .text_size(px(14.))
                    .child(div().truncate().child(label))
                    .when(enabled, |view| {
                        view.hover(move |style| {
                            style.bg(rgb(if primary {
                                theme::PRIMARY_HOVER
                            } else {
                                palette.card
                            }))
                        })
                        .active(move |style| {
                            style.bg(rgb(if primary {
                                theme::PRIMARY_ACTIVE
                            } else {
                                palette.card
                            }))
                        })
                    }),
            )
            .on_click(move |_, _, cx| {
                if enabled {
                    actions::dispatch(&target, click.clone(), cx);
                }
            })
            .on_action(move |_: &actions::ActivateButton, _, cx| {
                cx.stop_propagation();
                if enabled {
                    actions::dispatch(&keyboard, intent.clone(), cx);
                }
            })
            .when(enabled, |view| {
                view.on_a11y_action(AccessibleAction::Click, move |_, _, cx| {
                    actions::dispatch(&a11y, accessible.clone(), cx)
                })
            })
    }
}

impl Render for Presenter {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut prioritized = self
            .data
            .workbench
            .frames
            .iter()
            .filter(|(s, _)| *s != crate::workbench::Surface::Audience)
            .collect::<Vec<_>>();
        prioritized.sort_by_key(|(surface, _)| match surface {
            crate::workbench::Surface::Current => 0,
            crate::workbench::Surface::Media(_) => 1,
            crate::workbench::Surface::Next => 2,
            crate::workbench::Surface::Notes(_) => 3,
            crate::workbench::Surface::Thumbnail(_) => 3,
            _ => 4,
        });
        let mut frames = prioritized
            .into_iter()
            .map(|(surface, f)| (*surface, f.frame.clone()))
            .collect::<Vec<_>>();
        for frame in &self.data.workbench.media {
            frames.push((
                crate::workbench::Surface::Media(frame.slot_id()),
                frame.pixels().clone(),
            ));
        }
        frames.sort_by_key(|(surface, _)| match surface {
            crate::workbench::Surface::Current => 0,
            crate::workbench::Surface::Media(_) => 1,
            crate::workbench::Surface::Next => 2,
            crate::workbench::Surface::Notes(_) => 3,
            _ => 4,
        });
        let mut active = self.data.workbench.surfaces.clone();
        for (surface, _) in &frames {
            if !active.contains(surface) {
                active.push(*surface);
            }
        }
        active.retain(|surface| *surface != crate::workbench::Surface::Audience);
        active.sort_by_key(|surface| match surface {
            crate::workbench::Surface::Current => 0,
            crate::workbench::Surface::Media(_) => 1,
            crate::workbench::Surface::Next => 2,
            crate::workbench::Surface::Notes(_) => 3,
            _ => 3,
        });
        self.textures
            .borrow_mut()
            .sync(&frames, &active, self.data.workbench.budget, window, cx);
        let l = self.locale;
        let dark = matches!(
            window.appearance(),
            WindowAppearance::Dark | WindowAppearance::VibrantDark
        );
        let p = theme::palette(self.data.theme, dark);
        let stage = self.data.stage;
        let stage_label = match stage {
            SessionStage::Idle => l.text("stage-idle"),
            SessionStage::Ready => l.text("stage-ready"),
            SessionStage::Presenting => l.text("stage-presenting"),
        };
        let filename = self.data.filename.clone();
        let tooltip_filename = filename.clone();
        let modal = self.data.modal.clone();
        let error = self
            .textures
            .borrow()
            .error()
            .map(|error| l.error("wb-upload-error", error))
            .or_else(|| self.data.error.clone());
        let warnings = self.data.warnings.clone();
        let reason = self.data.start_reason.clone();
        let loading = self.data.loading;
        let busy = self.data.busy;
        let dirty = self.data.dirty;
        let save_error = self.data.save_error.clone();
        let pages = self.data.pages;
        let current = self.data.current;
        let preview = self.data.preview;
        let recent = self.data.recent.clone();
        window.set_window_title(&format!("{filename} — {stage_label} — FerroStage"));
        if modal.is_some() && !self.had_modal {
            self.return_focus = window.focused(cx);
            window.focus(&self.modal_focus, cx);
        }
        // Keep the return handle alive until the deferred tree check owns it.
        // A handle alone does not mean its former button is still rendered.
        let return_focus = if modal.is_none() && self.had_modal {
            Some(
                self.return_focus
                    .take()
                    .unwrap_or_else(|| self.focus.clone()),
            )
        } else {
            None
        };
        if let Some(return_focus) = &return_focus {
            window.focus(return_focus, cx);
        }
        self.had_modal = modal.is_some();
        if let Some(focused) = window.focused(cx) {
            let presenter = cx.weak_entity();
            // Auto-created button handles can outlive their removed element
            // (notably return_focus). Check the completed dispatch tree, after
            // the render/window lease ends. A newer focus choice wins.
            window.defer(cx, move |window, cx| {
                if window.focused(cx).as_ref() != Some(&focused) {
                    return;
                }
                if let Some(presenter) = presenter.upgrade() {
                    let presenter = presenter.read(cx);
                    let fallback = if presenter.had_modal {
                        presenter.modal_focus.clone()
                    } else {
                        presenter.focus.clone()
                    };
                    if !fallback.contains_focused(window, cx) {
                        window.focus(&fallback, cx);
                    }
                }
            });
        }
        if self.data.staging.is_none() {
            self.textures.borrow_mut().cancel_stage(cx);
        }
        let staging = self.data.staging.clone();
        let stage_cache = self.textures.clone();
        let stage_shell = self.shell.clone();
        let root = div()
            .id("presenter-root")
            .role(Role::Application)
            .aria_label(l.text("presenter-label"))
            .size_full()
            .flex()
            .flex_col()
            .bg(rgb(p.canvas))
            .text_color(rgb(p.body))
            .font(theme::sans_font())
            .text_size(px(16.))
            .child(
                gpui::canvas(
                    |_, _, _| (),
                    move |_, _, window, cx| {
                        if let Some(staging) = staging {
                            let result = stage_cache.borrow_mut().stage(
                                staging.attempt,
                                &staging.frames,
                                window,
                                cx,
                                crate::slide_surface::preupload,
                            );
                            cx.defer(move |cx| {
                                let _ = stage_shell.update(cx, |shell, cx| {
                                    shell.staging_status(staging.attempt, false, result, cx)
                                });
                            });
                        }
                    },
                )
                .absolute()
                .size_full(),
            )
            .capture_key_down(cx.listener(|this, event: &gpui::KeyDownEvent, _, cx| {
                if this.data.modal == Some(Modal::Settings)
                    && (this.data.preferences.panel.recording.is_some()
                        || this.data.preferences.panel.edit.is_some())
                {
                    actions::dispatch(
                        &this.shell,
                        Intent::SettingsKey {
                            key: event.keystroke.unparse(),
                            text: event.keystroke.key_char.clone().or_else(|| {
                                (event.keystroke.key.len() == 1)
                                    .then(|| event.keystroke.key.clone())
                            }),
                        },
                        cx,
                    );
                    cx.stop_propagation();
                }
            }))
            .on_key_down(cx.listener(|this, event: &gpui::KeyDownEvent, window, cx| {
                let key = event.keystroke.key.as_str();
                if key == "tab" {
                    if event.keystroke.modifiers.shift {
                        window.focus_prev(cx);
                    } else {
                        window.focus_next(cx);
                    }
                    cx.stop_propagation();
                } else if this.had_modal
                    && (key == "backspace" || key.len() == 1 && key.as_bytes()[0].is_ascii_digit())
                {
                    if let Err(error) = this.shell.update(cx, |shell, cx| shell.jump_key(key, cx)) {
                        tracing::debug!(%error,"jump target closed");
                    }
                    cx.stop_propagation();
                }
            }));
        if let Some(modal) = modal {
            if modal == Modal::Settings {
                return root
                    .track_focus(&self.modal_focus)
                    .key_context("FerroStageModal")
                    .child(self.settings_panel(p, window))
                    .into_any_element();
            }
            if modal == Modal::Document {
                return root
                    .track_focus(&self.modal_focus)
                    .child(self.document_panel(p))
                    .into_any_element();
            }
            if let Some(panel) = &self.data.displays {
                return root
                    .track_focus(&self.modal_focus)
                    .key_context("FerroStageModal")
                    .child(crate::display_settings::view::panel(
                        self, panel, l, p, false,
                    ))
                    .into_any_element();
            }
            let (title, description) = match modal {
                Modal::Settings => unreachable!("settings handled above"),
                Modal::Jump(ref text) => (
                    l.text("jump"),
                    l.format(
                        "jump-description",
                        &[
                            ("pages", pages.to_string()),
                            ("text", text.clone()),
                            ("preview", preview.unwrap_or(current).to_string()),
                        ],
                    ),
                ),
                Modal::Document => unreachable!("document modal handled above"),
                Modal::Displays(_) => unreachable!("display modal has prepared drawing data"),
            };
            return root
                .track_focus(&self.modal_focus)
                .key_context(if matches!(modal, Modal::Jump(_)) {
                    "FerroStageJump"
                } else {
                    "FerroStageModal"
                })
                .p(px(24.))
                .gap(px(16.))
                .child(div().text_size(px(20.)).text_color(rgb(p.ink)).child(title))
                .child(div().child(description))
                .when(matches!(modal, Modal::Jump(_)), |view| {
                    view.child(self.jump_preview_panel())
                })
                .when_some(error, |view, error| {
                    view.child(div().text_color(rgb(p.error)).child(error))
                })
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .gap(px(8.))
                        .when(matches!(modal, Modal::Jump(_)), |view| {
                            view.child(self.button(
                                "commit-jump".into(),
                                l.text("confirm-jump").into(),
                                true,
                                true,
                                Intent::Commit,
                                p,
                            ))
                        })
                        .child(self.button(
                            "close-modal".into(),
                            l.text("close").into(),
                            true,
                            false,
                            Intent::Escape,
                            p,
                        )),
                )
                .into_any_element();
        }
        let drop_target = self.shell.clone();
        let mut body = div()
            .id("presenter-content")
            .flex_1()
            .min_h_0()
            .overflow_y_scroll()
            .p(px(24.))
            .gap(px(16.))
            .flex()
            .flex_col();
        if stage == SessionStage::Idle {
            body = body
                .child(
                    div()
                        .font(theme::display_font())
                        .text_size(px(28.))
                        .text_color(rgb(p.ink))
                        .child("FerroStage"),
                )
                .child(div().child(l.text("welcome")))
                .child(
                    self.button(
                        "welcome-open".into(),
                        if loading {
                            l.text("loading")
                        } else {
                            l.text("open")
                        }
                        .into(),
                        self.data.can_open,
                        true,
                        Intent::Open,
                        p,
                    ),
                );
        } else {
            body = body.child(self.workbench_panel(p, window, cx));
        }

        if let Some(error) = error {
            body = body.child(
                div()
                    .id("presenter-error")
                    .debug_selector(|| "presenter-error".into())
                    .role(Role::Alert)
                    .text_color(rgb(p.error))
                    .child(error),
            );
        }
        for warning in warnings {
            body = body.child(div().text_color(rgb(p.warning)).child(warning));
        }
        if dirty {
            body = body
                .child(
                    div().text_color(rgb(p.warning)).child(
                        save_error
                            .map(|error| l.error("save-error", error))
                            .unwrap_or_else(|| l.text("unsaved").into()),
                    ),
                )
                .child(self.button(
                    "retry-save".into(),
                    l.text("retry-save").into(),
                    true,
                    false,
                    Intent::RetrySave,
                    p,
                ));
        }
        if stage != SessionStage::Presenting && !recent.is_empty() {
            body = body.child(div().text_color(rgb(p.muted)).child(l.text("recent")));
            for (index, path) in recent.into_iter().enumerate() {
                let label = std::path::Path::new(&path)
                    .file_name()
                    .map(|name| name.to_string_lossy().into_owned())
                    .unwrap_or_else(|| path.clone());
                body = body.child(self.button(
                    format!("recent-{index}"),
                    label,
                    !busy,
                    false,
                    Intent::Import(path.into()),
                    p,
                ));
            }
        }
        root.track_focus(&self.focus)
            .on_drop::<ExternalPaths>(move |paths, _, cx| {
                if let Some(path) = paths.paths().first() {
                    actions::dispatch(&drop_target, Intent::Import(path.clone()), cx);
                }
            })
            .child(
                div()
                    .p(px(16.))
                    .border_b_1()
                    .border_color(rgb(p.border))
                    .flex()
                    .flex_col()
                    .gap(px(8.))
                    .child(
                        div()
                            .id("document-title")
                            .aria_label(filename.clone())
                            .tooltip(move |_, cx| {
                                cx.new(|_| FilenameTooltip {
                                    filename: tooltip_filename.clone(),
                                    palette: p,
                                })
                                .into()
                            })
                            .text_size(px(20.))
                            .text_color(rgb(p.ink))
                            .truncate()
                            .child(filename),
                    )
                    .child(div().text_color(rgb(p.muted)).child(stage_label))
                    .child(self.timer_panel(p))
                    .child(
                        div()
                            .flex()
                            .flex_wrap()
                            .gap(px(8.))
                            .child(self.button(
                                "open".into(),
                                l.text("open").into(),
                                self.data.can_open,
                                false,
                                Intent::Open,
                                p,
                            ))
                            .child(self.button(
                                "settings".into(),
                                l.text("settings").into(),
                                !busy,
                                false,
                                Intent::Settings,
                                p,
                            ))
                            .child(self.button(
                                "displays".into(),
                                l.text("displays").into(),
                                !busy,
                                false,
                                Intent::Displays,
                                p,
                            )),
                    ),
            )
            .child(body)
            .when(!self.data.workbench.steps.is_empty(), |view| {
                view.child(self.tools_panel(p))
            })
            .child(
                div()
                    .p(px(16.))
                    .border_t_1()
                    .border_color(rgb(p.border))
                    .flex()
                    .flex_col()
                    .gap(px(8.))
                    .child(if stage == SessionStage::Presenting {
                        self.button(
                            "stop".into(),
                            l.text("stop").into(),
                            true,
                            true,
                            Intent::Stop,
                            p,
                        )
                    } else {
                        self.button(
                            "start".into(),
                            if busy {
                                l.text("preparing")
                            } else {
                                l.text("start")
                            }
                            .into(),
                            reason.is_none(),
                            true,
                            Intent::Start,
                            p,
                        )
                    })
                    .when_some(
                        if stage != SessionStage::Presenting {
                            reason
                        } else {
                            None
                        },
                        |view, reason| view.child(div().text_color(rgb(p.muted)).child(reason)),
                    ),
            )
            .into_any_element()
    }
}
