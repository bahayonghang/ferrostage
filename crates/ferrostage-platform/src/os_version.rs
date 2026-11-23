//! Windows build-number detection used to gate native window blur safely.

/// First Windows 11 build number; older Windows 10 builds (`< 22000`) exhibit
/// DWM drag lag under `ACCENT_ENABLE_ACRYLICBLURBEHIND`.
pub const WINDOWS_11_MIN_BUILD: u32 = 22_000;

/// Parse a Windows `CurrentBuildNumber` string (`> 0`, ASCII digits only after trimming).
#[must_use]
pub fn parse_windows_build_number(raw: &str) -> Option<u32> {
    let trimmed = raw.trim();
    if trimmed.is_empty() || !trimmed.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    trimmed.parse::<u32>().ok().filter(|&build| build > 0)
}

/// Return whether the given Windows build number permits native window blur.
#[must_use]
pub fn windows_build_allows_native_blur(build: Option<u32>) -> bool {
    build.is_some_and(|b| b >= WINDOWS_11_MIN_BUILD)
}

#[cfg(target_os = "windows")]
fn read_windows_build_number() -> Option<u32> {
    use winsafe::co;

    let value = winsafe::HKEY::LOCAL_MACHINE
        .RegGetValue(
            Some(r"SOFTWARE\Microsoft\Windows NT\CurrentVersion"),
            Some("CurrentBuildNumber"),
            co::RRF::RT_REG_SZ | co::RRF::RT_REG_EXPAND_SZ,
        )
        .ok()?;
    match value {
        winsafe::RegistryValue::Sz(text) | winsafe::RegistryValue::ExpandSz(text) => {
            parse_windows_build_number(&text)
        }
        _ => None,
    }
}

/// Read and process-cache the host Windows `CurrentBuildNumber` without `unsafe`.
#[cfg(target_os = "windows")]
#[must_use]
pub fn windows_build_number() -> Option<u32> {
    static BUILD: std::sync::OnceLock<Option<u32>> = std::sync::OnceLock::new();
    *BUILD.get_or_init(read_windows_build_number)
}

/// Read and process-cache the host Windows `CurrentBuildNumber` without `unsafe`.
#[cfg(not(target_os = "windows"))]
#[must_use]
pub fn windows_build_number() -> Option<u32> {
    None
}

/// Return whether this host supports Windows native window blur without drag stutter.
#[cfg(target_os = "windows")]
#[must_use]
pub fn host_supports_windows_native_blur() -> bool {
    windows_build_allows_native_blur(windows_build_number())
}

/// Return whether this host supports Windows native window blur without drag stutter.
#[cfg(not(target_os = "windows"))]
#[must_use]
pub fn host_supports_windows_native_blur() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_build_numbers_and_applies_win11_threshold() {
        for (raw, expected_build, expected_blur) in [
            ("18362", Some(18_362), false),
            ("19045", Some(19_045), false),
            ("21999", Some(21_999), false),
            (" 19045 ", Some(19_045), false),
            ("22000", Some(22_000), true),
            ("22621", Some(22_621), true),
            ("26300", Some(26_300), true),
            (" 22621 ", Some(22_621), true),
            ("", None, false),
            ("   ", None, false),
            ("0", None, false),
            ("+22000", None, false),
            ("-1", None, false),
            ("19045.1", None, false),
            ("abc", None, false),
        ] {
            let parsed = parse_windows_build_number(raw);
            assert_eq!(parsed, expected_build, "raw={raw:?}");
            assert_eq!(
                windows_build_allows_native_blur(parsed),
                expected_blur,
                "raw={raw:?}"
            );
        }

        assert!(!windows_build_allows_native_blur(None));
        assert!(!windows_build_allows_native_blur(Some(18_362)));
        assert!(!windows_build_allows_native_blur(Some(19_045)));
        assert!(!windows_build_allows_native_blur(Some(21_999)));
        assert!(windows_build_allows_native_blur(Some(22_000)));
        assert!(windows_build_allows_native_blur(Some(22_621)));
        assert!(windows_build_allows_native_blur(Some(26_300)));
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn windows_host_build_number_smoke_is_cached_and_consistent() {
        let first = windows_build_number();
        let second = windows_build_number();
        assert_eq!(first, second);
        let build = first.expect("Windows CurrentBuildNumber should be readable");
        assert!(build >= 18_362, "unexpected Windows build {build}");
        assert_eq!(
            host_supports_windows_native_blur(),
            windows_build_allows_native_blur(first)
        );
    }
}
