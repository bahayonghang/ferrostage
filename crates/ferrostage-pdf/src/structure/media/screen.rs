use super::*;
pub(super) fn extract<'a>(
    p: &mut Parser<'a>,
    a: &'a Dictionary,
    object: &'a Object,
    item: &mut MediaAnnotation,
) {
    let aa = p.dict(a, b"AA");
    let automatic = aa.and_then(|aa| p.dict(aa, b"PO").or_else(|| p.dict(aa, b"PV")));
    let action = automatic.or_else(|| p.dict(a, b"A"));
    let Some(action) = action else {
        p.warn("A or AA/PO/PV", DiagnosticKind::Missing);
        return;
    };
    if p.name(action, b"S") != Some(b"Rendition")
        || p.field(action, b"OP").and_then(|o| o.as_i64().ok()) != Some(0)
        || action.get(b"AN").ok() != Some(object)
    {
        p.warn("A/S,OP,AN", DiagnosticKind::Unsupported);
        return;
    }
    if action.has(b"JS") || action.has(b"Next") {
        p.warn("A/JS or Next", DiagnosticKind::Unsupported);
        return;
    }
    let Some(r) = p
        .dict(action, b"R")
        .filter(|r| p.name(r, b"S") == Some(b"MR"))
    else {
        p.warn("A/R/S", DiagnosticKind::Unsupported);
        return;
    };
    let mut clip = p.dict(r, b"C");
    if let Some(section) = clip.filter(|c| p.name(c, b"S") == Some(b"MCS")) {
        item.start = preference(p, section, b"B")
            .and_then(|o| o.as_dict().ok())
            .filter(|d| p.name(d, b"S") == Some(b"T"))
            .and_then(|d| p.dict(d, b"T"))
            .and_then(|t| timespan(p, t));
        if item.start.is_none() {
            p.warn("A/R/C/MH|BE/B/T", DiagnosticKind::Unsupported);
        }
        if preference(p, section, b"E").is_some() {
            p.warn("A/R/C/MH|BE/E", DiagnosticKind::Unsupported);
        }
        clip = p.dict(section, b"D");
    }
    let data = clip.filter(|c| p.name(c, b"S") == Some(b"MCD"));
    if data.is_none() {
        p.warn("A/R/C/S", DiagnosticKind::Unsupported);
    }
    let file = data.and_then(|d| p.field(d, b"D"));
    if file
        .and_then(|o| o.as_dict().ok())
        .is_some_and(|d| p.name(d, b"Type") == Some(b"Filespec"))
    {
        item.source = source::extract(p, file, "A/R/C[/D]/D");
    } else {
        p.warn("A/R/C[/D]/D/Type", DiagnosticKind::Unsupported);
    }
    if let Some(parameters) = p.dict(r, b"P") {
        item.duration = preference(p, parameters, b"D")
            .and_then(|o| o.as_dict().ok())
            .filter(|d| p.name(d, b"S") == Some(b"T"))
            .and_then(|d| p.dict(d, b"T"))
            .and_then(|t| timespan(p, t));
        if item.duration.is_none() {
            p.warn("A/R/P/MH|BE/D", DiagnosticKind::Unsupported);
        }
        item.repeat = preference(p, parameters, b"RC")
            .and_then(number)
            .filter(|v| *v >= 0.0)
            .map(|v| {
                if v == 0.0 {
                    MediaRepeat::Forever
                } else {
                    MediaRepeat::Count(v)
                }
            });
        if item.repeat.is_none() {
            p.warn("A/R/P/MH|BE/RC", DiagnosticKind::Missing);
        }
        item.play_on_activation = preference(p, parameters, b"A").and_then(|o| o.as_bool().ok());
        if item.play_on_activation.is_none() {
            p.warn("A/R/P/MH|BE/A", DiagnosticKind::Missing);
        }
    } else {
        p.warn("A/R/P", DiagnosticKind::Missing);
    }
    item.autostart = if automatic.is_some() {
        item.play_on_activation
    } else {
        None
    };
    if item.autostart.is_none() {
        p.warn("AA/PO/PV", DiagnosticKind::Missing);
    }
}
