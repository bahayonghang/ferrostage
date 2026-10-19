use std::fs;
use std::path::Path;

use ferrostage_core::MonitorRole;

use crate::schema::{MonitorBinding, Settings};
use crate::store::{ConfigPlatform, LoadFault, SaveFault, settings_file};
use crate::{SettingsStore, WritePolicy, from_toml};

use super::{Scratch, fingerprint};

#[test]
fn config_dirs_follow_each_platform() {
    let appdata = Path::new(r"C:\Users\ada\AppData\Roaming");
    let home = Path::new("/home/ada");
    let mac_home = Path::new("/Users/ada");
    let xdg = Path::new("/xdg");
    assert_eq!(
        settings_file(ConfigPlatform::Windows, Some(appdata), None, None).expect("windows"),
        appdata.join("ferrostage").join("settings.toml")
    );
    assert_eq!(
        settings_file(ConfigPlatform::Linux, None, Some(xdg), Some(home)).expect("xdg"),
        xdg.join("ferrostage").join("settings.toml")
    );
    assert_eq!(
        settings_file(ConfigPlatform::Linux, None, Some(Path::new("")), Some(home)).expect("home"),
        home.join(".config")
            .join("ferrostage")
            .join("settings.toml")
    );
    assert_eq!(
        settings_file(ConfigPlatform::Macos, None, None, Some(mac_home)).expect("macos"),
        mac_home
            .join("Library")
            .join("Application Support")
            .join("ferrostage")
            .join("settings.toml")
    );
    assert!(settings_file(ConfigPlatform::Windows, None, None, None).is_err());
}

#[test]
fn default_path_ends_at_ferrostage_settings() {
    let path = SettingsStore::default_path().expect("platform path");
    assert_eq!(
        path.file_name().and_then(|name| name.to_str()),
        Some("settings.toml")
    );
    assert_eq!(
        path.parent()
            .and_then(|parent| parent.file_name())
            .and_then(|name| name.to_str()),
        Some("ferrostage")
    );
}

#[test]
fn unknown_values_survive_a_known_edit() {
    let scratch = Scratch::new("merge");
    let path = scratch.child("settings.toml");
    let original = r#"
schema_version = 1
vendor_meta = { keep = true }

[general]
language = "zh-CN"
extra = "keep-me"
recent_files = ['C:\talks\first.pdf', 'unknown.pdf']
recent_opened_at = { 'C:\talks\first.pdf' = 1792370000 }

[display]
audience_fullscreen = true
custom_flag = 7

[[display.monitors]]
id = "A"
friendly_name = "First"
role = "presenter"
marker = "first"

[[display.monitors]]
id = "A"
friendly_name = "Second"
role = "audience"
marker = "second"

[[display.monitors]]
id = "B"
friendly_name = "Other"
role = "none"
marker = "b"
"#;
    fs::write(&path, original).expect("fixture");
    let store = SettingsStore::at(&path);
    let mut outcome = store.load().expect("load");
    assert!(outcome.write_policy.is_writable());
    outcome.settings.general.language = "en-US".to_owned();
    outcome.settings.display.monitors[1].role = MonitorRole::None;
    outcome.settings.display.monitors.remove(2);
    outcome.settings.display.monitors.push(MonitorBinding {
        id: "C".to_owned(),
        friendly_name: "New".to_owned(),
        role: MonitorRole::Presenter,
        source_occurrence: None,
    });
    store.save(&outcome).expect("save");
    let saved: toml::Table = fs::read_to_string(&path)
        .expect("read")
        .parse()
        .expect("toml");
    assert_eq!(saved["vendor_meta"]["keep"].as_bool(), Some(true));
    assert_eq!(saved["general"]["extra"].as_str(), Some("keep-me"));
    assert_eq!(saved["general"]["language"].as_str(), Some("en-US"));
    let reloaded = store.load().expect("reload");
    assert_eq!(
        reloaded.settings.general.recent_opened_at,
        [(r"C:\talks\first.pdf".into(), 1792370000)].into()
    );
    assert_eq!(
        reloaded.settings.general.recent_files,
        [r"C:\talks\first.pdf", "unknown.pdf"]
    );
    assert_eq!(reloaded.settings.schema_version, 1);
    assert_eq!(saved["display"]["custom_flag"].as_integer(), Some(7));
    let monitors = saved["display"]["monitors"].as_array().expect("monitors");
    assert_eq!(monitors[0]["marker"].as_str(), Some("first"));
    assert_eq!(monitors[0]["role"].as_str(), Some("presenter"));
    assert_eq!(monitors[1]["marker"].as_str(), Some("second"));
    assert_eq!(monitors[1]["role"].as_str(), Some("none"));
    assert!(
        monitors
            .iter()
            .all(|item| item.get("marker").and_then(|value| value.as_str()) != Some("b"))
    );
    let created = monitors
        .iter()
        .find(|item| item.get("id").and_then(|value| value.as_str()) == Some("C"))
        .expect("new monitor");
    assert!(created.get("marker").is_none());
    assert!(created.as_table().expect("table").len() == 3);
}

#[test]
fn recent_time_map_is_replaced_after_pruning_and_clearing() {
    let scratch = Scratch::new("recent-time-replacement");
    let store = SettingsStore::at(scratch.child("settings.toml"));
    fs::write(
        store.path(),
        "schema_version = 1\nvendor = 'keep'\n[general]\nextra = 7\nrecent_files_limit = 1\n\
         recent_files = ['first.pdf', 'trimmed.pdf']\n\
         [general.recent_opened_at]\n'first.pdf' = 10\n 'trimmed.pdf' = 20\n'orphan.pdf' = 30",
    )
    .unwrap();
    let mut session = crate::SettingsSession::from_outcome(store.load().unwrap());
    assert_eq!(session.settings.general.recent_files, ["first.pdf"]);
    assert_eq!(
        session.settings.general.recent_opened_at,
        [("first.pdf".into(), 10)].into()
    );
    store.save_session(&mut session).unwrap();
    let saved = store.load().unwrap();
    let map = saved.raw_table["general"]["recent_opened_at"]
        .as_table()
        .unwrap();
    assert_eq!(map.len(), 1);
    assert_eq!(map["first.pdf"].as_integer(), Some(10));
    assert_eq!(saved.raw_table["vendor"].as_str(), Some("keep"));
    assert_eq!(saved.raw_table["general"]["extra"].as_integer(), Some(7));

    session.settings.general.recent_files.clear();
    session.settings.general.recent_opened_at.clear();
    session.note_edit();
    store.save_session(&mut session).unwrap();
    let saved = store.load().unwrap();
    assert!(saved.settings.general.recent_files.is_empty());
    assert!(saved.settings.general.recent_opened_at.is_empty());
    assert!(
        saved.raw_table["general"]["recent_opened_at"]
            .as_table()
            .unwrap()
            .is_empty()
    );
    assert_eq!(saved.raw_table["general"]["extra"].as_integer(), Some(7));
}

#[test]
fn audience_click_roundtrips_with_unknown_display_and_monitor_fields() {
    let scratch = Scratch::new("audience-click");
    let store = SettingsStore::at(scratch.child("settings.toml"));
    fs::write(store.path(), "schema_version = 1\n[display]\ncustom = 7\n[[display.monitors]]\nid = 'A'\nrole = 'audience'\nmarker = 'keep'\n").unwrap();
    for enabled in [false, true] {
        let mut session = crate::SettingsSession::from_outcome(store.load().unwrap());
        session.settings.display.audience_click_to_advance = enabled;
        session.note_edit();
        store.save_session(&mut session).unwrap();
        let loaded = store.load().unwrap();
        assert_eq!(loaded.settings.display.audience_click_to_advance, enabled);
        assert_eq!(loaded.raw_table["display"]["custom"].as_integer(), Some(7));
        assert_eq!(
            loaded.raw_table["display"]["monitors"][0]["marker"].as_str(),
            Some("keep")
        );
        assert_eq!(loaded.settings.schema_version, 1);
    }
}

#[test]
fn duplicate_monitor_metadata_survives_repeated_session_saves() {
    let scratch = Scratch::new("duplicate-session");
    let store = SettingsStore::at(scratch.child("settings.toml"));
    let outcome = from_toml(
        r#"
[[display.monitors]]
id = "A"
role = "presenter"
marker = "first"
[[display.monitors]]
id = "A"
role = "audience"
marker = "second"
"#,
    )
    .expect("fixture");
    let mut session = crate::SettingsSession::from_outcome(outcome);
    session.settings.display.monitors.remove(0);
    store.save_session(&mut session).expect("first save");
    session.settings.display.monitors[0].role = MonitorRole::None;
    store.save_session(&mut session).expect("second save");
    let saved = store.load().expect("reload");
    let monitors = saved.raw_table["display"]["monitors"]
        .as_array()
        .expect("monitors");
    assert_eq!(monitors.len(), 1);
    assert_eq!(
        monitors[0].get("marker").and_then(toml::Value::as_str),
        Some("second")
    );
    assert_eq!(monitors[0]["role"].as_str(), Some("none"));
}

#[test]
fn failed_session_save_keeps_edits_and_retry_clears_dirty_state() {
    let scratch = Scratch::new("session-retry");
    let parent = scratch.child("blocked");
    fs::write(&parent, b"blocking file").expect("fixture");
    let store = SettingsStore::at(parent.join("custom.toml"));
    let mut session = crate::SettingsSession::from_outcome(from_toml("").expect("defaults"));
    session.settings.timer.target_minutes = 20;
    session.settings.general.ui_font_family = "Example Family".into();
    session.settings.general.ui_font_size = 22;
    session.settings.display.audience_click_to_advance = false;
    session
        .settings
        .general
        .record_recent_file("recent.pdf".into(), 1792371000);
    session.note_edit();
    let original_tree = session.raw_table.clone();
    assert!(matches!(
        store.save_session(&mut session),
        Err(crate::SaveError::CreateDir { .. })
    ));
    assert_eq!(session.settings.timer.target_minutes, 20);
    assert_eq!(session.settings.general.ui_font_family, "Example Family");
    assert_eq!(session.settings.general.ui_font_size, 22);
    assert!(!session.settings.display.audience_click_to_advance);
    assert_eq!(session.settings.general.recent_files, ["recent.pdf"]);
    assert_eq!(
        session.settings.general.recent_opened_at.get("recent.pdf"),
        Some(&1792371000)
    );
    assert_eq!(session.raw_table, original_tree);
    assert!(session.dirty);
    assert!(session.save_error.is_some());
    fs::remove_file(&parent).expect("release fixture");
    store.save_session(&mut session).expect("retry");
    assert!(!session.dirty);
    assert!(!session.needs_writeback);
    assert!(session.save_error.is_none());
    assert!(
        !store
            .load()
            .expect("reload")
            .settings
            .display
            .audience_click_to_advance
    );
    assert_eq!(
        store.load().expect("reload").settings.timer.target_minutes,
        20
    );
    let reloaded = store.load().expect("reload");
    assert_eq!(reloaded.settings.general.recent_files, ["recent.pdf"]);
    assert_eq!(
        reloaded.settings.general.recent_opened_at.get("recent.pdf"),
        Some(&1792371000)
    );
}

#[test]
fn legacy_save_drops_mapped_keys_and_keeps_indexes() {
    let scratch = Scratch::new("legacy");
    let path = scratch.child("settings.toml");
    fs::write(
        &path,
        "presenter_monitor = 1\naudience_monitor = 2\ncache_l1_mib = 128\nhot_reload_debounce_ms = 450\n",
    )
    .expect("fixture");
    let store = SettingsStore::at(&path);
    let outcome = store.load().expect("load");
    assert!(outcome.needs_writeback);
    store.save(&outcome).expect("save");
    let saved: toml::Table = fs::read_to_string(&path)
        .expect("read")
        .parse()
        .expect("toml");
    assert_eq!(saved["schema_version"].as_integer(), Some(1));
    assert_eq!(saved["cache"]["l1_gpu_mib"].as_integer(), Some(128));
    assert_eq!(saved["hot_reload"]["debounce_ms"].as_integer(), Some(450));
    assert!(saved.get("cache_l1_mib").is_none());
    assert!(saved.get("hot_reload_debounce_ms").is_none());
    assert_eq!(saved["presenter_monitor"].as_integer(), Some(1));
    assert_eq!(saved["audience_monitor"].as_integer(), Some(2));
}

#[test]
fn future_version_refuses_save_and_keeps_the_bytes() {
    let scratch = Scratch::new("future");
    let path = scratch.child("settings.toml");
    let body = "schema_version = 2\nvendor = \"keep\"\n\n[general]\nextra = true\nlanguage = \"en-US\"\nrecent_files = ['old.pdf']\nrecent_opened_at = { 'old.pdf' = 10 }\n\n[[display.monitors]]\nid = \"A\"\nrole = \"presenter\"\nsecret = 9\n";
    fs::write(&path, body).expect("fixture");
    let before = fs::read(&path).expect("before");
    let store = SettingsStore::at(&path);
    let mut outcome = store.load().expect("load");
    assert!(!outcome.write_policy.is_writable());
    outcome.settings.general.language = "zh-CN".to_owned();
    outcome.settings.display.audience_click_to_advance = false;
    assert_eq!(
        outcome.settings.general.recent_opened_at.get("old.pdf"),
        Some(&10)
    );
    outcome
        .settings
        .general
        .record_recent_file("new.pdf".into(), 20);
    let error = store.save(&outcome).expect_err("readonly");
    assert!(matches!(error, crate::SaveError::ReadOnly { .. }));
    let after = fs::read(&path).expect("after");
    assert_eq!(fingerprint(&before), fingerprint(&after));
    assert_eq!(before, after);
    assert!(
        dir_names(scratch.path())
            .iter()
            .all(|name| !name.contains(".tmp-"))
    );
}

#[test]
fn syntax_error_backs_up_and_returns_defaults() {
    let scratch = Scratch::new("syntax");
    let path = scratch.child("settings.toml");
    let body = b"this is not [ toml";
    fs::write(&path, body).expect("fixture");
    let store = SettingsStore::at(&path);
    let outcome = store.load().expect("recovered");
    assert_eq!(outcome.settings, Settings::default());
    assert!(outcome.write_policy.is_writable());
    assert!(!outcome.needs_writeback);
    assert_eq!(fs::read(&path).expect("original"), body);
    let backups = broken_backups(scratch.path());
    assert_eq!(backups.len(), 1, "{backups:?}");
    assert_eq!(fs::read(&backups[0]).expect("backup"), body);
    let again = store.load().expect("second load");
    assert_eq!(again.settings, Settings::default());
    assert_eq!(broken_backups(scratch.path()).len(), 1);
}

#[test]
fn failed_backup_keeps_the_original_and_forbids_writes() {
    let scratch = Scratch::new("backup-fail");
    let path = scratch.child("settings.toml");
    let body = b"not = [valid";
    fs::write(&path, body).expect("fixture");
    let store = SettingsStore::at(&path);
    let outcome = store
        .load_with_fault(LoadFault { fail_backup: true })
        .expect("recovered");
    assert_eq!(fs::read(&path).expect("original"), body);
    assert!(broken_backups(scratch.path()).is_empty());
    assert!(!outcome.write_policy.is_writable());
    let error = store.save(&outcome).expect_err("readonly");
    assert!(matches!(error, crate::SaveError::ReadOnly { .. }));
    assert_eq!(fs::read(&path).expect("still original"), body);
}

#[test]
fn explicit_save_after_backup_may_replace_the_original() {
    let scratch = Scratch::new("replace-broken");
    let path = scratch.child("settings.toml");
    fs::write(&path, b"[").expect("fixture");
    let store = SettingsStore::at(&path);
    let outcome = store.load().expect("load");
    assert!(matches!(outcome.write_policy, WritePolicy::Writable));
    store.save(&outcome).expect("save defaults");
    let loaded = store.load().expect("reload");
    assert_eq!(loaded.settings, Settings::default());
    assert_eq!(broken_backups(scratch.path()).len(), 1);
}

#[test]
fn save_creates_a_missing_directory() {
    let scratch = Scratch::new("mkdir");
    let path = scratch.child("nested").join("deeper").join("settings.toml");
    let store = SettingsStore::at(&path);
    let outcome = from_toml("").expect("defaults");
    store.save(&outcome).expect("save");
    assert!(path.is_file());
    let loaded = store.load().expect("load");
    assert_eq!(loaded.settings, Settings::default());
    assert!(!loaded.needs_writeback);
}

#[test]
fn save_failure_leaves_the_old_file() {
    let scratch = Scratch::new("save-fail");
    let path = scratch.child("settings.toml");
    let old = b"OLD_COMPLETE_SETTINGS\n";
    fs::write(&path, old).expect("fixture");
    let before = fingerprint(old);
    let store = SettingsStore::at(&path);
    let outcome = from_toml("").expect("defaults");
    let error = store
        .save_with_fault(
            &outcome,
            SaveFault {
                fail_rename: true,
                ..SaveFault::default()
            },
        )
        .expect_err("rename");
    assert!(matches!(error, crate::SaveError::Replace { .. }));
    assert_eq!(fingerprint(&fs::read(&path).expect("old")), before);
    assert_eq!(fs::read(&path).expect("bytes"), old);
    assert!(temp_files(scratch.path()).is_empty());
}

#[test]
fn injected_crash_after_temp_write_does_not_damage_settings() {
    let scratch = Scratch::new("crash");
    let path = scratch.child("settings.toml");
    let old = b"OLD_COMPLETE_SETTINGS\n";
    fs::write(&path, old).expect("fixture");
    let store = SettingsStore::at(&path);
    let outcome = from_toml("").expect("defaults");
    let error = store
        .save_with_fault(
            &outcome,
            SaveFault {
                crash_after_sync: true,
                ..SaveFault::default()
            },
        )
        .expect_err("crash");
    assert!(matches!(error, crate::SaveError::Write { .. }));
    assert_eq!(fs::read(&path).expect("old"), old);
    let temps = temp_files(scratch.path());
    assert_eq!(temps.len(), 1, "{temps:?}");
    let temp_name = temps[0]
        .file_name()
        .and_then(|name| name.to_str())
        .expect("name");
    assert!(temp_name.contains(".tmp-"));
    assert!(!temp_name.contains(".ferrostage.tmp"));
    assert_ne!(fs::read(&temps[0]).expect("temp"), old);
}

#[test]
fn temp_create_failure_does_not_touch_the_old_file() {
    let scratch = Scratch::new("temp-fail");
    let path = scratch.child("settings.toml");
    let old = b"OLD_COMPLETE_SETTINGS\n";
    fs::write(&path, old).expect("fixture");
    let store = SettingsStore::at(&path);
    let outcome = from_toml("").expect("defaults");
    let error = store
        .save_with_fault(
            &outcome,
            SaveFault {
                fail_create_temp: true,
                ..SaveFault::default()
            },
        )
        .expect_err("temp");
    assert!(matches!(error, crate::SaveError::Write { .. }));
    assert_eq!(fs::read(&path).expect("old"), old);
    assert!(temp_files(scratch.path()).is_empty());
}

#[test]
fn parent_file_blocks_directory_creation() {
    let scratch = Scratch::new("parent-file");
    let parent = scratch.child("not-a-directory");
    fs::write(&parent, b"x").expect("file");
    let store = SettingsStore::at(parent.join("settings.toml"));
    let outcome = from_toml("").expect("defaults");
    let error = store.save(&outcome).expect_err("mkdir");
    assert!(matches!(error, crate::SaveError::CreateDir { .. }));
    assert_eq!(fs::read(&parent).expect("parent"), b"x");
}

#[test]
fn startup_fills_missing_sections_and_check_style_load_does_not() {
    let scratch = Scratch::new("fill");
    let path = scratch.child("settings.toml");
    let partial = "schema_version = 1\n\n[timer]\ntarget_minutes = 12\n";
    fs::write(&path, partial).expect("fixture");
    let store = SettingsStore::at(&path);
    let loaded = store.load().expect("load");
    assert!(loaded.needs_writeback);
    assert_eq!(fs::read_to_string(&path).expect("untouched"), partial);
    let session = store.open_for_startup().expect("startup");
    assert!(!session.needs_writeback);
    assert!(!session.dirty);
    assert!(session.save_error.is_none());
    let text = fs::read_to_string(&path).expect("filled");
    assert!(text.contains("[general]"));
    assert!(text.contains("[cache]"));
    let again = store.load().expect("reload");
    assert_eq!(again.settings.timer.target_minutes, 12);
    assert!(!again.needs_writeback);
}

#[test]
fn startup_does_not_rewrite_a_future_file() {
    let scratch = Scratch::new("future-startup");
    let path = scratch.child("settings.toml");
    let body = "schema_version = 9\nkept = 1\n";
    fs::write(&path, body).expect("fixture");
    let before = fs::read(&path).expect("before");
    let session = SettingsStore::at(&path)
        .open_for_startup()
        .expect("startup");
    assert!(!session.write_policy.is_writable());
    assert!(!session.dirty);
    assert_eq!(fs::read(&path).expect("after"), before);
}

#[test]
fn missing_file_is_not_an_error_and_startup_can_create_it() {
    let scratch = Scratch::new("missing");
    let path = scratch.child("settings.toml");
    let store = SettingsStore::at(&path);
    let loaded = store.load().expect("missing");
    assert_eq!(loaded.settings, Settings::default());
    assert!(loaded.needs_writeback);
    assert!(!path.exists());
    let session = store.open_for_startup().expect("create");
    assert!(path.is_file());
    assert!(!session.dirty);
    assert_eq!(store.load().expect("reload").settings, Settings::default());
}

fn dir_names(dir: &Path) -> Vec<String> {
    fs::read_dir(dir)
        .expect("dir")
        .map(|entry| {
            entry
                .expect("entry")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect()
}

fn broken_backups(dir: &Path) -> Vec<std::path::PathBuf> {
    fs::read_dir(dir)
        .expect("dir")
        .map(|entry| entry.expect("entry").path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("settings.toml.broken-"))
        })
        .collect()
}

fn temp_files(dir: &Path) -> Vec<std::path::PathBuf> {
    fs::read_dir(dir)
        .expect("dir")
        .map(|entry| entry.expect("entry").path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.contains(".tmp-"))
        })
        .collect()
}

#[test]
fn interface_fonts_roundtrip_with_unknown_leaves() {
    let scratch = Scratch::new("interface-fonts");
    let path = scratch.child("settings.toml");
    fs::write(
        &path,
        "schema_version = 1\n[general]\nfuture_font_option = 'keep'",
    )
    .unwrap();
    let store = SettingsStore::at(&path);
    let mut session = crate::SettingsSession::from_outcome(store.load().unwrap());
    session.settings.general.ui_font_family = "Missing Family".into();
    session.settings.general.ui_font_size = 21;
    session.note_edit();
    store.save_session(&mut session).unwrap();
    let restored = store.load().unwrap();
    assert_eq!(restored.settings.general.ui_font_family, "Missing Family");
    assert_eq!(restored.settings.general.ui_font_size, 21);
    let saved: toml::Table = fs::read_to_string(&path).unwrap().parse().unwrap();
    assert_eq!(
        saved["general"]["future_font_option"].as_str(),
        Some("keep")
    );
}
