use super::*;
use lopdf::dictionary;
#[test]
fn authored_media_paths_times_and_inherited_geometry() {
    let s = extract_structure(include_bytes!("../../../../fixtures/beamer/media.pdf")).unwrap();
    assert_eq!(s.pages[0].geometry.visible_size(), (240., 360.));
    assert_eq!(s.pages[0].geometry.rotation_degrees, 90);
    let media = &s.pages[0].media;
    assert_eq!(media.len(), 5);
    for item in media.iter().take(3) {
        assert_eq!(
            item.rect,
            Some(NormalizedRect {
                x: 0.,
                y: 0.,
                w: 0.5,
                h: 0.5
            })
        );
        assert_eq!(item.autostart, Some(true));
    }
    for item in media.iter().take(2) {
        assert_eq!(item.start, Some(MediaTime::Seconds(2.)));
        assert_eq!(item.duration, Some(MediaTime::Seconds(8.)));
        assert_eq!(item.repeat, Some(MediaRepeat::Forever));
    }
    assert_eq!(
        media[0].source,
        Some(MediaSource::External("external-demo.mp4".into()))
    );
    let embedded = MediaSource::Embedded {
        name: Some("clip.mp4".into()),
        bytes: b"fixture-not-video".to_vec(),
    };
    assert_eq!(media[1].source, Some(embedded.clone()));
    assert_eq!(media[2].source, Some(embedded));
    assert_eq!(media[2].start, None);
    assert_eq!(media[2].repeat, None);
    assert_eq!(media[3].start, Some(MediaTime::Unscaled(120)));
    assert_eq!(media[3].duration, None);
    assert_eq!(media[3].repeat, None);
    assert!(media[4].source.is_none());
    assert!(
        s.diagnostics
            .iter()
            .any(|d| d.annotation == Some(3) && d.kind == DiagnosticKind::UnscaledTime)
    );
    assert!(
        s.diagnostics
            .iter()
            .any(|d| d.annotation == Some(4) && d.kind == DiagnosticKind::Unsupported)
    );
}
#[test]
fn rect_transform_all_rotations_and_clipping() {
    let expected = [
        (0., 0.5, 0.5, 0.5),
        (0., 0., 0.5, 0.5),
        (0.5, 0., 0.5, 0.5),
        (0.5, 0.5, 0.5, 0.5),
    ];
    for (index, rotation) in [0, 90, 180, 270].into_iter().enumerate() {
        let g = PageGeometry {
            bounds: [20., 30., 380., 270.],
            rotation_degrees: rotation,
        };
        let (x, y, w, h) = expected[index];
        assert_eq!(
            g.normalize_rect([20., 30., 200., 150.]),
            Some(NormalizedRect { x, y, w, h })
        );
        assert_eq!(
            g.normalize_rect([0., 0., 400., 300.]),
            Some(NormalizedRect::FULL)
        );
        assert_eq!(g.normalize_rect([0., 0., 10., 10.]), None);
        assert_eq!(g.normalize_rect([f32::NAN, 0., 10., 10.]), None);
    }
}
fn altered(change: impl FnOnce(&mut lopdf::Document)) -> Vec<u8> {
    let mut d =
        lopdf::Document::load_mem(include_bytes!("../../../../fixtures/beamer/media.pdf")).unwrap();
    change(&mut d);
    let mut bytes = Vec::new();
    d.save_to(&mut bytes).unwrap();
    bytes
}
#[test]
fn malformed_geometry_and_unresolved_actions_are_reported() {
    let bytes = altered(|d| d.get_dictionary_mut((2, 0)).unwrap().set("Rotate", 45));
    assert_eq!(
        extract_structure(&bytes),
        Err(PdfStructureError::InvalidPageGeometry)
    );
    let bytes = altered(|d| d.get_dictionary_mut((3, 0)).unwrap().set("UserUnit", 2));
    assert_eq!(
        extract_structure(&bytes),
        Err(PdfStructureError::UnsupportedUserUnit)
    );
    let bytes = altered(|d| {
        d.get_dictionary_mut((9, 0))
            .unwrap()
            .set("Annotation", (7, 0));
        d.get_dictionary_mut((5, 0)).unwrap().remove(b"AA");
    });
    let s = extract_structure(&bytes).unwrap();
    assert_eq!(s.pages[0].media[0].autostart, None);
    assert_eq!(s.pages[0].media[1].autostart, None);
    assert_eq!(s.pages[0].media[1].play_on_activation, Some(true));
    let bytes = altered(|d| {
        d.get_dictionary_mut((10, 0)).unwrap().set("OP", 1);
        d.get_dictionary_mut((4, 0))
            .unwrap()
            .set("Rect", Object::Null);
    });
    let s = extract_structure(&bytes).unwrap();
    assert!(s.pages[0].media[0].rect.is_none());
    assert!(s.pages[0].media[1].source.is_none());
}
#[test]
fn actual_beamer_movie_fixture_emits_scaled_times() {
    let bytes = include_bytes!("../../../../fixtures/beamer/movie.pdf");
    let s = extract_structure(bytes).unwrap();
    let m = &s.pages[0].media[0];
    assert_eq!(m.start, Some(MediaTime::Seconds(2.)));
    assert_eq!(m.duration, Some(MediaTime::Seconds(8.)));
    assert_eq!(m.repeat, Some(MediaRepeat::Forever));
    assert_eq!(m.autostart, Some(true));
}

#[test]
fn movie_title_targets_must_be_unique_and_action_overrides_take_precedence() {
    let mut d =
        lopdf::Document::load_mem(include_bytes!("../../../../fixtures/beamer/movie.pdf")).unwrap();
    let target = d
        .objects
        .iter()
        .find(|(_, o)| {
            o.as_dict().ok().is_some_and(|v| {
                v.get(b"Subtype").ok().and_then(|s| s.as_name().ok()) == Some(b"Movie")
            })
        })
        .map(|(id, _)| *id)
        .unwrap();
    let mut duplicate = d.get_object(target).unwrap().clone();
    duplicate
        .as_dict_mut()
        .unwrap()
        .set("Rect", vec![0.into(), 0.into(), 10.into(), 10.into()]);
    let duplicate_id = d.add_object(duplicate);
    let page = *d.get_pages().values().next().unwrap();
    d.get_dictionary_mut(page)
        .unwrap()
        .get_mut(b"Annots")
        .unwrap()
        .as_array_mut()
        .unwrap()
        .push(duplicate_id.into());
    let mut bytes = Vec::new();
    d.save_to(&mut bytes).unwrap();
    let s = extract_structure(&bytes).unwrap();
    assert!(s.pages[0].media.iter().all(|m| m.autostart.is_none()));
    assert!(
        s.diagnostics
            .iter()
            .any(|d| d.field.contains("ambiguous title"))
    );
    let bytes = altered(|d| {
        d.get_dictionary_mut((9, 0))
            .unwrap()
            .set("Start", vec![3000.into(), 1000.into()])
    });
    assert_eq!(
        extract_structure(&bytes).unwrap().pages[0].media[0].start,
        Some(MediaTime::Seconds(3.))
    );
}
#[test]
fn richmedia_selection_membership_and_screen_precedence_are_explicit() {
    let bytes = altered(|d| {
        let settings =
            dictionary! {"Activation"=>dictionary!{"Configuration"=>(15,0),"Condition"=>"XA"}};
        d.get_dictionary_mut((6, 0))
            .unwrap()
            .set("RichMediaSettings", settings);
    });
    let s = extract_structure(&bytes).unwrap();
    assert_eq!(s.pages[0].media[2].source, None);
    assert!(
        s.diagnostics
            .iter()
            .any(|d| d.annotation == Some(2) && d.field == "RichMediaContent/Configurations")
    );
    let bytes = altered(|d| {
        let settings =
            dictionary! {"Activation"=>dictionary!{"Configuration"=>(13,0),"Condition"=>"XA"}};
        d.get_dictionary_mut((6, 0))
            .unwrap()
            .set("RichMediaSettings", settings);
        d.get_dictionary_mut((10, 0))
            .unwrap()
            .set("JS", Object::string_literal("doNotExecute()"));
    });
    let s = extract_structure(&bytes).unwrap();
    assert!(s.pages[0].media[2].source.is_some());
    assert_eq!(s.pages[0].media[2].autostart, Some(false));
    assert!(s.pages[0].media[1].source.is_none());
    assert!(
        s.diagnostics
            .iter()
            .any(|d| d.annotation == Some(1) && d.kind == DiagnosticKind::Unsupported)
    );
}

#[test]
fn malformed_embedded_file_dictionary_does_not_become_an_external_source() {
    for value in [Object::Integer(42), Object::Reference((999, 0))] {
        let bytes = altered(|d| {
            d.get_dictionary_mut((12, 0)).unwrap().set("EF", value);
        });
        let s = extract_structure(&bytes).unwrap();
        for annotation in [1, 2] {
            assert_eq!(s.pages[0].media[annotation].source, None);
            assert!(s.diagnostics.iter().any(|d| {
                d.annotation == Some(annotation) && d.kind == DiagnosticKind::Malformed
            }));
        }
    }
    // PDF null entries are equivalent to absent entries; the external filename remains valid.
    let bytes = altered(|d| {
        d.get_dictionary_mut((12, 0))
            .unwrap()
            .set("EF", Object::Null);
    });
    assert_eq!(
        extract_structure(&bytes).unwrap().pages[0].media[1].source,
        Some(MediaSource::External("clip.mp4".into()))
    );
}

#[test]
fn movie_action_unsupported_parameters_are_diagnosed() {
    for (key, value) in [
        ("Rate", Object::Integer(-1)),
        ("FWScale", Object::Array(vec![2.into(), 1.into()])),
    ] {
        let bytes = altered(|d| {
            d.get_dictionary_mut((9, 0)).unwrap().set(key, value);
        });
        let s = extract_structure(&bytes).unwrap();
        assert_eq!(s.pages[0].media[0].autostart, Some(true));
        assert!(s.diagnostics.iter().any(|d| {
            d.annotation == Some(0)
                && d.kind == DiagnosticKind::Unsupported
                && d.field.contains(key)
        }));
    }
}
