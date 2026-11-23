//! Build metadata for Settings > About. The binary sets it once at startup.
use std::sync::OnceLock;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");
pub const LICENSE: &str = env!("CARGO_PKG_LICENSE");
pub const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");

/// Newest first. The heading is release copy; [`VERSION`] remains the package version.
pub const CHANGELOG: &[(&str, &str)] = &[
    ("1.4.0", "about-changelog-1-4-0"),
    ("1.3.1", "about-changelog-1-3-1"),
    ("1.3.0", "about-changelog-1-3-0"),
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

#[cfg(feature = "gpui")]
impl BuildInfo {
    pub(crate) fn local_time(self, zone: jiff::tz::TimeZone) -> Option<jiff::ZonedDateTime> {
        if self.built_at <= 0 {
            return None;
        }
        let timestamp = jiff::Timestamp::from_second(self.built_at).ok()?;
        Some(timestamp.to_zoned_datetime(zone))
    }
}

static BUILD: OnceLock<BuildInfo> = OnceLock::new();

pub fn set_build_info(info: BuildInfo) {
    let _ = BUILD.set(info);
}

pub fn build_info() -> Option<BuildInfo> {
    BUILD.get().copied()
}
