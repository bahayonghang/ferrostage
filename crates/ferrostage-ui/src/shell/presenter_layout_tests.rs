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
            kind: TransactionKind::Start(StartTimer::Restart),
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
                    // Integer raster dimensions round the original 4:3 page.
                    // They must not feed back into the document layout.
                    frame: ferrostage_core::BgraFrame::new(7, 5, 28, vec![255; 140]).unwrap(),
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
    // The reopened menu marks the active tool without a text glyph.
    redraw(handle, cx);
    let mut visual = gpui::VisualTestContext::from_window(handle, cx);
    let row = visual.debug_bounds("pen").unwrap();
    assert!(row.contains(&visual.debug_bounds("pen-selected").unwrap().center()));
    assert!(visual.debug_bounds("eraser-selected").is_none());
}

#[gpui::test]
fn notes_first_layout_puts_notes_left_and_larger_current_above_next(cx: &mut TestAppContext) {
    let (shell, handle) = workbench_fixture(cx);
    cx.test_window(handle)
        .simulate_resize(size(px(1700.), px(1000.)));
    shell.update(cx, |s, cx| {
        s.settings.settings.display.start_mode = ferrostage_settings::StartMode::DualNotes;
        cx.notify();
    });
    redraw(handle, cx);
    {
        let mut visual = gpui::VisualTestContext::from_window(handle, cx);
        let splits = visual.debug_bounds("workbench-splits").unwrap();
        let notes = visual.debug_bounds("notes-panel").unwrap();
        let current = visual.debug_bounds("current-pane").unwrap();
        let next = visual.debug_bounds("next-pane").unwrap();
        assert!(notes.right() <= current.left());
        assert!((notes.size.width / splits.size.width - 0.55).abs() < 0.02);
        assert!((notes.size.height - splits.size.height).abs() < px(1.));
        assert!(current.bottom() <= next.top());
        assert!(current.size.height > next.size.height);
        assert!(
            (current.size.height / (current.size.height + next.size.height) - 0.62).abs() < 0.01
        );
        assert!(visual.debug_bounds("column-split").is_some());
        assert!(visual.debug_bounds("row-split").is_some());
        assert!(visual.debug_bounds("horizontal-split").is_none());
    }
    // Single keeps the standard workbench in Ready.
    shell.update(cx, |s, cx| {
        s.settings.settings.display.start_mode = ferrostage_settings::StartMode::Single;
        cx.notify();
    });
    redraw(handle, cx);
    {
        let mut visual = gpui::VisualTestContext::from_window(handle, cx);
        let notes = visual.debug_bounds("notes-panel").unwrap();
        let current = visual.debug_bounds("current-pane").unwrap();
        assert!(current.right() <= notes.left());
        assert!(visual.debug_bounds("horizontal-split").is_some());
        assert!(visual.debug_bounds("column-split").is_none());
    }
    // Narrow windows stack in the standard order in every mode.
    shell.update(cx, |s, cx| {
        s.settings.settings.display.start_mode = ferrostage_settings::StartMode::DualNotes;
        cx.notify();
    });
    cx.test_window(handle)
        .simulate_resize(size(px(700.), px(900.)));
    redraw(handle, cx);
    let mut visual = gpui::VisualTestContext::from_window(handle, cx);
    let current = visual.debug_bounds("current-pane").unwrap();
    let next = visual.debug_bounds("next-pane").unwrap();
    let notes = visual.debug_bounds("notes-panel").unwrap();
    assert!(current.bottom() <= next.top() && next.bottom() <= notes.top());
    assert!(visual.debug_bounds("column-split").is_none());
}

#[gpui::test]
fn notes_first_dividers_drag_live_and_persist_only_their_ratios(cx: &mut TestAppContext) {
    let fixture = SaveFixture::new();
    fixture.unblock();
    let (shell, handle) = workbench_fixture(cx);
    cx.test_window(handle)
        .simulate_resize(size(px(1700.), px(1000.)));
    let standard = shell.update(cx, |s, cx| {
        s.store = fixture.store();
        s.settings.settings.display.start_mode = ferrostage_settings::StartMode::DualNotes;
        cx.notify();
        s.settings.settings.presenter.clone()
    });
    redraw(handle, cx);
    let drag = |from: gpui::Point<Pixels>, to: gpui::Point<Pixels>, cx: &mut TestAppContext| {
        let mut visual = gpui::VisualTestContext::from_window(handle, cx);
        visual.simulate_mouse_down(from, gpui::MouseButton::Left, gpui::Modifiers::default());
        visual.simulate_mouse_move(
            to,
            Some(gpui::MouseButton::Left),
            gpui::Modifiers::default(),
        );
        visual.simulate_mouse_up(to, gpui::MouseButton::Left, gpui::Modifiers::default());
    };
    let (splits, column) = {
        let mut visual = gpui::VisualTestContext::from_window(handle, cx);
        (
            visual.debug_bounds("workbench-splits").unwrap(),
            visual.debug_bounds("column-split").unwrap(),
        )
    };
    let target = gpui::point(
        splits.left() + px(4.) + splits.size.width * 0.62,
        column.center().y,
    );
    drag(column.center(), target, cx);
    redraw(handle, cx);
    let (previews, row) = {
        let mut visual = gpui::VisualTestContext::from_window(handle, cx);
        let notes = visual.debug_bounds("notes-panel").unwrap();
        assert!((notes.size.width / splits.size.width - 0.62).abs() < 0.01);
        (
            visual.debug_bounds("previews-column").unwrap(),
            visual.debug_bounds("row-split").unwrap(),
        )
    };
    let target = gpui::point(
        row.center().x,
        previews.top() + px(4.) + (previews.size.height - px(8.)) * 0.7,
    );
    drag(row.center(), target, cx);
    redraw(handle, cx);
    {
        let mut visual = gpui::VisualTestContext::from_window(handle, cx);
        let current = visual.debug_bounds("current-pane").unwrap();
        let next = visual.debug_bounds("next-pane").unwrap();
        assert!(
            (current.size.height / (current.size.height + next.size.height) - 0.7).abs() < 0.01
        );
    }
    shell.read_with(cx, |s, _| assert!(!s.settings.dirty));
    let saved = std::fs::read_to_string(fixture.store().path()).unwrap();
    let parsed = ferrostage_settings::from_toml(&saved)
        .unwrap()
        .settings
        .presenter;
    assert!((parsed.notes_first_notes_ratio - 0.62).abs() < 0.002);
    assert!((parsed.notes_first_current_ratio - 0.7).abs() < 0.002);
    assert_eq!(parsed.next_slide_ratio, standard.next_slide_ratio);
    assert_eq!(parsed.notes_ratio, standard.notes_ratio);
}

#[gpui::test]
fn start_mode_menu_selects_notes_first_and_switches_the_ready_layout(cx: &mut TestAppContext) {
    let fixture = SaveFixture::new();
    fixture.unblock();
    let (shell, handle) = workbench_fixture(cx);
    cx.test_window(handle)
        .simulate_resize(size(px(1700.), px(1000.)));
    shell.update(cx, |s, cx| {
        s.store = fixture.store();
        cx.notify();
    });
    redraw(handle, cx);
    {
        let mut visual = gpui::VisualTestContext::from_window(handle, cx);
        let start = visual.debug_bounds("start").unwrap();
        let trigger = visual.debug_bounds("start-mode").unwrap();
        assert!((start.size.height - trigger.size.height).abs() < px(1.));
        // Drawn chevrons, not font glyphs, mark both menu triggers.
        assert!(visual.debug_bounds("start-mode-chevron-closed").is_some());
        assert!(visual.debug_bounds("document-options-chevron-closed").is_some());
    }
    click_control(handle, "start-mode", cx);
    redraw(handle, cx);
    {
        let mut visual = gpui::VisualTestContext::from_window(handle, cx);
        let menu = visual.debug_bounds("start-mode-menu").unwrap();
        assert!((menu.size.width - px(360.)).abs() < px(1.));
        assert!(visual.debug_bounds("start-mode-chevron-open").is_some());
        let rows = ["start-mode-row-0", "start-mode-row-1", "start-mode-row-2"]
            .map(|row| visual.debug_bounds(row).unwrap());
        assert!(rows[0].bottom() <= rows[1].top() && rows[1].bottom() <= rows[2].top());
        assert!(visual.debug_bounds("start-mode-check-0").is_some());
    }
    click_control(handle, "start-mode-row-1", cx);
    cx.run_until_parked();
    shell.read_with(cx, |s, _| {
        assert!(s.contextual.is_none());
        assert_eq!(
            s.settings.settings.display.start_mode,
            ferrostage_settings::StartMode::DualNotes
        );
        assert_eq!(
            s.store.load().unwrap().settings.display.start_mode,
            ferrostage_settings::StartMode::DualNotes
        );
    });
    redraw(handle, cx);
    let mut visual = gpui::VisualTestContext::from_window(handle, cx);
    let notes = visual.debug_bounds("notes-panel").unwrap();
    let current = visual.debug_bounds("current-pane").unwrap();
    assert!(notes.right() <= current.left());
}

#[test]
fn airy_faces_keep_semantic_badge_and_pill_controls() {
    use crate::theme::{ButtonKind, DARK, LIGHT, PILL_RADIUS};
    for p in [LIGHT, DARK] {
        assert_eq!(crate::presenter::stage_dot(SessionStage::Ready, p), p.ink);
        assert_eq!(crate::presenter::stage_dot(SessionStage::Presenting, p), p.focus);
        assert_eq!(crate::presenter::stage_dot(SessionStage::Idle, p), p.muted);
        for enabled in [false, true] {
            assert_eq!(p.button(ButtonKind::Destructive, enabled, false).radius, PILL_RADIUS);
        }
    }
}

#[gpui::test]
fn start_mode_menu_keyboard_reaches_every_row_without_navigating(cx: &mut TestAppContext) {
    let fixture = SaveFixture::new();
    fixture.unblock();
    let (shell, handle) = workbench_fixture(cx);
    cx.test_window(handle)
        .simulate_resize(size(px(1700.), px(1000.)));
    shell.update(cx, |s, cx| {
        s.store = fixture.store();
        cx.notify();
    });
    redraw(handle, cx);
    let page = shell.read_with(cx, |s, _| s.state.current_page());
    click_control(handle, "start-mode", cx);
    redraw(handle, cx);
    // Focus starts on the checked Dual row; arrows stay inside the menu.
    cx.simulate_keystrokes(handle, "down down");
    redraw(handle, cx);
    shell.read_with(cx, |s, _| {
        assert_eq!(s.state.current_page(), page);
        assert!(matches!(
            s.contextual,
            Some(crate::page_browser::ContextualSurface::StartMode)
        ));
    });
    cx.simulate_keystrokes(handle, "enter");
    cx.run_until_parked();
    shell.read_with(cx, |s, _| {
        assert!(s.contextual.is_none());
        assert_eq!(
            s.settings.settings.display.start_mode,
            ferrostage_settings::StartMode::Single
        );
    });
    redraw(handle, cx);
    click_control(handle, "start-mode", cx);
    redraw(handle, cx);
    cx.simulate_keystrokes(handle, "up enter");
    cx.run_until_parked();
    shell.read_with(cx, |s, _| {
        assert_eq!(s.state.current_page(), page);
        assert_eq!(
            s.store.load().unwrap().settings.display.start_mode,
            ferrostage_settings::StartMode::DualNotes
        );
    });
    redraw(handle, cx);
    let mut visual = gpui::VisualTestContext::from_window(handle, cx);
    let notes = visual.debug_bounds("notes-panel").unwrap();
    let current = visual.debug_bounds("current-pane").unwrap();
    assert!(notes.right() <= current.left());
}


fn ordinary_session(shell: &Shell) -> crate::workbench::DocumentSession {
    let session = shell.workbench.document.as_ref().unwrap();
    let mut metadata = session.metadata.clone();
    for page in &mut metadata.pages {
        page.label = None;
    }
    crate::workbench::DocumentSession::build(metadata, session.structure.clone()).unwrap()
}

#[gpui::test]
fn notes_free_layout_uses_document_mapping_and_keeps_next_at_the_end(cx: &mut TestAppContext) {
    use crate::workbench::WorkbenchLayout;
    let (shell, handle) = workbench_fixture(cx);
    shell.update(cx, |s, cx| {
        let session = ordinary_session(s);
        s.state
            .apply_at(
                Command::LoadMappedDocument(
                    session
                        .document
                        .descriptor(ferrostage_core::DocumentGeneration(3))
                        .unwrap(),
                ),
                Instant::now(),
            )
            .unwrap();
        s.workbench.document = Some(session);
        assert_eq!(s.workbench_layout(), WorkbenchLayout::Previews);
        cx.notify();
    });
    for mode in [
        ferrostage_settings::StartMode::Dual,
        ferrostage_settings::StartMode::DualNotes,
        ferrostage_settings::StartMode::Single,
    ] {
        shell.update(cx, |s, cx| {
            s.settings.settings.display.start_mode = mode;
            cx.notify();
        });
        for width in [1700., 800., 799.] {
            cx.test_window(handle)
                .simulate_resize(size(px(width), px(1000.)));
            redraw(handle, cx);
            let mut visual = gpui::VisualTestContext::from_window(handle, cx);
            assert!(visual.debug_bounds("notes-panel").is_none());
            let current = visual.debug_bounds("current-pane").unwrap();
            let next = visual.debug_bounds("next-pane").unwrap();
            if width >= 800. {
                assert!(current.right() <= next.left());
                assert!(next.size.width >= px(240.));
                if width == 1700. {
                    assert!(
                        (current.size.width / (current.size.width + next.size.width) - 0.7).abs()
                            < 0.01
                    );
                }
            } else {
                assert!(current.bottom() <= next.top());
            }
        }
    }
    shell.update(cx, |s, cx| {
        s.apply(Command::JumpTo(PhysicalPage(3)), cx);
    });
    redraw(handle, cx);
    let mut visual = gpui::VisualTestContext::from_window(handle, cx);
    assert!(visual.debug_bounds("next-pane").is_some());
    assert!(visual.debug_bounds("next-end").is_some());
}

#[gpui::test]
fn notes_free_drag_reset_and_reopen_preserve_saved_note_ratios_and_presentation(
    cx: &mut TestAppContext,
) {
    let fixture = SaveFixture::new();
    fixture.unblock();
    let (shell, handle) = workbench_fixture(cx);
    cx.test_window(handle)
        .simulate_resize(size(px(1700.), px(1000.)));
    let (notes_session, saved_settings, snapshot) = shell.update(cx, |s, cx| {
        s.store = fixture.store();
        let notes = s.workbench.document.clone().unwrap();
        let session = ordinary_session(s);
        s.finish_import(
            PathBuf::from("ordinary.pdf"),
            ferrostage_core::DocumentGeneration(3),
            session,
            cx,
        );
        (
            notes,
            s.settings.settings.presenter.clone(),
            s.state.audience_snapshot(),
        )
    });
    redraw(handle, cx);
    let mut visual = gpui::VisualTestContext::from_window(handle, cx);
    let divider = visual.debug_bounds("horizontal-split").unwrap();
    let bounds = visual.debug_bounds("workbench-splits").unwrap();
    let target = gpui::point(bounds.left() + bounds.size.width * 0.55, divider.center().y);
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
    shell.read_with(cx, |s, _| {
        assert!((s.workbench.previews_ratio.unwrap() - 0.45).abs() < 0.01);
        assert_eq!(s.settings.settings.presenter, saved_settings);
        assert_eq!(s.state.audience_snapshot(), snapshot);
        assert!(!s.settings.dirty);
    });
    click_control(handle, "document-options", cx);
    redraw(handle, cx);
    cx.simulate_keystrokes(handle, "end down up space");
    redraw(handle, cx);
    shell.read_with(cx, |s, _| {
        assert!(s.contextual.is_none());
        assert_eq!(s.workbench.previews_ratio, Some(0.3));
        assert_eq!(s.settings.settings.presenter, saved_settings);
        assert_eq!(s.state.audience_snapshot(), snapshot);
    });
    assert!(layout_focus(handle, cx).1);
    let parsed = fixture.store().load().unwrap();
    assert_eq!(parsed.settings.presenter, saved_settings);
    shell.update(cx, |s, cx| {
        s.workbench.previews_ratio = Some(0.5);
        s.workbench.document = Some(notes_session);
        assert_eq!(
            s.workbench_layout(),
            crate::workbench::WorkbenchLayout::Standard
        );
        assert_eq!(s.settings.settings.presenter, saved_settings);
        let session = ordinary_session(s);
        s.finish_import(
            PathBuf::from("another.pdf"),
            ferrostage_core::DocumentGeneration(4),
            session,
            cx,
        );
        assert_eq!(s.workbench.previews_ratio, None);
    });
}

#[gpui::test]
fn mixed_notes_keep_geometry_between_absent_and_loading_pages(cx: &mut TestAppContext) {
    let (shell, handle) = workbench_fixture(cx);
    cx.test_window(handle)
        .simulate_resize(size(px(1200.), px(900.)));
    shell.update(cx, |s, cx| {
        s.apply(Command::JumpTo(PhysicalPage(2)), cx);
    });
    redraw(handle, cx);
    let before = {
        let mut visual = gpui::VisualTestContext::from_window(handle, cx);
        assert!(visual.debug_bounds("notes-empty").is_some());
        visual.debug_bounds("notes-panel").unwrap()
    };
    shell.update(cx, |s, cx| {
        s.apply(Command::JumpTo(PhysicalPage(0)), cx);
    });
    redraw(handle, cx);
    let mut visual = gpui::VisualTestContext::from_window(handle, cx);
    assert!(visual.debug_bounds("notes-loading").is_some());
    assert_eq!(visual.debug_bounds("notes-panel"), Some(before));
}

#[gpui::test]
fn unconfirmed_same_page_notes_keep_the_notes_workbench(cx: &mut TestAppContext) {
    let (shell, handle) = crop_gate_fixture(cx, false);
    shell.read_with(cx, |s, _| {
        assert!(s.crop_needs_confirmation());
        assert_eq!(
            s.workbench_layout(),
            crate::workbench::WorkbenchLayout::Standard
        );
    });
    redraw(handle, cx);
    let mut visual = gpui::VisualTestContext::from_window(handle, cx);
    assert!(visual.debug_bounds("notes-panel").is_some());
    shell.update(cx, |s, cx| {
        s.crop_confirmed = true;
        cx.notify();
    });
    redraw(handle, cx);
    assert_eq!(
        shell.read_with(cx, |s, _| s.workbench_layout()),
        crate::workbench::WorkbenchLayout::Previews
    );
}

#[gpui::test]
fn read_only_layout_menu_skips_disabled_reset_and_retains_session_reset(cx: &mut TestAppContext) {
    let (shell, handle) = workbench_fixture(cx);
    let (settings, snapshot) = shell.update(cx, |s, cx| {
        s.settings.write_policy = WritePolicy::ReadOnly {
            reason: "future schema".into(),
        };
        s.apply(
            Command::StartPresentation {
                audience_available: true,
            },
            cx,
        );
        (
            s.settings.settings.presenter.clone(),
            s.state.audience_snapshot(),
        )
    });
    redraw(handle, cx);
    click_control(handle, "document-options", cx);
    redraw(handle, cx);
    for key in ["down", "up", "tab", "shift-tab", "home", "end"] {
        cx.simulate_keystrokes(handle, key);
        redraw(handle, cx);
        assert_eq!(layout_focus(handle, cx).0, Some(3));
    }
    cx.simulate_keystrokes(handle, "enter");
    redraw(handle, cx);
    shell.read_with(cx, |s, _| {
        assert_eq!(s.modal, Some(Modal::Crop));
        assert_eq!(s.settings.settings.presenter, settings);
        assert_eq!(s.state.audience_snapshot(), snapshot);
    });
    cx.simulate_keystrokes(handle, "escape");
    redraw(handle, cx);
    shell.update(cx, |s, cx| {
        s.stop(cx);
        let session = ordinary_session(s);
        s.finish_import(
            PathBuf::from("readonly-ordinary.pdf"),
            ferrostage_core::DocumentGeneration(4),
            session,
            cx,
        );
        s.workbench.previews_ratio = Some(0.5);
        cx.notify();
    });
    redraw(handle, cx);
    click_control(handle, "document-options", cx);
    redraw(handle, cx);
    cx.simulate_keystrokes(handle, "end");
    redraw(handle, cx);
    assert_eq!(layout_focus(handle, cx).0, Some(4));
    cx.simulate_keystrokes(handle, "space");
    redraw(handle, cx);
    shell.read_with(cx, |s, _| {
        assert!(s.contextual.is_none());
        assert_eq!(s.workbench.previews_ratio, Some(0.3));
        assert_eq!(s.settings.settings.presenter, settings);
        assert!(!s.settings.dirty);
    });
}

#[gpui::test]
fn narrow_notes_first_reset_restores_the_visible_standard_proportions(
    cx: &mut TestAppContext,
) {
    let fixture = SaveFixture::new();
    fixture.unblock();
    let (shell, handle) = workbench_fixture(cx);
    cx.test_window(handle).simulate_resize(size(px(799.), px(1000.)));
    let (notes_width, current_share) = shell.update(cx, |s, cx| {
        s.store = fixture.store();
        s.settings.settings.display.start_mode = ferrostage_settings::StartMode::DualNotes;
        s.settings.settings.presenter.next_slide_ratio = 0.2;
        s.settings.settings.presenter.notes_ratio = 0.75;
        s.settings.settings.presenter.notes_first_notes_ratio = 0.65;
        s.settings.settings.presenter.notes_first_current_ratio = 0.75;
        cx.notify();
        (s.settings.settings.presenter.notes_first_notes_ratio,
         s.settings.settings.presenter.notes_first_current_ratio)
    });
    redraw(handle, cx);
    click_control(handle, "document-options", cx);
    redraw(handle, cx);
    cx.simulate_keystrokes(handle, "end enter");
    redraw(handle, cx);
    shell.read_with(cx, |s, _| {
        let settings = &s.settings.settings.presenter;
        let defaults = ferrostage_settings::PresenterSettings::default();
        assert_eq!(settings.next_slide_ratio, defaults.next_slide_ratio);
        assert_eq!(settings.notes_ratio, defaults.notes_ratio);
        assert_eq!(settings.notes_first_notes_ratio, notes_width);
        assert_eq!(settings.notes_first_current_ratio, current_share);
        assert!(s.contextual.is_none());
    });
}

#[gpui::test]
fn segmented_tools_mark_selection_in_a_fixed_slot_without_width_change(
    cx: &mut TestAppContext,
) {
    const TOOLS: [(&str, &str, crate::workbench::Tool); 4] = [
        ("laser", "laser-selected", crate::workbench::Tool::Laser),
        ("pen", "pen-selected", crate::workbench::Tool::Pen),
        ("highlighter", "highlighter-selected", crate::workbench::Tool::Highlighter),
        ("eraser", "eraser-selected", crate::workbench::Tool::Eraser),
    ];
    let (shell, handle) = workbench_fixture(cx);
    redraw(handle, cx);
    let mut visual = gpui::VisualTestContext::from_window(handle, cx);
    let group = visual.debug_bounds("tool-group").unwrap();
    let before = TOOLS.map(|(id, _, _)| visual.debug_bounds(id).unwrap());
    for (_, mark, _) in TOOLS {
        assert!(visual.debug_bounds(mark).is_none(), "{mark} at rest");
    }
    for (control, mark, tool) in TOOLS {
        click_control(handle, control, cx);
        redraw(handle, cx);
        let mut visual = gpui::VisualTestContext::from_window(handle, cx);
        let target = visual.debug_bounds(control).unwrap();
        let dot = visual
            .debug_bounds(mark)
            .expect("selected tool shows its indicator");
        assert!(target.contains(&dot.center()), "{control}: mark inside its button");
        for ((other, other_mark, _), bounds) in TOOLS.iter().zip(&before) {
            // Selection never inserts a glyph, so every option keeps its width.
            assert_eq!(visual.debug_bounds(other).unwrap().size, bounds.size, "{other}");
            if *other != control {
                assert!(visual.debug_bounds(other_mark).is_none(), "{other_mark}");
            }
        }
        assert_eq!(visual.debug_bounds("tool-group").unwrap().size, group.size);
        shell.read_with(cx, |s, _| assert_eq!(s.workbench.tool, tool));
    }
}

#[gpui::test]
fn blank_options_ring_their_swatch_without_width_change(cx: &mut TestAppContext) {
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
    let mut visual = gpui::VisualTestContext::from_window(handle, cx);
    let black = visual.debug_bounds("black").unwrap();
    let white = visual.debug_bounds("white").unwrap();
    assert!(visual.debug_bounds("black-selected").is_none());
    click_control(handle, "black", cx);
    redraw(handle, cx);
    let mut visual = gpui::VisualTestContext::from_window(handle, cx);
    assert!(black.contains(&visual.debug_bounds("black-selected").unwrap().center()));
    assert!(visual.debug_bounds("white-selected").is_none());
    assert_eq!(visual.debug_bounds("black").unwrap().size, black.size);
    assert_eq!(visual.debug_bounds("white").unwrap().size, white.size);
    shell.read_with(cx, |s, _| {
        assert_eq!(
            s.state.audience_snapshot().blank_mode,
            Some(ferrostage_core::BlankMode::Black)
        );
    });
}

#[gpui::test]
fn document_warnings_float_above_the_crop_notice_without_overlap(cx: &mut TestAppContext) {
    let (shell, handle) = crop_gate_fixture(cx, false);
    shell.update(cx, |s, cx| {
        let document = &mut s.workbench.document.as_mut().unwrap().document;
        document
            .warnings
            .push(ferrostage_document::DocumentWarning::MixedGeometry);
        document
            .warnings
            .push(ferrostage_document::DocumentWarning::AmbiguousLayout);
        cx.notify();
    });
    // Wide, ordinary and narrow windows; the narrow one wraps the notice.
    for (width, height) in [(1440., 900.), (1100., 760.), (520., 760.)] {
        cx.test_window(handle)
            .simulate_resize(size(px(width), px(height)));
        redraw(handle, cx);
        let mut visual = gpui::VisualTestContext::from_window(handle, cx);
        let hint = visual.debug_bounds("crop-hint").expect("crop notice shown");
        let stack = visual
            .debug_bounds("document-warning-stack")
            .expect("warnings shown");
        assert!(!stack.intersects(&hint), "{width}: {stack:?} overlaps {hint:?}");
        assert!(
            f32::from(hint.top() - stack.bottom()) >= 7.5,
            "{width}: 8px gap {stack:?} {hint:?}"
        );
        assert!((f32::from(stack.left() - hint.left())).abs() <= 1., "{width}");
        for flag in ["document-warning-0", "document-warning-1"] {
            let flag = visual.debug_bounds(flag).unwrap();
            assert!(flag.bottom() <= stack.bottom() && !flag.intersects(&hint));
        }
    }
    // Without the notice, the stack keeps its bottom-left position in the content.
    shell.update(cx, |s, cx| {
        s.crop_confirmed = true;
        cx.notify();
    });
    redraw(handle, cx);
    let mut visual = gpui::VisualTestContext::from_window(handle, cx);
    assert!(visual.debug_bounds("crop-hint").is_none());
    let content = visual.debug_bounds("presenter-content").unwrap();
    let stack = visual.debug_bounds("document-warning-stack").unwrap();
    assert!((f32::from(stack.left() - content.left()) - 12.).abs() <= 1.);
    assert!((f32::from(content.bottom() - stack.bottom()) - 12.).abs() <= 1.);
}
