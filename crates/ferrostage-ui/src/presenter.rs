mod home_view;
mod layout_navigation;
mod number_input;
mod page_browser;
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
    typography: theme::Typography,
    stage: SessionStage,
    filename: String,
    modal: Option<Modal>,
    browser: Option<page_browser::BrowserData>,
    layout_menu: Option<layout_navigation::LayoutMenuData>,
    number_focus: FocusHandle,
    displays: Option<crate::display_settings::DisplayPanel>,
    error: Option<String>,
    warnings: Vec<String>,
    start_reason: Option<String>,
    loading: bool,
    busy: bool,
    can_open: bool,
    dirty: bool,
    save_error: Option<String>,
    #[cfg(all(test, feature = "test-support"))]
    pages: usize,
    recent: Vec<String>,
    workbench: workbench_view::WorkbenchData,
    preferences: settings_view::SettingsData,
    staging: Option<crate::shell::reload::PresenterStaging>,
}

impl PresenterData {
    pub(crate) fn from_shell(shell: &Shell) -> Self {
        Self {
            theme: shell.settings.settings.general.theme,
            typography: theme::Typography::resolve(
                &shell.settings.settings.general,
                &shell.ui_fonts,
            ),
            staging: shell.presenter_staging(),
            #[cfg(all(test, feature = "test-support"))]
            pages: shell.state.page_count(),
            stage: shell.state.stage(),
            filename: shell
                .path
                .as_ref()
                .and_then(|path| path.file_name())
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_else(|| "FerroStage".into()),
            modal: shell.modal.clone(),
            browser: page_browser::BrowserData::from_shell(shell),
            layout_menu: layout_navigation::LayoutMenuData::from_shell(shell),
            number_focus: shell.number_input_focus.clone(),
            displays: shell.display_panel(),
            error: shell
                .graphics_error
                .as_ref()
                .map(|error| shell.locale.error("wb-upload-error", error))
                .or_else(|| shell.error.clone()),
            warnings: shell
                .warnings
                .iter()
                .cloned()
                .chain(layout_navigation::document_warnings(shell))
                .collect(),
            start_reason: shell.start_reason(),
            loading: shell.loading,
            busy: shell.busy(),
            can_open: shell.import_allowed() && !shell.busy(),
            dirty: shell.settings.dirty,
            save_error: shell.settings.save_error.as_ref().map(ToString::to_string),
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
    contextual_focus: FocusHandle,
    number_focus: FocusHandle,
    number_layout: number_input::NumberLayout,
    number_selecting: std::rc::Rc<std::cell::Cell<bool>>,
    contextual_anchor: std::rc::Rc<std::cell::Cell<gpui::Bounds<gpui::Pixels>>>,
    layout_anchor: std::rc::Rc<std::cell::Cell<gpui::Bounds<gpui::Pixels>>>,
    layout_trigger_focus: FocusHandle,
    layout_row_focus: [FocusHandle; layout_navigation::ROW_COUNT],
    layout_scroll: gpui::ScrollHandle,
    layout_row_anchors: [gpui::ScrollAnchor; layout_navigation::ROW_COUNT],
    had_layout_menu: bool,
    had_browser: Option<u64>,
    browser_scroll: gpui::ScrollHandle,
    had_browser_tile: Option<usize>,
    browser_tile_focus: [FocusHandle; 16],
    browser_tile_anchors: [gpui::ScrollAnchor; 16],
    progress_focus: FocusHandle,
    font_picker_focus: FocusHandle,
    had_font_picker: bool,
    home_focus: [FocusHandle; 7],
    home_scroll: gpui::ScrollHandle,
    home_anchors: [gpui::ScrollAnchor; 7],
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
    typography: theme::Typography,
}

impl Render for FilenameTooltip {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("presenter-text-tooltip")
            .debug_selector(|| "presenter-text-tooltip".into())
            .max_w(px(560.))
            .p(px(12.))
            .rounded(px(theme::RADIUS))
            .border_1()
            .border_color(rgb(self.palette.border))
            .bg(rgb(self.palette.card))
            .text_color(rgb(self.palette.ink))
            .font(self.typography.font())
            .text_size(px(self.typography.size(14.)))
            .whitespace_normal()
            .child(self.filename.clone())
    }
}

impl Presenter {
    #[cfg(all(test, feature = "test-support"))]
    pub(crate) fn seed_browser_failure(&mut self, index: usize, cx: &mut Context<Self>) {
        self.data.browser.as_mut().unwrap().failed_thumbnails = vec![index];
        cx.notify();
    }
    #[cfg(all(test, feature = "test-support"))]
    pub(crate) fn browser_focused_tile(&self, window: &Window) -> Option<usize> {
        self.browser_tile_focus
            .iter()
            .position(|focus| focus.is_focused(window))
    }
    #[cfg(all(test, feature = "test-support"))]
    pub(crate) fn font_picker_focused(&self, window: &Window) -> bool {
        self.font_picker_focus.is_focused(window)
    }
    #[cfg(all(test, feature = "test-support"))]
    pub(crate) fn typography_probe(&self) -> (String, f32, bool) {
        (
            self.data.typography.family.clone(),
            self.data.typography.size(14.),
            self.data.typography.missing,
        )
    }
    pub(crate) fn ui_text_size(&self, role: f32) -> f32 {
        self.data.typography.size(role)
    }
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
        let home_scroll = gpui::ScrollHandle::new();
        let home_anchors =
            std::array::from_fn(|_| gpui::ScrollAnchor::for_handle(home_scroll.clone()));
        let home_focus = std::array::from_fn(|_| cx.focus_handle());
        let browser_scroll = gpui::ScrollHandle::new();
        let browser_tile_anchors =
            std::array::from_fn(|_| gpui::ScrollAnchor::for_handle(browser_scroll.clone()));
        let browser_tile_focus = std::array::from_fn(|_| cx.focus_handle());
        let layout_scroll = gpui::ScrollHandle::new();
        let layout_row_anchors =
            std::array::from_fn(|_| gpui::ScrollAnchor::for_handle(layout_scroll.clone()));
        let layout_row_focus = std::array::from_fn(|_| cx.focus_handle());
        for (index, handle) in layout_row_focus.iter().enumerate() {
            cx.on_focus(handle, window, move |this, window, cx| {
                this.layout_row_anchors[index].scroll_to(window, cx);
                cx.notify();
            })
            .detach();
        }
        for (index, handle) in home_focus.iter().enumerate() {
            cx.on_focus(handle, window, move |this, window, cx| {
                this.home_anchors[index].scroll_to(window, cx);
                cx.notify();
            })
            .detach();
            cx.on_focus_out(handle, window, |_, _, _, cx| cx.notify())
                .detach();
        }
        let progress_focus = cx.focus_handle();
        cx.on_focus(&progress_focus, window, |_, _, cx| cx.notify())
            .detach();
        cx.on_focus_out(&progress_focus, window, |_, _, _, cx| cx.notify())
            .detach();
        let number_focus = data.number_focus.clone();
        Self {
            shell,
            data,
            locale,
            focus,
            modal_focus: cx.focus_handle(),
            contextual_focus: cx.focus_handle(),
            number_focus,
            number_layout: Default::default(),
            number_selecting: Default::default(),
            contextual_anchor: Default::default(),
            layout_anchor: Default::default(),
            layout_trigger_focus: cx.focus_handle(),
            layout_row_focus,
            layout_scroll,
            layout_row_anchors,
            had_layout_menu: false,
            had_browser: None,
            browser_scroll,
            had_browser_tile: None,
            browser_tile_focus,
            browser_tile_anchors,
            progress_focus,
            font_picker_focus: cx.focus_handle(),
            had_font_picker: false,
            home_focus,
            home_scroll,
            home_anchors,
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
        (
            &self.data.filename,
            self.data.workbench.overlay.current_page.0 + 1,
            self.data.pages,
        )
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
        let session = matches!(&intent, Intent::Start | Intent::Stop);
        let stop = matches!(&intent, Intent::Stop);
        let blank = match &intent {
            Intent::Black => Some(0x000000),
            Intent::White => Some(0xFFFFFF),
            _ => None,
        };
        let quiet_header = self.data.stage != SessionStage::Idle
            && matches!(
                &intent,
                Intent::Home | Intent::Open | Intent::Settings | Intent::Displays
            );
        let preference_control = matches!(
            &intent,
            Intent::SettingsCategory(_)
                | Intent::EditSetting(_, _)
                | Intent::BeginSettingEdit(_)
                | Intent::ToggleFontPicker
                | Intent::GridDensity { .. }
                | Intent::BrowserMode { .. }
        );
        let category_control = matches!(&intent, Intent::SettingsCategory(_));
        let kind = match &intent {
            _ if preference_control => theme::ButtonKind::Utility,
            Intent::Start => theme::ButtonKind::Primary,
            Intent::Stop | Intent::Core(ferrostage_core::PresentationCommand::ClearStrokes) => {
                theme::ButtonKind::Destructive
            }
            Intent::Next | Intent::Previous => theme::ButtonKind::Navigation,
            Intent::Black => theme::ButtonKind::Black,
            Intent::White => theme::ButtonKind::White,
            Intent::Tool(_) | Intent::Grid => theme::ButtonKind::Tool,
            _ if primary => theme::ButtonKind::Primary,
            _ => theme::ButtonKind::Utility,
        };
        let selected = match &intent {
            Intent::Black => {
                self.data.workbench.overlay.blank_mode == Some(ferrostage_core::BlankMode::Black)
            }
            Intent::White => {
                self.data.workbench.overlay.blank_mode == Some(ferrostage_core::BlankMode::White)
            }
            Intent::Tool(_) | Intent::Grid => primary,
            _ if preference_control => primary,
            _ => false,
        };
        let label = if selected {
            format!("✓ {label}")
        } else {
            label
        };
        let mut style = palette.button(kind, enabled, selected);
        if preference_control {
            style.fill = if selected {
                palette.card
            } else if category_control {
                palette.canvas
            } else {
                palette.soft
            };
            style.border = style.fill;
            style.ink = if enabled { palette.body } else { palette.muted };
        }
        if quiet_header {
            style.fill = palette.canvas;
            style.border = palette.canvas;
        }
        self.activation_target(id.clone(), label.clone(), enabled, intent, palette)
            .rounded(px(style.radius + theme::FOCUS_GAP))
            .child(
                div()
                    .id(SharedString::from(format!("{id}-face")))
                    .min_h(px(if session {
                        theme::SESSION_CONTROL
                    } else {
                        theme::CONTROL
                    }))
                    .px(px(if quiet_header {
                        8.
                    } else if session
                        || matches!(
                            kind,
                            theme::ButtonKind::Navigation
                                | theme::ButtonKind::Black
                                | theme::ButtonKind::White
                                | theme::ButtonKind::Tool
                        )
                    {
                        12.
                    } else {
                        theme::SPACING
                    }))
                    .flex()
                    .items_center()
                    .justify_center()
                    .gap(px(8.))
                    .relative()
                    .rounded(px(style.radius))
                    .bg(rgb(style.fill))
                    .text_color(rgb(style.ink))
                    .border_1()
                    .border_color(rgb(style.border))
                    .min_w_0()
                    .when(session, |view| {
                        view.min_w(px(self.data.typography.size(theme::SESSION_WIDTH)))
                    })
                    .text_size(px(self.data.typography.size(14.)))
                    .when(session, |view| {
                        view.child(Self::session_icon(stop, style.ink))
                    })
                    .when_some(blank, |view, color| {
                        view.child(
                            div()
                                .size(px(14.))
                                .flex_shrink_0()
                                .rounded(px(3.))
                                .border_1()
                                .border_color(rgb(palette.border))
                                .bg(rgb(color)),
                        )
                    })
                    .child(div().when(!session, |view| view.truncate()).child(label))
                    .when(enabled, |view| {
                        view.hover(move |state| state.bg(rgb(style.hover)))
                            .active(move |state| state.bg(rgb(style.pressed)).top(px(1.)))
                    }),
            )
    }

    /// Shared event/focus boundary; each caller retains its own visual face.
    fn activation_target(
        &self,
        id: String,
        label: String,
        enabled: bool,
        intent: Intent,
        palette: Palette,
    ) -> Stateful<Div> {
        let target = self.shell.clone();
        let keyboard = target.clone();
        let a11y = target.clone();
        let click = intent.clone();
        let accessible = intent.clone();
        let page_control = id.starts_with("page-")
            || id.starts_with("thumbnail-")
            || matches!(
                id.as_str(),
                "grid-prev" | "grid-next" | "commit-jump" | "close-browser"
            );
        let anchor = if id == "document-options" {
            self.layout_anchor.clone()
        } else {
            self.contextual_anchor.clone()
        };
        let measure_anchor = matches!(id.as_str(), "workbench-progress" | "document-options");
        let home_index = (self.data.stage == SessionStage::Idle)
            .then(|| {
                [
                    "settings",
                    "displays",
                    "welcome-open",
                    "recent-0",
                    "recent-1",
                    "recent-2",
                    "retry-save",
                ]
                .iter()
                .position(|candidate| *candidate == id)
            })
            .flatten();
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
            .tab_stop(
                enabled
                    && (self.data.browser.is_none() || page_control)
                    && (self.data.layout_menu.is_none() || id.starts_with("layout-row-")),
            )
            .border(px(theme::FOCUS_WIDTH))
            .border_color(rgb(palette.canvas))
            .rounded(px(theme::RADIUS + theme::FOCUS_GAP))
            .p(px(theme::FOCUS_GAP))
            .focus(|style| style.border_color(rgb(palette.focus)))
            .when(id == "document-options", |view| {
                view.aria_expanded(self.data.layout_menu.is_some())
                    .track_focus(&self.layout_trigger_focus.clone().tab_stop(
                        enabled && self.data.browser.is_none() && self.data.layout_menu.is_none(),
                    ))
            })
            .when_some(home_index, |view, index| {
                view.track_focus(&self.home_focus[index].clone().tab_stop(enabled))
                    .anchor_scroll(Some(self.home_anchors[index].clone()))
            })
            .when(measure_anchor, |view| {
                view.child(
                    gpui::canvas(
                        |_, _, _| (),
                        move |bounds, _, _, _| {
                            anchor.set(bounds.dilate(px(theme::FOCUS_WIDTH + theme::FOCUS_GAP)))
                        },
                    )
                    .absolute()
                    .size_full(),
                )
            })
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

    fn command_bar(&self, p: Palette, window: &Window) -> gpui::AnyElement {
        let l = self.locale;
        let stage = self.data.stage;
        let busy = self.data.busy;
        let stage_label = match stage {
            SessionStage::Idle => l.text("stage-idle"),
            SessionStage::Ready => l.text("stage-ready"),
            SessionStage::Presenting => l.text("stage-presenting"),
        };
        let filename = self.data.filename.clone();
        let narrow = window.viewport_size().width < px(800.);
        let control = px(theme::CONTROL + (theme::FOCUS_WIDTH + theme::FOCUS_GAP) * 2.);
        let tooltip_filename = filename.clone();
        let typography = self.data.typography.clone();
        let identity = div()
            .flex()
            .items_center()
            .gap(px(12.))
            .min_w_0()
            .flex_1()
            .when_some(self.clock_readout(p), |view, clock| view.child(clock))
            .child(
                div()
                    .id("document-title")
                    .debug_selector(|| "document-title".into())
                    .aria_label(filename.clone())
                    .tooltip(move |_, cx| {
                        cx.new(|_| FilenameTooltip {
                            filename: tooltip_filename.clone(),
                            palette: p,
                            typography: typography.clone(),
                        })
                        .into()
                    })
                    .min_h(control)
                    .min_w_0()
                    .flex()
                    .items_center()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_size(px(self.data.typography.size(16.)))
                    .text_color(rgb(p.ink))
                    .truncate()
                    .child(filename.clone()),
            )
            .child(
                div()
                    .id("stage-label")
                    .debug_selector(|| "stage-label".into())
                    .flex_shrink_0()
                    .px(px(8.))
                    .py(px(4.))
                    .rounded(px(4.))
                    .bg(rgb(p.soft))
                    .text_size(px(self.data.typography.size(12.)))
                    .text_color(rgb(p.muted))
                    .child(stage_label),
            );
        let timer = div()
            .flex_shrink_0()
            .min_h(control)
            .flex()
            .items_center()
            .child(self.timer_panel(p));
        let mut actions = div()
            .id("presenter-actions")
            .debug_selector(|| "presenter-actions".into())
            .flex()
            .items_center()
            .gap(px(8.))
            .flex_shrink_0();
        if stage != SessionStage::Idle {
            actions = actions.child(self.button(
                "home".into(),
                l.text("home").into(),
                !busy,
                false,
                Intent::Home,
                p,
            ));
        }
        actions = actions
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
            ));
        let mut bar = div()
            .id("presenter-bar")
            .debug_selector(|| "presenter-bar".into())
            .px(px(16.))
            .pt(px(8.))
            .pb(px(8.))
            .border_b_1()
            .border_color(rgb(p.border))
            .flex()
            .items_center()
            .gap(px(12.));
        if narrow {
            bar = bar.flex_col().child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(12.))
                    .min_w_0()
                    .w_full()
                    .child(identity)
                    .child(timer),
            );
        } else {
            bar = bar.child(identity).child(timer);
        }
        bar.child(actions).into_any_element()
    }
}

impl Render for Presenter {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let picker_open = self.data.preferences.panel.font_picker.is_some();
        if self.had_font_picker && !picker_open && self.data.modal == Some(Modal::Settings) {
            window.focus(&self.font_picker_focus, cx);
        }
        self.had_font_picker = picker_open;
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
        let browser_id = self.data.browser.as_ref().map(|data| data.browser.id);
        if browser_id != self.had_browser {
            // Thumbnail indexes acquire a new page meaning across browser identities.
            // Retire their rollback images before syncing any replacement frames.
            let retained_frames = frames
                .iter()
                .filter(|(surface, _)| !matches!(surface, crate::workbench::Surface::Thumbnail(_)))
                .cloned()
                .collect::<Vec<_>>();
            let retained_surfaces = active
                .iter()
                .copied()
                .filter(|surface| !matches!(surface, crate::workbench::Surface::Thumbnail(_)))
                .collect::<Vec<_>>();
            self.textures.borrow_mut().sync(
                &retained_frames,
                &retained_surfaces,
                self.data.workbench.budget,
                window,
                cx,
            );
        }
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
        let modal = self.data.modal.clone();
        let error = self
            .textures
            .borrow()
            .error()
            .map(|error| l.error("wb-upload-error", error))
            .or_else(|| self.data.error.clone());
        let warnings = self.data.warnings.clone();
        let dirty = self.data.dirty;
        let save_error = self.data.save_error.clone();
        if stage == SessionStage::Idle {
            window.set_window_title("FerroStage");
        } else {
            window.set_window_title(&format!("{filename} — {stage_label} — FerroStage"));
        }
        let layout_open = self.data.layout_menu.is_some();
        let surface_open = modal.is_some() || self.data.browser.is_some() || layout_open;
        if surface_open && !self.had_modal {
            self.return_focus = if layout_open {
                Some(self.layout_trigger_focus.clone())
            } else {
                window.focused(cx)
            };
            window.focus(
                if layout_open {
                    &self.layout_row_focus
                        [self.data.layout_menu.as_ref().unwrap().selected_layout()]
                } else if self.data.browser.is_some() {
                    &self.number_focus
                } else {
                    &self.modal_focus
                },
                cx,
            );
        }
        if surface_open
            && self.had_modal
            && (layout_open != self.had_layout_menu || browser_id != self.had_browser)
        {
            if let Some(menu) = &self.data.layout_menu {
                self.return_focus = Some(self.layout_trigger_focus.clone());
                window.focus(&self.layout_row_focus[menu.selected_layout()], cx);
            } else if self.data.browser.is_some() {
                self.return_focus = Some(self.progress_focus.clone());
                window.focus(&self.number_focus, cx);
            } else {
                window.focus(&self.modal_focus, cx);
            }
        }
        self.had_layout_menu = layout_open;
        self.had_browser = browser_id;
        let focused_tile = browser_id.and_then(|_| {
            self.browser_tile_focus
                .iter()
                .position(|focus| focus.is_focused(window))
        });
        if focused_tile != self.had_browser_tile {
            self.had_browser_tile = focused_tile;
            if let Some(index) = focused_tile {
                self.browser_tile_anchors[index].scroll_to(window, cx);
            }
        }
        // Keep the return handle alive until the deferred tree check owns it.
        // A handle alone does not mean its former button is still rendered.
        let return_focus = if !surface_open && self.had_modal {
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
        self.had_modal = surface_open;
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
                    let fallback = if presenter.data.browser.is_some()
                        || presenter.data.layout_menu.is_some()
                    {
                        presenter.contextual_focus.clone()
                    } else if presenter.had_modal {
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
            .relative()
            .flex()
            .flex_col()
            .bg(rgb(p.canvas))
            .text_color(rgb(p.body))
            .font(self.data.typography.font())
            .text_size(px(self.data.typography.size(16.)))
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
            .capture_key_down(cx.listener(|this, event: &gpui::KeyDownEvent, window, cx| {
                if this.layout_menu_key(event, window, cx) {
                    cx.stop_propagation();
                    return;
                }
                if this.number_key(event, window, cx) {
                    cx.stop_propagation();
                    return;
                }
                if this.data.modal == Some(Modal::Settings)
                    && (this.data.preferences.panel.recording.is_some()
                        || this.data.preferences.panel.edit.is_some()
                        || this.data.preferences.panel.font_picker.is_some())
                {
                    let key = event.keystroke.unparse();
                    let text = if this.data.preferences.panel.font_picker.is_some()
                        && matches!(key.as_str(), "ctrl-v" | "cmd-v" | "shift-insert")
                    {
                        // Paste providers may restore the clipboard before deferred dispatch.
                        cx.read_from_clipboard().and_then(|item| item.text())
                    } else {
                        event.keystroke.key_char.clone().or_else(|| {
                            (event.keystroke.key.len() == 1).then(|| event.keystroke.key.clone())
                        })
                    };
                    actions::dispatch(&this.shell, Intent::SettingsKey { key, text }, cx);
                    cx.stop_propagation();
                }
            }))
            .on_key_down(cx.listener(|_, event: &gpui::KeyDownEvent, window, cx| {
                let key = event.keystroke.key.as_str();
                if key == "tab" {
                    if event.keystroke.modifiers.shift {
                        window.focus_prev(cx);
                    } else {
                        window.focus_next(cx);
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
            if let Some(panel) = &self.data.displays {
                return root
                    .track_focus(&self.modal_focus)
                    .key_context("FerroStageModal")
                    .child(crate::display_settings::view::panel(
                        self,
                        panel,
                        l,
                        p,
                        false,
                        f32::from(window.viewport_size().width) - 32.,
                    ))
                    .into_any_element();
            }
            unreachable!("all modal drawing branches handled above");
        }
        let drop_target = self.shell.clone();
        let document_open = stage != SessionStage::Idle;
        let mut body = if document_open {
            div()
                .id("presenter-content")
                .flex_1()
                .min_h_0()
                .p(px(12.))
                .gap(px(8.))
                .flex()
                .flex_col()
                .overflow_hidden()
                .child(
                    div()
                        .flex_1()
                        .min_h_0()
                        .flex()
                        .flex_col()
                        .child(self.workbench_panel(p, window, cx)),
                )
        } else {
            self.home_content(p, window)
        };

        if let Some(error) = error {
            body = body.child(
                div()
                    .id("presenter-error")
                    .debug_selector(|| "presenter-error".into())
                    .flex_shrink_0()
                    .role(Role::Alert)
                    .text_color(rgb(p.error))
                    .child(error),
            );
        }
        for warning in warnings {
            body = body.child(
                div()
                    .flex_shrink_0()
                    .text_color(rgb(p.warning))
                    .child(warning),
            );
        }
        if dirty {
            body = body
                .child(
                    div().flex_shrink_0().text_color(rgb(p.warning)).child(
                        save_error
                            .map(|error| l.error("save-error", error))
                            .unwrap_or_else(|| l.text("unsaved").into()),
                    ),
                )
                .child(div().flex_shrink_0().child(self.button(
                    "retry-save".into(),
                    l.text("retry-save").into(),
                    true,
                    false,
                    Intent::RetrySave,
                    p,
                )));
        }
        let body = if document_open {
            body.into_any_element()
        } else {
            div()
                .id("presenter-content")
                .flex_1()
                .min_h_0()
                .min_w_0()
                .overflow_y_scroll()
                .track_scroll(&self.home_scroll)
                .flex()
                .flex_col()
                .items_center()
                .px(px(if window.viewport_size().width < px(800.) {
                    16.
                } else {
                    32.
                }))
                .pt(px(if window.viewport_size().height < px(600.) {
                    24.
                } else {
                    48.
                }))
                .pb(px(24.))
                .child(body.flex_shrink_0())
                .into_any_element()
        };
        root.track_focus(&self.focus)
            .on_drop::<ExternalPaths>(move |paths, _, cx| {
                if let Some(path) = paths.paths().first() {
                    actions::dispatch(&drop_target, Intent::Import(path.clone()), cx);
                }
            })
            .when(document_open, |view| {
                view.child(self.command_bar(p, window))
            })
            .child(body)
            .when(document_open, |view| {
                view.child(self.tools_panel(p, window))
            })
            .when(self.data.browser.is_some(), |view| {
                view.child(self.contextual_host(
                    self.page_browser_panel(p, window, cx),
                    self.contextual_anchor.get(),
                    gpui::Anchor::BottomLeft,
                ))
            })
            .when(layout_open, |view| {
                view.child(self.contextual_host(
                    self.layout_menu_panel(p, window),
                    self.layout_anchor.get(),
                    gpui::Anchor::BottomRight,
                ))
            })
            .into_any_element()
    }
}
