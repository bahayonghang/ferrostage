//! Same-directory synced replacement and syntax-error backups.

use super::{LoadFault, SaveFault};
use crate::error::{SaveError, SettingsError, brief_display};
use std::ffi::OsString;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const REPLACE_ATTEMPTS: u32 = 3;
const REPLACE_RETRY: Duration = Duration::from_millis(20);
static FILE_SEQ: AtomicU64 = AtomicU64::new(1);

pub(super) fn atomic_write(dest: &Path, bytes: &[u8], fault: SaveFault) -> Result<(), SaveError> {
    if let Some(parent) = dest.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent).map_err(|error| SaveError::CreateDir {
            reason: format!("{}: {}", parent.display(), brief_display(&error)),
        })?;
    }
    if fault.fail_create_temp {
        return fail(SaveError::Write {
            reason: "injected temp create failure".to_owned(),
        });
    }
    let (tmp, mut file) = create_temp(dest)?;
    let write_result = write_synced(&mut file, bytes);
    drop(file);
    if let Err(error) = write_result {
        remove_temp(&tmp);
        return fail(SaveError::Write {
            reason: format!("{}: {}", tmp.display(), brief_display(&error)),
        });
    }
    if fault.crash_after_sync {
        return fail(SaveError::Write {
            reason: "injected interrupt after temp sync".to_owned(),
        });
    }
    if fault.fail_rename {
        remove_temp(&tmp);
        return fail(SaveError::Replace {
            reason: "injected replace failure".to_owned(),
        });
    }
    if let Err(error) = rename_replacing(&tmp, dest) {
        remove_temp(&tmp);
        return fail(error);
    }
    Ok(())
}

fn write_synced(file: &mut File, bytes: &[u8]) -> io::Result<()> {
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(())
}

fn create_temp(dest: &Path) -> Result<(PathBuf, File), SaveError> {
    let mut last_error = None;
    for _ in 0..8 {
        let path = unique_temp_path(dest);
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(file) => return Ok((path, file)),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
                last_error = Some(error);
            }
            Err(error) => {
                return fail(SaveError::Write {
                    reason: format!("{}: {}", path.display(), brief_display(&error)),
                });
            }
        }
    }
    fail(SaveError::Write {
        reason: format!(
            "could not create a unique settings temp file: {}",
            last_error
                .as_ref()
                .map(brief_display)
                .unwrap_or_else(|| "unknown error".to_owned())
        ),
    })
}

fn unique_temp_path(dest: &Path) -> PathBuf {
    let file_name = dest
        .file_name()
        .map(OsString::from)
        .unwrap_or_else(|| OsString::from("settings.toml"));
    let mut name = file_name;
    name.push(".tmp-");
    name.push(unique_suffix());
    dest.with_file_name(name)
}

fn unique_suffix() -> String {
    let seq = FILE_SEQ.fetch_add(1, Ordering::Relaxed);
    format!("{}-{seq}", process::id())
}

fn rename_replacing(tmp: &Path, dest: &Path) -> Result<(), SaveError> {
    let mut last_error = None;
    for attempt in 1..=REPLACE_ATTEMPTS {
        match fs::rename(tmp, dest) {
            Ok(()) => return Ok(()),
            Err(error) => {
                last_error = Some(error);
                if attempt < REPLACE_ATTEMPTS {
                    thread::sleep(REPLACE_RETRY);
                }
            }
        }
    }
    Err(SaveError::Replace {
        reason: format!(
            "{}: {} (after {REPLACE_ATTEMPTS} attempts)",
            dest.display(),
            last_error
                .as_ref()
                .map(brief_display)
                .unwrap_or_else(|| "unknown error".to_owned())
        ),
    })
}

fn remove_temp(path: &Path) {
    match fs::remove_file(path) {
        Ok(()) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => {
            tracing::warn!(
                path = %path.display(),
                error = %brief_display(&error),
                "failed to remove settings temp file"
            );
        }
    }
}

pub(super) fn fail<T>(error: SaveError) -> Result<T, SaveError> {
    tracing::warn!(error = %error, "settings save failed");
    Err(error)
}

pub(super) fn backup_original(
    path: &Path,
    bytes: &[u8],
    fault: LoadFault,
) -> Result<PathBuf, SettingsError> {
    if fault.fail_backup {
        return Err(SettingsError::Backup("injected backup failure".to_owned()));
    }
    if let Some(existing) = find_identical_backup(path, bytes)? {
        return Ok(existing);
    }
    let backup = broken_backup_path(path);
    atomic_write(&backup, bytes, SaveFault::default())
        .map_err(|error| SettingsError::Backup(brief_display(&error)))?;
    let written = fs::read(&backup).map_err(|error| {
        SettingsError::Backup(format!("{}: {}", backup.display(), brief_display(&error)))
    })?;
    if written != bytes {
        return Err(SettingsError::Backup(
            "backup contents did not match the original file".to_owned(),
        ));
    }
    Ok(backup)
}

fn broken_backup_path(path: &Path) -> PathBuf {
    let name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| "settings.toml".to_owned());
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0);
    let seq = FILE_SEQ.fetch_add(1, Ordering::Relaxed);
    path.with_file_name(format!("{name}.broken-{millis}-{seq}"))
}

fn find_identical_backup(path: &Path, bytes: &[u8]) -> Result<Option<PathBuf>, SettingsError> {
    let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    else {
        return Ok(None);
    };
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return Ok(None);
    };
    let prefix = format!("{name}.broken-");
    let entries = match fs::read_dir(parent) {
        Ok(entries) => entries,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(SettingsError::Backup(format!(
                "{}: {}",
                parent.display(),
                brief_display(&error)
            )));
        }
    };
    for entry in entries {
        let entry = entry.map_err(|error| SettingsError::Backup(brief_display(&error)))?;
        let file_name = entry.file_name();
        let Some(file_name) = file_name.to_str() else {
            continue;
        };
        if !file_name.starts_with(&prefix) {
            continue;
        }
        match fs::read(entry.path()) {
            Ok(existing) if existing == bytes => return Ok(Some(entry.path())),
            Ok(_) => {}
            Err(error) => {
                tracing::warn!(
                    path = %entry.path().display(),
                    error = %brief_display(&error),
                    "skipped unreadable settings backup candidate"
                );
            }
        }
    }
    Ok(None)
}
