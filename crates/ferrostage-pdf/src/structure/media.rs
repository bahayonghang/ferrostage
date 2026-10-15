use super::*;
mod rich;
mod screen;

pub(super) fn extract<'a>(
    p: &mut Parser<'a>,
    a: &'a Dictionary,
    object: &'a Object,
    page: &'a Dictionary,
    geometry: PageGeometry,
) -> Option<MediaAnnotation> {
    let kind = match p.name(a, b"Subtype")? {
        b"Movie" => MediaKind::Movie,
        b"Screen" => MediaKind::Screen,
        b"RichMedia" => MediaKind::RichMedia,
        _ => return None,
    };
    let rect = p
        .field(a, b"Rect")
        .and_then(|o| o.as_array().ok())
        .filter(|a| a.len() == 4)
        .and_then(|a| {
            Some([
                number(&a[0])? as f32,
                number(&a[1])? as f32,
                number(&a[2])? as f32,
                number(&a[3])? as f32,
            ])
        })
        .and_then(|r| geometry.normalize_rect(r));
    if rect.is_none() {
        p.warn("Rect", DiagnosticKind::Malformed);
    }
    let mut item = MediaAnnotation {
        kind,
        rect,
        source: None,
        start: None,
        duration: None,
        repeat: None,
        autostart: None,
        play_on_activation: None,
    };
    match kind {
        MediaKind::Movie => movie(p, a, object, page, &mut item),
        MediaKind::Screen => screen::extract(p, a, object, &mut item),
        MediaKind::RichMedia => rich::extract(p, a, &mut item),
    }
    Some(item)
}
fn movie<'a>(
    p: &mut Parser<'a>,
    a: &'a Dictionary,
    object: &'a Object,
    page: &'a Dictionary,
    item: &mut MediaAnnotation,
) {
    let file = p.dict(a, b"Movie").and_then(|m| m.get(b"F").ok());
    item.source = source::extract(p, file, "Movie/F");
    let base = p.dict(a, b"A");
    let action = movie_action(p, a, object, page);
    let play = action.filter(|d| !d.has(b"Operation") || p.name(d, b"Operation") == Some(b"Play"));
    item.autostart = play.map(|_| true);
    if item.autostart.is_none() {
        p.warn("Movie page-open/visible action", DiagnosticKind::Missing);
    }
    let value = |key: &[u8]| {
        play.and_then(|d| p.field(d, key))
            .or_else(|| base.and_then(|d| p.field(d, key)))
    };
    let start = value(b"Start");
    let duration = value(b"Duration");
    let mode = value(b"Mode");
    item.start = movie_time(p, start, "A/Start");
    item.duration = movie_time(p, duration, "A/Duration");
    item.repeat = match mode.and_then(|o| o.as_name().ok()) {
        Some(b"Repeat") => Some(MediaRepeat::Forever),
        Some(b"Once" | b"Open") => Some(MediaRepeat::Count(1.0)),
        Some(_) => {
            p.warn("A/Mode", DiagnosticKind::Unsupported);
            None
        }
        None => {
            p.warn("A/Mode", DiagnosticKind::Missing);
            None
        }
    };
    if [base, play]
        .into_iter()
        .flatten()
        .any(|d| d.has(b"FWScale") || d.has(b"Rate"))
    {
        p.warn("A/Rate or FWScale", DiagnosticKind::Unsupported);
    }
}
fn ticks(o: &Object) -> Option<i64> {
    match o {
        Object::Integer(v) if *v >= 0 => Some(*v),
        Object::String(b, _) if b.len() == 8 => {
            let n = i64::from_be_bytes(b.as_slice().try_into().ok()?);
            (n >= 0).then_some(n)
        }
        _ => None,
    }
}
fn movie_time(p: &mut Parser<'_>, o: Option<&Object>, field: &str) -> Option<MediaTime> {
    let Some(o) = o else {
        p.warn(field, DiagnosticKind::Missing);
        return None;
    };
    if let Some(value) = ticks(o) {
        p.warn(field, DiagnosticKind::UnscaledTime);
        return Some(MediaTime::Unscaled(value));
    }
    if let Ok(array) = o.as_array()
        && array.len() == 2
        && let (Some(value), Ok(scale)) = (ticks(&array[0]), array[1].as_i64())
        && scale > 0
    {
        return Some(MediaTime::Seconds(value as f64 / scale as f64));
    }
    p.warn(field, DiagnosticKind::Malformed);
    None
}
fn timespan<'a>(p: &Parser<'a>, dictionary: &'a Dictionary) -> Option<MediaTime> {
    if p.name(dictionary, b"S") != Some(b"S") {
        return None;
    }
    p.number(dictionary, b"V")
        .filter(|n| *n >= 0.0)
        .map(MediaTime::Seconds)
}
fn preference<'a>(p: &Parser<'a>, dictionary: &'a Dictionary, key: &[u8]) -> Option<&'a Object> {
    p.dict(dictionary, b"MH")
        .and_then(|d| p.field(d, key))
        .or_else(|| p.dict(dictionary, b"BE").and_then(|d| p.field(d, key)))
}

// Movie actions may be attached to another annotation (Beamer uses a Widget).
// Title targets are accepted only when unique among Movie annotations on this page.
fn movie_action<'a>(
    p: &mut Parser<'a>,
    target: &'a Dictionary,
    object: &'a Object,
    page: &'a Dictionary,
) -> Option<&'a Dictionary> {
    let annotations = p.field(page, b"Annots").and_then(|o| o.as_array().ok())?;
    let title = p.field(target, b"T").and_then(text);
    let unique_title = title.as_ref().is_some_and(|title| {
        annotations
            .iter()
            .filter(|o| {
                resolve(p.document, o)
                    .and_then(|o| o.as_dict().ok())
                    .is_some_and(|d| {
                        p.name(d, b"Subtype") == Some(b"Movie")
                            && p.field(d, b"T").and_then(text).as_ref() == Some(title)
                    })
            })
            .count()
            == 1
    });
    let mut actions = Vec::new();
    if let Some(action) = p.dict(page, b"AA").and_then(|aa| p.dict(aa, b"O")) {
        actions.push(action);
    }
    for annotation in annotations {
        if let Some(aa) = resolve(p.document, annotation)
            .and_then(|o| o.as_dict().ok())
            .and_then(|d| p.dict(d, b"AA"))
        {
            for key in [b"PO", b"PV"] {
                if let Some(action) = p.dict(aa, key) {
                    actions.push(action);
                }
            }
        }
    }
    let mut selected = None;
    for action in actions {
        if p.name(action, b"S") != Some(b"Movie") {
            continue;
        }
        let by_ref = action.get(b"Annotation").ok() == Some(object);
        let by_title = title.is_some() && p.field(action, b"T").and_then(text) == title;
        if by_title && !unique_title {
            p.warn(
                "Movie action/T (ambiguous title)",
                DiagnosticKind::Unsupported,
            );
            return None;
        }
        if !by_ref && !by_title {
            continue;
        }
        if action.has(b"Next")
            || (action.has(b"Annotation") && action.has(b"T"))
            || selected.is_some()
        {
            p.warn(
                "Movie activation action chain or multiple targets",
                DiagnosticKind::Unsupported,
            );
            return None;
        }
        if action.has(b"Operation")
            && !matches!(
                p.name(action, b"Operation"),
                Some(b"Play" | b"Stop" | b"Pause" | b"Resume")
            )
        {
            p.warn("Movie action/Operation", DiagnosticKind::Malformed);
            return None;
        }
        selected = Some(action);
    }
    selected
}
