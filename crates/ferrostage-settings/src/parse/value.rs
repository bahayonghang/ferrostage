//! Typed leaf validation shared by section parsers.

use super::{SettingsWarning, warn};
use toml::Value;

pub(super) fn parse_bool(
    section: &toml::Table,
    key: &str,
    path: &str,
    default: bool,
    warnings: &mut Vec<SettingsWarning>,
) -> bool {
    match section.get(key) {
        None => default,
        Some(Value::Boolean(value)) => *value,
        Some(other) => {
            warn(
                warnings,
                path,
                format!("{path} expected a boolean, found {}", other.type_str()),
            );
            default
        }
    }
}

pub(super) fn parse_choice<T: Copy>(
    section: &toml::Table,
    key: &str,
    path: &str,
    choices: &[(&str, T)],
    default: T,
    warnings: &mut Vec<SettingsWarning>,
) -> T {
    match section.get(key) {
        None => default,
        Some(Value::String(value)) => choices
            .iter()
            .find(|(name, _)| *name == value)
            .map(|(_, parsed)| *parsed)
            .unwrap_or_else(|| {
                warn(
                    warnings,
                    path,
                    format!("{path} is not a recognized value; using the default"),
                );
                default
            }),
        Some(other) => {
            warn(
                warnings,
                path,
                format!("{path} expected a string, found {}", other.type_str()),
            );
            default
        }
    }
}

pub(super) fn parse_u32_inclusive(
    section: &toml::Table,
    key: &str,
    path: &str,
    default: u32,
    min: u32,
    max: u32,
    warnings: &mut Vec<SettingsWarning>,
) -> u32 {
    match section.get(key) {
        None => default,
        Some(Value::Integer(value)) => match u32::try_from(*value) {
            Ok(number) if (min..=max).contains(&number) => number,
            _ => {
                warn(
                    warnings,
                    path,
                    format!("{path} must be from {min} to {max}; using {default}"),
                );
                default
            }
        },
        Some(other) => {
            warn(
                warnings,
                path,
                format!("{path} expected an integer, found {}", other.type_str()),
            );
            default
        }
    }
}

pub(super) fn parse_u32_at_least(
    section: &toml::Table,
    key: &str,
    path: &str,
    default: u32,
    warnings: &mut Vec<SettingsWarning>,
) -> u32 {
    match section.get(key) {
        None => default,
        Some(Value::Integer(value)) => match u32::try_from(*value) {
            Ok(number) => number,
            Err(_) => {
                warn(
                    warnings,
                    path,
                    format!("{path} must be greater than or equal to 0; using {default}"),
                );
                default
            }
        },
        Some(other) => {
            warn(
                warnings,
                path,
                format!("{path} expected an integer, found {}", other.type_str()),
            );
            default
        }
    }
}

pub(super) fn parse_positive_u32(
    section: &toml::Table,
    key: &str,
    path: &str,
    default: u32,
    warnings: &mut Vec<SettingsWarning>,
) -> u32 {
    match section.get(key) {
        None => default,
        Some(Value::Integer(value)) => match u32::try_from(*value) {
            Ok(number) if number > 0 => number,
            _ => {
                warn(
                    warnings,
                    path,
                    format!("{path} must be greater than 0; using {default}"),
                );
                default
            }
        },
        Some(other) => {
            warn(
                warnings,
                path,
                format!("{path} expected an integer, found {}", other.type_str()),
            );
            default
        }
    }
}

pub(super) fn parse_positive_u64(
    section: &toml::Table,
    key: &str,
    path: &str,
    default: u64,
    warnings: &mut Vec<SettingsWarning>,
) -> u64 {
    match section.get(key) {
        None => default,
        Some(Value::Integer(value)) => match u64::try_from(*value) {
            Ok(number) if number > 0 => number,
            _ => {
                warn(
                    warnings,
                    path,
                    format!("{path} must be greater than 0; using {default}"),
                );
                default
            }
        },
        Some(other) => {
            warn(
                warnings,
                path,
                format!("{path} expected an integer, found {}", other.type_str()),
            );
            default
        }
    }
}

pub(super) fn parse_f64(
    section: &toml::Table,
    key: &str,
    path: &str,
    default: f64,
    accept: fn(f64) -> bool,
    warnings: &mut Vec<SettingsWarning>,
) -> f64 {
    match section.get(key) {
        None => default,
        Some(value) => match finite_number(value) {
            Some(number) if accept(number) => number,
            Some(_) => {
                warn(
                    warnings,
                    path,
                    format!("{path} must be finite and within range; using the default"),
                );
                default
            }
            None => {
                warn(
                    warnings,
                    path,
                    format!(
                        "{path} expected a finite number, found {}",
                        value.type_str()
                    ),
                );
                default
            }
        },
    }
}

fn finite_number(value: &Value) -> Option<f64> {
    match value {
        Value::Float(number) if number.is_finite() => Some(*number),
        Value::Integer(number) => i64_to_f64(*number),
        _ => None,
    }
}

fn i64_to_f64(value: i64) -> Option<f64> {
    const EXACT_LIMIT: i64 = 1 << 53;
    if (-EXACT_LIMIT..EXACT_LIMIT).contains(&value) {
        #[allow(clippy::cast_precision_loss)]
        let number = value as f64;
        Some(number)
    } else {
        None
    }
}

pub(super) fn ratio_ok(value: f64) -> bool {
    value > 0.0 && value <= 1.0
}

pub(super) fn font_scale_ok(value: f64) -> bool {
    value > 0.0 && value <= 8.0
}

pub(super) fn width_ok(value: f64) -> bool {
    value > 0.0 && value <= 0.25
}

pub(super) fn alpha_ok(value: f64) -> bool {
    (0.0..=1.0).contains(&value)
}

pub(super) fn parse_color(
    section: &toml::Table,
    key: &str,
    path: &str,
    default: [u8; 4],
    warnings: &mut Vec<SettingsWarning>,
) -> [u8; 4] {
    let Some(value) = section.get(key) else {
        return default;
    };
    let Some(items) = value.as_array() else {
        warn(
            warnings,
            path,
            format!(
                "{path} expected an array of 4 integers, found {}",
                value.type_str()
            ),
        );
        return default;
    };
    if items.len() != 4 {
        warn(
            warnings,
            path,
            format!("{path} must contain 4 color bytes; using the default"),
        );
        return default;
    }
    let mut color = [0u8; 4];
    for (index, item) in items.iter().enumerate() {
        let Some(channel) = item.as_integer() else {
            warn(
                warnings,
                path,
                format!("{path}[{index}] expected an integer from 0 to 255"),
            );
            return default;
        };
        let Ok(channel) = u8::try_from(channel) else {
            warn(
                warnings,
                path,
                format!("{path}[{index}] expected an integer from 0 to 255"),
            );
            return default;
        };
        color[index] = channel;
    }
    color
}
