use super::*;
use ferrostage_core::{BlankMode, PresentationCommand as C};
use std::time::Instant;

struct CountingRaster(std::sync::Arc<std::sync::atomic::AtomicUsize>);
fn hundred_pages() -> DocumentMetadata {
    DocumentMetadata {
        pages: (0..100)
            .map(|index| ferrostage_pdf::PageMetadata {
                page: PhysicalPage(index),
                width_points: 640.,
                height_points: 360.,
                rotation_degrees: 0,
                label: None,
            })
            .collect(),
    }
}
impl ferrostage_pdf::raster::RasterBackend for CountingRaster {
    fn probe(&mut self, _: Vec<u8>) -> Result<DocumentMetadata, ferrostage_pdf::PdfError> {
        Ok(hundred_pages())
    }
    fn load(
        &mut self,
        _: DocumentGeneration,
        bytes: Vec<u8>,
    ) -> Result<DocumentMetadata, ferrostage_pdf::PdfError> {
        self.probe(bytes)
    }
    fn render(
        &mut self,
        _: DocumentGeneration,
        _: PhysicalPage,
        _: NormalizedRect,
        size: PixelSize,
    ) -> Result<BgraFrame, ferrostage_pdf::PdfError> {
        self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        BgraFrame::new(
            size.width,
            size.height,
            size.width as usize * 4,
            vec![255; size.width as usize * size.height as usize * 4],
        )
        .map_err(|_| ferrostage_pdf::PdfError::InvalidBitmap)
    }
    fn remove(&mut self, _: DocumentGeneration) {}
}

#[test]
fn closed_grid_schedules_no_thumbnails_and_repeated_schedule_submits_nothing() {
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
    use std::time::Duration;
    let calls = Arc::new(AtomicUsize::new(0));
    let backend_calls = calls.clone();
    let worker = RenderHandle::with_backend(
        move || Box::new(CountingRaster(backend_calls)),
        ferrostage_render::CacheBudgets::default(),
    )
    .unwrap();
    worker
        .load_active(DocumentGeneration(1), vec![1])
        .unwrap()
        .recv()
        .unwrap()
        .unwrap();
    let metadata = hundred_pages();
    let structure = DocumentStructure {
        pages: metadata
            .pages
            .iter()
            .map(|page| ferrostage_pdf::structure::PageStructure {
                page: page.page,
                geometry: ferrostage_pdf::structure::PageGeometry {
                    bounds: [0., 0., 640., 360.],
                    rotation_degrees: 0,
                },
                media: vec![],
            })
            .collect(),
        diagnostics: vec![],
    };
    let mut workbench = Workbench {
        document: Some(DocumentSession::build(metadata, structure).unwrap()),
        presenter_target: Some(PixelSize {
            width: 32,
            height: 18,
        }),
        ..Default::default()
    };
    let state = PresentationState::new(100).unwrap();
    let settings = ferrostage_settings::PresenterSettings {
        show_thumbnail_strip: true,
        ..Default::default()
    };
    fn settle(workbench: &mut Workbench, worker: &RenderHandle) {
        let deadline = Instant::now() + Duration::from_secs(5);
        while !workbench.pending.is_empty() {
            assert!(workbench.poll(worker).is_empty());
            assert!(Instant::now() < deadline, "raster worker did not settle");
            std::thread::yield_now();
        }
    }
    workbench.schedule(&worker, &state, &settings).unwrap();
    assert!(
        !workbench
            .requested_surfaces()
            .iter()
            .any(|s| matches!(s, Surface::Thumbnail(_)))
    );
    settle(&mut workbench, &worker);
    let closed_bytes = workbench.bytes();
    let closed_calls = calls.load(Ordering::SeqCst);
    let epoch = workbench.epoch;
    workbench.schedule(&worker, &state, &settings).unwrap();
    assert!(workbench.pending.is_empty());
    assert_eq!(workbench.epoch, epoch);
    assert_eq!(calls.load(Ordering::SeqCst), closed_calls);
    workbench.grid = true;
    workbench.schedule(&worker, &state, &settings).unwrap();
    assert_eq!(
        workbench
            .requested_surfaces()
            .iter()
            .filter(|s| matches!(s, Surface::Thumbnail(_)))
            .count(),
        16
    );
    settle(&mut workbench, &worker);
    assert!(workbench.bytes() > closed_bytes);
    let open_bytes = workbench.bytes();
    workbench.grid = false;
    workbench.schedule(&worker, &state, &settings).unwrap();
    settle(&mut workbench, &worker);
    assert_eq!(workbench.bytes(), closed_bytes);
    assert!(
        !workbench
            .frames
            .iter()
            .any(|(s, _)| matches!(s, Surface::Thumbnail(_)))
    );
    println!(
        "100-page scheduling: closed thumbnails=0 bytes={closed_bytes}; open thumbnails=16 bytes={open_bytes}; closed again bytes={}",
        workbench.bytes()
    );
}

/// Explicit native gate: cargo test -p ferrostage-ui --features pdfium --lib
/// native_perf100_grid_retention -- --nocapture
#[cfg(feature = "pdfium")]
#[test]
fn native_perf100_grid_retention() {
    use std::time::Duration;
    let worker = RenderHandle::spawn(
        ferrostage_pdf::RuntimeConfig {
            library_path: std::env::var_os("FERROSTAGE_TEST_PDFIUM")
                .expect("native suite requires FERROSTAGE_TEST_PDFIUM")
                .into(),
            font_directories: vec![],
        },
        ferrostage_render::CacheBudgets::default(),
    )
    .unwrap();
    let bytes = include_bytes!("../../../../fixtures/pdf/perf-100.pdf").to_vec();
    let structure = ferrostage_pdf::extract_structure(&bytes).unwrap();
    let metadata = worker
        .load_active(DocumentGeneration(1), bytes)
        .unwrap()
        .recv_timeout(Duration::from_secs(30))
        .unwrap()
        .unwrap();
    assert_eq!(metadata.pages.len(), 100);
    let session = DocumentSession::build(metadata, structure).unwrap();
    let mut state = PresentationState::idle();
    state
        .apply_at(
            C::LoadMappedDocument(session.document.descriptor(DocumentGeneration(1)).unwrap()),
            Instant::now(),
        )
        .unwrap();
    let mut workbench = Workbench {
        document: Some(session),
        presenter_target: Some(PixelSize {
            width: 960,
            height: 540,
        }),
        ..Default::default()
    };
    let mut settings = ferrostage_settings::PresenterSettings {
        show_thumbnail_strip: true,
        grid_visible_pages: 4,
        ..Default::default()
    };
    let mut measurements = Vec::new();
    // The first case models the removed four-thumbnail strip with the same
    // request keys and real PDFium pixels. It is not an old-executable timing run.
    for (name, grid, visible) in [
        ("old-strip-request-model", true, 4),
        ("closed", false, 4),
        ("open", true, 16),
        ("closed-again", false, 16),
    ] {
        workbench.grid = grid;
        settings.grid_visible_pages = visible;
        workbench.schedule(&worker, &state, &settings).unwrap();
        let submitted = workbench.pending.len();
        let deadline = Instant::now() + Duration::from_secs(30);
        while !workbench.pending.is_empty() {
            assert!(workbench.poll(&worker).is_empty());
            assert!(Instant::now() < deadline, "native render did not settle");
            std::thread::sleep(Duration::from_millis(1));
        }
        let thumbnails = workbench
            .requested_surfaces()
            .iter()
            .filter(|s| matches!(s, Surface::Thumbnail(_)))
            .count();
        println!(
            "{name}: requested={} submitted={submitted} thumbnails={thumbnails} retained_bytes={}",
            workbench.requested.len(),
            workbench.bytes()
        );
        measurements.push((thumbnails, workbench.bytes()));
        workbench.schedule(&worker, &state, &settings).unwrap();
        assert!(
            workbench.pending.is_empty(),
            "unchanged native scheduling submitted duplicate work"
        );
    }
    assert_eq!(
        measurements.iter().map(|m| m.0).collect::<Vec<_>>(),
        [4, 0, 16, 0]
    );
    assert!(measurements[0].1 > measurements[1].1);
    assert!(measurements[2].1 > measurements[1].1);
    assert_eq!(measurements[1].1, measurements[3].1);
}
fn frame(state: &PresentationState) -> ReadyFrame {
    let mut request = request_for(state.generation(), state.current_page(), state.slide_crop());
    request.priority = RenderPriority::Audience;
    ReadyFrame {
        request,
        frame: BgraFrame::new(2, 1, 8, vec![12, 34, 56, 255, 12, 34, 56, 255]).unwrap(),
    }
}
#[test]
fn projection_rejects_every_mismatched_key_and_presenter_notes_priority() {
    let mut state = PresentationState::new(3).unwrap();
    state
        .apply_at(
            C::StartPresentation {
                audience_available: true,
            },
            Instant::now(),
        )
        .unwrap();
    let empty = state.audience_snapshot();
    let good = frame(&state);
    let target = Some(good.request.target_px);
    assert!(
        assemble(empty.clone(), &empty, Some(&good), target, 64 * 1024 * 1024)
            .visual
            .is_some()
    );
    for kind in 0..5 {
        let mut wrong = good.clone();
        match kind {
            0 => wrong.request.generation = DocumentGeneration(9),
            1 => wrong.request.page = PhysicalPage(1),
            2 => wrong.request.crop = NormalizedRect::RIGHT_HALF,
            3 => wrong.request.target_px.width += 1,
            _ => wrong.request.priority = RenderPriority::Presenter,
        };
        assert!(
            assemble(
                empty.clone(),
                &empty,
                Some(&wrong),
                target,
                64 * 1024 * 1024
            )
            .visual
            .is_none()
        );
    }
}
#[test]
fn awaiting_frame_keeps_complete_old_identity_and_immediate_covers() {
    let mut state = PresentationState::new(3).unwrap();
    let now = Instant::now();
    state
        .apply_at(
            C::StartPresentation {
                audience_available: true,
            },
            now,
        )
        .unwrap();
    let good = frame(&state);
    let target = Some(good.request.target_px);
    let old = assemble(
        state.audience_snapshot(),
        &state.audience_snapshot(),
        Some(&good),
        target,
        64 * 1024 * 1024,
    );
    state.apply_at(C::NextStep, now).unwrap();
    state
        .apply_at(C::UpdateLaserPointer(Some((0.4, 0.4))), now)
        .unwrap();
    state
        .apply_at(C::AppendStrokePoint((0.2, 0.3)), now)
        .unwrap();
    let held = assemble(
        state.audience_snapshot(),
        &old,
        None,
        target,
        64 * 1024 * 1024,
    );
    assert_eq!(held.current_page, old.current_page);
    assert_eq!(held.generation, old.generation);
    assert_eq!(held.slide_crop, old.slide_crop);
    assert_eq!(held.visual, old.visual);
    assert!(held.strokes.is_empty());
    assert!(held.laser_pos_norm.is_none());
    state
        .apply_at(C::ToggleBlank(BlankMode::White), now)
        .unwrap();
    assert_eq!(
        assemble(
            state.audience_snapshot(),
            &old,
            None,
            target,
            64 * 1024 * 1024
        )
        .blank_mode,
        Some(BlankMode::White)
    );
    state.apply_at(C::StopPresentation, now).unwrap();
    let curtain = assemble(
        state.audience_snapshot(),
        &old,
        None,
        target,
        64 * 1024 * 1024,
    );
    assert_eq!(curtain.stage, AudienceStage::Curtain);
    assert!(curtain.visual.is_none());
}
#[test]
fn direct_jumps_hold_previous_complete_projection_until_matching_frame() {
    let now = Instant::now();
    let mut state = PresentationState::new(20).unwrap();
    state
        .apply_at(
            C::StartPresentation {
                audience_available: true,
            },
            now,
        )
        .unwrap();
    let good = frame(&state);
    let target = Some(good.request.target_px);
    let old = assemble(
        state.audience_snapshot(),
        &state.audience_snapshot(),
        Some(&good),
        target,
        64 * 1024 * 1024,
    );
    for page in 1..=10 {
        state.apply_at(C::JumpTo(PhysicalPage(page)), now).unwrap();
        assert_eq!(
            assemble(
                state.audience_snapshot(),
                &old,
                Some(&good),
                target,
                64 * 1024 * 1024
            ),
            old
        );
    }
    assert_eq!(
        assemble(
            state.audience_snapshot(),
            &old,
            Some(&good),
            target,
            64 * 1024 * 1024
        )
        .current_page,
        PhysicalPage(0)
    );
    let committed = frame(&state);
    assert_eq!(
        assemble(
            state.audience_snapshot(),
            &old,
            Some(&committed),
            target,
            64 * 1024 * 1024
        )
        .current_page,
        PhysicalPage(10)
    );
}

#[test]
fn audience_target_respects_its_half_of_the_gpu_budget() {
    let budget = 8 * 1024 * 1024;
    let target = budgeted_target(
        PixelSize {
            width: 3840,
            height: 2160,
        },
        budget,
    );
    assert!(target.width as usize * target.height as usize * 4 <= budget);
    assert!((target.width as f64 / target.height as f64 - 16. / 9.).abs() < 0.002);
}
