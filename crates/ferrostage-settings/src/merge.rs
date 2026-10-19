//! Merge known leaves into the original TOML tree.
//!
//! Unknown top-level keys, unknown keys inside known sections, and unknown
//! fields on monitor elements stay. Monitor elements match on `id` plus the
//! stable duplicate index recorded at load. A new monitor gets a fresh table.

use toml::Value;

use crate::error::SaveError;
use crate::schema::{
    CacheSettings, DisplaySettings, GeneralSettings, HotReloadSettings, Keybindings,
    MonitorBinding, PointerSettings, PresenterSettings, SCHEMA_VERSION, Settings, TimerSettings,
    role_name,
};

pub(crate) fn merge_document(
    raw: &toml::Table,
    settings: &Settings,
    consumed_legacy_keys: &[String],
) -> Result<toml::Table, SaveError> {
    let mut doc = raw.clone();
    for key in consumed_legacy_keys {
        doc.remove(key);
    }
    doc.insert("schema_version".to_owned(), Value::Integer(SCHEMA_VERSION));
    merge_general(&mut doc, &settings.general)?;
    merge_display(&mut doc, &settings.display)?;
    merge_presenter(&mut doc, &settings.presenter)?;
    merge_timer(&mut doc, &settings.timer)?;
    merge_pointer(&mut doc, &settings.pointer)?;
    merge_keybindings(&mut doc, &settings.keybindings)?;
    merge_cache(&mut doc, &settings.cache)?;
    merge_hot_reload(&mut doc, &settings.hot_reload)?;
    with_section(&mut doc, "media", |table| {
        table.insert(
            "max_decode_height".into(),
            u32_value(settings.media.max_decode_height),
        );
    });
    Ok(doc)
}

fn merge_general(doc: &mut toml::Table, general: &GeneralSettings) -> Result<(), SaveError> {
    let language = string_value(&general.language);
    let theme = string_value(general.theme.as_str());
    let reopen = bool_value(general.reopen_last_document);
    let limit = u32_value(general.recent_files_limit);
    let recent = strings_value(&general.recent_files);
    let debug = bool_value(general.debug_overlay);
    with_section(doc, "general", |table| {
        table.insert("language".to_owned(), language);
        table.insert("theme".to_owned(), theme);
        table.insert(
            "ui_font_family".into(),
            string_value(&general.ui_font_family),
        );
        table.insert("ui_font_size".into(), u32_value(general.ui_font_size));
        table.insert("reopen_last_document".to_owned(), reopen);
        table.insert("recent_files_limit".to_owned(), limit);
        table.insert("recent_files".to_owned(), recent);
        // The path map is one known leaf, so cleared/pruned paths cannot return
        // through the raw document's unknown-data preservation.
        table.insert(
            "recent_opened_at".to_owned(),
            Value::Table(
                general
                    .recent_opened_at
                    .iter()
                    .map(|(path, seconds)| (path.clone(), Value::Integer(*seconds)))
                    .collect(),
            ),
        );
        table.insert("debug_overlay".to_owned(), debug);
    });
    Ok(())
}

fn merge_display(doc: &mut toml::Table, display: &DisplaySettings) -> Result<(), SaveError> {
    let monitors = {
        let existing = doc.get("display").and_then(|value| value.get("monitors"));
        merge_monitors(existing, &display.monitors)
    };
    let mode = string_value(display.single_monitor_mode.as_str());
    let audience = bool_value(display.audience_fullscreen);
    let click_to_advance = bool_value(display.audience_click_to_advance);
    let presenter = bool_value(display.presenter_start_fullscreen);
    with_section(doc, "display", |table| {
        table.insert("single_monitor_mode".to_owned(), mode);
        table.insert("audience_fullscreen".to_owned(), audience);
        table.insert("audience_click_to_advance".to_owned(), click_to_advance);
        table.insert("presenter_start_fullscreen".to_owned(), presenter);
        table.insert("monitors".to_owned(), monitors);
    });
    Ok(())
}

fn merge_presenter(doc: &mut toml::Table, presenter: &PresenterSettings) -> Result<(), SaveError> {
    let next_slide = float_value(presenter.next_slide_ratio);
    let notes = float_value(presenter.notes_ratio);
    let notes_column = float_value(presenter.notes_column_ratio);
    let font = float_value(presenter.notes_font_scale);
    let clock = bool_value(presenter.show_clock);
    let strip = bool_value(presenter.show_thumbnail_strip);
    let columns = u32_value(presenter.grid_columns);
    let pages = u32_value(presenter.grid_visible_pages);
    with_section(doc, "presenter", |table| {
        table.insert("next_slide_ratio".to_owned(), next_slide);
        table.insert("notes_ratio".to_owned(), notes);
        table.insert("notes_column_ratio".to_owned(), notes_column);
        table.insert("notes_font_scale".to_owned(), font);
        table.insert("show_clock".to_owned(), clock);
        table.insert("show_thumbnail_strip".to_owned(), strip);
        table.insert("grid_columns".to_owned(), columns);
        table.insert("grid_visible_pages".to_owned(), pages);
    });
    Ok(())
}

fn merge_timer(doc: &mut toml::Table, timer: &TimerSettings) -> Result<(), SaveError> {
    let mode = string_value(timer.mode.as_str());
    let target = u32_value(timer.target_minutes);
    let yellow = u32_value(timer.warn_yellow_remaining_min);
    let red = u32_value(timer.warn_red_remaining_min);
    with_section(doc, "timer", |table| {
        table.insert("mode".to_owned(), mode);
        table.insert("target_minutes".to_owned(), target);
        table.insert("warn_yellow_remaining_min".to_owned(), yellow);
        table.insert("warn_red_remaining_min".to_owned(), red);
    });
    Ok(())
}

fn merge_pointer(doc: &mut toml::Table, pointer: &PointerSettings) -> Result<(), SaveError> {
    let laser = color_value(pointer.laser_color_bgra);
    let laser_radius = float_value(pointer.laser_radius_norm);
    let pen = color_value(pointer.pen_color_bgra);
    let pen_width = float_value(pointer.pen_width_norm);
    let alpha = float_value(pointer.highlighter_alpha);
    with_section(doc, "pointer", |table| {
        table.insert("laser_color_bgra".to_owned(), laser);
        table.insert("laser_radius_norm".to_owned(), laser_radius);
        table.insert("pen_color_bgra".to_owned(), pen);
        table.insert("pen_width_norm".to_owned(), pen_width);
        table.insert("highlighter_alpha".to_owned(), alpha);
    });
    Ok(())
}

fn merge_keybindings(doc: &mut toml::Table, bindings: &Keybindings) -> Result<(), SaveError> {
    let pairs = [
        ("next", strings_value(&bindings.next)),
        ("previous", strings_value(&bindings.previous)),
        (
            "start_presentation",
            strings_value(&bindings.start_presentation),
        ),
        (
            "stop_presentation",
            strings_value(&bindings.stop_presentation),
        ),
        ("blank_black", strings_value(&bindings.blank_black)),
        ("blank_white", strings_value(&bindings.blank_white)),
        ("swap_screens", strings_value(&bindings.swap_screens)),
        ("toggle_laser", strings_value(&bindings.toggle_laser)),
        ("toggle_pen", strings_value(&bindings.toggle_pen)),
        ("jump", strings_value(&bindings.jump)),
    ];
    with_section(doc, "keybindings", |table| {
        for (key, value) in pairs {
            table.insert(key.to_owned(), value);
        }
    });
    Ok(())
}

fn merge_cache(doc: &mut toml::Table, cache: &CacheSettings) -> Result<(), SaveError> {
    let l1 = u32_value(cache.l1_gpu_mib);
    let l2 = u32_value(cache.l2_cpu_mib);
    let l3 = u32_value(cache.l3_thumb_mib);
    with_section(doc, "cache", |table| {
        table.insert("l1_gpu_mib".to_owned(), l1);
        table.insert("l2_cpu_mib".to_owned(), l2);
        table.insert("l3_thumb_mib".to_owned(), l3);
    });
    Ok(())
}

fn merge_hot_reload(
    doc: &mut toml::Table,
    hot_reload: &HotReloadSettings,
) -> Result<(), SaveError> {
    let enabled = bool_value(hot_reload.enabled);
    let debounce = u64_value(hot_reload.debounce_ms)?;
    with_section(doc, "hot_reload", |table| {
        table.insert("enabled".to_owned(), enabled);
        table.insert("debounce_ms".to_owned(), debounce);
    });
    Ok(())
}

fn with_section(doc: &mut toml::Table, name: &str, fill: impl FnOnce(&mut toml::Table)) {
    if !matches!(doc.get(name), Some(Value::Table(_))) {
        doc.insert(name.to_owned(), Value::Table(toml::Table::new()));
    }
    if let Some(Value::Table(table)) = doc.get_mut(name) {
        fill(table);
    }
}

fn merge_monitors(existing: Option<&Value>, bindings: &[MonitorBinding]) -> Value {
    let raw_items = existing
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut used = vec![false; raw_items.len()];
    let mut output = Vec::with_capacity(bindings.len());
    for binding in bindings {
        let matched = binding
            .source_occurrence
            .and_then(|occurrence| nth_id_index(&raw_items, &binding.id, occurrence, &used));
        if let Some(index) = matched {
            used[index] = true;
            let mut table = raw_items[index]
                .as_table()
                .cloned()
                .unwrap_or_else(toml::Table::new);
            write_binding(&mut table, binding);
            output.push(Value::Table(table));
        } else {
            let mut table = toml::Table::new();
            write_binding(&mut table, binding);
            output.push(Value::Table(table));
        }
    }
    for (index, item) in raw_items.into_iter().enumerate() {
        if !used[index] && monitor_id(&item).is_none() {
            output.push(item);
        }
    }
    Value::Array(output)
}

fn nth_id_index(items: &[Value], id: &str, occurrence: usize, used: &[bool]) -> Option<usize> {
    let mut seen = 0usize;
    for (index, item) in items.iter().enumerate() {
        if monitor_id(item) == Some(id) {
            if seen == occurrence {
                if used[index] {
                    return None;
                }
                return Some(index);
            }
            seen += 1;
        }
    }
    None
}

fn monitor_id(value: &Value) -> Option<&str> {
    value
        .as_table()
        .and_then(|table| table.get("id"))
        .and_then(Value::as_str)
}

fn write_binding(table: &mut toml::Table, binding: &MonitorBinding) {
    table.insert("id".to_owned(), string_value(&binding.id));
    table.insert(
        "friendly_name".to_owned(),
        string_value(&binding.friendly_name),
    );
    table.insert("role".to_owned(), string_value(role_name(binding.role)));
}

fn string_value(value: &str) -> Value {
    Value::String(value.to_owned())
}

fn bool_value(value: bool) -> Value {
    Value::Boolean(value)
}

fn u32_value(value: u32) -> Value {
    Value::Integer(i64::from(value))
}

fn u64_value(value: u64) -> Result<Value, SaveError> {
    i64::try_from(value)
        .map(Value::Integer)
        .map_err(|_| SaveError::Write {
            reason: "hot_reload.debounce_ms does not fit in a TOML integer".to_owned(),
        })
}

fn float_value(value: f64) -> Value {
    Value::Float(value)
}

fn strings_value(values: &[String]) -> Value {
    Value::Array(values.iter().cloned().map(Value::String).collect())
}

fn color_value(color: [u8; 4]) -> Value {
    Value::Array(
        color
            .into_iter()
            .map(|channel| Value::Integer(i64::from(channel)))
            .collect(),
    )
}
