#[gpui::test]
fn presenter_start_waits_for_native_fullscreen_and_timeout_restores(cx: &mut TestAppContext) {
    let (shell, handle) = input_presenter(cx);
    let original = handle.update(cx, |_, window, _| window.bounds()).unwrap();
    shell.update(cx, |s, cx| {
        load_ready(s);
        s.enter_presenter_fullscreen(cx).unwrap();
        assert!(s.presenter.as_ref().unwrap().fullscreen);
        // TestPlatform toggles the flag but does not produce native fullscreen
        // geometry. It must not count as a successfully started presentation.
        let audience = test_window(s, true, cx);
        s.pending = Some(Transaction {
            kind: TransactionKind::Start,
            presenter: None,
            audience: Some(audience),
            deadline: Instant::now() + CONFIRM_TIMEOUT,
        });
        s.poll_transaction(cx);
        assert_eq!(s.state.stage(), SessionStage::Ready);
        assert!(s.pending.is_some());
        assert_eq!(s.state.elapsed_at(Instant::now()), Duration::ZERO);
        s.pending.as_mut().unwrap().deadline = Instant::now() - Duration::from_millis(1);
        s.poll_transaction(cx);
        assert!(s.pending.is_none());
        assert!(s.audience.is_none());
        assert!(s.presenter_restore.is_none());
        assert_eq!(s.presenter.as_ref().unwrap().handle, handle);
    });
    handle
        .update(cx, |_, window, _| {
            assert!(!window.is_fullscreen());
            assert_eq!(window.bounds(), original);
        })
        .unwrap();
}

#[gpui::test]
fn presenter_stop_waits_for_queued_fullscreen_entry_before_restoring(cx: &mut TestAppContext) {
    let (shell, handle) = input_presenter(cx);
    shell.update(cx, |s, cx| s.enter_presenter_fullscreen(cx).unwrap());
    // Model the native Windows queue: its entry request has not executed yet.
    handle
        .update(cx, |_, window, _| window.toggle_fullscreen())
        .unwrap();
    shell.update(cx, |s, cx| {
        s.stop(cx);
        assert!(s.restoring_presenter());
        assert!(s.busy());
        assert!(s.start_reason().is_some());
    });
    // Delivery of the queued entry must be followed by one exit, not mistaken
    // for a new presentation or left fullscreen after the cancelled Start.
    handle
        .update(cx, |_, window, _| window.toggle_fullscreen())
        .unwrap();
    shell.update(cx, |s, cx| {
        s.poll_presenter_restore(cx);
        assert!(s.presenter_restore.is_none());
    });
    assert!(
        !handle
            .update(cx, |_, window, _| window.is_fullscreen())
            .unwrap()
    );
}

#[gpui::test]
fn presenter_fullscreen_repeated_restore_and_already_fullscreen_are_idempotent(
    cx: &mut TestAppContext,
) {
    let (shell, handle) = input_presenter(cx);
    for _ in 0..2 {
        shell.update(cx, |s, cx| {
            s.enter_presenter_fullscreen(cx).unwrap();
            s.stop(cx);
            s.stop(cx);
            assert!(s.presenter_restore.is_none());
        });
        assert!(
            !handle
                .update(cx, |_, window, _| window.is_fullscreen())
                .unwrap()
        );
    }
    handle
        .update(cx, |_, window, _| window.toggle_fullscreen())
        .unwrap();
    shell.update(cx, |s, cx| {
        s.enter_presenter_fullscreen(cx).unwrap();
        s.stop(cx);
        assert!(s.presenter_restore.is_none());
    });
    assert!(
        handle
            .update(cx, |_, window, _| window.is_fullscreen())
            .unwrap()
    );
}

#[gpui::test]
fn presenter_split_drag_tracks_lower_notes_height_and_persists(cx: &mut TestAppContext) {
    let fixture = SaveFixture::new();
    fixture.unblock();
    let (shell, handle) = workbench_fixture(cx);
    shell.update(cx, |s, _| s.store = fixture.store());
    for desired in [0.58, 0.76] {
        redraw(handle, cx);
        let mut visual = gpui::VisualTestContext::from_window(handle, cx);
        let right = visual.debug_bounds("next-notes-pane").unwrap();
        let divider = visual.debug_bounds("vertical-split").unwrap();
        let target = gpui::point(
            divider.center().x,
            right.top() + px(7.) + (right.size.height - px(14.)) * (1. - desired),
        );
        visual.simulate_mouse_down(
            divider.center(),
            gpui::MouseButton::Left,
            gpui::Modifiers::default(),
        );
        visual.simulate_mouse_move(
            target,
            Some(gpui::MouseButton::Left),
            gpui::Modifiers::default(),
        );
        visual.simulate_mouse_up(target, gpui::MouseButton::Left, gpui::Modifiers::default());
        redraw(handle, cx);
        let mut visual = gpui::VisualTestContext::from_window(handle, cx);
        let notes = visual.debug_bounds("notes-panel").unwrap();
        let next = visual.debug_bounds("next-pane").unwrap();
        assert!(next.bottom() < notes.top());
        assert!(
            (notes.size.height / (notes.size.height + next.size.height) - desired).abs() < 0.01
        );
        let saved = std::fs::read_to_string(fixture.store().path()).unwrap();
        let parsed = ferrostage_settings::from_toml(&saved).unwrap();
        assert!((parsed.settings.presenter.notes_ratio - f64::from(desired)).abs() < 0.001);
    }
}

#[test]
fn presenter_panes_have_no_black_letterbox() {
    use crate::workbench::Surface;
    for surface in [
        Surface::Current,
        Surface::Next,
        Surface::Notes(0),
        Surface::Notes(1),
    ] {
        assert_eq!(crate::presenter::letterbox(surface), None);
    }
    assert_eq!(
        crate::presenter::letterbox(Surface::Thumbnail(0)),
        Some(0x000000)
    );
}

#[test]
fn timer_buttons_share_the_quiet_header_face() {
    use ferrostage_core::PresentationCommand as Command;
    for command in [
        Command::PauseTimer,
        Command::ResumeTimer,
        Command::ResetTimer,
    ] {
        assert!(crate::presenter::quiet_header(
            SessionStage::Presenting,
            &Intent::Core(command.clone())
        ));
        assert!(!crate::presenter::quiet_header(
            SessionStage::Idle,
            &Intent::Core(command.clone())
        ));
    }
    assert!(crate::presenter::quiet_header(
        SessionStage::Presenting,
        &Intent::Displays
    ));
    assert!(!crate::presenter::quiet_header(
        SessionStage::Presenting,
        &Intent::Next
    ));
}

#[gpui::test]
fn original_notes_keep_width_and_natural_height_for_multiple_pages(cx: &mut TestAppContext) {
    use crate::workbench::{ReadyFrame, Surface};
    let (shell, handle) = workbench_fixture(cx);
    shell.update(cx, |s, cx| {
        for index in 0..2 {
            s.workbench.frames.push((
                Surface::Notes(index),
                ReadyFrame {
                    request: crate::workbench::request_for(
                        s.state.generation(),
                        PhysicalPage(1),
                        ferrostage_core::NormalizedRect::FULL,
                    ),
                    frame: ferrostage_core::BgraFrame::new(4, 3, 16, vec![255; 48]).unwrap(),
                },
            ));
        }
        cx.notify();
    });
    redraw(handle, cx);
    let mut visual = gpui::VisualTestContext::from_window(handle, cx);
    let notes = visual.debug_bounds("notes-panel").unwrap();
    let first = visual.debug_bounds("surface-Notes(0)").unwrap();
    let second = visual.debug_bounds("surface-Notes(1)").unwrap();
    assert!((first.size.width - (notes.size.width - px(24.))).abs() < px(1.));
    assert!((first.size.width / first.size.height - 4. / 3.).abs() < 0.01);
    assert!((first.size.height - second.size.height).abs() < px(1.));
    assert!(
        second.bottom() > notes.bottom(),
        "multiple notes scroll instead of shrinking"
    );
}

#[gpui::test]
fn presentation_menus_preserve_keyboard_focus_and_escape_precedence(cx: &mut TestAppContext) {
    let (shell, handle) = workbench_fixture(cx);
    shell.update(cx, |s, cx| {
        s.apply(
            Command::StartPresentation {
                audience_available: true,
            },
            cx,
        );
        cx.notify();
    });
    redraw(handle, cx);
    click_control(handle, "more", cx);
    redraw(handle, cx);
    cx.simulate_keystrokes(handle, "escape");
    redraw(handle, cx);
    assert_eq!(
        shell.read_with(cx, |s, _| s.state.stage()),
        SessionStage::Presenting
    );
    click_control(handle, "drawing-tools", cx);
    redraw(handle, cx);
    cx.simulate_keystrokes(handle, "enter");
    shell.read_with(cx, |s, _| {
        assert_eq!(s.workbench.tool, crate::workbench::Tool::Pen);
        assert!(s.contextual.is_none());
        assert_eq!(s.state.current_page(), PhysicalPage(0));
    });
    redraw(handle, cx);
    // Focus returns to Drawing, so Space reopens it instead of advancing.
    cx.simulate_keystrokes(handle, "space");
    assert!(shell.read_with(cx, |s, _| matches!(
        s.contextual,
        Some(crate::page_browser::ContextualSurface::PresenterMenu { tools: true })
    )));
}
