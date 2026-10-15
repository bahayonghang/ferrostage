//! Cache/reload settings and migration from the original flat keys.

use super::value::{parse_bool, parse_positive_u32, parse_positive_u64};
use super::{SettingsWarning, warn};
use crate::schema::{CacheSettings, HotReloadSettings};
use toml::Value;

pub(super) fn parse_cache(
    root: &toml::Table,
    section: Option<&toml::Table>,
    legacy: bool,
    warnings: &mut Vec<SettingsWarning>,
    consumed: &mut Vec<String>,
) -> CacheSettings {
    let defaults = CacheSettings::default();
    let mut cache = if let Some(section) = section {
        CacheSettings {
            l1_gpu_mib: parse_positive_u32(
                section,
                "l1_gpu_mib",
                "cache.l1_gpu_mib",
                defaults.l1_gpu_mib,
                warnings,
            ),
            l2_cpu_mib: parse_positive_u32(
                section,
                "l2_cpu_mib",
                "cache.l2_cpu_mib",
                defaults.l2_cpu_mib,
                warnings,
            ),
            l3_thumb_mib: parse_positive_u32(
                section,
                "l3_thumb_mib",
                "cache.l3_thumb_mib",
                defaults.l3_thumb_mib,
                warnings,
            ),
        }
    } else {
        defaults
    };
    if legacy {
        cache.l1_gpu_mib = apply_legacy_u32(
            root,
            "cache_l1_mib",
            section,
            "l1_gpu_mib",
            cache.l1_gpu_mib,
            warnings,
            consumed,
        );
        cache.l2_cpu_mib = apply_legacy_u32(
            root,
            "cache_l2_mib",
            section,
            "l2_cpu_mib",
            cache.l2_cpu_mib,
            warnings,
            consumed,
        );
        cache.l3_thumb_mib = apply_legacy_u32(
            root,
            "cache_l3_mib",
            section,
            "l3_thumb_mib",
            cache.l3_thumb_mib,
            warnings,
            consumed,
        );
    }
    cache
}

pub(super) fn parse_hot_reload(
    root: &toml::Table,
    section: Option<&toml::Table>,
    legacy: bool,
    warnings: &mut Vec<SettingsWarning>,
    consumed: &mut Vec<String>,
) -> HotReloadSettings {
    let defaults = HotReloadSettings::default();
    let mut hot_reload = if let Some(section) = section {
        HotReloadSettings {
            enabled: parse_bool(
                section,
                "enabled",
                "hot_reload.enabled",
                defaults.enabled,
                warnings,
            ),
            debounce_ms: parse_positive_u64(
                section,
                "debounce_ms",
                "hot_reload.debounce_ms",
                defaults.debounce_ms,
                warnings,
            ),
        }
    } else {
        defaults
    };
    if legacy {
        hot_reload.debounce_ms = apply_legacy_u64(
            root,
            "hot_reload_debounce_ms",
            section,
            "debounce_ms",
            hot_reload.debounce_ms,
            warnings,
            consumed,
        );
    }
    hot_reload
}

fn apply_legacy_u32(
    root: &toml::Table,
    legacy_key: &str,
    section: Option<&toml::Table>,
    new_key: &str,
    current: u32,
    warnings: &mut Vec<SettingsWarning>,
    consumed: &mut Vec<String>,
) -> u32 {
    if section.is_some_and(|table| table.contains_key(new_key)) {
        return current;
    }
    match root.get(legacy_key) {
        None => current,
        Some(Value::Integer(value)) => match u32::try_from(*value) {
            Ok(number) if number > 0 => {
                consumed.push(legacy_key.to_owned());
                number
            }
            _ => {
                warn(
                    warnings,
                    legacy_key,
                    format!("{legacy_key} must be greater than 0; using {current}"),
                );
                current
            }
        },
        Some(other) => {
            warn(
                warnings,
                legacy_key,
                format!("expected an integer, found {}", other.type_str()),
            );
            current
        }
    }
}

fn apply_legacy_u64(
    root: &toml::Table,
    legacy_key: &str,
    section: Option<&toml::Table>,
    new_key: &str,
    current: u64,
    warnings: &mut Vec<SettingsWarning>,
    consumed: &mut Vec<String>,
) -> u64 {
    if section.is_some_and(|table| table.contains_key(new_key)) {
        return current;
    }
    match root.get(legacy_key) {
        None => current,
        Some(Value::Integer(value)) => match u64::try_from(*value) {
            Ok(number) if number > 0 => {
                consumed.push(legacy_key.to_owned());
                number
            }
            _ => {
                warn(
                    warnings,
                    legacy_key,
                    format!("{legacy_key} must be greater than 0; using {current}"),
                );
                current
            }
        },
        Some(other) => {
            warn(
                warnings,
                legacy_key,
                format!("expected an integer, found {}", other.type_str()),
            );
            current
        }
    }
}
