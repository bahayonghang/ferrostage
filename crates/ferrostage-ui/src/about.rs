//! Build metadata for Settings > About. The binary sets it once at startup.
use std::sync::OnceLock;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");
pub const LICENSE: &str = env!("CARGO_PKG_LICENSE");
pub const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");

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
