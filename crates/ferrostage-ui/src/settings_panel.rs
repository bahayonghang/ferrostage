//! Presenter-only preferences model. The schema parser remains the range authority.
use crate::display_settings::DisplayDraft;
use ferrostage_settings::{Keybindings, Settings};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Category {
    #[default]
    General,
    Displays,
    Presenter,
    Timer,
    Pointer,
    Keys,
    Advanced,
}
impl Category {
    pub const ALL: [Self; 7] = [
        Self::General,
        Self::Displays,
        Self::Presenter,
        Self::Timer,
        Self::Pointer,
        Self::Keys,
        Self::Advanced,
    ];
    pub fn id(self) -> &'static str {
        match self {
            Self::General => "general",
            Self::Displays => "displays",
            Self::Presenter => "presenter",
            Self::Timer => "timer",
            Self::Pointer => "pointer",
            Self::Keys => "keys",
            Self::Advanced => "advanced",
        }
    }
}
#[derive(Clone, Copy)]
pub enum Kind {
    Toggle,
    FontFamily,
    Choice(&'static [&'static str]),
    Number {
        min: f64,
        max: f64,
        step: f64,
        integer: bool,
    },
    Color,
}
#[derive(Clone, Copy)]
pub struct Field {
    pub id: &'static str,
    pub category: Category,
    pub kind: Kind,
}
macro_rules! field {
    ($id:literal, $category:ident, $kind:expr) => {
        Field {
            id: $id,
            category: Category::$category,
            kind: $kind,
        }
    };
}
use Kind::*;
pub const FIELDS: &[Field] = &[
    field!("hot_reload.enabled", Advanced, Toggle),
    field!(
        "hot_reload.debounce_ms",
        Advanced,
        Number {
            min: 1.,
            max: 10000.,
            step: 50.,
            integer: true
        }
    ),
    field!("general.language", General, Choice(&["zh-CN", "en-US"])),
    field!(
        "general.theme",
        General,
        Choice(&["dark", "light", "system"])
    ),
    field!("general.ui_font_family", General, FontFamily),
    field!(
        "general.ui_font_size",
        General,
        Number {
            min: *ferrostage_settings::UI_FONT_SIZE_RANGE.start() as f64,
            max: *ferrostage_settings::UI_FONT_SIZE_RANGE.end() as f64,
            step: 1.,
            integer: true,
        }
    ),
    field!("general.reopen_last_document", General, Toggle),
    field!(
        "general.recent_files_limit",
        General,
        Number {
            min: 1.,
            max: 100.,
            step: 1.,
            integer: true
        }
    ),
    field!("display.audience_fullscreen", Displays, Toggle),
    field!("display.audience_click_to_advance", Displays, Toggle),
    field!("display.presenter_start_fullscreen", Displays, Toggle),
    field!(
        "presenter.notes_column_ratio",
        Presenter,
        Number {
            min: 0.40,
            max: 0.75,
            step: 0.01,
            integer: false
        }
    ),
    field!(
        "presenter.notes_font_scale",
        Presenter,
        Number {
            min: 0.1,
            max: 8.,
            step: 0.1,
            integer: false
        }
    ),
    field!("presenter.show_clock", Presenter, Toggle),
    field!(
        "presenter.grid_columns",
        Presenter,
        Number {
            min: 1.,
            max: 4.,
            step: 1.,
            integer: true
        }
    ),
    field!(
        "presenter.grid_visible_pages",
        Presenter,
        Number {
            min: 1.,
            max: 16.,
            step: 1.,
            integer: true
        }
    ),
    field!("timer.mode", Timer, Choice(&["count_up", "count_down"])),
    field!(
        "timer.target_minutes",
        Timer,
        Number {
            min: 1.,
            max: 240.,
            step: 1.,
            integer: true
        }
    ),
    field!(
        "timer.warn_yellow_remaining_min",
        Timer,
        Number {
            min: 1.,
            max: 120.,
            step: 1.,
            integer: true
        }
    ),
    field!(
        "timer.warn_red_remaining_min",
        Timer,
        Number {
            min: 0.,
            max: 120.,
            step: 1.,
            integer: true
        }
    ),
    field!("pointer.laser_color_bgra", Pointer, Color),
    field!(
        "pointer.laser_radius_norm",
        Pointer,
        Number {
            min: 0.001,
            max: 0.25,
            step: 0.001,
            integer: false
        }
    ),
    field!("pointer.pen_color_bgra", Pointer, Color),
    field!(
        "pointer.pen_width_norm",
        Pointer,
        Number {
            min: 0.001,
            max: 0.25,
            step: 0.001,
            integer: false
        }
    ),
    field!(
        "pointer.highlighter_alpha",
        Pointer,
        Number {
            min: 0.,
            max: 1.,
            step: 0.05,
            integer: false
        }
    ),
    field!(
        "cache.l1_gpu_mib",
        Advanced,
        Number {
            min: 1.,
            max: 2048.,
            step: 16.,
            integer: true
        }
    ),
    field!(
        "cache.l2_cpu_mib",
        Advanced,
        Number {
            min: 1.,
            max: 4096.,
            step: 32.,
            integer: true
        }
    ),
    field!(
        "cache.l3_thumb_mib",
        Advanced,
        Number {
            min: 1.,
            max: 1024.,
            step: 16.,
            integer: true
        }
    ),
    field!("general.debug_overlay", Advanced, Toggle),
    field!(
        "media.max_decode_height",
        Advanced,
        Number {
            min: 1.,
            max: 2160.,
            step: 120.,
            integer: true
        }
    ),
];
#[derive(Clone, Debug, Default)]
pub struct PanelState {
    pub category: Category,
    pub edit: Option<(&'static str, String)>,
    pub recording: Option<&'static str>,
    pub display: Option<DisplayDraft>,
    pub error: Option<String>,
    pub confirm_reset: bool,
    pub font_picker: Option<FontPicker>,
}
#[derive(Clone, Debug, Default)]
pub struct FontPicker {
    pub filter: String,
    pub selected: usize,
}
impl FontPicker {
    // None represents the system-default sentinel, always available in the list.
    pub fn matches<'a>(&self, families: &'a [String]) -> Vec<Option<&'a str>> {
        let filter = self.filter.to_lowercase();
        std::iter::once(None)
            .chain(
                families
                    .iter()
                    .filter(|family| family.to_lowercase().contains(&filter))
                    .map(|family| Some(family.as_str())),
            )
            .collect()
    }
}

fn table(settings: &Settings) -> Result<toml::Table, String> {
    toml::Table::try_from(settings).map_err(|error| error.to_string())
}
fn locate(id: &str) -> Result<(&str, &str), String> {
    id.split_once('.').ok_or_else(|| "Unknown setting".into())
}
pub fn value(settings: &Settings, field: Field) -> String {
    let Ok(table) = table(settings) else {
        return String::new();
    };
    let Ok((section, key)) = locate(field.id) else {
        return String::new();
    };
    let value = &table[section][key];
    if matches!(field.kind, Color)
        && let Some(channels) = value.as_array()
    {
        return format!(
            "{:02X}{:02X}{:02X}{:02X}",
            channels[2].as_integer().unwrap_or(0),
            channels[1].as_integer().unwrap_or(0),
            channels[0].as_integer().unwrap_or(0),
            channels[3].as_integer().unwrap_or(0)
        );
    }
    match value {
        toml::Value::String(text) => text.clone(),
        _ => value.to_string(),
    }
}
/// Invalid input is rejected, never silently replaced by schema defaults.
pub fn edit(settings: &Settings, id: &str, input: &str) -> Result<Settings, String> {
    let field = FIELDS
        .iter()
        .find(|field| field.id == id)
        .ok_or("Unknown setting")?;
    let mut table = table(settings)?;
    if matches!(field.kind, FontFamily) && input.chars().any(char::is_control) {
        return Err("Font family cannot contain control characters".into());
    }
    let input = input.trim();
    let value = match field.kind {
        Toggle => toml::Value::Boolean(input.parse().map_err(|_| "Expected true or false")?),
        Choice(_) | FontFamily => toml::Value::String(input.into()),
        Number { integer: true, .. } => {
            toml::Value::Integer(input.parse().map_err(|_| "Expected an integer")?)
        }
        Number { integer: false, .. } => {
            toml::Value::Float(input.parse().map_err(|_| "Expected a number")?)
        }
        Color => {
            let raw = input.trim_start_matches('#');
            if raw.len() != 8 || !raw.is_ascii() {
                return Err("Expected RRGGBBAA hexadecimal color".into());
            }
            let rgba = (0..4)
                .map(|i| u8::from_str_radix(&raw[i * 2..i * 2 + 2], 16))
                .collect::<Result<Vec<_>, _>>()
                .map_err(|_| "Expected RRGGBBAA hexadecimal color")?;
            toml::Value::Array(
                [rgba[2], rgba[1], rgba[0], rgba[3]]
                    .into_iter()
                    .map(|v| toml::Value::Integer(i64::from(v)))
                    .collect(),
            )
        }
    };
    let (section, key) = locate(id)?;
    table[section]
        .as_table_mut()
        .ok_or("Invalid section")?
        .insert(key.into(), value);
    let outcome =
        ferrostage_settings::from_toml(&toml::to_string(&table).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    // Existing shortcut diagnostics are allowed; invalid values and coupled
    // timer bounds are rejected instead of adopting the loader's fallback.
    if let Some(warning) = outcome
        .warnings
        .iter()
        .find(|w| w.key == id || (section == "timer" && w.key.starts_with("timer.")))
    {
        return Err(warning.reason.clone());
    }
    let mut next = outcome.settings;
    next.display.monitors = settings.display.monitors.clone();
    Ok(next)
}
/// Display real history context without inventing metadata or changing order.
pub fn recent_copy(path: &str) -> (String, String) {
    let file = std::path::Path::new(path);
    let name = file
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.into());
    let parent = file
        .parent()
        .map(|parent| parent.to_string_lossy().into_owned())
        .unwrap_or_default();
    (name, parent)
}

pub fn bindings_mut<'a>(keys: &'a mut Keybindings, action: &str) -> Option<&'a mut Vec<String>> {
    Some(match action {
        "next" => &mut keys.next,
        "previous" => &mut keys.previous,
        "start_presentation" => &mut keys.start_presentation,
        "stop_presentation" => &mut keys.stop_presentation,
        "blank_black" => &mut keys.blank_black,
        "blank_white" => &mut keys.blank_white,
        "swap_screens" => &mut keys.swap_screens,
        "toggle_laser" => &mut keys.toggle_laser,
        "toggle_pen" => &mut keys.toggle_pen,
        "jump" => &mut keys.jump,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn field_edits_reject_fallbacks_and_keep_unrelated_values() {
        let mut settings = Settings::default();
        settings.keybindings.next = vec!["F6".into()];
        for (id, value) in [
            ("presenter.grid_columns", "5"),
            ("presenter.notes_column_ratio", "NaN"),
            ("timer.warn_yellow_remaining_min", "0"),
            ("pointer.laser_color_bgra", "bad"),
            ("cache.l1_gpu_mib", "0"),
            ("general.theme", "rainbow"),
            ("general.ui_font_size", "25"),
            ("general.ui_font_size", "16.5"),
            ("general.ui_font_family", "bad\nfamily"),
            ("display.audience_click_to_advance", "invalid"),
        ] {
            assert!(edit(&settings, id, value).is_err(), "{id}");
        }
        let next = edit(&settings, "pointer.laser_color_bgra", "112233AA").unwrap();
        assert_eq!(next.pointer.laser_color_bgra, [0x33, 0x22, 0x11, 0xAA]);
        assert_eq!(next.keybindings, settings.keybindings);
        let next = edit(&next, "display.audience_click_to_advance", "false").unwrap();
        assert!(!next.display.audience_click_to_advance);
        let next = edit(&next, "presenter.grid_visible_pages", "13").unwrap();
        assert_eq!(next.presenter.grid_visible_pages, 13);
        assert_eq!(next.presenter.grid_columns, settings.presenter.grid_columns);
        for field in FIELDS {
            assert!(
                matches!(field.kind, FontFamily) || !value(&next, *field).is_empty(),
                "{}",
                field.id
            );
        }
    }
    #[test]
    fn font_filter_and_history_keep_real_values() {
        let picker = FontPicker {
            filter: "yahei".into(),
            selected: 0,
        };
        let families = vec!["Arial".into(), "Microsoft YaHei UI".into()];
        assert_eq!(
            picker.matches(&families),
            vec![None, Some("Microsoft YaHei UI")]
        );
        let (a, parent_a) = recent_copy("alpha/same.pdf");
        let (b, parent_b) = recent_copy("beta/same.pdf");
        assert_eq!(a, b);
        assert_ne!(parent_a, parent_b);
    }
    #[test]
    fn recent_limit_trims_and_coupled_timer_validation_is_atomic() {
        let mut settings = Settings::default();
        settings.general.recent_files = vec!["a.pdf".into(), "b.pdf".into(), "c.pdf".into()];
        settings.general.recent_opened_at = std::collections::BTreeMap::from([
            ("a.pdf".into(), 10),
            ("b.pdf".into(), 20),
            ("c.pdf".into(), 30),
        ]);
        let next = edit(&settings, "general.recent_files_limit", "1").unwrap();
        assert_eq!(next.general.recent_files, vec!["a.pdf"]);
        assert_eq!(
            next.general.recent_opened_at,
            std::collections::BTreeMap::from([("a.pdf".into(), 10)])
        );
        assert!(
            edit(
                &settings,
                "timer.warn_red_remaining_min",
                &settings.timer.warn_yellow_remaining_min.to_string()
            )
            .is_err()
        );
        let next = edit(&settings, "timer.target_minutes", "42").unwrap();
        assert_eq!(next.timer.target_minutes, 42);
    }
}
