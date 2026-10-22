//! Build metadata for Settings > About. The binary sets it once at startup.
use std::sync::OnceLock;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");
pub const LICENSE: &str = env!("CARGO_PKG_LICENSE");
pub const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");

/// Newest first. The heading is release copy; [`VERSION`] remains the package version.
pub const CHANGELOG: &[(&str, &str)] = &[
    ("1.2.1", "about-changelog-1-2-1"),
    ("1.2.0", "about-changelog-1-2-0"),
    ("1.1.1", "about-changelog-1-1-1"),
    ("1.1.0", "about-changelog-1-1-0"),
    ("1.0.0", "about-changelog-1-0-0"),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BuildInfo {
    /// Build time in seconds since the Unix epoch; 0 when unknown.
    pub built_at: i64,
}

static BUILD: OnceLock<BuildInfo> = OnceLock::new();

pub fn set_build_info(info: BuildInfo) {
    let _ = BUILD.set(info);
}

pub fn build_info() -> Option<BuildInfo> {
    BUILD.get().copied()
}
