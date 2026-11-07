fn return_fixture(cx: &mut TestAppContext) -> (Entity<Shell>, AnyWindowHandle) {
    let (shell, presenter) = workbench_fixture(cx);
    shell.update(cx, |s, cx| {
        let old = s.workbench.document.as_ref().unwrap();
        let mut metadata = old.metadata.clone();
        let mut structure = old.structure.clone();
        metadata.pages.push(metadata.pages[3].clone());
        structure.pages.push(structure.pages[3].clone());
        for (index, label) in ["1", "notes:1", "1", "2", "3"].into_iter().enumerate() {
            metadata.pages[index].page = PhysicalPage(index);
            metadata.pages[index].label = Some(label.into());
            structure.pages[index].page = PhysicalPage(index);
        }
        let session = crate::workbench::DocumentSession::build(metadata, structure).unwrap();
        s.state
            .apply_at(
                Command::LoadMappedDocument(
                    session.document.descriptor(s.state.generation()).unwrap(),
                ),
                Instant::now(),
            )
            .unwrap();
        s.workbench.document = Some(session);
        s.apply(Command::NextStep, cx);
        assert_eq!(s.state.current_page(), PhysicalPage(2));
    });
    redraw(presenter, cx);
    (shell, presenter)
}

fn return_current_frame(s: &mut Shell, cx: &mut Context<Shell>) {
    use crate::workbench::{ReadyFrame, Surface};
    let request = s
        .workbench
        .requests(&s.state, &s.settings.settings.presenter)
        .unwrap()
        .into_iter()
        .find(|(surface, _)| *surface == Surface::Current)
        .unwrap()
        .1;
    let width = request.target_px.width;
    let height = request.target_px.height;
    s.workbench
        .frames
        .retain(|(surface, _)| *surface != Surface::Current);
    s.workbench.frames.push((
        Surface::Current,
        ReadyFrame {
            request,
            frame: ferrostage_core::BgraFrame::new(
                width,
                height,
                width as usize * 4,
                vec![request.page.0 as u8 * 30; (width * height * 4) as usize],
            )
            .unwrap(),
        },
    ));
    cx.notify();
}

#[gpui::test]
fn return_first_successful_jump_keeps_exact_overlay_across_followups(cx: &mut TestAppContext) {
    let (shell, presenter) = return_fixture(cx);
    shell.update(cx, |s, cx| {
        s.apply(Command::JumpTo(PhysicalPage(2)), cx);
        assert!(s.return_target().is_none());
        s.apply(Command::JumpTo(PhysicalPage(1)), cx); // private page, invalid
        assert!(s.return_target().is_none());
        s.apply(Command::JumpTo(PhysicalPage(3)), cx);
        assert_eq!(s.return_target(), Some(PhysicalPage(2)));
        assert!(s.return_label().unwrap().contains("2/2"));
        s.apply(Command::JumpTo(PhysicalPage(4)), cx);
        s.apply(Command::PreviousStep, cx);
        assert_eq!(s.return_target(), Some(PhysicalPage(2)));
    });
    redraw(presenter, cx);
    cx.simulate_keystrokes(presenter, "alt-left");
    shell.read_with(cx, |s, _| {
        assert_eq!(s.state.current_page(), PhysicalPage(2));
        assert_eq!(
            s.return_target(),
            Some(PhysicalPage(2)),
            "selection alone cannot consume"
        );
    });
    redraw(presenter, cx); // settle footer reflow before rasterizing the exact current target
    shell.update(cx, return_current_frame);
    redraw(presenter, cx);
    // The first real image creates the measured canvas target (the placeholder
    // has none). Its old fallback-size reply cannot authorize consumption.
    shell.read_with(cx, |s, _| assert!(s.return_target().is_some()));
    shell.update(cx, return_current_frame);
    redraw(presenter, cx);
    shell.read_with(cx, |s, _| {
        assert!(
            s.return_target().is_none(),
            "actual Current paint consumes Ready return"
        )
    });
}

#[gpui::test]
fn return_ready_rejects_late_same_page_and_retired_window_paints(cx: &mut TestAppContext) {
    let (shell, presenter) = return_fixture(cx);
    let (old_token, old_request) = shell.update(cx, |s, cx| {
        s.apply(Command::JumpTo(PhysicalPage(3)), cx);
        s.dispatch(Intent::ReturnPosition, cx);
        return_current_frame(s, cx);
        (
            s.ready_return_token().unwrap(),
            s.workbench
                .frame(crate::workbench::Surface::Current)
                .unwrap()
                .request,
        )
    });
    shell.update(cx, |s, cx| {
        s.apply(Command::JumpTo(PhysicalPage(4)), cx);
        s.apply(Command::JumpTo(PhysicalPage(2)), cx);
        return_current_frame(s, cx);
        s.return_preview_painted(presenter.window_id(), &old_token, old_request, cx);
        assert!(s.return_target().is_some());
        let token = s.ready_return_token().unwrap();
        let request = s
            .workbench
            .frame(crate::workbench::Surface::Current)
            .unwrap()
            .request;
        let other = test_window(s, false, cx);
        s.return_preview_painted(other.handle.window_id(), &token, request, cx);
        assert!(s.return_target().is_some());
    });
    redraw(presenter, cx);
    shell.read_with(cx, |s, _| assert!(s.return_target().is_none()));
}

#[gpui::test]
fn return_presenting_waits_for_current_settled_and_masked_exact_readiness(cx: &mut TestAppContext) {
    let (shell, _presenter) = return_fixture(cx);
    let audience = shell.update(cx, |s, cx| {
        let window = test_window(s, true, cx);
        let handle = window.handle;
        s.audience = Some(window);
        s.apply(
            Command::StartPresentation {
                audience_available: true,
            },
            cx,
        );
        timed_frame(s, cx);
        handle
    });
    redraw(audience, cx);
    shell.update(cx, |s, cx| {
        s.apply(Command::JumpTo(PhysicalPage(3)), cx);
        s.workbench.tool = crate::workbench::Tool::Pen;
        s.apply(Command::PauseTimer, cx);
        s.timed.clock.pause(true, Instant::now());
        s.dispatch(Intent::ReturnPosition, cx);
        assert_eq!(s.state.current_page(), PhysicalPage(2));
        assert!(s.return_target().is_some());
        s.graphics_error = Some("upload failed".into());
        s.publish(false, cx);
        assert!(s.return_target().is_some());
        s.graphics_error = None;
        timed_frame(s, cx);
    });
    redraw(audience, cx);
    shell.read_with(cx, |s, _| {
        assert!(s.return_target().is_none());
        assert!(s.state.is_timer_paused());
        assert!(s.autoplay_paused());
        assert_eq!(s.workbench.tool, crate::workbench::Tool::Pen);
    });
    shell.update(cx, |s, cx| {
        s.apply(Command::JumpTo(PhysicalPage(3)), cx);
        s.apply(Command::ToggleBlank(BlankMode::Black), cx);
        s.workbench
            .frames
            .retain(|(surface, _)| *surface != crate::workbench::Surface::Audience);
        s.dispatch(Intent::ReturnPosition, cx);
        assert!(s.return_target().is_some());
        timed_frame(s, cx);
        assert!(s.return_target().is_none());
        assert_eq!(
            s.state.audience_snapshot().blank_mode,
            Some(BlankMode::Black)
        );
        assert!(s.state.is_timer_paused());
        assert!(s.autoplay_paused());
    });
}

#[gpui::test]
fn return_modal_menu_rebinding_and_explicit_clear_use_real_input(cx: &mut TestAppContext) {
    let (shell, presenter) = return_fixture(cx);
    shell.update(cx, |s, cx| s.apply(Command::JumpTo(PhysicalPage(3)), cx));
    redraw(presenter, cx);
    click_control(presenter, "document-options", cx);
    cx.simulate_keystrokes(presenter, "alt-left");
    shell.read_with(cx, |s, _| {
        assert_eq!(s.state.current_page(), PhysicalPage(3))
    });
    cx.simulate_keystrokes(presenter, "escape");
    redraw(presenter, cx);
    cx.simulate_keystrokes(presenter, "g");
    shell.update(cx, |s, cx| s.poll_focus(cx));
    redraw(presenter, cx);
    shell.read_with(cx, |s, _| assert!(s.browser().is_some()));
    cx.simulate_keystrokes(presenter, "alt-left");
    shell.read_with(cx, |s, _| {
        assert_eq!(s.state.current_page(), PhysicalPage(3))
    });
    cx.simulate_keystrokes(presenter, "escape");
    shell.update(cx, |s, cx| {
        s.settings.settings.keybindings.return_position = vec!["Alt+R".into()];
        actions::replace_bindings(&s.settings.settings.keybindings, s.locale, cx);
    });
    cx.simulate_keystrokes(presenter, "alt-left");
    shell.read_with(cx, |s, _| {
        assert_eq!(s.state.current_page(), PhysicalPage(3))
    });
    cx.simulate_keystrokes(presenter, "alt-r");
    shell.read_with(cx, |s, _| {
        assert_eq!(s.state.current_page(), PhysicalPage(2))
    });
    redraw(presenter, cx);
    click_control(presenter, "clear-return-position", cx);
    shell.read_with(cx, |s, _| assert!(s.return_target().is_none()));
}

#[gpui::test]
fn return_stop_resume_timer_reset_and_failed_crop_preserve_anchor(cx: &mut TestAppContext) {
    let (shell, _) = return_fixture(cx);
    shell.update(cx, |s, cx| {
        s.apply(Command::JumpTo(PhysicalPage(3)), cx);
        s.state
            .apply_at(
                Command::StartPresentation {
                    audience_available: true,
                },
                Instant::now() - Duration::from_secs(2),
            )
            .unwrap();
        s.apply(Command::StopPresentation, cx);
        s.apply(
            Command::ResumePresentation {
                audience_available: true,
            },
            cx,
        );
        assert_eq!(s.state.stage(), SessionStage::Presenting);
        assert_eq!(s.return_target(), Some(PhysicalPage(2)));
        s.apply(Command::ResetTimer, cx);
        s.apply(Command::StopPresentation, cx);
        assert_eq!(s.return_target(), Some(PhysicalPage(2)));
        s.apply(
            Command::ResumePresentation {
                audience_available: false,
            },
            cx,
        );
        assert_eq!(s.return_target(), Some(PhysicalPage(2)));
        let old = s.workbench.document.take();
        assert!(
            s.apply_layout_options(ferrostage_document::DocumentOverrides::default())
                .is_err()
        );
        s.workbench.document = old;
        assert_eq!(s.return_target(), Some(PhysicalPage(2)));
        let mut options = s.workbench.document.as_ref().unwrap().document.overrides;
        options.layout = Some(ferrostage_document::NotesLayout::Right);
        s.apply_layout_options(options).unwrap();
        assert!(s.return_target().is_none());
    });
}

#[gpui::test]
fn return_stopped_footer_keeps_resume_return_and_clear_inside_normal_window(
    cx: &mut TestAppContext,
) {
    let (shell, presenter) = return_fixture(cx);
    shell.update(cx, |s, cx| {
        s.locale = crate::locale::Locale::new("zh-CN");
        // Test-platform glyphs are narrower than native Chinese fonts. A
        // supported larger UI size exercises the same constrained footer.
        s.settings.settings.general.ui_font_size = 24;
        s.apply(Command::JumpTo(PhysicalPage(3)), cx);
        s.apply(Command::JumpTo(PhysicalPage(4)), cx);
        s.state
            .apply_at(
                Command::StartPresentation {
                    audience_available: true,
                },
                Instant::now() - Duration::from_secs(2),
            )
            .unwrap();
        s.apply(Command::StopPresentation, cx);
        assert_eq!(s.return_target(), Some(PhysicalPage(2)));
        assert!(s.resumable_elapsed().is_some());
    });
    cx.test_window(presenter)
        .simulate_resize(size(px(1100.), px(760.)));
    redraw(presenter, cx);
    let mut visual = gpui::VisualTestContext::from_window(presenter, cx);
    for name in [
        "start-resume",
        "start-restart",
        "start-mode",
        "previous",
        "next",
        "workbench-progress",
        "return-position",
        "clear-return-position",
    ] {
        let bounds = visual.debug_bounds(name).unwrap();
        assert!(
            bounds.left() >= px(0.)
                && bounds.right() <= px(1100.)
                && bounds.top() >= px(0.)
                && bounds.bottom() <= px(760.),
            "{name} must remain fully visible: {bounds:?}"
        );
        assert!(bounds.size.height >= px(44.));
    }
    let current = visual.debug_bounds("current-pane").unwrap();
    let controls = visual.debug_bounds("workbench-controls").unwrap();
    assert!(current.size.width > px(200.) && current.size.height > px(200.));
    assert!(current.bottom() <= controls.top());
    click_control(presenter, "clear-return-position", cx);
    shell.read_with(cx, |s, _| assert!(s.return_target().is_none()));
}

#[gpui::test]
fn return_ready_failed_current_cache_keeps_anchor_then_retries_actual_paint(
    cx: &mut TestAppContext,
) {
    let (shell, presenter) = return_fixture(cx);
    shell.update(cx, |s, cx| s.apply(Command::JumpTo(PhysicalPage(3)), cx));
    redraw(presenter, cx);
    shell.update(cx, return_current_frame);
    redraw(presenter, cx); // commit B's texture before returning to A
    shell.update(cx, |s, cx| {
        s.dispatch(Intent::ReturnPosition, cx);
        s.settings.settings.cache.l1_gpu_mib = 0; // inject capacity refusal with old texture retained
        cx.notify();
    });
    redraw(presenter, cx);
    shell.update(cx, return_current_frame);
    redraw(presenter, cx);
    redraw(presenter, cx);
    shell.read_with(cx, |s, _| {
        assert!(
            s.return_target().is_some(),
            "a ready CPU frame and failed current GPU paint are insufficient"
        )
    });
    let view = presenter.downcast::<Presenter>().unwrap().root(cx).unwrap();
    view.read_with(cx, |view, _| assert!(view.presented_error().is_some()));
    shell.update(cx, |s, cx| {
        s.settings.settings.cache.l1_gpu_mib = 256;
        cx.notify();
    });
    redraw(presenter, cx);
    shell.read_with(cx, |s, _| assert!(s.return_target().is_none()));
}

#[gpui::test]
fn return_new_open_and_home_clear_only_after_success(cx: &mut TestAppContext) {
    let (shell, _) = return_fixture(cx);
    shell.update(cx, |s, cx| {
        s.apply(Command::JumpTo(PhysicalPage(3)), cx);
        let session = s.workbench.document.as_ref().unwrap().clone();
        s.settings.write_policy = ferrostage_settings::from_toml("schema_version=999")
            .unwrap()
            .write_policy;
        s.finish_import("new-open.pdf".into(), DocumentGeneration(3), session, cx);
        assert!(s.return_target().is_none());
        s.apply(Command::JumpTo(PhysicalPage(3)), cx);
        assert!(s.return_target().is_some());
        s.return_home(cx);
        assert_eq!(s.state.stage(), SessionStage::Idle);
        assert!(s.return_position.is_none());
    });
}

#[gpui::test]
fn return_explicit_same_page_button_retries_latched_current_upload(cx: &mut TestAppContext) {
    let (shell, presenter) = return_fixture(cx);
    shell.update(cx, |s, cx| s.apply(Command::JumpTo(PhysicalPage(3)), cx));
    redraw(presenter, cx);
    shell.update(cx, return_current_frame);
    redraw(presenter, cx);
    let old = shell.read_with(cx, |s, _| {
        s.workbench
            .frame(crate::workbench::Surface::Current)
            .unwrap()
            .frame
            .clone()
    });
    shell.update(cx, |s, cx| {
        s.graphics_error = Some("hold while arranging the one-shot owner failure".into());
        s.dispatch(Intent::ReturnPosition, cx);
    });
    redraw(presenter, cx);
    shell.update(cx, return_current_frame);
    let wanted = shell.read_with(cx, |s, _| {
        s.workbench
            .frame(crate::workbench::Surface::Current)
            .unwrap()
            .frame
            .clone()
    });
    presenter
        .downcast::<Presenter>()
        .unwrap()
        .update(cx, |view, window, cx| {
            view.seed_budget_image(old.clone(), 128 * 1024 * 1024, window, cx);
            view.fail_current_upload(wanted.clone(), window, cx);
        })
        .unwrap();
    shell.update(cx, |s, cx| {
        s.graphics_error = None;
        cx.notify();
    });
    redraw(presenter, cx);
    redraw(presenter, cx);
    shell.read_with(cx, |s, _| {
        assert_eq!(s.return_target(), Some(PhysicalPage(2)))
    });
    // Identical redraws do not reattempt the failed desired image.
    redraw(presenter, cx);
    shell.read_with(cx, |s, _| assert!(s.return_target().is_some()));
    let view = presenter.downcast::<Presenter>().unwrap().root(cx).unwrap();
    view.read_with(cx, |view, _| {
        let (frame, error) = view.current_texture_probe();
        assert_eq!(
            frame.unwrap().pixels_bgra().as_ptr(),
            old.pixels_bgra().as_ptr()
        );
        assert!(error.unwrap().contains("one-shot"));
    });
    let attempt = shell.read_with(cx, |s, _| s.return_attempt_token().unwrap());
    click_control(presenter, "return-position", cx);
    shell.read_with(cx, |s, _| {
        assert!(!Rc::ptr_eq(&attempt, &s.return_attempt_token().unwrap()))
    });
    redraw(presenter, cx);
    redraw(presenter, cx);
    let view = presenter.downcast::<Presenter>().unwrap().root(cx).unwrap();
    view.read_with(cx, |view, _| {
        let (frame, error) = view.current_texture_probe();
        assert!(
            error.is_none(),
            "the failed desired upload was actually retried: {error:?}"
        );
        assert_eq!(
            frame.unwrap().pixels_bgra().as_ptr(),
            wanted.pixels_bgra().as_ptr()
        );
    });
    // Removing the error row changes the canvas target. Supply that exact
    // result, as the real render worker does, before accepting the Ready paint.
    shell.update(cx, return_current_frame);
    redraw(presenter, cx);
    shell.read_with(cx, |s, _| {
        assert_eq!(s.state.current_page(), PhysicalPage(2));
        assert!(
            s.return_target().is_none(),
            "explicit retry succeeds without navigation away"
        );
    });
    cx.simulate_keystrokes(presenter, "right");
    shell.read_with(cx, |s, _| {
        assert_eq!(s.state.current_page(), PhysicalPage(3));
        assert!(s.return_target().is_none());
    });
    cx.simulate_keystrokes(presenter, "backspace");
    shell.read_with(cx, |s, _| {
        assert_eq!(s.state.current_page(), PhysicalPage(2));
        assert!(s.return_target().is_none());
    });
}

struct ReturnRaster {
    metadata: DocumentMetadata,
    fail: std::sync::Arc<std::sync::atomic::AtomicBool>,
}
impl ferrostage_pdf::raster::RasterBackend for ReturnRaster {
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
        _: ferrostage_core::NormalizedRect,
        target: ferrostage_pdf::PixelSize,
    ) -> Result<ferrostage_core::BgraFrame, ferrostage_pdf::PdfError> {
        if page == PhysicalPage(2) && self.fail.load(std::sync::atomic::Ordering::SeqCst) {
            return Err(ferrostage_pdf::PdfError::Unreadable);
        }
        Ok(ferrostage_core::BgraFrame::new(
            target.width,
            target.height,
            target.width as usize * 4,
            vec![page.0 as u8 * 30; (target.width * target.height * 4) as usize],
        )
        .unwrap())
    }
    fn remove(&mut self, _: DocumentGeneration) {}
}
fn return_poll_until(
    shell: &Entity<Shell>,
    presenter: AnyWindowHandle,
    cx: &mut TestAppContext,
    condition: impl Fn(&Shell) -> bool,
) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while !shell.read_with(cx, |s, _| condition(s)) {
        shell.update(cx, |s, cx| {
            s.update_render();
            s.publish(false, cx);
            cx.notify();
        });
        redraw(presenter, cx);
        std::thread::sleep(Duration::from_millis(1));
        assert!(Instant::now() < deadline, "return raster fixture stalled");
    }
}
#[gpui::test]
fn return_explicit_same_page_button_retries_failed_raster_request(cx: &mut TestAppContext) {
    use crate::workbench::Surface;
    let (shell, presenter) = return_fixture(cx);
    let fail = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
    shell.update(cx, |s, cx| {
        s.apply(Command::JumpTo(PhysicalPage(3)), cx);
        let metadata = s.workbench.document.as_ref().unwrap().metadata.clone();
        let flag = fail.clone();
        let worker = RenderHandle::with_backend(
            move || {
                Box::new(ReturnRaster {
                    metadata,
                    fail: flag,
                })
            },
            Default::default(),
        )
        .unwrap();
        worker
            .load_active(s.state.generation(), vec![1])
            .unwrap()
            .recv()
            .unwrap()
            .unwrap();
        s.renderer = Ok(worker);
        s.dispatch(Intent::ReturnPosition, cx);
    });
    return_poll_until(&shell, presenter, cx, |s| {
        s.workbench.failed_surfaces().contains(&Surface::Current)
    });
    fail.store(false, std::sync::atomic::Ordering::SeqCst);
    // Recovery alone must not spin-retry an identical failed raster.
    for _ in 0..3 {
        shell.update(cx, |s, _| s.update_render());
        std::thread::sleep(Duration::from_millis(1));
    }
    shell.read_with(cx, |s, _| {
        assert!(s.return_target().is_some());
        assert!(s.workbench.failed_surfaces().contains(&Surface::Current));
    });
    redraw(presenter, cx);
    click_control(presenter, "return-position", cx);
    return_poll_until(&shell, presenter, cx, |s| s.return_target().is_none());
    shell.read_with(cx, |s, _| {
        assert_eq!(s.state.current_page(), PhysicalPage(2))
    });
}
