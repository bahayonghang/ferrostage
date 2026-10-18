#![cfg(feature = "pdfium")]
use ferrostage_core::{DocumentGeneration, NormalizedRect, PhysicalPage};
use ferrostage_document::*;
use ferrostage_pdf::{RuntimeConfig, extract_structure};
use ferrostage_render::{CacheBudgets, RenderHandle};
use std::{path::PathBuf, time::Duration};

#[test]
fn compiled_beamer_notes_project_only_slide_pixels_and_preserve_overlays() {
    use ferrostage_core::{AudienceStage, PresentationCommand, PresentationState};
    use std::time::Instant;

    let worker = RenderHandle::spawn(
        RuntimeConfig {
            library_path: PathBuf::from(
                std::env::var_os("FERROSTAGE_TEST_PDFIUM")
                    .expect("native suite requires FERROSTAGE_TEST_PDFIUM"),
            ),
            font_directories: vec![],
        },
        CacheBudgets::default(),
    )
    .unwrap();
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/notes-functional/pympress-notes.pdf");
    let bytes = std::fs::read(&fixture).unwrap();
    let metadata = worker
        .probe(bytes.clone())
        .unwrap()
        .recv_timeout(Duration::from_secs(20))
        .unwrap()
        .unwrap();
    let doc = LoadedDocument::load(&metadata, &extract_structure(&bytes).unwrap()).unwrap();
    assert_eq!(doc.layout, NotesLayout::Right);
    assert_eq!(doc.page_count, 5);
    assert_eq!(
        doc.labels,
        ["1", "2", "3", "3", "4"].map(|label| Some(label.into()))
    );
    assert_eq!(
        doc.slide_map.groups(),
        [
            vec![PhysicalPage(0)],
            vec![PhysicalPage(1)],
            vec![PhysicalPage(2), PhysicalPage(3)],
            vec![PhysicalPage(4)],
        ]
    );
    assert_eq!(doc.slide_map.steps(), [0, 1, 2, 3, 4].map(PhysicalPage));
    let mut state = PresentationState::idle();
    state
        .apply_at(
            PresentationCommand::LoadMappedDocument(doc.descriptor(DocumentGeneration(1)).unwrap()),
            Instant::now(),
        )
        .unwrap();
    state
        .apply_at(
            PresentationCommand::StartPresentation {
                audience_available: true,
            },
            Instant::now(),
        )
        .unwrap();
    worker
        .load_active(DocumentGeneration(1), bytes)
        .unwrap()
        .recv_timeout(Duration::from_secs(20))
        .unwrap()
        .unwrap();
    let mut first_overlay = None;
    for page in 0..5 {
        let snapshot = LoadedDocument::project(&state, None);
        assert_eq!(snapshot.stage, AudienceStage::Live);
        assert_eq!(snapshot.current_page, PhysicalPage(page));
        assert_eq!(snapshot.slide_crop, NormalizedRect::LEFT_HALF);
        let full = render_page(
            &worker,
            snapshot.current_page,
            NormalizedRect::FULL,
            1280,
            360,
        );
        let audience = render_page(
            &worker,
            snapshot.current_page,
            snapshot.slide_crop,
            640,
            360,
        );
        let notes = render_page(
            &worker,
            snapshot.current_page,
            notes_crop(doc.layout).unwrap(),
            640,
            360,
        );
        for (crop, offset) in [(&audience, 0usize), (&notes, 640usize)] {
            let mut ink = 0;
            for y in 2..358usize {
                for x in 2..638usize {
                    for channel in 0..4 {
                        let actual = crop.pixels_bgra()[y * crop.stride() + x * 4 + channel];
                        let expected =
                            full.pixels_bgra()[y * full.stride() + (x + offset) * 4 + channel];
                        assert!(
                            actual.abs_diff(expected) <= 1,
                            "page {page}, crop {offset}, pixel ({x}, {y}), channel {channel}"
                        );
                        ink += usize::from(actual != 255);
                    }
                }
            }
            assert!(ink > 100, "page {page}, crop {offset} must be nonempty");
        }
        assert_ne!(audience.pixels_bgra(), notes.pixels_bgra());
        if page == 2 {
            first_overlay = Some((audience.clone(), notes.clone()));
            let mut logical_navigation = state.clone();
            logical_navigation
                .apply_at(PresentationCommand::NextLogicalSlide, Instant::now())
                .unwrap();
            assert_eq!(logical_navigation.current_page(), PhysicalPage(4));
        } else if page == 3 {
            let (first_slide, first_notes) = first_overlay.as_ref().unwrap();
            assert_ne!(first_slide.pixels_bgra(), audience.pixels_bgra());
            assert_ne!(first_notes.pixels_bgra(), notes.pixels_bgra());
        }
        println!(
            "compiled Beamer page {page}: audience matches full-page left crop; distinct nonempty notes right crop"
        );
        state
            .apply_at(PresentationCommand::NextStep, Instant::now())
            .unwrap();
    }
    println!(
        "compiled Beamer model: 5 physical / 4 logical; labels {:?}; source {}",
        doc.labels,
        fixture.display()
    );
    worker.shutdown().unwrap();
}

fn render_page(
    worker: &RenderHandle,
    page: PhysicalPage,
    crop: NormalizedRect,
    width: u32,
    height: u32,
) -> ferrostage_core::BgraFrame {
    use ferrostage_render::{RenderPriority, RenderRequest};
    let request = RenderRequest {
        generation: DocumentGeneration(1),
        page,
        crop,
        target_px: ferrostage_pdf::PixelSize { width, height },
        priority: RenderPriority::Audience,
        epoch: 0,
    };
    let id = worker.submit_uncached(request).unwrap();
    let result = worker
        .results()
        .recv_timeout(Duration::from_secs(20))
        .unwrap();
    assert_eq!(result.id, id);
    assert_eq!(result.request, request);
    result.outcome.unwrap().frame
}
