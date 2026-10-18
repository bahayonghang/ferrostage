//! General, monitor and presenter section parsing.

use super::value::{
    font_scale_ok, parse_bool, parse_choice, parse_f64, parse_u32_inclusive, ratio_ok,
};
use super::{SettingsWarning, warn};
use crate::schema::{
    DisplaySettings, GRID_COLUMNS_RANGE, GRID_VISIBLE_PAGES_RANGE, GeneralSettings, LANGUAGES,
    MonitorBinding, PresenterSettings, RECENT_FILES_LIMIT_RANGE, SingleMonitorMode, Theme,
    role_from_name,
};
use ferrostage_core::MonitorRole;
use std::collections::BTreeMap;
use toml::Value;

pub(super) fn parse_general(
    section: Option<&toml::Table>,
    warnings: &mut Vec<SettingsWarning>,
) -> GeneralSettings {
    let defaults = GeneralSettings::default();
    let Some(section) = section else {
        return defaults;
    };
    let recent_files_limit = parse_u32_inclusive(
        section,
        "recent_files_limit",
        "general.recent_files_limit",
        defaults.recent_files_limit,
        *RECENT_FILES_LIMIT_RANGE.start(),
        *RECENT_FILES_LIMIT_RANGE.end(),
        warnings,
    );
    GeneralSettings {
        language: parse_language(section, &defaults.language, warnings),
        theme: parse_theme(section, defaults.theme, warnings),
        reopen_last_document: parse_bool(
            section,
            "reopen_last_document",
            "general.reopen_last_document",
            defaults.reopen_last_document,
            warnings,
        ),
        recent_files_limit,
        recent_files: parse_recent_files(section, recent_files_limit, warnings),
        debug_overlay: parse_bool(
            section,
            "debug_overlay",
            "general.debug_overlay",
            defaults.debug_overlay,
            warnings,
        ),
    }
}

fn parse_language(
    section: &toml::Table,
    default: &str,
    warnings: &mut Vec<SettingsWarning>,
) -> String {
    match section.get("language") {
        None => default.to_owned(),
        Some(Value::String(value)) if LANGUAGES.contains(&value.as_str()) => value.clone(),
        Some(Value::String(_)) => {
            warn(
                warnings,
                "general.language",
                "general.language must be zh-CN or en-US; using the default",
            );
            default.to_owned()
        }
        Some(other) => {
            warn(
                warnings,
                "general.language",
                format!("expected a string, found {}", other.type_str()),
            );
            default.to_owned()
        }
    }
}

fn parse_theme(
    section: &toml::Table,
    default: Theme,
    warnings: &mut Vec<SettingsWarning>,
) -> Theme {
    parse_choice(
        section,
        "theme",
        "general.theme",
        &[
            (Theme::Dark.as_str(), Theme::Dark),
            (Theme::Light.as_str(), Theme::Light),
            (Theme::System.as_str(), Theme::System),
        ],
        default,
        warnings,
    )
}

pub(super) fn parse_display(
    section: Option<&toml::Table>,
    warnings: &mut Vec<SettingsWarning>,
) -> DisplaySettings {
    let defaults = DisplaySettings::default();
    let Some(section) = section else {
        return defaults;
    };
    DisplaySettings {
        single_monitor_mode: parse_choice(
            section,
            "single_monitor_mode",
            "display.single_monitor_mode",
            &[
                (
                    SingleMonitorMode::PresenterOnly.as_str(),
                    SingleMonitorMode::PresenterOnly,
                ),
                (
                    SingleMonitorMode::WindowedAudience.as_str(),
                    SingleMonitorMode::WindowedAudience,
                ),
            ],
            defaults.single_monitor_mode,
            warnings,
        ),
        audience_fullscreen: parse_bool(
            section,
            "audience_fullscreen",
            "display.audience_fullscreen",
            defaults.audience_fullscreen,
            warnings,
        ),
        presenter_start_fullscreen: parse_bool(
            section,
            "presenter_start_fullscreen",
            "display.presenter_start_fullscreen",
            defaults.presenter_start_fullscreen,
            warnings,
        ),
        monitors: parse_monitors(section.get("monitors"), warnings),
    }
}

fn parse_monitors(
    value: Option<&Value>,
    warnings: &mut Vec<SettingsWarning>,
) -> Vec<MonitorBinding> {
    let Some(value) = value else {
        return Vec::new();
    };
    let Some(items) = value.as_array() else {
        warn(
            warnings,
            "display.monitors",
            format!("expected an array, found {}", value.type_str()),
        );
        return Vec::new();
    };
    let mut occurrences = BTreeMap::<String, usize>::new();
    let mut bindings = Vec::new();
    for (index, item) in items.iter().enumerate() {
        let Some(table) = item.as_table() else {
            warn(
                warnings,
                &format!("display.monitors[{index}]"),
                "expected a table",
            );
            continue;
        };
        let Some(id) = table.get("id").and_then(Value::as_str) else {
            warn(
                warnings,
                &format!("display.monitors[{index}].id"),
                "expected a string id",
            );
            continue;
        };
        let occurrence = occurrences.entry(id.to_owned()).or_insert(0);
        let source_occurrence = Some(*occurrence);
        *occurrence += 1;
        let friendly_name = match table.get("friendly_name") {
            None => String::new(),
            Some(Value::String(name)) => name.clone(),
            Some(other) => {
                warn(
                    warnings,
                    &format!("display.monitors[{index}].friendly_name"),
                    format!("expected a string, found {}", other.type_str()),
                );
                String::new()
            }
        };
        let role = parse_monitor_role(table.get("role"), index, warnings);
        bindings.push(MonitorBinding {
            id: id.to_owned(),
            friendly_name,
            role,
            source_occurrence,
        });
    }
    bindings
}

fn parse_monitor_role(
    value: Option<&Value>,
    index: usize,
    warnings: &mut Vec<SettingsWarning>,
) -> MonitorRole {
    let key = format!("display.monitors[{index}].role");
    match value {
        None => {
            warn(warnings, &key, format!("{key} is missing; using none"));
            MonitorRole::None
        }
        Some(Value::String(name)) => role_from_name(name).unwrap_or_else(|| {
            warn(
                warnings,
                &key,
                format!("{key} is not presenter, audience, or none; using none"),
            );
            MonitorRole::None
        }),
        Some(other) => {
            warn(
                warnings,
                &key,
                format!("expected a string, found {}; using none", other.type_str()),
            );
            MonitorRole::None
        }
    }
}

pub(super) fn parse_presenter(
    section: Option<&toml::Table>,
    warnings: &mut Vec<SettingsWarning>,
) -> PresenterSettings {
    let defaults = PresenterSettings::default();
    let Some(section) = section else {
        return defaults;
    };
    PresenterSettings {
        next_slide_ratio: parse_f64(
            section,
            "next_slide_ratio",
            "presenter.next_slide_ratio",
            defaults.next_slide_ratio,
            ratio_ok,
            warnings,
        ),
        notes_ratio: parse_f64(
            section,
            "notes_ratio",
            "presenter.notes_ratio",
            defaults.notes_ratio,
            ratio_ok,
            warnings,
        ),
        notes_font_scale: parse_f64(
            section,
            "notes_font_scale",
            "presenter.notes_font_scale",
            defaults.notes_font_scale,
            font_scale_ok,
            warnings,
        ),
        show_clock: parse_bool(
            section,
            "show_clock",
            "presenter.show_clock",
            defaults.show_clock,
            warnings,
        ),
        show_thumbnail_strip: parse_bool(
            section,
            "show_thumbnail_strip",
            "presenter.show_thumbnail_strip",
            defaults.show_thumbnail_strip,
            warnings,
        ),
        grid_columns: parse_u32_inclusive(
            section,
            "grid_columns",
            "presenter.grid_columns",
            defaults.grid_columns,
            *GRID_COLUMNS_RANGE.start(),
            *GRID_COLUMNS_RANGE.end(),
            warnings,
        ),
        grid_visible_pages: parse_u32_inclusive(
            section,
            "grid_visible_pages",
            "presenter.grid_visible_pages",
            defaults.grid_visible_pages,
            *GRID_VISIBLE_PAGES_RANGE.start(),
            *GRID_VISIBLE_PAGES_RANGE.end(),
            warnings,
        ),
    }
}

fn parse_recent_files(
    section: &toml::Table,
    limit: u32,
    warnings: &mut Vec<SettingsWarning>,
) -> Vec<String> {
    let Some(value) = section.get("recent_files") else {
        return Vec::new();
    };
    let Some(items) = value.as_array() else {
        warn(
            warnings,
            "general.recent_files",
            format!(
                "general.recent_files expected an array, found {}",
                value.type_str()
            ),
        );
        return Vec::new();
    };
    let mut files = Vec::new();
    for (index, item) in items.iter().enumerate() {
        match item.as_str() {
            Some(path) => {
                if !files.iter().any(|existing| existing == path) {
                    files.push(path.to_owned());
                }
            }
            None => warn(
                warnings,
                &format!("general.recent_files[{index}]"),
                format!("general.recent_files[{index}] expected a string"),
            ),
        }
    }
    let limit = usize::try_from(limit).unwrap_or(0);
    files.truncate(limit);
    files
}
