use super::*;
use crate::shell::tests::{input_presenter, redraw};
use ferrostage_core::NormalizedRect;
use ferrostage_pdf::{PixelSize, RuntimeConfig};

fn step(shell: &Entity<Shell>, cx: &mut gpui::TestAppContext) {
    let windows = shell.update(cx, |shell, cx| {
        shell.poll_import(cx);
        shell.poll_reload(cx);
        shell.publish(false, cx);
        cx.notify();
        [
            shell.presenter.as_ref().map(|w| w.handle),
            shell.audience.as_ref().map(|w| w.handle),
        ]
    });
    for window in windows.into_iter().flatten() {
        redraw(window, cx);
    }
    cx.run_until_parked();
    std::thread::sleep(Duration::from_millis(2));
}
fn until(shell: &Entity<Shell>, cx: &mut gpui::TestAppContext, condition: impl Fn(&Shell) -> bool) {
    let deadline = Instant::now() + Duration::from_secs(20);
    while !shell.read_with(cx, |shell, _| condition(shell)) {
        step(shell, cx);
        assert!(
            Instant::now() < deadline,
            "reload stalled: {:?}",
            shell.read_with(cx, |s, _| s.error.clone())
        );
    }
}
#[gpui::test]
fn real_pdfium_two_window_commit_obsolete_ack_rollback_and_latest_state(
    cx: &mut gpui::TestAppContext,
) {
    // pdfium-render owns process-global bindings; this native integration gets
    // its own process, not a second PDFium initialization in the UI test process.
    if std::env::var_os("FERROSTAGE_RELOAD_TEST_CHILD").is_none() {
        let status = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "shell::reload::tests::real_pdfium_two_window_commit_obsolete_ack_rollback_and_latest_state", "--nocapture"])
            .env("FERROSTAGE_RELOAD_TEST_CHILD", "1").status().unwrap();
        assert!(status.success());
        return;
    }
    let bytes = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/notes-functional/pympress-notes.pdf"
    ))
    .to_vec();
    let structure = ferrostage_pdf::extract_structure(&bytes).unwrap();
    let worker = RenderHandle::spawn(
        RuntimeConfig {
            library_path: std::env::var_os("FERROSTAGE_TEST_PDFIUM")
                .expect("pinned PDFium required")
                .into(),
            font_directories: vec![],
        },
        Default::default(),
    )
    .unwrap();
    let metadata = worker
        .load_active(DocumentGeneration(1), bytes.clone())
        .unwrap()
        .recv()
        .unwrap()
        .unwrap();
    let session = DocumentSession::build(metadata, structure.clone()).unwrap();
    let target = PixelSize {
        width: 64,
        height: 36,
    };
    let request = ferrostage_render::RenderRequest {
        generation: DocumentGeneration(1),
        page: PhysicalPage(0),
        crop: ferrostage_document::slide_crop(session.document.layout),
        target_px: target,
        priority: RenderPriority::Audience,
        epoch: 0,
    };
    worker.submit(request).unwrap();
    let frame = worker.results().recv().unwrap().outcome.unwrap().frame;
    let (shell, presenter) = input_presenter(cx);
    shell.update(cx, |shell, cx| {
        shell.renderer = Ok(worker);
        shell
            .state
            .apply_at(
                Command::LoadMappedDocument(
                    session.document.descriptor(DocumentGeneration(1)).unwrap(),
                ),
                Instant::now(),
            )
            .unwrap();
        shell
            .state
            .apply_at(
                Command::StartPresentation {
                    audience_available: true,
                },
                Instant::now(),
            )
            .unwrap();
        shell.workbench.document = Some(session);
        shell.workbench.audience_target = Some(target);
        shell.workbench.presenter_target = Some(target);
        shell.workbench.frames = vec![
            (
                Surface::Audience,
                ReadyFrame {
                    request,
                    frame: frame.clone(),
                },
            ),
            (
                Surface::Current,
                ReadyFrame {
                    request: ferrostage_render::RenderRequest {
                        priority: RenderPriority::Presenter,
                        ..request
                    },
                    frame,
                },
            ),
        ];
        shell.publish(false, cx);
        shell.audience = Some(
            shell
                .open(
                    ferrostage_core::MonitorId("headless-test-display".into()),
                    false,
                    true,
                    cx,
                )
                .unwrap(),
        );
        cx.notify();
    });
    redraw(presenter, cx);
    cx.run_until_parked();
    shell.update(cx, |shell, _| {
        shell.reload.attempt = 1;
        shell.reload.bytes = Some(bytes.clone());
        shell.reload.structure = Some(structure.clone());
    });
    until(&shell, cx, |s| {
        s.reload
            .pending
            .as_ref()
            .is_some_and(|p| matches!(p.phase, Phase::Commit(_)))
    });
    let old = shell.read_with(cx, |shell, cx| shell.snapshot.read(cx).clone());
    assert_eq!(old.generation, DocumentGeneration(1));
    assert!(old.visual.is_some());
    assert!(old.staging.is_some());
    shell.update(cx, |shell, cx| {
        // A filesystem event arrives after Commit was sent, before its ack is
        // consumed. Keep the reply; the already committed native candidate must
        // be rolled back, then a fresh generation rebuilt from the latest bytes.
        shell.invalidate_reload(false);
        shell
            .state
            .apply_at(Command::PauseTimer, Instant::now())
            .unwrap();
        shell
            .state
            .apply_at(Command::ToggleBlank(BlankMode::White), Instant::now())
            .unwrap();
        shell
            .state
            .apply_at(Command::AppendStrokePoint((0.2, 0.3)), Instant::now())
            .unwrap();
        shell.publish(false, cx);
    });
    let elapsed = shell.read_with(cx, |s, _| s.state.elapsed_at(Instant::now()));
    until(&shell, cx, |s| {
        s.state.generation().0 >= 3 && s.reload.pending.is_none()
    });
    shell.read_with(cx, |shell, cx| {
        assert!(!shell.reload.failed_worker);
        assert_eq!(shell.state.stage(), SessionStage::Presenting);
        assert_eq!(shell.state.elapsed_at(Instant::now()), elapsed);
        let snapshot = shell.snapshot.read(cx);
        assert_eq!(snapshot.generation, shell.state.generation());
        assert_eq!(snapshot.blank_mode, Some(BlankMode::White));
        assert_eq!(snapshot.strokes[0].points, [(0.2, 0.3)]);
        assert!(snapshot.staging.is_none());
        assert_ne!(
            snapshot.slide_crop,
            NormalizedRect::FULL,
            "notes remain cropped away"
        );
        assert_eq!(snapshot.visual.as_ref().unwrap().page.width(), 64);
    });
    let previous = shell.read_with(cx, |shell, _| shell.state.generation());
    shell.update(cx, |shell, _| {
        shell.invalidate_reload(true);
        shell.reload.bytes = Some(bytes.clone());
        shell.reload.structure = Some(structure.clone());
    });
    until(&shell, cx, |s| {
        s.reload
            .pending
            .as_ref()
            .is_some_and(|p| matches!(p.phase, Phase::Upload))
    });
    shell.update(cx, |shell, _| {
        shell
            .state
            .apply_at(Command::NextStep, Instant::now())
            .unwrap();
        shell.workbench.audience_target = Some(PixelSize {
            width: 96,
            height: 54,
        });
    });
    let page = shell.read_with(cx, |shell, _| shell.state.current_page());
    until(&shell, cx, |s| {
        s.state.generation().0 > previous.0 + 1 && s.reload.pending.is_none()
    });
    shell.read_with(cx, |shell, cx| {
        assert_eq!(shell.state.current_page(), page);
        assert_eq!(shell.snapshot.read(cx).current_page, page);
        assert_eq!(
            shell
                .snapshot
                .read(cx)
                .visual
                .as_ref()
                .unwrap()
                .page
                .width(),
            96
        );
    });
    // A failed upload keeps both old complete projection and worker generation.
    let previous = shell.read_with(cx, |shell, cx| shell.snapshot.read(cx).clone());
    shell.update(cx, |shell, _| {
        shell.invalidate_reload(true);
        shell.reload.bytes = Some(bytes.clone());
        shell.reload.structure = Some(structure.clone());
    });
    until(&shell, cx, |s| {
        s.reload
            .pending
            .as_ref()
            .is_some_and(|p| matches!(p.phase, Phase::Upload))
    });
    shell.update(cx, |shell, cx| {
        shell.staging_status(
            shell.reload.attempt,
            true,
            Err("injected GPU upload error".into()),
            cx,
        );
    });
    until(&shell, cx, |s| s.reload.pending.is_none());
    shell.read_with(cx, |shell, cx| {
        let snapshot = shell.snapshot.read(cx);
        assert_eq!(snapshot.generation, previous.generation);
        assert_eq!(snapshot.visual, previous.visual);
        assert!(snapshot.staging.is_none());
        assert!(shell.error.as_ref().unwrap().contains("injected GPU"));
    });
    // Real notify -> read-only stability worker -> lopdf/PDFium -> two real
    // GPUI view paint callbacks. Only this newly created temporary file is written.
    let dir =
        std::env::temp_dir().join(format!("ferrostage-native-reload-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&dir).unwrap();
    let target_path = dir.join("deck.pdf");
    std::fs::write(&target_path, &bytes).unwrap();
    shell.update(cx, |shell, _| {
        shell.path = Some(target_path.clone());
        shell
            .replace_pdf_watch(&ferrostage_settings::HotReloadSettings::default())
            .unwrap();
        shell.error = None;
    });
    let before = shell.read_with(cx, |s, _| s.state.generation());
    let (write_path, write_bytes) = (target_path.clone(), bytes.clone());
    let writing = std::thread::spawn(move || {
        use std::io::Write;
        let mut file = std::fs::File::create(write_path).unwrap();
        for chunk in write_bytes.chunks(65536) {
            file.write_all(chunk).unwrap();
            file.flush().unwrap();
            std::thread::sleep(Duration::from_millis(50));
        }
    });
    until(&shell, cx, |s| {
        s.state.generation() > before && s.reload.pending.is_none()
    });
    writing.join().unwrap();
    let written = shell.read_with(cx, |s, _| s.state.generation());
    let settle = Instant::now() + Duration::from_millis(450);
    while Instant::now() < settle {
        step(&shell, cx);
    }
    shell.read_with(cx, |shell, cx| {
        assert_eq!(
            shell.state.generation(),
            written,
            "one complete replacement publishes once"
        );
        assert_eq!(shell.snapshot.read(cx).generation, written);
        assert!(shell.snapshot.read(cx).visual.is_some());
        assert!(
            shell.error.is_none(),
            "chunked write must not show parse errors"
        );
    });
    let replacement = dir.join("replacement.tmp");
    std::fs::write(&replacement, &bytes).unwrap();
    std::fs::remove_file(&target_path).unwrap();
    std::fs::rename(replacement, &target_path).unwrap();
    until(&shell, cx, |s| {
        s.state.generation() > written && s.reload.pending.is_none()
    });
    shell.update(cx, |shell, _| {
        shell
            .replace_pdf_watch(&ferrostage_settings::HotReloadSettings {
                enabled: false,
                ..Default::default()
            })
            .unwrap()
    });
    std::fs::remove_dir_all(dir).unwrap();
    // A bad replacement must retain the complete published state and settle its
    // Abort/Retire replies. It must not spin on the same failed bytes.
    let generation = shell.read_with(cx, |s, _| s.state.generation());
    shell.update(cx, |shell, _| {
        shell.invalidate_reload(true);
        shell.reload.bytes = Some(b"invalid PDF\n%%EOF".to_vec());
    });
    until(&shell, cx, |s| {
        s.reload.attempted == Some(s.reload.attempt)
            && s.reload.pending.is_none()
            && s.reload.parsing.is_none()
    });
    assert_eq!(shell.read_with(cx, |s, _| s.state.generation()), generation);
    assert!(shell.read_with(cx, |s, _| s.error.is_none()));
    // A stale EOF can also pass structure inspection yet be rejected by PDFium
    // or by the metadata/structure consistency check. These are input failures,
    // not runtime faults, and must leave the complete projection untouched.
    let previous = shell.read_with(cx, |s, cx| s.snapshot.read(cx).clone());
    for native_reject in [true, false] {
        shell.update(cx, |shell, _| {
            shell.invalidate_reload(true);
            shell.reload.bytes = Some(if native_reject {
                b"invalid PDF\n%%EOF".to_vec()
            } else {
                bytes.clone()
            });
            let mut candidate_structure = structure.clone();
            if !native_reject {
                candidate_structure.pages.clear();
            }
            shell.reload.structure = Some(candidate_structure);
        });
        until(&shell, cx, |s| {
            s.reload.attempted == Some(s.reload.attempt) && s.reload.pending.is_none()
        });
        shell.read_with(cx, |shell, cx| {
            assert!(shell.error.is_none());
            assert_eq!(shell.state.generation(), generation);
            assert_eq!(*shell.snapshot.read(cx), previous);
        });
    }
    // New import is queued while a Commit acknowledgement is outstanding. Its
    // worker Prepare is sent only after rollback and candidate retirement settle.
    shell.update(cx, |shell, _| {
        shell.invalidate_reload(true);
        shell.reload.bytes = Some(bytes.clone());
        shell.reload.structure = Some(structure);
        shell.settings.write_policy = ferrostage_settings::from_toml("schema_version=999")
            .unwrap()
            .write_policy;
    });
    until(&shell, cx, |s| {
        s.reload
            .pending
            .as_ref()
            .is_some_and(|p| matches!(p.phase, Phase::Commit(_)))
    });
    shell.update(cx, |shell, cx| {
        shell.stop(cx);
        shell.begin_import(
            PathBuf::from(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../fixtures/notes-functional/pympress-notes.pdf"
            )),
            cx,
        );
        assert!(shell.reload.queued_import.is_some());
    });
    until(&shell, cx, |s| {
        s.reload.pending.is_none() && !s.loading && s.state.generation() > generation
    });
    assert_eq!(
        std::fs::read(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../fixtures/notes-functional/pympress-notes.pdf"
        ))
        .unwrap(),
        bytes
    );
    shell.update(cx, |shell, _| shell.shutdown_reload());
}

#[test]
fn invalid_pdf_errors_do_not_hide_runtime_failures() {
    use ferrostage_pdf::PdfError;
    for error in [
        PdfError::Unreadable,
        PdfError::EmptyDocument,
        PdfError::InvalidPageGeometry,
    ] {
        assert!(invalid_pdf(&RenderError::Pdf(error)));
    }
    for error in [
        RenderError::WorkerUnavailable,
        RenderError::InvalidRollback,
        RenderError::GenerationInUse,
        RenderError::Pdf(PdfError::LibraryUnavailable),
        RenderError::Pdf(PdfError::RuntimeBusy),
        RenderError::Pdf(PdfError::UnknownGeneration),
        RenderError::Pdf(PdfError::RenderFailed),
        RenderError::Pdf(PdfError::InvalidBitmap),
    ] {
        assert!(!invalid_pdf(&error));
    }
}

#[gpui::test]
fn worker_disconnect_after_commit_owns_the_correct_generation(cx: &mut gpui::TestAppContext) {
    struct DisconnectBackend(DocumentMetadata);
    impl ferrostage_pdf::raster::RasterBackend for DisconnectBackend {
        fn probe(&mut self, _: Vec<u8>) -> Result<DocumentMetadata, ferrostage_pdf::PdfError> {
            panic!("intentional worker disconnect after successful commit");
        }
        fn load(
            &mut self,
            _: DocumentGeneration,
            _: Vec<u8>,
        ) -> Result<DocumentMetadata, ferrostage_pdf::PdfError> {
            Ok(self.0.clone())
        }
        fn render(
            &mut self,
            _: DocumentGeneration,
            _: PhysicalPage,
            _: NormalizedRect,
            target: PixelSize,
        ) -> Result<BgraFrame, ferrostage_pdf::PdfError> {
            Ok(BgraFrame::new(
                target.width,
                target.height,
                target.width as usize * 4,
                vec![255; target.width as usize * target.height as usize * 4],
            )
            .unwrap())
        }
        fn remove(&mut self, _: DocumentGeneration) {}
    }
    let bytes = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/notes-functional/pympress-notes.pdf"
    ))
    .to_vec();
    let structure = ferrostage_pdf::extract_structure(&bytes).unwrap();
    let metadata = DocumentMetadata {
        pages: structure
            .pages
            .iter()
            .map(|page| {
                let (width_points, height_points) = page.geometry.visible_size();
                ferrostage_pdf::PageMetadata {
                    page: page.page,
                    width_points,
                    height_points,
                    rotation_degrees: page.geometry.rotation_degrees,
                    label: None,
                }
            })
            .collect(),
    };
    for obsolete in [true, false] {
        let native_metadata = metadata.clone();
        let worker = RenderHandle::with_backend(
            move || Box::new(DisconnectBackend(native_metadata)),
            Default::default(),
        )
        .unwrap();
        worker
            .load_active(DocumentGeneration(1), bytes.clone())
            .unwrap()
            .recv()
            .unwrap()
            .unwrap();
        let session = DocumentSession::build(metadata.clone(), structure.clone()).unwrap();
        let (shell, presenter) = input_presenter(cx);
        shell.update(cx, |shell, cx| {
            shell.renderer = Ok(worker);
            shell
                .state
                .apply_at(
                    Command::LoadMappedDocument(
                        session.document.descriptor(DocumentGeneration(1)).unwrap(),
                    ),
                    Instant::now(),
                )
                .unwrap();
            shell.workbench.document = Some(session);
            shell.workbench.presenter_target = Some(PixelSize {
                width: 64,
                height: 36,
            });
            shell.reload.attempt = 1;
            shell.reload.bytes = Some(bytes.clone());
            shell.reload.structure = Some(structure.clone());
            shell.publish(false, cx);
        });
        until(&shell, cx, |shell| {
            shell
                .reload
                .pending
                .as_ref()
                .is_some_and(|p| matches!(p.phase, Phase::Commit(_)))
        });
        let candidate = shell.read_with(cx, |shell, _| {
            shell.reload.pending.as_ref().unwrap().generation
        });
        shell.update(cx, |shell, _| {
            // This real worker command runs after Commit. Its injected backend
            // failure closes the channel while the successful Commit reply is
            // still owned by the coordinator.
            let disconnected = shell.renderer.as_ref().unwrap().probe(Vec::new()).unwrap();
            assert!(disconnected.recv_timeout(Duration::from_secs(5)).is_err());
            if obsolete {
                shell.invalidate_reload(false);
            }
        });
        step(&shell, cx);
        shell.read_with(cx, |shell, cx| {
            assert!(
                shell.reload.pending.is_none(),
                "must settle without routing through Abort"
            );
            assert_eq!(shell.reload.failed_worker, obsolete);
            assert_eq!(shell.reload_blocks_render(), obsolete);
            let expected = if obsolete {
                DocumentGeneration(1)
            } else {
                candidate
            };
            assert_eq!(shell.state.generation(), expected);
            assert_eq!(shell.snapshot.read(cx).generation, expected);
            assert_eq!(
                shell.reload.orphans,
                [if obsolete {
                    candidate
                } else {
                    DocumentGeneration(1)
                }]
            );
            assert!(shell.error.is_some());
        });
        shell.update(cx, |shell, _| shell.shutdown_reload());
        presenter
            .update(cx, |_, window, _| window.remove_window())
            .unwrap();
    }
}

#[gpui::test]
fn runtime_watch_toggle_debounce_and_creation_failure_are_transactional(
    cx: &mut gpui::TestAppContext,
) {
    let dir = std::env::temp_dir().join(format!(
        "ferrostage-watch-settings-{}",
        uuid::Uuid::new_v4()
    ));
    std::fs::create_dir(&dir).unwrap();
    let path = dir.join("document.pdf");
    std::fs::write(&path, b"readonly fixture\n%%EOF").unwrap();
    let (shell, _) = input_presenter(cx);
    shell.update(cx, |shell, cx| {
        shell.store = SettingsStore::at(dir.join("settings.toml"));
        shell.path = Some(path.clone());
        shell.dispatch(
            Intent::EditSetting("hot_reload.enabled", "false".into()),
            cx,
        );
        assert!(!shell.settings.settings.hot_reload.enabled);
        assert!(shell.reload.watch.is_none());
        shell.dispatch(
            Intent::EditSetting("hot_reload.debounce_ms", "75".into()),
            cx,
        );
        shell.dispatch(Intent::EditSetting("hot_reload.enabled", "true".into()), cx);
        assert!(shell.reload.watch.is_some());
        assert_eq!(shell.settings.settings.hot_reload.debounce_ms, 75);
        assert_eq!(
            shell.store.load().unwrap().settings.hot_reload,
            shell.settings.settings.hot_reload
        );
        let before = shell.settings.settings.clone();
        shell.path = Some(dir.join("missing-parent").join("document.pdf"));
        shell.dispatch(
            Intent::EditSetting("hot_reload.debounce_ms", "90".into()),
            cx,
        );
        assert_eq!(shell.settings.settings, before);
        assert!(shell.reload.watch.is_some());
        assert!(shell.preferences.error.is_some());
        assert_eq!(
            shell.store.load().unwrap().settings.hot_reload,
            before.hot_reload
        );
        shell.dispatch(
            Intent::EditSetting("hot_reload.enabled", "false".into()),
            cx,
        );
        assert!(shell.reload.watch.is_none());
    });
    assert_eq!(std::fs::read(path).unwrap(), b"readonly fixture\n%%EOF");
    std::fs::remove_dir_all(dir).unwrap();
}
