//! Settings file location, backup, and same-directory replace.
//!
//! The only path this module renames is the settings file it was given. It does
//! not open or replace a PDF. The sibling temp name contains `.tmp-` and is
//! never `.ferrostage.tmp`.

use crate::error::{SaveError, SettingsError, brief_display};
use crate::merge::merge_document;
use crate::parse::{LoadOutcome, SettingsWarning, WritePolicy, from_toml};
use crate::schema::Settings;
use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

mod files;
mod paths;
use files::{atomic_write, backup_original, fail};
#[cfg(test)]
pub(crate) use paths::ConfigPlatform;
pub(crate) use paths::settings_file;
use paths::{current_platform, env_path};

/// In-memory settings plus the original tree and save status.
///
/// Task 09 stores this for the session. A failed save leaves `settings` as the
/// caller set them, sets `dirty`, and records `save_error`.
#[derive(Clone, Debug, PartialEq)]
pub struct SettingsSession {
    /// Typed settings for the session.
    pub settings: Settings,
    /// Original tree, updated to the merged document after a successful save.
    pub raw_table: toml::Table,
    /// Warnings from the last load.
    pub warnings: Vec<SettingsWarning>,
    /// Whether the on-disk file may be replaced.
    pub write_policy: WritePolicy,
    /// The writable file is missing a version stamp or a section table.
    pub needs_writeback: bool,
    /// `true` after an edit or a failed save that the UI should retry.
    pub dirty: bool,
    /// Last save failure. `None` after a successful save.
    pub save_error: Option<SaveError>,
    /// Flat keys already copied into sections. Cleared after they are removed.
    pub consumed_legacy_keys: Vec<String>,
}

impl SettingsSession {
    /// Build a session from a load. It is not dirty until an edit or a failed save.
    #[must_use]
    pub fn from_outcome(outcome: LoadOutcome) -> Self {
        Self {
            settings: outcome.settings,
            raw_table: outcome.raw_table,
            warnings: outcome.warnings,
            write_policy: outcome.write_policy,
            needs_writeback: outcome.needs_writeback,
            dirty: false,
            save_error: None,
            consumed_legacy_keys: outcome.consumed_legacy_keys,
        }
    }

    /// Mark the session unsaved after a user edit. Does not write.
    pub fn note_edit(&mut self) {
        self.dirty = true;
        self.save_error = None;
    }

    fn as_outcome(&self) -> LoadOutcome {
        LoadOutcome {
            settings: self.settings.clone(),
            raw_table: self.raw_table.clone(),
            warnings: self.warnings.clone(),
            write_policy: self.write_policy.clone(),
            needs_writeback: self.needs_writeback,
            consumed_legacy_keys: self.consumed_legacy_keys.clone(),
        }
    }
}

/// Faults used by tests. Production saves use [`SaveFault::default`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct SaveFault {
    pub fail_create_temp: bool,
    pub crash_after_sync: bool,
    pub fail_rename: bool,
}

/// Faults used by tests of the syntax-error backup.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct LoadFault {
    pub fail_backup: bool,
}

/// A settings file path.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SettingsStore {
    path: PathBuf,
}

impl SettingsStore {
    /// Store that reads and writes `path`. This does not create the file.
    #[must_use]
    pub fn at(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// Platform config path.
    ///
    /// Windows: `%APPDATA%\ferrostage\settings.toml`.
    /// Linux: `$XDG_CONFIG_HOME/ferrostage/settings.toml`, or `~/.config` when
    /// that variable is unset. macOS: `~/Library/Application Support/ferrostage/settings.toml`.
    ///
    /// # Errors
    ///
    /// Returns [`SettingsError::Path`] when the required environment variable is missing.
    pub fn default_path() -> Result<PathBuf, SettingsError> {
        settings_file(
            current_platform(),
            env_path("APPDATA").as_deref(),
            env_path("XDG_CONFIG_HOME").as_deref(),
            env_path("HOME").as_deref(),
        )
    }

    /// Path passed to [`Self::at`] or resolved by [`Self::default_path`].
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Load the file.
    ///
    /// A missing file is the defaults and `needs_writeback`, not an error.
    /// Invalid syntax is backed up to `settings.toml.broken-<timestamp>` and the
    /// session uses defaults. The original bytes stay in place until a later
    /// explicit save. If the backup fails, the original bytes stay and writes
    /// are refused. This method does not fill missing sections.
    ///
    /// # Errors
    ///
    /// Returns [`SettingsError::Read`] when the file exists but cannot be read.
    pub fn load(&self) -> Result<LoadOutcome, SettingsError> {
        self.load_with_fault(LoadFault::default())
    }

    pub(crate) fn load_with_fault(&self, fault: LoadFault) -> Result<LoadOutcome, SettingsError> {
        match fs::read(&self.path) {
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(missing_file_outcome()),
            Err(error) => Err(SettingsError::Read(format!(
                "{}: {}",
                self.path.display(),
                brief_display(&error)
            ))),
            Ok(bytes) => self.finish_load(&bytes, fault),
        }
    }

    /// Load, then save when the writable current schema is missing sections.
    ///
    /// `load` and `from_toml` do not call this. A failed fill keeps the session
    /// values, sets `dirty`, and stores [`SaveError`].
    ///
    /// # Errors
    ///
    /// Returns the same read errors as [`Self::load`]. Save failures are stored
    /// on the session instead of being returned.
    pub fn open_for_startup(&self) -> Result<SettingsSession, SettingsError> {
        let mut session = SettingsSession::from_outcome(self.load()?);
        if session.needs_writeback && session.write_policy.is_writable() {
            match self.save_session(&mut session) {
                Ok(()) => {}
                Err(error) => {
                    tracing::warn!(
                        error = %error,
                        path = %self.path.display(),
                        "settings startup fill failed"
                    );
                }
            }
        }
        Ok(session)
    }

    /// Merge known leaves and replace the file.
    ///
    /// Writes a unique sibling temp in the same directory, syncs and closes it,
    /// then [`std::fs::rename`]s it onto the destination. Rename replaces an
    /// existing same-volume file. The destination is not deleted first. On
    /// failure the previous bytes stay and the temp file is removed, except for
    /// the crash fault that leaves the temp behind on purpose.
    ///
    /// # Errors
    ///
    /// Returns [`SaveError::ReadOnly`] before creating a temp file when the
    /// policy forbids writes. Directory, temp, and replace failures are separate
    /// variants. A replace failure leaves the previous file complete.
    pub fn save(&self, outcome: &LoadOutcome) -> Result<toml::Table, SaveError> {
        self.save_with_fault(outcome, SaveFault::default())
    }

    pub(crate) fn save_with_fault(
        &self,
        outcome: &LoadOutcome,
        fault: SaveFault,
    ) -> Result<toml::Table, SaveError> {
        if !outcome.write_policy.is_writable() {
            let reason = match &outcome.write_policy {
                WritePolicy::ReadOnly { reason } => reason.clone(),
                WritePolicy::Writable => "writes are disabled".to_owned(),
            };
            return fail(SaveError::ReadOnly { reason });
        }
        let merged = merge_document(
            &outcome.raw_table,
            &outcome.settings,
            &outcome.consumed_legacy_keys,
        )?;
        let text = encode_document(&merged)?;
        atomic_write(&self.path, text.as_bytes(), fault)?;
        Ok(merged)
    }

    /// Save `session` and update its dirty flag.
    ///
    /// On failure the session values stay, `dirty` is set, and `save_error` is
    /// recorded. The on-disk bytes are unchanged when the error is a refused or
    /// failed replace.
    ///
    /// # Errors
    ///
    /// Returns the same errors as [`Self::save`].
    pub fn save_session(&self, session: &mut SettingsSession) -> Result<(), SaveError> {
        match self.save(&session.as_outcome()) {
            Ok(merged) => {
                session.raw_table = merged;
                // Provenance now refers to the newly saved tree, whose duplicate
                // order may differ after deleting or reordering bindings.
                let mut occurrences = BTreeMap::<&str, usize>::new();
                for binding in &mut session.settings.display.monitors {
                    let occurrence = occurrences.entry(&binding.id).or_default();
                    binding.source_occurrence = Some(*occurrence);
                    *occurrence += 1;
                }
                session.consumed_legacy_keys.clear();
                session.needs_writeback = false;
                session.dirty = false;
                session.save_error = None;
                Ok(())
            }
            Err(error) => {
                session.dirty = true;
                session.save_error = Some(error.clone());
                Err(error)
            }
        }
    }

    fn finish_load(&self, bytes: &[u8], fault: LoadFault) -> Result<LoadOutcome, SettingsError> {
        let text = match str::from_utf8(bytes) {
            Ok(text) => text,
            Err(_) => {
                return self.recover_syntax(bytes, fault, "settings file is not valid UTF-8");
            }
        };
        match from_toml(text) {
            Ok(outcome) => Ok(outcome),
            Err(SettingsError::Toml(reason)) => self.recover_syntax(bytes, fault, &reason),
            Err(error) => Err(error),
        }
    }

    fn recover_syntax(
        &self,
        bytes: &[u8],
        fault: LoadFault,
        reason: &str,
    ) -> Result<LoadOutcome, SettingsError> {
        let mut outcome = missing_file_outcome();
        outcome.needs_writeback = false;
        match backup_original(&self.path, bytes, fault) {
            Ok(backup) => {
                outcome.warnings.push(SettingsWarning {
                    key: "<document>".to_owned(),
                    reason: format!(
                        "invalid settings syntax ({}); backed up to {}",
                        brief_display(&reason),
                        backup.display()
                    ),
                });
                log_warnings(&outcome.warnings);
                Ok(outcome)
            }
            Err(error) => {
                outcome.write_policy = WritePolicy::ReadOnly {
                    reason: format!(
                        "backup failed ({}); the original file is unchanged and writes are disabled",
                        brief_display(&error)
                    ),
                };
                outcome.warnings.push(SettingsWarning {
                    key: "<document>".to_owned(),
                    reason: "invalid settings syntax; backup failed so the original file is unchanged and writes are disabled"
                        .to_owned(),
                });
                log_warnings(&outcome.warnings);
                Ok(outcome)
            }
        }
    }
}

fn missing_file_outcome() -> LoadOutcome {
    LoadOutcome {
        settings: Settings::default(),
        raw_table: toml::Table::new(),
        warnings: Vec::new(),
        write_policy: WritePolicy::Writable,
        needs_writeback: true,
        consumed_legacy_keys: Vec::new(),
    }
}

fn log_warnings(warnings: &[SettingsWarning]) {
    for warning in warnings {
        tracing::warn!(key = %warning.key, reason = %warning.reason, "settings warning");
    }
}

fn encode_document(table: &toml::Table) -> Result<String, SaveError> {
    let mut text = toml::to_string_pretty(table).map_err(|error| SaveError::Write {
        reason: brief_display(&error),
    })?;
    if !text.ends_with('\n') {
        text.push('\n');
    }
    Ok(text)
}
