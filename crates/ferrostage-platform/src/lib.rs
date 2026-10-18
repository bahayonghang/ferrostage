//! OS integration policy.
//!
//! `notify` 8.2.0 (latest stable; 9.0.0 is still a release candidate) watches
//! the open PDF. The watcher debounces for [`HOT_RELOAD_DEBOUNCE`],
//! waits until the file ends with `%%EOF`, then reads a shadow copy. The
//! original path is never rewritten in place.

use std::time::Duration;

#[cfg(any(target_os = "windows", test))]
mod friendly;
mod monitor;
mod reload;
mod topology;
mod watch;
#[cfg(target_os = "windows")]
mod windows;

pub use monitor::{
    IdentityConfidence, IdentitySource, MonitorInfo, PlatformError, ScaleSource, enumerate_monitors,
};
pub use reload::{PdfWatchEvent, PdfWatcher, watch_pdf};
pub use topology::{
    ResolutionSource, ResolvedTopology, TopologyDiff, TopologyWarning, diff, resolve_roles,
};
pub use watch::{TOPOLOGY_POLL_INTERVAL, TopologyEvent, TopologyWatcher, watch_topology};

/// Latest stable `notify` release this crate links.
pub const NOTIFY_VERSION: &str = "8.2.0";

/// Quiet period after the last filesystem event before a reload is attempted.
pub const HOT_RELOAD_DEBOUNCE: Duration = Duration::from_millis(300);

/// PDF end-of-file marker. A reload waits until a write has produced this.
pub const PDF_EOF_MARKER: &[u8] = b"%%EOF";

/// Concrete watcher `notify` selected for this OS.
///
/// `RecommendedWatcher` is a type alias, so this is the platform type
/// (`ReadDirectoryChangesWatcher` on Windows).
#[must_use]
pub fn recommended_watcher_type() -> &'static str {
    std::any::type_name::<notify::RecommendedWatcher>()
}

/// True when `bytes` looks like a finished PDF write.
#[must_use]
pub fn has_pdf_eof(bytes: &[u8]) -> bool {
    bytes[bytes.len().saturating_sub(1024)..]
        .windows(PDF_EOF_MARKER.len())
        .any(|window| window == PDF_EOF_MARKER)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eof_marker_is_required() {
        assert!(!has_pdf_eof(b"%PDF-1.7"));
        assert!(has_pdf_eof(b"%PDF-1.7\n%%EOF\n"));
        assert_eq!(HOT_RELOAD_DEBOUNCE.as_millis(), 300);
        assert_eq!(NOTIFY_VERSION, "8.2.0");
        assert!(recommended_watcher_type().contains("notify"));
        let mut stale = b"%%EOF".to_vec();
        stale.resize(1030, b' ');
        assert!(!has_pdf_eof(&stale));
    }
}
