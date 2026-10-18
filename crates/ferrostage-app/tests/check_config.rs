//! `check-config` is read-only. Startup fill runs only without that subcommand.

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::{self, Command};
use std::time::{SystemTime, UNIX_EPOCH};

fn ferrostage() -> &'static str {
    env!("CARGO_BIN_EXE_ferrostage")
}

fn assert_no_startup_error_log(scratch: &Scratch) {
    assert!(
        !scratch.child("startup-error.log").exists(),
        "check-config and successful startup must not write startup-error.log"
    );
}

struct Scratch(PathBuf);

impl Scratch {
    fn new(label: &str) -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let path =
            env::temp_dir().join(format!("ferrostage-app-{label}-{}-{nanos}", process::id()));
        fs::create_dir_all(&path).expect("temp dir");
        Self(path)
    }

    fn child(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn check_config_leaf_fallback_exits_zero_and_does_not_write() {
    let scratch = Scratch::new("leaf");
    let path = scratch.child("settings.toml");
    let body = "schema_version = 1\n\n[timer]\ntarget_minutes = 0\n";
    fs::write(&path, body).expect("fixture");
    let before = fs::read(&path).expect("before");
    let output = Command::new(ferrostage())
        .arg("check-config")
        .arg(&path)
        .output()
        .expect("run");
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("timer.target_minutes"), "{stderr}");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("target_minutes: 45"), "{stdout}");
    assert_eq!(fs::read(&path).expect("after"), before);
    assert!(fs::read_dir(&scratch.0).expect("dir").all(|entry| {
        let name = entry.expect("entry").file_name();
        let name = name.to_string_lossy();
        !name.contains(".tmp-") && !name.contains(".broken-")
    }));
    assert_no_startup_error_log(&scratch);
}

#[test]
fn check_config_syntax_error_exits_one_without_backup() {
    let scratch = Scratch::new("syntax");
    let path = scratch.child("settings.toml");
    let other = scratch.child("other.toml");
    fs::write(&path, b"this is [ not toml").expect("fixture");
    let before = fs::read(&path).expect("before");
    let output = Command::new(ferrostage())
        .arg("--config")
        .arg(&other)
        .arg("check-config")
        .arg(&path)
        .output()
        .expect("run");
    assert_eq!(
        output.status.code(),
        Some(1),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(fs::read(&path).expect("after"), before);
    assert!(!other.exists());
    assert!(fs::read_dir(&scratch.0).expect("dir").all(|entry| {
        let name = entry.expect("entry").file_name();
        !name.to_string_lossy().contains(".broken-")
    }));
    assert_no_startup_error_log(&scratch);
}

#[test]
fn missing_config_exits_one() {
    let scratch = Scratch::new("missing");
    let path = scratch.child("absent.toml");
    let output = Command::new(ferrostage())
        .arg("check-config")
        .arg(&path)
        .output()
        .expect("run");
    assert_eq!(output.status.code(), Some(1));
    assert!(!path.exists());
    assert_no_startup_error_log(&scratch);
}

#[test]
#[cfg(not(feature = "gpui"))]
fn startup_config_fills_missing_sections() {
    let scratch = Scratch::new("startup");
    let path = scratch.child("settings.toml");
    fs::write(
        &path,
        "schema_version = 1\n\n[timer]\ntarget_minutes = 12\n",
    )
    .expect("fixture");
    let output = Command::new(ferrostage())
        .arg("--config")
        .arg(&path)
        .output()
        .expect("run");
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let text = fs::read_to_string(&path).expect("filled");
    assert!(text.contains("[general]"), "{text}");
    assert!(text.contains("target_minutes = 12"), "{text}");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("skeleton"), "{stdout}");
    assert_no_startup_error_log(&scratch);
}

#[test]
fn startup_directory_config_appends_startup_error_log() {
    let scratch = Scratch::new("startup-dir");
    let config_dir = scratch.child("settings-dir");
    fs::create_dir_all(&config_dir).expect("config dir");
    let output = Command::new(ferrostage())
        .arg("--config")
        .arg(&config_dir)
        .output()
        .expect("run");
    assert_eq!(
        output.status.code(),
        Some(1),
        "stdout {}\nstderr {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    let reported = stderr
        .lines()
        .find(|line| line.starts_with("ferrostage:"))
        .expect("stderr line");
    let log = fs::read_to_string(scratch.child("startup-error.log")).expect("startup log");
    assert!(
        log.lines().any(|line| line == reported),
        "stderr {reported}\nlog {log}"
    );
    assert!(
        fs::read_dir(&config_dir)
            .expect("config dir")
            .next()
            .is_none()
    );
}
