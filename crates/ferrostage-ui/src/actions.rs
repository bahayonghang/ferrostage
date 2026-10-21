use crate::{policy, shell::Shell};
use gpui::{App, KeyBinding, WeakEntity, actions};
use std::path::PathBuf;

actions!(
    ferrostage,
    [
        Next,
        Previous,
        Start,
        Stop,
        Black,
        White,
        Swap,
        Open,
        Settings,
        Jump,
        Escape,
        Commit,
        ActivateButton,
        Laser,
        Pen
    ]
);

#[derive(Clone, Debug)]
pub(crate) enum Intent {
    ToggleAutoplay,
    Media(u64),
    SettingsCategory(crate::settings_panel::Category),
    EditSetting(&'static str, String),
    BeginSettingEdit(&'static str),
    ToggleFontPicker,
    FontPickerMove(isize),
    SettingsKey {
        key: String,
        text: Option<String>,
    },
    RecordBinding(&'static str),
    RemoveBinding(&'static str, usize),
    ResetBindings,
    ClearRecent,
    OpenConfigDirectory,
    ResetPreferences(bool),
    SurfaceSize(crate::workbench::Surface, ferrostage_pdf::PixelSize),
    Core(ferrostage_core::PresentationCommand),
    Tool(crate::workbench::Tool),
    GridMove(isize),
    BrowserSubmit {
        id: u64,
    },
    BrowserMode {
        id: u64,
        mode: crate::workbench::PageNumberMode,
    },
    BrowserView {
        id: u64,
        view: crate::workbench::BrowserView,
    },
    BrowserOutlineJump {
        id: u64,
        entry_id: usize,
    },
    BrowserEdit {
        id: u64,
        edit: crate::page_browser::NumberEdit,
    },
    BrowserJump {
        id: u64,
        page: ferrostage_core::PhysicalPage,
    },
    GridDensity {
        column: bool,
        value: u32,
    },
    CloseContextual,
    PresenterMenu(bool),
    Document,
    StartModeMenu,
    DismissError,
    DismissDocumentWarning(String),
    OpenRepository,
    SetStartMode(ferrostage_settings::StartMode),
    Reconfigure {
        id: u64,
        options: ferrostage_document::DocumentOverrides,
    },
    /// Divider drag. `primary`/`secondary` are the two ratios of `layout`
    /// (see `WorkbenchLayout::ratios`).
    Split {
        layout: crate::workbench::WorkbenchLayout,
        primary: f64,
        secondary: f64,
        save: bool,
    },
    Next,
    Previous,
    Start,
    /// Start that continues the timer frozen by the last stop.
    Resume,
    Stop,
    Black,
    White,
    Swap,
    Open,
    Home,
    Settings,
    Jump,
    Escape,
    RetrySave,
    Displays,
    Identify,
    RefreshDisplays,
    SelectPresenter(ferrostage_core::MonitorId),
    SelectPdf(crate::display_settings::PdfTarget),
    ApplyDisplays,
    CancelDisplayDraft,
    ToggleDisplayDetails(ferrostage_core::MonitorId),
    Import(PathBuf),
}

pub(crate) fn dispatch(shell: &WeakEntity<Shell>, intent: Intent, cx: &mut App) {
    let shell = shell.clone();
    // Input callbacks run while GPUI has taken the current Window out of App.
    // Release that window before coordinator actions update/activate/remove it.
    cx.defer(move |cx| {
        if let Err(error) = shell.update(cx, |shell, cx| shell.dispatch(intent, cx)) {
            tracing::debug!(%error, "action target retired");
        }
    });
}

pub(crate) fn install(
    shell: WeakEntity<Shell>,
    settings: &ferrostage_settings::Keybindings,
    locale: crate::locale::Locale,
    cx: &mut App,
) -> Vec<String> {
    let editing = shell.clone();
    cx.intercept_keystrokes(move |event, window, cx| {
        if let Some(shell) = editing.upgrade() {
            let shell = shell.read(cx);
            if shell.captures_preferences(window.window_handle().window_id())
                || shell.browser().is_some() && shell.number_input_focus.is_focused(window)
                || (shell.layout_menu_open()
                    || shell.browser().is_some_and(|_| {
                        shell.workbench.browser_view == crate::workbench::BrowserView::Outline
                    })
                    || matches!(
                        shell.contextual,
                        Some(
                            crate::page_browser::ContextualSurface::PresenterMenu { .. }
                                | crate::page_browser::ContextualSurface::StartMode
                        )
                    ))
                    && matches!(
                        event.keystroke.unparse().as_str(),
                        "up" | "down"
                            | "left"
                            | "right"
                            | "home"
                            | "end"
                            | "pageup"
                            | "pagedown"
                            | "enter"
                            | "space"
                            | "tab"
                            | "shift-tab"
                            | "escape"
                    )
            {
                // Preserve the raw event for Presenter capture; skip keymap actions.
                cx.stop_propagation();
            }
        }
    })
    .detach();
    macro_rules! register {
        ($action:ident, $intent:ident) => {{
            let target = shell.clone();
            cx.on_action(move |_: &$action, cx| dispatch(&target, Intent::$intent, cx));
        }};
    }
    let target = shell.clone();
    cx.on_action(move |_: &Laser, cx| {
        dispatch(&target, Intent::Tool(crate::workbench::Tool::Laser), cx)
    });
    let target = shell.clone();
    cx.on_action(move |_: &Pen, cx| {
        dispatch(&target, Intent::Tool(crate::workbench::Tool::Pen), cx)
    });
    register!(Next, Next);
    register!(Previous, Previous);
    let target = shell.clone();
    cx.on_action(move |_: &Start, cx| {
        let target = target.clone();
        // Resolve after the window lease, as `dispatch` does, so the choice
        // reads the session state the coordinator will act on.
        cx.defer(move |cx| {
            if let Err(error) = target.update(cx, |shell, cx| {
                let intent = if shell.resumable_elapsed().is_some() {
                    Intent::Resume
                } else {
                    Intent::Start
                };
                shell.dispatch(intent, cx)
            }) {
                tracing::debug!(%error, "action target retired");
            }
        });
    });
    register!(Stop, Stop);
    register!(Black, Black);
    register!(White, White);
    register!(Swap, Swap);
    register!(Open, Open);
    register!(Settings, Settings);
    register!(Jump, Jump);
    register!(Escape, Escape);
    let submit = shell.clone();
    cx.on_action(move |_: &Commit, cx| {
        let id = submit
            .upgrade()
            .and_then(|shell| shell.read(cx).browser().map(|browser| browser.id));
        if let Some(id) = id {
            dispatch(&submit, Intent::BrowserSubmit { id }, cx);
        }
    });
    replace_bindings(settings, locale, cx)
}

pub(crate) fn replace_bindings(
    settings: &ferrostage_settings::Keybindings,
    locale: crate::locale::Locale,
    cx: &mut App,
) -> Vec<String> {
    cx.clear_key_bindings();
    let (bindings, mut warnings) = policy::bindings(settings, locale);
    for (key, action) in bindings {
        // Escape is routed once, with modal consumption before stop.
        if key == "escape" {
            continue;
        }
        match action {
            "next" => cx.bind_keys([KeyBinding::new(&key, Next, None)]),
            "previous" => cx.bind_keys([KeyBinding::new(&key, Previous, None)]),
            "start_presentation" => cx.bind_keys([KeyBinding::new(&key, Start, None)]),
            "stop_presentation" => cx.bind_keys([KeyBinding::new(&key, Stop, None)]),
            "blank_black" => cx.bind_keys([KeyBinding::new(&key, Black, None)]),
            "blank_white" => cx.bind_keys([KeyBinding::new(&key, White, None)]),
            "swap_screens" => cx.bind_keys([KeyBinding::new(&key, Swap, None)]),
            "open" => cx.bind_keys([KeyBinding::new(&key, Open, None)]),
            "settings" => cx.bind_keys([KeyBinding::new(&key, Settings, None)]),
            "commit" => cx.bind_keys([KeyBinding::new(&key, Commit, None)]),
            "jump" => cx.bind_keys([KeyBinding::new(&key, Jump, None)]),
            "toggle_laser" => cx.bind_keys([KeyBinding::new(&key, Laser, None)]),
            "toggle_pen" => cx.bind_keys([KeyBinding::new(&key, Pen, None)]),
            _ => {
                let label = match action {
                    "toggle_laser" => locale.text("feature-laser"),
                    "toggle_pen" => locale.text("feature-pen"),
                    _ => action,
                };
                warnings.push(locale.format(
                    "unwired-shortcut",
                    &[("action", label.into()), ("key", key)],
                ));
            }
        }
    }
    cx.bind_keys([KeyBinding::new("escape", Escape, None)]);
    cx.bind_keys([
        KeyBinding::new("enter", ActivateButton, Some("FerroStageButton")),
        KeyBinding::new("space", ActivateButton, Some("FerroStageButton")),
    ]);
    // The focused number editor consumes editing keys before configurable actions.
    for key in [
        "0",
        "1",
        "2",
        "3",
        "4",
        "5",
        "6",
        "7",
        "8",
        "9",
        "backspace",
        "delete",
        "left",
        "right",
        "home",
        "end",
        "shift-left",
        "shift-right",
        "shift-home",
        "shift-end",
        "ctrl-a",
        "cmd-a",
        "ctrl-v",
        "cmd-v",
        "shift-insert",
        "ctrl-c",
        "cmd-c",
        "ctrl-x",
        "cmd-x",
        "space",
        "enter",
        "escape",
    ] {
        cx.bind_keys([KeyBinding::new(
            key,
            gpui::NoAction,
            Some("FerroStagePageInput"),
        )]);
    }
    for key in ["left", "right", "up", "down", "home", "end"] {
        cx.bind_keys([KeyBinding::new(
            key,
            gpui::NoAction,
            Some("FerroStageSettingsSlider"),
        )]);
    }
    warnings
}
