//! NTFS replacement cases. These run only on Windows and only inside a temp directory.

use std::fs::{self, OpenOptions};
use std::os::windows::fs::OpenOptionsExt;
use std::process::Command;

use crate::store::SaveFault;
use crate::{SettingsStore, from_toml};

use super::Scratch;

const FILE_SHARE_READ: u32 = 0x0000_0001;
const FILE_SHARE_WRITE: u32 = 0x0000_0002;

#[test]
#[allow(clippy::permissions_set_readonly_false)]
fn ntfs_replace_keeps_a_complete_file() {
    let scratch = Scratch::new("ntfs");
    let path = scratch.child("settings.toml");
    let store = SettingsStore::at(&path);
    let new_outcome = from_toml("").expect("defaults");
    let new_bytes = {
        store.save(&new_outcome).expect("missing target");
        fs::read(&path).expect("created")
    };
    assert!(!new_bytes.is_empty());
    assert_eq!(fs::read(&path).expect("full"), new_bytes);
    assert!(temp_names(&scratch).is_empty());
    println!(
        "WINDOWS_REPLACE missing_target=new_complete bytes={}",
        new_bytes.len()
    );

    let old = b"OLD_COMPLETE_SETTINGS\n";
    fs::write(&path, old).expect("existing");
    store.save(&new_outcome).expect("existing target");
    let replaced = fs::read(&path).expect("replaced");
    assert_eq!(replaced, new_bytes);
    assert!(temp_names(&scratch).is_empty());
    println!("WINDOWS_REPLACE existing_target=new_complete");

    fs::write(&path, old).expect("readonly fixture");
    let mut permissions = fs::metadata(&path).expect("meta").permissions();
    permissions.set_readonly(true);
    fs::set_permissions(&path, permissions).expect("readonly");
    let readonly_result = store.save(&new_outcome);
    let readonly_bytes = fs::read(&path).expect("readonly bytes");
    let readonly_outcome = classify(&readonly_bytes, old, &new_bytes);
    println!("WINDOWS_REPLACE readonly_attribute={readonly_outcome} save={readonly_result:?}");
    assert_eq!(
        readonly_bytes,
        if readonly_result.is_ok() {
            &new_bytes[..]
        } else {
            &old[..]
        }
    );
    assert!(temp_names(&scratch).is_empty());
    let mut permissions = fs::metadata(&path).expect("meta").permissions();
    permissions.set_readonly(false);
    fs::set_permissions(&path, permissions).expect("clear readonly");

    fs::write(&path, old).expect("lock fixture");
    let locked = OpenOptions::new()
        .read(true)
        .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
        .open(&path)
        .expect("share lock");
    let lock_result = store.save(&new_outcome);
    let locked_bytes = fs::read(&path).expect("locked bytes");
    let lock_outcome = classify(&locked_bytes, old, &new_bytes);
    println!(
        "WINDOWS_REPLACE share_mode=FILE_SHARE_READ|FILE_SHARE_WRITE no FILE_SHARE_DELETE outcome={lock_outcome} save={lock_result:?}"
    );
    assert!(matches!(lock_result, Err(crate::SaveError::Replace { .. })));
    assert_eq!(locked_bytes, old);
    assert!(temp_names(&scratch).is_empty());
    drop(locked);
    store.save(&new_outcome).expect("retry after unlock");
    assert_eq!(fs::read(&path).expect("retried"), new_bytes);
    assert!(temp_names(&scratch).is_empty());
    println!("WINDOWS_REPLACE retry_after_unlock=new_complete temps=0");

    fs::write(&path, old).expect("interrupt fixture");
    let interrupt = store.save_with_fault(
        &new_outcome,
        SaveFault {
            crash_after_sync: true,
            ..SaveFault::default()
        },
    );
    assert!(interrupt.is_err());
    assert_eq!(fs::read(&path).expect("interrupted"), old);
    let temps = temp_names(&scratch);
    assert_eq!(temps.len(), 1, "{temps:?}");
    assert_eq!(
        fs::read(scratch.child(&temps[0])).expect("synced temp"),
        new_bytes
    );
    println!(
        "WINDOWS_REPLACE interrupt_after_tmp=old_complete temp={}",
        temps[0]
    );
    fs::remove_file(scratch.child(&temps[0])).expect("remove interrupted fixture");

    fs::write(&path, old).expect("replace fixture");
    let replace = store.save_with_fault(
        &new_outcome,
        SaveFault {
            fail_rename: true,
            ..SaveFault::default()
        },
    );
    assert!(replace.is_err());
    assert_eq!(fs::read(&path).expect("replace failed"), old);
    assert!(temp_names(&scratch).is_empty());
    println!("WINDOWS_REPLACE injected_replace_failure=old_complete temps=0");

    let volume = Command::new("powershell.exe")
        .args([
            "-NoLogo",
            "-NoProfile",
            "-Command",
            "[System.Environment]::OSVersion.VersionString; [System.IO.DriveInfo]::new([System.IO.Path]::GetPathRoot($env:FERROSTAGE_TEST_TEMP)).DriveFormat",
        ])
        .env("FERROSTAGE_TEST_TEMP", scratch.path())
        .output()
        .expect("drive format");
    assert!(
        volume.status.success(),
        "{}",
        String::from_utf8_lossy(&volume.stderr)
    );
    let format = String::from_utf8_lossy(&volume.stdout);
    println!("WINDOWS_VOLUME format={}", format.trim());
    println!("WINDOWS_VOLUME temp={}", scratch.path().display());
}

fn classify(actual: &[u8], old: &[u8], new_bytes: &[u8]) -> &'static str {
    if actual == new_bytes {
        "new_complete"
    } else if actual == old {
        "old_complete"
    } else {
        "torn"
    }
}

fn temp_names(scratch: &Scratch) -> Vec<String> {
    fs::read_dir(scratch.path())
        .expect("dir")
        .map(|entry| {
            entry
                .expect("entry")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .filter(|name| name.contains(".tmp-"))
        .collect()
}
