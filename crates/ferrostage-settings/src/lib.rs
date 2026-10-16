//! Sectioned TOML settings.
//!
//! [`from_toml`] reads the value tree leaf by leaf. One bad value falls back
//! without dropping the rest of the file, and it never writes. [`SettingsStore`]
//! owns the config path, the broken-file backup, and the same-directory replace
//! of the settings file only.

mod error;
mod keybindings;
mod merge;
mod parse;
mod schema;
mod store;

pub use error::{SaveError, SettingsError};
pub use ferrostage_core::MonitorRole;
pub use parse::{LoadOutcome, SettingsWarning, WritePolicy, from_toml};
pub use schema::{
    BindingIssue, CacheSettings, DisplaySettings, GeneralSettings, HotReloadSettings, Keybindings,
    MediaSettings, MonitorBinding, PointerSettings, PresenterSettings, SCHEMA_VERSION, Settings,
    SingleMonitorMode, Theme, TimerMode, TimerSettings,
};
pub use store::{SettingsSession, SettingsStore};

#[cfg(test)]
mod tests;
