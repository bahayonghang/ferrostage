struct CropRaster {
    metadata: DocumentMetadata,
    fail: bool,
}

impl ferrostage_pdf::raster::RasterBackend for CropRaster {
    fn probe(&mut self, _: Vec<u8>) -> Result<DocumentMetadata, ferrostage_pdf::PdfError> {
        Ok(self.metadata.clone())
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
        page: PhysicalPage,
        crop: ferrostage_core::NormalizedRect,
        _: ferrostage_pdf::PixelSize,
    ) -> Result<ferrostage_core::BgraFrame, ferrostage_pdf::PdfError> {
        if self.fail {
            return Err(ferrostage_pdf::PdfError::Unreadable);
        }
        // Encode the exact half and page the single worker was asked to read.
        ferrostage_core::BgraFrame::new(
            1,
            1,
            4,
            vec![page.0 as u8, (crop.x * 2.) as u8, (crop.w * 2.) as u8, 255],
        )
        .map_err(|_| ferrostage_pdf::PdfError::Unreadable)
    }
    fn remove(&mut self, _: DocumentGeneration) {}
}

fn crop_gate_fixture(cx: &mut TestAppContext, fail: bool) -> (Entity<Shell>, AnyWindowHandle) {
    let (shell, presenter) = startable_fixture(cx);
    shell.update(cx, |s, cx| {
        let generation = DocumentGeneration(3);
        let mut session = s.workbench.document.clone().unwrap();
        for (page, raw) in session
            .metadata
            .pages
            .iter_mut()
            .zip(&mut session.structure.pages)
        {
            page.label = None;
            page.width_points = 1600.;
            page.height_points = 480.;
            raw.geometry.bounds = [0., 0., 1600., 480.];
        }
        let metadata = session.metadata.clone();
        s.renderer = Ok(RenderHandle::with_backend(
            move || Box::new(CropRaster { metadata, fail }),
            Default::default(),
        )
        .unwrap());
        s.renderer
            .as_ref()
            .unwrap()
            .load_active(generation, vec![1])
            .unwrap()
            .recv()
            .unwrap()
            .unwrap();
        let session =
            crate::workbench::DocumentSession::build(session.metadata, session.structure).unwrap();
        s.state
            .apply_at(
                Command::LoadMappedDocument(session.document.descriptor(generation).unwrap()),
                Instant::now(),
            )
            .unwrap();
        s.workbench.document = Some(session);
        assert!(s.crop_needs_confirmation());
        cx.notify();
    });
    redraw(presenter, cx);
    (shell, presenter)
}

fn settle_crop_gate(shell: &Entity<Shell>, cx: &mut TestAppContext) -> crop::CropPreview {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let preview = shell.update(cx, |s, _| {
            s.update_render();
            s.crop_preview().unwrap()
        });
        if !preview.pending {
            return preview;
        }
        assert!(
            Instant::now() < deadline,
            "timed out waiting for crop previews"
        );
        std::thread::sleep(Duration::from_millis(1));
    }
}

#[gpui::test]
fn crop_cancel_and_escape_at_start_leave_only_the_presenter(cx: &mut TestAppContext) {
    let (shell, presenter) = crop_gate_fixture(cx, false);
    interrupt(&shell, 8, cx);
    for (start, escape) in [(Intent::Start, false), (Intent::Resume, true)] {
        shell.update(cx, |s, cx| s.dispatch(start, cx));
        let preview = settle_crop_gate(&shell, cx);
        assert!(preview.can_confirm && preview.from_start);
        assert_eq!(cx.windows(), vec![presenter]);
        shell.read_with(cx, |s, _| {
            assert!(s.pending.is_none() && s.audience.is_none());
            assert_eq!(s.state.stage(), SessionStage::Ready);
        });
        redraw(presenter, cx);
        if escape {
            cx.simulate_keystrokes(presenter, "escape");
        } else {
            click_control(presenter, "crop-cancel", cx);
        }
        cx.run_until_parked();
        shell.read_with(cx, |s, cx| {
            assert!(s.crop.is_none() && s.modal.is_none());
            assert!(s.pending.is_none() && s.audience.is_none());
            assert!(!s.crop_confirmed);
            assert_eq!(s.snapshot.read(cx).stage, AudienceStage::Curtain);
            assert_eq!(s.state.stage(), SessionStage::Ready);
            assert_eq!(s.resumable_elapsed().unwrap().as_secs(), 8);
        });
        assert_eq!(cx.windows(), vec![presenter]);
    }
}

#[gpui::test]
fn crop_manual_preview_changes_only_after_matching_confirmation(cx: &mut TestAppContext) {
    let (shell, presenter) = crop_gate_fixture(cx, false);
    shell.update(cx, |s, cx| {
        s.state
            .apply_at(
                Command::UpdateLaserPointer(Some((0.5, 0.5))),
                Instant::now(),
            )
            .unwrap();
        s.dispatch(Intent::OpenCrop, cx);
    });
    let whole = settle_crop_gate(&shell, cx);
    assert!(whole.can_confirm && whole.notes_frames.is_empty());
    assert_eq!(whole.public_frame.unwrap().pixels_bgra(), &[0, 0, 2, 255]);
    shell.update(cx, |s, cx| {
        s.dispatch(
            Intent::CropLayout {
                id: whole.id,
                layout: ferrostage_document::NotesLayout::Right,
            },
            cx,
        )
    });
    let split = settle_crop_gate(&shell, cx);
    shell.update(cx, |s, cx| {
        s.dispatch(
            Intent::CropSample {
                id: split.id,
                delta: 1,
            },
            cx,
        );
        s.dispatch(Intent::ConfirmCrop(split.id), cx);
        assert_eq!(s.state.current_page(), PhysicalPage(0));
        assert_eq!(s.state.slide_crop(), ferrostage_core::NormalizedRect::FULL);
        assert!(s.state.has_annotations());
    });
    let next = settle_crop_gate(&shell, cx);
    assert!(next.changing_with_annotations && next.can_confirm);
    assert_eq!(next.public_frame.unwrap().pixels_bgra(), &[1, 0, 1, 255]);
    assert_eq!(next.notes_frames[0].pixels_bgra(), &[1, 1, 1, 255]);
    shell.update(cx, |s, cx| {
        assert!(
            s.workbench
                .frames
                .iter()
                .all(|(surface, _)| !matches!(surface, crate::workbench::Surface::CropSample(_)))
        );
        s.dispatch(Intent::ConfirmCrop(next.id), cx);
        assert_eq!(
            s.state.slide_crop(),
            ferrostage_core::NormalizedRect::LEFT_HALF
        );
        assert_eq!(s.state.current_page(), PhysicalPage(0));
        assert!(s.crop_confirmed && !s.state.has_annotations());
        assert!(s.crop.is_none() && s.pending.is_none() && s.audience.is_none());
        s.dispatch(Intent::Start, cx);
        assert_eq!(s.pending_start(), Some(StartTimer::Restart));
    });
    assert!(cx.windows().contains(&presenter));
}

#[gpui::test]
fn crop_preview_failure_blocks_start_and_preserves_state(cx: &mut TestAppContext) {
    let (shell, presenter) = crop_gate_fixture(cx, true);
    shell.update(cx, |s, cx| s.dispatch(Intent::Start, cx));
    let failed = settle_crop_gate(&shell, cx);
    assert!(failed.error.is_some() && !failed.can_confirm);
    shell.update(cx, |s, cx| {
        s.dispatch(Intent::ConfirmCrop(failed.id), cx);
        assert!(s.crop.is_some() && !s.crop_confirmed);
        assert!(s.pending.is_none() && s.audience.is_none());
        assert_eq!(s.state.slide_crop(), ferrostage_core::NormalizedRect::FULL);
        s.dispatch(Intent::CancelCrop(failed.id), cx);
        assert_eq!(s.state.stage(), SessionStage::Ready);
    });
    assert_eq!(cx.windows(), vec![presenter]);
}

#[gpui::test]
fn crop_confirming_existing_rule_preserves_ink_and_resume_across_stop(cx: &mut TestAppContext) {
    let (shell, _) = crop_gate_fixture(cx, false);
    interrupt(&shell, 8, cx);
    shell.update(cx, |s, cx| {
        s.state
            .apply_at(
                Command::UpdateLaserPointer(Some((0.5, 0.5))),
                Instant::now(),
            )
            .unwrap();
        s.dispatch(Intent::Resume, cx);
    });
    let whole = settle_crop_gate(&shell, cx);
    shell.update(cx, |s, cx| {
        s.dispatch(Intent::ConfirmCrop(whole.id), cx);
        assert!(s.crop_confirmed && s.state.has_annotations());
        assert_eq!(s.pending_start(), Some(StartTimer::Resume));
    });
    commit_pending_start(&shell, cx);
    shell.update(cx, |s, cx| {
        assert!(s.state.elapsed_at(Instant::now()).as_secs() >= 8);
        s.stop(cx);
        s.dispatch(Intent::Resume, cx);
        assert!(s.crop.is_none());
        assert_eq!(s.pending_start(), Some(StartTimer::Resume));
    });
}
