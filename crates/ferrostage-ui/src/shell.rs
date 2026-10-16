mod preferences;
pub(crate) mod reload;
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
    Document,
    Displays(DisplayDraft),
    Jump(String),
}

struct WindowRecord {
    handle: AnyWindowHandle,
    monitor: MonitorId,
    display: u64,
    fullscreen: bool,
    client_size: gpui::Size<Pixels>,
    resize_attempt: Cell<Option<ClientResizeAttempt>>,
    bounds_observed: Rc<Cell<bool>>,
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
    preference_change: Option<preferences::PendingPreferences>,
    store: SettingsStore,
    renderer: Result<RenderHandle, RenderError>,
    topology: Receiver<TopologyEvent>,
    _watcher: Result<TopologyWatcher, ferrostage_platform::PlatformError>,
    monitors: Vec<MonitorInfo>,
    topology_valid: bool,
    topology_revision: u64,
    identify: Vec<AnyWindowHandle>,
    identify_deadline: Option<Instant>,
    recover_windowed: bool,
    mapped: Result<BTreeMap<MonitorId, u64>, ShellError>,
    presenter: Option<WindowRecord>,
    audience: Option<WindowRecord>,
    pending: Option<Transaction>,
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
        let (topology_valid, monitors, error) = match inputs.initial_monitors {
            Ok(monitors) => (true, monitors, None),
            Err(error) => (false, Vec::new(), Some(error.to_string())),
        };
        let warnings = inputs
            .settings
            .warnings
            .iter()
            .map(|warning| format!("{}：{}", warning.key, warning.reason))
            .collect();
        let error = inputs
            .watcher
            .as_ref()
            .err()
            .map(ToString::to_string)
            .or(error);
        let escape_stops = policy::bindings(&inputs.settings.settings.keybindings, locale)
            .0
            .iter()
            .any(|(key, action)| key == "escape" && *action == "stop_presentation");
        let recover_windowed = !topology_valid || monitors.is_empty();
        Self {
            locale,
            state,
            workbench: Default::default(),
            media: Default::default(),
            next_generation: 1,
            settings: inputs.settings,
            store: inputs.store,
            preferences: Default::default(),
            preference_change: None,
            renderer: inputs.renderer,
            topology: inputs.topology,
            _watcher: inputs.watcher,
            monitors,
            topology_valid,
            topology_revision: 0,
            identify: Vec::new(),
            identify_deadline: None,
            recover_windowed,
            mapped: Err(ShellError::AmbiguousDisplays),
            presenter: None,
            audience: None,
            pending: None,
            focus_intent: None,
            focus_token: 0,
            import: None,
            reload: Default::default(),
            import_token: 0,
            loading: false,
            path: None,
            error,
            graphics_error: None,
            warnings,
            modal: None,
            snapshot,
            escape_stops,
            poll_task: None,
            import_read_task: None,
            path_prompt_task: None,
        }
    }

    fn refresh_mapping(&mut self, cx: &App) {
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

    pub fn busy(&self) -> bool {
        self.pending.is_some()
            || self.loading
            || self.preference_change.is_some()
            || self.reload_busy()
    }

    fn publish(&mut self, curtain: bool, cx: &mut Context<Self>) {
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
        self.snapshot.update(cx, |value, cx| {
            *value = snapshot;
            cx.notify();
        });
    }

    fn apply(&mut self, command: Command, cx: &mut Context<Self>) {
        if let Err(error) = self.state.apply_at(command, Instant::now()) {
            self.error = Some(error.to_string());
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
        match intent {
            Intent::Media(id) => {
                if self.modal.is_none() && self.state.jump_preview().is_none() {
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
            Intent::Core(command) if self.pending.is_none() && self.modal.is_none() => {
                self.apply(command, cx)
            }
            Intent::Tool(tool) => {
                self.workbench.tool = if self.workbench.tool == tool {
                    crate::workbench::Tool::Navigate
                } else {
                    tool
                };
                self.apply(Command::EndStroke, cx);
                self.apply(Command::UpdateLaserPointer(None), cx);
            }
            Intent::Grid => {
                self.workbench.grid = !self.workbench.grid;
            }
            Intent::GridMove(delta) => {
                let count = self
                    .workbench
                    .document
                    .as_ref()
                    .map_or(0, |s| s.document.slide_map.steps().len());
                self.workbench.grid_start = self
                    .workbench
                    .grid_start
                    .saturating_add_signed(delta)
                    .min(count.saturating_sub(1));
            }
            Intent::Preview(page) => {
                self.apply(Command::JumpPreview(page), cx);
            }
            Intent::Document if !self.busy() => self.modal = Some(Modal::Document),
            Intent::Reconfigure(options) => {
                if let Some(session) = &mut self.workbench.document {
                    match session.reconfigure(options).and_then(|document| {
                        let descriptor = document
                            .descriptor(self.state.generation())
                            .map_err(|e| e.to_string())?;
                        self.state
                            .apply_at(Command::ReconfigureDocument(descriptor), Instant::now())
                            .map_err(|e| e.to_string())?;
                        session.document = document;
                        Ok(())
                    }) {
                        Ok(()) => {
                            self.workbench.clear_frames();
                            self.error = None;
                        }
                        Err(error) => self.error = Some(error),
                    }
                }
            }
            Intent::Split { next, notes, save }
                if !self.busy() && self.settings.write_policy == WritePolicy::Writable =>
            {
                self.settings.settings.presenter.next_slide_ratio = next.clamp(0.1, 0.8);
                self.settings.settings.presenter.notes_ratio = notes.clamp(0.1, 0.8);
                self.settings.note_edit();
                if save {
                    self.save(cx);
                }
            }
            Intent::Escape => {
                self.dismiss_preferences();
                self.cancel_focus();
                self.close_identify(cx);
                match transactions::escape(
                    self.modal.take().is_some(),
                    self.state.jump_preview().is_some(),
                    self.pending.is_some() || self.escape_stops,
                ) {
                    transactions::Escape::CancelPreview => self.apply(Command::CancelJump, cx),
                    transactions::Escape::Stop => self.stop(cx),
                    transactions::Escape::CloseModal | transactions::Escape::Nothing => {}
                }
            }
            Intent::Stop => self.stop(cx),
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
            Intent::Identify => self.identify_displays(cx),
            Intent::Import(path) => self.begin_import(path, cx),
            Intent::RetrySave => {
                if !self.busy() {
                    self.save(cx);
                }
            }
            Intent::Commit => {
                if self.modal.is_none() && self.state.jump_preview().is_some() {
                    self.apply(Command::CommitJump, cx);
                }
                if let Some(Modal::Jump(text)) = &self.modal {
                    if let Some(page) = transactions::jump_page(text, self.state.page_count())
                        .filter(|page| {
                            self.state
                                .slide_map()
                                .and_then(|map| map.logical_for(PhysicalPage(*page)))
                                .is_some()
                        })
                    {
                        self.apply(Command::JumpPreview(PhysicalPage(page)), cx);
                        self.modal = None;
                        self.apply(Command::CommitJump, cx);
                    } else {
                        self.error = Some(self.locale.text("invalid-page").into());
                    }
                }
            }
            action
                if self.modal.is_none()
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

    pub fn jump_key(&mut self, key: &str, cx: &mut Context<Self>) {
        if let Some(Modal::Jump(text)) = &mut self.modal {
            if key == "backspace" {
                text.pop();
            } else if key.len() == 1 && key.as_bytes()[0].is_ascii_digit() && text.len() < 10 {
                text.push_str(key);
            } else {
                return;
            }
            if let Some(page) = transactions::jump_page(text, self.state.page_count()) {
                self.error = None;
                self.apply(Command::JumpPreview(PhysicalPage(page)), cx);
            } else {
                self.error = Some(self.locale.text("invalid-page").into());
            }
            cx.notify();
        }
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
        if !self.import_allowed() {
            self.error = Some(self.locale.text("stop-before-import").into());
            return;
        }
        if self.queue_reload_import(path.clone()) {
            return;
        }
        if let Err(error) = &self.renderer {
            self.error = Some(self.locale.error("import-unavailable", error));
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
                                    .map_err(std::io::Error::other)
                            })
                            .await
                    }
                    Err(error) => Err(error),
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
                                    shell.error = Some(error);
                                }
                            }
                        }
                        Err(error) => {
                            shell.loading = false;
                            shell.error = Some(shell.locale.error("read-error", error));
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
            Err(error) => self.error = Some(self.locale.error("import-error", error)),
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
                self.workbench.clear_frames();
                self.workbench.document = Some(session);
                if old.0 != 0
                    && let Ok(worker) = &self.renderer
                {
                    let _ = worker.retire_generation(old);
                }
                self.error = None;
                if self.settings.write_policy == WritePolicy::Writable {
                    let recent = &mut self.settings.settings.general.recent_files;
                    let path = path.to_string_lossy().into_owned();
                    recent.retain(|old| old != &path);
                    recent.insert(0, path);
                    recent.truncate(self.settings.settings.general.recent_files_limit as usize);
                    self.settings.note_edit();
                    self.save(cx);
                }
                self.publish(false, cx);
            }
            Err(error) => self.error = Some(error.to_string()),
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
                self.preferences = Default::default();
                self.modal = Some(Modal::Settings);
            }
            Intent::Displays => {
                self.detect_displays(cx);
                self.new_display_draft();
            }
            Intent::Jump if self.state.page_count() > 0 => {
                self.modal = Some(Modal::Jump(String::new()));
                self.apply(Command::JumpPreview(self.state.current_page()), cx);
            }
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
        let handle: AnyWindowHandle = if audience {
            let snapshot = self.snapshot.clone();
            cx.open_window(options, move |window, cx| {
                Self::intercept_close(window, cx);
                cx.new(|cx| {
                    cx.observe(&snapshot, |_, _, cx| cx.notify()).detach();
                    cx.observe_window_bounds(window, move |_, window, cx| {
                        flag.set(true);
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
                Self::intercept_close(window, cx);
                cx.new(|cx| {
                    cx.observe_window_bounds(window, move |_, window, cx| {
                        flag.set(true);
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
        self.close_identify(cx);
        self.refresh_mapping(cx);
        if let Some(reason) = self.start_reason() {
            self.error = Some(reason);
            return;
        }
        match policy::audience_target(&self.monitors, &self.settings.settings.display) {
            Ok((target, fullscreen)) => {
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
                    Err(error) => self.error = Some(error),
                }
            }
            Err(error) => self.error = Some(error.localized(self.locale)),
        }
    }

    fn retire_pending(&mut self, reason: &'static str, cx: &mut Context<Self>) {
        if let Some(pending) = self.pending.take() {
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
        }
    }

    fn cancel_transaction(&mut self, reason: &'static str, cx: &mut Context<Self>) {
        self.retire_pending(reason, cx);
        self.publish(false, cx);
    }

    fn abandon_modal(&mut self) {
        self.modal = None;
        if self.state.jump_preview().is_some()
            && let Err(error) = self.state.apply_at(Command::CancelJump, Instant::now())
        {
            self.error = Some(error.to_string());
        }
        // The caller publishes its Curtain/transaction result. Do not publish
        // a Live snapshot while Stop or Swap is abandoning the Jump UI.
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
        cx.notify();
    }

    fn swap(&mut self, cx: &mut Context<Self>) {
        if self.pending.is_some() || self.loading || self.preference_change.is_some() {
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
        if self.pending.is_some() || !self.topology_valid {
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
        })
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
        if let Some(problem) = panel.problem {
            self.error = Some(problem);
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
                .is_none_or(|window| Self::confirmed(window, cx));
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
        self.poll_focus(cx);
        if let Some(audience) = &self.audience {
            if let Ok(target) = audience.handle.update(cx, |_, window, _| {
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
        if let Some(presenter) = &self.presenter
            && let Ok(target) = presenter.handle.update(cx, |_, window, _| {
                let size = window.viewport_size();
                let scale = window.scale_factor();
                ferrostage_pdf::PixelSize {
                    width: (f32::from(size.width)
                        * scale
                        * (1. - self.settings.settings.presenter.next_slide_ratio as f32))
                        .round()
                        .max(1.) as u32,
                    height: ((f32::from(size.height) - 300.).max(100.) * scale)
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
            }
            TopologyEvent::Unavailable { error, .. } => {
                self.topology_valid = false;
                self.error = Some(error.to_string());
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
            assert!(
                shell
                    .error
                    .as_ref()
                    .unwrap()
                    .contains("controlled pending read")
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
            assert_eq!(shell.error.as_deref(), Some(shell.locale.text("displays-stale")));
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
        cx.simulate_keystrokes(handle, "tab enter");
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
        cx.simulate_keystrokes(handle, "shift-tab enter");
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
            pages: vec![ferrostage_pdf::structure::PageStructure {
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
            assert!(shell.settings.save_error.is_none());
            assert!(!shell.settings.dirty);
            assert_eq!(shell.error.as_deref(), Some("unrelated operation error"));
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
        cx.simulate_keystrokes(handle, "tab");
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

        cx.simulate_keystrokes(handle, "tab"); // Return to Open.
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
        cx.simulate_keystrokes(handle, "space enter"); // Same now-disabled Open.
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
            cx.simulate_keystrokes(handle, "shift-tab");
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
                    shell.modal,
                    Some(Modal::Jump("2".into())),
                    "numeric binding: {numeric_binding}"
                );
                assert_eq!(shell.state.current_page(), PhysicalPage(0));
                assert_eq!(shell.state.jump_preview(), Some(PhysicalPage(1)));
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
            shell.read_with(cx, |shell, _| shell.modal.clone()),
            Some(Modal::Jump("2".into()))
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
    fn v5_stop_discards_jump_preview_before_following_navigation(cx: &mut TestAppContext) {
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
            assert!(shell.state.jump_preview().is_none());
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
    fn v5_swap_failure_discards_abandoned_jump_preview(cx: &mut TestAppContext) {
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
            shell.modal = Some(Modal::Jump("2".into()));
            shell
                .state
                .apply_at(Command::JumpPreview(PhysicalPage(1)), Instant::now())
                .unwrap();
            cx.notify();
        });
        cx.dispatch_action(handle, actions::Swap);
        shell.read_with(cx, |shell, _| {
            assert!(
                shell.error.is_some(),
                "unmapped test display makes replacement fail"
            );
            assert!(shell.modal.is_none());
            assert!(shell.state.jump_preview().is_none());
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
    fn v5_escape_from_jump_cancels_preview_without_stopping(cx: &mut TestAppContext) {
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
            shell
                .state
                .apply_at(Command::JumpPreview(PhysicalPage(1)), Instant::now())
                .unwrap();
            shell.modal = Some(Modal::Jump("2".into()));
            cx.notify();
        });
        redraw(handle, cx);
        cx.simulate_keystrokes(handle, "escape");
        shell.read_with(cx, |shell, _| {
            assert!(shell.modal.is_none());
            assert!(shell.state.jump_preview().is_none());
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
            shell
                .state
                .apply_at(Command::JumpPreview(PhysicalPage(1)), Instant::now())
                .unwrap();
            shell.modal = Some(Modal::Jump("2".into()));
            handle
        });
        sender.send(event).unwrap();
        shell.update(cx, |shell, cx| shell.tick(cx));
        shell.read_with(cx, |shell, cx| {
            assert_eq!(shell.state.stage(), SessionStage::Ready);
            assert!(shell.audience.is_none());
            assert!(shell.state.jump_preview().is_none());
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
            shell
                .state
                .apply_at(Command::JumpPreview(PhysicalPage(2)), Instant::now())
                .unwrap();
            shell.modal = Some(Modal::Jump("3".into()));
        });
        audience
            .update(cx, |_, window, _| window.remove_window())
            .unwrap();
        shell.read_with(cx, |shell, cx| {
            assert_eq!(shell.state.stage(), SessionStage::Ready);
            assert!(shell.audience.is_none());
            assert!(shell.state.jump_preview().is_none());
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
    fn workbench_narrow_stacks_and_grid_wheel_moves_only_private_preview(cx: &mut TestAppContext) {
        let (shell, handle) = workbench_fixture(cx);
        shell.update(cx, |shell, cx| {
            shell.workbench.grid = true;
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
                pages: (0..4)
                    .map(|index| ferrostage_pdf::structure::PageStructure {
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

    #[gpui::test]
    fn workbench_controls_reconfigure_preview_and_keep_note_pages_private(cx: &mut TestAppContext) {
        let (shell, handle) = workbench_fixture(cx);
        let mut visual = gpui::VisualTestContext::from_window(handle, cx);
        let pen = visual.debug_bounds("pen").unwrap();
        visual.simulate_click(pen.center(), gpui::Modifiers::default());
        assert_eq!(
            shell.read_with(cx, |shell, _| shell.workbench.tool),
            crate::workbench::Tool::Pen
        );
        shell.update(cx, |shell, cx| {
            shell.dispatch(Intent::Preview(PhysicalPage(2)), cx);
            assert_eq!(shell.state.current_page(), PhysicalPage(0));
            shell.modal = Some(Modal::Jump("2".into())); // physical 2 is a private notes page
            shell.dispatch(Intent::Commit, cx);
            assert!(shell.modal.is_some());
            assert_eq!(shell.state.current_page(), PhysicalPage(0));
            shell.dispatch(Intent::Escape, cx);
            assert!(shell.state.jump_preview().is_none());
            shell.dispatch(Intent::Preview(PhysicalPage(2)), cx);
            shell.dispatch(Intent::Commit, cx);
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
            shell.dispatch(Intent::Reconfigure(options), cx);
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
}
