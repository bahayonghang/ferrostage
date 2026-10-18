//! Human-facing monitor names. Identity stays on the device path.

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ResolvedName {
    pub name: String,
    pub built_in: bool,
}

/// Prefer a model token from Windows' friendly name. A miss keeps the driver
/// description and never fails enumeration.
pub(crate) fn human_name(driver: &str, friendly_raw: Option<&str>) -> ResolvedName {
    let Some(raw) = friendly_raw
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return ResolvedName {
            name: driver.to_owned(),
            built_in: false,
        };
    };
    let resolved = resolve_indirect(raw);
    let built_in = is_built_in(&resolved);
    if let Some(model) = model_token(&resolved) {
        return ResolvedName {
            name: model,
            built_in,
        };
    }
    if resolved.is_empty() || is_generic_phrase(&resolved) {
        return ResolvedName {
            name: driver.to_owned(),
            built_in,
        };
    }
    ResolvedName {
        name: resolved,
        built_in,
    }
}

/// `\\?\DISPLAY#{hardware}#{instance}#{guid}` to the PnP enum key.
pub(crate) fn display_enum_subkey(path: &str) -> Option<String> {
    let marker = "display#";
    let start = path.to_ascii_lowercase().find(marker)? + marker.len();
    let rest = path.get(start..)?;
    let hash = rest.find('#')?;
    let hardware = &rest[..hash];
    let after = rest.get(hash + 1..)?;
    let end = after.find("#{")?;
    let instance = &after[..end];
    if !is_instance_token(hardware) || !is_instance_token(instance) {
        return None;
    }
    Some(format!(
        r"SYSTEM\CurrentControlSet\Enum\DISPLAY\{hardware}\{instance}"
    ))
}

fn resolve_indirect(raw: &str) -> String {
    let raw = raw.trim();
    let Some(rest) = raw.strip_prefix('@') else {
        return raw.to_owned();
    };
    let mut sections = rest.split(';');
    let _resource = sections.next();
    let fallback = sections.next().unwrap_or("").trim();
    if fallback.is_empty() {
        return String::new();
    }
    if let Some(argument) = sections.next() {
        let token = strip_parens(argument.trim());
        if fallback.contains("%1") && !token.is_empty() {
            return fallback.replace("%1", &token);
        }
    }
    fallback.to_owned()
}

fn strip_parens(value: &str) -> String {
    let value = value.trim();
    if let Some(inner) = value
        .strip_prefix('(')
        .and_then(|value| value.strip_suffix(')'))
    {
        return inner.trim().to_owned();
    }
    value.to_owned()
}

fn model_token(resolved: &str) -> Option<String> {
    let resolved = resolved.trim();
    let open = resolved.rfind('(')?;
    let close = resolved.rfind(')')?;
    if close != resolved.len().saturating_sub(1) || close <= open {
        return None;
    }
    let inner = resolved[open + 1..close].trim();
    if inner.is_empty() || inner.contains('%') || is_generic_token(inner) {
        None
    } else {
        Some(inner.to_owned())
    }
}

fn is_built_in(resolved: &str) -> bool {
    let lower = resolved.to_ascii_lowercase();
    lower.contains("integrated") || resolved.contains("集成")
}

fn is_generic_phrase(value: &str) -> bool {
    let value = value.trim().trim_end_matches('.').to_ascii_lowercase();
    matches!(
        value.as_str(),
        "generic pnp monitor"
            | "generic monitor"
            | "generic non-pnp monitor"
            | "integrated monitor"
            | "pnp monitor"
            | "default monitor"
            | "plug and play monitor"
    ) || (value.contains("通用") && value.contains("监视"))
}

fn is_generic_token(value: &str) -> bool {
    let value = value.trim().to_ascii_lowercase();
    value.is_empty()
        || value.contains("generic")
        || value.contains("pnp")
        || value == "monitor"
        || value.contains("通用")
}

fn is_instance_token(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.chars().all(|c| c.is_ascii_alphanumeric() || c == '&')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn friendly_name_keeps_model_tokens_and_drops_generic_wrappers() {
        let external = human_name(
            "Generic PnP Monitor",
            Some(r"@System32\drivers\dxgkrnl.sys,#303;Generic Monitor (%1);(BKX270)"),
        );
        assert_eq!(
            external,
            ResolvedName {
                name: "BKX270".into(),
                built_in: false,
            }
        );
        let panel = human_name(
            "Generic PnP Monitor",
            Some(r"@System32\drivers\dxgkrnl.sys,#304;Integrated Monitor (%1);(NE160QDM-NM9)"),
        );
        assert_eq!(
            panel,
            ResolvedName {
                name: "NE160QDM-NM9".into(),
                built_in: true,
            }
        );
        let unnamed = human_name(
            "Generic PnP Monitor",
            Some(r"@System32\drivers\dxgkrnl.sys,#300;Generic Monitor"),
        );
        assert_eq!(
            unnamed,
            ResolvedName {
                name: "Generic PnP Monitor".into(),
                built_in: false,
            }
        );
        let plain = human_name("Generic PnP Monitor", Some("LG ULTRAFINE"));
        assert_eq!(plain.name, "LG ULTRAFINE");
        assert!(!plain.built_in);
        let missing = human_name("Generic PnP Monitor", None);
        assert_eq!(missing.name, "Generic PnP Monitor");
        assert!(!missing.built_in);
        let broken = human_name("Dell U2720Q", Some("@monitor.inf"));
        assert_eq!(broken.name, "Dell U2720Q");
    }

    #[test]
    fn device_path_maps_to_one_enum_key() {
        let path =
            r"\\?\DISPLAY#BKX2700#5&16631d0&0&UID8451#{e6f07b5f-ee97-4a90-b076-33f57bf4eaa7}";
        assert_eq!(
            display_enum_subkey(path).as_deref(),
            Some(r"SYSTEM\CurrentControlSet\Enum\DISPLAY\BKX2700\5&16631d0&0&UID8451")
        );
        assert!(display_enum_subkey(r"\\?\DISPLAY#BKX2700#5&1..#{guid}").is_none());
        assert!(display_enum_subkey("").is_none());
        assert!(display_enum_subkey(r"\\?\DISPLAY#bad/id#instance#{guid}").is_none());
    }
}
