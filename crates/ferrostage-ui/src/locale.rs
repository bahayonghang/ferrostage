//! Small startup-selected catalog; task09 owns any language settings interface.
use std::{collections::BTreeMap, sync::LazyLock};

static EN: LazyLock<BTreeMap<String, String>> =
    LazyLock::new(|| catalog(include_str!("../../../assets/locales/en-US/app.toml")));
static ZH: LazyLock<BTreeMap<String, String>> =
    LazyLock::new(|| catalog(include_str!("../../../assets/locales/zh-CN/app.toml")));

fn catalog(source: &str) -> BTreeMap<String, String> {
    // These immutable embedded inputs are validated by the catalog contract test.
    toml::from_str(source).expect("valid embedded application locale")
}

#[derive(Clone, Copy)]
pub struct Locale(&'static BTreeMap<String, String>);
impl Locale {
    pub fn new(language: &str) -> Self {
        Self(if language == "en-US" { &EN } else { &ZH })
    }
    pub fn text(self, key: &str) -> &'static str {
        self.0
            .get(key)
            .map(String::as_str)
            .expect("application locale key exists")
    }
    pub fn format(self, key: &str, values: &[(&str, String)]) -> String {
        let mut text = self.text(key).to_string();
        for (name, value) in values {
            text = text.replace(&format!("{{{name}}}"), value);
        }
        text
    }
    pub fn error(self, key: &str, error: impl std::fmt::Display) -> String {
        self.format(key, &[("error", error.to_string())])
    }
    pub fn monitor_error(self, error: &ferrostage_platform::PlatformError) -> String {
        use ferrostage_platform::PlatformError;
        let key = match error {
            PlatformError::UnsupportedPlatform => "monitor-unsupported",
            PlatformError::NativeEnumeration { code, .. } => {
                return self.format("monitor-native-error", &[("code", code.to_string())]);
            }
            PlatformError::IncompleteTopology { .. } => "monitor-incomplete",
            PlatformError::TopologyChanging { .. } => "monitor-changing",
            PlatformError::AmbiguousConnection { .. } => "monitor-ambiguous",
            PlatformError::InvalidGeometry { .. } => "monitor-invalid-geometry",
            PlatformError::WatcherStart { .. } => "monitor-watcher-start",
            PlatformError::WatcherPanicked => "monitor-watcher-panicked",
        };
        self.text(key).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fields(value: &str) -> Vec<&str> {
        let mut result = value
            .split('{')
            .skip(1)
            .filter_map(|part| part.split('}').next())
            .collect::<Vec<_>>();
        result.sort();
        result
    }
    #[test]
    fn monitor_failures_are_localized_without_raw_diagnostics() {
        use ferrostage_platform::PlatformError;
        let failures = [
            (PlatformError::UnsupportedPlatform, "monitor-unsupported"),
            (
                PlatformError::IncompleteTopology {
                    devices: vec!["private-device-path".into()],
                    geometry: Vec::new(),
                },
                "monitor-incomplete",
            ),
            (
                PlatformError::TopologyChanging {
                    before: vec!["private-device-path".into()],
                    after: Vec::new(),
                },
                "monitor-changing",
            ),
            (
                PlatformError::AmbiguousConnection {
                    connection: "private-device-path".into(),
                },
                "monitor-ambiguous",
            ),
            (
                PlatformError::InvalidGeometry {
                    connection: "private-device-path".into(),
                },
                "monitor-invalid-geometry",
            ),
            (
                PlatformError::WatcherStart {
                    message: "private-device-path".into(),
                },
                "monitor-watcher-start",
            ),
            (PlatformError::WatcherPanicked, "monitor-watcher-panicked"),
        ];
        for language in ["zh-CN", "en-US"] {
            let locale = Locale::new(language);
            for (error, key) in &failures {
                let message = locale.monitor_error(error);
                assert_eq!(message, locale.text(key));
                assert!(!message.contains("private-device-path"));
                assert_ne!(message, error.to_string());
            }
            let message = locale.monitor_error(&PlatformError::NativeEnumeration {
                operation: "private-device-path",
                code: 5,
            });
            assert!(message.contains('5'));
            assert!(!message.contains("{code}") && !message.contains("private-device-path"));
            for key in [
                "displays-recovery-wait",
                "displays-recovery-windows",
                "displays-recovery-apply",
            ] {
                assert!(!locale.text(key).is_empty());
            }
            assert!(locale.text("displays-recovery-windows").contains("Win + P"));
            assert!(locale.text("displays-recovery-apply").contains("PDF"));
        }
        assert!(
            Locale::new("zh-CN")
                .text("displays-recovery-windows")
                .contains("设置 > 系统 > 屏幕（显示）> 多显示器")
        );
        assert!(
            Locale::new("zh-CN")
                .text("displays-recovery-apply")
                .contains("应用")
        );
    }

    #[test]
    fn catalogs_have_identical_keys_and_interpolation_fields() {
        assert_eq!(EN.keys().collect::<Vec<_>>(), ZH.keys().collect::<Vec<_>>());
        for (key, value) in EN.iter() {
            assert!(!value.is_empty());
            assert!(!ZH[key].is_empty());
            assert_eq!(fields(value), fields(&ZH[key]), "{key}");
        }
        assert_eq!(Locale::new("en-US").text("start"), "Start presentation");
        assert_eq!(Locale::new("zh-CN").text("start"), "开始演示");
        assert_eq!(
            Locale::new("en-US").format("disabled-label", &[("label", "Open PDF".into())]),
            "Open PDF (unavailable)"
        );
    }
}
