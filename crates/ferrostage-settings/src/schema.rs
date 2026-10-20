//! Typed settings schema, version 1.
//!
//! Defaults match `docs/PROJECT_PLAN.md` section 4.4, plus `debug_overlay`,
//! `recent_files`, and the two presenter grid keys. The media section defaults compatibly for existing v1 files.

pub use crate::keybindings::{BindingIssue, Keybindings};
use ferrostage_core::MonitorRole;
use ferrostage_render::{L1_GPU_BUDGET_MIB, L2_CPU_BUDGET_MIB, L3_THUMB_BUDGET_MIB};
use serde::Serialize;
use serde::Serializer;
use std::collections::{BTreeMap, BTreeSet};

/// Writable schema produced by this crate.
pub const SCHEMA_VERSION: i64 = 1;

/// `general.language` values that have a locale file under `assets/locales`.
pub const LANGUAGES: [&str; 2] = ["zh-CN", "en-US"];

/// Inclusive logical-pixel range for interface base size.
pub const UI_FONT_SIZE_RANGE: std::ops::RangeInclusive<u32> = 12..=24;

/// Inclusive range for `general.recent_files_limit`.
pub const RECENT_FILES_LIMIT_RANGE: std::ops::RangeInclusive<u32> = 1..=100;

/// Inclusive range for `presenter.grid_columns`.
pub const GRID_COLUMNS_RANGE: std::ops::RangeInclusive<u32> = 1..=4;

/// Inclusive range for `presenter.grid_visible_pages`.
pub const GRID_VISIBLE_PAGES_RANGE: std::ops::RangeInclusive<u32> = 1..=16;

/// Color theme. `system` follows the OS theme once the UI task applies it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Theme {
    /// Dark palette.
    Dark,
    /// Light palette.
    Light,
    /// Follow the operating system.
    System,
}

impl Theme {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Dark => "dark",
            Self::Light => "light",
            Self::System => "system",
        }
    }
}

/// What to do when only one monitor is attached.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SingleMonitorMode {
    /// Fullscreen audience window on that monitor, above the presenter window.
    FullscreenAudience,
    /// Allow a windowed audience view on the same monitor.
    WindowedAudience,
}

impl SingleMonitorMode {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::FullscreenAudience => "fullscreen_audience",
            Self::WindowedAudience => "windowed_audience",
        }
    }
}

/// How Start places the playback window.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StartMode {
    /// Presenter and audience follow the display roles.
    #[default]
    Dual,
    /// Fullscreen playback on the presenter monitor, at any monitor count.
    Single,
}

impl StartMode {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Dual => "dual",
            Self::Single => "single",
        }
    }
}

/// Countdown or elapsed timer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TimerMode {
    /// Elapsed time since the presentation started.
    CountUp,
    /// Remaining time against [`TimerSettings::target_minutes`].
    CountDown,
}

impl TimerMode {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::CountUp => "count_up",
            Self::CountDown => "count_down",
        }
    }
}

/// One saved monitor binding.
///
/// `source_occurrence` is not part of the file. It records which duplicate `id`
/// this binding was loaded from so a later save updates that element only.
/// A binding created in memory uses `None` and does not inherit another
/// monitor's unknown fields. Equality ignores the occurrence.
#[derive(Clone, Debug, Serialize)]
pub struct MonitorBinding {
    /// Stable monitor identity. Task 03 turns this into `MonitorId`.
    pub id: String,
    /// Label shown in the settings UI.
    pub friendly_name: String,
    /// Role stored as `presenter`, `audience`, or `none`.
    #[serde(serialize_with = "serialize_role")]
    pub role: MonitorRole,
    /// Load-time index of `id` among duplicates. Not serialized.
    #[serde(skip_serializing)]
    pub source_occurrence: Option<usize>,
}

impl PartialEq for MonitorBinding {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.friendly_name == other.friendly_name && self.role == other.role
    }
}

impl Eq for MonitorBinding {}

fn serialize_role<S>(role: &MonitorRole, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(role_name(*role))
}

pub(crate) const fn role_name(role: MonitorRole) -> &'static str {
    match role {
        MonitorRole::Presenter => "presenter",
        MonitorRole::Audience => "audience",
        MonitorRole::None => "none",
    }
}

pub(crate) fn role_from_name(name: &str) -> Option<MonitorRole> {
    match name {
        "presenter" => Some(MonitorRole::Presenter),
        "audience" => Some(MonitorRole::Audience),
        "none" => Some(MonitorRole::None),
        _ => None,
    }
}

/// `general` section.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct GeneralSettings {
    /// `zh-CN` or `en-US`.
    pub language: String,
    /// Palette selection.
    pub theme: Theme,
    /// Installed interface family; empty selects the system default.
    pub ui_font_family: String,
    /// Interface base size in logical pixels; content fonts are unaffected.
    pub ui_font_size: u32,
    /// Open the previous document on the next launch.
    pub reopen_last_document: bool,
    /// Maximum length of [`Self::recent_files`] after dedup.
    pub recent_files_limit: u32,
    /// Recent document paths. Task 07 maintains the list.
    pub recent_files: Vec<String>,
    /// Last successful open time per recent path, in nonnegative Unix seconds.
    /// Missing entries have no known open time.
    pub recent_opened_at: BTreeMap<String, i64>,
    /// Presenter debug overlay. Task 08 draws it.
    pub debug_overlay: bool,
}

impl Default for GeneralSettings {
    fn default() -> Self {
        Self {
            language: "zh-CN".to_owned(),
            theme: Theme::Dark,
            ui_font_family: String::new(),
            ui_font_size: 16,
            reopen_last_document: false,
            recent_files_limit: 10,
            recent_files: Vec::new(),
            recent_opened_at: BTreeMap::new(),
            debug_overlay: false,
        }
    }
}

impl GeneralSettings {
    /// Record a successful open and maintain the bounded, deduplicated MRU list.
    /// A negative timestamp leaves the opened path's time unknown.
    pub fn record_recent_file(&mut self, path: String, opened_at: i64) {
        self.recent_files.insert(0, path.clone());
        let mut seen = BTreeSet::new();
        self.recent_files.retain(|path| seen.insert(path.clone()));
        self.recent_files
            .truncate(usize::try_from(self.recent_files_limit).unwrap_or(0));
        if opened_at >= 0 {
            self.recent_opened_at.insert(path, opened_at);
        } else {
            self.recent_opened_at.remove(&path);
        }
        self.prune_recent_opened_at();
    }

    pub(crate) fn prune_recent_opened_at(&mut self) {
        self.recent_opened_at
            .retain(|path, _| self.recent_files.contains(path));
    }
}

/// `display` section.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DisplaySettings {
    /// Single-monitor policy. The default opens a fullscreen audience on that monitor.
    pub single_monitor_mode: SingleMonitorMode,
    /// Start placement chosen next to the Start button.
    pub start_mode: StartMode,
    /// Audience window starts fullscreen when a presentation starts.
    pub audience_fullscreen: bool,
    /// Completed left clicks in the playback window advance one content step.
    pub audience_click_to_advance: bool,
    /// Presenter window starts fullscreen.
    pub presenter_start_fullscreen: bool,
    /// Saved role bindings. Empty means task 03 may apply its unsaved heuristic.
    pub monitors: Vec<MonitorBinding>,
}

impl Default for DisplaySettings {
    fn default() -> Self {
        Self {
            single_monitor_mode: SingleMonitorMode::FullscreenAudience,
            start_mode: StartMode::Dual,
            audience_fullscreen: true,
            audience_click_to_advance: true,
            presenter_start_fullscreen: false,
            monitors: Vec::new(),
        }
    }
}

/// `presenter` section.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PresenterSettings {
    /// Fraction of the presenter width used by the next-slide preview.
    pub next_slide_ratio: f64,
    /// Fraction of the right column height used by notes.
    pub notes_ratio: f64,
    /// Notes font scale relative to the default size.
    pub notes_font_scale: f64,
    /// Show the clock.
    pub show_clock: bool,
    /// Legacy read-compatible value; the removed thumbnail strip ignores it.
    pub show_thumbnail_strip: bool,
    /// Overview columns, from 1 to 4.
    pub grid_columns: u32,
    /// Overview pages, from 1 to 16. Independent of the column count.
    pub grid_visible_pages: u32,
}

impl PresenterSettings {
    /// Normalize old small-note layouts without rewriting user settings.
    pub fn effective_notes_ratio(&self) -> f64 {
        self.notes_ratio.clamp(0.55, 0.8)
    }
}

impl Default for PresenterSettings {
    fn default() -> Self {
        Self {
            next_slide_ratio: 0.45,
            notes_ratio: 0.65,
            notes_font_scale: 1.0,
            show_clock: true,
            show_thumbnail_strip: false,
            grid_columns: 4,
            grid_visible_pages: 16,
        }
    }
}

/// `timer` section.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct TimerSettings {
    /// Count up or count down.
    pub mode: TimerMode,
    /// Countdown length in minutes. Zero is rejected and falls back to 45.
    pub target_minutes: u32,
    /// Yellow warning when this many minutes remain. Must be greater than red.
    pub warn_yellow_remaining_min: u32,
    /// Red warning when this many minutes remain.
    pub warn_red_remaining_min: u32,
}

impl Default for TimerSettings {
    fn default() -> Self {
        Self {
            mode: TimerMode::CountDown,
            target_minutes: 45,
            warn_yellow_remaining_min: 10,
            warn_red_remaining_min: 3,
        }
    }
}

/// `pointer` section. Colors are BGRA, matching the shared frame format.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PointerSettings {
    /// Laser color as four bytes, blue, green, red, alpha.
    pub laser_color_bgra: [u8; 4],
    /// Laser radius as a fraction of the page width.
    pub laser_radius_norm: f64,
    /// Pen color as four BGRA bytes.
    pub pen_color_bgra: [u8; 4],
    /// Pen width as a fraction of the page width.
    pub pen_width_norm: f64,
    /// Highlighter opacity from 0 to 1.
    pub highlighter_alpha: f64,
}

impl Default for PointerSettings {
    fn default() -> Self {
        Self {
            laser_color_bgra: [0, 0, 255, 255],
            laser_radius_norm: 0.008,
            pen_color_bgra: [0, 0, 255, 255],
            pen_width_norm: 0.004,
            highlighter_alpha: 0.5,
        }
    }
}

/// `cache` section. Sizes come from `ferrostage-render`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CacheSettings {
    /// GPU atlas budget in mebibytes.
    pub l1_gpu_mib: u32,
    /// CPU bitmap budget in mebibytes.
    pub l2_cpu_mib: u32,
    /// Thumbnail budget in mebibytes.
    pub l3_thumb_mib: u32,
}

impl Default for CacheSettings {
    fn default() -> Self {
        Self {
            l1_gpu_mib: L1_GPU_BUDGET_MIB,
            l2_cpu_mib: L2_CPU_BUDGET_MIB,
            l3_thumb_mib: L3_THUMB_BUDGET_MIB,
        }
    }
}

/// `hot_reload` section.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct HotReloadSettings {
    /// Watch the open PDF for replacement. Task 10 owns the watcher.
    pub enabled: bool,
    /// Quiet period in milliseconds. The platform policy uses 300.
    pub debounce_ms: u64,
}

impl Default for HotReloadSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            debounce_ms: 300,
        }
    }
}

/// Media output limits; changes apply to the next playback instance.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct MediaSettings {
    pub max_decode_height: u32,
}
impl Default for MediaSettings {
    fn default() -> Self {
        Self {
            max_decode_height: 1080,
        }
    }
}

/// Resolved settings for schema version 1.
///
/// This type serializes for tests and for a fresh document. Loading does not
/// deserialize the whole file into this struct: one bad leaf would otherwise
/// drop every other key.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Settings {
    /// Schema version understood by the typed model.
    pub schema_version: i64,
    /// Language, theme, and recent files.
    pub general: GeneralSettings,
    /// Monitor roles and fullscreen flags.
    pub display: DisplaySettings,
    /// Presenter layout.
    pub presenter: PresenterSettings,
    /// Talk timer.
    pub timer: TimerSettings,
    /// Laser, pen, and highlighter.
    pub pointer: PointerSettings,
    /// Raw key-binding strings.
    pub keybindings: Keybindings,
    /// Render cache budgets.
    pub cache: CacheSettings,
    /// PDF reload watch.
    pub hot_reload: HotReloadSettings,
    pub media: MediaSettings,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            general: GeneralSettings::default(),
            display: DisplaySettings::default(),
            presenter: PresenterSettings::default(),
            timer: TimerSettings::default(),
            pointer: PointerSettings::default(),
            keybindings: Keybindings::default(),
            cache: CacheSettings::default(),
            hot_reload: HotReloadSettings::default(),
            media: MediaSettings::default(),
        }
    }
}

pub(crate) const SECTION_NAMES: [&str; 9] = [
    "general",
    "display",
    "presenter",
    "timer",
    "pointer",
    "keybindings",
    "cache",
    "hot_reload",
    "media",
];
