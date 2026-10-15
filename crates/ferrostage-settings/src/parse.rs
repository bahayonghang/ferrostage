//! Leaf-by-leaf TOML parse.
//!
//! The document is read as a value tree. Each known leaf is checked on its own.
//! A bad type or a bad range replaces that leaf with the default and records a
//! warning. A section that is not a table falls back as a whole section. Unknown
//! keys stay in the original tree for [`crate::merge`].

use toml::Value;
use tracing::warn;

use crate::error::{SettingsError, brief_display};
use crate::schema::{SCHEMA_VERSION, SECTION_NAMES, Settings};

mod controls;
mod legacy;
mod sections;
mod value;
use controls::{parse_keybindings, parse_pointer, parse_timer};
use legacy::{parse_cache, parse_hot_reload};
use sections::{parse_display, parse_general, parse_presenter};

/// One settings warning. `key` is a dotted path. `reason` never contains the file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SettingsWarning {
    /// Dotted key, or `<document>` when the failure has no leaf.
    pub key: String,
    /// What was wrong and which default was used.
    pub reason: String,
}

/// Whether a later explicit save may replace the file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WritePolicy {
    /// Current writable schema. Startup may fill missing sections.
    Writable,
    /// Future or unreadable version, or a backup that failed.
    ReadOnly {
        /// Why the file must stay unchanged.
        reason: String,
    },
}

impl WritePolicy {
    /// `true` when an explicit save is allowed to replace the file.
    #[must_use]
    pub fn is_writable(&self) -> bool {
        matches!(self, Self::Writable)
    }
}

/// Result of parsing one document. Parsing does not write.
#[derive(Clone, Debug, PartialEq)]
pub struct LoadOutcome {
    /// Typed settings after per-leaf fallback.
    pub settings: Settings,
    /// Original value tree, including unknown keys.
    pub raw_table: toml::Table,
    /// Key path and reason. The document text is not included.
    pub warnings: Vec<SettingsWarning>,
    /// Whether save may replace the file.
    pub write_policy: WritePolicy,
    /// Current writable schema is missing `schema_version` or a section table.
    pub needs_writeback: bool,
    /// Flat skeleton keys that were copied into sections and should not be kept.
    pub consumed_legacy_keys: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DocumentVersion {
    ImplicitV1,
    Current,
    Future(i64),
    Unrecognized,
}

/// Parse settings from a TOML document without writing.
///
/// An empty document is [`Settings::default`]. Syntax errors return
/// [`SettingsError::Toml`]. Invalid leaves are warnings, not errors.
///
/// # Errors
///
/// Returns [`SettingsError::Toml`] when `text` is not a TOML document.
pub fn from_toml(text: &str) -> Result<LoadOutcome, SettingsError> {
    let table = text
        .parse::<toml::Table>()
        .map_err(|error| SettingsError::Toml(brief_display(&error)))?;
    Ok(outcome_from_table(table))
}

fn outcome_from_table(table: toml::Table) -> LoadOutcome {
    let mut warnings = Vec::new();
    let version = classify_version(&table, &mut warnings);
    let legacy = matches!(version, DocumentVersion::ImplicitV1);
    let mut consumed_legacy_keys = Vec::new();
    warn_legacy_indexes(&table, &mut warnings);
    let settings = Settings {
        schema_version: match version {
            DocumentVersion::Future(version) => version,
            DocumentVersion::ImplicitV1
            | DocumentVersion::Current
            | DocumentVersion::Unrecognized => SCHEMA_VERSION,
        },
        general: parse_general(section(&table, "general", &mut warnings), &mut warnings),
        display: parse_display(section(&table, "display", &mut warnings), &mut warnings),
        presenter: parse_presenter(section(&table, "presenter", &mut warnings), &mut warnings),
        timer: parse_timer(section(&table, "timer", &mut warnings), &mut warnings),
        pointer: parse_pointer(section(&table, "pointer", &mut warnings), &mut warnings),
        keybindings: parse_keybindings(
            section(&table, "keybindings", &mut warnings),
            &mut warnings,
        ),
        cache: parse_cache(
            &table,
            section(&table, "cache", &mut warnings),
            legacy,
            &mut warnings,
            &mut consumed_legacy_keys,
        ),
        hot_reload: parse_hot_reload(
            &table,
            section(&table, "hot_reload", &mut warnings),
            legacy,
            &mut warnings,
            &mut consumed_legacy_keys,
        ),
    };
    let write_policy = write_policy(version);
    let needs_writeback = needs_writeback(&table, version, &consumed_legacy_keys);
    LoadOutcome {
        settings,
        raw_table: table,
        warnings,
        write_policy,
        needs_writeback,
        consumed_legacy_keys,
    }
}

fn classify_version(table: &toml::Table, warnings: &mut Vec<SettingsWarning>) -> DocumentVersion {
    match table.get("schema_version") {
        None => DocumentVersion::ImplicitV1,
        Some(Value::Integer(SCHEMA_VERSION)) => DocumentVersion::Current,
        Some(Value::Integer(version)) if *version > SCHEMA_VERSION => {
            warn(
                warnings,
                "schema_version",
                format!(
                    "schema_version {version} is newer than {SCHEMA_VERSION}; the file is read-only"
                ),
            );
            DocumentVersion::Future(*version)
        }
        Some(Value::Integer(version)) => {
            warn(
                warnings,
                "schema_version",
                format!("schema_version {version} is not supported; the file is read-only"),
            );
            DocumentVersion::Unrecognized
        }
        Some(other) => {
            warn(
                warnings,
                "schema_version",
                format!(
                    "expected an integer, found {}; the file is read-only",
                    other.type_str()
                ),
            );
            DocumentVersion::Unrecognized
        }
    }
}

fn write_policy(version: DocumentVersion) -> WritePolicy {
    match version {
        DocumentVersion::ImplicitV1 | DocumentVersion::Current => WritePolicy::Writable,
        DocumentVersion::Future(version) => WritePolicy::ReadOnly {
            reason: format!("schema_version {version} is newer than {SCHEMA_VERSION}"),
        },
        DocumentVersion::Unrecognized => WritePolicy::ReadOnly {
            reason: "schema_version is not the supported writable version".to_owned(),
        },
    }
}

fn needs_writeback(table: &toml::Table, version: DocumentVersion, consumed: &[String]) -> bool {
    if !matches!(
        version,
        DocumentVersion::ImplicitV1 | DocumentVersion::Current
    ) {
        return false;
    }
    if matches!(version, DocumentVersion::ImplicitV1) || !consumed.is_empty() {
        return true;
    }
    SECTION_NAMES
        .iter()
        .any(|name| !matches!(table.get(*name), Some(Value::Table(_))))
}

fn section<'a>(
    table: &'a toml::Table,
    name: &str,
    warnings: &mut Vec<SettingsWarning>,
) -> Option<&'a toml::Table> {
    match table.get(name) {
        None => None,
        Some(Value::Table(section)) => Some(section),
        Some(other) => {
            warn(
                warnings,
                name,
                format!("expected a table, found {}", other.type_str()),
            );
            None
        }
    }
}

fn warn(warnings: &mut Vec<SettingsWarning>, key: &str, reason: impl Into<String>) {
    let reason = reason.into();
    warn!(key, reason = %reason, "settings warning");
    warnings.push(SettingsWarning {
        key: key.to_owned(),
        reason,
    });
}

fn warn_legacy_indexes(table: &toml::Table, warnings: &mut Vec<SettingsWarning>) {
    for key in ["presenter_monitor", "audience_monitor"] {
        if table.contains_key(key) {
            warn(
                warnings,
                key,
                format!(
                    "{key} is a display index and cannot become a monitor id; rebind this monitor"
                ),
            );
        }
    }
}
