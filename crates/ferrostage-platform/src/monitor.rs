use ferrostage_core::MonitorId;
use thiserror::Error;

/// What actually supplied an identity. No EDID claim is made by this adapter.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IdentitySource {
    DeviceInterfacePath,
    NameAndResolution,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IdentityConfidence {
    /// A Windows device interface, potentially changed by ports or drivers.
    ConnectionScoped,
    /// Name and current resolution; collisions require manual rebinding.
    Degraded,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScaleSource {
    /// Per-monitor DESKTOPHORZRES / HORZRES in the calling DPI context.
    /// This is not a measured effective DPI. In a DPI-aware context a value of
    /// 1 can mean already-physical coordinates, not a 100% desktop setting.
    /// Validate with the actual UI context before physical/logical mapping.
    GdiDeviceCapsRatio,
}

/// Owned OS output. Geometry is in physical desktop pixels, never UI points.
#[derive(Clone, Debug, PartialEq)]
pub struct MonitorInfo {
    pub id: MonitorId,
    pub name: String,
    /// GDI connection name for this enumeration, not a persistent identity.
    pub connection_name: String,
    pub native_size: (u32, u32),
    pub position: (i32, i32),
    pub scale_factor: f32,
    pub scale_source: ScaleSource,
    pub rotation_degrees: u16,
    pub is_primary: bool,
    pub identity_source: IdentitySource,
    pub identity_confidence: IdentityConfidence,
    /// Windows marked this panel as an integrated monitor. A false value does
    /// not prove the panel is external.
    pub built_in: bool,
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum PlatformError {
    #[error("monitor discovery is not implemented on this platform")]
    UnsupportedPlatform,
    #[error("{operation} failed with Windows error {code}")]
    NativeEnumeration { operation: &'static str, code: u32 },
    #[error("device and geometry connection sets disagree")]
    IncompleteTopology {
        devices: Vec<String>,
        geometry: Vec<String>,
    },
    #[error("connection {connection} does not identify exactly one display")]
    AmbiguousConnection { connection: String },
    #[error("invalid monitor geometry or scale for {connection}")]
    InvalidGeometry { connection: String },
    #[error("could not start topology watcher: {message}")]
    WatcherStart { message: String },
    #[error("topology watcher panicked")]
    WatcherPanicked,
}

/// Enumerate a complete topology or return a typed failure. Empty is valid.
pub fn enumerate_monitors() -> Result<Vec<MonitorInfo>, PlatformError> {
    #[cfg(target_os = "windows")]
    {
        crate::windows::enumerate()
    }
    #[cfg(not(target_os = "windows"))]
    {
        Err(PlatformError::UnsupportedPlatform)
    }
}

#[cfg(any(target_os = "windows", test))]
pub(crate) fn identity(
    path: &str,
    name: &str,
    size: (u32, u32),
) -> (MonitorId, IdentitySource, IdentityConfidence) {
    if path.is_empty() {
        (
            MonitorId(format!(
                "name:{}|{}x{}",
                name.to_lowercase(),
                size.0,
                size.1
            )),
            IdentitySource::NameAndResolution,
            IdentityConfidence::Degraded,
        )
    } else {
        (
            MonitorId(format!("device:{}", path.to_lowercase())),
            IdentitySource::DeviceInterfacePath,
            IdentityConfidence::ConnectionScoped,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_is_owned_case_normalized_and_honestly_graded() {
        let a = identity("PATH-A", "same", (1920, 1080));
        assert_eq!(a, identity("path-a", "renamed", (1080, 1920)));
        assert_eq!(a.2, IdentityConfidence::ConnectionScoped);
        let fallback = identity("", "same", (1920, 1080));
        assert_eq!(fallback.1, IdentitySource::NameAndResolution);
        assert_eq!(fallback.2, IdentityConfidence::Degraded);
        assert_eq!(fallback, identity("", "SAME", (1920, 1080)));
        assert_ne!(fallback, identity("", "same", (1080, 1920)));
    }
}
