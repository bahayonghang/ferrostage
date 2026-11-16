use crate::{
    actions::{self, Intent},
    audience::Audience,
    policy::{self, ShellError},
    presenter::{Presenter, PresenterData},
    transactions::{self, Operation, Shutdown},
};
use ferrostage_core::{
    AudienceProjectionSnapshot, AudienceStage, BlankMode, MonitorId, PhysicalPage,
    PresentationCommand as Command, PresentationState, SessionStage,
};
use ferrostage_pdf::DocumentMetadata;
use ferrostage_platform::{MonitorInfo, TopologyEvent, TopologyWatcher};
use ferrostage_render::{RenderError, RenderHandle, Reply};
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
    sync::mpsc::{Receiver, TryRecvError},
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
    Jump(String),
}

struct WindowRecord {
    handle: AnyWindowHandle,
    monitor: MonitorId,
    display: u64,
    fullscreen: bool,
    bounds_observed: Rc<Cell<bool>>,
}

enum TransactionKind {
    Start,
    Swap(DisplaySettings),
    RecoverPresenter,
}
impl TransactionKind {
    fn operation(&self) -> Operation {
        match self {
            Self::Start => Operation::Start,
            Self::Swap(_) => Operation::Swap,
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
    presenter: WindowId,
    deadline: Instant,
}
struct ImportReply {
    token: u64,
    path: PathBuf,
    reply: Reply<DocumentMetadata>,
}

pub(crate) struct Shell {
    pub locale: crate::locale::Locale,
    pub state: PresentationState,
    pub settings: SettingsSession,
    store: SettingsStore,
    renderer: Result<RenderHandle, RenderError>,
    topology: Receiver<TopologyEvent>,
    _watcher: Result<TopologyWatcher, ferrostage_platform::PlatformError>,
    monitors: Vec<MonitorInfo>,
    topology_valid: bool,
    mapped: Result<BTreeMap<MonitorId, u64>, ShellError>,
    presenter: Option<WindowRecord>,
    audience: Option<WindowRecord>,
    pending: Option<Transaction>,
    focus_intent: Option<FocusIntent>,
    focus_token: u64,
    import: Option<ImportReply>,
    import_token: u64,
    pub loading: bool,
    pub path: Option<PathBuf>,
    pub error: Option<String>,
    pub warnings: Vec<String>,
    pub modal: Option<Modal>,
    pub snapshot: Entity<AudienceProjectionSnapshot>,
    escape_stops: bool,
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
            cx.spawn(async move |weak, cx| {
                loop {
                    cx.background_executor()
                        .timer(Duration::from_millis(25))
                        .await;
                    if weak.update(cx, |shell, cx| shell.tick(cx)).is_err() {
                        break;
                    }
                }
            })
            .detach();
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
    fn new(inputs: ShellInputs, cx: &mut Context<Self>) -> Self {
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
        Self {
            locale,
            state,
            settings: inputs.settings,
            store: inputs.store,
            renderer: inputs.renderer,
            topology: inputs.topology,
            _watcher: inputs.watcher,
            monitors,
            topology_valid,
            mapped: Err(ShellError::AmbiguousDisplays),
            presenter: None,
            audience: None,
            pending: None,
            focus_intent: None,
            focus_token: 0,
            import: None,
            import_token: 0,
            loading: false,
            path: None,
            error,
            warnings,
            modal: None,
            snapshot,
            escape_stops,
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
        if self.loading || self.pending.is_some() {
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
        self.pending.is_some() || self.loading
    }

    fn publish(&self, curtain: bool, cx: &mut Context<Self>) {
        let mut snapshot = self.state.audience_snapshot();
        if curtain {
            snapshot.stage = AudienceStage::Curtain;
            snapshot.visual = None;
        }
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
        match intent {
            Intent::Escape => {
                self.cancel_focus();
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
            Intent::Open | Intent::Settings | Intent::Jump => self.request_focus(intent, cx),
            Intent::Import(path) => self.begin_import(path, cx),
            Intent::RetrySave => self.save(cx),
            Intent::Commit => {
                if let Some(Modal::Jump(text)) = &self.modal {
                    if let Some(page) = transactions::jump_page(text, self.state.page_count()) {
                        self.apply(Command::JumpPreview(PhysicalPage(page)), cx);
                        self.modal = None;
                        self.apply(Command::CommitJump, cx);
                    } else {
                        self.error = Some(self.locale.text("invalid-page").into());
                    }
                }
            }
            action if self.modal.is_none() && self.pending.is_none() => {
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
            self.error = Some(self.locale.error("save-error", error));
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

    fn begin_import(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        if !transactions::import_allowed(
            self.state.stage(),
            self.pending
                .as_ref()
                .map(|pending| pending.kind.operation()),
        ) {
            self.error = Some(self.locale.text("stop-before-import").into());
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
        cx.spawn(async move |weak, cx| {
            let bytes = read.await;
            if let Err(error) = weak.update(cx, |shell, cx| {
                if shell.import_token != token {
                    return;
                }
                match bytes {
                    Ok(bytes) => match shell
                        .renderer
                        .as_ref()
                        .map_err(|e| e.to_string())
                        .and_then(|worker| worker.probe(bytes).map_err(|e| e.to_string()))
                    {
                        Ok(reply) => shell.import = Some(ImportReply { token, path, reply }),
                        Err(error) => {
                            shell.loading = false;
                            shell.error = Some(error);
                        }
                    },
                    Err(error) => {
                        shell.loading = false;
                        shell.error = Some(shell.locale.error("read-error", error));
                    }
                }
                cx.notify();
            }) {
                tracing::debug!(%error,"import target closed");
            }
        })
        .detach();
    }

    fn poll_import(&mut self, cx: &mut Context<Self>) {
        let result = self
            .import
            .as_ref()
            .and_then(|pending| match pending.reply.try_recv() {
                Ok(result) => Some(result.map_err(|e| e.to_string())),
                Err(TryRecvError::Disconnected) => {
                    Some(Err(self.locale.text("worker-closed").into()))
                }
                Err(TryRecvError::Empty) => None,
            });
        let Some(result) = result else {
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
            Ok(metadata) => {
                match transactions::commit_import(
                    &mut self.state,
                    &mut self.path,
                    pending.path.clone(),
                    metadata.pages.len(),
                    Instant::now(),
                ) {
                    Ok(_) => {
                        self.error = None;
                        if self.settings.write_policy == WritePolicy::Writable {
                            let recent = &mut self.settings.settings.general.recent_files;
                            let path = pending.path.to_string_lossy().into_owned();
                            recent.retain(|old| old != &path);
                            recent.insert(0, path);
                            recent.truncate(
                                self.settings.settings.general.recent_files_limit as usize,
                            );
                            self.settings.note_edit();
                            self.save(cx);
                        }
                        self.publish(false, cx);
                    }
                    Err(error) => self.error = Some(error.to_string()),
                }
            }
            Err(error) => self.error = Some(self.locale.error("import-error", error)),
        }
        cx.notify();
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
            Intent::Open => {
                let focus_token = self.focus_token;
                let receiver = cx.prompt_for_paths(PathPromptOptions {
                    files: true,
                    directories: false,
                    multiple: false,
                    prompt: Some(self.locale.text("open").into()),
                });
                cx.spawn(async move |weak, cx| {
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
                })
                .detach();
            }
            Intent::Settings => self.modal = Some(Modal::Settings),
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
            .bounds();
        let width = f32::from(bounds.size.width).min(if audience { 800. } else { 1100. });
        let height = f32::from(bounds.size.height).min(760.);
        let restore = Bounds::new(
            point(
                bounds.origin.x + px((f32::from(bounds.size.width) - width) / 2.),
                bounds.origin.y + px((f32::from(bounds.size.height) - height) / 2.),
            ),
            size(px(width), px(height)),
        );
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
        let handle = if audience {
            let snapshot = self.snapshot.clone();
            cx.open_window(options, move |window, cx| {
                cx.new(|cx| {
                    cx.observe(&snapshot, |_, _, cx| cx.notify()).detach();
                    cx.observe_window_bounds(window, move |_, _, _| flag.set(true))
                        .detach();
                    Audience { snapshot }
                })
            })
            .map(Into::into)
        } else {
            let shell = cx.weak_entity();
            let locale = self.locale;
            let data = PresenterData::from_shell(self);
            cx.open_window(options, move |window, cx| {
                cx.new(|cx| {
                    cx.observe_window_bounds(window, move |_, _, _| flag.set(true))
                        .detach();
                    cx.observe_window_appearance(window, |_, _, cx| cx.notify())
                        .detach();
                    Presenter::new(shell, locale, data, window, cx)
                })
            })
            .map(Into::into)
        }
        .map_err(|error| error.to_string())?;
        tracing::info!(?handle,monitor=?monitor,target_display,fullscreen,audience, "window created behind confirmation gate");
        Ok(WindowRecord {
            handle,
            monitor,
            display: target_display,
            fullscreen,
            bounds_observed: observed,
        })
    }

    fn confirmed(record: &WindowRecord, cx: &mut App) -> bool {
        record
            .handle
            .update(cx, |_, window, cx| {
                let display = window.display(cx);
                let Some(display) = display else {
                    return false;
                };
                policy::placement_confirmed(
                    record.display,
                    Some(u64::from(display.id())),
                    record.fullscreen,
                    window.is_fullscreen(),
                    record.bounds_observed.get(),
                    rect(window.bounds()),
                    rect(display.bounds()),
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
                actual_bounds=?rect(window.bounds()), actual_fullscreen=window.is_fullscreen(),
                bounds_observed=record.bounds_observed.get(), "window placement observation"
            );
        }) {
            tracing::info!(reason, window=?record.handle.window_id(), %error, "window placement unavailable");
        }
    }

    fn start(&mut self, cx: &mut Context<Self>) {
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
            tracing::info!(reason, operation=?pending.kind.operation(), "window transaction cancelled");
            if let Some(window) = pending.audience {
                Self::log_placement(&window, reason, cx);
                Self::close(window, cx);
            }
            if let Some(window) = pending.presenter {
                Self::log_placement(&window, reason, cx);
                Self::close(window, cx);
            }
        }
    }

    fn cancel_transaction(&mut self, reason: &'static str, cx: &mut Context<Self>) {
        self.retire_pending(reason, cx);
        self.publish(false, cx);
    }

    fn stop(&mut self, cx: &mut Context<Self>) {
        self.cancel_focus();
        self.modal = None;
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
        if self.pending.is_some() || self.loading {
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
        let Some(target) = policy::roles(&self.monitors, &settings).presenter else {
            return;
        };
        self.cancel_focus();
        self.modal = None;
        self.publish(true, cx);
        let prepared =
            transactions::prepare(self.state.stage() == SessionStage::Presenting, |audience| {
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
                    kind: TransactionKind::Swap(settings),
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
        if self
            .presenter
            .as_ref()
            .is_some_and(|p| self.monitors.iter().any(|m| m.id == p.monitor))
        {
            return;
        }
        let fullscreen =
            self.presenter.is_none() && self.settings.settings.display.presenter_start_fullscreen;
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

    fn poll_transaction(&mut self, cx: &mut Context<Self>) {
        let Some(pending) = &self.pending else {
            return;
        };
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
        let old_audience = if pending.audience.is_some() {
            std::mem::replace(&mut self.audience, pending.audience)
        } else {
            None
        };
        if let TransactionKind::Swap(settings) = pending.kind {
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
        tracing::info!(stage=?self.state.stage(),presenter=?self.presenter.as_ref().map(|w|w.handle),audience=?self.audience.as_ref().map(|w|w.handle),"window transaction committed");
        cx.notify();
    }

    fn tick(&mut self, cx: &mut Context<Self>) {
        while let Ok(event) = self.topology.try_recv() {
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
                self.stop(cx);
            } else {
                self.publish(false, cx);
            }
            self.recover_presenter(cx);
            cx.notify();
        }
        self.poll_import(cx);
        self.poll_transaction(cx);
        self.poll_focus(cx);
        // If startup had zero screens, the platform watcher owns recovery.
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
    use gpui::TestAppContext;

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

    fn input_presenter(cx: &mut TestAppContext) -> (Entity<Shell>, AnyWindowHandle) {
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
    fn unsupported_shortcuts_use_user_visible_feature_names(cx: &mut TestAppContext) {
        let (shell, _) = headless_shell(cx);
        for language in ["en-US", "zh-CN"] {
            cx.update(|cx| {
                let keys = shell.read(cx).settings.settings.keybindings.clone();
                let locale = crate::locale::Locale::new(language);
                let warnings = actions::install(shell.downgrade(), &keys, locale, cx);
                assert!(warnings.iter().all(|warning| !warning.contains("toggle_")));
                for key in ["feature-laser", "feature-pen"] {
                    assert!(
                        warnings
                            .iter()
                            .any(|warning| warning.contains(locale.text(key)))
                    );
                }
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
}
