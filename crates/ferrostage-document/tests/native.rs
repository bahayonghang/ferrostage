#![cfg(feature = "pdfium")]
use ferrostage_core::{DocumentGeneration, NormalizedRect, PhysicalPage};
use ferrostage_document::*;
use ferrostage_pdf::{RuntimeConfig, extract_structure};
use ferrostage_render::{CacheBudgets, RenderHandle};
use std::{path::PathBuf, time::Duration};
#[test]
fn actual_pdfium_metadata_builds_fixture_models() {
    let path = PathBuf::from(
        std::env::var_os("FERROSTAGE_TEST_PDFIUM")
            .expect("native suite requires FERROSTAGE_TEST_PDFIUM"),
    );
    let worker = RenderHandle::spawn(
        RuntimeConfig {
            library_path: path,
            font_directories: vec![],
        },
        CacheBudgets::default(),
    )
    .unwrap();
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/beamer");
    for name in [
        "overlays",
        "notes-right",
        "independent",
        "bottom-wide",
        "bottom-classic",
        "top-rotated",
        "wide-counterexample",
        "portrait-counterexample",
        "ordinary",
        "tall-ordinary",
        "mixed",
        "media",
        "movie",
    ] {
        let bytes = std::fs::read(root.join(format!("{name}.pdf"))).unwrap();
        let metadata = worker
            .probe(bytes.clone())
            .unwrap()
            .recv_timeout(Duration::from_secs(20))
            .unwrap()
            .unwrap();
        println!("{name} metadata: {metadata:?}");
        let structure = extract_structure(&bytes).unwrap();
        let fresh = LoadedDocument::load(&metadata, &structure).unwrap();
        assert_eq!(fresh.layout, NotesLayout::None);
        assert_eq!(slide_crop(fresh.layout), NormalizedRect::FULL);
        let doc = LoadedDocument::build(
            &metadata,
            &structure,
            DocumentOverrides {
                layout: None,
                ..Default::default()
            },
        )
        .unwrap();
        println!(
            "{name} model: layout={:?}, groups={:?}, warnings={:?}",
            doc.layout,
            doc.slide_map.groups(),
            doc.warnings
        );
        match name {
            "overlays" => {
                assert_eq!(doc.page_count, 3);
                assert_eq!(doc.slide_map.groups().len(), 2);
                assert_eq!(doc.slide_map.steps().len(), 3);
            }
            "notes-right" => {
                assert_eq!(doc.layout, NotesLayout::Right);
                assert_eq!(slide_crop(doc.layout), NormalizedRect::LEFT_HALF);
                assert_eq!(notes_crop(doc.layout), Some(NormalizedRect::RIGHT_HALF));
                assert_eq!(doc.labels, vec![Some("1".into()), Some("1".into())]);
                assert_eq!(doc.slide_map.groups().len(), 1);
                let physical = LoadedDocument::build(
                    &metadata,
                    &structure,
                    DocumentOverrides {
                        physical_pages: true,
                        ..Default::default()
                    },
                )
                .unwrap();
                assert_eq!(physical.slide_map.groups().len(), 2);
            }
            "independent" => {
                assert_eq!(
                    doc.slide_map.steps(),
                    [1, 4, 6, 8, 9, 11, 12, 13].map(PhysicalPage)
                );
                assert_eq!(doc.slide_map.groups().len(), 7);
                assert_eq!(
                    doc.notes_pages_for(PhysicalPage(1)),
                    [PhysicalPage(2), PhysicalPage(3)]
                );
            }
            "bottom-wide" | "bottom-classic" | "portrait-counterexample" | "top-rotated" => {
                assert_eq!(doc.layout, NotesLayout::Bottom)
            }
            "wide-counterexample" => assert_eq!(doc.layout, NotesLayout::Right),
            "ordinary" | "tall-ordinary" | "mixed" => assert_eq!(doc.layout, NotesLayout::None),
            "media" => assert_eq!(doc.media.len(), 5),
            "movie" => {
                assert_eq!(doc.media.len(), 1);
                assert_eq!(doc.media[0].media.start, Some(MediaTime::Seconds(2.)));
                assert_eq!(doc.media[0].media.duration, Some(MediaTime::Seconds(8.)));
                assert_eq!(doc.media[0].media.autostart, Some(true));
            }
            _ => unreachable!(),
        }
        let forced = LoadedDocument::build(
            &metadata,
            &structure,
            DocumentOverrides {
                layout: Some(NotesLayout::None),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(
            forced
                .descriptor(DocumentGeneration(1))
                .unwrap()
                .slide_crop(),
            NormalizedRect::FULL
        );
    }
    let bytes = std::fs::read(root.join("notes-right.pdf")).unwrap();
    worker
        .load_active(DocumentGeneration(1), bytes)
        .unwrap()
        .recv_timeout(Duration::from_secs(20))
        .unwrap()
        .unwrap();
    let full = render(&worker, NormalizedRect::FULL, 1024, 288);
    let slide = render(&worker, NormalizedRect::LEFT_HALF, 512, 288);
    let notes = render(&worker, NormalizedRect::RIGHT_HALF, 512, 288);
    assert_eq!((full.width(), full.height()), (1024, 288));
    assert_eq!((slide.width(), slide.height()), (512, 288));
    for (crop, offset) in [(&slide, 0usize), (&notes, 512usize)] {
        let mut ink = 0;
        for y in 2..286usize {
            for x in 2..510usize {
                for channel in 0..4 {
                    let a = full.pixels_bgra()[y * full.stride() + (x + offset) * 4 + channel];
                    let b = crop.pixels_bgra()[y * crop.stride() + x * 4 + channel];
                    assert!(
                        a.abs_diff(b) <= 1,
                        "crop mismatch at ({x},{y}), channel {channel}"
                    );
                    ink += usize::from(b != 255);
                }
            }
        }
        assert!(
            ink > 100,
            "fixture crop must contain independently rendered content"
        );
    }
    assert_ne!(slide.pixels_bgra(), notes.pixels_bgra());
    worker.shutdown().unwrap();
}

fn render(
    worker: &RenderHandle,
    crop: NormalizedRect,
    width: u32,
    height: u32,
) -> ferrostage_core::BgraFrame {
    use ferrostage_render::{RenderPriority, RenderRequest};
    let request = RenderRequest {
        generation: DocumentGeneration(1),
        page: PhysicalPage(0),
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
