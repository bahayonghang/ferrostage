use super::*;
use std::collections::HashSet;
fn assets<'a>(
    document: &'a Document,
    node: &'a Dictionary,
    seen: &mut HashSet<lopdf::ObjectId>,
    out: &mut Vec<&'a Object>,
) -> Option<()> {
    if let Ok(names) = node.get(b"Names") {
        let array = resolve(document, names)?.as_array().ok()?;
        if array.len() % 2 != 0 {
            return None;
        }
        for pair in array.as_chunks::<2>().0 {
            out.push(&pair[1]);
        }
    }
    if let Ok(kids) = node.get(b"Kids") {
        for child in resolve(document, kids)?.as_array().ok()? {
            let id = child.as_reference().ok()?;
            if !seen.insert(id) {
                return None;
            }
            assets(document, document.get_dictionary(id).ok()?, seen, out)?;
        }
    }
    Some(())
}
pub(super) fn extract<'a>(p: &mut Parser<'a>, a: &'a Dictionary, item: &mut MediaAnnotation) {
    let activation = p
        .dict(a, b"RichMediaSettings")
        .and_then(|s| p.dict(s, b"Activation"));
    item.autostart = match activation.and_then(|d| p.name(d, b"Condition")) {
        Some(b"PO" | b"PV") => Some(true),
        Some(b"XA") => Some(false),
        _ => {
            p.warn(
                "RichMediaSettings/Activation/Condition",
                DiagnosticKind::Missing,
            );
            None
        }
    };
    let Some(content) = p.dict(a, b"RichMediaContent") else {
        p.warn("RichMediaContent", DiagnosticKind::Missing);
        return;
    };
    let configs = p
        .field(content, b"Configurations")
        .and_then(|o| o.as_array().ok());
    let requested = activation.and_then(|a| a.get(b"Configuration").ok());
    let selected = match (requested, configs) {
        (Some(r), Some(configs)) if configs.contains(r) => Some(r),
        (None, Some(configs)) => configs.first(),
        _ => None,
    };
    let Some(config) = selected
        .and_then(|o| resolve(p.document, o))
        .and_then(|o| o.as_dict().ok())
    else {
        p.warn("RichMediaContent/Configurations", DiagnosticKind::Malformed);
        return;
    };
    let instances = p
        .field(config, b"Instances")
        .and_then(|o| o.as_array().ok());
    // A single native Video/Sound asset is the supported subset; never guess an asset in Flash scenes.
    let Some(instance) = instances
        .filter(|i| i.len() == 1)
        .and_then(|i| resolve(p.document, &i[0]))
        .and_then(|o| o.as_dict().ok())
        .filter(|d| matches!(p.name(d, b"Subtype"), Some(b"Video" | b"Sound")))
    else {
        p.warn(
            "RichMediaContent/Configurations/Instances",
            DiagnosticKind::Unsupported,
        );
        return;
    };
    let asset = instance.get(b"Asset").ok();
    let mut available = Vec::new();
    let valid = p
        .dict(content, b"Assets")
        .and_then(|d| assets(p.document, d, &mut HashSet::new(), &mut available))
        .is_some();
    if !valid || !asset.is_some_and(|a| available.contains(&a)) {
        p.warn("RichMediaContent/Assets", DiagnosticKind::Malformed);
        return;
    }
    item.source = source::extract(p, asset, "RichMediaContent/Configurations/Instances/Asset");
    p.warn(
        "RichMedia timing and repeat (Flash Params unsupported)",
        DiagnosticKind::Unsupported,
    );
    if activation.is_some_and(|a| a.has(b"Scripts")) || instance.has(b"Params") {
        p.warn("RichMedia Scripts/Params", DiagnosticKind::Unsupported);
    }
}
