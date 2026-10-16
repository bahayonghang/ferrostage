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
    Media(u64),
    SettingsCategory(crate::settings_panel::Category),
    EditSetting(&'static str, String),
    BeginSettingEdit(&'static str),
    SettingsKey { key: String, text: Option<String> },
    RecordBinding(&'static str),
    RemoveBinding(&'static str, usize),
    ResetBindings,
    ClearRecent,
    OpenConfigDirectory,
    ResetPreferences(bool),
    SurfaceSize(crate::workbench::Surface, ferrostage_pdf::PixelSize),
    Core(ferrostage_core::PresentationCommand),
    Tool(crate::workbench::Tool),
    Grid,
    GridMove(isize),
    Preview(ferrostage_core::PhysicalPage),
    Document,
    Reconfigure(ferrostage_document::DocumentOverrides),
    Split { next: f64, notes: f64, save: bool },
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
    RetrySave,
    Displays,
    Identify,
    RefreshDisplays,
    SelectPresenter(ferrostage_core::MonitorId),
    SelectPdf(crate::display_settings::PdfTarget),
    ApplyDisplays,
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
    cx.intercept_keystrokes(move |_, window, cx| {
        if let Some(shell) = editing.upgrade() {
            let shell = shell.read(cx);
            if shell.captures_preferences(window.window_handle().window_id()) {
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
    register!(Start, Start);
    register!(Stop, Stop);
    register!(Black, Black);
    register!(White, White);
    register!(Swap, Swap);
    register!(Open, Open);
    register!(Settings, Settings);
    register!(Jump, Jump);
    register!(Escape, Escape);
    register!(Commit, Commit);
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
        // Escape is routed once, with modal/preview consumption before stop.
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
    // Leave these events to the existing Jump editor instead of presentation
    // bindings. GPUI NoAction suppresses matching global bindings in context.
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
    ] {
        cx.bind_keys([KeyBinding::new(key, gpui::NoAction, Some("FerroStageJump"))]);
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
