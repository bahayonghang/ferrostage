// Exercise the actual read -> structure -> single worker -> private preview path.
struct HomePreviewBackend {
    loaded: Option<DocumentGeneration>,
    removals: std::sync::Arc<std::sync::atomic::AtomicUsize>,
}
impl ferrostage_pdf::raster::RasterBackend for HomePreviewBackend {
    fn probe(&mut self, bytes: Vec<u8>) -> Result<DocumentMetadata, ferrostage_pdf::PdfError> {
        let structure = ferrostage_pdf::extract_structure(&bytes)
            .map_err(|_| ferrostage_pdf::PdfError::Unreadable)?;
        Ok(DocumentMetadata {
            pages: structure
                .pages
                .iter()
                .map(|page| ferrostage_pdf::PageMetadata {
                    page: page.page,
                    width_points: page.geometry.visible_size().0,
                    height_points: page.geometry.visible_size().1,
                    rotation_degrees: page.geometry.rotation_degrees,
                    label: None,
                })
                .collect(),
        })
    }
    fn load(
        &mut self,
        generation: DocumentGeneration,
        bytes: Vec<u8>,
    ) -> Result<DocumentMetadata, ferrostage_pdf::PdfError> {
        self.loaded = Some(generation);
        self.probe(bytes)
    }
    fn render(
        &mut self,
        generation: DocumentGeneration,
        _: PhysicalPage,
        _: ferrostage_core::NormalizedRect,
        _: ferrostage_pdf::PixelSize,
    ) -> Result<ferrostage_core::BgraFrame, ferrostage_pdf::PdfError> {
        assert_eq!(self.loaded, Some(generation));
        Ok(ferrostage_core::BgraFrame::new(4, 3, 16, vec![255; 48]).unwrap())
    }
    fn remove(&mut self, generation: DocumentGeneration) {
        if self.loaded == Some(generation) {
            self.loaded = None;
        }
        self.removals
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }
}
fn settle_home_preview(shell: &Entity<Shell>, cx: &mut TestAppContext) {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        cx.run_until_parked();
        shell.update(cx, |s, cx| s.poll_home_preview(cx));
        if shell.read_with(cx, |s, _| {
            s.home_preview.frame.is_some() || s.home_preview.error.is_some()
        }) {
            break;
        }
        assert!(Instant::now() < deadline, "preview did not complete");
        std::thread::sleep(Duration::from_millis(2));
    }
}
#[gpui::test]
fn home_preview_is_private_releases_candidate_and_only_open_imports(cx: &mut TestAppContext) {
    let (shell, handle) = input_presenter(cx);
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/pdf/raster.pdf");
    let removals = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let observed = removals.clone();
    let history = shell.update(cx, |s, cx| {
        s.renderer = Ok(RenderHandle::with_backend(
            move || {
                Box::new(HomePreviewBackend {
                    loaded: None,
                    removals: observed,
                })
            },
            ferrostage_render::CacheBudgets::default(),
        )
        .unwrap());
        s.settings.settings.general.recent_files = vec![
            path.to_string_lossy().into_owned(),
            "missing-preview.pdf".into(),
        ];
        cx.notify();
        s.settings.settings.general.clone()
    });
    redraw(handle, cx);
    click_control(handle, "recent-0", cx);
    settle_home_preview(&shell, cx);
    shell.read_with(cx, |s, cx| {
        assert!(s.home_preview.frame.is_some(), "{:?}", s.home_preview.error);
        assert!(s.home_preview_pending.is_none());
        assert_eq!(s.state.stage(), SessionStage::Idle);
        assert_eq!(s.state.generation(), DocumentGeneration(0));
        assert!(s.path.is_none() && s.workbench.document.is_none() && s.audience.is_none());
        assert!(s.snapshot.read(cx).visual.is_none());
        assert_eq!(s.settings.settings.general, history);
    });
    // An ordered worker acknowledgement proves the abort released the candidate.
    shell.read_with(cx, |s, _| {
        s.renderer
            .as_ref()
            .unwrap()
            .set_cache_budgets(ferrostage_render::CacheBudgets::default())
            .unwrap()
            .recv_timeout(Duration::from_secs(2))
            .unwrap()
            .unwrap();
    });
    assert_eq!(removals.load(std::sync::atomic::Ordering::SeqCst), 1);
    cx.simulate_window_resize(handle, size(px(1600.), px(1000.)));
    redraw(handle, cx);
    click_control(handle, "recent-1", cx);
    shell.read_with(cx, |s, _| {
        assert!(
            s.home_preview.frame.is_none(),
            "old pixels must disappear immediately"
        )
    });
    settle_home_preview(&shell, cx);
    shell.read_with(cx, |s, _| {
        assert_eq!(s.home_preview.error, Some("home-preview-read-error"))
    });
    // Start two reads without waiting. Only the latest path may publish.
    shell.update(cx, |s, cx| {
        s.dispatch(Intent::HomePreview(path.clone()), cx);
        s.dispatch(
            Intent::HomePreview(PathBuf::from("missing-preview.pdf")),
            cx,
        );
    });
    settle_home_preview(&shell, cx);
    shell.read_with(cx, |s, _| {
        assert_eq!(
            s.home_preview.path,
            Some(PathBuf::from("missing-preview.pdf"))
        );
        assert!(s.home_preview.frame.is_none());
        assert_eq!(s.settings.settings.general, history);
    });
    shell.update(cx, |s, cx| {
        s.dispatch(Intent::HomePreview(path.clone()), cx)
    });
    settle_home_preview(&shell, cx);
    redraw(handle, cx);
    click_control(handle, "home-open-selected", cx);
    let deadline = Instant::now() + Duration::from_secs(5);
    while shell.read_with(cx, |s, _| s.state.stage() == SessionStage::Idle) {
        cx.run_until_parked();
        shell.update(cx, |s, cx| s.poll_import(cx));
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(2));
    }
    shell.read_with(cx, |s, _| {
        assert_eq!(s.state.stage(), SessionStage::Ready);
        assert_eq!(s.path, Some(path));
        assert!(s.home_preview.path.is_none() && s.home_preview.frame.is_none());
        assert!(s.audience.is_none());
    });
}

#[gpui::test]
fn home_preview_clear_and_shutdown_cancel_foreground_reads(cx: &mut TestAppContext) {
    let (shell, _) = input_presenter(cx);
    shell.update(cx, |s, cx| {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/pdf/raster.pdf");
        s.settings.settings.general.recent_files = vec![path.to_string_lossy().into_owned()];
        s.renderer = Ok(RenderHandle::with_backend(
            || Box::new(SettingsBackend),
            ferrostage_render::CacheBudgets::default(),
        )
        .unwrap());
        s.select_home_preview(path, cx);
        assert!(s.home_preview_task.is_some());
        s.settings.settings.general.recent_files.clear();
        s.poll_home_preview(cx);
        assert!(s.home_preview.path.is_none());
        assert!(s.home_preview_task.is_none());
        s.stop_tasks("home preview test");
    });
    cx.run_until_parked();
    shell.read_with(cx, |s, _| assert!(s.home_preview_pending.is_none()));
}

#[gpui::test]
fn home_preview_failure_is_local_and_retry_is_keyboard_reachable(cx: &mut TestAppContext) {
    let (shell, handle) = input_presenter(cx);
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/pdf/raster.pdf");
    shell.update(cx, |s, cx| {
        s.settings.settings.general.recent_files = vec![path.to_string_lossy().into_owned()];
        s.renderer = Ok(RenderHandle::with_backend(
            || Box::new(SettingsBackend),
            ferrostage_render::CacheBudgets::default(),
        )
        .unwrap());
        s.poll_home_preview(cx);
    });
    settle_home_preview(&shell, cx);
    let token = shell.read_with(cx, |s, _| {
        assert_eq!(s.home_preview.error, Some("home-preview-invalid"));
        assert!(s.home_preview_pending.is_none() && s.error.is_none() && s.path.is_none());
        s.home_preview.token
    });
    cx.simulate_window_resize(handle, size(px(320.), px(240.)));
    handle
        .update(cx, |_, window, _| window.activate_window())
        .unwrap();
    redraw(handle, cx);
    for id in [
        "settings",
        "displays",
        "welcome-open",
        "recent-0",
        "home-open-selected",
        "home-preview-retry",
    ] {
        cx.simulate_keystrokes(handle, "tab");
        cx.run_until_parked();
        cx.test_window(handle).simulate_scheduled_frame();
        redraw(handle, cx);
        cx.run_until_parked();
        cx.test_window(handle).simulate_scheduled_frame();
        redraw(handle, cx);
        let mut visual = gpui::VisualTestContext::from_window(handle, cx);
        let control = visual.debug_bounds(id).unwrap();
        assert!(
            control.top() >= px(0.) && control.bottom() <= px(240.),
            "{id}: {control:?}"
        );
    }
    cx.simulate_keystrokes(handle, "enter");
    settle_home_preview(&shell, cx);
    shell.read_with(cx, |s, _| assert!(s.home_preview.token > token));
}
