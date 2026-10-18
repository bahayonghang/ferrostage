use super::*;
use ferrostage_pdf::{
    PageMetadata,
    structure::{PageGeometry, PageStructure},
};
fn input(labels: &[Option<&str>], w: f32, h: f32) -> (DocumentMetadata, DocumentStructure) {
    (
        DocumentMetadata {
            pages: labels
                .iter()
                .enumerate()
                .map(|(i, label)| PageMetadata {
                    page: PhysicalPage(i),
                    width_points: w,
                    height_points: h,
                    rotation_degrees: 0,
                    label: label.map(str::to_owned),
                })
                .collect(),
        },
        DocumentStructure {
            pages: (0..labels.len())
                .map(|i| PageStructure {
                    page: PhysicalPage(i),
                    geometry: PageGeometry {
                        bounds: [0.0, 0.0, w, h],
                        rotation_degrees: 0,
                    },
                    media: vec![],
                })
                .collect(),
            diagnostics: vec![],
        },
    )
}
#[test]
fn reload_maps_unique_label_ordinal_and_ambiguous_physical_content() {
    let make = |labels: &[Option<&str>]| {
        let (m, s) = input(labels, 320., 180.);
        LoadedDocument::load(&m, &s).unwrap()
    };
    let old = make(&[Some("A"), Some("B"), Some("B"), Some("notes:B"), Some("C")]);
    let next = make(&[Some("X"), Some("A"), Some("B"), Some("B"), Some("notes:B")]);
    let mapped = old.reload_mapping(&next, PhysicalPage(2));
    assert_eq!(mapped.position, PhysicalPage(3));
    assert!(mapped.pages.contains(&(PhysicalPage(2), PhysicalPage(3))));
    assert!(!mapped.pages.iter().any(|(old, _)| *old == PhysicalPage(4)));
    let repeated = make(&[Some("B"), Some("X"), Some("B"), Some("notes:B")]);
    assert_eq!(
        old.reload_mapping(&repeated, PhysicalPage(4)).position,
        PhysicalPage(2)
    );
    let short = make(&[None]);
    assert_eq!(
        old.reload_mapping(&short, PhysicalPage(4)).position,
        PhysicalPage(0)
    );
}
#[test]
fn labels_group_only_contiguous_content_and_notes_pair_by_step() {
    let (m, s) = input(
        &[
            Some("notes:Intro"),
            Some("Intro"),
            Some("notes:Intro"),
            Some("notes:Intro"),
            Some("Intro"),
            Some("notes:Intro"),
            Some("B"),
            Some("notes:Intro"),
            Some("Intro"),
            Some("01"),
            Some("notes:1"),
            Some("1"),
            None,
            Some(""),
        ],
        320.0,
        180.0,
    );
    let doc = LoadedDocument::load(&m, &s).unwrap();
    assert_eq!(
        doc.slide_map.steps(),
        [1, 4, 6, 8, 9, 11, 12, 13].map(PhysicalPage)
    );
    assert_eq!(doc.slide_map.groups().len(), 7);
    assert_eq!(
        doc.notes_pages_for(PhysicalPage(1)),
        [PhysicalPage(2), PhysicalPage(3)]
    );
    assert_eq!(doc.notes_pages_for(PhysicalPage(4)), [PhysicalPage(5)]);
    assert_eq!(
        doc.label_candidates("Intro"),
        [LogicalSlide(0), LogicalSlide(2)]
    );
    assert_eq!(
        doc.warnings
            .iter()
            .filter(|w| matches!(w, DocumentWarning::OrphanNotes { .. }))
            .count(),
        3
    );
    let physical = LoadedDocument::build(
        &m,
        &s,
        DocumentOverrides {
            physical_pages: true,
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(physical.slide_map.groups().len(), 8);
    let all = LoadedDocument::build(
        &m,
        &s,
        DocumentOverrides {
            identify_independent_notes: false,
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(all.slide_map.steps().len(), 14);
}
#[test]
fn layout_candidates_overrides_counterexamples_and_reload_scope() {
    for (w, h, layout) in [
        (640., 180., NotesLayout::Right),
        (480., 180., NotesLayout::Right),
        (320., 360., NotesLayout::Bottom),
        (240., 360., NotesLayout::Bottom),
        (320., 180., NotesLayout::None),
        (180., 320., NotesLayout::None),
    ] {
        let (m, s) = input(&[Some("A"), Some("notes:A")], w, h);
        let d = LoadedDocument::load(&m, &s).unwrap();
        assert_eq!(d.layout, layout);
        if layout != NotesLayout::None {
            assert!(d.warnings.contains(&DocumentWarning::AmbiguousLayout));
        }
        for forced in [
            NotesLayout::None,
            NotesLayout::Left,
            NotesLayout::Right,
            NotesLayout::Top,
            NotesLayout::Bottom,
        ] {
            let d = LoadedDocument::build(
                &m,
                &s,
                DocumentOverrides {
                    layout: Some(forced),
                    ..Default::default()
                },
            )
            .unwrap();
            assert_eq!(d.layout, forced);
            assert_eq!(d.layout_source, LayoutSource::Manual);
            assert_eq!(d.slide_map.steps(), [PhysicalPage(0)]);
            assert_eq!(d.reload(&m, &s).unwrap().layout, forced);
            assert_eq!(
                LoadedDocument::load(&m, &s).unwrap().overrides,
                DocumentOverrides::default()
            );
        }
    }
    let (mut m, mut s) = input(&[None, None], 640., 180.);
    m.pages[1].width_points = 320.;
    s.pages[1].geometry.bounds[2] = 320.;
    let d = LoadedDocument::load(&m, &s).unwrap();
    assert_eq!(d.layout, NotesLayout::None);
    assert!(d.warnings.contains(&DocumentWarning::MixedGeometry));
}
#[test]
fn invalid_models_are_rejected_and_projection_has_no_secrets() {
    let (m, s) = input(&[Some("notes:A")], 320., 180.);
    assert_eq!(
        LoadedDocument::load(&m, &s),
        Err(DocumentError::Core(CoreError::EmptyContent))
    );
    let (mut m, s) = input(&[Some("A")], 320., 180.);
    let mut d = LoadedDocument::load(&m, &s).unwrap();
    d.notes.push(SlideNotes {
        page: PhysicalPage(0),
        markdown: "SECRET MARKDOWN".into(),
    });
    assert_eq!(d.notes_for(PhysicalPage(0)), Some("SECRET MARKDOWN"));
    let mut state = PresentationState::idle();
    state
        .apply_at(
            ferrostage_core::PresentationCommand::LoadMappedDocument(
                d.descriptor(DocumentGeneration(1)).unwrap(),
            ),
            std::time::Instant::now(),
        )
        .unwrap();
    let snapshot = LoadedDocument::project(&state, d.notes.first());
    assert!(!format!("{snapshot:?}").contains("SECRET"));
    assert_eq!(snapshot, LoadedDocument::project(&state, None));
    m.pages[0].width_points = f32::NAN;
    assert_eq!(
        LoadedDocument::load(&m, &s),
        Err(DocumentError::MetadataMismatch)
    );
}
#[test]
fn crop_halves_and_media_projection_are_explicit() {
    use ferrostage_core::NormalizedRect as R;
    for (layout, slide, notes) in [
        (NotesLayout::None, R::FULL, None),
        (NotesLayout::Left, R::RIGHT_HALF, Some(R::LEFT_HALF)),
        (NotesLayout::Right, R::LEFT_HALF, Some(R::RIGHT_HALF)),
        (NotesLayout::Top, R::BOTTOM_HALF, Some(R::TOP_HALF)),
        (NotesLayout::Bottom, R::TOP_HALF, Some(R::BOTTOM_HALF)),
    ] {
        assert_eq!(slide_crop(layout), slide);
        assert_eq!(notes_crop(layout), notes);
    }
    assert_eq!(
        crate::media_slots::in_crop(
            R {
                x: 0.25,
                y: 0.2,
                w: 0.5,
                h: 0.4
            },
            R::LEFT_HALF
        ),
        Some(R {
            x: 0.5,
            y: 0.2,
            w: 0.5,
            h: 0.40000004
        })
    );
    assert_eq!(
        crate::media_slots::in_crop(R::RIGHT_HALF, R::LEFT_HALF),
        None
    );
}
#[test]
fn raw_geometry_mismatch_or_nonfinite_is_rejected_and_mixed_rotation_is_not_cropped() {
    let (m, mut s) = input(&[None, None], 640., 180.);
    s.pages[0].geometry.bounds[2] = f32::NAN;
    assert_eq!(
        LoadedDocument::load(&m, &s),
        Err(DocumentError::MetadataMismatch)
    );
    let (mut m, mut s) = input(&[None, None], 640., 180.);
    m.pages[1].rotation_degrees = 180;
    s.pages[1].geometry.rotation_degrees = 180;
    let d = LoadedDocument::load(&m, &s).unwrap();
    assert_eq!(d.layout, NotesLayout::None);
    assert!(d.warnings.contains(&DocumentWarning::MixedGeometry));
    m.pages[1].page = PhysicalPage(7);
    assert_eq!(
        LoadedDocument::load(&m, &s),
        Err(DocumentError::MetadataMismatch)
    );
}

#[test]
fn real_media_pdf_slots_keep_parameters_and_public_source_crop() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/media");
    for filename in ["media-smoke.pdf", "media-crop-smoke.pdf"] {
        let bytes = std::fs::read(root.join(filename)).unwrap();
        let structure = ferrostage_pdf::structure::extract_structure(&bytes).unwrap();
        let metadata = DocumentMetadata {
            pages: structure
                .pages
                .iter()
                .map(|p| {
                    let (w, h) = p.geometry.visible_size();
                    ferrostage_pdf::PageMetadata {
                        page: p.page,
                        width_points: w,
                        height_points: h,
                        rotation_degrees: 0,
                        label: None,
                    }
                })
                .collect(),
        };
        let document = LoadedDocument::build(
            &metadata,
            &structure,
            crate::DocumentOverrides {
                layout: Some(crate::NotesLayout::Right),
                ..Default::default()
            },
        )
        .unwrap();
        if filename == "media-crop-smoke.pdf" {
            assert_eq!(document.media.len(), 3);
            let public = &document.media[0];
            assert!(public.slide_rect.is_some());
            assert_eq!(
                public.source_crop.unwrap(),
                ferrostage_core::NormalizedRect::LEFT_HALF
            );
            assert!((public.aspect_ratio.unwrap() - 1.6).abs() < 1e-6);
            assert!(document.media[1].slide_rect.is_none());
            assert!(document.media[1].source_crop.is_none());
            assert!(document.media[2].slide_rect.is_some());
        } else {
            assert_eq!(document.media.len(), 6);
            assert_eq!(
                document.media[2].media.start,
                Some(ferrostage_pdf::structure::MediaTime::Seconds(2.))
            );
            assert_eq!(
                document.media[2].media.duration,
                Some(ferrostage_pdf::structure::MediaTime::Seconds(8.))
            );
            assert!(
                matches!(&document.media[4].media.source,Some(ferrostage_pdf::structure::MediaSource::Embedded{bytes,..}) if bytes.len()>500_000)
            );
        }
    }
}
