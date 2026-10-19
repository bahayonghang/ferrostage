//! Recognize bounded action metadata; never inspect or execute script streams.
use super::*;

fn unsupported_action(document: &Document, root: &Object) -> bool {
    let mut pending = vec![(root, 0usize)];
    let mut remaining = 128usize;
    while let Some((object, depth)) = pending.pop() {
        if remaining == 0 {
            break;
        }
        remaining -= 1;
        if depth > 8 {
            continue;
        }
        let Some(object) = resolve(document, object) else {
            continue;
        };
        match object {
            Object::Dictionary(dict) => {
                if dict
                    .get(b"S")
                    .ok()
                    .and_then(|s| resolve(document, s))
                    .and_then(|s| s.as_name().ok())
                    .is_some_and(|s| matches!(s, b"JavaScript" | b"SetOCGState"))
                {
                    return true;
                }
                if let Ok(next) = dict.get(b"Next") {
                    pending.push((next, depth + 1));
                }
            }
            Object::Array(array) => {
                pending.extend(array.iter().take(remaining).map(|o| (o, depth + 1)))
            }
            _ => {}
        }
    }
    false
}

fn actions(document: &Document, dictionary: &Dictionary) -> bool {
    [b"A".as_slice(), b"OpenAction"].into_iter().any(|key| {
        dictionary
            .get(key)
            .is_ok_and(|action| unsupported_action(document, action))
    }) || dictionary
        .get(b"AA")
        .ok()
        .and_then(|aa| resolve(document, aa))
        .and_then(|aa| aa.as_dict().ok())
        .is_some_and(|aa| {
            aa.iter()
                .take(32)
                .any(|(_, action)| unsupported_action(document, action))
        })
}

pub(super) fn document(document: &Document, diagnostics: &mut Vec<StructureDiagnostic>) {
    let Some(catalog) = document
        .trailer
        .get(b"Root")
        .ok()
        .and_then(|r| resolve(document, r))
        .and_then(|r| r.as_dict().ok())
    else {
        return;
    };
    let scripts = catalog
        .get(b"Names")
        .ok()
        .and_then(|n| resolve(document, n))
        .and_then(|n| n.as_dict().ok())
        .is_some_and(|n| n.has(b"JavaScript"));
    if scripts || actions(document, catalog) {
        diagnostics.push(StructureDiagnostic {
            page: PhysicalPage(0),
            annotation: None,
            field: "Document/ActiveContent".into(),
            kind: DiagnosticKind::Unsupported,
        });
    }
}

pub(super) fn page(
    document: &Document,
    page: PhysicalPage,
    dictionary: &Dictionary,
    diagnostics: &mut Vec<StructureDiagnostic>,
) {
    let annotation_action = dictionary
        .get(b"Annots")
        .ok()
        .and_then(|a| resolve(document, a))
        .and_then(|a| a.as_array().ok())
        .is_some_and(|items| {
            items.iter().take(256).any(|a| {
                resolve(document, a)
                    .and_then(|a| a.as_dict().ok())
                    .is_some_and(|a| actions(document, a))
            })
        });
    if actions(document, dictionary) || annotation_action {
        diagnostics.push(StructureDiagnostic {
            page,
            annotation: None,
            field: "Page/ActiveContent".into(),
            kind: DiagnosticKind::Unsupported,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lopdf::dictionary;
    #[test]
    fn known_script_and_ocg_actions_are_detected_but_static_layers_are_not() {
        let mut doc = Document::with_version("1.7");
        let script = doc.add_object(dictionary! { "S" => "JavaScript", "JS" => Object::string_literal("private script content must never be reported") });
        assert!(actions(&doc, &dictionary! { "A" => script }));
        assert!(actions(
            &doc,
            &dictionary! { "AA" => dictionary! { "O" => dictionary! { "S" => "SetOCGState" } } }
        ));
        assert!(!actions(
            &doc,
            &dictionary! { "OCProperties" => dictionary! { "OCGs" => vec![Object::Integer(1)] } }
        ));
        let cycle = doc.new_object_id();
        doc.objects.insert(
            cycle,
            Object::Dictionary(dictionary! { "S" => "GoTo", "Next" => cycle }),
        );
        assert!(!unsupported_action(&doc, &Object::Reference(cycle)));
        let mut diagnostics = Vec::new();
        page(
            &doc,
            PhysicalPage(4),
            &dictionary! { "A" => script },
            &mut diagnostics,
        );
        assert_eq!(diagnostics[0].page, PhysicalPage(4));
        assert_eq!(diagnostics[0].field, "Page/ActiveContent");
    }
}
