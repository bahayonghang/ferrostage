//! Pure shell decisions. Native placement and focus must still be observed.
use std::collections::{BTreeMap, BTreeSet};

use crate::locale::Locale;
use ferrostage_core::{MonitorId, MonitorRole};
use ferrostage_platform::{MonitorInfo, ResolvedTopology, resolve_roles};
use ferrostage_settings::{
    DisplaySettings, Keybindings, MonitorBinding, SingleMonitorMode, StartMode,
};
use thiserror::Error;
use uuid::Uuid;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum ShellError {
    #[error("ambiguous display identities")]
    AmbiguousDisplays,
    #[error("no audience display")]
    NoAudience,
    #[error("two distinct displays are required")]
    CannotSwap,
    #[error("window placement confirmation timed out")]
    PlacementTimeout,
    #[error("presenter activation timed out")]
    ActivationTimeout,
}

impl ShellError {
    pub fn localized(&self, locale: Locale) -> String {
        locale
            .text(match self {
                Self::AmbiguousDisplays => "ambiguous-displays",
                Self::NoAudience => "no-audience",
                Self::CannotSwap => "cannot-swap",
                Self::PlacementTimeout => "placement-timeout",
                Self::ActivationTimeout => "activation-timeout",
            })
            .to_string()
    }
}

/// Exactly the pinned Windows algorithm, including its full zero-filled buffer.
pub fn display_uuid(connection: &str) -> Result<Uuid, ShellError> {
    let units: Vec<_> = connection.encode_utf16().collect();
    if units.is_empty() || units.len() >= 32 || units.contains(&0) {
        return Err(ShellError::AmbiguousDisplays);
    }
    let mut bytes = [0_u8; 64];
    for (index, unit) in units.into_iter().enumerate() {
        bytes[index * 2..index * 2 + 2].copy_from_slice(&unit.to_be_bytes());
    }
    Ok(Uuid::new_v5(&Uuid::NAMESPACE_DNS, &bytes))
}

pub fn map_displays(
    monitors: &[MonitorInfo],
    displays: &[(u64, Uuid)],
) -> Result<BTreeMap<MonitorId, u64>, ShellError> {
    let mut ids = BTreeSet::new();
    let mut names = BTreeSet::new();
    let mut uuids = BTreeSet::new();
    let mut native = BTreeMap::new();
    if monitors.is_empty() || monitors.len() != displays.len() {
        return Err(ShellError::AmbiguousDisplays);
    }
    for (id, uuid) in displays {
        if !ids.insert(*id) || native.insert(*uuid, *id).is_some() {
            return Err(ShellError::AmbiguousDisplays);
        }
    }
    let mut result = BTreeMap::new();
    for monitor in monitors {
        let uuid = display_uuid(&monitor.connection_name)?;
        let id = native.get(&uuid).ok_or(ShellError::AmbiguousDisplays)?;
        if !names.insert(&monitor.connection_name)
            || !uuids.insert(uuid)
            || result.insert(monitor.id.clone(), *id).is_some()
        {
            return Err(ShellError::AmbiguousDisplays);
        }
    }
    Ok(result)
}

pub fn roles(monitors: &[MonitorInfo], settings: &DisplaySettings) -> ResolvedTopology {
    resolve_roles(
        monitors,
        &settings
            .monitors
            .iter()
            .map(|binding| (MonitorId(binding.id.clone()), binding.role))
            .collect::<Vec<_>>(),
    )
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AudienceTarget {
    pub monitor: MonitorId,
    pub fullscreen: bool,
    /// The audience shares the presenter monitor (one monitor, or `StartMode::Single`).
    pub shared: bool,
}

pub fn audience_target(
    monitors: &[MonitorInfo],
    settings: &DisplaySettings,
) -> Result<AudienceTarget, ShellError> {
    let resolved = roles(monitors, settings);
    if monitors.len() == 1 {
        return resolved
            .presenter
            .map(|monitor| AudienceTarget {
                monitor,
                fullscreen: settings.single_monitor_mode == SingleMonitorMode::FullscreenAudience,
                shared: true,
            })
            .ok_or(ShellError::NoAudience);
    }
    resolved
        .audience
        .map(|monitor| AudienceTarget {
            monitor,
            fullscreen: settings.audience_fullscreen,
            shared: false,
        })
        .ok_or(ShellError::NoAudience)
}

/// Start placement. `Single` plays fullscreen on the presenter monitor at any
/// monitor count; `Dual` follows the display roles.
pub fn start_target(
    monitors: &[MonitorInfo],
    settings: &DisplaySettings,
) -> Result<AudienceTarget, ShellError> {
    if settings.start_mode == StartMode::Single {
        return roles(monitors, settings)
            .presenter
            .map(|monitor| AudienceTarget {
                monitor,
                fullscreen: true,
                shared: true,
            })
            .ok_or(ShellError::NoAudience);
    }
    audience_target(monitors, settings)
}

/// Prepare settings only; the caller commits them after all replacement windows
/// are confirmed. Failed window creation cannot mutate this authority.
pub fn swapped(
    monitors: &[MonitorInfo],
    settings: &DisplaySettings,
) -> Result<DisplaySettings, ShellError> {
    let resolved = roles(monitors, settings);
    let (Some(presenter), Some(audience)) = (resolved.presenter, resolved.audience) else {
        return Err(ShellError::CannotSwap);
    };
    if presenter == audience || monitors.len() < 2 {
        return Err(ShellError::CannotSwap);
    }
    let mut next = settings.clone();
    for (id, role) in [
        (presenter, MonitorRole::Audience),
        (audience, MonitorRole::Presenter),
    ] {
        if let Some(binding) = next.monitors.iter_mut().find(|binding| binding.id == id.0) {
            binding.role = role;
        } else {
            next.monitors.push(MonitorBinding {
                friendly_name: monitors
                    .iter()
                    .find(|monitor| monitor.id == id)
                    .map(|monitor| monitor.name.clone())
                    .unwrap_or_default(),
                id: id.0,
                role,
                source_occurrence: None,
            });
        }
    }
    Ok(next)
}

/// Human-readable settings use uppercase letters for unshifted shortcuts.
pub fn canonical_key(input: &str) -> Option<String> {
    let input = input.trim().to_ascii_lowercase().replace('+', "-");
    let mut pieces: Vec<_> = input.split('-').collect();
    let key = *pieces.last()?;
    let named = [
        "pagedown",
        "pageup",
        "space",
        "right",
        "left",
        "up",
        "down",
        "backspace",
        "escape",
        "enter",
        "tab",
        "home",
        "end",
        "delete",
    ];
    let function = key
        .strip_prefix('f')
        .and_then(|value| value.parse::<u8>().ok())
        .is_some_and(|value| (1..=24).contains(&value));
    if !(named.contains(&key)
        || function
        || (key.len() == 1 && key.as_bytes()[0].is_ascii_alphanumeric()))
    {
        return None;
    }
    pieces.pop();
    let mut modifiers = BTreeSet::new();
    for modifier in pieces {
        let modifier = match modifier {
            "control" => "ctrl",
            "super" | "win" => "cmd",
            value => value,
        };
        if !["ctrl", "alt", "shift", "cmd", "fn"].contains(&modifier) || !modifiers.insert(modifier)
        {
            return None;
        }
    }
    let mut output: Vec<_> = modifiers.into_iter().collect();
    output.push(key);
    Some(output.join("-"))
}

pub fn bindings(
    settings: &Keybindings,
    locale: Locale,
) -> (Vec<(String, &'static str)>, Vec<String>) {
    let mut candidates: BTreeMap<String, BTreeSet<&'static str>> = BTreeMap::new();
    let mut warnings = Vec::new();
    for (key, action) in [
        ("ctrl-o", "open"),
        ("ctrl-,", "settings"),
        ("enter", "commit"),
    ] {
        candidates
            .entry(key.to_string())
            .or_default()
            .insert(action);
    }
    for (action, keys) in settings.actions() {
        for input in keys {
            if let Some(key) = canonical_key(input) {
                candidates.entry(key).or_default().insert(action);
            } else {
                warnings.push(locale.format(
                    "invalid-shortcut",
                    &[("key", format!("{input:?}")), ("action", action.into())],
                ));
            }
        }
    }
    let mut valid = Vec::new();
    for (key, actions) in candidates {
        if actions.len() != 1 {
            warnings.push(locale.format(
                "conflicting-shortcut",
                &[("key", key.clone()), ("actions", format!("{actions:?}"))],
            ));
        } else if let Some(action) = actions.first() {
            if key == "escape" && *action != "stop_presentation" {
                warnings.push(locale.format("reserved-escape", &[("action", (*action).into())]));
                continue;
            }
            valid.push((key, *action));
        }
    }
    (valid, warnings)
}

/// Reserve room for native borders/titlebar around the client area. Work-area
/// bounds already exclude the taskbar/dock; the inset also avoids edge clipping.
const WINDOW_MARGIN: f32 = 32.;

pub fn windowed_bounds(visible: [f32; 4], desired: [f32; 2]) -> [f32; 4] {
    let width = desired[0].min((visible[2] - 2. * WINDOW_MARGIN).max(1.));
    let height = desired[1].min((visible[3] - 2. * WINDOW_MARGIN).max(1.));
    [
        visible[0] + (visible[2] - width) / 2.,
        visible[1] + (visible[3] - height) / 2.,
        width,
        height,
    ]
}

/// Fullscreen requires an observed native bounds event and matching full display
/// bounds. Windowed callers provide the usable display area and must additionally
/// check the requested client size after any cross-DPI creation has settled.
pub fn placement_confirmed(
    requested: u64,
    actual: Option<u64>,
    fullscreen: bool,
    actual_fullscreen: bool,
    observed: bool,
    bounds: [f32; 4],
    display: [f32; 4],
) -> bool {
    actual == Some(requested)
        && fullscreen == actual_fullscreen
        && bounds.into_iter().chain(display).all(f32::is_finite)
        && bounds[2] > 0.0
        && bounds[3] > 0.0
        && display[2] > 0.0
        && display[3] > 0.0
        && if fullscreen {
            observed
                && bounds
                    .into_iter()
                    .zip(display)
                    .all(|(a, b)| (a - b).abs() <= 2.0)
        } else {
            bounds[0] >= display[0] + WINDOW_MARGIN - 2.
                && bounds[1] >= display[1] + WINDOW_MARGIN - 2.
                && bounds[0] + bounds[2] <= display[0] + display[2] - WINDOW_MARGIN + 2.
                && bounds[1] + bounds[3] <= display[1] + display[3] - WINDOW_MARGIN + 2.
        }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferrostage_platform::{IdentityConfidence, IdentitySource, ScaleSource};
    fn monitor(id: &str, name: &str) -> MonitorInfo {
        MonitorInfo {
            id: MonitorId(id.into()),
            connection_name: name.into(),
            name: name.into(),
            native_size: (1920, 1080),
            position: (0, 0),
            scale_factor: 1.0,
            scale_source: ScaleSource::GdiDeviceCapsRatio,
            rotation_degrees: 0,
            is_primary: id == "a",
            identity_source: IdentitySource::DeviceInterfacePath,
            identity_confidence: IdentityConfidence::ConnectionScoped,
            built_in: false,
        }
    }
    #[test]
    fn uuid_includes_terminator_and_full_utf16_buffer() {
        let mut bytes = [0u8; 64];
        bytes[1] = b'A';
        assert_eq!(
            display_uuid("A").unwrap(),
            Uuid::new_v5(&Uuid::NAMESPACE_DNS, &bytes)
        );
        assert_ne!(
            display_uuid("A").unwrap(),
            Uuid::new_v5(&Uuid::NAMESPACE_DNS, b"A")
        );
        for name in [
            "".to_string(),
            "A\0B".into(),
            "a".repeat(32),
            "😀".repeat(16),
        ] {
            assert!(display_uuid(&name).is_err());
        }
    }
    #[test]
    fn mapping_rejects_partial_duplicate_and_unknown_sets() {
        let a = monitor("a", "A");
        let b = monitor("b", "B");
        let displays = [
            (3, display_uuid("A").unwrap()),
            (7, display_uuid("B").unwrap()),
        ];
        assert_eq!(
            map_displays(&[b.clone(), a.clone()], &displays).unwrap()[&a.id],
            3
        );
        assert!(map_displays(std::slice::from_ref(&a), &displays).is_err());
        assert!(map_displays(&[a.clone(), a.clone()], &displays).is_err());
        assert!(
            map_displays(
                &[a.clone(), b.clone()],
                &[(3, displays[0].1), (3, displays[1].1)]
            )
            .is_err()
        );
        assert!(
            map_displays(
                &[a, b],
                &[(3, displays[0].1), (7, display_uuid("C").unwrap())]
            )
            .is_err()
        );
    }
    #[test]
    fn conflicts_are_disabled_after_normalization() {
        let keys = Keybindings {
            next: vec!["b".into(), "ctrl++b".into()],
            ..Default::default()
        };
        let (valid, warnings) = bindings(&keys, Locale::new("en-US"));
        assert!(!valid.iter().any(|(key, _)| key == "b"));
        assert_eq!(warnings.len(), 2);
        assert_eq!(canonical_key("Ctrl+Shift+B"), Some("ctrl-shift-b".into()));
        assert_eq!(canonical_key("F25"), None);
    }
    #[test]
    fn configured_bindings_cannot_collide_with_shell_shortcuts() {
        let keys = Keybindings {
            next: vec!["Ctrl+O".into(), "Enter".into()],
            ..Default::default()
        };
        let (valid, warnings) = bindings(&keys, Locale::new("en-US"));
        assert!(
            !valid
                .iter()
                .any(|(key, _)| key == "ctrl-o" || key == "enter")
        );
        assert_eq!(warnings.len(), 2);
    }
    #[test]
    fn swap_prepares_authority_without_mutating_old_settings() {
        let monitors = [monitor("a", "A"), monitor("b", "B")];
        let old = DisplaySettings::default();
        let candidate = swapped(&monitors, &old).unwrap();
        assert!(old.monitors.is_empty());
        assert_eq!(
            roles(&monitors, &candidate).audience,
            Some(MonitorId("a".into()))
        );
        assert!(swapped(&monitors[..1], &old).is_err());
    }
    #[test]
    fn single_monitor_defaults_to_shared_fullscreen_and_keeps_windowed_mode() {
        let target = |fullscreen, shared| AudienceTarget {
            monitor: MonitorId("a".into()),
            fullscreen,
            shared,
        };
        let monitors = [monitor("a", "A")];
        let mut settings = DisplaySettings::default();
        assert_eq!(
            audience_target(&monitors, &settings).unwrap(),
            target(true, true)
        );
        settings.single_monitor_mode = SingleMonitorMode::WindowedAudience;
        assert_eq!(
            audience_target(&monitors, &settings).unwrap(),
            target(false, true)
        );
        // Two monitors ignore the single-monitor mode.
        let two = [monitor("a", "A"), monitor("b", "B")];
        for mode in [
            SingleMonitorMode::FullscreenAudience,
            SingleMonitorMode::WindowedAudience,
        ] {
            settings.single_monitor_mode = mode;
            settings.monitors = vec![];
            let resolved = audience_target(&two, &settings).unwrap();
            assert!(!resolved.shared && resolved.fullscreen);
            settings.monitors = ["a", "b"]
                .map(|id| MonitorBinding {
                    id: id.into(),
                    friendly_name: id.into(),
                    role: MonitorRole::None,
                    source_occurrence: None,
                })
                .into();
            assert_eq!(
                audience_target(&two, &settings),
                Err(ShellError::NoAudience)
            );
        }
    }
    #[test]
    fn single_start_mode_plays_fullscreen_on_the_presenter_at_any_monitor_count() {
        let mut settings = DisplaySettings {
            start_mode: StartMode::Single,
            ..DisplaySettings::default()
        };
        let presenter = AudienceTarget {
            monitor: MonitorId("a".into()),
            fullscreen: true,
            shared: true,
        };
        let two = [monitor("a", "A"), monitor("b", "B")];
        for monitors in [&two[..1], &two[..]] {
            assert_eq!(start_target(monitors, &settings).unwrap(), presenter);
        }
        settings.single_monitor_mode = SingleMonitorMode::WindowedAudience;
        assert_eq!(start_target(&two[..1], &settings).unwrap(), presenter);
        settings.start_mode = StartMode::Dual;
        assert_eq!(
            start_target(&two, &settings),
            audience_target(&two, &settings)
        );
        assert!(!start_target(&two, &settings).unwrap().shared);
        assert!(start_target(&[], &settings).is_err());
    }
    #[test]
    fn placement_v10_portrait_native_overflow_is_not_confirmation() {
        // V9 actual native read-back on C at 150%, created from B at 125%.
        assert!(!placement_confirmed(
            65735,
            Some(65735),
            false,
            false,
            true,
            [1708., -273.33334, 1280., 912.],
            [1706.6666, -761.3333, 1066.6666, 1706.6666],
        ));
    }
    #[test]
    fn placement_v10_work_area_and_narrow_clients_leave_native_frame_space() {
        let portrait = [1706.6666, -761.3333, 1066.6666, 1706.6666];
        let client = windowed_bounds(portrait, [1100., 760.]);
        assert!((client[2] - 1002.6666).abs() < 0.01);
        assert!(placement_confirmed(
            1,
            Some(1),
            false,
            false,
            true,
            client,
            portrait
        ));
        let work = [-1706.6666, -112.6667, 1706.6666, 1020.];
        for wanted in [[800., 600.], [390., 844.], [320., 640.]] {
            let client = windowed_bounds(work, wanted);
            assert_eq!([client[2], client[3]], wanted);
            assert!(placement_confirmed(
                1,
                Some(1),
                false,
                false,
                true,
                client,
                work
            ));
        }
        // A client reaching the taskbar or placing its titlebar off the top must
        // not pass even though its center still belongs to the requested screen.
        for bad in [[100., 100., 700., 900.], [100., 0., 700., 700.]] {
            assert!(!placement_confirmed(
                1,
                Some(1),
                false,
                false,
                true,
                bad,
                [0., 0., 1000., 1000.]
            ));
        }
    }
    #[test]
    fn fullscreen_request_without_native_bounds_evidence_is_not_confirmation() {
        let display = [0., 0., 1920., 1080.];
        assert!(!placement_confirmed(
            1,
            Some(1),
            true,
            true,
            false,
            display,
            display
        ));
        assert!(!placement_confirmed(
            1,
            Some(2),
            true,
            true,
            true,
            display,
            display
        ));
        assert!(!placement_confirmed(
            1,
            Some(1),
            true,
            true,
            true,
            [0., 0., 800., 600.],
            display
        ));
        assert!(placement_confirmed(
            1,
            Some(1),
            true,
            true,
            true,
            display,
            display
        ));
    }
}
