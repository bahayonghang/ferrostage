//! Timer, pointer and keybinding section parsing.

use super::value::{alpha_ok, parse_choice, parse_color, parse_f64, parse_u32_at_least, width_ok};
use super::{SettingsWarning, warn};
use crate::schema::{BindingIssue, Keybindings, PointerSettings, TimerMode, TimerSettings};
use toml::Value;

pub(super) fn parse_timer(
    section: Option<&toml::Table>,
    warnings: &mut Vec<SettingsWarning>,
) -> TimerSettings {
    let defaults = TimerSettings::default();
    let Some(section) = section else {
        return defaults;
    };
    let mode = parse_choice(
        section,
        "mode",
        "timer.mode",
        &[
            (TimerMode::CountUp.as_str(), TimerMode::CountUp),
            (TimerMode::CountDown.as_str(), TimerMode::CountDown),
        ],
        defaults.mode,
        warnings,
    );
    let target_minutes = parse_target_minutes(section, defaults.target_minutes, warnings);
    let mut red = parse_u32_at_least(
        section,
        "warn_red_remaining_min",
        "timer.warn_red_remaining_min",
        defaults.warn_red_remaining_min,
        warnings,
    );
    let mut yellow = parse_u32_at_least(
        section,
        "warn_yellow_remaining_min",
        "timer.warn_yellow_remaining_min",
        defaults.warn_yellow_remaining_min,
        warnings,
    );
    if yellow <= red {
        warn(
            warnings,
            "timer.warn_yellow_remaining_min",
            "timer.warn_yellow_remaining_min must be greater than timer.warn_red_remaining_min; using defaults",
        );
        warn(
            warnings,
            "timer.warn_red_remaining_min",
            "timer.warn_red_remaining_min must be less than timer.warn_yellow_remaining_min; using defaults",
        );
        yellow = defaults.warn_yellow_remaining_min;
        red = defaults.warn_red_remaining_min;
    }
    TimerSettings {
        mode,
        target_minutes,
        warn_yellow_remaining_min: yellow,
        warn_red_remaining_min: red,
    }
}

fn parse_target_minutes(
    section: &toml::Table,
    default: u32,
    warnings: &mut Vec<SettingsWarning>,
) -> u32 {
    const KEY: &str = "timer.target_minutes";
    match section.get("target_minutes") {
        None => default,
        Some(Value::Integer(value)) => match u32::try_from(*value) {
            Ok(minutes) if minutes > 0 => minutes,
            _ => {
                warn(
                    warnings,
                    KEY,
                    format!("{KEY} must be greater than 0; using {default}"),
                );
                default
            }
        },
        Some(other) => {
            warn(
                warnings,
                KEY,
                format!("{KEY} expected an integer, found {}", other.type_str()),
            );
            default
        }
    }
}

pub(super) fn parse_pointer(
    section: Option<&toml::Table>,
    warnings: &mut Vec<SettingsWarning>,
) -> PointerSettings {
    let defaults = PointerSettings::default();
    let Some(section) = section else {
        return defaults;
    };
    PointerSettings {
        laser_color_bgra: parse_color(
            section,
            "laser_color_bgra",
            "pointer.laser_color_bgra",
            defaults.laser_color_bgra,
            warnings,
        ),
        laser_radius_norm: parse_f64(
            section,
            "laser_radius_norm",
            "pointer.laser_radius_norm",
            defaults.laser_radius_norm,
            width_ok,
            warnings,
        ),
        pen_color_bgra: parse_color(
            section,
            "pen_color_bgra",
            "pointer.pen_color_bgra",
            defaults.pen_color_bgra,
            warnings,
        ),
        pen_width_norm: parse_f64(
            section,
            "pen_width_norm",
            "pointer.pen_width_norm",
            defaults.pen_width_norm,
            width_ok,
            warnings,
        ),
        highlighter_alpha: parse_f64(
            section,
            "highlighter_alpha",
            "pointer.highlighter_alpha",
            defaults.highlighter_alpha,
            alpha_ok,
            warnings,
        ),
    }
}

pub(super) fn parse_keybindings(
    section: Option<&toml::Table>,
    warnings: &mut Vec<SettingsWarning>,
) -> Keybindings {
    let defaults = Keybindings::default();
    let Some(section) = section else {
        return defaults;
    };
    let bindings = Keybindings {
        next: parse_binding_list(section, "next", defaults.next, warnings),
        previous: parse_binding_list(section, "previous", defaults.previous, warnings),
        start_presentation: parse_binding_list(
            section,
            "start_presentation",
            defaults.start_presentation,
            warnings,
        ),
        stop_presentation: parse_binding_list(
            section,
            "stop_presentation",
            defaults.stop_presentation,
            warnings,
        ),
        blank_black: parse_binding_list(section, "blank_black", defaults.blank_black, warnings),
        blank_white: parse_binding_list(section, "blank_white", defaults.blank_white, warnings),
        swap_screens: parse_binding_list(section, "swap_screens", defaults.swap_screens, warnings),
        toggle_laser: parse_binding_list(section, "toggle_laser", defaults.toggle_laser, warnings),
        toggle_pen: parse_binding_list(section, "toggle_pen", defaults.toggle_pen, warnings),
        jump: parse_binding_list(section, "jump", defaults.jump, warnings),
    };
    warn_binding_issues(&bindings, warnings);
    bindings
}

fn parse_binding_list(
    section: &toml::Table,
    action: &str,
    default: Vec<String>,
    warnings: &mut Vec<SettingsWarning>,
) -> Vec<String> {
    let path = format!("keybindings.{action}");
    match section.get(action) {
        None => default,
        Some(Value::Array(items)) => {
            let mut bindings = Vec::new();
            for (index, item) in items.iter().enumerate() {
                match item.as_str() {
                    Some(binding) => bindings.push(binding.to_owned()),
                    None => {
                        warn(
                            warnings,
                            &path,
                            format!(
                                "{path}[{index}] expected a string; using the default bindings"
                            ),
                        );
                        return default;
                    }
                }
            }
            bindings
        }
        Some(other) => {
            warn(
                warnings,
                &path,
                format!("expected an array of strings, found {}", other.type_str()),
            );
            default
        }
    }
}

fn warn_binding_issues(bindings: &Keybindings, warnings: &mut Vec<SettingsWarning>) {
    for issue in bindings.issues() {
        match issue {
            BindingIssue::Empty { action } => {
                let key = format!("keybindings.{action}");
                warn(warnings, &key, format!("{key} contains an empty binding"));
            }
            BindingIssue::Duplicate { action, binding } => {
                let key = format!("keybindings.{action}");
                warn(warnings, &key, format!("{key} repeats \"{binding}\""));
            }
            BindingIssue::Conflict { binding, actions } => {
                let key = format!("keybindings.{}", actions[1]);
                warn(
                    warnings,
                    &key,
                    format!(
                        "binding \"{binding}\" is on both keybindings.{} and keybindings.{}; it must not run both actions",
                        actions[0], actions[1]
                    ),
                );
            }
        }
    }
}
