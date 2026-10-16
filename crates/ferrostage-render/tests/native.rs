#![cfg(feature = "pdfium")]
use ferrostage_core::{DocumentGeneration, NormalizedRect, PhysicalPage};
use ferrostage_pdf::{PdfError, PixelSize, RuntimeConfig};
use ferrostage_render::*;
use std::{path::PathBuf, time::Duration};

fn receive<T>(reply: Reply<T>) -> Result<T, RenderError> {
    reply.recv_timeout(Duration::from_secs(15)).unwrap()
}
fn request(page: usize, crop: NormalizedRect, width: u32, height: u32) -> RenderRequest {
    RenderRequest {
        generation: DocumentGeneration(1),
        page: PhysicalPage(page),
        crop,
        target_px: PixelSize { width, height },
        priority: RenderPriority::Audience,
        epoch: 0,
    }
}
fn render(worker: &RenderHandle, request: RenderRequest) -> RenderResult {
    let id = worker.submit_uncached(request).unwrap();
    let result = worker
        .results()
        .recv_timeout(Duration::from_secs(15))
        .unwrap();
    assert_eq!(result.id, id);
    assert_eq!(result.request, request);
    let result = result.outcome.unwrap();
    assert_eq!((result.id, result.request), (id, request));
    assert!(!result.cache_hit);
    assert!(result.render_duration.is_some());
    result
}
fn compare(full: &RenderResult, crop: &RenderResult, x: usize, y: usize) {
    let a = full.frame.pixels_bgra();
    let b = crop.frame.pixels_bgra();
    let mut nonwhite = 0;
    for row in 2..crop.frame.height() as usize - 2 {
        for col in 2..crop.frame.width() as usize - 2 {
            for channel in 0..4 {
                let expected = a[(row + y) * full.frame.stride() + (col + x) * 4 + channel];
                let actual = b[row * crop.frame.stride() + col * 4 + channel];
                assert!(
                    expected.abs_diff(actual) <= 1,
                    "page {:?}, ({col},{row}), channel {channel}: {expected} != {actual}",
                    crop.request.page
                );
                nonwhite += usize::from(actual != 255);
            }
        }
    }
    assert!(
        nonwhite > 1000,
        "comparison must contain colored/text pixels"
    );
}

#[test]
fn native_golden_and_transaction() {
    // Missing configuration is a failing native suite, never a successful skip.
    let path = PathBuf::from(
        std::env::var_os("FERROSTAGE_TEST_PDFIUM")
            .expect("native suite requires FERROSTAGE_TEST_PDFIUM"),
    );
    let config = RuntimeConfig {
        library_path: path,
        font_directories: vec![],
    };
    let worker = RenderHandle::spawn(config.clone(), CacheBudgets::default()).unwrap();
    assert!(matches!(
        RenderHandle::spawn(config.clone(), CacheBudgets::default()),
        Err(RenderError::Pdf(PdfError::RuntimeBusy))
    ));
    let bytes = include_bytes!("../../../fixtures/pdf/raster.pdf").to_vec();
    assert_eq!(
        receive(worker.probe(b"garbage".to_vec()).unwrap()),
        Err(RenderError::Pdf(PdfError::Unreadable))
    );
    let metadata = receive(worker.probe(bytes.clone()).unwrap()).unwrap();
    assert_eq!(metadata.pages.len(), 9);
    assert_eq!(metadata.pages[0].label.as_deref(), Some("red"));
    assert_eq!(metadata.pages[2].label.as_deref(), Some("notes:1"));
    for index in [1, 3, 4, 6, 7] {
        assert_eq!(
            (
                metadata.pages[index].width_points,
                metadata.pages[index].height_points
            ),
            (640.0, 180.0)
        );
    }
    assert_eq!(metadata.pages[3].rotation_degrees, 90);
    receive(
        worker
            .load_active(DocumentGeneration(1), bytes.clone())
            .unwrap(),
    )
    .unwrap();
    // Task09 runtime budgets update on this same native worker, preserving
    // the active generation and byte-identical PDFium output.
    let cached_request = request(0, NormalizedRect::FULL, 640, 360);
    let cached_frame = |worker: &RenderHandle| {
        worker.submit(cached_request).unwrap();
        worker
            .results()
            .recv_timeout(Duration::from_secs(15))
            .unwrap()
            .outcome
            .unwrap()
    };
    let first = cached_frame(&worker);
    assert!(!first.cache_hit);
    assert!(cached_frame(&worker).cache_hit);
    let budgets = CacheBudgets {
        l2_bytes: 1,
        l3_bytes: 1,
    };
    let ack = receive(worker.set_cache_budgets(budgets).unwrap()).unwrap();
    assert_eq!(ack.budgets, budgets);
    assert_eq!((ack.l2_used_bytes, ack.l3_used_bytes), (0, 0));
    let again = cached_frame(&worker);
    assert!(!again.cache_hit);
    assert_eq!(again.frame.pixels_bgra(), first.frame.pixels_bgra());
    receive(worker.set_cache_budgets(CacheBudgets::default()).unwrap()).unwrap();
    for (invalid, expected) in [
        (
            request(usize::MAX, NormalizedRect::FULL, 320, 180),
            PdfError::PageOutOfRange,
        ),
        (
            request(0, NormalizedRect::FULL, 0, 180),
            PdfError::InvalidTarget,
        ),
        (
            request(0, NormalizedRect::FULL, 32769, 180),
            PdfError::InvalidTarget,
        ),
        (
            request(
                0,
                NormalizedRect {
                    w: 0.0,
                    ..NormalizedRect::FULL
                },
                320,
                180,
            ),
            PdfError::InvalidCrop,
        ),
    ] {
        let id = worker.submit(invalid).unwrap();
        let event = worker
            .results()
            .recv_timeout(Duration::from_secs(15))
            .unwrap();
        assert_eq!((event.id, event.request), (id, invalid));
        assert_eq!(event.outcome.unwrap_err(), RenderError::Pdf(expected));
    }
    let thumbnail = RenderRequest::thumbnail(
        DocumentGeneration(1),
        &metadata.pages[1],
        NormalizedRect::LEFT_HALF,
        0,
    )
    .unwrap();
    assert_eq!(render(&worker, thumbnail).frame.height(), THUMBNAIL_HEIGHT);
    let red = render(&worker, request(0, NormalizedRect::FULL, 320, 180));
    assert_eq!(red.frame.stride(), 1280);
    assert_eq!(&red.frame.pixels_bgra()[..4], &[0, 0, 255, 255]);
    assert!(
        red.frame
            .pixels_bgra()
            .as_chunks::<4>()
            .0
            .iter()
            .all(|pixel| *pixel == [0, 0, 255, 255])
    );
    let alpha = render(&worker, request(5, NormalizedRect::FULL, 320, 180));
    let pixel = &alpha.frame.pixels_bgra()[..4];
    assert!(
        pixel[0].abs_diff(128) <= 1
            && pixel[1].abs_diff(128) <= 1
            && pixel[2] == 255
            && pixel[3] == 255,
        "alpha composite {pixel:?}"
    );
    assert_eq!(&alpha.frame.pixels_bgra()[200 * 4..200 * 4 + 4], &[255; 4]);
    for page in [1, 3, 4, 6, 7] {
        let full = render(&worker, request(page, NormalizedRect::FULL, 1280, 360));
        let red = [0, 0, 255, 255];
        let green = [0, 255, 0, 255];
        let blue = [255, 0, 0, 255];
        let yellow = [0, 255, 255, 255];
        let expected = match page {
            1 | 4 => [red, green, blue, yellow],
            3 => [blue, red, yellow, green],
            6 => [yellow, blue, green, red],
            7 => [green, yellow, red, blue],
            _ => unreachable!(),
        };
        for ((x, y), expected) in [(10, 10), (1270, 10), (10, 350), (1270, 350)]
            .into_iter()
            .zip(expected)
        {
            let corner = y * full.frame.stride() + x * 4;
            assert_eq!(
                &full.frame.pixels_bgra()[corner..corner + 4],
                &expected,
                "independent orientation sentinel page {page}, ({x},{y})"
            );
        }
        assert!(
            full.frame
                .pixels_bgra()
                .as_chunks::<4>()
                .0
                .iter()
                .filter(|pixel| pixel[..3] == [0, 0, 0])
                .count()
                > 100,
            "embedded glyphs must render black"
        );
        let left = render(&worker, request(page, NormalizedRect::LEFT_HALF, 640, 360));
        let right = render(&worker, request(page, NormalizedRect::RIGHT_HALF, 640, 360));
        compare(&full, &left, 0, 0);
        compare(&full, &right, 640, 0);
    }
    let full = render(&worker, request(2, NormalizedRect::FULL, 640, 720));
    for (y, origin) in [(0.0, 0), (0.5, 360)] {
        let half = render(
            &worker,
            request(
                2,
                NormalizedRect {
                    x: 0.0,
                    y,
                    w: 1.0,
                    h: 0.5,
                },
                640,
                360,
            ),
        );
        compare(&full, &half, 0, origin);
    }
    // Failed load and aborted candidate leave active generation renderable.
    assert!(
        receive(
            worker
                .prepare_candidate(DocumentGeneration(2), vec![])
                .unwrap()
        )
        .is_err()
    );
    render(&worker, request(0, NormalizedRect::FULL, 320, 180));
    receive(
        worker
            .prepare_candidate(DocumentGeneration(3), bytes)
            .unwrap(),
    )
    .unwrap();
    let ready = receive(
        worker
            .pre_render_candidate(
                DocumentGeneration(3),
                vec![RenderRequest {
                    generation: DocumentGeneration(3),
                    ..request(0, NormalizedRect::FULL, 320, 180)
                }],
            )
            .unwrap(),
    )
    .unwrap();
    assert_eq!(ready.frames[0].request.generation, DocumentGeneration(3));
    receive(worker.commit_candidate(ready).unwrap()).unwrap();
    receive(worker.retire_generation(DocumentGeneration(1)).unwrap()).unwrap();
    assert!(
        red.frame
            .pixels_bgra()
            .as_chunks::<4>()
            .0
            .iter()
            .all(|pixel| *pixel == [0, 0, 255, 255]),
        "retained red pixels must not be overwritten by later alpha/art pages"
    );
    worker.shutdown().unwrap();
    let changed = RuntimeConfig {
        font_directories: vec![std::env::temp_dir()],
        ..config
    };
    assert!(matches!(
        RenderHandle::spawn(changed, CacheBudgets::default()),
        Err(RenderError::Pdf(PdfError::RuntimeConfigurationChanged))
    ));
}
