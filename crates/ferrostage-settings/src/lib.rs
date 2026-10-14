//! User settings loaded from TOML.
//!
//! Missing keys fall back to [`Settings::default`]. The file is local; this
//! crate does not phone home.

use serde::{Deserialize, Serialize};
use thiserror::Error;

use ferrostage_render::{L1_GPU_BUDGET_MIB, L2_CPU_BUDGET_MIB, L3_THUMB_BUDGET_MIB};

/// Cache sizes are mebibytes. Debounce is milliseconds, matching the platform policy.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub presenter_monitor: Option<u32>,
    pub audience_monitor: Option<u32>,
    pub cache_l1_mib: u32,
    pub cache_l2_mib: u32,
    pub cache_l3_mib: u32,
    pub hot_reload_debounce_ms: u64,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            presenter_monitor: None,
            audience_monitor: None,
            cache_l1_mib: L1_GPU_BUDGET_MIB,
            cache_l2_mib: L2_CPU_BUDGET_MIB,
            cache_l3_mib: L3_THUMB_BUDGET_MIB,
            hot_reload_debounce_ms: 300,
        }
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SettingsError {
    #[error("settings TOML is invalid: {0}")]
    Toml(String),
}

/// Parse settings from a TOML document.
///
/// # Errors
///
/// Returns [`SettingsError::Toml`] when the document is not valid settings TOML.
pub fn from_toml(text: &str) -> Result<Settings, SettingsError> {
    toml::from_str(text).map_err(|error| SettingsError::Toml(error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_the_cache_budget() {
        let settings = Settings::default();
        assert_eq!(settings.cache_l1_mib, 256);
        assert_eq!(settings.cache_l2_mib, 512);
        assert_eq!(settings.cache_l3_mib, 64);
        assert_eq!(settings.hot_reload_debounce_ms, 300);
    }

    #[test]
    fn partial_toml_keeps_other_defaults() {
        let settings = from_toml("presenter_monitor = 1\n").unwrap();
        assert_eq!(settings.presenter_monitor, Some(1));
        assert_eq!(settings.cache_l1_mib, 256);
        assert_eq!(settings.audience_monitor, None);
    }

    #[test]
    fn invalid_toml_is_an_error() {
        assert!(from_toml("cache_l1_mib = ").is_err());
    }
}
