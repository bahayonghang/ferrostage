//! Pure monitor identities and roles; platform owns discovery and confidence.

/// An owned monitor identity, never an enumeration index or window handle.
///
/// The platform reports its source and confidence separately. Even a device
/// interface path is not guaranteed to survive a connection or driver change.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MonitorId(pub String);

/// What a physical display is for.
///
/// `None` means the display is intentionally unused. An unrecognized settings
/// value becomes `None` and is never coerced to `Audience`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MonitorRole {
    /// The presenter window, notes, and controls.
    Presenter,
    /// The audience projection.
    Audience,
    /// Present, but not used for either window.
    None,
}

#[cfg(test)]
mod tests {
    use super::MonitorRole;

    #[test]
    fn roles_are_distinct() {
        assert_ne!(MonitorRole::Presenter, MonitorRole::Audience);
        assert_ne!(MonitorRole::Audience, MonitorRole::None);
        assert_ne!(MonitorRole::Presenter, MonitorRole::None);
    }

    #[test]
    fn core_manifest_does_not_link_io_crates() {
        let manifest = include_str!("../Cargo.toml");
        for forbidden in [
            "gpui",
            "pdfium",
            "lopdf",
            "notify",
            "ferrostage-settings",
            "ferrostage-platform",
            "windows",
            "winit",
        ] {
            assert!(
                !manifest.contains(forbidden),
                "ferrostage-core must not depend on {forbidden}"
            );
        }
    }
}
