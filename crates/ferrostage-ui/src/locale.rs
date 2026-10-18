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
