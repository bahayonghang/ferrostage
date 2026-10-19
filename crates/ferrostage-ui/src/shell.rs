mod layout_navigation;
mod page_browser;
mod preferences;
mod presenter_fullscreen;
pub(crate) mod reload;
mod timed_playback;
use crate::{
    actions::{self, Intent},
    audience::Audience,
    display_settings::{self, DisplayDraft, DisplayPanel},
    identify::Identify,
    policy::{self, ShellError},
    presenter::{Presenter, PresenterData},
    transactions::{self, Operation, Shutdown},
};
use ferrostage_core::{
    AudienceProjectionSnapshot, AudienceStage, BlankMode, MonitorId, PhysicalPage,
    PresentationCommand as Command, PresentationState, SessionStage,
};
#[cfg(all(test, feature = "test-support"))]
use ferrostage_pdf::DocumentMetadata;
use ferrostage_platform::{MonitorInfo, TopologyEvent, TopologyWatcher};
use ferrostage_render::{RenderError, RenderHandle};
use ferrostage_settings::{DisplaySettings, SettingsSession, SettingsStore, WritePolicy};
use gpui::{
    AnyWindowHandle, App, AppContext, Bounds, Context, DisplayId, Entity, Global,
    PathPromptOptions, Pixels, TitlebarOptions, WindowBounds, WindowId, WindowOptions, point, px,
    size,
};
use std::{
    cell::Cell,
    collections::BTreeMap,
    path::PathBuf,
    rc::Rc,
    sync::mpsc::Receiver,
    time::{Duration, Instant},
};

const CONFIRM_TIMEOUT: Duration = Duration::from_secs(3);

#[derive(Debug, thiserror::Error)]
enum ImportReadError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Structure(#[from] ferrostage_pdf::PdfStructureError),
}

impl ImportReadError {
    fn message(&self, locale: crate::locale::Locale) -> String {
        use ferrostage_pdf::PdfStructureError;
        let key = match self {
            Self::Io(error) => match (error.kind(), error.raw_os_error()) {
                // Windows sharing/lock violations often have PermissionDenied kind.
                (_, Some(32 | 33)) => "read-locked",
                (std::io::ErrorKind::NotFound, _) => "read-missing",
                (std::io::ErrorKind::PermissionDenied, _) => "read-permission",
                _ => "read-failed",
            },
            Self::Structure(PdfStructureError::Unreadable) => "read-incomplete",
            Self::Structure(PdfStructureError::InvalidPageGeometry) => "read-geometry",
            Self::Structure(PdfStructureError::UnsupportedUserUnit) => "read-user-unit",
        };
        locale.text(key).into()
    }
}

/// The binary composes services; all GPUI objects stay in this crate.
pub struct ShellInputs {
    pub settings: SettingsSession,
    pub store: SettingsStore,
    pub renderer: Result<RenderHandle, RenderError>,
    pub topology: Receiver<TopologyEvent>,
    pub watcher: Result<TopologyWatcher, ferrostage_platform::PlatformError>,
    pub initial_monitors: Result<Vec<MonitorInfo>, ferrostage_platform::PlatformError>,
    pub initial_pdf: Option<PathBuf>,
}

struct ShellRoot {
    _entity: Entity<Shell>,
}
impl Global for ShellRoot {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Modal {
    Settings,
    Displays(DisplayDraft),
}

struct WindowRecord {
    handle: AnyWindowHandle,
    monitor: MonitorId,
    display: u64,
    fullscreen: bool,
    client_size: gpui::Size<Pixels>,
    resize_attempt: Cell<Option<ClientResizeAttempt>>,
    bounds_observed: Rc<Cell<bool>>,
    observed_display: Rc<Cell<Option<u64>>>,
}

#[derive(Clone, Copy)]
struct ClientResizeAttempt {
    requested: gpui::Size<Pixels>,
    before: gpui::Size<Pixels>,
    corrected: bool,
}

enum TransactionKind {
    Start,
    Assign(DisplaySettings),
    Preferences(DisplaySettings),
    RecoverPresenter,
}
impl TransactionKind {
    fn operation(&self) -> Operation {
        match self {
            Self::Start => Operation::Start,
            Self::Assign(_) | Self::Preferences(_) => Operation::Swap,
            Self::RecoverPresenter => Operation::RecoverPresenter,
        }
    }
}
struct Transaction {
    kind: TransactionKind,
    presenter: Option<WindowRecord>,
    audience: Option<WindowRecord>,
    deadline: Instant,
}

struct FocusIntent {
    intent: Intent,
    open_rejected: bool,
    presenter: WindowId,
    deadline: Instant,
}
struct ImportReply {
    token: u64,
    path: PathBuf,
    load: crate::document_load::PendingLoad,
}

pub(crate) struct Shell {
    pub locale: crate::locale::Locale,
    pub state: PresentationState,
    pub workbench: crate::workbench::Workbench,
    pub media: crate::media_playback::Playback,
    next_generation: u64,
    pub settings: SettingsSession,
    pub preferences: crate::settings_panel::PanelState,
    pub ui_fonts: std::sync::Arc<Vec<String>>,
    ui_fonts_loaded: bool,
    preference_change: Option<preferences::PendingPreferences>,
    store: SettingsStore,
    renderer: Result<RenderHandle, RenderError>,
    topology: Receiver<TopologyEvent>,
    _watcher: Result<TopologyWatcher, ferrostage_platform::PlatformError>,
    monitors: Vec<MonitorInfo>,
    monitor_error: Option<ferrostage_platform::PlatformError>,
    topology_valid: bool,
    topology_revision: u64,
    identify: Vec<AnyWindowHandle>,
    identify_deadline: Option<Instant>,
    recover_windowed: bool,
    mapped: Result<BTreeMap<MonitorId, u64>, ShellError>,
    display_details: std::collections::BTreeSet<MonitorId>,
    presenter: Option<WindowRecord>,
    audience: Option<WindowRecord>,
    pending: Option<Transaction>,
    presenter_restore: Option<presenter_fullscreen::Restore>,
    timed: timed_playback::TimedPlayback,
    focus_intent: Option<FocusIntent>,
    focus_token: u64,
    import: Option<ImportReply>,
    reload: reload::Reload,
    import_token: u64,
    pub loading: bool,
    pub path: Option<PathBuf>,
    pub error: Option<String>,
    pub graphics_error: Option<String>,
    pub warnings: Vec<String>,
    pub modal: Option<Modal>,
    pub contextual: Option<crate::page_browser::ContextualSurface>,
    contextual_serial: u64,
    pub number_input_focus: gpui::FocusHandle,
    pub snapshot: Entity<AudienceProjectionSnapshot>,
    escape_stops: bool,
    poll_task: Option<gpui::Task<()>>,
    import_read_task: Option<gpui::Task<()>>,
    path_prompt_task: Option<gpui::Task<()>>,
}

pub fn run(inputs: ShellInputs) {
    gpui_platform::application().run(move |cx| {
        let initial = inputs.initial_pdf.clone().or_else(|| {
            inputs
                .settings
                .settings
                .general
                .reopen_last_document
                .then(|| {
                    inputs
                        .settings
                        .settings
                        .general
                        .recent_files
                        .first()
                        .map(PathBuf::from)
                })
                .flatten()
        });
        let shell = cx.new(|cx| Shell::new(inputs, cx));
        cx.set_global(ShellRoot {
            _entity: shell.clone(),
        });
        let keys = shell.read(cx).settings.settings.keybindings.clone();
        let warnings = actions::install(shell.downgrade(), &keys, shell.read(cx).locale, cx);
        shell.update(cx, |shell, cx| {
            shell.warnings.extend(warnings);
            shell.refresh_mapping(cx);
            shell.recover_presenter(cx);
            if let Some(path) = initial {
                shell.begin_import(path, cx);
            }
            shell.start_polling(cx);
        });
        install_close_handler(&shell, cx);
        cx.activate(true);
    });
}

fn install_close_handler(shell: &Entity<Shell>, cx: &mut App) {
    // Topology recovery may temporarily leave no windows. Only closing the
    // current Presenter asks the coordinator to quit.
    cx.set_quit_mode(gpui::QuitMode::Explicit);
    let weak = shell.downgrade();
    cx.on_window_closed(move |cx, id| {
        let weak = weak.clone();
        // remove_window invokes this hook before the calling Shell lease ends.
        // Classify the original ID after that lease, against current handles.
        cx.defer(move |cx| {
            if let Err(error) = weak.update(cx, |shell, cx| shell.closed(id, cx)) {
                tracing::debug!(%error,"closed retired shell");
            }
        });
    })
    .detach();
}

impl Shell {
    fn start_polling(&mut self, cx: &mut Context<Self>) {
        self.poll_task = Some(cx.spawn(async move |weak, cx| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(25))
                    .await;
                if weak.update(cx, |shell, cx| shell.tick(cx)).is_err() {
                    break;
                }
            }
        }));
    }

    fn stop_tasks(&mut self, reason: &'static str) {
        self.media.shutdown();
        self.shutdown_reload();
        // WeakEntity is checked only after a foreground wake. Cancel while the
        // platform dispatcher HWND is still alive so a late background timer
        // cannot enqueue another Shell wake after platform destruction.
        if self.poll_task.take().is_some() {
            tracing::debug!(reason, "shell polling task cancelled before quit");
        }
        drop(self.import_read_task.take());
        drop(self.path_prompt_task.take());
    }

    fn new(inputs: ShellInputs, cx: &mut Context<Self>) -> Self {
        cx.on_app_quit(|shell, _| {
            shell.stop_tasks("app shutdown");
            async {}
        })
        .detach();
        let locale = crate::locale::Locale::new(&inputs.settings.settings.general.language);
        let state = PresentationState::idle();
        let snapshot = cx.new(|_| state.audience_snapshot());
        let (topology_valid, monitors, monitor_error) = match inputs.initial_monitors {
            Ok(monitors) => (true, monitors, None),
            Err(error) => {
                tracing::warn!(
                    ?error,
                    origin = "startup discovery",
                    "monitor discovery unavailable"
                );
                (false, Vec::new(), Some(error))
            }
        };
        let warnings = inputs
            .settings
            .warnings
            .iter()
            .map(|warning| format!("{}：{}", warning.key, warning.reason))
            .collect();
        if let Err(error) = &inputs.watcher {
            tracing::warn!(
                ?error,
                origin = "watcher startup",
                "monitor watcher unavailable"
            );
        }
        let escape_stops = policy::bindings(&inputs.settings.settings.keybindings, locale)
            .0
            .iter()
            .any(|(key, action)| key == "escape" && *action == "stop_presentation");
        let recover_windowed = !topology_valid || monitors.is_empty();
        // A saved override needs resolution before first draw. Otherwise defer
        // enumeration until the family selector opens, then cache for the session.
        let ui_fonts_loaded = !inputs.settings.settings.general.ui_font_family.is_empty();
        let ui_fonts = std::sync::Arc::new(if ui_fonts_loaded {
            cx.text_system().all_font_names()
        } else {
            Vec::new()
        });
        Self {
            locale,
            state,
            workbench: Default::default(),
            media: Default::default(),
            next_generation: 1,
            settings: inputs.settings,
            store: inputs.store,
            preferences: Default::default(),
            ui_fonts,
            ui_fonts_loaded,
            preference_change: None,
            renderer: inputs.renderer,
            topology: inputs.topology,
            _watcher: inputs.watcher,
            monitors,
            monitor_error,
            topology_valid,
            topology_revision: 0,
            identify: Vec::new(),
            identify_deadline: None,
            recover_windowed,
            mapped: Err(ShellError::AmbiguousDisplays),
            display_details: Default::default(),
            presenter: None,
            audience: None,
            pending: None,
            presenter_restore: None,
            timed: Default::default(),
            focus_intent: None,
            focus_token: 0,
            import: None,
            reload: Default::default(),
            import_token: 0,
            loading: false,
            path: None,
            error: None,
            graphics_error: None,
            warnings,
            modal: None,
            contextual: None,
            contextual_serial: 0,
            number_input_focus: cx.focus_handle(),
            snapshot,
            escape_stops,
            poll_task: None,
            import_read_task: None,
            path_prompt_task: None,
        }
    }

    fn refresh_mapping(&mut self, cx: &mut App) {
        for warning in policy::roles(&self.monitors, &self.settings.settings.display).warnings {
            tracing::warn!(?warning, "display role resolution warning");
        }
        self.mapped = if self.topology_valid {
            let displays = cx
                .displays()
                .into_iter()
                .map(|display| display.uuid().map(|uuid| (u64::from(display.id()), uuid)))
                .collect::<Result<Vec<_>, _>>();
            displays
                .map_err(|_| ShellError::AmbiguousDisplays)
                .and_then(|displays| policy::map_displays(&self.monitors, &displays))
        } else {
            Err(ShellError::AmbiguousDisplays)
        };
        tracing::info!(monitors=?self.monitors, mapping=?self.mapped, "GPUI/platform display identity comparison");
        self.sample_window_locations(cx);
    }

    pub fn start_reason(&self) -> Option<String> {
        if self.state.stage() != SessionStage::Ready {
            return Some(self.locale.text("need-document").into());
        }
        if self.busy() {
            return Some(self.locale.text("confirming").into());
        }
        if self._watcher.is_err() {
            return Some(self.locale.text("watcher-unavailable").into());
        }
        if let Err(error) = &self.mapped {
            return Some(error.localized(self.locale));
        }
        policy::audience_target(&self.monitors, &self.settings.settings.display)
            .err()
            .map(|e| e.localized(self.locale))
    }

    pub(crate) fn monitor_error_text(&self) -> Option<String> {
        self.monitor_error
            .as_ref()
            .or_else(|| self._watcher.as_ref().err())
            .map(|error| self.locale.monitor_error(error))
    }

    pub fn busy(&self) -> bool {
        self.pending.is_some()
            || self.restoring_presenter()
            || self.loading
            || self.preference_change.is_some()
            || self.reload_busy()
    }

    fn publish(&mut self, curtain: bool, cx: &mut Context<Self>) {
        self.sync_playback(curtain, cx);
        self.media.sync(
            &self.state,
            self.workbench.document.as_ref().map(|s| &s.document),
            self.path.as_deref(),
            self.settings.settings.media.max_decode_height,
            curtain,
        );
        let mut snapshot = self.state.audience_snapshot();
        if curtain {
            snapshot.stage = AudienceStage::Curtain;
            snapshot.visual = None;
        }
        let mut snapshot = crate::workbench::assemble(
            snapshot,
            self.snapshot.read(cx),
            self.workbench.frame(crate::workbench::Surface::Audience),
            self.workbench.audience_target,
            self.settings.settings.cache.l1_gpu_mib as usize * 1024 * 1024 / 2,
        );
        if let Some(visual) = &mut snapshot.visual {
            visual.media = if snapshot.generation == self.state.generation()
                && snapshot.current_page == self.state.current_page()
                && snapshot.slide_crop == self.state.slide_crop()
            {
                self.media.frames(
                    &self.state,
                    self.workbench.document.as_ref().map(|s| &s.document),
                )
            } else {
                vec![]
            };
        }
        snapshot.staging = self.audience_staging();
        self.attach_playback(&mut snapshot);
        self.snapshot.update(cx, |value, cx| {
            *value = snapshot;
            cx.notify();
        });
    }

    fn apply(&mut self, command: Command, cx: &mut Context<Self>) {
        let before = (
            self.state.generation(),
            self.state.current_page(),
            self.state.slide_crop(),
        );
        let sequential = matches!(command, Command::NextStep | Command::PreviousStep);
        let jump = matches!(command, Command::JumpTo(_));
        let annotation = matches!(
            command,
            Command::UpdateLaserPointer(Some(_))
                | Command::BeginStroke { .. }
                | Command::AppendStrokePoint(_)
                | Command::EraseStrokeAt(_)
                | Command::ClearStrokes
        );
        if let Err(error) = self.state.apply_at(command, Instant::now()) {
            self.error = Some(error.to_string());
        } else {
            let after = (
                self.state.generation(),
                self.state.current_page(),
                self.state.slide_crop(),
            );
            if before != after || jump {
                self.select_playback(sequential);
            }
            if annotation {
                self.pause_for_annotation();
            }
        }
        self.publish(self.pending.is_some(), cx);
        cx.notify();
    }

    pub fn dispatch(&mut self, intent: Intent, cx: &mut Context<Self>) {
        if self.dispatch_preference(&intent, cx) {
            cx.notify();
            return;
        }
        if self.modal == Some(Modal::Settings)
            && matches!(
                intent,
                Intent::Start
                    | Intent::Stop
                    | Intent::Swap
                    | Intent::Next
                    | Intent::Previous
                    | Intent::Black
                    | Intent::White
                    | Intent::Tool(_)
            )
        {
            return;
        }
        if matches!(
            self.contextual,
            Some(crate::page_browser::ContextualSurface::PresenterMenu { .. })
        ) && matches!(
            intent,
            Intent::Home
                | Intent::Open
                | Intent::Settings
                | Intent::Tool(_)
                | Intent::Core(Command::ClearStrokes)
        ) {
            self.close_contextual();
        }
        match intent {
            Intent::ToggleAutoplay
                if self.has_authored_timing() && self.state.stage() == SessionStage::Presenting =>
            {
                self.timed
                    .clock
                    .pause(!self.timed.clock.paused, Instant::now());
            }
            Intent::PresenterMenu(tools) if !self.busy() => {
                if matches!(self.contextual, Some(crate::page_browser::ContextualSurface::PresenterMenu { tools: current }) if current == tools)
                {
                    self.close_contextual();
                } else {
                    self.abandon_modal();
                    self.contextual =
                        Some(crate::page_browser::ContextualSurface::PresenterMenu { tools });
                }
            }
            Intent::Media(id) => {
                if self.modal.is_none() {
                    self.media.select(
                        id,
                        &self.state,
                        self.workbench.document.as_ref().map(|s| &s.document),
                        self.path.as_deref(),
                        self.settings.settings.media.max_decode_height,
                    );
                    self.publish(self.pending.is_some(), cx);
                    cx.notify();
                }
            }
            Intent::SurfaceSize(surface, target) => {
                if self.workbench.surface_targets.get(&surface) != Some(&target) {
                    self.workbench.surface_targets.insert(surface, target);
                } else {
                    return;
                }
            }
            Intent::Core(command)
                if self.pending.is_none() && self.modal.is_none() && self.contextual.is_none() =>
            {
                self.apply(command, cx)
            }
            Intent::Tool(tool) => {
                self.workbench.tool = if self.workbench.tool == tool {
                    crate::workbench::Tool::Navigate
                } else {
                    tool
                };
                if self.workbench.tool != crate::workbench::Tool::Navigate {
                    self.pause_for_annotation();
                }
                self.apply(Command::EndStroke, cx);
                self.apply(Command::UpdateLaserPointer(None), cx);
            }
            Intent::Grid if !self.busy() => {
                if self.browser().is_some() {
                    self.close_contextual();
                } else {
                    self.open_browser();
                }
            }
            Intent::CloseContextual => self.close_contextual(),
            Intent::BrowserSubmit { id } => self.submit_browser(id, None, cx),
            Intent::BrowserMode { id, mode } => self.browser_mode(id, mode),
            Intent::BrowserView { id, view } => self.browser_view(id, view),
            Intent::BrowserOutlineJump { id, entry_id } => {
                self.browser_outline_jump(id, entry_id, cx)
            }
            Intent::BrowserEdit { id, edit } => self.edit_browser(id, edit),
            Intent::BrowserJump { id, page } if self.workbench.grid => {
                self.submit_browser(id, Some(page), cx)
            }
            Intent::GridDensity { column, value } => self.grid_density(column, value, cx),
            Intent::GridMove(delta)
                if self.browser().is_some() && self.workbench.grid && !self.busy() =>
            {
                let count = self.browser_entries().len();
                let capacity = self.settings.settings.presenter.grid_visible_pages as usize;
                let last = count.saturating_sub(1) / capacity * capacity;
                self.workbench.grid_start = self
                    .workbench
                    .grid_start
                    .saturating_add_signed(delta)
                    .min(last);
            }
            Intent::Document if !self.busy() => {
                if self.layout_menu_open() {
                    self.close_contextual();
                } else if self.workbench.document.is_some() {
                    self.abandon_modal();
                    self.contextual_serial += 1;
                    self.contextual =
                        Some(crate::page_browser::ContextualSurface::LayoutNavigation {
                            id: self.contextual_serial,
                            generation: self.state.generation(),
                            error: None,
                        });
                }
            }
            Intent::Reconfigure { id, options } => self.reconfigure_layout(id, options, cx),
            Intent::Split { next, notes, save }
                if !self.busy() && self.settings.write_policy == WritePolicy::Writable =>
            {
                self.settings.settings.presenter.next_slide_ratio = next.clamp(0.1, 0.8);
                self.settings.settings.presenter.notes_ratio = notes.clamp(0.55, 0.8);
                self.settings.note_edit();
                if save {
                    self.save(cx);
                }
            }
            Intent::Escape if self.contextual.is_some() => self.close_contextual(),
            Intent::Escape => {
                self.dismiss_preferences();
                self.cancel_focus();
                self.close_identify(cx);
                match transactions::escape(
                    self.modal.take().is_some(),
                    self.pending.is_some() || self.escape_stops,
                ) {
                    transactions::Escape::Stop => self.stop(cx),
                    transactions::Escape::CloseModal | transactions::Escape::Nothing => {}
                }
            }
            Intent::Stop => self.stop(cx),
            Intent::Home => self.return_home(cx),
            Intent::Start => self.start(cx),
            Intent::Swap => self.swap(cx),
            Intent::Open | Intent::Settings | Intent::Jump | Intent::Displays => {
                self.request_focus(intent, cx)
            }
            Intent::RefreshDisplays => {
                if self.display_draft().is_some() && !self.busy() {
                    self.detect_displays(cx);
                    self.new_display_draft();
                }
            }
            Intent::SelectPresenter(id) => {
                if self.display_editable()
                    && let Some(draft) = self.display_draft_mut()
                {
                    draft.presenter = Some(id);
                }
            }
            Intent::SelectPdf(target) => {
                if self.display_editable()
                    && let Some(draft) = self.display_draft_mut()
                {
                    draft.pdf = target;
                }
            }
            Intent::ApplyDisplays => {
                if self.display_editable() {
                    self.detect_displays(cx);
                    self.apply_display_draft(cx);
                }
            }
            Intent::CancelDisplayDraft => {
                if self.modal == Some(Modal::Settings) {
                    if !self.busy() {
                        self.new_display_draft();
                    }
                } else {
                    self.dispatch(Intent::Escape, cx);
                }
            }
            Intent::ToggleDisplayDetails(id) => {
                if !self.display_details.remove(&id) {
                    self.display_details.insert(id);
                }
            }
            Intent::Identify => self.identify_displays(cx),
            Intent::Import(path) => self.begin_import(path, cx),
            Intent::RetrySave => {
                if !self.busy() {
                    self.save(cx);
                }
            }
            action
                if self.modal.is_none()
                    && self.contextual.is_none()
                    && self.pending.is_none()
                    && self.state.stage() != SessionStage::Idle =>
            {
                let command = match action {
                    Intent::Next => Some(Command::NextStep),
                    Intent::Previous => Some(Command::PreviousStep),
                    Intent::Black => Some(Command::ToggleBlank(BlankMode::Black)),
                    Intent::White => Some(Command::ToggleBlank(BlankMode::White)),
                    _ => None,
                };
                if let Some(command) = command {
                    self.apply(command, cx);
                }
            }
            _ => {}
        }
        self.sync_playback(self.pending.is_some(), cx);
        cx.notify();
    }

    fn save(&mut self, cx: &mut Context<Self>) {
        if let Err(error) = self.store.save_session(&mut self.settings) {
            // SettingsSession owns persistence feedback and clears it on retry.
            // Do not duplicate it or overwrite an unrelated operation error.
            tracing::warn!(%error, "settings remain unsaved");
        }
        cx.notify();
    }

    pub(crate) fn import_allowed(&self) -> bool {
        !self.loading
            && self.preference_change.is_none()
            && transactions::import_allowed(
                self.state.stage(),
                self.pending
                    .as_ref()
                    .map(|pending| pending.kind.operation()),
            )
    }

    fn begin_import(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        self.close_contextual();
        if !self.import_allowed() {
            self.error = Some(self.locale.text("stop-before-import").into());
            return;
        }
        if self.queue_reload_import(path.clone()) {
            return;
        }
        if let Err(error) = &self.renderer {
            tracing::warn!(%error, "PDF import unavailable");
            self.error = Some(self.locale.text("import-unavailable").into());
            return;
        }
        self.import_token += 1;
        let token = self.import_token;
        self.import = None;
        self.loading = true;
        self.error = None;
        let read_path = path.clone();
        let read = cx
            .background_executor()
            .spawn(async move { std::fs::read(read_path) });
        self.await_import_read(path, token, read, cx);
    }

    fn await_import_read(
        &mut self,
        path: PathBuf,
        token: u64,
        read: gpui::Task<std::io::Result<Vec<u8>>>,
        cx: &mut Context<Self>,
    ) {
        self.import_read_task =
            Some(cx.spawn(async move |weak, cx| {
                let bytes = read.await;
                let bytes = match bytes {
                    Ok(bytes) => {
                        cx.background_executor()
                            .spawn(async move {
                                ferrostage_pdf::structure::extract_structure(&bytes)
                                    .map(|structure| (bytes, structure))
                                    .map_err(ImportReadError::Structure)
                            })
                            .await
                    }
                    Err(error) => Err(ImportReadError::Io(error)),
                };
                if let Err(error) = weak.update(cx, |shell, cx| {
                    if shell.import_token != token {
                        return;
                    }
                    match bytes {
                        Ok((bytes, structure)) => {
                            let generation = ferrostage_core::DocumentGeneration(
                                shell.next_generation.max(shell.state.generation().0 + 1),
                            );
                            shell.next_generation = generation.0 + 1;
                            let load = shell.renderer.as_ref().map_err(|e| e.to_string()).and_then(
                                |worker| {
                                    crate::document_load::PendingLoad::new(
                                        worker, generation, bytes, structure,
                                    )
                                },
                            );
                            match load {
                                Ok(load) => shell.import = Some(ImportReply { token, path, load }),
                                Err(error) => {
                                    shell.loading = false;
                                    tracing::warn!(%error, "PDF candidate preparation failed");
                                    shell.error = Some(shell.locale.text("import-error").into());
                                }
                            }
                        }
                        Err(error) => {
                            shell.loading = false;
                            tracing::warn!(%error, "PDF import read or structure check failed");
                            shell.error = Some(error.message(shell.locale));
                        }
                    }
                    cx.notify();
                }) {
                    tracing::debug!(%error,"import target closed");
                }
            }));
    }

    fn poll_import(&mut self, cx: &mut Context<Self>) {
        let Some(pending) = &mut self.import else {
            return;
        };
        let Ok(worker) = &self.renderer else { return };
        let Some(result) = pending.load.poll(worker) else {
            return;
        };
        let Some(pending) = self.import.take() else {
            return;
        };
        if pending.token != self.import_token {
            return;
        }
        self.loading = false;
        match result {
            Ok(session) => self.finish_import(pending.path, pending.load.generation, session, cx),
            Err(error) => {
                tracing::warn!(%error, "PDF candidate load failed");
                self.error = Some(self.locale.text("import-error").into());
            }
        }
        cx.notify();
    }

    fn finish_import(
        &mut self,
        path: PathBuf,
        generation: ferrostage_core::DocumentGeneration,
        session: crate::workbench::DocumentSession,
        cx: &mut Context<Self>,
    ) {
        let old = self.state.generation();
        match session
            .document
            .descriptor(generation)
            .and_then(|descriptor| {
                self.state
                    .apply_at(Command::LoadMappedDocument(descriptor), Instant::now())
            }) {
            Ok(()) => {
                self.path = Some(path.clone());
                self.reload.failed_worker = false;
                self.recover_reload_orphans();
                if let Err(error) =
                    self.replace_pdf_watch(&self.settings.settings.hot_reload.clone())
                {
                    self.warnings.push(error);
                }
                self.close_contextual();
                self.workbench.clear_frames();
                self.workbench.page_number_mode = crate::workbench::PageNumberMode::Content;
                self.workbench.browser_view = crate::workbench::BrowserView::Thumbnails;
                self.workbench.document = Some(session);
                if old.0 != 0
                    && let Ok(worker) = &self.renderer
                {
                    let _ = worker.retire_generation(old);
                }
                self.error = None;
                if self.settings.write_policy == WritePolicy::Writable {
                    self.settings.settings.general.record_recent_file(
                        path.to_string_lossy().into_owned(),
                        chrono::Utc::now().timestamp(),
                    );
                    self.settings.note_edit();
                    self.save(cx);
                }
                self.publish(false, cx);
            }
            Err(error) => {
                tracing::warn!(%error, "PDF document acceptance failed");
                self.error = Some(self.locale.text("import-error").into());
            }
        }
    }

    fn cancel_focus(&mut self) {
        self.focus_intent = None;
        self.focus_token += 1;
    }

    fn request_focus(&mut self, intent: Intent, cx: &mut Context<Self>) {
        if self.pending.is_some() {
            return;
        }
        self.cancel_focus();
        let Some(presenter) = &self.presenter else {
            return;
        };
        let handle = presenter.handle;
        if let Err(error) = handle.update(cx, |_, window, _| window.activate_window()) {
            self.error = Some(error.to_string());
            return;
        }
        self.focus_intent = Some(FocusIntent {
            open_rejected: matches!(intent, Intent::Open) && !self.import_allowed(),
            intent,
            presenter: handle.window_id(),
            deadline: Instant::now() + CONFIRM_TIMEOUT,
        });
    }

    fn poll_focus(&mut self, cx: &mut Context<Self>) {
        let Some(pending) = self.focus_intent.as_ref() else {
            return;
        };
        let handle = self.presenter.as_ref().map(|p| p.handle);
        let cached_active = handle
            .and_then(|handle| {
                handle
                    .update(cx, |_, window, _| window.is_window_active())
                    .ok()
            })
            .unwrap_or(false);
        match transactions::focus(
            pending.presenter,
            handle.map(|handle| handle.window_id()),
            cached_active,
            cx.active_window().map(|handle| handle.window_id()),
            Instant::now() > pending.deadline,
        ) {
            transactions::Focus::Cancel => {
                self.cancel_focus();
                self.error = Some(ShellError::ActivationTimeout.localized(self.locale));
                cx.notify();
                return;
            }
            transactions::Focus::Wait => return,
            transactions::Focus::Confirmed => {}
        }
        let Some(handle) = handle else {
            return;
        };
        let Some(pending) = self.focus_intent.take() else {
            return;
        };
        tracing::info!(window=?handle.window_id(),"presenter activation confirmed");
        match pending.intent {
            Intent::Open if pending.open_rejected || !self.import_allowed() => {
                self.error = Some(self.locale.text("stop-before-import").into());
            }
            Intent::Open => {
                let focus_token = self.focus_token;
                let receiver = cx.prompt_for_paths(PathPromptOptions {
                    files: true,
                    directories: false,
                    multiple: false,
                    prompt: Some(self.locale.text("open").into()),
                });
                self.path_prompt_task = Some(cx.spawn(async move |weak, cx| {
                    let result = receiver.await;
                    if let Err(error) = weak.update(cx, |shell, cx| {
                        // A retired owner must not deliver a late import.
                        if shell.focus_token != focus_token
                            || shell.presenter.as_ref().map(|p| p.handle.window_id())
                                != Some(pending.presenter)
                        {
                            return;
                        }
                        match result {
                            Ok(Ok(Some(paths))) => {
                                if let Some(path) = paths.into_iter().next() {
                                    shell.begin_import(path, cx);
                                }
                            }
                            Ok(Ok(None)) => {}
                            Ok(Err(error)) => shell.error = Some(error.to_string()),
                            Err(error) => shell.error = Some(error.to_string()),
                        }
                        cx.notify();
                    }) {
                        tracing::debug!(%error,"dialog owner closed");
                    }
                }));
            }
            Intent::Settings => {
                self.close_contextual();
                self.preferences = Default::default();
                self.modal = Some(Modal::Settings);
            }
            Intent::Displays => {
                self.close_contextual();
                self.detect_displays(cx);
                self.new_display_draft();
            }
            Intent::Jump if self.state.page_count() > 0 => self.open_browser(),
            _ => {}
        }
        cx.notify();
    }

    fn open(
        &self,
        monitor: MonitorId,
        fullscreen: bool,
        audience: bool,
        cx: &mut Context<Self>,
    ) -> Result<WindowRecord, String> {
        let target_display = *self
            .mapped
            .as_ref()
            .map_err(|error| error.localized(self.locale))?
            .get(&monitor)
            .ok_or_else(|| ShellError::AmbiguousDisplays.localized(self.locale))?;
        let bounds = cx
            .displays()
            .into_iter()
            .find(|item| u64::from(item.id()) == target_display)
            .ok_or_else(|| ShellError::AmbiguousDisplays.localized(self.locale))?
            .visible_bounds();
        let [x, y, width, height] =
            policy::windowed_bounds(rect(bounds), [if audience { 800. } else { 1100. }, 760.]);
        let restore = Bounds::new(point(px(x), px(y)), size(px(width), px(height)));
        let options = WindowOptions {
            display_id: Some(DisplayId::new(target_display)),
            window_bounds: Some(if fullscreen {
                WindowBounds::Fullscreen(restore)
            } else {
                WindowBounds::Windowed(restore)
            }),
            titlebar: if audience {
                None
            } else {
                Some(TitlebarOptions {
                    title: Some("FerroStage".into()),
                    ..Default::default()
                })
            },
            focus: !audience,
            window_min_size: Some(size(px(320.), px(240.))),
            ..Default::default()
        };
        let observed = Rc::new(Cell::new(false));
        let flag = observed.clone();
        let observed_display = Rc::new(Cell::new(None));
        let location = observed_display.clone();
        let handle: AnyWindowHandle = if audience {
            let snapshot = self.snapshot.clone();
            cx.open_window(options, move |window, cx| {
                location.set(window.display(cx).map(|display| u64::from(display.id())));
                Self::intercept_close(window, cx);
                cx.new(|cx| {
                    cx.observe(&snapshot, |_, _, cx| cx.notify()).detach();
                    cx.observe_window_bounds(window, move |_, window, cx| {
                        flag.set(true);
                        location.set(window.display(cx).map(|display| u64::from(display.id())));
                        Self::log_bounds(window, cx);
                    })
                    .detach();
                    Audience::new(snapshot, cx)
                })
            })
            .map(Into::into)
        } else {
            let shell = cx.weak_entity();
            let locale = self.locale;
            let data = PresenterData::from_shell(self);
            cx.open_window(options, move |window, cx| {
                location.set(window.display(cx).map(|display| u64::from(display.id())));
                Self::intercept_close(window, cx);
                cx.new(|cx| {
                    cx.observe_window_bounds(window, move |_, window, cx| {
                        flag.set(true);
                        location.set(window.display(cx).map(|display| u64::from(display.id())));
                        Self::log_bounds(window, cx);
                    })
                    .detach();
                    cx.observe_window_appearance(window, |_, _, cx| cx.notify())
                        .detach();
                    Presenter::new(shell, locale, data, window, cx)
                })
            })
            .map(Into::into)
        }
        .map_err(|error| error.to_string())?;
        if audience {
            let entity = handle
                .read(cx, |entity: Entity<Audience>, _| entity)
                .map_err(|error| error.to_string())?;
            cx.subscribe(
                &entity,
                move |_, _, event: &crate::audience::PlaybackEvent, cx| {
                    let event = *event;
                    let shell = cx.weak_entity();
                    cx.defer(move |cx| {
                        let _ = shell.update(cx, |shell, cx| {
                            shell.playback_event(handle.window_id(), event, cx)
                        });
                    });
                },
            )
            .detach();
            cx.subscribe(
                &entity,
                move |_, _, _: &crate::audience::AdvanceRequested, cx| {
                    // Input still owns its source window lease. Consume the request
                    // after that lease ends, and validate its current owner then.
                    let shell = cx.weak_entity();
                    cx.defer(move |cx| {
                        let _ = shell.update(cx, |shell, cx| {
                            let projection = shell.snapshot.read(cx);
                            if shell.audience.as_ref().is_some_and(|w| w.handle == handle)
                                && shell.state.stage() == SessionStage::Presenting
                                && shell.settings.settings.display.audience_click_to_advance
                                && shell.modal.is_none()
                                && shell.contextual.is_none()
                                && !shell.busy()
                                && projection.stage == AudienceStage::Live
                                && projection.blank_mode.is_none()
                                && shell.state.audience_snapshot().blank_mode.is_none()
                            {
                                shell.dispatch(Intent::Next, cx);
                            }
                        });
                    });
                },
            )
            .detach();
            cx.subscribe(
                &entity,
                move |shell, _, status: &crate::audience::StagingStatus, cx| {
                    if shell
                        .audience
                        .as_ref()
                        .is_some_and(|record| record.handle == handle)
                    {
                        shell.staging_status(status.0, true, status.1.clone(), cx);
                    }
                },
            )
            .detach();
            cx.subscribe(
                &entity,
                move |shell, _, status: &crate::audience::RenderStatus, cx| {
                    let active = shell
                        .audience
                        .as_ref()
                        .is_some_and(|record| record.handle == handle)
                        || shell
                            .pending
                            .as_ref()
                            .and_then(|pending| pending.audience.as_ref())
                            .is_some_and(|record| record.handle == handle);
                    if active {
                        shell.graphics_error = status.0.clone();
                        cx.notify();
                    }
                },
            )
            .detach();
        }
        tracing::info!(?handle,monitor=?monitor,target_display,fullscreen,audience, "window created behind confirmation gate");
        let record = WindowRecord {
            handle,
            monitor,
            display: target_display,
            fullscreen,
            client_size: restore.size,
            resize_attempt: Cell::new(None),
            bounds_observed: observed,
            observed_display,
        };
        Self::restore_client_size(&record, cx);
        Ok(record)
    }

    fn restore_client_size(record: &WindowRecord, cx: &mut App) {
        if record.fullscreen {
            return;
        }
        if let Err(error) = record.handle.update(cx, |_, window, _| {
            let actual = window.bounds().size;
            let delta = record.client_size - actual;
            if delta.width.abs() <= px(2.) && delta.height.abs() <= px(2.) {
                return;
            }
            // Cross-DPI creation may scale twice. The platform's cached native
            // frame inset can also leave a small client-size error after resize.
            // Wait for a changed native observation, then compensate that error
            // once. Unchanged/stale events and nonconvergence keep the timeout.
            let previous = record.resize_attempt.get();
            let requested = if let Some(previous) = previous {
                if previous.corrected
                    || !record.bounds_observed.get()
                    || actual == previous.before
                    || !f32::from(delta.width).is_finite()
                    || !f32::from(delta.height).is_finite()
                    || delta.width.abs() > px(32.)
                    || delta.height.abs() > px(32.)
                {
                    return;
                }
                previous.requested + delta
            } else {
                record.client_size
            };
            record.resize_attempt.set(Some(ClientResizeAttempt {
                requested,
                before: actual,
                corrected: previous.is_some(),
            }));
            record.bounds_observed.set(false);
            tracing::info!(window=?record.handle.window_id(), actual=?rect(window.bounds()), target=?record.client_size, requested=?requested, correction=previous.is_some(), "restoring target client size after creation");
            window.resize(requested);
        }) {
            tracing::debug!(%error, "pending window client resize unavailable");
        }
    }

    fn intercept_close(window: &gpui::Window, cx: &App) {
        window.on_window_should_close(cx, |window, cx| {
            let handle = window.window_handle();
            // Return to the platform close callback before removal. This keeps
            // GPUI's platform Drop as the sole owner of native destruction.
            cx.spawn(async move |cx| {
                if let Err(error) = handle.update(cx, |_, window, _| window.remove_window()) {
                    tracing::debug!(%error, "close request for retired window");
                }
            })
            .detach();
            false
        });
    }

    fn confirmed(record: &WindowRecord, cx: &mut App) -> bool {
        record
            .handle
            .update(cx, |_, window, cx| {
                let display = window.display(cx);
                record
                    .observed_display
                    .set(display.as_ref().map(|display| u64::from(display.id())));
                let Some(display) = display else {
                    return false;
                };
                let actual = window.bounds();
                if !record.fullscreen
                    && ((actual.size.width - record.client_size.width).abs() > px(2.)
                        || (actual.size.height - record.client_size.height).abs() > px(2.))
                {
                    return false;
                }
                policy::placement_confirmed(
                    record.display,
                    Some(u64::from(display.id())),
                    record.fullscreen,
                    window.is_fullscreen(),
                    record.bounds_observed.get(),
                    rect(actual),
                    rect(if record.fullscreen {
                        display.bounds()
                    } else {
                        display.visible_bounds()
                    }),
                )
            })
            .unwrap_or(false)
    }

    fn close(record: WindowRecord, cx: &mut App) {
        if let Err(error) = record
            .handle
            .update(cx, |_, window, _| window.remove_window())
        {
            tracing::debug!(%error,"window already retired");
        }
    }

    fn log_placement(record: &WindowRecord, reason: &'static str, cx: &mut App) {
        if let Err(error) = record.handle.update(cx, |_, window, cx| {
            let observed_display = window.display(cx);
            tracing::info!(
                reason, window=?record.handle.window_id(), monitor=?record.monitor,
                requested_display=record.display, requested_fullscreen=record.fullscreen,
                actual_display=?observed_display.as_ref().map(|display| u64::from(display.id())),
                actual_display_bounds=?observed_display.as_ref().map(|display| rect(display.bounds())),
                actual_usable_bounds=?observed_display.as_ref().map(|display| rect(display.visible_bounds())),
                requested_client_size=?record.client_size,
                actual_bounds=?rect(window.bounds()), actual_fullscreen=window.is_fullscreen(),
                bounds_observed=record.bounds_observed.get(), "window placement observation"
            );
        }) {
            tracing::info!(reason, window=?record.handle.window_id(), %error, "window placement unavailable");
        }
    }

    fn log_bounds(window: &gpui::Window, cx: &App) {
        tracing::debug!(
            window = ?window.window_handle().window_id(),
            bounds = ?rect(window.bounds()),
            viewport_size = ?window.viewport_size(),
            scale_factor = window.scale_factor(),
            display = ?window.display(cx).map(|display| u64::from(display.id())),
            fullscreen = window.is_fullscreen(),
            "window bounds changed"
        );
    }

    fn start(&mut self, cx: &mut Context<Self>) {
        if !self.topology_valid || self._watcher.is_err() {
            return;
        }
        self.close_identify(cx);
        self.refresh_mapping(cx);
        if let Some(reason) = self.start_reason() {
            self.error = Some(reason);
            return;
        }
        match policy::audience_target(&self.monitors, &self.settings.settings.display) {
            Ok((target, fullscreen)) => {
                if let Err(error) = self.enter_presenter_fullscreen(cx) {
                    self.error = Some(error);
                    return;
                }
                self.publish(true, cx);
                match self.open(target, fullscreen, true, cx) {
                    Ok(audience) => {
                        self.pending = Some(Transaction {
                            kind: TransactionKind::Start,
                            presenter: None,
                            audience: Some(audience),
                            deadline: Instant::now() + CONFIRM_TIMEOUT,
                        })
                    }
                    Err(error) => {
                        self.error = Some(error);
                        self.restore_presenter(cx);
                    }
                }
            }
            Err(error) => self.error = Some(error.localized(self.locale)),
        }
    }

    fn retire_pending(&mut self, reason: &'static str, cx: &mut Context<Self>) {
        if let Some(pending) = self.pending.take() {
            let starting = matches!(pending.kind, TransactionKind::Start);
            let preferences = matches!(pending.kind, TransactionKind::Preferences(_));
            tracing::info!(reason, operation=?pending.kind.operation(), "window transaction cancelled");
            if let Some(window) = pending.audience {
                Self::log_placement(&window, reason, cx);
                Self::close(window, cx);
            }
            if let Some(window) = pending.presenter {
                Self::log_placement(&window, reason, cx);
                Self::close(window, cx);
            }
            if preferences {
                self.rollback_preferences(reason.into(), cx);
            }
            if starting {
                self.restore_presenter(cx);
            }
        }
    }

    fn cancel_transaction(&mut self, reason: &'static str, cx: &mut Context<Self>) {
        self.retire_pending(reason, cx);
        self.publish(false, cx);
    }

    fn abandon_modal(&mut self) {
        self.close_contextual();
        self.modal = None;
    }

    fn stop(&mut self, cx: &mut Context<Self>) {
        self.graphics_error = None;
        self.close_identify(cx);
        self.cancel_focus();
        self.abandon_modal();
        for step in transactions::shutdown() {
            match step {
                Shutdown::Curtain => self.publish(true, cx),
                Shutdown::RetirePending => self.retire_pending("stop", cx),
                Shutdown::StopCore => {
                    if self.state.stage() == SessionStage::Presenting {
                        self.apply(Command::StopPresentation, cx);
                    }
                }
                Shutdown::RetireAudience => {
                    if let Some(audience) = self.audience.take() {
                        Self::close(audience, cx);
                    }
                }
            }
        }
        self.restore_presenter(cx);
        cx.notify();
    }

    fn return_home(&mut self, cx: &mut Context<Self>) {
        if self.state.stage() == SessionStage::Idle
            || self.loading
            || self.pending.is_some()
            || self.preference_change.is_some()
            || self.reload_busy()
        {
            return;
        }
        let generation = self.state.generation();
        if self.state.stage() == SessionStage::Presenting {
            self.stop(cx);
        } else {
            self.close_identify(cx);
            self.cancel_focus();
            self.abandon_modal();
        }
        self.media.stop();
        if let Err(error) = self.state.apply_at(Command::CloseDocument, Instant::now()) {
            self.error = Some(error.to_string());
            cx.notify();
            return;
        }
        self.path = None;
        self.workbench.document = None;
        self.workbench.clear_frames();
        if let Err(error) = self.replace_pdf_watch(&self.settings.settings.hot_reload.clone()) {
            self.warnings.push(error);
        }
        if generation.0 != 0
            && let Ok(worker) = &self.renderer
        {
            let _ = worker.retire_generation(generation);
        }
        self.publish(false, cx);
        cx.notify();
    }

    fn swap(&mut self, cx: &mut Context<Self>) {
        self.close_contextual();
        if !self.topology_valid
            || self.pending.is_some()
            || self.loading
            || self.preference_change.is_some()
        {
            return;
        }
        if self.settings.write_policy != WritePolicy::Writable {
            self.error = Some(self.locale.text("readonly-swap").into());
            return;
        }
        self.refresh_mapping(cx);
        let settings = match policy::swapped(&self.monitors, &self.settings.settings.display) {
            Ok(value) => value,
            Err(error) => {
                self.error = Some(error.localized(self.locale));
                return;
            }
        };
        self.assign_displays(settings, cx);
    }

    // Both explicit assignments and the compatibility Swap shortcut use this
    // one preparation/placement/commit transaction.
    fn assign_displays(&mut self, settings: DisplaySettings, cx: &mut Context<Self>) {
        tracing::info!(stage=?self.state.stage(), page=self.state.current_page().0,
            elapsed_ms=self.state.elapsed_at(Instant::now()).as_millis(), requested=?settings,
            "display assignment preparing");
        let Some(target) = policy::roles(&self.monitors, &settings).presenter else {
            return;
        };
        self.close_identify(cx);
        self.cancel_focus();
        self.abandon_modal();
        self.publish(true, cx);
        let playing = self.state.stage() == SessionStage::Presenting
            && policy::audience_target(&self.monitors, &settings).is_ok();
        let prepared = transactions::prepare(playing, |audience| {
            if audience {
                policy::audience_target(&self.monitors, &settings)
                    .map_err(|error| error.localized(self.locale))
                    .and_then(|(target, fullscreen)| self.open(target, fullscreen, true, cx))
            } else {
                self.open(target.clone(), false, false, cx)
            }
        });
        match prepared {
            Ok(prepared) => {
                self.pending = Some(Transaction {
                    kind: if self.preference_change.is_some() {
                        TransactionKind::Preferences(settings)
                    } else {
                        TransactionKind::Assign(settings)
                    },
                    presenter: Some(prepared.presenter),
                    audience: prepared.audience,
                    deadline: Instant::now() + CONFIRM_TIMEOUT,
                })
            }
            Err(failure) => {
                for window in failure.created {
                    Self::close(window, cx);
                }
                self.error = Some(failure.error);
                self.publish(false, cx);
            }
        }
    }

    fn recover_presenter(&mut self, cx: &mut Context<Self>) {
        if self.pending.is_some() || self.restoring_presenter() || !self.topology_valid {
            return;
        }
        let Some(target) = policy::roles(&self.monitors, &self.settings.settings.display).presenter
        else {
            return;
        };
        let Some(display) = self
            .mapped
            .as_ref()
            .ok()
            .and_then(|mapping| mapping.get(&target))
        else {
            return;
        };
        if self.presenter.as_ref().is_some_and(|p| {
            p.monitor == target
                && p.display == *display
                && (!self.recover_windowed || !p.fullscreen)
        }) {
            return;
        }
        let fullscreen = self.presenter.is_none()
            && !self.recover_windowed
            && self.settings.settings.display.presenter_start_fullscreen;
        match self.open(target, fullscreen, false, cx) {
            Ok(presenter) => {
                self.pending = Some(Transaction {
                    kind: TransactionKind::RecoverPresenter,
                    presenter: Some(presenter),
                    audience: None,
                    deadline: Instant::now() + CONFIRM_TIMEOUT,
                })
            }
            Err(error) => self.error = Some(error),
        }
    }

    fn new_display_draft(&mut self) {
        tracing::info!(stage=?self.state.stage(), page=self.state.current_page().0,
            elapsed_ms=self.state.elapsed_at(Instant::now()).as_millis(), revision=self.topology_revision,
            "display settings opened");
        let draft = DisplayDraft::new(
            self.topology_revision,
            &self.monitors,
            &self.settings.settings.display,
        );
        if self.modal == Some(Modal::Settings) {
            self.preferences.display = Some(draft);
        } else {
            self.modal = Some(Modal::Displays(draft));
        }
    }

    fn display_draft(&self) -> Option<&DisplayDraft> {
        match &self.modal {
            Some(Modal::Displays(draft)) => Some(draft),
            Some(Modal::Settings) => self.preferences.display.as_ref(),
            _ => None,
        }
    }
    fn display_draft_mut(&mut self) -> Option<&mut DisplayDraft> {
        match &mut self.modal {
            Some(Modal::Displays(draft)) => Some(draft),
            Some(Modal::Settings) => self.preferences.display.as_mut(),
            _ => None,
        }
    }
    fn display_editable(&self) -> bool {
        !self.busy()
            && self.settings.write_policy == WritePolicy::Writable
            && self.topology_valid
            && self.mapped.is_ok()
            && self
                .display_draft()
                .is_some_and(|draft| draft.revision == self.topology_revision)
    }

    pub(crate) fn display_panel(&self) -> Option<DisplayPanel> {
        let draft = self.display_draft()?;
        let problem = if self.settings.write_policy != WritePolicy::Writable {
            Some("displays-readonly")
        } else if !self.topology_valid {
            Some("displays-unavailable")
        } else if self.monitors.is_empty() {
            Some("displays-empty")
        } else if draft.revision != self.topology_revision {
            Some("displays-stale")
        } else if self.mapped.is_err() {
            Some("ambiguous-displays")
        } else if self.busy() {
            Some("confirming")
        } else {
            draft.validation_error(self.topology_revision, &self.monitors)
        };
        let problem = problem.map(|key| self.locale.text(key).to_owned());
        Some(DisplayPanel {
            monitors: self.monitors.clone(),
            effective: self.settings.settings.display.clone(),
            draft: draft.clone(),
            monitor_error: self.monitor_error_text(),
            operation_error: self
                .error
                .as_ref()
                .filter(|error| Some(*error) != problem.as_ref())
                .cloned(),
            problem,
            topology_valid: self.topology_valid,
            editable: self.display_editable(),
            can_identify: self.topology_valid
                && self.mapped.is_ok()
                && self.state.stage() != SessionStage::Presenting
                && !self.busy()
                && !self.monitors.is_empty(),
            roles: display_settings::RoleProjection::new(
                self.window_location(self.presenter.as_ref(), true),
                self.window_location(
                    self.audience.as_ref(),
                    self.state.stage() == SessionStage::Presenting,
                ),
                DisplayDraft::new(
                    self.topology_revision,
                    &self.monitors,
                    &self.settings.settings.display,
                ),
                draft,
            ),
            details: self.display_details.clone(),
            busy: self.busy(),
        })
    }

    fn window_location(
        &self,
        record: Option<&WindowRecord>,
        running: bool,
    ) -> display_settings::WindowLocation {
        display_settings::observed_location(
            running,
            record.and_then(|record| record.observed_display.get()),
            self.topology_valid,
            &self.monitors,
            self.mapped.as_ref().ok(),
        )
    }

    fn sample_window_locations(&self, cx: &mut App) {
        for record in [self.presenter.as_ref(), self.audience.as_ref()]
            .into_iter()
            .flatten()
        {
            let display = record
                .handle
                .update(cx, |_, window, cx| {
                    window.display(cx).map(|display| u64::from(display.id()))
                })
                .ok()
                .flatten();
            record.observed_display.set(display);
        }
    }

    fn detect_displays(&mut self, cx: &mut Context<Self>) {
        // Consume older watcher deliveries before the fresh synchronous probe.
        while let Ok(event) = self.topology.try_recv() {
            self.topology_changed(event, cx);
        }
        match ferrostage_platform::enumerate_monitors() {
            Ok(monitors) => self.topology_changed(TopologyEvent::Changed(monitors), cx),
            Err(error) => self.topology_changed(
                TopologyEvent::Unavailable {
                    error,
                    last_known: self.monitors.clone(),
                },
                cx,
            ),
        }
        self.refresh_mapping(cx);
    }

    fn apply_display_draft(&mut self, cx: &mut Context<Self>) {
        let Some(panel) = self.display_panel() else {
            return;
        };
        if panel.problem.is_some() {
            return;
        }
        match panel.draft.settings(
            self.topology_revision,
            &self.monitors,
            &self.settings.settings.display,
        ) {
            Ok(settings) => {
                self.error = None;
                if self.modal == Some(Modal::Settings) {
                    let mut next = self.settings.settings.clone();
                    next.display = settings;
                    self.apply_preferences(next, cx);
                } else {
                    self.assign_displays(settings, cx);
                }
            }
            Err(key) => self.error = Some(self.locale.text(key).into()),
        }
    }

    fn close_identify(&mut self, cx: &mut App) {
        self.identify_deadline = None;
        for handle in self.identify.drain(..) {
            if let Err(error) = handle.update(cx, |_, window, _| window.remove_window()) {
                tracing::debug!(%error, "identify window already retired");
            }
        }
    }

    fn identify_displays(&mut self, cx: &mut Context<Self>) {
        if !self.display_panel().is_some_and(|panel| panel.can_identify) {
            return;
        }
        self.detect_displays(cx);
        if !self.display_panel().is_some_and(|panel| panel.can_identify) {
            return;
        }
        self.open_identify(cx);
    }

    fn open_identify(&mut self, cx: &mut Context<Self>) {
        self.close_identify(cx);
        let targets = display_settings::numbered(&self.monitors)
            .into_iter()
            .map(|(number, monitor)| {
                let display_id = self
                    .mapped
                    .as_ref()
                    .ok()
                    .and_then(|mapping| mapping.get(&monitor.id))
                    .copied()
                    .ok_or(ShellError::AmbiguousDisplays)?;
                let display = cx
                    .displays()
                    .into_iter()
                    .find(|d| u64::from(d.id()) == display_id)
                    .ok_or(ShellError::AmbiguousDisplays)?;
                let copy = display_settings::screen_copy(self.locale, &self.monitors, monitor);
                Ok((
                    number,
                    monitor.id.clone(),
                    copy.title,
                    copy.identify,
                    display_id,
                    display.bounds(),
                ))
            })
            .collect::<Result<Vec<_>, ShellError>>();
        let targets = match targets {
            Ok(targets) => targets,
            Err(error) => {
                self.error = Some(error.localized(self.locale));
                return;
            }
        };
        for (number, monitor, title, name, display_id, bounds) in targets {
            let size = size(px(460.), px(240.));
            let origin = point(
                bounds.origin.x + (bounds.size.width - size.width) / 2.,
                bounds.origin.y + (bounds.size.height - size.height) / 2.,
            );
            let result = cx.open_window(
                WindowOptions {
                    display_id: Some(DisplayId::new(display_id)),
                    window_bounds: Some(WindowBounds::Windowed(Bounds::new(origin, size))),
                    titlebar: None,
                    focus: false,
                    kind: gpui::WindowKind::PopUp,
                    is_movable: false,
                    is_resizable: false,
                    is_minimizable: false,
                    ..Default::default()
                },
                move |window, cx| {
                    Self::intercept_close(window, cx);
                    window.set_window_title(&format!("FerroStage {number} · {title}"));
                    cx.new(|cx| {
                        cx.observe_window_bounds(window, |_, window, cx| {
                            Self::log_bounds(window, cx)
                        })
                        .detach();
                        Identify { number, name }
                    })
                },
            );
            match result {
                Ok(handle) => {
                    tracing::info!(number, ?monitor, display_id, window=?handle.window_id(), "identify display");
                    self.identify.push(handle.into());
                }
                Err(error) => {
                    self.error = Some(error.to_string());
                    self.close_identify(cx);
                    return;
                }
            }
        }
        self.identify_deadline = Some(Instant::now() + Duration::from_secs(3));
    }

    fn poll_transaction(&mut self, cx: &mut Context<Self>) {
        let Some(pending) = &self.pending else {
            return;
        };
        if Instant::now() <= pending.deadline {
            for window in [pending.presenter.as_ref(), pending.audience.as_ref()]
                .into_iter()
                .flatten()
            {
                Self::restore_client_size(window, cx);
            }
        }
        let confirmed = pending
            .presenter
            .as_ref()
            .is_none_or(|window| Self::confirmed(window, cx))
            && pending
                .audience
                .as_ref()
                .is_none_or(|window| Self::confirmed(window, cx))
            && (!matches!(pending.kind, TransactionKind::Start)
                || self.start_presenter_confirmed(cx));
        if !confirmed {
            if Instant::now() > pending.deadline {
                self.error = Some(ShellError::PlacementTimeout.localized(self.locale));
                self.cancel_transaction("placement timeout", cx);
                cx.notify();
            }
            return;
        }
        let Some(pending) = self.pending.take() else {
            return;
        };
        for record in [pending.presenter.as_ref(), pending.audience.as_ref()]
            .into_iter()
            .flatten()
        {
            Self::log_placement(record, "confirmed", cx);
        }
        if matches!(pending.kind, TransactionKind::Start)
            && let Err(error) = self.state.apply_at(
                Command::StartPresentation {
                    audience_available: true,
                },
                Instant::now(),
            )
        {
            self.error = Some(error.to_string());
            if let Some(window) = pending.audience {
                Self::close(window, cx);
            }
            self.restore_presenter(cx);
            return;
        }
        let old_presenter = if pending.presenter.is_some() {
            std::mem::replace(&mut self.presenter, pending.presenter)
        } else {
            None
        };
        let no_play = matches!(&pending.kind, TransactionKind::Assign(settings) | TransactionKind::Preferences(settings)
            if policy::audience_target(&self.monitors, settings).is_err());
        let old_audience = if pending.audience.is_some() || no_play {
            // A replacement starts behind Curtain and has not painted yet.
            // Its future status events own diagnostics after this commit.
            self.graphics_error = None;
            std::mem::replace(&mut self.audience, pending.audience)
        } else {
            None
        };
        if matches!(pending.kind, TransactionKind::Preferences(_)) && !self.commit_preferences(cx) {
            // Failed final budget application must restore both old windows.
            if let Some(old) = old_presenter
                && let Some(new) = self.presenter.replace(old)
            {
                Self::close(new, cx);
            }
            if let Some(new) = std::mem::replace(&mut self.audience, old_audience) {
                Self::close(new, cx);
            }
            self.publish(false, cx);
            cx.notify();
            return;
        }
        if no_play && self.state.stage() == SessionStage::Presenting {
            // Placement has succeeded. Only now may no-play commit its safe
            // stop; an earlier failure must leave the old session running.
            self.publish(true, cx);
            self.apply(Command::StopPresentation, cx);
        }

        if let TransactionKind::Assign(settings) = pending.kind {
            self.settings.settings.display = settings;
            self.settings.note_edit();
            self.save(cx);
        }
        self.publish(false, cx);
        if let Some(window) = old_audience {
            Self::close(window, cx);
        }
        if let Some(window) = old_presenter {
            Self::close(window, cx);
            self.presenter_restore = None;
        }
        tracing::info!(stage=?self.state.stage(),page=self.state.current_page().0,
            elapsed_ms=self.state.elapsed_at(Instant::now()).as_millis(),
            presenter=?self.presenter.as_ref().map(|w|w.handle),audience=?self.audience.as_ref().map(|w|w.handle),"window transaction committed");
        cx.notify();
    }

    fn tick(&mut self, cx: &mut Context<Self>) {
        while let Ok(event) = self.topology.try_recv() {
            self.topology_changed(event, cx);
        }
        if self
            .identify_deadline
            .is_some_and(|deadline| Instant::now() >= deadline)
        {
            self.close_identify(cx);
        }
        self.poll_preferences(cx);
        self.poll_import(cx);
        self.poll_transaction(cx);
        self.poll_presenter_restore(cx);
        self.poll_focus(cx);
        if let Some(audience) = &self.audience {
            audience.observed_display.set(None);
            if let Ok(target) = audience.handle.update(cx, |_, window, cx| {
                audience
                    .observed_display
                    .set(window.display(cx).map(|display| u64::from(display.id())));
                let size = window.viewport_size();
                let scale = window.scale_factor();
                ferrostage_pdf::PixelSize {
                    width: (f32::from(size.width) * scale).round().max(1.) as u32,
                    height: (f32::from(size.height) * scale).round().max(1.) as u32,
                }
            }) {
                self.workbench.audience_target = Some(crate::workbench::budgeted_target(
                    target,
                    self.settings.settings.cache.l1_gpu_mib as usize * 1024 * 1024 / 4,
                ));
            }
        } else {
            self.workbench.audience_target = None;
        }
        if let Some(presenter) = &self.presenter {
            presenter.observed_display.set(None);
        }
        if let Some(presenter) = &self.presenter
            && let Ok(target) = presenter.handle.update(cx, |_, window, cx| {
                presenter
                    .observed_display
                    .set(window.display(cx).map(|display| u64::from(display.id())));
                let size = window.viewport_size();
                let scale = window.scale_factor();
                ferrostage_pdf::PixelSize {
                    width: (f32::from(size.width)
                        * scale
                        * (1. - self.settings.settings.presenter.next_slide_ratio as f32))
                        .round()
                        .max(1.) as u32,
                    // Command bar, tool row, and start control. The canvas
                    // reports the real pane size on the following frame.
                    height: ((f32::from(size.height) - 220.).max(100.) * scale)
                        .round()
                        .max(1.) as u32,
                }
            })
        {
            self.workbench.presenter_target = Some(target);
        }
        let pointer = &self.settings.settings.pointer;
        let _ = self.state.apply_at(
            Command::SetLaserStyle(ferrostage_core::LaserStyle {
                color_bgra: pointer.laser_color_bgra,
                radius_norm: pointer.laser_radius_norm as f32,
            }),
            Instant::now(),
        );
        self.poll_reload(cx);
        if !self.reload_blocks_render()
            && let Ok(worker) = &self.renderer
        {
            for error in self.workbench.poll(worker) {
                self.error = Some(error);
            }
            if let Err(error) =
                self.workbench
                    .schedule(worker, &self.state, &self.settings.settings.presenter)
            {
                self.error = Some(error);
            }
        }
        self.publish(self.pending.is_some(), cx);
        self.poll_timed_playback(cx);
        cx.notify();
        // If startup had zero screens, the platform watcher owns recovery.
    }

    fn topology_changed(&mut self, event: TopologyEvent, cx: &mut Context<Self>) {
        if matches!(&event, TopologyEvent::Changed(monitors) if self.topology_valid && *monitors == self.monitors)
        {
            return;
        }
        self.topology_revision += 1;
        self.close_identify(cx);
        let settings_open = self.modal == Some(Modal::Settings);
        let display_draft = match &self.modal {
            Some(Modal::Displays(draft)) => Some(draft.clone()),
            _ => None,
        };
        tracing::info!(previous_monitors=?self.monitors,previous_valid=self.topology_valid,?event,"topology event received");
        self.publish(true, cx);
        self.retire_pending("topology changed", cx);
        self.cancel_focus();
        match event {
            TopologyEvent::Changed(monitors) => {
                self.monitors = monitors;
                self.topology_valid = true;
                self.monitor_error = None;
            }
            TopologyEvent::Unavailable { error, .. } => {
                if self.monitor_error.as_ref() != Some(&error) {
                    tracing::warn!(
                        ?error,
                        origin = "topology observation",
                        "monitor discovery unavailable"
                    );
                }
                self.topology_valid = false;
                self.monitor_error = Some(error);
            }
        }
        self.refresh_mapping(cx);
        let audience_valid = self.audience.as_ref().is_none_or(|window| {
            self.mapped
                .as_ref()
                .is_ok_and(|mapping| mapping.get(&window.monitor) == Some(&window.display))
        });
        if !self.topology_valid || self.monitors.is_empty() || !audience_valid {
            self.recover_windowed = true;
            self.stop(cx);
        } else {
            self.publish(false, cx);
        }
        if let Some(draft) = display_draft {
            self.modal = Some(Modal::Displays(draft));
        } else if settings_open {
            self.modal = Some(Modal::Settings);
        }
        self.recover_presenter(cx);
        cx.notify();
    }

    fn closed(&mut self, id: WindowId, cx: &mut Context<Self>) {
        let pending = self
            .pending
            .iter()
            .flat_map(|pending| [pending.presenter.as_ref(), pending.audience.as_ref()])
            .flatten()
            .map(|window| window.handle.window_id())
            .collect::<Vec<_>>();
        match transactions::closed(
            id,
            self.presenter
                .as_ref()
                .map(|window| window.handle.window_id()),
            self.audience
                .as_ref()
                .map(|window| window.handle.window_id()),
            &pending,
        ) {
            transactions::Closed::Audience => {
                self.audience = None;
                self.stop(cx);
            }
            transactions::Closed::Presenter => {
                self.stop_tasks("presenter closed");
                self.presenter = None;
                self.stop(cx);
                self.import_token += 1;
                cx.quit();
            }
            transactions::Closed::Pending => {
                self.cancel_transaction("pending window closed", cx);
                cx.notify();
            }
            transactions::Closed::Retired => {}
        }
        // Closed IDs of retired replacements cannot stop the current session.
    }
}

fn rect(bounds: Bounds<Pixels>) -> [f32; 4] {
    [
        bounds.origin.x.into(),
        bounds.origin.y.into(),
        bounds.size.width.into(),
        bounds.size.height.into(),
    ]
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;
    use crate::display_settings::PdfTarget;
    use gpui::TestAppContext;

    #[gpui::test]
    fn incomplete_pdf_import_is_localized_and_preserves_active_document(cx: &mut TestAppContext) {
        let (shell, _) = workbench_fixture(cx);
        let generation = shell.read_with(cx, |shell, _| shell.state.generation());
        shell.update(cx, |shell, cx| {
            shell.locale = crate::locale::Locale::new("zh-CN");
            shell.path = Some("active.pdf".into());
            shell.settings.settings.general.recent_files = vec!["active.pdf".into()];
            shell.settings.settings.general.recent_opened_at =
                BTreeMap::from([("active.pdf".into(), 123)]);
            let read = cx
                .background_executor()
                .spawn(async { Ok(b"%PDF-1.7\n1 0 obj\n<< /Type /Catalog >>".to_vec()) });
            shell.loading = true;
            shell.await_import_read("partial.pdf".into(), shell.import_token, read, cx);
        });
        cx.run_until_parked();
        shell.read_with(cx, |shell, _| {
            let message = shell.error.as_deref().unwrap();
            assert!(message.contains("不完整或无效"), "{message}");
            assert!(message.contains("若正在编译"), "{message}");
            assert!(!message.contains("lopdf"), "{message}");
            assert!(!shell.loading && shell.import.is_none());
            assert_eq!(shell.state.generation(), generation);
            assert_eq!(shell.path, Some("active.pdf".into()));
            assert!(shell.workbench.document.is_some());
            assert_eq!(shell.settings.settings.general.recent_files, ["active.pdf"]);
            assert_eq!(
                shell.settings.settings.general.recent_opened_at,
                BTreeMap::from([("active.pdf".into(), 123)])
            );
        });
    }

    #[test]
    fn import_read_errors_keep_actionable_localized_categories() {
        use ferrostage_pdf::PdfStructureError;
        let cases = [
            (
                ImportReadError::Io(std::io::Error::from_raw_os_error(32)),
                "read-locked",
            ),
            (
                ImportReadError::Io(std::io::Error::from_raw_os_error(33)),
                "read-locked",
            ),
            (
                ImportReadError::Io(std::io::Error::from(std::io::ErrorKind::NotFound)),
                "read-missing",
            ),
            (
                ImportReadError::Io(std::io::Error::from(std::io::ErrorKind::PermissionDenied)),
                "read-permission",
            ),
            (
                ImportReadError::Io(std::io::Error::other("private diagnostic")),
                "read-failed",
            ),
            (
                ImportReadError::Structure(PdfStructureError::Unreadable),
                "read-incomplete",
            ),
            (
                ImportReadError::Structure(PdfStructureError::InvalidPageGeometry),
                "read-geometry",
            ),
            (
                ImportReadError::Structure(PdfStructureError::UnsupportedUserUnit),
                "read-user-unit",
            ),
        ];
        for language in ["zh-CN", "en-US"] {
            let locale = crate::locale::Locale::new(language);
            for (error, key) in &cases {
                assert_eq!(error.message(locale), locale.text(key));
                assert_ne!(error.message(locale), error.to_string());
            }
        }
    }

    fn pending_import_read(
        shell: &Entity<Shell>,
        cx: &mut TestAppContext,
    ) -> std::sync::Arc<std::sync::atomic::AtomicBool> {
        let completed = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let flag = completed.clone();
        shell.update(cx, |shell, cx| {
            let timer = cx.background_executor().timer(Duration::from_millis(100));
            let read = cx.background_executor().spawn(async move {
                timer.await;
                flag.store(true, std::sync::atomic::Ordering::SeqCst);
                Err(std::io::Error::other("controlled pending read"))
            });
            shell.loading = true;
            shell.await_import_read(
                PathBuf::from("controlled.pdf"),
                shell.import_token,
                read,
                cx,
            );
        });
        cx.run_until_parked();
        completed
    }

    fn pending_path_prompt(
        shell: &Entity<Shell>,
        presenter: AnyWindowHandle,
        cx: &mut TestAppContext,
    ) {
        cx.simulate_keystrokes(presenter, "ctrl-o");
        cx.run_until_parked();
        shell.update(cx, |shell, cx| shell.poll_focus(cx));
        cx.run_until_parked();
        assert!(cx.did_prompt_for_paths());
    }

    #[gpui::test]
    fn retirement_v13_presenter_close_cancels_pending_import_read(cx: &mut TestAppContext) {
        let (shell, presenter) = input_presenter(cx);
        let completed = pending_import_read(&shell, cx);
        let mut visual = gpui::VisualTestContext::from_window(presenter, cx);
        assert!(!visual.simulate_close());
        cx.run_until_parked();
        cx.executor().advance_clock(Duration::from_millis(200));
        cx.run_until_parked();
        assert!(
            !completed.load(std::sync::atomic::Ordering::SeqCst),
            "closing the owner must drop its pending read awaiter before a later delivery"
        );
        assert!(cx.windows().is_empty());
        assert!(shell.read_with(cx, |shell, _| shell.error.is_none()));
    }

    #[gpui::test]
    fn retirement_v13_shutdown_cancels_pending_import_read(cx: &mut TestAppContext) {
        let (shell, _) = input_presenter(cx);
        let completed = pending_import_read(&shell, cx);
        cx.update(|cx| cx.shutdown());
        cx.run_until_parked();
        cx.executor().advance_clock(Duration::from_millis(200));
        cx.run_until_parked();
        assert!(
            !completed.load(std::sync::atomic::Ordering::SeqCst),
            "app shutdown must cancel a pending read independently of its import token"
        );
        assert!(shell.read_with(cx, |shell, _| shell.error.is_none()));
    }

    #[gpui::test]
    fn retirement_v13_shutdown_cancels_pending_path_prompt(cx: &mut TestAppContext) {
        let (shell, presenter) = input_presenter(cx);
        pending_path_prompt(&shell, presenter, cx);
        cx.update(|cx| cx.shutdown());
        cx.run_until_parked();
        cx.simulate_path_prompt_response(|_| Some(vec![PathBuf::from("late-choice.pdf")]));
        cx.run_until_parked();
        assert!(
            shell.read_with(cx, |shell, _| shell.error.is_none()),
            "a retained entity must not process a dialog response after shutdown"
        );
    }

    #[gpui::test]
    fn retirement_v13_nonowner_close_preserves_read_and_prompt_completion(cx: &mut TestAppContext) {
        let (shell, presenter) = input_presenter(cx);
        pending_path_prompt(&shell, presenter, cx);
        let completed = pending_import_read(&shell, cx);
        let retired = shell.update(cx, |shell, cx| test_window(shell, false, cx).handle);
        let mut visual = gpui::VisualTestContext::from_window(retired, cx);
        assert!(!visual.simulate_close());
        cx.run_until_parked();
        cx.executor().advance_clock(Duration::from_millis(200));
        cx.run_until_parked();
        assert!(completed.load(std::sync::atomic::Ordering::SeqCst));
        shell.update(cx, |shell, _| {
            assert_eq!(
                shell.error.as_deref(),
                Some(shell.locale.text("read-failed"))
            );
            assert!(!shell.loading);
            shell.error = None;
        });
        cx.simulate_path_prompt_response(|_| Some(vec![PathBuf::from("live-choice.pdf")]));
        cx.run_until_parked();
        assert!(shell.read_with(cx, |shell, _| shell.error.is_some()));
        assert!(cx.windows().contains(&presenter));
    }

    #[gpui::test]
    fn retirement_v12_presenter_close_cancels_pending_poll_wake(cx: &mut TestAppContext) {
        let (shell, topology) = headless_shell(cx);
        let presenter = shell.update(cx, |shell, cx| {
            shell.presenter = Some(test_window(shell, false, cx));
            shell.start_polling(cx);
            shell.presenter.as_ref().unwrap().handle
        });
        cx.run_until_parked(); // Poll loop is suspended in its real executor timer.
        let mut visual = gpui::VisualTestContext::from_window(presenter, cx);
        assert!(!visual.simulate_close());
        cx.run_until_parked();
        assert!(cx.windows().is_empty());
        topology
            .send(TopologyEvent::Unavailable {
                error: ferrostage_platform::PlatformError::UnsupportedPlatform,
                last_known: Vec::new(),
            })
            .unwrap();
        cx.executor().advance_clock(Duration::from_millis(100));
        cx.run_until_parked();
        assert_eq!(
            shell.read_with(cx, |shell, _| shell.topology_revision),
            0,
            "a timer must not resume Shell work after the current Presenter requested quit"
        );
        assert!(cx.windows().is_empty());
    }

    #[gpui::test]
    fn retirement_v12_app_shutdown_cancels_pending_poll_wake(cx: &mut TestAppContext) {
        let (shell, topology) = headless_shell(cx);
        shell.update(cx, |shell, cx| shell.start_polling(cx));
        cx.run_until_parked();
        cx.update(|cx| cx.shutdown());
        cx.run_until_parked();
        topology
            .send(TopologyEvent::Unavailable {
                error: ferrostage_platform::PlatformError::UnsupportedPlatform,
                last_known: Vec::new(),
            })
            .unwrap();
        cx.executor().advance_clock(Duration::from_millis(100));
        cx.run_until_parked();
        assert_eq!(
            shell.read_with(cx, |shell, _| shell.topology_revision),
            0,
            "shutdown must cancel before the platform dispatcher can be destroyed"
        );
    }

    #[gpui::test]
    fn retirement_v12_nonowner_close_keeps_live_polling(cx: &mut TestAppContext) {
        let (shell, topology) = headless_shell(cx);
        let presenter = shell.update(cx, |shell, cx| {
            load_ready(shell);
            shell.presenter = Some(test_window(shell, false, cx));
            shell.start_polling(cx);
            shell.presenter.as_ref().unwrap().handle
        });
        cx.run_until_parked();
        for kind in ["retired", "pending", "audience"] {
            let closing = shell.update(cx, |shell, cx| {
                if kind == "audience" {
                    set_playing(shell, cx, Instant::now());
                    shell.audience.as_ref().unwrap().handle
                } else {
                    let window = test_window(shell, false, cx);
                    let handle = window.handle;
                    if kind == "pending" {
                        shell.pending = Some(Transaction {
                            kind: TransactionKind::RecoverPresenter,
                            presenter: Some(window),
                            audience: None,
                            deadline: Instant::now() + CONFIRM_TIMEOUT,
                        });
                    }
                    handle
                }
            });
            let mut visual = gpui::VisualTestContext::from_window(closing, cx);
            assert!(!visual.simulate_close());
            cx.run_until_parked();
            shell.read_with(cx, |shell, _| {
                assert!(
                    shell.poll_task.is_some(),
                    "{kind} is not the application owner"
                );
                assert_eq!(shell.presenter.as_ref().unwrap().handle, presenter);
            });
        }
        topology
            .send(TopologyEvent::Unavailable {
                error: ferrostage_platform::PlatformError::UnsupportedPlatform,
                last_known: Vec::new(),
            })
            .unwrap();
        cx.executor().advance_clock(Duration::from_millis(100));
        cx.run_until_parked();
        assert_eq!(
            shell.read_with(cx, |shell, _| shell.topology_revision),
            1,
            "the live coordinator must still consume topology after nonowner close"
        );
    }

    fn display_fixture(cx: &mut TestAppContext) -> (Entity<Shell>, AnyWindowHandle) {
        let (shell, handle) = input_presenter(cx);
        shell.update(cx, |shell, cx| {
            load_ready(shell);
            shell.monitors = vec![test_monitor()];
            shell.new_display_draft();
            cx.notify();
        });
        redraw(handle, cx);
        (shell, handle)
    }

    #[gpui::test]
    fn monitor_recovery_clears_only_display_error_for_unchanged_topology(cx: &mut TestAppContext) {
        let (shell, _) = display_fixture(cx);
        for operation_error in [None, Some("unrelated PDF import failed".to_string())] {
            shell.update(cx, |shell, cx| {
                let monitors = shell.monitors.clone();
                shell.error = operation_error.clone();
                shell.topology_changed(
                    TopologyEvent::Unavailable {
                        error: ferrostage_platform::PlatformError::IncompleteTopology {
                            devices: vec!["DISPLAY1".into()],
                            geometry: Vec::new(),
                        },
                        last_known: monitors.clone(),
                    },
                    cx,
                );
                assert!(!shell.topology_valid);
                shell.topology_changed(TopologyEvent::Changed(monitors), cx);
                assert!(shell.topology_valid);
                assert_eq!(
                    shell.display_panel().unwrap().operation_error,
                    operation_error,
                    "recovered display discovery must remove its stale banner and preserve unrelated errors"
                );
            });
        }
    }

    #[gpui::test]
    fn monitor_apply_recheck_failure_keeps_discovery_error_owned(cx: &mut TestAppContext) {
        for operation_error in [None, Some("unrelated PDF import failed".to_string())] {
            let (shell, _) = display_fixture(cx);
            shell.update(cx, |shell, cx| {
                shell.error = operation_error.clone();
                // Apply passes its initial editability guard, but its fresh
                // discovery fails before the draft is applied.
                assert!(shell.display_editable());
                let monitors = shell.monitors.clone();
                shell.topology_changed(
                    TopologyEvent::Unavailable {
                        error: ferrostage_platform::PlatformError::IncompleteTopology {
                            devices: vec!["DISPLAY1".into()],
                            geometry: Vec::new(),
                        },
                        last_known: monitors.clone(),
                    },
                    cx,
                );
                shell.apply_display_draft(cx);
                assert_eq!(
                    shell.error, operation_error,
                    "Apply must not copy a detection error into the operation-error owner"
                );
                assert!(shell.monitor_error.is_some());
                shell.topology_changed(TopologyEvent::Changed(monitors), cx);
                assert!(shell.monitor_error.is_none());
                assert_eq!(
                    shell.display_panel().unwrap().operation_error,
                    operation_error
                );
            });
        }
    }

    #[gpui::test]
    fn monitor_stale_draft_validation_recovers_without_operation_error(cx: &mut TestAppContext) {
        for operation_error in [None, Some("unrelated PDF import failed".to_string())] {
            let (shell, handle) = display_fixture(cx);
            shell.update(cx, |shell, cx| {
                shell.error = operation_error.clone();
                let mut changed = test_monitor();
                changed.native_size = (1280, 720);
                shell.topology_changed(TopologyEvent::Changed(vec![changed]), cx);
                // TestPlatform has no native connection identity. Retain its
                // owned mapping to isolate the stale-draft validation path.
                let display = shell.presenter.as_ref().unwrap().display;
                shell.mapped = Ok([(test_monitor().id, display)].into());
                assert_eq!(
                    shell.display_panel().unwrap().problem.as_deref(),
                    Some(shell.locale.text("displays-stale"))
                );
                shell.apply_display_draft(cx);
                assert_eq!(
                    shell.error, operation_error,
                    "live draft validation must not become a persistent operation error"
                );
            });
            redraw(handle, cx);
            let mut visual = gpui::VisualTestContext::from_window(handle, cx);
            assert!(visual.debug_bounds("display-validation").is_some());
            shell.update(cx, |shell, cx| {
                shell.new_display_draft();
                assert!(shell.display_panel().unwrap().problem.is_none());
                assert_eq!(shell.error, operation_error);
                cx.notify();
            });
            redraw(handle, cx);
            assert!(visual.debug_bounds("display-validation").is_none());
        }
    }

    #[gpui::test]
    fn monitor_unavailable_start_and_swap_shortcuts_preserve_error_ownership(
        cx: &mut TestAppContext,
    ) {
        for shortcut in ["f5", "s"] {
            for operation_error in [None, Some("unrelated PDF import failed".to_string())] {
                let (shell, handle) = input_presenter(cx);
                let settings = shell.update(cx, |shell, cx| {
                    load_ready(shell);
                    shell.monitors = vec![test_monitor()];
                    shell.error = operation_error.clone();
                    shell.topology_changed(
                        TopologyEvent::Unavailable {
                            error: ferrostage_platform::PlatformError::IncompleteTopology {
                                devices: vec!["DISPLAY1".into()],
                                geometry: Vec::new(),
                            },
                            last_known: shell.monitors.clone(),
                        },
                        cx,
                    );
                    shell.settings.settings.clone()
                });
                redraw(handle, cx);
                cx.simulate_keystrokes(handle, shortcut);
                shell.read_with(cx, |shell, _| {
                    assert_eq!(
                        shell.error, operation_error,
                        "{shortcut} must not duplicate a monitor error in the operation-error owner"
                    );
                    assert_eq!(shell.settings.settings, settings);
                    assert!(shell.pending.is_none() && shell.audience.is_none());
                    assert_eq!(shell.state.stage(), SessionStage::Ready);
                });
                shell.update(cx, |shell, cx| {
                    shell.topology_changed(TopologyEvent::Changed(vec![test_monitor()]), cx);
                    assert!(shell.monitor_error.is_none());
                    assert_eq!(shell.error, operation_error);
                });
                if shortcut == "f5" {
                    // Even a healthy discovery cannot restore a watcher that
                    // failed to start; it retains its own visible error owner.
                    redraw(handle, cx);
                    cx.simulate_keystrokes(handle, shortcut);
                    shell.read_with(cx, |shell, _| assert_eq!(shell.error, operation_error));
                }
            }
        }
    }

    #[gpui::test]
    fn monitor_watcher_failure_localizes_visible_banner_and_preserves_operation_errors(
        cx: &mut TestAppContext,
    ) {
        let (shell, events) = headless_shell(cx);
        let handle = shell.update(cx, |shell, cx| {
            load_ready(shell);
            shell.monitors = vec![test_monitor()];
            shell.presenter = Some(test_window(shell, false, cx));
            shell.presenter.as_ref().unwrap().handle
        });
        let view = handle.downcast::<Presenter>().unwrap().root(cx).unwrap();
        let failure = ferrostage_platform::PlatformError::IncompleteTopology {
            devices: vec!["DISPLAY1".into()],
            geometry: Vec::new(),
        };
        events
            .send(TopologyEvent::Unavailable {
                error: failure.clone(),
                last_known: vec![test_monitor()],
            })
            .unwrap();
        shell.update(cx, |shell, cx| {
            set_playing(shell, cx, Instant::now());
            shell.tick(cx);
            assert!(!shell.topology_valid);
            assert!(shell.start_reason().is_some());
            assert!(shell.audience.is_none());
            assert_eq!(shell.state.stage(), SessionStage::Ready);
            assert_eq!(shell.snapshot.read(cx).stage, AudienceStage::Curtain);
        });
        redraw(handle, cx);
        view.read_with(cx, |view, _| {
            assert_eq!(
                view.presented_error(),
                Some("显示器设备与屏幕位置、尺寸信息不一致。请在“设置 > 显示器”重新检测。")
            );
        });
        events
            .send(TopologyEvent::Changed(vec![test_monitor()]))
            .unwrap();
        shell.update(cx, |shell, cx| shell.tick(cx));
        redraw(handle, cx);
        view.read_with(cx, |view, _| {
            // This fixture has an intentionally absent watcher. Its independent
            // startup failure must survive recovery of the discovery failure.
            assert_eq!(view.presented_error(), Some("当前系统不支持显示器检测。"));
        });
        shell.update(cx, |shell, cx| {
            shell.error = Some("unrelated PDF import failed".into());
            shell.topology_changed(
                TopologyEvent::Unavailable {
                    error: failure,
                    last_known: shell.monitors.clone(),
                },
                cx,
            );
            shell.new_display_draft();
            let panel = shell.display_panel().unwrap();
            assert!(panel.monitor_error.unwrap().contains("位置、尺寸"));
            assert_eq!(
                panel.operation_error.as_deref(),
                Some("unrelated PDF import failed")
            );
            shell.topology_changed(TopologyEvent::Changed(vec![test_monitor()]), cx);
        });
        redraw(handle, cx);
        view.read_with(cx, |view, _| {
            assert_eq!(view.presented_error(), Some("unrelated PDF import failed"));
        });
        shell.read_with(cx, |shell, _| assert!(shell.monitor_error.is_none()));
    }

    #[gpui::test]
    fn monitor_startup_error_uses_current_locale_and_keeps_watcher_failure(
        cx: &mut TestAppContext,
    ) {
        for language in ["zh-CN", "en-US"] {
            let (_, topology) = std::sync::mpsc::channel();
            let shell = cx.new(|cx| {
                let mut settings = SettingsSession::from_outcome(
                    ferrostage_settings::from_toml("schema_version = 1").unwrap(),
                );
                settings.settings.general.language = language.into();
                Shell::new(
                    ShellInputs {
                        settings,
                        store: SettingsStore::at(PathBuf::from("unused-headless-settings.toml")),
                        renderer: Err(RenderError::FeatureDisabled),
                        topology,
                        watcher: Err(ferrostage_platform::PlatformError::WatcherStart {
                            message: "raw OS error".into(),
                        }),
                        initial_monitors: Err(
                            ferrostage_platform::PlatformError::TopologyChanging {
                                before: vec!["DISPLAY1".into()],
                                after: vec!["DISPLAY1".into()],
                            },
                        ),
                        initial_pdf: None,
                    },
                    cx,
                )
            });
            let handle = shell.update(cx, |shell, cx| {
                assert!(!shell.topology_valid);
                assert!(shell.error.is_none());
                assert_eq!(
                    shell.monitor_error_text().as_deref(),
                    Some(shell.locale.text("monitor-changing"))
                );
                assert!(shell.mapped.is_err());
                // Seed only the TestPlatform window lookup so the real first-draw
                // capture can be inspected while discovery stays unavailable.
                let display = u64::from(cx.displays().first().unwrap().id());
                shell.mapped = Ok([(test_monitor().id, display)].into());
                shell.presenter = Some(test_window(shell, false, cx));
                shell.presenter.as_ref().unwrap().handle
            });
            let view = handle.downcast::<Presenter>().unwrap().root(cx).unwrap();
            view.read_with(cx, |view, _| {
                assert_eq!(
                    view.presented_error(),
                    Some(crate::locale::Locale::new(language).text("monitor-changing"))
                );
            });
            shell.update(cx, |shell, cx| {
                shell.topology_changed(TopologyEvent::Changed(Vec::new()), cx);
                assert!(shell.monitor_error.is_none());
                assert_eq!(
                    shell.monitor_error_text().as_deref(),
                    Some(shell.locale.text("monitor-watcher-start"))
                );
            });
            redraw(handle, cx);
            view.read_with(cx, |view, _| {
                assert_eq!(
                    view.presented_error(),
                    Some(crate::locale::Locale::new(language).text("monitor-watcher-start"))
                );
            });
        }
    }

    #[gpui::test]
    fn monitor_recovery_help_and_retry_work_in_both_display_hosts(cx: &mut TestAppContext) {
        for language in ["zh-CN", "en-US"] {
            for embedded in [false, true] {
                let (shell, _) = headless_shell(cx);
                let handle = shell.update(cx, |shell, cx| {
                    shell.locale = crate::locale::Locale::new(language);
                    shell.monitors = vec![test_monitor()];
                    load_ready(shell);
                    shell.presenter = Some(test_window(shell, false, cx));
                    shell.new_display_draft();
                    if embedded {
                        shell.preferences.category = crate::settings_panel::Category::Displays;
                        shell.preferences.display = shell.display_draft().cloned();
                        shell.modal = Some(Modal::Settings);
                    }
                    shell.error = Some("unrelated PDF import failed".into());
                    shell.topology_changed(
                        TopologyEvent::Unavailable {
                            error: ferrostage_platform::PlatformError::InvalidGeometry {
                                connection: "synthetic-stale-display".into(),
                            },
                            last_known: shell.monitors.clone(),
                        },
                        cx,
                    );
                    shell.presenter.as_ref().unwrap().handle
                });
                redraw(handle, cx);
                let revision = shell.read_with(cx, |shell, _| shell.topology_revision);
                let mut visual = gpui::VisualTestContext::from_window(handle, cx);
                assert!(
                    visual.debug_bounds("display-recovery").is_some(),
                    "{language}, embedded={embedded}"
                );
                assert!(visual.debug_bounds("refresh-displays").is_some());
                click_control(handle, "apply-displays", cx);
                shell.read_with(cx, |shell, _| {
                    let panel = shell.display_panel().unwrap();
                    assert!(!panel.editable && !panel.can_identify);
                    assert!(shell.audience.is_none());
                    assert_eq!(shell.error.as_deref(), Some("unrelated PDF import failed"));
                });
                click_control(handle, "refresh-displays", cx);
                shell.read_with(cx, |shell, _| {
                    assert!(shell.topology_revision > revision, "retry must reach the existing native enumeration path");
                    assert_eq!(shell.display_draft().unwrap().revision, shell.topology_revision);
                    assert_eq!(shell.error.as_deref(), Some("unrelated PDF import failed"));
                    assert!(!matches!(&shell.monitor_error, Some(ferrostage_platform::PlatformError::InvalidGeometry { connection }) if connection == "synthetic-stale-display"));
                    if shell.topology_valid {
                        assert!(shell.monitor_error.is_none());
                    } else {
                        assert!(!shell.display_panel().unwrap().editable);
                    }
                });
                // Keep the final recovery assertion deterministic even on a host
                // whose native display query is itself unavailable.
                shell.update(cx, |shell, cx| {
                    shell.topology_changed(TopologyEvent::Changed(vec![test_monitor()]), cx);
                });
                redraw(handle, cx);
                assert!(visual.debug_bounds("display-recovery").is_none());
            }
        }
    }

    #[gpui::test]
    fn displays_design_observation_uses_native_move_callback_and_current_owner(
        cx: &mut TestAppContext,
    ) {
        use display_settings::WindowLocation;
        let (shell, handle) = display_fixture(cx);
        let (settings, sample) = shell.update(cx, |shell, _| {
            let sample = shell.presenter.as_ref().unwrap().observed_display.clone();
            assert_eq!(
                shell.display_panel().unwrap().roles.app,
                WindowLocation::Monitor(test_monitor().id)
            );
            assert_eq!(
                shell.display_panel().unwrap().roles.pdf,
                WindowLocation::NotRunning
            );
            sample.set(None);
            assert_eq!(
                shell.display_panel().unwrap().roles.app,
                WindowLocation::Unknown
            );
            (shell.settings.settings.clone(), sample)
        });
        // The platform's real on_moved callback calls bounds_changed and our observer.
        cx.test_window(handle).simulate_move();
        shell.update(cx, |shell, cx| {
            assert_eq!(
                shell.display_panel().unwrap().roles.app,
                WindowLocation::Monitor(test_monitor().id)
            );
            assert_eq!(shell.settings.settings, settings);
            set_playing(shell, cx, Instant::now());
            let pdf_sample = shell.audience.as_ref().unwrap().observed_display.clone();
            assert_eq!(
                shell.display_panel().unwrap().roles.pdf,
                WindowLocation::Monitor(test_monitor().id)
            );
            shell.stop(cx);
            shell.new_display_draft();
            pdf_sample.set(sample.get()); // A retired window cannot resurrect PDF presence.
            assert_eq!(
                shell.display_panel().unwrap().roles.pdf,
                WindowLocation::NotRunning
            );
            let replacement = test_window(shell, false, cx);
            let old = shell.presenter.replace(replacement).unwrap();
            sample.set(Some(u64::MAX));
            assert_eq!(
                shell.display_panel().unwrap().roles.app,
                WindowLocation::Monitor(test_monitor().id)
            );
            Shell::close(old, cx);
            shell.topology_valid = false;
            assert_eq!(
                shell.display_panel().unwrap().roles.app,
                WindowLocation::Unknown
            );
            assert_eq!(shell.settings.settings, settings);
        });
    }

    #[gpui::test]
    fn displays_design_both_hosts_fit_three_rows_and_cancel_only_allocation(
        cx: &mut TestAppContext,
    ) {
        let fixture = SaveFixture::new();
        fixture.unblock();
        let (shell, handle) = display_fixture(cx);
        shell.update(cx, |shell, cx| {
            shell.store = fixture.store();
            let mut left = test_monitor();
            left.id.0.push('b');
            left.position = (-2560, 0);
            left.native_size = (2560, 1600);
            left.is_primary = false;
            let mut right = test_monitor();
            right.id.0.push('c');
            right.position = (1920, 0);
            right.native_size = (1600, 2560);
            right.is_primary = false;
            shell.monitors.extend([left, right]);
            let display = shell
                .presenter
                .as_ref()
                .unwrap()
                .observed_display
                .get()
                .unwrap();
            shell.mapped = Ok(shell
                .monitors
                .iter()
                .enumerate()
                .map(|(i, m)| (m.id.clone(), display + i as u64))
                .collect());
            cx.notify();
        });
        cx.simulate_window_resize(handle, size(px(1100.), px(760.)));
        for embedded in [false, true] {
            shell.update(cx, |shell, cx| {
                shell.modal = if embedded {
                    Some(Modal::Settings)
                } else {
                    None
                };
                shell.preferences.category = crate::settings_panel::Category::Displays;
                shell.new_display_draft();
                shell.display_details.clear();
                cx.notify();
            });
            redraw(handle, cx);
            let mut visual = gpui::VisualTestContext::from_window(handle, cx);
            let topology = visual.debug_bounds("display-topology").unwrap();
            assert!(topology.size.width <= px(crate::theme::DISPLAY_PREVIEW_WIDTH));
            assert_eq!(
                topology.size.height,
                px(crate::theme::DISPLAY_PREVIEW_HEIGHT)
            );
            assert_eq!(
                visual.debug_bounds("display-panel-content").is_none(),
                embedded,
                "only standalone owns this scroll"
            );
            let content = visual
                .debug_bounds(if embedded {
                    "settings-content"
                } else {
                    "display-panel-content"
                })
                .unwrap();
            for name in [
                "display-card-1",
                "display-card-2",
                "display-card-3",
                "display-role-summary",
                "apply-displays",
                "cancel-displays",
            ] {
                let bounds = visual.debug_bounds(name).unwrap();
                let (top, bottom) =
                    if !embedded && matches!(name, "apply-displays" | "cancel-displays") {
                        (px(0.), px(760.)) // Standalone allocation actions live outside its body scroll.
                    } else {
                        (content.top(), content.bottom())
                    };
                assert!(
                    bounds.top() >= top && bounds.bottom() <= bottom,
                    "{embedded} {name}: {bounds:?}, visible y={top:?}..{bottom:?}"
                );
            }
            assert!(visual.debug_bounds("display-diagnostics-1").is_none());
            click_control(handle, "display-details-1", cx);
            redraw(handle, cx);
            assert!(visual.debug_bounds("display-diagnostics-1").is_some());
            click_control(handle, "display-details-1", cx);
            redraw(handle, cx);
            let actual = shell.read_with(cx, |shell, _| shell.display_panel().unwrap().roles.app);
            click_control(handle, "display-pdf-3", cx);
            redraw(handle, cx);
            let pending_bottom = if embedded {
                visual.debug_bounds("settings-content").unwrap().bottom()
            } else {
                px(760.)
            };
            for action in ["apply-displays", "cancel-displays"] {
                assert!(
                    visual.debug_bounds(action).unwrap().bottom() <= pending_bottom,
                    "{embedded} {action} must remain visible with pending allocation feedback"
                );
            }
            shell.read_with(cx, |shell, _| {
                let panel = shell.display_panel().unwrap();
                assert!(panel.roles.pending);
                assert_eq!(panel.roles.app, actual);
                assert_eq!(
                    panel.roles.pdf,
                    display_settings::WindowLocation::NotRunning
                );
                assert!(shell.pending.is_none());
            });
            if embedded {
                // This preference is already saved, independent of the allocation draft.
                shell.update(cx, |shell, cx| {
                    shell.settings.settings.display.audience_fullscreen = false;
                    shell.settings.note_edit();
                    shell.save(cx);
                });
            }
            click_control(handle, "cancel-displays", cx);
            redraw(handle, cx);
            shell.read_with(cx, |shell, _| {
                if embedded {
                    assert_eq!(shell.modal, Some(Modal::Settings));
                    assert!(!shell.display_panel().unwrap().roles.pending);
                    assert!(!shell.settings.settings.display.audience_fullscreen);
                    assert!(
                        !shell
                            .store
                            .load()
                            .unwrap()
                            .settings
                            .display
                            .audience_fullscreen
                    );
                } else {
                    assert!(shell.modal.is_none());
                }
            });
        }
    }

    #[gpui::test]
    fn displays_v8_mouse_draft_cancel_keeps_windows_core_and_disk(cx: &mut TestAppContext) {
        let fixture = SaveFixture::new();
        fixture.unblock();
        let (shell, handle) = display_fixture(cx);
        let (settings, state) = shell.update(cx, |shell, cx| {
            shell.store = fixture.store();
            shell.save(cx);
            (shell.settings.settings.clone(), shell.state.clone())
        });
        let bytes = std::fs::read(fixture.0.join("blocked/settings.toml")).unwrap();
        click_control(handle, "display-pdf-windowed", cx);
        shell.read_with(cx, |shell, _| {
            assert!(matches!(&shell.modal, Some(Modal::Displays(draft)) if draft.pdf == PdfTarget::Windowed));
            assert_eq!(shell.settings.settings, settings);
            assert_eq!(shell.state, state);
            assert!(shell.pending.is_none() && shell.audience.is_none());
        });
        redraw(handle, cx);
        click_control(handle, "cancel-displays", cx);
        assert_eq!(cx.windows(), vec![handle]);
        assert_eq!(
            std::fs::read(fixture.0.join("blocked/settings.toml")).unwrap(),
            bytes
        );
        redraw(handle, cx);
        assert_tab_opens_pdf(&shell, handle, cx);
    }

    #[gpui::test]
    fn displays_v8_future_settings_disable_mouse_keyboard_and_apply(cx: &mut TestAppContext) {
        let (shell, handle) = display_fixture(cx);
        let original = shell.update(cx, |shell, cx| {
            shell.settings = SettingsSession::from_outcome(
                ferrostage_settings::from_toml("schema_version = 999").unwrap(),
            );
            shell.new_display_draft();
            cx.notify();
            shell.settings.settings.clone()
        });
        redraw(handle, cx);
        click_control(handle, "display-pdf-windowed", cx);
        click_control(handle, "apply-displays", cx);
        shell.update(cx, |shell, cx| {
            shell.dispatch(Intent::SelectPdf(PdfTarget::Windowed), cx);
            shell.dispatch(Intent::ApplyDisplays, cx);
            assert_eq!(shell.settings.settings, original);
            assert!(!shell.settings.dirty);
            assert!(shell.pending.is_none());
            assert!(
                shell
                    .display_panel()
                    .unwrap()
                    .problem
                    .unwrap()
                    .contains(shell.locale.text("displays-readonly"))
            );
        });
        assert_eq!(cx.windows(), vec![handle]);
        // Begin keyboard traversal from the modal's entry focus, independently
        // of the preceding clicks on disabled controls.
        cx.simulate_keystrokes(handle, "escape");
        redraw(handle, cx);
        shell.update(cx, |shell, cx| {
            shell.new_display_draft();
            cx.notify();
        });
        redraw(handle, cx);
        cx.simulate_keystrokes(handle, "shift-tab");
        redraw(handle, cx);
        cx.simulate_keystrokes(handle, "enter"); // Disabled Apply is skipped; Cancel closes.
        assert!(shell.read_with(cx, |shell, _| shell.modal.is_none()));
    }

    #[gpui::test]
    fn placement_v11_windows_frame_delta_is_corrected_from_actual_readback(
        cx: &mut TestAppContext,
    ) {
        let (shell, handle) = input_presenter(cx);
        shell.update(cx, |shell, _| {
            shell.presenter.as_mut().unwrap().client_size = size(px(1002.6666), px(760.));
        });
        cx.simulate_window_resize(handle, size(px(1202.6666), px(912.)));
        for _ in 0..2 {
            shell.update(cx, |shell, cx| {
                Shell::restore_client_size(shell.presenter.as_ref().unwrap(), cx);
            });
            // TestPlatform's resize is exact. Feed the actual Windows boundary
            // response through its real resize callback: at150%, the cached
            // non-client conversion in V10 lost4x9 physical pixels.
            let requested = handle
                .update(cx, |_, window, _| window.bounds().size)
                .unwrap();
            cx.simulate_window_resize(
                handle,
                size(
                    px(((f32::from(requested.width) * 1.5).round() - 4.) / 1.5),
                    px(((f32::from(requested.height) * 1.5).round() - 9.) / 1.5),
                ),
            );
        }
        shell.update(cx, |shell, cx| {
            assert!(Shell::confirmed(shell.presenter.as_ref().unwrap(), cx));
        });
    }

    #[gpui::test]
    fn placement_v11_large_or_nonconvergent_frame_response_stops_and_rolls_back(
        cx: &mut TestAppContext,
    ) {
        for error in [size(px(33.), px(6.)), size(px(4. / 1.5), px(6.))] {
            let (shell, owner) = display_fixture(cx);
            let (state, settings, audience, replacement, target) = shell.update(cx, |shell, cx| {
                set_playing(shell, cx, Instant::now() - Duration::from_secs(10));
                let state = shell.state.clone();
                let settings = shell.settings.settings.clone();
                let audience = shell.audience.as_ref().unwrap().handle;
                shell.new_display_draft();
                shell.dispatch(Intent::SelectPdf(PdfTarget::None), cx);
                shell.apply_display_draft(cx);
                let pending = shell.pending.as_ref().unwrap().presenter.as_ref().unwrap();
                (
                    state,
                    settings,
                    audience,
                    pending.handle,
                    pending.client_size,
                )
            });
            cx.simulate_window_resize(replacement, target.map(|value| value * 1.2));
            shell.update(cx, |shell, cx| {
                Shell::restore_client_size(
                    shell.pending.as_ref().unwrap().presenter.as_ref().unwrap(),
                    cx,
                );
            });
            cx.simulate_window_resize(replacement, target - error);
            shell.update(cx, |shell, cx| shell.poll_transaction(cx));
            shell.read_with(cx, |shell, _| {
                let attempt = shell
                    .pending
                    .as_ref()
                    .unwrap()
                    .presenter
                    .as_ref()
                    .unwrap()
                    .resize_attempt
                    .get()
                    .unwrap();
                assert_eq!(attempt.corrected, error.width <= px(32.));
            });
            // Reverse a small native error instead of converging. The correction
            // budget is spent, so another callback cannot start oscillating retries.
            let uncorrected = if error.width <= px(32.) {
                target + error
            } else {
                target - error
            };
            cx.simulate_window_resize(replacement, uncorrected);
            for _ in 0..3 {
                shell.update(cx, |shell, cx| shell.poll_transaction(cx));
                assert_eq!(
                    replacement
                        .update(cx, |_, window, _| window.bounds().size)
                        .unwrap(),
                    uncorrected
                );
            }
            shell.update(cx, |shell, cx| {
                shell.pending.as_mut().unwrap().deadline = Instant::now() - Duration::from_secs(1);
                shell.poll_transaction(cx);
                assert!(shell.pending.is_none());
                assert_eq!(shell.state, state);
                assert_eq!(shell.settings.settings, settings);
                assert_eq!(shell.presenter.as_ref().unwrap().handle, owner);
                assert_eq!(shell.audience.as_ref().unwrap().handle, audience);
                assert_eq!(shell.snapshot.read(cx).stage, AudienceStage::Live);
            });
            assert!(!cx.windows().contains(&replacement));
        }
    }

    #[gpui::test]
    fn placement_v10_scaled_client_size_cannot_commit(cx: &mut TestAppContext) {
        let (shell, handle) = input_presenter(cx);
        cx.simulate_window_resize(handle, size(px(1320.), px(912.)));
        shell.update(cx, |shell, cx| {
            let presenter = shell.presenter.as_ref().unwrap();
            assert!(!Shell::confirmed(presenter, cx));
            Shell::restore_client_size(presenter, cx);
            assert!(Shell::confirmed(presenter, cx));
        });
    }

    #[gpui::test]
    fn narrow_v10_display_panel_keeps_apply_cancel_inside_client(cx: &mut TestAppContext) {
        for language in ["zh-CN", "en-US"] {
            let (shell, _) = headless_shell(cx);
            let handle = shell.update(cx, |shell, cx| {
                shell.locale = crate::locale::Locale::new(language);
                load_ready(shell);
                shell.monitors = vec![test_monitor()];
                for suffix in ["b", "c"] {
                    let mut monitor = test_monitor();
                    monitor.id.0.push_str(suffix);
                    monitor.name = format!("Display {suffix}");
                    monitor.is_primary = false;
                    shell.monitors.push(monitor);
                }
                shell.presenter = Some(test_window(shell, false, cx));
                shell.presenter.as_ref().unwrap().handle
            });
            for (width, height) in [(800., 600.), (390., 844.), (320., 640.)] {
                shell.update(cx, |shell, cx| {
                    shell.new_display_draft();
                    cx.notify();
                });
                cx.simulate_window_resize(handle, size(px(width), px(height)));
                redraw(handle, cx);
                let mut visual = gpui::VisualTestContext::from_window(handle, cx);
                for control in ["apply-displays", "cancel-displays"] {
                    let b = visual.debug_bounds(control).unwrap();
                    assert!(
                        b.left() >= px(0.)
                            && b.top() >= px(0.)
                            && b.right() <= px(width)
                            && b.bottom() <= px(height),
                        "{language} {width}x{height} {control}: {b:?}"
                    );
                    assert!(b.size.height >= px(40.), "footer control must not collapse");
                }
                visual.simulate_event(gpui::ScrollWheelEvent {
                    position: point(px(width / 2.), px(height / 2.)),
                    delta: gpui::ScrollDelta::Pixels(point(px(0.), px(-3000.))),
                    ..Default::default()
                });
                redraw(handle, cx);
                let b = visual.debug_bounds("display-pdf-3").unwrap();
                let footer = visual.debug_bounds("apply-displays").unwrap();
                assert!(
                    b.left() >= px(0.)
                        && b.top() >= px(0.)
                        && b.right() <= px(width)
                        && b.bottom() <= footer.top(),
                    "last selector must be reachable above footer: {b:?}"
                );
                click_control(handle, "display-pdf-3", cx);
                shell.read_with(cx, |shell, _| {
                    assert!(matches!(&shell.modal, Some(Modal::Displays(draft)) if draft.pdf == PdfTarget::Display(shell.monitors[2].id.clone())));
                });
                click_control(handle, "cancel-displays", cx);
                assert!(shell.read_with(cx, |shell, _| shell.modal.is_none()));
                redraw(handle, cx);
            }
        }
    }

    #[gpui::test]
    fn placement_v10_uncorrected_client_times_out_and_keeps_running_owner(cx: &mut TestAppContext) {
        let (shell, owner) = display_fixture(cx);
        let (state, settings, audience, replacement) = shell.update(cx, |shell, cx| {
            set_playing(shell, cx, Instant::now() - Duration::from_secs(10));
            let before = shell.state.clone();
            let settings = shell.settings.settings.clone();
            let audience = shell.audience.as_ref().unwrap().handle;
            shell.new_display_draft();
            shell.dispatch(Intent::SelectPdf(PdfTarget::None), cx);
            shell.apply_display_draft(cx);
            let replacement = shell
                .pending
                .as_ref()
                .unwrap()
                .presenter
                .as_ref()
                .unwrap()
                .handle;
            (before, settings, audience, replacement)
        });
        cx.simulate_window_resize(replacement, size(px(1320.), px(912.)));
        shell.update(cx, |shell, cx| {
            Shell::restore_client_size(
                shell.pending.as_ref().unwrap().presenter.as_ref().unwrap(),
                cx,
            );
        });
        // Model no native response: an old queued callback repeats the dimensions
        // from before the request. It cannot authorize a compensating resize.
        cx.simulate_window_resize(replacement, size(px(1320.), px(912.)));
        shell.update(cx, |shell, cx| {
            shell.poll_transaction(cx);
            assert!(
                shell.pending.is_some(),
                "no early commit on correct display with wrong client size"
            );
            assert!(
                !shell
                    .pending
                    .as_ref()
                    .unwrap()
                    .presenter
                    .as_ref()
                    .unwrap()
                    .resize_attempt
                    .get()
                    .unwrap()
                    .corrected
            );
            shell.pending.as_mut().unwrap().deadline = Instant::now() - Duration::from_secs(1);
            shell.poll_transaction(cx);
            assert!(shell.pending.is_none());
            assert_eq!(shell.state, state);
            assert_eq!(shell.settings.settings, settings);
            assert_eq!(shell.presenter.as_ref().unwrap().handle, owner);
            assert_eq!(shell.audience.as_ref().unwrap().handle, audience);
            assert_eq!(shell.snapshot.read(cx).stage, AudienceStage::Live);
        });
        assert!(!cx.windows().contains(&replacement));
    }

    #[gpui::test]
    fn displays_v9_conflict_and_offline_draft_refuse_apply_from_real_controls(
        cx: &mut TestAppContext,
    ) {
        let fixture = SaveFixture::new();
        fixture.unblock();
        let (shell, handle) = display_fixture(cx);
        // TestPlatform has one native display. These extra owned records test
        // list/selector input only; no physical mapping or placement is claimed.
        let (settings, state, selected_pdf) = shell.update(cx, |shell, cx| {
            for suffix in ["b", "c"] {
                let mut monitor = test_monitor();
                monitor.id.0.push_str(suffix);
                monitor.connection_name.push_str(suffix);
                monitor.name = format!("Display {suffix}");
                monitor.is_primary = false;
                shell.monitors.push(monitor);
            }
            shell.store = fixture.store();
            shell.save(cx);
            shell.new_display_draft();
            cx.notify();
            (
                shell.settings.settings.clone(),
                shell.state.clone(),
                shell.monitors[1].id.clone(),
            )
        });
        let bytes = std::fs::read(fixture.0.join("blocked/settings.toml")).unwrap();
        cx.simulate_window_resize(handle, size(px(1200.), px(1100.)));
        redraw(handle, cx);
        click_control(handle, "display-pdf-2", cx);
        redraw(handle, cx);
        click_control(handle, "display-app-2", cx);
        redraw(handle, cx);
        shell.read_with(cx, |shell, _| {
            let panel = shell.display_panel().unwrap();
            assert_eq!(panel.draft.presenter.as_ref(), Some(&selected_pdf));
            assert_eq!(panel.draft.pdf, PdfTarget::Display(selected_pdf.clone()));
            assert_eq!(
                panel.problem.as_deref(),
                Some(shell.locale.text("displays-conflict"))
            );
        });
        click_control(handle, "apply-displays", cx);
        shell.read_with(cx, |shell, _| {
            assert_eq!(shell.settings.settings, settings);
            assert_eq!(shell.state, state);
            assert!(shell.pending.is_none() && shell.audience.is_none());
        });
        click_control(handle, "display-app-1", cx);
        shell.update(cx, |shell, cx| {
            assert!(shell.display_panel().unwrap().problem.is_none());
            let remaining = shell
                .monitors
                .iter()
                .filter(|m| m.id != selected_pdf)
                .cloned()
                .collect();
            shell.topology_changed(TopologyEvent::Changed(remaining), cx);
            let panel = shell.display_panel().unwrap();
            assert_eq!(panel.draft.pdf, PdfTarget::Display(selected_pdf));
            assert_ne!(panel.draft.revision, shell.topology_revision);
            assert_eq!(
                panel.problem.as_deref(),
                Some(shell.locale.text("displays-stale"))
            );
        });
        redraw(handle, cx);
        click_control(handle, "apply-displays", cx);
        shell.read_with(cx, |shell, _| {
            assert_eq!(shell.settings.settings, settings);
            assert_eq!(shell.state, state);
            assert!(shell.pending.is_none() && shell.audience.is_none());
        });
        assert_eq!(cx.windows(), vec![handle]);
        assert_eq!(
            std::fs::read(fixture.0.join("blocked/settings.toml")).unwrap(),
            bytes
        );
    }

    #[gpui::test]
    fn displays_v8_topology_revision_invalidates_draft_and_distinguishes_failure(
        cx: &mut TestAppContext,
    ) {
        let (shell, _) = display_fixture(cx);
        shell.update(cx, |shell, cx| {
            let old_settings = shell.settings.settings.clone();
            let mut changed = test_monitor();
            changed.native_size = (1280, 720);
            shell.topology_changed(TopologyEvent::Changed(vec![changed]), cx);
            assert!(matches!(&shell.modal, Some(Modal::Displays(draft)) if draft.revision != shell.topology_revision));
            shell.apply_display_draft(cx);
            assert_eq!(shell.display_panel().unwrap().problem.as_deref(), Some(shell.locale.text("displays-stale")));
            assert!(shell.error.is_none());
            assert_eq!(shell.settings.settings, old_settings);
            shell.topology_changed(TopologyEvent::Unavailable {
                error: ferrostage_platform::PlatformError::UnsupportedPlatform,
                last_known: shell.monitors.clone(),
            }, cx);
            let panel = shell.display_panel().unwrap();
            assert!(!panel.topology_valid && !panel.editable);
            assert_eq!(panel.problem.as_deref(), Some(shell.locale.text("displays-unavailable")));
            shell.topology_changed(TopologyEvent::Changed(vec![]), cx);
            let panel = shell.display_panel().unwrap();
            assert!(panel.topology_valid && panel.monitors.is_empty() && !panel.editable);
            assert_eq!(panel.problem.as_deref(), Some(shell.locale.text("displays-empty")));
            assert_eq!(shell.settings.settings, old_settings);
        });
    }

    fn set_playing(shell: &mut Shell, cx: &mut Context<Shell>, started: Instant) {
        shell
            .state
            .apply_at(
                Command::StartPresentation {
                    audience_available: true,
                },
                started,
            )
            .unwrap();
        shell.state.apply_at(Command::NextStep, started).unwrap();
        shell
            .state
            .apply_at(Command::ToggleBlank(BlankMode::White), started)
            .unwrap();
        shell.audience = Some(test_window(shell, true, cx));
        shell.publish(false, cx);
    }

    #[gpui::test]
    fn displays_v8_valid_migration_preserves_page_blank_timer_and_save_retry(
        cx: &mut TestAppContext,
    ) {
        let fixture = SaveFixture::new();
        let (shell, old_presenter) = display_fixture(cx);
        let started = Instant::now() - Duration::from_secs(10);
        let (old_audience, replacement) = shell.update(cx, |shell, cx| {
            shell.store = fixture.store();
            set_playing(shell, cx, started);
            let before = shell.state.clone();
            let old_audience = shell.audience.as_ref().unwrap().handle;
            shell.new_display_draft();
            shell.dispatch(Intent::SelectPdf(PdfTarget::Windowed), cx);
            // TestPlatform has no OS/GDI identity. Supply its already tested
            // owned mapping and exercise the real preparation/confirmation.
            shell.apply_display_draft(cx);
            assert_eq!(
                shell.settings.settings.display.single_monitor_mode,
                ferrostage_settings::SingleMonitorMode::PresenterOnly
            );
            assert_eq!(shell.snapshot.read(cx).stage, AudienceStage::Curtain);
            shell.poll_transaction(cx);
            assert_eq!(shell.state, before);
            assert_eq!(
                shell.state.elapsed_at(started + Duration::from_secs(20)),
                Duration::from_secs(20)
            );
            assert_eq!(shell.snapshot.read(cx).blank_mode, Some(BlankMode::White));
            assert!(shell.settings.dirty && shell.settings.save_error.is_some());
            (old_audience, shell.presenter.as_ref().unwrap().handle)
        });
        assert!(!cx.windows().contains(&old_presenter));
        assert!(!cx.windows().contains(&old_audience));
        assert_eq!(cx.windows().len(), 2);
        fixture.unblock();
        shell.update(cx, |shell, cx| {
            shell.dispatch(Intent::RetrySave, cx);
            assert_eq!(
                shell.store.load().unwrap().settings.display,
                shell.settings.settings.display
            );
            assert!(!shell.settings.dirty && shell.settings.save_error.is_none());
        });
        redraw(replacement, cx);
    }

    #[gpui::test]
    fn displays_v8_no_play_waits_for_success_then_stops_freezes_and_closes(
        cx: &mut TestAppContext,
    ) {
        let fixture = SaveFixture::new();
        let (shell, old_presenter) = display_fixture(cx);
        let started = Instant::now() - Duration::from_secs(10);
        let old_audience = shell.update(cx, |shell, cx| {
            shell.store = fixture.store();
            set_playing(shell, cx, started);
            shell.new_display_draft();
            shell.dispatch(Intent::SelectPdf(PdfTarget::None), cx);
            let audience = shell.audience.as_ref().unwrap().handle;
            shell.apply_display_draft(cx);
            assert_eq!(shell.state.stage(), SessionStage::Presenting);
            assert_eq!(shell.audience.as_ref().unwrap().handle, audience);
            assert_eq!(shell.snapshot.read(cx).stage, AudienceStage::Curtain);
            shell.poll_transaction(cx);
            assert_eq!(shell.state.stage(), SessionStage::Ready);
            assert_eq!(shell.state.current_page(), PhysicalPage(1));
            assert!(shell.audience.is_none());
            assert_eq!(shell.snapshot.read(cx).stage, AudienceStage::Curtain);
            let elapsed = shell.state.elapsed_at(Instant::now());
            assert!(elapsed >= Duration::from_secs(10));
            assert_eq!(
                shell
                    .state
                    .elapsed_at(Instant::now() + Duration::from_secs(100)),
                elapsed
            );
            assert!(
                policy::audience_target(&shell.monitors, &shell.settings.settings.display).is_err()
            );
            audience
        });
        assert_eq!(cx.windows().len(), 1);
        assert!(!cx.windows().contains(&old_audience));
        assert!(!cx.windows().contains(&old_presenter));
    }

    #[gpui::test]
    fn displays_v8_failed_no_play_placement_and_creation_roll_back_running_session(
        cx: &mut TestAppContext,
    ) {
        let (shell, presenter) = display_fixture(cx);
        shell.update(cx, |shell, cx| {
            let started = Instant::now() - Duration::from_secs(10);
            set_playing(shell, cx, started);
            let before = shell.state.clone();
            let settings = shell.settings.settings.clone();
            let audience = shell.audience.as_ref().unwrap().handle;
            shell.new_display_draft();
            shell.dispatch(Intent::SelectPdf(PdfTarget::None), cx);
            shell.apply_display_draft(cx);
            let pending = shell.pending.as_mut().unwrap();
            pending.presenter.as_mut().unwrap().display = u64::MAX;
            pending.deadline = Instant::now() - Duration::from_secs(1);
            shell.poll_transaction(cx);
            assert_eq!(shell.state, before);
            assert_eq!(shell.settings.settings, settings);
            assert_eq!(shell.presenter.as_ref().unwrap().handle, presenter);
            assert_eq!(shell.audience.as_ref().unwrap().handle, audience);
            assert_eq!(shell.snapshot.read(cx).stage, AudienceStage::Live);
            assert!(shell.pending.is_none());
            let candidate = DisplayDraft {
                revision: shell.topology_revision,
                presenter: Some(test_monitor().id),
                pdf: PdfTarget::None,
            }
            .settings(
                shell.topology_revision,
                &shell.monitors,
                &shell.settings.settings.display,
            )
            .unwrap();
            shell.mapped = Err(ShellError::AmbiguousDisplays);
            shell.assign_displays(candidate, cx);
            assert!(shell.pending.is_none());
            assert_eq!(shell.state, before);
            assert_eq!(shell.settings.settings, settings);
            assert_eq!(cx.windows().len(), 2);
        });
    }

    #[gpui::test]
    fn displays_v8_identify_is_label_only_expires_and_is_refused_presenting(
        cx: &mut TestAppContext,
    ) {
        let (shell, presenter) = display_fixture(cx);
        shell.update(cx, |shell, cx| {
            shell.open_identify(cx);
            assert_eq!(shell.identify.len(), 1);
            let label = shell.identify[0]
                .downcast::<Identify>()
                .unwrap()
                .root(cx)
                .unwrap();
            assert_eq!(label.read(cx).number, 1);
            let caption = crate::display_settings::screen_copy(
                shell.locale,
                &shell.monitors,
                &shell.monitors[0],
            );
            assert_eq!(label.read(cx).name, caption.identify);
            assert!(shell.audience.is_none());
            shell.identify_deadline = Some(Instant::now() - Duration::from_secs(1));
            shell.tick(cx);
            assert!(shell.identify.is_empty());
            assert_eq!(cx.windows(), vec![presenter]);
            set_playing(shell, cx, Instant::now());
            shell.new_display_draft();
            shell.dispatch(Intent::Identify, cx);
            assert!(shell.identify.is_empty());
            assert!(!shell.display_panel().unwrap().can_identify);
        });
    }

    #[derive(Clone)]
    struct CapturedLog(std::sync::Arc<std::sync::Mutex<Vec<u8>>>);
    impl std::io::Write for CapturedLog {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    #[gpui::test]
    fn retirement_v8_queued_activation_does_not_visit_removed_window(cx: &mut TestAppContext) {
        let (output, _capture) = capture_window_errors();
        let (shell, _) = headless_shell(cx);
        let current = shell.update(cx, |shell, cx| {
            load_ready(shell);
            shell.presenter = Some(test_window(shell, false, cx));
            let old = shell.presenter.take().unwrap();
            old.handle
                .update(cx, |_, window, _| window.activate_window())
                .unwrap();
            shell.presenter = Some(test_window(shell, false, cx));
            shell
                .presenter
                .as_ref()
                .unwrap()
                .handle
                .update(cx, |_, window, _| window.activate_window())
                .unwrap();
            Shell::close(old, cx);
            shell.presenter.as_ref().unwrap().handle
        });
        cx.run_until_parked();
        assert_eq!(cx.windows(), vec![current]);
        assert_eq!(cx.update(|cx| cx.active_window()), Some(current));
        let log = String::from_utf8(output.lock().unwrap().clone()).unwrap();
        assert!(!log.contains("window not found"), "late activation: {log}");
        shell.read_with(cx, |shell, _| {
            assert_eq!(shell.state.stage(), SessionStage::Ready);
            assert_eq!(shell.import_token, 0);
        });
    }

    fn capture_window_errors() -> (
        std::sync::Arc<std::sync::Mutex<Vec<u8>>>,
        tracing::dispatcher::DefaultGuard,
    ) {
        use tracing_subscriber::util::SubscriberInitExt as _;
        let output = std::sync::Arc::new(std::sync::Mutex::new(Vec::<u8>::new()));
        let writer = CapturedLog(output.clone());
        let guard = tracing_subscriber::fmt()
            .with_ansi(false)
            .with_file(true)
            .with_line_number(true)
            .with_max_level(tracing::Level::ERROR)
            .with_writer(move || writer.clone())
            .finish()
            .set_default();
        (output, guard)
    }

    #[gpui::test]
    fn retirement_v14_retained_hover_cannot_visit_removed_window(cx: &mut TestAppContext) {
        let (output, _capture) = capture_window_errors();
        let window: AnyWindowHandle = cx.add_window(|_, _| gpui::EmptyView).into();
        let platform = cx.test_window(window);
        window
            .update(cx, |_, window, _| window.remove_window())
            .unwrap();
        assert!(cx.windows().is_empty());
        platform.simulate_hover_status_change(false);
        assert!(
            output.lock().unwrap().is_empty(),
            "{}",
            String::from_utf8_lossy(&output.lock().unwrap())
        );
    }

    #[gpui::test]
    fn retirement_v14_retained_notifications_cannot_visit_removed_window(cx: &mut TestAppContext) {
        let (output, _capture) = capture_window_errors();
        let window: AnyWindowHandle = cx.add_window(|_, _| gpui::EmptyView).into();
        let mut platform = cx.test_window(window);
        window
            .update(cx, |_, window, _| window.remove_window())
            .unwrap();
        assert!(cx.windows().is_empty());
        platform.simulate_visibility_change(gpui::WindowVisibility::Hidden);
        platform.simulate_visual_viewport_change(gpui::Bounds::default());
        platform.simulate_insets_change(gpui::WindowInsets::default());
        platform.simulate_resize(size(px(320.), px(640.)));
        platform.simulate_move();
        platform.simulate_button_layout_change();
        platform.simulate_tab_commands();
        platform.simulate_appearance_change(gpui::WindowAppearance::Dark);
        cx.run_until_parked();
        assert!(
            output.lock().unwrap().is_empty(),
            "{}",
            String::from_utf8_lossy(&output.lock().unwrap())
        );
    }

    #[gpui::test]
    fn retirement_v14_retained_input_hit_test_and_frame_are_inert(cx: &mut TestAppContext) {
        let (output, _capture) = capture_window_errors();
        let window: AnyWindowHandle = cx.add_window(|_, _| gpui::EmptyView).into();
        let mut platform = cx.test_window(window);
        window
            .update(cx, |_, window, _| window.remove_window())
            .unwrap();
        assert!(cx.windows().is_empty());
        assert!(
            platform.simulate_input(gpui::PlatformInput::MouseMove(gpui::MouseMoveEvent {
                position: gpui::point(px(5.), px(6.)),
                pressed_button: None,
                modifiers: gpui::Modifiers::default(),
            }))
        );
        assert!(platform.simulate_window_control_hit_test().is_none());
        let wakes = platform.frame_wake_count();
        platform.simulate_frame_request(gpui::RequestFrameOptions::default());
        assert_eq!(platform.frame_wake_count(), wakes);
        assert!(
            output.lock().unwrap().is_empty(),
            "{}",
            String::from_utf8_lossy(&output.lock().unwrap())
        );
    }

    #[gpui::test]
    fn retirement_v14_deferred_appearance_rechecks_owner(cx: &mut TestAppContext) {
        let (output, _capture) = capture_window_errors();
        let window: AnyWindowHandle = cx.add_window(|_, _| gpui::EmptyView).into();
        cx.test_window(window)
            .simulate_appearance_change(gpui::WindowAppearance::Dark);
        window
            .update(cx, |_, window, _| window.remove_window())
            .unwrap();
        cx.run_until_parked();
        assert!(
            output.lock().unwrap().is_empty(),
            "{}",
            String::from_utf8_lossy(&output.lock().unwrap())
        );
    }

    #[gpui::test]
    fn retirement_v14_live_callbacks_apply_and_leased_errors_remain_visible(
        cx: &mut TestAppContext,
    ) {
        let (output, _capture) = capture_window_errors();
        let window: AnyWindowHandle = cx.add_window(|_, _| gpui::EmptyView).into();
        let mut platform = cx.test_window(window);
        platform.simulate_visibility_change(gpui::WindowVisibility::Hidden);
        platform.simulate_resize(size(px(320.), px(640.)));
        platform.simulate_appearance_change(gpui::WindowAppearance::Dark);
        cx.run_until_parked();
        window
            .update(cx, |_, window, _| {
                assert_eq!(window.visibility(), gpui::WindowVisibility::Hidden);
                assert_eq!(window.viewport_size(), size(px(320.), px(640.)));
                assert_eq!(window.appearance(), gpui::WindowAppearance::Dark);
            })
            .unwrap();
        assert!(output.lock().unwrap().is_empty());
        cx.with_window_lease(window, || platform.simulate_hover_status_change(false));
        let log = String::from_utf8(output.lock().unwrap().clone()).unwrap();
        assert!(
            log.contains("window not found"),
            "live leased window must still report its update error: {log}"
        );
    }

    #[gpui::test]
    fn retirement_v8_user_close_is_deferred_and_stops_projection(cx: &mut TestAppContext) {
        let (shell, presenter) = input_presenter(cx);
        let audience = shell.update(cx, |shell, cx| {
            load_ready(shell);
            shell
                .state
                .apply_at(
                    Command::StartPresentation {
                        audience_available: true,
                    },
                    Instant::now(),
                )
                .unwrap();
            shell.audience = Some(test_window(shell, true, cx));
            shell.audience.as_ref().unwrap().handle
        });
        let mut visual = gpui::VisualTestContext::from_window(audience, cx);
        assert!(
            !visual.simulate_close(),
            "OS must not destroy the native window first"
        );
        assert!(
            cx.windows().contains(&audience),
            "remove after the callback returns"
        );
        cx.run_until_parked();
        assert_eq!(cx.windows(), vec![presenter]);
        shell.read_with(cx, |shell, _| {
            assert_eq!(shell.state.stage(), SessionStage::Ready);
            assert!(shell.audience.is_none());
        });
        let mut visual = gpui::VisualTestContext::from_window(presenter, cx);
        assert!(!visual.simulate_close());
        cx.run_until_parked();
        assert!(cx.windows().is_empty());
        shell.read_with(cx, |shell, _| assert_eq!(shell.import_token, 1));
    }

    #[gpui::test]
    fn retirement_v8_pending_and_retired_close_requests_cannot_quit_replacement(
        cx: &mut TestAppContext,
    ) {
        let (shell, original) = input_presenter(cx);
        let pending = shell.update(cx, |shell, cx| {
            load_ready(shell);
            let window = test_window(shell, false, cx);
            let handle = window.handle;
            shell.pending = Some(Transaction {
                kind: TransactionKind::RecoverPresenter,
                presenter: Some(window),
                audience: None,
                deadline: Instant::now() + CONFIRM_TIMEOUT,
            });
            handle
        });
        let mut visual = gpui::VisualTestContext::from_window(pending, cx);
        assert!(!visual.simulate_close());
        cx.run_until_parked();
        assert_eq!(cx.windows(), vec![original]);
        shell.read_with(cx, |shell, _| {
            assert!(shell.pending.is_none());
            assert_eq!(shell.import_token, 0);
        });

        let mut visual = gpui::VisualTestContext::from_window(original, cx);
        assert!(!visual.simulate_close());
        let replacement = shell.update(cx, |shell, cx| {
            shell.pending = Some(Transaction {
                kind: TransactionKind::RecoverPresenter,
                presenter: Some(test_window(shell, false, cx)),
                audience: None,
                deadline: Instant::now() + CONFIRM_TIMEOUT,
            });
            shell.poll_transaction(cx); // Retire the old owner before queued close consumes it.
            set_playing(shell, cx, Instant::now());
            shell.presenter.as_ref().unwrap().handle
        });
        cx.run_until_parked();
        shell.read_with(cx, |shell, _| {
            assert_eq!(shell.state.stage(), SessionStage::Presenting);
            assert_eq!(shell.presenter.as_ref().unwrap().handle, replacement);
            assert_eq!(shell.import_token, 0);
        });
        let mut visual = gpui::VisualTestContext::from_window(replacement, cx);
        assert!(!visual.simulate_close());
        cx.run_until_parked();
        assert!(cx.windows().is_empty());
        shell.read_with(cx, |shell, _| {
            assert_eq!(shell.state.stage(), SessionStage::Ready);
            assert!(shell.audience.is_none());
            assert_eq!(shell.import_token, 1);
        });
    }

    include!("shell/preferences_tests.rs");
    include!("shell/presenter_time_tests.rs");
    include!("shell/outline_tests.rs");
    include!("shell/presenter_layout_tests.rs");
    include!("shell/timed_playback_tests.rs");

    fn headless_shell(
        cx: &mut TestAppContext,
    ) -> (Entity<Shell>, std::sync::mpsc::Sender<TopologyEvent>) {
        let settings = SettingsSession::from_outcome(
            ferrostage_settings::from_toml("schema_version = 1").unwrap(),
        );
        let (sender, topology) = std::sync::mpsc::channel();
        let shell = cx.new(|cx| {
            Shell::new(
                ShellInputs {
                    settings,
                    // No test path is read or written; the test only constructs views.
                    store: SettingsStore::at(PathBuf::from("unused-headless-settings.toml")),
                    renderer: Err(RenderError::FeatureDisabled),
                    topology,
                    watcher: Err(ferrostage_platform::PlatformError::UnsupportedPlatform),
                    initial_monitors: Ok(Vec::new()),
                    initial_pdf: None,
                },
                cx,
            )
        });
        cx.update(|cx| install_close_handler(&shell, cx));
        shell.update(cx, |shell, cx| {
            let id = MonitorId("headless-test-display".into());
            let display = u64::from(cx.displays().first().unwrap().id());
            // Identity mapping has separate tests. Exercise the actual native
            // coordinator open path with GPUI's no-OS-window TestPlatform.
            shell.mapped = Ok([(id.clone(), display)].into());
        });
        (shell, sender)
    }

    fn test_window(shell: &Shell, audience: bool, cx: &mut Context<Shell>) -> WindowRecord {
        shell
            .open(
                MonitorId("headless-test-display".into()),
                false,
                audience,
                cx,
            )
            .unwrap()
    }

    #[gpui::test]
    fn page_browser_real_input_caret_selection_paste_and_binding_conflicts(
        cx: &mut TestAppContext,
    ) {
        let (shell, handle) = input_presenter(cx);
        shell.update(cx, |shell, cx| {
            shell
                .state
                .apply_at(
                    Command::LoadDocument {
                        generation: ferrostage_core::DocumentGeneration(1),
                        page_count: 100,
                    },
                    Instant::now(),
                )
                .unwrap();
            shell
                .settings
                .settings
                .keybindings
                .next
                .extend(["6".into(), "backspace".into()]);
            actions::replace_bindings(&shell.settings.settings.keybindings, shell.locale, cx);
            shell.open_browser();
            cx.notify();
        });
        redraw(handle, cx);
        shell.read_with(cx, |shell, _| {
            assert_eq!(shell.browser().unwrap().edit.selection(), 0..1)
        });
        cx.simulate_keystrokes(handle, "6 9 left delete right 8");
        shell.read_with(cx, |shell, _| {
            assert_eq!(shell.browser().unwrap().edit.text, "68");
            assert_eq!(shell.state.current_page(), PhysicalPage(0));
        });
        cx.simulate_keystrokes(handle, "home shift-end 6 9 space b backspace 9");
        shell.read_with(cx, |shell, _| {
            assert_eq!(shell.browser().unwrap().edit.text, "69");
            assert_eq!(shell.browser().unwrap().target, Some(PhysicalPage(68)));
            assert_eq!(shell.state.current_page(), PhysicalPage(0));
            assert_eq!(shell.state.audience_snapshot().blank_mode, None);
            assert_eq!(shell.workbench.grid_start, 64);
        });
        cx.write_to_clipboard(gpui::ClipboardItem::new_string(" 42\n".into()));
        cx.simulate_keystrokes(handle, "ctrl-a ctrl-v");
        cx.write_to_clipboard(gpui::ClipboardItem::new_string(
            "unrelated restored clipboard".into(),
        ));
        shell.read_with(cx, |shell, _| {
            assert_eq!(shell.browser().unwrap().edit.text, "42")
        });
        cx.simulate_keystrokes(handle, "enter");
        shell.read_with(cx, |shell, _| {
            assert_eq!(shell.state.current_page(), PhysicalPage(41));
            assert!(shell.contextual.is_none());
            assert!(!shell.workbench.grid);
        });
    }

    #[gpui::test]
    fn page_browser_configured_rows_fit_normal_window(cx: &mut TestAppContext) {
        let (shell, handle) = workbench_fixture(cx);
        cx.test_window(handle)
            .simulate_resize(size(px(1100.), px(760.)));
        shell.update(cx, |shell, cx| {
            let session = shell.workbench.document.as_mut().unwrap();
            session.metadata.pages = (0..100)
                .map(|index| {
                    let mut page = session.metadata.pages[0].clone();
                    page.page = PhysicalPage(index);
                    page.label = None;
                    page
                })
                .collect();
            session.structure.pages = (0..100)
                .map(|index| {
                    let mut page = session.structure.pages[0].clone();
                    page.page = PhysicalPage(index);
                    page
                })
                .collect();
            session.document =
                ferrostage_document::LoadedDocument::load(&session.metadata, &session.structure)
                    .unwrap();
            shell
                .state
                .apply_at(
                    Command::LoadMappedDocument(
                        session
                            .document
                            .descriptor(ferrostage_core::DocumentGeneration(3))
                            .unwrap(),
                    ),
                    Instant::now(),
                )
                .unwrap();
            shell.open_browser();
            cx.notify();
        });
        redraw(handle, cx);
        let mut visual = gpui::VisualTestContext::from_window(handle, cx);
        let images = visual.debug_bounds("page-browser-images").unwrap();
        let grid = visual.debug_bounds("thumbnail-grid").unwrap();
        // The added numbering row keeps readable images; the grid alone scrolls.
        // Keyboard traversal must reveal the final tile instead of shrinking its image.
        for _ in 0..40 {
            if handle
                .downcast::<Presenter>()
                .unwrap()
                .update(cx, |view, window, _| view.browser_focused_tile(window))
                .unwrap()
                == Some(15)
            {
                break;
            }
            cx.simulate_keystrokes(handle, "tab");
            cx.run_until_parked();
            cx.test_window(handle).simulate_scheduled_frame();
            redraw(handle, cx);
        }
        cx.run_until_parked();
        cx.test_window(handle).simulate_scheduled_frame();
        redraw(handle, cx);
        assert_eq!(
            handle
                .downcast::<Presenter>()
                .unwrap()
                .update(cx, |view, window, _| view.browser_focused_tile(window))
                .unwrap(),
            Some(15)
        );
        let mut visual = gpui::VisualTestContext::from_window(handle, cx);
        let final_tile = visual.debug_bounds("thumbnail-15").unwrap();
        assert!(
            final_tile.bottom() <= images.bottom(),
            "last tile {final_tile:?}, viewport {images:?}"
        );
        let image = visual.debug_bounds("thumbnail-image-15").unwrap();
        let panel = visual.debug_bounds("page-browser").unwrap();
        let trigger = visual.debug_bounds("workbench-progress").unwrap();
        assert!(image.size.height >= px(30.));
        assert!(final_tile.right() <= images.right());
        assert!(panel.top() >= px(12.));
        assert!(
            panel.bottom() <= trigger.top() - px(8.),
            "panel {panel:?}, trigger {trigger:?}"
        );
        println!(
            "normal browser: panel={panel:?}, images={images:?}, grid={grid:?}, final image={image:?}"
        );
    }

    #[gpui::test]
    fn page_browser_raster_stays_inside_its_image_slot(cx: &mut TestAppContext) {
        let (shell, handle) = workbench_fixture(cx);
        cx.test_window(handle)
            .simulate_resize(size(px(1100.), px(760.)));
        shell.update(cx, |shell, cx| {
            shell.open_browser();
            shell.workbench.frames.push((
                crate::workbench::Surface::Thumbnail(0),
                crate::workbench::ReadyFrame {
                    request: crate::workbench::request_for(
                        shell.state.generation(),
                        PhysicalPage(0),
                        shell.state.slide_crop(),
                    ),
                    frame: ferrostage_core::BgraFrame::new(4, 3, 16, vec![255; 48]).unwrap(),
                },
            ));
            cx.notify();
        });
        redraw(handle, cx);
        let mut visual = gpui::VisualTestContext::from_window(handle, cx);
        let slot = visual.debug_bounds("thumbnail-image-0").unwrap();
        let raster = visual.debug_bounds("surface-Thumbnail(0)").unwrap();
        let caption = visual.debug_bounds("thumbnail-caption-0").unwrap();
        assert!(
            raster.size.height <= slot.size.height,
            "{raster:?} exceeds {slot:?}"
        );
        assert!(
            raster.bottom() <= caption.top(),
            "raster overlaps its caption"
        );
        handle
            .downcast::<Presenter>()
            .unwrap()
            .update(cx, |presenter, _, cx| {
                presenter.seed_browser_failure(0, cx);
            })
            .unwrap();
        redraw(handle, cx);
        let mut visual = gpui::VisualTestContext::from_window(handle, cx);
        let error = visual.debug_bounds("thumbnail-error-0").unwrap();
        assert_eq!(error.size, slot.size);
        assert!(visual.debug_bounds("surface-Thumbnail(0)").is_none());
    }

    #[gpui::test]
    fn page_browser_jump_refocuses_editor_after_modal_and_reopen(cx: &mut TestAppContext) {
        let fixture = SaveFixture::new();
        fixture.unblock();
        let (shell, handle) = workbench_fixture(cx);
        shell.update(cx, |shell, cx| {
            shell.store = fixture.store();
            shell.modal = Some(Modal::Settings);
            cx.notify();
        });
        redraw(handle, cx);
        for repeated in [false, true] {
            if repeated {
                click_control(handle, "page-columns-2", cx);
                redraw(handle, cx);
            }
            shell.update(cx, |shell, cx| shell.dispatch(Intent::Jump, cx));
            cx.run_until_parked();
            shell.update(cx, |shell, cx| shell.poll_focus(cx));
            redraw(handle, cx);
            cx.simulate_keystrokes(handle, "2");
            shell.read_with(cx, |shell, _| {
                assert_eq!(shell.browser().unwrap().edit.text, "2");
                assert_eq!(shell.state.current_page(), PhysicalPage(0));
            });
        }
        cx.simulate_keystrokes(handle, "enter");
        shell.read_with(cx, |shell, _| {
            assert!(shell.contextual.is_none());
            assert_eq!(shell.state.current_page(), PhysicalPage(2));
        });
    }

    #[gpui::test]
    fn page_browser_outside_press_dismisses_without_drawing_or_click_through(
        cx: &mut TestAppContext,
    ) {
        let (shell, handle) = workbench_fixture(cx);
        shell.update(cx, |shell, cx| {
            shell.workbench.tool = crate::workbench::Tool::Pen;
            shell.open_browser();
            cx.notify();
        });
        redraw(handle, cx);
        let mut visual = gpui::VisualTestContext::from_window(handle, cx);
        assert!(visual.debug_bounds("page-browser").is_some());
        assert!(visual.debug_bounds("current-pane").is_some());
        let surface = visual.debug_bounds("current-pane").unwrap();
        let outside = gpui::point(surface.left() + px(5.), surface.top() + px(5.));
        visual.simulate_click(outside, gpui::Modifiers::default());
        shell.read_with(cx, |shell, _| {
            assert!(shell.contextual.is_none());
            assert_eq!(shell.state.current_page(), PhysicalPage(0));
            assert!(shell.state.audience_snapshot().strokes.is_empty());
        });
        redraw(handle, cx);
        cx.simulate_keystrokes(handle, "pagedown");
        shell.read_with(cx, |shell, _| {
            assert_eq!(shell.state.current_page(), PhysicalPage(2))
        });
    }

    #[gpui::test]
    fn page_browser_density_legacy_atomic_readback_failure_retry_and_readonly(
        cx: &mut TestAppContext,
    ) {
        let fixture = SaveFixture::new();
        let (shell, handle) = workbench_fixture(cx);
        shell.update(cx, |shell, cx| {
            shell.store = fixture.store();
            shell.settings = SettingsSession::from_outcome(ferrostage_settings::from_toml("schema_version = 1\n[presenter]\ngrid_columns = 4\ngrid_visible_pages = 13\nvendor_leaf = 'keep'\n").unwrap());
            shell.open_browser();
            assert_eq!(shell.settings.settings.presenter.grid_visible_pages, 13);
            assert!(!shell.settings.dirty);
            cx.notify();
        });
        redraw(handle, cx);
        let mut visual = gpui::VisualTestContext::from_window(handle, cx);
        let columns = visual.debug_bounds("page-columns-2").unwrap();
        visual.simulate_click(columns.center(), gpui::Modifiers::default());
        shell.read_with(cx, |shell, _| {
            assert_eq!(
                (
                    shell.settings.settings.presenter.grid_columns,
                    shell.settings.settings.presenter.grid_visible_pages
                ),
                (2, 8)
            );
            assert!(shell.settings.dirty && shell.settings.save_error.is_some());
            assert!(shell.browser().is_some());
            assert_eq!(shell.state.current_page(), PhysicalPage(0));
        });
        fixture.unblock();
        shell.update(cx, |shell, cx| {
            shell.dispatch(Intent::RetrySave, cx);
            assert!(!shell.settings.dirty);
            shell.dispatch(Intent::GridDensity { column: false, value: 3 }, cx);
            let readback = shell.store.load().unwrap();
            assert_eq!((readback.settings.presenter.grid_columns, readback.settings.presenter.grid_visible_pages), (2, 6));
            assert_eq!(readback.raw_table["presenter"]["vendor_leaf"].as_str(), Some("keep"));
            let bytes = std::fs::read(shell.store.path()).unwrap();
            shell.settings = SettingsSession::from_outcome(ferrostage_settings::from_toml("schema_version = 999\n[presenter]\ngrid_columns = 1\ngrid_visible_pages = 16\n").unwrap());
            shell.close_contextual();
            shell.open_browser();
            shell.dispatch(Intent::GridDensity { column: false, value: 2 }, cx);
            assert_eq!((shell.settings.settings.presenter.grid_columns, shell.settings.settings.presenter.grid_visible_pages), (1, 16));
            assert_eq!(std::fs::read(shell.store.path()).unwrap(), bytes);
        });
    }

    #[gpui::test]
    fn page_browser_tile_keyboard_activation_uses_enter_and_space_once(cx: &mut TestAppContext) {
        let (shell, handle) = workbench_fixture(cx);
        for key in ["enter", "space"] {
            shell.update(cx, |shell, cx| {
                shell.apply(Command::JumpTo(PhysicalPage(0)), cx);
                shell.open_browser();
                cx.notify();
            });
            redraw(handle, cx);
            // Field -> Go -> four column choices -> four row choices -> tile 0 -> tile 1.
            for _ in 0..24 {
                if handle
                    .downcast::<Presenter>()
                    .unwrap()
                    .update(cx, |presenter, window, _| {
                        presenter.browser_focused_tile(window)
                    })
                    .unwrap()
                    == Some(1)
                {
                    break;
                }
                cx.simulate_keystrokes(handle, "tab");
                redraw(handle, cx);
            }
            assert_eq!(
                handle
                    .downcast::<Presenter>()
                    .unwrap()
                    .update(cx, |presenter, window, _| presenter
                        .browser_focused_tile(window))
                    .unwrap(),
                Some(1)
            );
            cx.simulate_keystrokes(handle, key);
            shell.read_with(cx, |shell, _| {
                assert_eq!(shell.state.current_page(), PhysicalPage(2), "{key}");
                assert!(shell.contextual.is_none());
            });
            redraw(handle, cx);
        }
    }

    #[gpui::test]
    fn page_browser_rejects_private_notes_and_stale_session_targets(cx: &mut TestAppContext) {
        let (shell, handle) = workbench_fixture(cx);
        shell.update(cx, |shell, cx| {
            shell.workbench.page_number_mode = crate::workbench::PageNumberMode::Pdf;
            set_jump(shell, "2");
            assert_eq!(shell.browser().unwrap().error, Some("pb-notes-page"));
            shell.dispatch(
                Intent::BrowserSubmit {
                    id: shell.browser().map_or(0, |browser| browser.id),
                },
                cx,
            );
            assert_eq!(shell.state.current_page(), PhysicalPage(0));
            let id = shell.browser().unwrap().id;
            shell.close_contextual();
            shell.open_browser();
            shell.dispatch(
                Intent::BrowserJump {
                    id,
                    page: PhysicalPage(2),
                },
                cx,
            );
            shell.dispatch(
                Intent::BrowserEdit {
                    id,
                    edit: crate::page_browser::NumberEdit::new("3".into()),
                },
                cx,
            );
            assert_eq!(shell.browser().unwrap().edit.text, "1");
            assert_eq!(shell.state.current_page(), PhysicalPage(0));
            let current = shell.browser().unwrap().id;
            shell.loading = true;
            shell.dispatch(
                Intent::BrowserJump {
                    id: current,
                    page: PhysicalPage(2),
                },
                cx,
            );
            assert!(shell.browser().is_some());
            assert_eq!(shell.state.current_page(), PhysicalPage(0));
            shell.loading = false;
            cx.notify();
        });
        redraw(handle, cx);
        cx.simulate_keystrokes(handle, "escape");
        assert!(shell.read_with(cx, |shell, _| shell.contextual.is_none()));
    }

    fn set_jump(shell: &mut Shell, text: &str) {
        shell.open_browser();
        let id = shell.browser().expect("open page browser").id;
        shell.edit_browser(id, crate::page_browser::NumberEdit::new(text.into()));
    }

    fn load_ready(shell: &mut Shell) {
        shell
            .state
            .apply_at(
                Command::LoadDocument {
                    generation: ferrostage_core::DocumentGeneration(1),
                    page_count: 3,
                },
                Instant::now(),
            )
            .unwrap();
    }

    fn audience_input_fixture(
        cx: &mut TestAppContext,
    ) -> (Entity<Shell>, AnyWindowHandle, AnyWindowHandle) {
        let (shell, presenter) = input_presenter(cx);
        let audience = shell.update(cx, |shell, cx| {
            load_ready(shell);
            let started = Instant::now() - Duration::from_secs(10);
            shell
                .state
                .apply_at(
                    Command::StartPresentation {
                        audience_available: true,
                    },
                    started,
                )
                .unwrap();
            shell
                .state
                .apply_at(Command::PauseTimer, started + Duration::from_secs(4))
                .unwrap();
            shell.audience = Some(test_window(shell, true, cx));
            shell.publish(false, cx);
            shell.audience.as_ref().unwrap().handle
        });
        redraw(audience, cx);
        (shell, presenter, audience)
    }

    fn click_audience(handle: AnyWindowHandle, cx: &mut TestAppContext) {
        redraw(handle, cx);
        let mut visual = gpui::VisualTestContext::from_window(handle, cx);
        // The client corner is also a valid click target when the PDF is letterboxed.
        let bounds = visual.debug_bounds("audience-playback-surface").unwrap();
        assert!(bounds.contains(&point(px(2.), px(2.))), "{bounds:?}");
        visual.simulate_click(point(px(2.), px(2.)), gpui::Modifiers::default());
        cx.run_until_parked();
    }

    #[gpui::test]
    fn audience_click_requires_completed_left_input_and_uses_next_once(cx: &mut TestAppContext) {
        let (shell, presenter, audience) = audience_input_fixture(cx);
        shell.update(cx, |shell, cx| {
            shell.snapshot.update(cx, |snapshot, cx| {
                snapshot.visual = Some(ferrostage_core::AudienceVisual {
                    page: ferrostage_core::BgraFrame::new(2, 1, 8, vec![255; 8]).unwrap(),
                    media: vec![],
                    texture_budget_bytes: 32,
                });
                cx.notify();
            });
        });
        redraw(audience, cx);
        let position = point(px(2.), px(2.));
        let mut visual = gpui::VisualTestContext::from_window(audience, cx);
        visual.simulate_mouse_down(
            position,
            gpui::MouseButton::Left,
            gpui::Modifiers::default(),
        );
        cx.run_until_parked();
        assert_eq!(
            shell.read_with(cx, |s, _| s.state.current_page()),
            PhysicalPage(0)
        );
        let mut visual = gpui::VisualTestContext::from_window(audience, cx);
        visual.simulate_mouse_up(
            point(px(-10.), px(-10.)),
            gpui::MouseButton::Left,
            gpui::Modifiers::default(),
        );
        for button in [gpui::MouseButton::Right, gpui::MouseButton::Middle] {
            visual.simulate_mouse_down(position, button, gpui::Modifiers::default());
            visual.simulate_mouse_up(position, button, gpui::Modifiers::default());
        }
        cx.run_until_parked();
        assert_eq!(
            shell.read_with(cx, |s, _| s.state.current_page()),
            PhysicalPage(0)
        );
        redraw(presenter, cx);
        let mut visual = gpui::VisualTestContext::from_window(presenter, cx);
        visual.simulate_click(point(px(2.), px(2.)), gpui::Modifiers::default());
        cx.run_until_parked();
        assert_eq!(
            shell.read_with(cx, |s, _| s.state.current_page()),
            PhysicalPage(0)
        );
        for (tool, expected) in [
            (crate::workbench::Tool::Pen, 1),
            (crate::workbench::Tool::Laser, 2),
        ] {
            shell.update(cx, |s, _| s.workbench.tool = tool);
            click_audience(audience, cx);
            shell.read_with(cx, |s, _| {
                assert_eq!(s.state.current_page(), PhysicalPage(expected));
                assert_eq!(s.state.stage(), SessionStage::Presenting);
                assert_eq!(s.state.elapsed_at(Instant::now()), Duration::from_secs(4));
                assert_eq!(s.workbench.tool, tool);
            });
        }
        click_audience(audience, cx);
        assert_eq!(
            shell.read_with(cx, |s, _| s.state.current_page()),
            PhysicalPage(2)
        );
        shell.update(cx, |s, cx| {
            // Navigation retains the previous complete pixels until matching
            // raster completion, just like keyboard/button Next.
            assert_eq!(s.snapshot.read(cx).current_page, PhysicalPage(0));
            let target = ferrostage_pdf::PixelSize {
                width: 2,
                height: 1,
            };
            s.workbench.audience_target = Some(target);
            s.workbench.frames.push((
                crate::workbench::Surface::Audience,
                crate::workbench::ReadyFrame {
                    request: ferrostage_render::RenderRequest {
                        generation: s.state.generation(),
                        page: s.state.current_page(),
                        crop: s.state.slide_crop(),
                        target_px: target,
                        priority: ferrostage_render::RenderPriority::Audience,
                        epoch: 0,
                    },
                    frame: ferrostage_core::BgraFrame::new(2, 1, 8, vec![255; 8]).unwrap(),
                },
            ));
            s.publish(false, cx);
            assert_eq!(s.snapshot.read(cx).current_page, PhysicalPage(2));
        });
    }

    #[gpui::test]
    fn audience_click_does_not_bind_the_presenter_slide_preview(cx: &mut TestAppContext) {
        let (shell, presenter) = workbench_fixture(cx);
        let audience = shell.update(cx, |shell, cx| {
            shell
                .state
                .apply_at(
                    Command::StartPresentation {
                        audience_available: true,
                    },
                    Instant::now(),
                )
                .unwrap();
            shell.workbench.frames.push((
                crate::workbench::Surface::Current,
                crate::workbench::ReadyFrame {
                    request: crate::workbench::request_for(
                        shell.state.generation(),
                        shell.state.current_page(),
                        shell.state.slide_crop(),
                    ),
                    frame: ferrostage_core::BgraFrame::new(4, 3, 16, vec![255; 48]).unwrap(),
                },
            ));
            shell.audience = Some(test_window(shell, true, cx));
            shell.publish(false, cx);
            cx.notify();
            shell.audience.as_ref().unwrap().handle
        });
        redraw(presenter, cx);
        let mut visual = gpui::VisualTestContext::from_window(presenter, cx);
        let preview = visual.debug_bounds("surface-Current").unwrap();
        visual.simulate_click(preview.center(), gpui::Modifiers::default());
        cx.run_until_parked();
        assert_eq!(
            shell.read_with(cx, |s, _| s.state.current_page()),
            PhysicalPage(0)
        );
        click_audience(audience, cx);
        assert_eq!(
            shell.read_with(cx, |s, _| s.state.current_page()),
            PhysicalPage(2)
        );
    }

    #[gpui::test]
    fn audience_click_honors_live_policy_modal_blanking_and_busy_guards(cx: &mut TestAppContext) {
        let fixture = SaveFixture::new();
        fixture.unblock();
        let (shell, _, audience) = audience_input_fixture(cx);
        shell.update(cx, |s, _| s.store = fixture.store());
        for guard in 0..9 {
            shell.update(cx, |s, cx| match guard {
                0 => s.settings.settings.display.audience_click_to_advance = false,
                1 => s.modal = Some(Modal::Settings),
                2 => s.open_browser(),
                3 => s.loading = true,
                4 => s
                    .snapshot
                    .update(cx, |p, _| p.stage = AudienceStage::Curtain),
                5 => s
                    .snapshot
                    .update(cx, |p, _| p.blank_mode = Some(BlankMode::Black)),
                6 | 7 => {
                    s.state
                        .apply_at(
                            Command::ToggleBlank(if guard == 6 {
                                BlankMode::Black
                            } else {
                                BlankMode::White
                            }),
                            Instant::now(),
                        )
                        .unwrap();
                }
                8 => {
                    s.pending = Some(Transaction {
                        kind: TransactionKind::RecoverPresenter,
                        presenter: None,
                        audience: None,
                        deadline: Instant::now() + CONFIRM_TIMEOUT,
                    })
                }
                _ => unreachable!(),
            });
            click_audience(audience, cx);
            shell.update(cx, |s, cx| {
                assert_eq!(s.state.current_page(), PhysicalPage(0), "guard {guard}");
                s.settings.settings.display.audience_click_to_advance = true;
                s.modal = None;
                s.close_contextual();
                s.loading = false;
                s.pending = None;
                if let Some(blank) = s.state.audience_snapshot().blank_mode {
                    s.state
                        .apply_at(Command::ToggleBlank(blank), Instant::now())
                        .unwrap();
                }
                s.publish(false, cx);
            });
        }
        // An asynchronous settings edit remains busy even after its modal closes.
        shell.update(cx, |s, cx| {
            s.renderer = Ok(ferrostage_render::RenderHandle::with_backend(
                || Box::new(SettingsBackend),
                ferrostage_render::CacheBudgets::default(),
            )
            .unwrap());
            s.dispatch(Intent::EditSetting("cache.l2_cpu_mib", "17".into()), cx);
            assert!(s.preference_change.is_some());
        });
        click_audience(audience, cx);
        assert_eq!(
            shell.read_with(cx, |s, _| s.state.current_page()),
            PhysicalPage(0)
        );
        // Settlement is controlled by polling, not completion of the worker itself.
        settle_preferences(&shell, cx);
        click_audience(audience, cx);
        assert_eq!(
            shell.read_with(cx, |s, _| s.state.current_page()),
            PhysicalPage(1)
        );
    }

    #[gpui::test]
    fn audience_click_rejects_pending_retired_and_nonpresenting_windows(cx: &mut TestAppContext) {
        let (shell, _, audience) = audience_input_fixture(cx);
        let pending = shell.update(cx, |s, cx| {
            let candidate = test_window(s, true, cx);
            let handle = candidate.handle;
            s.pending = Some(Transaction {
                kind: TransactionKind::Start,
                presenter: None,
                audience: Some(candidate),
                deadline: Instant::now() + CONFIRM_TIMEOUT,
            });
            handle
        });
        click_audience(pending, cx);
        assert_eq!(
            shell.read_with(cx, |s, _| s.state.current_page()),
            PhysicalPage(0)
        );
        shell.update(cx, |s, _| {
            let replacement = s.pending.take().unwrap().audience.unwrap();
            s.audience = Some(replacement); // Keep retired native window open to deliver its late input.
        });
        click_audience(audience, cx);
        assert_eq!(
            shell.read_with(cx, |s, _| s.state.current_page()),
            PhysicalPage(0)
        );
        click_audience(pending, cx);
        assert_eq!(
            shell.read_with(cx, |s, _| s.state.current_page()),
            PhysicalPage(1)
        );
        for command in [Command::StopPresentation, Command::CloseDocument] {
            shell.update(cx, |s, _| {
                s.state.apply_at(command, Instant::now()).unwrap();
            });
            let before = shell.read_with(cx, |s, _| s.state.clone());
            click_audience(pending, cx);
            assert_eq!(shell.read_with(cx, |s, _| s.state.clone()), before);
        }
    }

    #[gpui::test]
    fn audience_click_preserves_overlay_steps_and_excludes_notes(cx: &mut TestAppContext) {
        let (shell, _, audience) = audience_input_fixture(cx);
        shell.update(cx, |s, cx| {
            s.state
                .apply_at(Command::StopPresentation, Instant::now())
                .unwrap();
            let map = ferrostage_core::SlideMap::new(
                5,
                vec![
                    vec![PhysicalPage(0), PhysicalPage(2)],
                    vec![PhysicalPage(4)],
                ],
            )
            .unwrap();
            let document = ferrostage_core::DocumentDescriptor::mapped(
                ferrostage_core::DocumentGeneration(2),
                map,
                ferrostage_core::NormalizedRect::FULL,
            )
            .unwrap();
            s.state
                .apply_at(Command::LoadMappedDocument(document), Instant::now())
                .unwrap();
            s.state
                .apply_at(
                    Command::StartPresentation {
                        audience_available: true,
                    },
                    Instant::now(),
                )
                .unwrap();
            s.publish(false, cx);
        });
        for physical in [2, 4, 4] {
            click_audience(audience, cx);
            shell.read_with(cx, |s, cx| {
                assert_eq!(s.state.current_page(), PhysicalPage(physical));
                assert_eq!(s.snapshot.read(cx).current_page, PhysicalPage(physical));
            });
        }
    }

    pub(super) fn input_presenter(cx: &mut TestAppContext) -> (Entity<Shell>, AnyWindowHandle) {
        let (shell, _) = headless_shell(cx);
        cx.update(|cx| {
            let model = shell.read(cx);
            let keys = model.settings.settings.keybindings.clone();
            let locale = model.locale;
            actions::install(shell.downgrade(), &keys, locale, cx);
        });
        let handle = shell.update(cx, |shell, cx| {
            shell.error = None;
            shell.presenter = Some(test_window(shell, false, cx));
            cx.notify();
            shell.presenter.as_ref().unwrap().handle
        });
        (shell, handle)
    }

    #[gpui::test]
    fn repeated_shell_ticks_never_start_or_advance_a_presentation(cx: &mut TestAppContext) {
        let (shell, handle) = input_presenter(cx);
        shell.update(cx, |shell, cx| {
            load_ready(shell);
            for _ in 0..8 {
                shell.tick(cx);
                assert_eq!(shell.state.stage(), SessionStage::Ready);
                assert_eq!(shell.state.current_page(), PhysicalPage(0));
                assert_eq!(shell.state.elapsed_at(Instant::now()), Duration::ZERO);
                assert!(shell.audience.is_none());
                assert_eq!(shell.snapshot.read(cx).stage, AudienceStage::Curtain);
            }
        });
        for overtime in [false, true] {
            shell.update(cx, |shell, cx| {
                let target = u64::from(shell.settings.settings.timer.target_minutes) * 60;
                let elapsed = if overtime { target + 1 } else { target - 1 };
                let started = Instant::now() - Duration::from_secs(elapsed);
                shell
                    .state
                    .apply_at(
                        Command::StartPresentation {
                            audience_available: true,
                        },
                        started,
                    )
                    .unwrap();
                for _ in 0..8 {
                    shell.tick(cx);
                    assert_eq!(shell.state.stage(), SessionStage::Presenting);
                    assert_eq!(shell.state.current_page(), PhysicalPage(0));
                    assert!(shell.state.elapsed_at(Instant::now()) >= Duration::from_secs(elapsed));
                }
            });
            redraw(handle, cx);
            cx.simulate_keystrokes(handle, "pagedown");
            assert_eq!(
                shell.read_with(cx, |shell, _| shell.state.current_page()),
                PhysicalPage(1)
            );
            cx.simulate_keystrokes(handle, "pageup");
            shell.update(cx, |shell, _| {
                assert_eq!(shell.state.current_page(), PhysicalPage(0));
                shell
                    .state
                    .apply_at(Command::StopPresentation, Instant::now())
                    .unwrap();
            });
        }
    }

    fn assert_open_waits_for_owner(
        shell: &Entity<Shell>,
        handle: AnyWindowHandle,
        cx: &mut TestAppContext,
    ) {
        shell.read_with(cx, |shell, _| {
            assert!(shell.error.is_none(), "input error: {:?}", shell.error);
            assert_eq!(
                shell.focus_intent.as_ref().map(|intent| intent.presenter),
                Some(handle.window_id())
            );
        });
        assert!(
            !cx.did_prompt_for_paths(),
            "input must not synchronously open a chooser"
        );
        cx.run_until_parked();
        shell.update(cx, |shell, cx| shell.poll_focus(cx));
        assert!(
            cx.did_prompt_for_paths(),
            "confirmed current Presenter must own the chooser"
        );
        assert_eq!(cx.update(|cx| cx.active_window()), Some(handle));
        cx.simulate_path_prompt_response(|_| None);
        cx.run_until_parked();
    }

    #[gpui::test]
    fn input_route_ctrl_o_releases_window_before_activation(cx: &mut TestAppContext) {
        let (shell, handle) = input_presenter(cx);
        cx.simulate_keystrokes(handle, "ctrl-o");
        assert_open_waits_for_owner(&shell, handle, cx);
    }

    pub(super) fn redraw(handle: AnyWindowHandle, cx: &mut TestAppContext) {
        handle
            .update(cx, |_, window, cx| window.draw(cx).clear(cx))
            .unwrap();
    }

    fn click_control(handle: AnyWindowHandle, name: &'static str, cx: &TestAppContext) {
        let mut visual = gpui::VisualTestContext::from_window(handle, cx);
        let bounds = visual.debug_bounds(name).unwrap();
        visual.simulate_click(bounds.center(), gpui::Modifiers::default());
    }

    fn assert_tab_opens_pdf(
        shell: &Entity<Shell>,
        handle: AnyWindowHandle,
        cx: &mut TestAppContext,
    ) {
        // Home precedes Open once a document is loaded.
        cx.simulate_keystrokes(handle, "tab tab enter");
        assert_open_waits_for_owner(shell, handle, cx);
    }

    #[gpui::test]
    fn v6_removed_modal_button_restores_keyboard_traversal(cx: &mut TestAppContext) {
        let (shell, handle) = input_presenter(cx);
        shell.update(cx, |shell, cx| {
            load_ready(shell);
            cx.notify();
        });
        redraw(handle, cx);
        click_control(handle, "settings", cx);
        shell.update(cx, |shell, cx| shell.poll_focus(cx));
        redraw(handle, cx);
        cx.simulate_keystrokes(handle, "tab enter"); // Settings Close is first in the header.
        assert!(shell.read_with(cx, |shell, _| shell.modal.is_none()));
        redraw(handle, cx);
        let before = handle
            .update(cx, |_, window, cx| window.focused(cx))
            .unwrap();
        cx.simulate_keystrokes(handle, "tab");
        redraw(handle, cx);
        handle
            .update(cx, |_, window, cx| {
                assert_ne!(window.focused(cx), before, "Tab must traverse after Close");
                assert!(window.is_action_available(&actions::ActivateButton, cx));
            })
            .unwrap();
    }

    #[gpui::test]
    fn v6_removed_jump_commit_restores_keyboard_traversal(cx: &mut TestAppContext) {
        let (shell, handle) = input_presenter(cx);
        shell.update(cx, |shell, cx| {
            load_ready(shell);
            cx.notify();
        });
        redraw(handle, cx);
        cx.simulate_keystrokes(handle, "g");
        shell.update(cx, |shell, cx| shell.poll_focus(cx));
        redraw(handle, cx);
        cx.simulate_keystrokes(handle, "2 tab enter");
        assert_eq!(
            shell.read_with(cx, |shell, _| shell.state.current_page()),
            PhysicalPage(1)
        );
        redraw(handle, cx);
        assert_tab_opens_pdf(&shell, handle, cx);
    }

    #[gpui::test]
    fn v6_removed_stop_restores_keyboard_traversal(cx: &mut TestAppContext) {
        let (shell, handle) = input_presenter(cx);
        shell.update(cx, |shell, cx| {
            load_ready(shell);
            shell
                .state
                .apply_at(
                    Command::StartPresentation {
                        audience_available: true,
                    },
                    Instant::now(),
                )
                .unwrap();
            cx.notify();
        });
        redraw(handle, cx);
        cx.simulate_keystrokes(handle, "tab shift-tab enter");
        assert_eq!(
            shell.read_with(cx, |shell, _| shell.state.stage()),
            SessionStage::Ready
        );
        redraw(handle, cx);
        assert_tab_opens_pdf(&shell, handle, cx);
    }

    struct SaveFixture(PathBuf);

    impl SaveFixture {
        fn new() -> Self {
            static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
            let path = std::env::temp_dir().join(format!(
                "ferrostage-ui-save-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            ));
            std::fs::create_dir(&path).unwrap();
            std::fs::write(path.join("blocked"), b"block parent directory").unwrap();
            Self(path)
        }

        fn store(&self) -> SettingsStore {
            SettingsStore::at(self.0.join("blocked/settings.toml"))
        }

        fn unblock(&self) {
            std::fs::remove_file(self.0.join("blocked")).unwrap();
        }
    }

    impl Drop for SaveFixture {
        fn drop(&mut self) {
            std::fs::remove_dir_all(&self.0).unwrap();
        }
    }

    #[gpui::test]
    fn v6_save_failure_has_one_owner_and_keeps_unrelated_error(cx: &mut TestAppContext) {
        let fixture = SaveFixture::new();
        let (shell, _) = input_presenter(cx);
        for unrelated in [None, Some("unrelated import error".to_string())] {
            shell.update(cx, |shell, cx| {
                shell.store = fixture.store();
                shell.error = unrelated.clone();
                shell.dispatch(Intent::RetrySave, cx);
                assert!(shell.settings.dirty);
                assert!(shell.settings.save_error.is_some());
                assert_eq!(
                    shell.error, unrelated,
                    "save feedback must not duplicate/overwrite the general error"
                );
            });
        }
        fixture.unblock();
        shell.update(cx, |shell, cx| {
            shell.dispatch(Intent::RetrySave, cx);
            assert!(!shell.settings.dirty);
            assert!(shell.settings.save_error.is_none());
            assert_eq!(shell.error.as_deref(), Some("unrelated import error"));
        });
    }

    #[gpui::test]
    fn v6_import_and_swap_keep_runtime_results_when_saving_fails(cx: &mut TestAppContext) {
        let fixture = SaveFixture::new();
        let (shell, _) = input_presenter(cx);
        let metadata = DocumentMetadata {
            pages: vec![ferrostage_pdf::PageMetadata {
                page: PhysicalPage(0),
                width_points: 640.,
                height_points: 480.,
                rotation_degrees: 0,
                label: None,
            }],
        };
        let structure = ferrostage_pdf::DocumentStructure {
            outline: Default::default(),
            pages: vec![ferrostage_pdf::structure::PageStructure {
                presentation: Default::default(),
                page: PhysicalPage(0),
                geometry: ferrostage_pdf::structure::PageGeometry {
                    bounds: [0., 0., 640., 480.],
                    rotation_degrees: 0,
                },
                media: vec![],
            }],
            diagnostics: vec![],
        };
        let session = crate::workbench::DocumentSession::build(metadata, structure).unwrap();
        let opened_after = chrono::Utc::now().timestamp();
        shell.update(cx, |shell, cx| {
            shell.store = fixture.store();
            shell.error = Some("previous import failure".into());
            shell.finish_import(
                PathBuf::from("owned-metadata.pdf"),
                ferrostage_core::DocumentGeneration(2),
                session,
                cx,
            );
            assert_eq!(shell.state.stage(), SessionStage::Ready);
            assert_eq!(
                shell.path.as_deref(),
                Some(std::path::Path::new("owned-metadata.pdf"))
            );
            assert_eq!(
                shell.settings.settings.general.recent_files[0],
                "owned-metadata.pdf"
            );
            assert!(shell.error.is_none());
            assert!(shell.settings.dirty && shell.settings.save_error.is_some());
            let opened_at = shell.settings.settings.general.recent_opened_at["owned-metadata.pdf"];
            assert!((opened_after..=chrono::Utc::now().timestamp()).contains(&opened_at));

            let mut candidate = shell.settings.settings.display.clone();
            candidate
                .monitors
                .push(ferrostage_settings::MonitorBinding {
                    id: "new-role-authority".into(),
                    friendly_name: "Headless role".into(),
                    role: ferrostage_core::MonitorRole::Presenter,
                    source_occurrence: None,
                });
            // Exercise the actual confirmed replacement commit/save branch.
            // Physical role mapping/migration remains separate native evidence.
            let replacement = test_window(shell, false, cx);
            let replacement_handle = replacement.handle;
            shell.pending = Some(Transaction {
                kind: TransactionKind::Assign(candidate),
                presenter: Some(replacement),
                audience: None,
                deadline: Instant::now() + CONFIRM_TIMEOUT,
            });
            shell.error = Some("unrelated operation error".into());
            shell.poll_transaction(cx);
            assert!(shell.pending.is_none());
            assert_eq!(shell.presenter.as_ref().unwrap().handle, replacement_handle);
            assert_eq!(
                shell.settings.settings.display.monitors[0].id,
                "new-role-authority"
            );
            assert_eq!(shell.error.as_deref(), Some("unrelated operation error"));
            assert!(shell.settings.dirty && shell.settings.save_error.is_some());
        });
        fixture.unblock();
        shell.update(cx, |shell, cx| {
            shell.dispatch(Intent::RetrySave, cx);
            let saved = shell.store.load().unwrap();
            assert_eq!(saved.settings.display.monitors[0].id, "new-role-authority");
            assert_eq!(saved.settings.general.recent_files[0], "owned-metadata.pdf");
            assert_eq!(
                saved.settings.general.recent_opened_at,
                shell.settings.settings.general.recent_opened_at
            );
            assert!(shell.settings.save_error.is_none());
            assert!(!shell.settings.dirty);
            assert_eq!(shell.error.as_deref(), Some("unrelated operation error"));
        });
    }

    #[gpui::test]
    fn import_history_records_reopens_only_after_acceptance_and_respects_readonly(
        cx: &mut TestAppContext,
    ) {
        let fixture = SaveFixture::new();
        fixture.unblock();
        let (shell, _) = workbench_fixture(cx);
        shell.update(cx, |shell, cx| {
            shell.store = fixture.store();
            shell.settings.settings.hot_reload.enabled = false;
            shell.settings.settings.general.recent_files =
                vec!["other.pdf".into(), "reopen.pdf".into()];
            shell.settings.settings.general.recent_opened_at =
                BTreeMap::from([("other.pdf".into(), 123), ("reopen.pdf".into(), 456)]);
            let session = shell.workbench.document.clone().unwrap();
            let opened_after = chrono::Utc::now().timestamp();
            shell.finish_import(
                "reopen.pdf".into(),
                ferrostage_core::DocumentGeneration(3),
                session.clone(),
                cx,
            );
            assert_eq!(
                shell.settings.settings.general.recent_files,
                ["reopen.pdf", "other.pdf"]
            );
            assert_eq!(
                shell.settings.settings.general.recent_opened_at["other.pdf"],
                123
            );
            assert!(
                (opened_after..=chrono::Utc::now().timestamp())
                    .contains(&shell.settings.settings.general.recent_opened_at["reopen.pdf"])
            );
            let saved_bytes = std::fs::read(shell.store.path()).unwrap();
            let history = shell.settings.settings.general.clone();
            shell
                .state
                .apply_at(
                    Command::StartPresentation {
                        audience_available: true,
                    },
                    Instant::now(),
                )
                .unwrap();
            shell.finish_import(
                "rejected.pdf".into(),
                ferrostage_core::DocumentGeneration(4),
                session.clone(),
                cx,
            );
            assert_eq!(shell.settings.settings.general, history);
            assert_eq!(shell.path, Some("reopen.pdf".into()));
            assert_eq!(
                shell.state.generation(),
                ferrostage_core::DocumentGeneration(3)
            );
            assert_eq!(
                shell.error.as_deref(),
                Some(shell.locale.text("import-error"))
            );
            shell
                .state
                .apply_at(Command::StopPresentation, Instant::now())
                .unwrap();
            shell.settings.write_policy = WritePolicy::ReadOnly {
                reason: "future schema".into(),
            };
            shell.finish_import(
                "readonly.pdf".into(),
                ferrostage_core::DocumentGeneration(5),
                session,
                cx,
            );
            assert_eq!(shell.path, Some("readonly.pdf".into()));
            assert_eq!(shell.settings.settings.general, history);
            assert!(!shell.settings.dirty);
            assert_eq!(std::fs::read(shell.store.path()).unwrap(), saved_bytes);
        });
    }

    #[gpui::test]
    fn v6_retry_success_clears_save_feedback_and_restores_keyboard(cx: &mut TestAppContext) {
        let fixture = SaveFixture::new();
        let (shell, handle) = input_presenter(cx);
        shell.update(cx, |shell, cx| {
            load_ready(shell);
            shell.store = fixture.store();
            shell
                .settings
                .settings
                .display
                .monitors
                .push(ferrostage_settings::MonitorBinding {
                    id: "saved-presenter".into(),
                    role: ferrostage_core::MonitorRole::Presenter,
                    friendly_name: "Saved presenter".into(),
                    source_occurrence: None,
                });
            shell.settings.note_edit();
            shell.dispatch(Intent::RetrySave, cx);
        });
        redraw(handle, cx);
        fixture.unblock();
        cx.simulate_keystrokes(handle, "tab shift-tab enter"); // Retry is the last enabled control.
        shell.read_with(cx, |shell, _| {
            assert!(!shell.settings.dirty);
            assert!(shell.settings.save_error.is_none());
            assert!(
                shell.error.is_none(),
                "successful Retry left stale save error: {:?}",
                shell.error
            );
            let saved = shell.store.load().unwrap();
            assert_eq!(saved.settings.display.monitors[0].id, "saved-presenter");
            assert_eq!(
                saved.settings.display.monitors[0].role,
                ferrostage_core::MonitorRole::Presenter
            );
        });
        redraw(handle, cx);
        assert_tab_opens_pdf(&shell, handle, cx);
    }

    #[gpui::test]
    fn v6_notify_preserves_valid_control_and_audience_activation(cx: &mut TestAppContext) {
        let (shell, handle) = input_presenter(cx);
        cx.simulate_keystrokes(handle, "tab");
        let focus = handle
            .update(cx, |_, window, cx| window.focused(cx))
            .unwrap();
        let audience = shell.update(cx, |shell, cx| {
            let audience = test_window(shell, true, cx);
            let handle = audience.handle;
            shell.audience = Some(audience);
            cx.notify();
            handle
        });
        audience
            .update(cx, |_, window, _| window.activate_window())
            .unwrap();
        cx.run_until_parked();
        redraw(handle, cx);
        assert_eq!(
            handle
                .update(cx, |_, window, cx| window.focused(cx))
                .unwrap(),
            focus
        );
        assert_eq!(cx.update(|cx| cx.active_window()), Some(audience));
    }

    #[gpui::test]
    fn v6_focus_chosen_after_draw_wins_over_deferred_recovery(cx: &mut TestAppContext) {
        let (shell, handle) = input_presenter(cx);
        cx.simulate_keystrokes(handle, "tab");
        shell.update(cx, |_, cx| cx.notify());
        let newer = handle
            .update(cx, |_, window, cx| {
                window.draw(cx).clear(cx);
                window.focus_next(cx);
                window.focused(cx)
            })
            .unwrap();
        assert_eq!(
            handle
                .update(cx, |_, window, cx| window.focused(cx))
                .unwrap(),
            newer
        );
        shell.update(cx, |_, cx| cx.notify());
        handle
            .update(cx, |_, window, cx| {
                window.draw(cx).clear(cx);
                window.blur(cx);
            })
            .unwrap();
        assert!(
            handle
                .update(cx, |_, window, cx| window.focused(cx))
                .unwrap()
                .is_none()
        );
    }

    #[gpui::test]
    fn v5_tab_focused_open_consumes_enter_and_space_before_global_actions(cx: &mut TestAppContext) {
        let (shell, handle) = input_presenter(cx);
        shell.update(cx, |shell, cx| {
            load_ready(shell);
            cx.notify();
        });
        redraw(handle, cx);
        cx.simulate_keystrokes(handle, "tab tab"); // Home, then Open.
        redraw(handle, cx);
        for key in ["space", "enter"] {
            cx.simulate_keystrokes(handle, key);
            let token = shell.read_with(cx, |shell, _| shell.focus_token);
            handle
                .update(cx, |_, window, cx| {
                    use gpui::InputEvent;
                    window.dispatch_event(
                        gpui::KeyUpEvent {
                            keystroke: gpui::Keystroke::parse(key).unwrap(),
                        }
                        .to_platform_input(),
                        cx,
                    );
                })
                .unwrap();
            assert_eq!(
                shell.read_with(cx, |shell, _| shell.focus_token),
                token,
                "keyup must not activate twice"
            );
            shell.read_with(cx, |shell, _| {
                assert_eq!(
                    shell.state.current_page(),
                    PhysicalPage(0),
                    "{key} must not navigate from a focused button"
                )
            });
            assert_open_waits_for_owner(&shell, handle, cx);
        }
    }

    #[gpui::test]
    fn v5_nonbutton_navigation_and_disabled_controls_keep_their_scope(cx: &mut TestAppContext) {
        let (shell, handle) = input_presenter(cx);
        shell.update(cx, |shell, cx| {
            load_ready(shell);
            cx.notify();
        });
        redraw(handle, cx);
        cx.simulate_keystrokes(handle, "space");
        assert_eq!(
            shell.read_with(cx, |shell, _| shell.state.current_page()),
            PhysicalPage(1)
        );
        cx.simulate_keystrokes(handle, "tab shift-tab");
        redraw(handle, cx);
        // Start is disabled without a watcher. Reverse traversal must skip it
        // and reach Displays, the last enabled control in this Ready fixture.
        cx.simulate_keystrokes(handle, "space");
        shell.read_with(cx, |shell, _| {
            assert_eq!(shell.state.current_page(), PhysicalPage(1));
            assert!(shell.error.is_none());
            assert!(matches!(
                shell.focus_intent.as_ref().map(|pending| &pending.intent),
                Some(Intent::Displays)
            ));
        });
        let mut visual = gpui::VisualTestContext::from_window(handle, cx);
        let bounds = visual.debug_bounds("start").unwrap();
        visual.simulate_click(bounds.center(), gpui::Modifiers::default());
        assert!(shell.read_with(cx, |shell, _| shell.pending.is_none()));

        cx.simulate_keystrokes(handle, "tab tab"); // Home, then Open.
        redraw(handle, cx);
        shell.update(cx, |shell, cx| {
            shell.error = None;
            shell.cancel_focus(); // Finish the preceding Displays activation assertion.
            shell
                .state
                .apply_at(
                    Command::StartPresentation {
                        audience_available: true,
                    },
                    Instant::now(),
                )
                .unwrap();
            cx.notify();
        });
        redraw(handle, cx);
        click_control(handle, "more", cx);
        redraw(handle, cx);
        click_control(handle, "open", cx); // Disabled Open remains inert in More.
        cx.simulate_keystrokes(handle, "escape");
        shell.read_with(cx, |shell, _| {
            assert_eq!(shell.state.stage(), SessionStage::Presenting);
            assert_eq!(shell.state.current_page(), PhysicalPage(1));
            assert!(shell.focus_intent.is_none());
            assert!(shell.error.is_none());
        });
        assert!(!cx.did_prompt_for_paths());
    }

    #[gpui::test]
    fn v5_tab_focused_stop_consumes_enter_and_space(cx: &mut TestAppContext) {
        for key in ["space", "enter"] {
            let (shell, handle) = input_presenter(cx);
            shell.update(cx, |shell, cx| {
                load_ready(shell);
                shell
                    .state
                    .apply_at(
                        Command::StartPresentation {
                            audience_available: true,
                        },
                        Instant::now(),
                    )
                    .unwrap();
                cx.notify();
            });
            redraw(handle, cx);
            cx.simulate_keystrokes(handle, "tab shift-tab");
            redraw(handle, cx);
            cx.simulate_keystrokes(handle, key);
            shell.read_with(cx, |shell, _| {
                assert_eq!(
                    shell.state.stage(),
                    SessionStage::Ready,
                    "{key} must activate Stop"
                );
                assert_eq!(shell.state.current_page(), PhysicalPage(0));
            });
            handle
                .update(cx, |_, window, _| window.remove_window())
                .unwrap();
        }
    }

    #[gpui::test]
    fn v5_modal_close_button_has_priority_over_global_enter_and_space(cx: &mut TestAppContext) {
        let (shell, handle) = input_presenter(cx);
        shell.update(cx, |shell, cx| {
            load_ready(shell);
            cx.notify();
        });
        for key in ["space", "enter"] {
            shell.update(cx, |shell, cx| {
                shell.modal = Some(Modal::Settings);
                cx.notify();
            });
            redraw(handle, cx);
            cx.simulate_keystrokes(handle, "tab"); // Settings Close is first in the header.
            redraw(handle, cx);
            cx.simulate_keystrokes(handle, key);
            shell.read_with(cx, |shell, _| {
                assert!(
                    shell.modal.is_none(),
                    "{key} must activate the focused Close button"
                );
                assert_eq!(shell.state.stage(), SessionStage::Ready);
                assert_eq!(shell.state.current_page(), PhysicalPage(0));
            });
            redraw(handle, cx);
        }
    }

    #[gpui::test]
    fn v5_idle_navigation_and_blank_actions_do_not_report_core_errors(cx: &mut TestAppContext) {
        let (shell, handle) = input_presenter(cx);
        for key in ["space", "pagedown", "pageup", "b", "w", "escape"] {
            cx.simulate_keystrokes(handle, key);
            shell.read_with(cx, |shell, cx| {
                assert!(shell.error.is_none(), "{key}: {:?}", shell.error);
                assert_eq!(shell.state.stage(), SessionStage::Idle);
                assert_eq!(
                    shell.state.generation(),
                    ferrostage_core::DocumentGeneration(0)
                );
                assert_eq!(shell.snapshot.read(cx).stage, AudienceStage::Curtain);
                assert!(shell.audience.is_none());
            });
        }
        assert_eq!(cx.windows(), vec![handle]);
    }

    #[gpui::test]
    fn v5_jump_editing_precedes_global_backspace_and_numeric_bindings(cx: &mut TestAppContext) {
        let (shell, handle) = input_presenter(cx);
        shell.update(cx, |shell, cx| {
            load_ready(shell);
            cx.notify();
        });
        for numeric_binding in [false, true] {
            if numeric_binding {
                shell.update(cx, |shell, cx| {
                    shell.settings.settings.keybindings.next.push("2".into());
                    actions::install(
                        cx.weak_entity(),
                        &shell.settings.settings.keybindings,
                        shell.locale,
                        cx,
                    );
                });
            }
            cx.simulate_keystrokes(handle, "g");
            shell.update(cx, |shell, cx| shell.poll_focus(cx));
            redraw(handle, cx);
            cx.simulate_keystrokes(handle, "2 3 backspace");
            shell.read_with(cx, |shell, _| {
                assert_eq!(
                    shell.browser().map(|browser| browser.edit.text.as_str()),
                    Some("2"),
                    "numeric binding: {numeric_binding}"
                );
                assert_eq!(shell.state.current_page(), PhysicalPage(0));
            });
            cx.simulate_keystrokes(handle, "escape");
            redraw(handle, cx);
        }
        cx.simulate_keystrokes(handle, "g");
        shell.update(cx, |shell, cx| shell.poll_focus(cx));
        redraw(handle, cx);
        cx.simulate_keystrokes(handle, "2 enter");
        shell.read_with(cx, |shell, _| {
            assert_eq!(shell.state.current_page(), PhysicalPage(1));
            assert!(shell.modal.is_none());
        });
    }

    #[gpui::test]
    fn v5_jump_numeric_text_precedes_configured_navigation(cx: &mut TestAppContext) {
        let (shell, handle) = input_presenter(cx);
        shell.update(cx, |shell, cx| {
            load_ready(shell);
            shell.settings.settings.keybindings.next.push("2".into());
            actions::install(
                cx.weak_entity(),
                &shell.settings.settings.keybindings,
                shell.locale,
                cx,
            );
        });
        cx.simulate_keystrokes(handle, "g");
        shell.update(cx, |shell, cx| shell.poll_focus(cx));
        redraw(handle, cx);
        cx.simulate_keystrokes(handle, "2");
        assert_eq!(
            shell.read_with(cx, |shell, _| shell
                .browser()
                .map(|browser| browser.edit.text.clone())),
            Some("2".into())
        );
    }

    #[gpui::test]
    fn v5_presenting_open_refuses_chooser_after_presenter_activation(cx: &mut TestAppContext) {
        let (shell, presenter) = input_presenter(cx);
        let audience = shell.update(cx, |shell, cx| {
            load_ready(shell);
            shell
                .state
                .apply_at(
                    Command::StartPresentation {
                        audience_available: true,
                    },
                    Instant::now(),
                )
                .unwrap();
            shell.audience = Some(test_window(shell, true, cx));
            cx.notify();
            shell.audience.as_ref().unwrap().handle
        });
        audience
            .update(cx, |_, window, _| window.activate_window())
            .unwrap();
        cx.run_until_parked();
        cx.simulate_keystrokes(audience, "ctrl-o");
        assert!(!cx.did_prompt_for_paths());
        assert!(
            shell.read_with(cx, |shell, _| shell.error.is_none()),
            "warning waits for Presenter confirmation"
        );
        shell.update(cx, |shell, cx| shell.poll_focus(cx));
        assert!(
            !cx.did_prompt_for_paths(),
            "Presenting must never enter a file chooser"
        );
        assert_eq!(cx.update(|cx| cx.active_window()), Some(presenter));
        shell.read_with(cx, |shell, _| {
            assert_eq!(
                shell.error.as_deref(),
                Some(shell.locale.text("stop-before-import"))
            );
            assert_eq!(shell.state.stage(), SessionStage::Presenting);
            assert_eq!(shell.state.current_page(), PhysicalPage(0));
            assert_eq!(shell.import_token, 0);
        });
    }

    #[gpui::test]
    fn v5_open_rechecks_stage_when_activation_resolves(cx: &mut TestAppContext) {
        let (shell, presenter) = input_presenter(cx);
        shell.update(cx, |shell, cx| {
            load_ready(shell);
            cx.notify();
        });
        cx.simulate_keystrokes(presenter, "ctrl-o");
        shell.update(cx, |shell, cx| {
            shell
                .state
                .apply_at(
                    Command::StartPresentation {
                        audience_available: true,
                    },
                    Instant::now(),
                )
                .unwrap();
            shell.poll_focus(cx);
        });
        assert!(
            !cx.did_prompt_for_paths(),
            "a stage change before activation must reject the queued chooser"
        );
        assert!(shell.read_with(cx, |shell, _| shell.focus_intent.is_none()));
    }

    #[gpui::test]
    fn v5_stop_discards_jump_input_before_following_navigation(cx: &mut TestAppContext) {
        let (shell, handle) = input_presenter(cx);
        shell.update(cx, |shell, cx| {
            load_ready(shell);
            shell
                .state
                .apply_at(
                    Command::StartPresentation {
                        audience_available: true,
                    },
                    Instant::now(),
                )
                .unwrap();
            cx.notify();
        });
        cx.simulate_keystrokes(handle, "g");
        shell.update(cx, |shell, cx| shell.poll_focus(cx));
        redraw(handle, cx);
        cx.simulate_keystrokes(handle, "2");
        cx.dispatch_action(handle, actions::Stop);
        shell.read_with(cx, |shell, _| {
            assert!(shell.modal.is_none());
            assert_eq!(shell.state.stage(), SessionStage::Ready);
        });
        redraw(handle, cx);
        cx.simulate_keystrokes(handle, "pagedown");
        assert_eq!(
            shell.read_with(cx, |shell, _| shell.state.current_page()),
            PhysicalPage(1)
        );
    }

    #[gpui::test]
    fn v5_swap_failure_discards_abandoned_jump_input(cx: &mut TestAppContext) {
        let (shell, handle) = input_presenter(cx);
        shell.update(cx, |shell, cx| {
            load_ready(shell);
            let first = test_monitor();
            let mut second = first.clone();
            second.id = MonitorId("other-test-display".into());
            second.connection_name = "other-test".into();
            second.is_primary = false;
            shell.settings.settings.display.monitors = vec![
                ferrostage_settings::MonitorBinding {
                    id: first.id.0.clone(),
                    friendly_name: first.name.clone(),
                    role: ferrostage_core::MonitorRole::Presenter,
                    source_occurrence: None,
                },
                ferrostage_settings::MonitorBinding {
                    id: second.id.0.clone(),
                    friendly_name: second.name.clone(),
                    role: ferrostage_core::MonitorRole::Audience,
                    source_occurrence: None,
                },
            ];
            shell.monitors = vec![first, second];
            set_jump(shell, "2");
            cx.notify();
        });
        cx.dispatch_action(handle, actions::Swap);
        shell.read_with(cx, |shell, _| {
            assert!(
                shell.error.is_some(),
                "unmapped test display makes replacement fail"
            );
            assert!(shell.modal.is_none());
            assert!(shell.pending.is_none());
        });
        redraw(handle, cx);
        cx.simulate_keystrokes(handle, "pagedown");
        assert_eq!(
            shell.read_with(cx, |shell, _| shell.state.current_page()),
            PhysicalPage(1)
        );
    }

    #[gpui::test]
    fn v5_escape_from_jump_discards_input_without_stopping(cx: &mut TestAppContext) {
        let (shell, handle) = input_presenter(cx);
        shell.update(cx, |shell, cx| {
            load_ready(shell);
            shell
                .state
                .apply_at(
                    Command::StartPresentation {
                        audience_available: true,
                    },
                    Instant::now(),
                )
                .unwrap();
            set_jump(shell, "2");
            cx.notify();
        });
        redraw(handle, cx);
        cx.simulate_keystrokes(handle, "escape");
        shell.read_with(cx, |shell, _| {
            assert!(shell.modal.is_none());
            assert_eq!(shell.state.stage(), SessionStage::Presenting);
        });
    }

    fn test_monitor() -> MonitorInfo {
        MonitorInfo {
            id: MonitorId("headless-test-display".into()),
            name: "Headless display".into(),
            connection_name: "test-display".into(),
            native_size: (1920, 1080),
            position: (0, 0),
            scale_factor: 1.0,
            scale_source: ferrostage_platform::ScaleSource::GdiDeviceCapsRatio,
            rotation_degrees: 0,
            is_primary: true,
            identity_source: ferrostage_platform::IdentitySource::DeviceInterfacePath,
            identity_confidence: ferrostage_platform::IdentityConfidence::ConnectionScoped,
            built_in: false,
        }
    }

    fn topology_recovery_is_windowed(event: TopologyEvent, cx: &mut TestAppContext) {
        let (shell, sender) = headless_shell(cx);
        let original = shell.update(cx, |shell, cx| {
            load_ready(shell);
            shell
                .state
                .apply_at(
                    Command::StartPresentation {
                        audience_available: true,
                    },
                    Instant::now(),
                )
                .unwrap();
            let mut lost = test_monitor();
            lost.id = MonitorId("lost-audience".into());
            lost.connection_name = "lost-audience".into();
            shell.monitors = vec![test_monitor(), lost];
            shell.recover_windowed = false;
            shell.settings.settings.display.presenter_start_fullscreen = true;
            let mut presenter = test_window(shell, false, cx);
            presenter
                .handle
                .update(cx, |_, window, _| window.toggle_fullscreen())
                .unwrap();
            presenter.fullscreen = true;
            let handle = presenter.handle;
            shell.presenter = Some(presenter);
            shell.audience = Some(test_window(shell, true, cx));
            set_jump(shell, "2");
            handle
        });
        sender.send(event).unwrap();
        shell.update(cx, |shell, cx| shell.tick(cx));
        shell.read_with(cx, |shell, cx| {
            assert_eq!(shell.state.stage(), SessionStage::Ready);
            assert!(shell.audience.is_none());
            assert!(
                shell.pending.is_none(),
                "unmapped TestDisplay must not authorize placement"
            );
            assert_eq!(shell.snapshot.read(cx).stage, AudienceStage::Curtain);
        });
        sender
            .send(TopologyEvent::Changed(vec![test_monitor()]))
            .unwrap();
        shell.update(cx, |shell, cx| {
            shell.tick(cx);
            // TestPlatform has a random UUID, with no physical GDI connection.
            // Mapping is separately validated; supply its owned result here
            // only after the real Changed event and refusal of unmapped input.
            shell.mapped = Ok([(test_monitor().id, u64::from(cx.displays()[0].id()))].into());
            shell.recover_presenter(cx);
            assert!(
                shell.pending.is_some(),
                "fullscreen survivor must be replaced after unsafe topology"
            );
            assert!(
                !shell
                    .pending
                    .as_ref()
                    .unwrap()
                    .presenter
                    .as_ref()
                    .unwrap()
                    .fullscreen
            );
            shell.poll_transaction(cx);
        });
        shell.read_with(cx, |shell, _| {
            assert_eq!(shell.state.stage(), SessionStage::Ready);
            assert!(!shell.presenter.as_ref().unwrap().fullscreen);
            assert_ne!(shell.presenter.as_ref().unwrap().handle, original);
            assert!(shell.audience.is_none());
        });
        assert!(!cx.windows().contains(&original));
    }

    #[gpui::test]
    fn v5_topology_unavailable_recovers_windowed_without_restart(cx: &mut TestAppContext) {
        topology_recovery_is_windowed(
            TopologyEvent::Unavailable {
                error: ferrostage_platform::PlatformError::UnsupportedPlatform,
                last_known: vec![test_monitor()],
            },
            cx,
        );
    }

    #[gpui::test]
    fn v5_audience_loss_recovers_fullscreen_presenter_windowed(cx: &mut TestAppContext) {
        topology_recovery_is_windowed(TopologyEvent::Changed(vec![test_monitor()]), cx);
    }

    #[gpui::test]
    fn v5_recovery_replaces_stale_display_but_preserves_initial_fullscreen(
        cx: &mut TestAppContext,
    ) {
        let (shell, _) = headless_shell(cx);
        shell.update(cx, |shell, cx| {
            shell.monitors = vec![test_monitor()];
            shell.recover_windowed = false;
            shell.settings.settings.display.presenter_start_fullscreen = true;
            let mut presenter = test_window(shell, false, cx);
            presenter
                .handle
                .update(cx, |_, window, _| window.toggle_fullscreen())
                .unwrap();
            presenter.fullscreen = true;
            let original = presenter.handle;
            shell.presenter = Some(presenter);
            shell.recover_presenter(cx);
            assert!(
                shell.pending.is_none(),
                "ordinary initial topology retains configured fullscreen"
            );
            shell.presenter.as_mut().unwrap().display = u64::MAX;
            shell.recover_presenter(cx);
            assert!(
                shell.pending.is_some(),
                "same MonitorId with stale GPUI identity needs replacement"
            );
            shell.poll_transaction(cx);
            assert_ne!(shell.presenter.as_ref().unwrap().handle, original);
            assert!(!shell.presenter.as_ref().unwrap().fullscreen);
        });
    }

    #[gpui::test]
    fn input_route_open_button_releases_window_before_activation(cx: &mut TestAppContext) {
        let (shell, handle) = input_presenter(cx);
        let mut visual = gpui::VisualTestContext::from_window(handle, cx);
        let bounds = visual.debug_bounds("welcome-open").unwrap();
        visual.simulate_click(bounds.center(), gpui::Modifiers::default());
        assert_open_waits_for_owner(&shell, handle, cx);
    }

    #[gpui::test]
    fn input_route_audience_open_activates_presenter_and_escape_cancels(cx: &mut TestAppContext) {
        let (shell, presenter) = input_presenter(cx);
        let audience = shell.update(cx, |shell, cx| test_window(shell, true, cx).handle);
        audience
            .update(cx, |_, window, _| window.activate_window())
            .unwrap();
        cx.run_until_parked();
        assert_eq!(cx.update(|cx| cx.active_window()), Some(audience));
        cx.simulate_keystrokes(audience, "ctrl-o");
        assert_open_waits_for_owner(&shell, presenter, cx);
        cx.simulate_keystrokes(presenter, "ctrl-o escape");
        shell.update(cx, |shell, cx| shell.poll_focus(cx));
        assert!(
            !cx.did_prompt_for_paths(),
            "Escape must invalidate a queued popup"
        );
        assert!(shell.read_with(cx, |shell, _| shell.focus_intent.is_none()));
    }

    #[gpui::test]
    fn workbench_shortcuts_are_wired_without_stale_feature_warnings(cx: &mut TestAppContext) {
        let (shell, _) = headless_shell(cx);
        for language in ["en-US", "zh-CN"] {
            cx.update(|cx| {
                let keys = shell.read(cx).settings.settings.keybindings.clone();
                let locale = crate::locale::Locale::new(language);
                let warnings = actions::install(shell.downgrade(), &keys, locale, cx);
                assert!(warnings.iter().all(|warning| !warning.contains("toggle_")));
                assert!(warnings.is_empty());
            });
        }
    }

    #[gpui::test]
    fn presenter_creation_draws_while_controller_is_leased(cx: &mut TestAppContext) {
        let (shell, _) = headless_shell(cx);
        let handle = shell.update(cx, |shell, cx| test_window(shell, false, cx).handle);
        let view = handle.downcast::<Presenter>().unwrap().root(cx).unwrap();
        assert_eq!(view.read_with(cx, |view, _| view.rendered_document().2), 0);
        shell.update(cx, |shell, cx| {
            shell
                .state
                .apply_at(
                    Command::LoadDocument {
                        generation: ferrostage_core::DocumentGeneration(1),
                        page_count: 3,
                    },
                    Instant::now(),
                )
                .unwrap();
            shell.path = Some(PathBuf::from("loaded.pdf"));
            cx.notify();
        });
        assert_eq!(
            view.read_with(cx, |view, _| {
                let (name, page, pages) = view.rendered_document();
                (name.to_string(), page, pages)
            }),
            ("loaded.pdf".into(), 1, 3)
        );
        // Replacement windows draw under the same controller lease as startup.
        shell.update(cx, |shell, cx| {
            let replacement = shell
                .open(MonitorId("headless-test-display".into()), false, false, cx)
                .unwrap();
            Shell::close(replacement, cx);
        });
    }

    #[gpui::test]
    fn close_hook_topology_cancels_pending_under_controller_lease(cx: &mut TestAppContext) {
        let (shell, sender) = headless_shell(cx);
        shell.update(cx, |shell, cx| {
            load_ready(shell);
            shell.monitors = vec![test_monitor()];
            shell.pending = Some(Transaction {
                kind: TransactionKind::RecoverPresenter,
                presenter: Some(test_window(shell, false, cx)),
                audience: None,
                deadline: Instant::now() + CONFIRM_TIMEOUT,
            });
        });
        sender.send(TopologyEvent::Changed(Vec::new())).unwrap();
        shell.update(cx, |shell, cx| shell.tick(cx));
        shell.read_with(cx, |shell, cx| {
            assert!(shell.pending.is_none());
            assert!(shell.presenter.is_none());
            assert_eq!(shell.state.stage(), SessionStage::Ready);
            assert_eq!(shell.snapshot.read(cx).stage, AudienceStage::Curtain);
            assert_eq!(
                shell.import_token, 0,
                "retired close must not take current-presenter quit branch"
            );
        });
        assert!(cx.windows().is_empty());
    }

    #[gpui::test]
    fn close_hook_timeout_preserves_ready_and_current_presenter(cx: &mut TestAppContext) {
        let (shell, _) = headless_shell(cx);
        let current = shell.update(cx, |shell, cx| {
            load_ready(shell);
            shell.presenter = Some(test_window(shell, false, cx));
            let mut audience = test_window(shell, true, cx);
            audience.display = u64::MAX; // Actual window is deliberately on the wrong display.
            shell.pending = Some(Transaction {
                kind: TransactionKind::Start,
                presenter: None,
                audience: Some(audience),
                deadline: Instant::now() - Duration::from_secs(1),
            });
            shell.presenter.as_ref().unwrap().handle
        });
        shell.update(cx, |shell, cx| shell.poll_transaction(cx));
        shell.read_with(cx, |shell, cx| {
            assert!(shell.pending.is_none());
            assert!(shell.audience.is_none());
            assert_eq!(shell.state.stage(), SessionStage::Ready);
            assert_eq!(shell.state.elapsed_at(Instant::now()), Duration::ZERO);
            assert_eq!(shell.snapshot.read(cx).stage, AudienceStage::Curtain);
            assert_eq!(shell.presenter.as_ref().unwrap().handle, current);
            assert_eq!(
                shell.error.as_deref(),
                Some(shell.locale.text("placement-timeout"))
            );
        });
        assert_eq!(cx.windows(), vec![current]);
    }

    #[gpui::test]
    fn close_hook_replacement_keeps_live_session_and_user_close_stops(cx: &mut TestAppContext) {
        let (shell, _) = headless_shell(cx);
        let started = Instant::now() - Duration::from_secs(10);
        let (presenter, audience) = shell.update(cx, |shell, cx| {
            load_ready(shell);
            shell
                .state
                .apply_at(
                    Command::StartPresentation {
                        audience_available: true,
                    },
                    started,
                )
                .unwrap();
            shell.state.apply_at(Command::NextStep, started).unwrap();
            shell.presenter = Some(test_window(shell, false, cx));
            shell.audience = Some(test_window(shell, true, cx));
            shell.pending = Some(Transaction {
                kind: TransactionKind::RecoverPresenter,
                presenter: Some(test_window(shell, false, cx)),
                audience: None,
                deadline: Instant::now() + CONFIRM_TIMEOUT,
            });
            let new_presenter = shell
                .pending
                .as_ref()
                .unwrap()
                .presenter
                .as_ref()
                .unwrap()
                .handle;
            let audience = shell.audience.as_ref().unwrap().handle;
            shell.poll_transaction(cx); // Actually commits a replacement and removes the old window.
            (new_presenter, audience)
        });
        shell.read_with(cx, |shell, cx| {
            assert_eq!(shell.presenter.as_ref().unwrap().handle, presenter);
            assert_eq!(shell.state.stage(), SessionStage::Presenting);
            assert_eq!(shell.state.current_page(), PhysicalPage(1));
            assert_eq!(
                shell.state.elapsed_at(started + Duration::from_secs(20)),
                Duration::from_secs(20)
            );
            assert_eq!(shell.snapshot.read(cx).stage, AudienceStage::Live);
            assert_eq!(shell.import_token, 0);
        });
        shell.update(cx, |shell, _| {
            set_jump(shell, "3");
        });
        audience
            .update(cx, |_, window, _| window.remove_window())
            .unwrap();
        shell.read_with(cx, |shell, cx| {
            assert_eq!(shell.state.stage(), SessionStage::Ready);
            assert!(shell.audience.is_none());
            assert_eq!(shell.snapshot.read(cx).stage, AudienceStage::Curtain);
            assert_eq!(shell.presenter.as_ref().unwrap().handle, presenter);
        });
        presenter
            .update(cx, |_, window, _| window.remove_window())
            .unwrap();
        shell.read_with(cx, |shell, _| {
            assert!(shell.presenter.is_none());
            assert_eq!(
                shell.import_token, 1,
                "current user close takes the explicit quit branch"
            );
        });
    }
    #[gpui::test]
    fn workbench_split_release_outside_saves_and_future_schema_stays_readonly(
        cx: &mut TestAppContext,
    ) {
        let fixture = SaveFixture::new();
        fixture.unblock();
        let (shell, handle) = workbench_fixture(cx);
        shell.update(cx, |shell, _| shell.store = fixture.store());
        redraw(handle, cx);
        {
            let mut visual = gpui::VisualTestContext::from_window(handle, cx);
            let split = visual.debug_bounds("horizontal-split").unwrap();
            visual.simulate_mouse_down(
                split.center(),
                gpui::MouseButton::Left,
                gpui::Modifiers::default(),
            );
            visual.simulate_mouse_move(
                split.center() - gpui::point(px(100.), px(0.)),
                Some(gpui::MouseButton::Left),
                gpui::Modifiers::default(),
            );
            visual.simulate_mouse_up(
                gpui::point(px(1.), px(1.)),
                gpui::MouseButton::Left,
                gpui::Modifiers::default(),
            );
        }
        shell.read_with(cx, |shell, _| {
            assert!(!shell.settings.dirty);
            assert!(shell.settings.settings.presenter.next_slide_ratio > 0.4);
        });
        let bytes = std::fs::read(fixture.0.join("blocked/settings.toml")).unwrap();
        let parsed = std::str::from_utf8(&bytes)
            .unwrap()
            .parse::<toml::Table>()
            .unwrap();
        let saved = parsed["presenter"]["next_slide_ratio"].as_float().unwrap();
        shell.read_with(cx, |shell, _| {
            assert_eq!(saved, shell.settings.settings.presenter.next_slide_ratio)
        });
        shell.update(cx, |shell, cx| {
            shell.settings.write_policy = WritePolicy::ReadOnly {
                reason: "future schema 99".into(),
            };
            shell.dispatch(
                Intent::Split {
                    next: 0.2,
                    notes: 0.3,
                    save: true,
                },
                cx,
            );
            assert_eq!(shell.settings.settings.presenter.next_slide_ratio, saved);
        });
        assert_eq!(
            std::fs::read(fixture.0.join("blocked/settings.toml")).unwrap(),
            bytes
        );
    }

    #[gpui::test]
    fn workbench_narrow_stacks_and_grid_wheel_never_navigates(cx: &mut TestAppContext) {
        let (shell, handle) = workbench_fixture(cx);
        shell.update(cx, |shell, cx| {
            shell.open_browser();
            shell.settings.settings.presenter.grid_columns = 1;
            shell.settings.settings.presenter.grid_visible_pages = 2;
            cx.notify();
        });
        cx.test_window(handle)
            .simulate_resize(size(px(700.), px(1200.)));
        redraw(handle, cx);
        {
            let mut visual = gpui::VisualTestContext::from_window(handle, cx);
            let current = visual.debug_bounds("current-pane").unwrap();
            let next = visual.debug_bounds("next-notes-pane").unwrap();
            assert!(next.origin.y >= current.bottom());
            assert!((next.origin.x - current.origin.x).abs() < px(1.));
        }
        cx.test_window(handle)
            .simulate_resize(size(px(1280.), px(1200.)));
        redraw(handle, cx);
        let mut visual = gpui::VisualTestContext::from_window(handle, cx);
        visual.simulate_event(gpui::ScrollWheelEvent {
            position: gpui::point(px(300.), px(450.)),
            delta: gpui::ScrollDelta::Pixels(gpui::point(px(0.), px(-600.))),
            ..Default::default()
        });
        let tile = visual.debug_bounds("thumbnail-0").unwrap();
        visual.simulate_mouse_move(
            tile.origin + gpui::point(px(5.), px(5.)),
            None,
            gpui::Modifiers::default(),
        );
        visual.simulate_event(gpui::ScrollWheelEvent {
            position: tile.origin + gpui::point(px(5.), px(5.)),
            delta: gpui::ScrollDelta::Lines(gpui::point(0., -1.)),
            ..Default::default()
        });
        shell.read_with(&visual.cx, |shell, _| {
            assert_eq!(shell.workbench.grid_start, 1);
            assert_eq!(shell.state.current_page(), PhysicalPage(0));
        });
    }

    #[gpui::test]
    fn workbench_pointer_exit_ends_gesture_until_new_press(cx: &mut TestAppContext) {
        let (shell, handle) = workbench_fixture(cx);
        shell.update(cx, |shell, cx| {
            shell.workbench.tool = crate::workbench::Tool::Highlighter;
            shell.workbench.frames.push((
                crate::workbench::Surface::Current,
                crate::workbench::ReadyFrame {
                    request: crate::workbench::request_for(
                        shell.state.generation(),
                        PhysicalPage(0),
                        shell.state.slide_crop(),
                    ),
                    frame: ferrostage_core::BgraFrame::new(4, 3, 16, vec![255; 48]).unwrap(),
                },
            ));
            cx.notify();
        });
        redraw(handle, cx);
        let mut visual = gpui::VisualTestContext::from_window(handle, cx);
        let surface = visual.debug_bounds("surface-Current").unwrap();
        let center = surface.center();
        visual.simulate_mouse_move(center, None, gpui::Modifiers::default());
        visual.simulate_mouse_down(center, gpui::MouseButton::Left, gpui::Modifiers::default());
        visual.simulate_mouse_move(
            gpui::point(px(1.), px(1.)),
            Some(gpui::MouseButton::Left),
            gpui::Modifiers::default(),
        );
        visual.simulate_mouse_move(
            center,
            Some(gpui::MouseButton::Left),
            gpui::Modifiers::default(),
        );
        shell.read_with(&visual.cx, |shell, _| {
            let snapshot = shell.state.audience_snapshot();
            assert_eq!(snapshot.strokes.len(), 1);
            assert!(snapshot.strokes[0].is_highlighter);
            assert_eq!(snapshot.strokes[0].points.len(), 1);
        });
    }

    #[gpui::test]
    fn audience_budget_failure_is_localized_visible_and_recovers_without_losing_shell_errors(
        cx: &mut TestAppContext,
    ) {
        let (shell, presenter) = workbench_fixture(cx);
        let audience = shell.update(cx, |shell, cx| {
            let record = test_window(shell, true, cx);
            let handle = record.handle;
            shell.audience = Some(record);
            shell.snapshot.update(cx, |snapshot, cx| {
                snapshot.stage = AudienceStage::Live;
                snapshot.visual = Some(ferrostage_core::AudienceVisual {
                    page: ferrostage_core::BgraFrame::new(2, 2, 8, vec![255; 16]).unwrap(),
                    media: vec![],
                    texture_budget_bytes: 32,
                });
                cx.notify();
            });
            handle
        });
        redraw(audience, cx);
        shell.update(cx, |shell, cx| {
            shell.error = Some("independent document diagnostic".into());
            shell.snapshot.update(cx, |snapshot, cx| {
                snapshot.visual.as_mut().unwrap().texture_budget_bytes = 16;
                cx.notify();
            });
        });
        redraw(audience, cx);
        cx.run_until_parked();
        shell.read_with(cx, |shell, _| assert!(shell.graphics_error.is_some()));
        redraw(presenter, cx);
        let mut visual = gpui::VisualTestContext::from_window(presenter, cx);
        assert!(visual.debug_bounds("presenter-error").is_some());
        shell.read_with(cx, |shell, _| {
            let message = shell
                .locale
                .error("wb-upload-error", shell.graphics_error.as_ref().unwrap());
            assert!(!message.contains("{error}"));
            assert!(message.contains("GPU budget"));
        });
        shell.update(cx, |shell, cx| {
            shell.snapshot.update(cx, |snapshot, cx| {
                snapshot.visual.as_mut().unwrap().texture_budget_bytes = 32;
                cx.notify();
            })
        });
        redraw(audience, cx);
        cx.run_until_parked();
        shell.read_with(cx, |shell, _| {
            assert!(shell.graphics_error.is_none());
            assert_eq!(
                shell.error.as_deref(),
                Some("independent document diagnostic")
            );
        });
        shell.update(cx, |shell, cx| {
            shell.snapshot.update(cx, |snapshot, cx| {
                snapshot.stage = AudienceStage::Curtain;
                snapshot.visual = None;
                cx.notify();
            })
        });
        redraw(audience, cx);
        cx.run_until_parked();
        shell.read_with(cx, |shell, _| assert!(shell.graphics_error.is_none()));
        shell.update(cx, |shell, cx| {
            shell.graphics_error = Some("retired audience upload failure".into());
            let replacement = test_window(shell, true, cx);
            let replacement_handle = replacement.handle;
            shell.pending = Some(Transaction {
                kind: TransactionKind::Start,
                presenter: None,
                audience: Some(replacement),
                deadline: Instant::now() + CONFIRM_TIMEOUT,
            });
            shell.poll_transaction(cx);
            assert!(shell.pending.is_none());
            assert_eq!(shell.audience.as_ref().unwrap().handle, replacement_handle);
            assert!(shell.graphics_error.is_none());
            assert_eq!(
                shell.error.as_deref(),
                Some("independent document diagnostic")
            );
        });
    }

    fn workbench_fixture(cx: &mut TestAppContext) -> (Entity<Shell>, AnyWindowHandle) {
        let (shell, handle) = input_presenter(cx);
        shell.update(cx, |shell, cx| {
            let metadata = DocumentMetadata {
                pages: (0..4)
                    .map(|index| ferrostage_pdf::PageMetadata {
                        page: PhysicalPage(index),
                        width_points: 640.,
                        height_points: 480.,
                        rotation_degrees: 0,
                        label: Some(["1", "notes:1", "2", "3"][index].into()),
                    })
                    .collect(),
            };
            let structure = ferrostage_pdf::DocumentStructure {
                outline: Default::default(),
                pages: (0..4)
                    .map(|index| ferrostage_pdf::structure::PageStructure {
                        presentation: Default::default(),
                        page: PhysicalPage(index),
                        geometry: ferrostage_pdf::structure::PageGeometry {
                            bounds: [0., 0., 640., 480.],
                            rotation_degrees: 0,
                        },
                        media: vec![],
                    })
                    .collect(),
                diagnostics: vec![],
            };
            let session = crate::workbench::DocumentSession::build(metadata, structure).unwrap();
            shell
                .state
                .apply_at(
                    Command::LoadMappedDocument(
                        session
                            .document
                            .descriptor(ferrostage_core::DocumentGeneration(2))
                            .unwrap(),
                    ),
                    Instant::now(),
                )
                .unwrap();
            shell.workbench.document = Some(session);
            cx.notify();
        });
        redraw(handle, cx);
        (shell, handle)
    }

    fn layout_focus(handle: AnyWindowHandle, cx: &mut TestAppContext) -> (Option<usize>, bool) {
        handle
            .downcast::<Presenter>()
            .unwrap()
            .update(cx, |view, window, _| view.layout_focus_probe(window))
            .unwrap()
    }

    const LAYOUT_ROWS: [&str; 3] = ["layout-row-0", "layout-row-1", "layout-row-2"];
    const LAYOUT_CHECKS: [&str; 3] = ["layout-check-0", "layout-check-1", "layout-check-2"];

    #[gpui::test]
    fn layout_menu_applies_each_layout_and_retains_upward_workbench_geometry(
        cx: &mut TestAppContext,
    ) {
        use ferrostage_document::NotesLayout;
        let (shell, handle) = workbench_fixture(cx);
        cx.test_window(handle)
            .simulate_resize(size(px(1100.), px(760.)));
        redraw(handle, cx);
        let mut visual = gpui::VisualTestContext::from_window(handle, cx);
        let before = visual.debug_bounds("current-pane").unwrap();
        for (index, layout) in [None, Some(NotesLayout::None)].into_iter().enumerate() {
            click_control(handle, "document-options", cx);
            redraw(handle, cx);
            let mut visual = gpui::VisualTestContext::from_window(handle, cx);
            let panel = visual.debug_bounds("layout-navigation-menu").unwrap();
            let trigger = visual.debug_bounds("document-options").unwrap();
            assert_eq!(visual.debug_bounds("current-pane"), Some(before));
            assert_eq!(panel.size.width, px(320.));
            assert!(
                (f32::from(trigger.right().clamp(px(332.), px(1088.))) - f32::from(panel.right()))
                    .abs()
                    <= 1.,
                "panel={panel:?}, trigger={trigger:?}"
            );
            assert!(
                (f32::from(trigger.top() - panel.bottom()) - 8.).abs() <= 1.,
                "{panel:?}, {trigger:?}"
            );
            assert!(panel.top() >= px(12.));
            for row in LAYOUT_ROWS {
                let bounds = visual.debug_bounds(row).unwrap();
                assert!(bounds.size.height >= px(44.));
                assert!(bounds.bottom() <= panel.bottom());
            }
            click_control(handle, LAYOUT_ROWS[index], cx);
            shell.read_with(cx, |shell, cx| {
                assert!(shell.contextual.is_none());
                let doc = &shell.workbench.document.as_ref().unwrap().document;
                assert_eq!(doc.overrides.layout, layout);
                assert_eq!(
                    shell.state.slide_crop(),
                    ferrostage_document::slide_crop(layout.unwrap_or(NotesLayout::None))
                );
                assert_eq!(shell.snapshot.read(cx).slide_crop, shell.state.slide_crop());
                assert!(!doc.slide_map.steps().contains(&PhysicalPage(1)));
            });
            redraw(handle, cx);
            assert!(layout_focus(handle, cx).1, "menu trigger regains focus");
            cx.simulate_keystrokes(handle, "enter");
            redraw(handle, cx);
            assert_eq!(layout_focus(handle, cx).0, Some(index));
            let mut visual = gpui::VisualTestContext::from_window(handle, cx);
            for (candidate, selector) in LAYOUT_CHECKS.iter().enumerate().take(2) {
                assert_eq!(visual.debug_bounds(selector).is_some(), candidate == index);
            }
            cx.simulate_keystrokes(handle, "escape");
            redraw(handle, cx);
        }
    }

    #[gpui::test]
    fn layout_menu_keyboard_toggles_and_dismissal_do_not_leak_navigation(cx: &mut TestAppContext) {
        let (shell, handle) = workbench_fixture(cx);
        shell.update(cx, |shell, cx| {
            shell.workbench.tool = crate::workbench::Tool::Pen;
            shell
                .state
                .apply_at(
                    Command::StartPresentation {
                        audience_available: true,
                    },
                    Instant::now(),
                )
                .unwrap();
            cx.notify();
        });
        redraw(handle, cx);
        let row = 2;
        for expected in [false, true] {
            click_control(handle, "document-options", cx);
            redraw(handle, cx);
            cx.simulate_keystrokes(
                handle,
                "end left right pageup pagedown home down up tab shift-tab",
            );
            assert_eq!(layout_focus(handle, cx).0, Some(0));
            shell.read_with(cx, |shell, _| {
                assert_eq!(shell.state.current_page(), PhysicalPage(0))
            });
            cx.simulate_keystrokes(handle, if expected { "end enter" } else { "end space" });
            redraw(handle, cx);
            shell.read_with(cx, |shell, _| {
                assert!(shell.contextual.is_none());
                let doc = &shell.workbench.document.as_ref().unwrap().document;
                assert_eq!(doc.overrides.identify_independent_notes, expected);
                if doc.overrides.identify_independent_notes {
                    assert!(!doc.slide_map.steps().contains(&PhysicalPage(1)));
                }
                assert_eq!(shell.state.stage(), SessionStage::Presenting);
            });
            cx.simulate_keystrokes(handle, "enter");
            redraw(handle, cx);
            let mut visual = gpui::VisualTestContext::from_window(handle, cx);
            assert_eq!(visual.debug_bounds(LAYOUT_CHECKS[row]).is_some(), expected);
            cx.simulate_keystrokes(handle, "escape");
            redraw(handle, cx);
            shell.read_with(cx, |shell, _| {
                assert_eq!(shell.state.stage(), SessionStage::Presenting)
            });
        }
        click_control(handle, "document-options", cx);
        redraw(handle, cx);
        let mut visual = gpui::VisualTestContext::from_window(handle, cx);
        let current = visual.debug_bounds("current-pane").unwrap();
        visual.simulate_click(
            gpui::point(current.left() + px(5.), current.top() + px(5.)),
            gpui::Modifiers::default(),
        );
        redraw(handle, cx);
        assert!(layout_focus(handle, cx).1);
        shell.read_with(cx, |shell, _| {
            assert!(shell.contextual.is_none());
            assert_eq!(shell.state.current_page(), PhysicalPage(0));
            assert!(!shell.state.has_annotations());
            assert_eq!(shell.state.stage(), SessionStage::Presenting);
        });
    }

    #[gpui::test]
    fn layout_menu_noop_preserves_off_page_annotations_frames_and_failure_retains_checks(
        cx: &mut TestAppContext,
    ) {
        let (shell, handle) = workbench_fixture(cx);
        shell.update(cx, |shell, cx| {
            shell
                .state
                .apply_at(Command::AppendStrokePoint((0.2, 0.3)), Instant::now())
                .unwrap();
            shell
                .state
                .apply_at(Command::EndStroke, Instant::now())
                .unwrap();
            shell
                .state
                .apply_at(Command::NextStep, Instant::now())
                .unwrap();
            shell.workbench.frames.push((
                crate::workbench::Surface::Current,
                crate::workbench::ReadyFrame {
                    request: crate::workbench::request_for(
                        shell.state.generation(),
                        shell.state.current_page(),
                        shell.state.slide_crop(),
                    ),
                    frame: ferrostage_core::BgraFrame::new(4, 3, 16, vec![255; 48]).unwrap(),
                },
            ));
            cx.notify();
        });
        redraw(handle, cx);
        let original = shell.read_with(cx, |shell, _| shell.state.clone());
        click_control(handle, "document-options", cx);
        redraw(handle, cx);
        let mut visual = gpui::VisualTestContext::from_window(handle, cx);
        assert!(visual.debug_bounds("layout-annotation-notice").is_some());
        click_control(handle, "layout-row-1", cx);
        shell.read_with(cx, |shell, _| {
            assert!(shell.contextual.is_none());
            assert_eq!(shell.state, original);
            assert_eq!(shell.workbench.frames.len(), 1);
        });
        redraw(handle, cx);
        click_control(handle, "document-options", cx);
        let removed_page = shell.update(cx, |shell, _| {
            shell
                .workbench
                .document
                .as_mut()
                .unwrap()
                .structure
                .pages
                .pop()
                .unwrap()
        });
        redraw(handle, cx);
        click_control(handle, "layout-row-0", cx);
        redraw(handle, cx);
        shell.read_with(cx, |shell, _| {
            assert!(shell.layout_menu_open());
            assert_eq!(shell.state, original);
            assert_eq!(
                shell
                    .workbench
                    .document
                    .as_ref()
                    .unwrap()
                    .document
                    .overrides
                    .layout,
                Some(ferrostage_document::NotesLayout::None)
            );
            assert_eq!(shell.workbench.frames.len(), 1);
        });
        let mut visual = gpui::VisualTestContext::from_window(handle, cx);
        assert!(visual.debug_bounds("layout-menu-error").is_some());
        assert!(visual.debug_bounds("layout-check-1").is_some());
        assert!(visual.debug_bounds("layout-check-0").is_none());
        cx.simulate_keystrokes(handle, "escape enter");
        redraw(handle, cx);
        assert!(
            visual.debug_bounds("layout-menu-error").is_none(),
            "reopen discards stale local error"
        );
        shell.update(cx, |shell, _| {
            shell
                .workbench
                .document
                .as_mut()
                .unwrap()
                .structure
                .pages
                .push(removed_page)
        });
        click_control(handle, "layout-row-0", cx);
        redraw(handle, cx);
        shell.read_with(cx, |shell, cx| {
            assert!(shell.contextual.is_none());
            assert!(
                !shell.state.has_annotations(),
                "actual reconfiguration retains existing clear policy"
            );
            assert!(shell.workbench.frames.is_empty());
            assert_eq!(
                shell.state.slide_crop(),
                ferrostage_core::NormalizedRect::FULL
            );
            assert_eq!(shell.snapshot.read(cx).slide_crop, shell.state.slide_crop());
        });
    }

    #[gpui::test]
    fn layout_menu_and_browser_are_exclusive_and_lifecycle_actions_discard_menu(
        cx: &mut TestAppContext,
    ) {
        let (shell, handle) = workbench_fixture(cx);
        shell.update(cx, |shell, cx| {
            shell.open_browser();
            cx.notify();
        });
        redraw(handle, cx);
        shell.update(cx, |shell, cx| shell.dispatch(Intent::Document, cx));
        redraw(handle, cx);
        let mut visual = gpui::VisualTestContext::from_window(handle, cx);
        assert!(visual.debug_bounds("layout-navigation-menu").is_some());
        assert!(visual.debug_bounds("page-browser").is_none());
        shell.read_with(cx, |shell, _| assert!(!shell.workbench.grid));
        assert_eq!(layout_focus(handle, cx).0, Some(1));
        shell.update(cx, |shell, cx| {
            shell.open_browser();
            cx.notify();
        });
        redraw(handle, cx);
        assert!(visual.debug_bounds("layout-navigation-menu").is_none());
        assert!(visual.debug_bounds("page-browser").is_some());
        for action in [Intent::Stop, Intent::Swap, Intent::Home] {
            shell.update(cx, |shell, cx| shell.dispatch(Intent::Document, cx));
            redraw(handle, cx);
            shell.read_with(cx, |shell, _| assert!(shell.layout_menu_open()));
            shell.update(cx, |shell, cx| shell.dispatch(action.clone(), cx));
            redraw(handle, cx);
            shell.read_with(cx, |shell, _| assert!(shell.contextual.is_none()));
        }
    }

    #[gpui::test]
    fn layout_menu_rejects_busy_and_retired_selection_without_persistence(cx: &mut TestAppContext) {
        let (shell, handle) = workbench_fixture(cx);
        click_control(handle, "document-options", cx);
        redraw(handle, cx);
        let (retired, settings, state) = shell.read_with(cx, |shell, _| {
            (
                shell.contextual_serial,
                shell.settings.settings.clone(),
                shell.state.clone(),
            )
        });
        shell.update(cx, |shell, cx| {
            shell.close_contextual();
            shell.dispatch(Intent::Document, cx);
            let options = ferrostage_document::DocumentOverrides {
                layout: Some(ferrostage_document::NotesLayout::Left),
                ..Default::default()
            };
            shell.dispatch(
                Intent::Reconfigure {
                    id: retired,
                    options,
                },
                cx,
            );
            assert_eq!(shell.state, state);
            shell.loading = true;
            shell.dispatch(
                Intent::Reconfigure {
                    id: shell.contextual_serial,
                    options,
                },
                cx,
            );
            assert_eq!(shell.state, state);
            shell.loading = false;
            shell.dispatch(
                Intent::Reconfigure {
                    id: shell.contextual_serial,
                    options,
                },
                cx,
            );
            assert_eq!(shell.settings.settings, settings);
            assert!(!shell.settings.dirty);
        });
    }

    #[gpui::test]
    fn workbench_controls_jump_directly_and_keep_note_pages_private(cx: &mut TestAppContext) {
        let (shell, handle) = workbench_fixture(cx);
        let mut visual = gpui::VisualTestContext::from_window(handle, cx);
        let progress = visual.debug_bounds("workbench-progress").unwrap();
        visual.simulate_mouse_move(progress.center(), None, gpui::Modifiers::default());
        redraw(handle, cx);
        cx.run_until_parked();
        cx.executor().advance_clock(Duration::from_secs(1));
        cx.run_until_parked();
        redraw(handle, cx);
        assert!(visual.debug_bounds("presenter-text-tooltip").is_some());
        for _ in 0..16 {
            cx.simulate_keystrokes(handle, "tab");
            redraw(handle, cx);
            let mut visual = gpui::VisualTestContext::from_window(handle, cx);
            if visual.debug_bounds("progress-detail").is_some() {
                assert_eq!(
                    visual.debug_bounds("workbench-progress"),
                    Some(progress),
                    "focus detail cannot move the control during activation"
                );
                break;
            }
        }
        let mut visual = gpui::VisualTestContext::from_window(handle, cx);
        assert!(visual.debug_bounds("progress-detail").is_some());
        visual.simulate_mouse_move(progress.center(), None, gpui::Modifiers::default());
        redraw(handle, cx);
        cx.run_until_parked();
        cx.executor().advance_clock(Duration::from_secs(1));
        cx.run_until_parked();
        redraw(handle, cx);
        assert!(visual.debug_bounds("progress-detail").is_some());
        assert!(
            visual.debug_bounds("presenter-text-tooltip").is_none(),
            "focused progress already presents full detail; hovering must not duplicate it"
        );
        assert_eq!(visual.debug_bounds("workbench-progress"), Some(progress));
        visual.simulate_click(progress.center(), gpui::Modifiers::default());
        shell.update(cx, |shell, cx| shell.poll_focus(cx));
        shell.read_with(cx, |shell, _| assert!(shell.browser().is_some()));
        redraw(handle, cx);
        cx.simulate_keystrokes(handle, "2");
        shell.read_with(cx, |shell, _| {
            assert_eq!(shell.state.current_page(), PhysicalPage(0));
            assert!(shell.browser().is_some());
        });
        cx.simulate_keystrokes(handle, "enter");
        assert_eq!(
            shell.read_with(cx, |shell, _| shell.state.current_page()),
            PhysicalPage(2)
        );
        redraw(handle, cx);
        let mut visual = gpui::VisualTestContext::from_window(handle, cx);
        let pen = visual.debug_bounds("pen").unwrap();
        visual.simulate_click(pen.center(), gpui::Modifiers::default());
        assert_eq!(
            shell.read_with(cx, |shell, _| shell.workbench.tool),
            crate::workbench::Tool::Pen
        );
        shell.update(cx, |shell, cx| {
            shell.dispatch(Intent::Core(Command::JumpTo(PhysicalPage(2))), cx);
            assert_eq!(shell.state.current_page(), PhysicalPage(2));
            shell.workbench.page_number_mode = crate::workbench::PageNumberMode::Pdf;
            set_jump(shell, "2"); // physical 2 is a private notes page
            shell.dispatch(
                Intent::BrowserSubmit {
                    id: shell.browser().map_or(0, |browser| browser.id),
                },
                cx,
            );
            assert!(shell.browser().is_some());
            assert_eq!(shell.state.current_page(), PhysicalPage(2));
            shell.dispatch(Intent::Escape, cx);
            shell.dispatch(Intent::Core(Command::JumpTo(PhysicalPage(2))), cx);
            shell.dispatch(
                Intent::BrowserSubmit {
                    id: shell.browser().map_or(0, |browser| browser.id),
                },
                cx,
            );
            assert_eq!(shell.state.current_page(), PhysicalPage(2));
            shell.dispatch(Intent::Document, cx);
            let mut options = shell
                .workbench
                .document
                .as_ref()
                .unwrap()
                .document
                .overrides;
            options.layout = Some(ferrostage_document::NotesLayout::Right);
            let generation = shell.state.generation();
            shell.dispatch(
                Intent::Reconfigure {
                    id: shell.contextual_serial,
                    options,
                },
                cx,
            );
            assert_eq!(shell.state.generation(), generation);
            assert_eq!(
                shell.state.slide_crop(),
                ferrostage_core::NormalizedRect::LEFT_HALF
            );
            shell.loading = true;
            let token = shell.import_token;
            shell.dispatch(Intent::Import(PathBuf::from("must-not-supersede.pdf")), cx);
            assert_eq!(shell.import_token, token);
            assert!(!shell.import_allowed());
        });
    }

    fn same_row(origin: gpui::Pixels, other: gpui::Pixels) -> bool {
        (f32::from(origin) - f32::from(other)).abs() < 8.
    }

    #[gpui::test]
    fn thumbnail_whole_tile_activates_once_and_cancelled_press_is_inert(cx: &mut TestAppContext) {
        let (shell, handle) = workbench_fixture(cx);
        for target in ["thumbnail-image-1", "thumbnail-caption-1", "thumbnail-1"] {
            shell.update(cx, |shell, cx| {
                shell.apply(Command::JumpTo(PhysicalPage(0)), cx);
                shell.open_browser();
                cx.notify();
            });
            redraw(handle, cx);
            {
                let mut visual = gpui::VisualTestContext::from_window(handle, cx);
                let tile = visual.debug_bounds(target).unwrap();
                visual.simulate_mouse_move(tile.center(), None, gpui::Modifiers::default());
                visual.simulate_mouse_down(
                    tile.center(),
                    gpui::MouseButton::Left,
                    gpui::Modifiers::default(),
                );
            }
            shell.read_with(cx, |shell, _| {
                assert_eq!(shell.state.current_page(), PhysicalPage(0));
                assert!(shell.workbench.grid, "press must not commit before release");
            });
            {
                let mut visual = gpui::VisualTestContext::from_window(handle, cx);
                let outside = gpui::point(px(1.), px(1.));
                visual.simulate_mouse_move(
                    outside,
                    Some(gpui::MouseButton::Left),
                    gpui::Modifiers::default(),
                );
                visual.simulate_mouse_up(
                    outside,
                    gpui::MouseButton::Left,
                    gpui::Modifiers::default(),
                );
            }
            shell.read_with(cx, |shell, _| {
                assert_eq!(shell.state.current_page(), PhysicalPage(0));
                assert!(
                    shell.workbench.grid,
                    "cancelled click must keep the grid open"
                );
            });
            {
                let mut visual = gpui::VisualTestContext::from_window(handle, cx);
                let tile = visual.debug_bounds(target).unwrap();
                visual.simulate_click(tile.center(), gpui::Modifiers::default());
            }
            shell.read_with(cx, |shell, _| {
                assert_eq!(shell.state.current_page(), PhysicalPage(2));
                assert!(!shell.workbench.grid);
                assert!(shell.modal.is_none());
            });
            redraw(handle, cx);
            cx.simulate_keystrokes(handle, "pagedown");
            assert_eq!(
                shell.read_with(cx, |shell, _| shell.state.current_page()),
                PhysicalPage(3)
            );
        }
    }

    #[gpui::test]
    fn presenter_bar_shares_one_row_and_notes_fill_the_column(cx: &mut TestAppContext) {
        let (shell, handle) = workbench_fixture(cx);
        shell.update(cx, |shell, cx| {
            shell.settings.settings.presenter.notes_ratio = 0.4;
            shell.settings.settings.presenter.show_thumbnail_strip = true;
            shell.settings.settings.general.recent_files = vec!["kept.pdf".into()];
            cx.notify();
        });
        cx.test_window(handle)
            .simulate_resize(size(px(1100.), px(760.)));
        redraw(handle, cx);
        {
            let mut visual = gpui::VisualTestContext::from_window(handle, cx);
            let title = visual.debug_bounds("document-title").unwrap();
            let progress = visual.debug_bounds("workbench-progress").unwrap();
            let timer = visual.debug_bounds("presenter-timer").unwrap();
            let home = visual.debug_bounds("home").unwrap();
            assert!(
                same_row(title.origin.y, timer.origin.y) && same_row(title.origin.y, home.origin.y)
            );
            let start = visual.debug_bounds("start").unwrap();
            let previous = visual.debug_bounds("previous").unwrap();
            let next = visual.debug_bounds("next").unwrap();
            assert!(same_row(start.center().y, previous.center().y));
            assert!(same_row(start.center().y, next.center().y));
            assert!(same_row(next.center().y, progress.center().y));
            assert!(progress.origin.y > title.bottom());
            assert!(start.size.height >= px(crate::theme::SESSION_CONTROL));
            assert!(visual.debug_bounds("stop").is_none());
            let notes = visual.debug_bounds("notes-panel").unwrap();
            let right = visual.debug_bounds("next-notes-pane").unwrap();
            assert!(notes.size.height >= right.size.height * 0.50);
            let next = visual.debug_bounds("next-pane").unwrap();
            assert!(next.bottom() < notes.origin.y);
            assert!(notes.size.height > next.size.height);
            assert!(visual.debug_bounds("thumbnail-0").is_none());
            assert!(visual.debug_bounds("recent-0").is_none());
            let tools = visual.debug_bounds("previous").unwrap();
            assert!(notes.bottom() <= tools.origin.y);
            let current = visual.debug_bounds("current-pane").unwrap();
            let splits = visual.debug_bounds("workbench-splits").unwrap();
            assert!((current.size.height - splits.size.height).abs() < px(1.));
            assert!(current.size.height > px(200.));
        }
        shell.update(cx, |shell, cx| {
            shell.open_browser();
            cx.notify();
        });
        redraw(handle, cx);
        {
            let mut visual = gpui::VisualTestContext::from_window(handle, cx);
            let current = visual.debug_bounds("current-pane").unwrap();
            let workbench = visual.debug_bounds("workbench").unwrap();
            assert!(current.size.height + px(8.) >= workbench.size.height * 0.5);
        }
        cx.test_window(handle)
            .simulate_resize(size(px(700.), px(900.)));
        redraw(handle, cx);
        let mut visual = gpui::VisualTestContext::from_window(handle, cx);
        let title = visual.debug_bounds("document-title").unwrap();
        let home = visual.debug_bounds("home").unwrap();
        assert!(home.origin.y > title.bottom());
        let current = visual.debug_bounds("current-pane").unwrap();
        let next = visual.debug_bounds("next-notes-pane").unwrap();
        assert!(next.origin.y >= current.bottom());
        assert!(current.size.height > px(40.));
    }

    #[gpui::test]
    fn workbench_session_strip_survives_empty_steps_and_busy_start(cx: &mut TestAppContext) {
        let (shell, handle) = input_presenter(cx);
        shell.update(cx, |shell, cx| {
            load_ready(shell);
            shell.loading = true;
            cx.notify();
        });
        redraw(handle, cx);
        let mut visual = gpui::VisualTestContext::from_window(handle, cx);
        let start = visual.debug_bounds("start").unwrap();
        assert!(visual.debug_bounds("previous").is_none());
        assert!(visual.debug_bounds("start-reason").is_some());
        visual.simulate_click(start.center(), gpui::Modifiers::default());
        shell.read_with(cx, |shell, _| {
            assert_eq!(shell.state.stage(), SessionStage::Ready);
            assert!(shell.pending.is_none());
            assert_eq!(shell.state.elapsed_at(Instant::now()), Duration::ZERO);
        });
        shell.update(cx, |shell, cx| {
            shell.loading = false;
            shell
                .state
                .apply_at(
                    Command::StartPresentation {
                        audience_available: true,
                    },
                    Instant::now(),
                )
                .unwrap();
            cx.notify();
        });
        redraw(handle, cx);
        let mut visual = gpui::VisualTestContext::from_window(handle, cx);
        assert!(visual.debug_bounds("start").is_none());
        let stop = visual.debug_bounds("stop").unwrap();
        assert_eq!(start.origin.x, stop.origin.x);
        assert!(stop.size.height >= px(44.));
        assert!(
            stop.size.height <= start.size.height,
            "Presenting compacts the session strip"
        );
        visual.simulate_click(stop.center(), gpui::Modifiers::default());
        assert_eq!(
            shell.read_with(cx, |shell, _| shell.state.stage()),
            SessionStage::Ready
        );
    }

    #[gpui::test]
    fn idle_presenter_has_no_home_button(cx: &mut TestAppContext) {
        let (_shell, handle) = input_presenter(cx);
        redraw(handle, cx);
        let mut visual = gpui::VisualTestContext::from_window(handle, cx);
        assert!(visual.debug_bounds("home").is_none());
        assert!(visual.debug_bounds("welcome-open").is_some());
        assert!(visual.debug_bounds("home-brand").is_some());
        assert!(visual.debug_bounds("home-empty").is_some());
        for hidden in [
            "open",
            "document-title",
            "stage-label",
            "presenter-timer",
            "start",
            "stop",
            "presenter-bar",
        ] {
            assert!(
                visual.debug_bounds(hidden).is_none(),
                "Idle rendered {hidden}"
            );
        }
    }

    #[gpui::test]
    fn home_history_aligns_known_unknown_times_and_preserves_row_activation(
        cx: &mut TestAppContext,
    ) {
        let (shell, handle) = input_presenter(cx);
        shell.update(cx, |shell, cx| {
            shell.settings.settings.general.ui_font_family = "WenYuan Serif SC".into();
            shell.settings.settings.general.ui_font_size = 18;
            shell.settings.settings.general.recent_files = vec![
                r"C:\演示资料\Very long directory\same.pdf".into(),
                r"D:\Archive\same.pdf".into(),
                "unknown.pdf".into(),
                "hidden-fourth.pdf".into(),
            ];
            shell.settings.settings.general.recent_opened_at = BTreeMap::from([
                (
                    shell.settings.settings.general.recent_files[0].clone(),
                    1_792_375_595,
                ),
                (
                    shell.settings.settings.general.recent_files[2].clone(),
                    i64::MAX,
                ),
            ]);
            cx.notify();
        });
        cx.simulate_window_resize(handle, size(px(1100.), px(760.)));
        redraw(handle, cx);
        handle
            .downcast::<Presenter>()
            .unwrap()
            .read_with(cx, |view, _| {
                let recent = view.recent_probe();
                assert_eq!(recent.len(), 3);
                assert_eq!(recent[0].0, r"C:\演示资料\Very long directory\same.pdf");
                assert_eq!(recent[0].1.len(), 16);
                assert_eq!(recent[1], (r"D:\Archive\same.pdf", "暂无记录"));
                assert_eq!(recent[2], ("unknown.pdf", "暂无记录"));
            })
            .unwrap();
        let mut visual = gpui::VisualTestContext::from_window(handle, cx);
        let header = visual.debug_bounds("recent-time-header").unwrap();
        let mut previous_bottom = header.bottom();
        for index in 0..3 {
            let row = visual
                .debug_bounds(["recent-0", "recent-1", "recent-2"][index])
                .unwrap();
            let name = visual
                .debug_bounds(["recent-0-name", "recent-1-name", "recent-2-name"][index])
                .unwrap();
            let time = visual
                .debug_bounds(["recent-0-time", "recent-1-time", "recent-2-time"][index])
                .unwrap();
            assert_eq!(time.origin.x, header.origin.x);
            assert_eq!(time.right(), header.right());
            assert!(name.right() <= time.origin.x - px(16.));
            assert!(time.top() >= row.top() && time.bottom() <= row.bottom());
            assert!(row.top() >= previous_bottom);
            assert!(
                time.size.height <= px(21.),
                "normal-window timestamp must fit one line: {time:?}"
            );
            previous_bottom = row.bottom();
            println!(
                "HOME_HISTORY_GEOMETRY client=1100x760 font_size=18 index={index} header={header:?} row={row:?} name={name:?} time={time:?}"
            );
        }
        assert!(visual.debug_bounds("recent-3").is_none());
        let history = shell.read_with(cx, |shell, _| shell.settings.settings.general.clone());
        click_control(handle, "recent-0", cx);
        for key in ["enter", "space"] {
            shell.update(cx, |shell, cx| {
                shell.error = None;
                cx.notify();
            });
            redraw(handle, cx);
            cx.simulate_keystrokes(handle, key);
            shell.read_with(cx, |shell, _| {
                assert_eq!(
                    shell.error.as_deref(),
                    Some(shell.locale.text("import-unavailable"))
                );
                assert_eq!(shell.settings.settings.general, history);
            });
        }
    }

    #[gpui::test]
    fn home_layout_keeps_natural_action_and_compact_rows_across_window_range(
        cx: &mut TestAppContext,
    ) {
        for language in ["en-US", "zh-CN"] {
            for theme in [
                ferrostage_settings::Theme::Light,
                ferrostage_settings::Theme::Dark,
            ] {
                let (shell, _) = headless_shell(cx);
                let handle = shell.update(cx, |shell, cx| {
                    shell.locale = crate::locale::Locale::new(language);
                    shell.settings.settings.general.theme = theme;
                    shell.presenter = Some(test_window(shell, false, cx));
                    shell.presenter.as_ref().unwrap().handle
                });
                for count in [0, 1, 3, 4] {
                    shell.update(cx, |shell, cx| {
                        shell.settings.settings.general.recent_files = [
                            r"C:\演示资料\项目评审\同名文件.pdf",
                            r"D:\Archive\Long directory with spaces\同名文件.pdf",
                            r"D:\AnotherVeryLongDirectoryWithoutSpacesForNarrowWindowVerification\presentation-with-a-long-filename.pdf",
                            r"D:\hidden-fourth.pdf",
                        ].into_iter().take(count).map(String::from).collect();
                        cx.notify();
                    });
                    for (width, height) in [
                        (1440., 900.),
                        (1100., 760.),
                        (799., 600.),
                        (800., 600.),
                        (599., 600.),
                        (600., 600.),
                        (320., 760.),
                        (1100., 240.),
                        (320., 240.),
                        (390., 844.),
                        (320., 640.),
                    ] {
                        cx.simulate_window_resize(handle, size(px(width), px(height)));
                        redraw(handle, cx);
                        let mut visual = gpui::VisualTestContext::from_window(handle, cx);
                        let column = visual.debug_bounds("home-content").unwrap();
                        let brand = visual.debug_bounds("home-brand").unwrap();
                        let utilities = visual.debug_bounds("home-utilities").unwrap();
                        let open = visual.debug_bounds("welcome-open").unwrap();
                        let face = visual.debug_bounds("welcome-open-face").unwrap();
                        assert!(column.size.width <= px(760.));
                        assert!(column.origin.x >= px(16.) && column.right() <= px(width - 16.));
                        assert!(face.size.width >= px(120.) && face.size.width <= px(200.));
                        assert!(face.size.height >= px(44.));
                        assert!((open.origin.x - column.origin.x).abs() <= px(2.));
                        assert!((brand.origin.x - column.origin.x).abs() <= px(2.));
                        if width < 600. {
                            assert!(utilities.origin.y >= brand.bottom());
                        }
                        let mut previous_bottom = None;
                        for index in 0..count.min(3) {
                            let row = visual
                                .debug_bounds(["recent-0", "recent-1", "recent-2"][index])
                                .unwrap();
                            assert!(
                                (48. ..=56.).contains(&f32::from(row.size.height)),
                                "{row:?}"
                            );
                            assert!((row.origin.x - column.origin.x).abs() <= px(2.));
                            assert!(row.right() <= column.right());
                            if let Some(bottom) = previous_bottom {
                                assert!(row.origin.y - bottom <= px(8.));
                            }
                            previous_bottom = Some(row.bottom());
                        }
                        assert!(visual.debug_bounds("recent-3").is_none());
                        assert_eq!(visual.debug_bounds("home-empty").is_some(), count == 0);
                        println!(
                            "HOME_GEOMETRY locale={language} theme={theme:?} recents={count} client={width}x{height} column={column:?} brand={brand:?} utilities={utilities:?} open_target={open:?} open_face={face:?}"
                        );
                    }
                }
                handle
                    .update(cx, |_, window, _| window.remove_window())
                    .unwrap();
            }
        }
    }

    #[gpui::test]
    fn home_keyboard_reaches_recents_and_restores_modal_focus_at_minimum_size(
        cx: &mut TestAppContext,
    ) {
        let (shell, handle) = input_presenter(cx);
        shell.update(cx, |shell, cx| {
            shell.settings.settings.general.recent_files =
                vec![r"C:\missing\目录\presentation.pdf".into()];
            cx.notify();
        });
        cx.simulate_window_resize(handle, size(px(320.), px(240.)));
        redraw(handle, cx);
        cx.simulate_keystrokes(handle, "tab enter");
        shell.update(cx, |shell, cx| shell.poll_focus(cx));
        redraw(handle, cx);
        assert_eq!(
            shell.read_with(cx, |shell, _| shell.modal.clone()),
            Some(Modal::Settings)
        );
        cx.simulate_keystrokes(handle, "escape");
        redraw(handle, cx);
        cx.simulate_keystrokes(handle, "tab tab tab"); // Displays, Open, recent.
        cx.run_until_parked();
        cx.test_window(handle).simulate_scheduled_frame();
        redraw(handle, cx);
        let mut visual = gpui::VisualTestContext::from_window(handle, cx);
        assert!(visual.debug_bounds("recent-full-path").is_some());
        let row = visual.debug_bounds("recent-0").unwrap();
        assert!(
            row.origin.y >= px(0.) && row.bottom() <= px(240.),
            "focused row must scroll into view: {row:?}"
        );
        cx.simulate_keystrokes(handle, "shift-tab");
        redraw(handle, cx);
        let mut visual = gpui::VisualTestContext::from_window(handle, cx);
        assert!(visual.debug_bounds("recent-full-path").is_none());
        cx.simulate_keystrokes(handle, "space");
        assert_open_waits_for_owner(&shell, handle, cx);
        redraw(handle, cx);
        shell.read_with(cx, |shell, _| {
            assert_eq!(shell.state.stage(), SessionStage::Idle);
            assert!(!shell.loading);
        });
    }

    #[gpui::test]
    fn home_focused_long_path_wraps_inside_minimum_width(cx: &mut TestAppContext) {
        let (shell, handle) = input_presenter(cx);
        shell.update(cx, |shell, cx| {
            shell.settings.settings.general.recent_files = vec![
                format!(
                    r"C:\{}\演示文稿.pdf",
                    "LongDirectoryWithoutSpaces目录".repeat(8)
                ),
                r"D:\Archive\演示文稿.pdf".into(),
                r"D:\Other\third.pdf".into(),
            ];
            cx.notify();
        });
        cx.simulate_window_resize(handle, size(px(320.), px(240.)));
        redraw(handle, cx);
        cx.simulate_keystrokes(handle, "tab tab tab tab");
        cx.run_until_parked();
        cx.test_window(handle).simulate_scheduled_frame();
        redraw(handle, cx);
        let mut visual = gpui::VisualTestContext::from_window(handle, cx);
        let path = visual.debug_bounds("recent-full-path").unwrap();
        assert!(
            path.origin.x >= px(0.) && path.right() <= px(320.),
            "{path:?}"
        );
        assert!(path.size.height > px(18.), "long path must wrap: {path:?}");
    }

    #[gpui::test]
    fn home_returns_to_welcome_and_keeps_recent_files(cx: &mut TestAppContext) {
        let (shell, handle) = workbench_fixture(cx);
        shell.update(cx, |shell, cx| {
            shell.path = Some(PathBuf::from("raster.pdf"));
            shell.settings.settings.general.recent_files = vec!["kept.pdf".into()];
            shell.loading = true;
            cx.notify();
        });
        redraw(handle, cx);
        click_control(handle, "home", cx);
        shell.read_with(cx, |shell, _| {
            assert_eq!(shell.state.stage(), SessionStage::Ready);
            assert_eq!(shell.path, Some(PathBuf::from("raster.pdf")));
        });
        shell.update(cx, |shell, cx| {
            shell.loading = false;
            cx.notify();
        });
        redraw(handle, cx);
        click_control(handle, "home", cx);
        shell.read_with(cx, |shell, _| {
            assert_eq!(shell.state.stage(), SessionStage::Idle);
            assert!(shell.path.is_none());
            assert!(shell.workbench.document.is_none());
            assert!(shell.workbench.frames.is_empty());
            assert_eq!(shell.state.generation().0, 0);
            assert_eq!(shell.settings.settings.general.recent_files, ["kept.pdf"]);
        });
        redraw(handle, cx);
        let mut visual = gpui::VisualTestContext::from_window(handle, cx);
        assert!(visual.debug_bounds("welcome-open").is_some());
        assert!(visual.debug_bounds("home").is_none());
    }

    #[gpui::test]
    fn home_from_presentation_closes_the_audience(cx: &mut TestAppContext) {
        let (shell, handle) = workbench_fixture(cx);
        let audience = shell.update(cx, |shell, cx| {
            shell.path = Some(PathBuf::from("raster.pdf"));
            shell
                .state
                .apply_at(
                    Command::StartPresentation {
                        audience_available: true,
                    },
                    Instant::now(),
                )
                .unwrap();
            let audience = test_window(shell, true, cx);
            let handle = audience.handle;
            shell.audience = Some(audience);
            cx.notify();
            handle
        });
        redraw(handle, cx);
        click_control(handle, "more", cx);
        redraw(handle, cx);
        click_control(handle, "home", cx);
        shell.read_with(cx, |shell, _| {
            assert_eq!(shell.state.stage(), SessionStage::Idle);
            assert!(shell.path.is_none());
            assert!(shell.workbench.document.is_none());
            assert!(shell.audience.is_none());
        });
        assert!(!cx.windows().contains(&audience));
    }
}
