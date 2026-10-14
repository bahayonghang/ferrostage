//! OS integration policy.
//!
//! `notify` 8.2.0 (latest stable; 9.0.0 is still a release candidate) watches
//! the open PDF. The watcher debounces for [`HOT_RELOAD_DEBOUNCE`],
//! waits until the file ends with `%%EOF`, then reads a shadow copy. The
//! original path is never rewritten in place.

use std::time::Duration;

/// Latest stable `notify` release this crate links.
pub const NOTIFY_VERSION: &str = "8.2.0";

/// Quiet period after the last filesystem event before a reload is attempted.
pub const HOT_RELOAD_DEBOUNCE: Duration = Duration::from_millis(300);

/// Suffix of the temporary file used when FerroStage itself writes a PDF.
///
/// Write this sibling, validate it, then atomically rename it over the
/// destination. Never open the destination with truncate-and-write.
pub const SHADOW_TMP_SUFFIX: &str = ".ferrostage.tmp";

/// PDF end-of-file marker. A reload waits until a write has produced this.
pub const PDF_EOF_MARKER: &[u8] = b"%%EOF";

/// Concrete watcher `notify` selected for this OS.
///
/// `RecommendedWatcher` is a type alias, so this is the platform type
/// (`ReadDirectoryChangesWatcher` on Windows). The hot-reload loop is not
/// started in the skeleton; naming the type keeps the crate linked.
#[must_use]
pub fn recommended_watcher_type() -> &'static str {
    std::any::type_name::<notify::RecommendedWatcher>()
}

/// True when `bytes` looks like a finished PDF write.
#[must_use]
pub fn has_pdf_eof(bytes: &[u8]) -> bool {
    bytes
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
        assert_eq!(SHADOW_TMP_SUFFIX, ".ferrostage.tmp");
    }
}
