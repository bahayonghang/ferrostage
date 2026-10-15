use super::*;
const MAX_EMBEDDED_BYTES: usize = 64 * 1024 * 1024;

pub(super) fn extract(
    parser: &mut Parser<'_>,
    object: Option<&Object>,
    field: &str,
) -> Option<MediaSource> {
    let Some(object) = object.and_then(|o| resolve(parser.document, o)) else {
        parser.warn(field, DiagnosticKind::Missing);
        return None;
    };
    if let Some(path) = text(object) {
        return Some(MediaSource::External(path));
    }
    let Ok(dict) = object.as_dict() else {
        parser.warn(field, DiagnosticKind::Unsupported);
        return None;
    };
    let name = dict
        .get(b"UF")
        .or_else(|_| dict.get(b"F"))
        .ok()
        .and_then(|o| resolve(parser.document, o))
        .and_then(text);
    if let Ok(ef) = dict.get(b"EF")
        && !matches!(resolve(parser.document, ef), Some(Object::Null))
    {
        let Some(ef) = resolve(parser.document, ef).and_then(|o| o.as_dict().ok()) else {
            parser.warn(field, DiagnosticKind::Malformed);
            return None;
        };
        let stream = ef
            .get(b"UF")
            .or_else(|_| ef.get(b"F"))
            .ok()
            .and_then(|o| resolve(parser.document, o))
            .and_then(|o| o.as_stream().ok());
        match stream.and_then(|s| s.decompressed_content_with_limit(MAX_EMBEDDED_BYTES).ok()) {
            Some(bytes) => return Some(MediaSource::Embedded { name, bytes }),
            None => {
                parser.warn(field, DiagnosticKind::Malformed);
                return None;
            }
        }
    }
    match name {
        Some(name) => Some(MediaSource::External(name)),
        None => {
            parser.warn(field, DiagnosticKind::Missing);
            None
        }
    }
}
