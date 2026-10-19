use ferrostage_core::{PhysicalPage, TransitionStyle};
use ferrostage_document::LoadedDocument;
use ferrostage_pdf::{DocumentMetadata, PageMetadata, extract_structure};

const PDF: &[u8] = include_bytes!("../../../fixtures/beamer-timing/overlays-timing.pdf");
const LABELS: [&str; 33] = [
    "1", "notes:1", "1", "notes:1", "notes:1", "2", "notes:2", "2", "notes:2", "2", "notes:2", "3",
    "notes:3", "3", "notes:3", "3", "notes:3", "4", "notes:4", "4", "notes:4", "4", "notes:4", "4",
    "notes:4", "5", "notes:5", "5", "notes:5", "5", "notes:5", "6", "notes:6",
];
const CONTENT: [usize; 16] = [0, 2, 5, 7, 9, 11, 13, 15, 17, 19, 21, 23, 25, 27, 29, 31];

fn verify_model(metadata: &DocumentMetadata) -> LoadedDocument {
    let structure = extract_structure(PDF).unwrap();
    let doc = LoadedDocument::load(metadata, &structure).unwrap();
    assert_eq!(doc.labels, LABELS.map(|label| Some(label.into())));
    assert_eq!(doc.slide_map.steps(), CONTENT.map(PhysicalPage));
    assert_eq!(
        doc.slide_map
            .groups()
            .iter()
            .map(Vec::len)
            .collect::<Vec<_>>(),
        [2, 3, 3, 4, 3, 1]
    );
    assert_eq!(
        doc.notes_pages_for(PhysicalPage(2)),
        [PhysicalPage(3), PhysicalPage(4)]
    );
    for (index, page) in CONTENT.iter().copied().enumerate() {
        assert_eq!(
            doc.slide_map.next_step(PhysicalPage(page)),
            Some(PhysicalPage(CONTENT[(index + 1).min(15)]))
        );
        assert_eq!(
            doc.slide_map.previous_step(PhysicalPage(page)),
            Some(PhysicalPage(CONTENT[index.saturating_sub(1)]))
        );
        assert_eq!(
            doc.notes_pages_for(PhysicalPage(page))[0],
            PhysicalPage(page + 1)
        );
        assert_eq!(doc.page_presentation(PhysicalPage(page + 1)), None);
        let dwell = doc
            .page_presentation(PhysicalPage(page))
            .unwrap()
            .dwell_seconds;
        assert_eq!(
            dwell,
            match page {
                17 | 19 | 21 | 27 | 31 => Some(0.),
                25 => Some(1.5),
                _ => None,
            }
        );
    }
    let transition = doc
        .page_presentation(PhysicalPage(27))
        .unwrap()
        .transition
        .unwrap();
    assert_eq!(transition.style, TransitionStyle::Fade);
    assert!((transition.duration_seconds - 0.3).abs() < 0.00001);
    doc
}

#[test]
fn compiled_overlay_fixture_preserves_all_steps_notes_and_timing() {
    let structure = extract_structure(PDF).unwrap();
    let metadata = DocumentMetadata {
        pages: structure
            .pages
            .iter()
            .zip(LABELS)
            .map(|(page, label)| PageMetadata {
                page: page.page,
                width_points: page.geometry.visible_size().0,
                height_points: page.geometry.visible_size().1,
                rotation_degrees: page.geometry.rotation_degrees,
                label: Some(label.into()),
            })
            .collect(),
    };
    verify_model(&metadata);
}

#[cfg(feature = "pdfium")]
#[test]
fn actual_pdfium_labels_and_every_overlay_raster_match_compiled_fixture() {
    use ferrostage_core::{DocumentGeneration, NormalizedRect};
    use ferrostage_pdf::{PixelSize, RuntimeConfig};
    use ferrostage_render::{CacheBudgets, RenderHandle, RenderPriority, RenderRequest};
    use std::{path::PathBuf, time::Duration};
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
    let metadata = worker
        .probe(PDF.to_vec())
        .unwrap()
        .recv_timeout(Duration::from_secs(20))
        .unwrap()
        .unwrap();
    let doc = verify_model(&metadata);
    worker
        .load_active(DocumentGeneration(1), PDF.to_vec())
        .unwrap()
        .recv_timeout(Duration::from_secs(20))
        .unwrap()
        .unwrap();
    let mut previous = None;
    for page in 0..33 {
        let request = RenderRequest {
            generation: DocumentGeneration(1),
            page: PhysicalPage(page),
            crop: NormalizedRect::FULL,
            target_px: PixelSize {
                width: 640,
                height: 360,
            },
            priority: RenderPriority::Audience,
            epoch: 0,
        };
        let id = worker.submit_uncached(request).unwrap();
        let result = worker
            .results()
            .recv_timeout(Duration::from_secs(20))
            .unwrap();
        assert_eq!(result.id, id);
        let frame = result.outcome.unwrap().frame;
        assert!(frame.pixels_bgra().iter().any(|value| *value < 128));
        if CONTENT.contains(&page) {
            if let Some(previous) = previous.replace(frame.clone()) {
                assert_ne!(
                    previous.pixels_bgra(),
                    frame.pixels_bgra(),
                    "compiled step {page} must render its new overlay content"
                );
            }
        } else {
            assert_eq!(doc.page_presentation(PhysicalPage(page)), None);
            assert_ne!(
                previous.as_ref().unwrap().pixels_bgra(),
                frame.pixels_bgra(),
                "notes must remain distinct from public pixels"
            );
        }
    }
    worker.shutdown().unwrap();
}
