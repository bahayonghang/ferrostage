//! Settings errors.
//!
//! Read, backup, write, and replace failures stay distinct. Messages name the
//! path and the OS error. They do not include the file body.

use std::fmt::Display;

use thiserror::Error;

/// Failure while resolving a path, reading a file, or parsing syntax.
///
/// A missing settings file is not an error. Invalid syntax is an error for
/// [`crate::from_toml`] and a recovered session for [`crate::SettingsStore::load`].
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum SettingsError {
    /// The document is not valid TOML. The string is a short summary, not the file.
    #[error("settings TOML is invalid: {0}")]
    Toml(String),

    /// The file exists but could not be read.
    #[error("cannot read settings: {0}")]
    Read(String),

    /// A syntax-error backup could not be completed. The original file is unchanged.
    #[error("cannot back up settings: {0}")]
    Backup(String),

    /// The platform config directory could not be resolved.
    #[error("cannot resolve settings path: {0}")]
    Path(String),
}

/// Failure while replacing `settings.toml`.
///
/// Task 09 keeps the in-memory session, marks it dirty, and can retry.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum SaveError {
    /// Future schema, or a backup that did not succeed. The file must not change.
    #[error("settings are read-only: {reason}")]
    ReadOnly {
        /// Why writes are refused.
        reason: String,
    },

    /// The parent directory could not be created.
    #[error("cannot create settings directory: {reason}")]
    CreateDir {
        /// Path and OS error.
        reason: String,
    },

    /// The sibling temp file could not be written or synced.
    #[error("cannot write settings: {reason}")]
    Write {
        /// Path and OS error, or an injected fault.
        reason: String,
    },

    /// The temp file was written but could not replace the destination.
    #[error("cannot replace settings file: {reason}")]
    Replace {
        /// Path and OS error. The previous destination is left in place.
        reason: String,
    },
}

pub(crate) fn brief_text(text: &str) -> String {
    let first = text.lines().next().unwrap_or("unknown error");
    let mut brief = first.to_owned();
    const MAX: usize = 180;
    if brief.len() > MAX {
        let end = brief.floor_char_boundary(MAX);
        brief.truncate(end);
    }
    brief
}

pub(crate) fn brief_display(error: &impl Display) -> String {
    brief_text(&error.to_string())
}
