//! Platform configuration paths; pure inputs are testable on every host.

use crate::error::SettingsError;
use std::env;
use std::path::{Path, PathBuf};

/// Which operating-system config directory to use.
///
/// Only the host variant is constructed by [`current_platform`]. The other
/// variants are constructed by tests so every platform path is checked on Windows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub(crate) enum ConfigPlatform {
    Windows,
    Linux,
    Macos,
}

pub(crate) fn settings_file(
    platform: ConfigPlatform,
    appdata: Option<&Path>,
    xdg_config_home: Option<&Path>,
    home: Option<&Path>,
) -> Result<PathBuf, SettingsError> {
    let dir = config_dir(platform, appdata, xdg_config_home, home)?;
    Ok(dir.join("ferrostage").join("settings.toml"))
}

fn config_dir(
    platform: ConfigPlatform,
    appdata: Option<&Path>,
    xdg_config_home: Option<&Path>,
    home: Option<&Path>,
) -> Result<PathBuf, SettingsError> {
    match platform {
        ConfigPlatform::Windows => non_empty(appdata)
            .map(PathBuf::from)
            .ok_or_else(|| SettingsError::Path("APPDATA is not set".to_owned())),
        ConfigPlatform::Linux => {
            if let Some(xdg) = non_empty(xdg_config_home) {
                Ok(PathBuf::from(xdg))
            } else {
                non_empty(home)
                    .map(|path| path.join(".config"))
                    .ok_or_else(|| SettingsError::Path("HOME is not set".to_owned()))
            }
        }
        ConfigPlatform::Macos => non_empty(home)
            .map(|path| path.join("Library").join("Application Support"))
            .ok_or_else(|| SettingsError::Path("HOME is not set".to_owned())),
    }
}

fn non_empty(path: Option<&Path>) -> Option<&Path> {
    path.filter(|path| !path.as_os_str().is_empty())
}

pub(super) fn current_platform() -> ConfigPlatform {
    #[cfg(target_os = "windows")]
    {
        ConfigPlatform::Windows
    }
    #[cfg(target_os = "macos")]
    {
        ConfigPlatform::Macos
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        ConfigPlatform::Linux
    }
}

pub(super) fn env_path(key: &str) -> Option<PathBuf> {
    env::var_os(key)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}
