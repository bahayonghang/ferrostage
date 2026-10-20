fn outline_fixture(cx: &mut TestAppContext) -> (Entity<Shell>, AnyWindowHandle) {
    let (shell, handle) = workbench_fixture(cx);
    shell.update(cx, |shell, cx| {
        let session = shell.workbench.document.as_mut().unwrap();
        session.structure.outline = ferrostage_pdf::PdfOutline {
            entries: [
                (0, 1, Some("第一章"), Some(0), None),
                (1, 2, Some("1.1 完整保留的章节标题"), Some(2), None),
                (2, 1, Some("备注目标"), Some(1), None),
                (
                    3,
                    1,
                    Some("外部目标"),
                    None,
                    Some(ferrostage_pdf::PdfOutlineIssue::External),
                ),
                (
                    4,
                    1,
                    None,
                    None,
                    Some(ferrostage_pdf::PdfOutlineIssue::Unavailable),
                ),
                (5, 2, Some("有效子标题"), Some(3), None),
            ]
            .into_iter()
            .map(
                |(id, depth, title, page, issue)| ferrostage_pdf::PdfOutlineEntry {
                    id,
                    parent: (depth == 2).then_some(if id == 1 { 0 } else { 4 }),
                    depth,
                    title: title.map(str::to_owned),
                    page: page.map(PhysicalPage),
                    issue,
                },
            )
            .collect(),
            partial: true,
        };
        session.document =
            ferrostage_document::LoadedDocument::load(&session.metadata, &session.structure)
                .unwrap();
        cx.notify();
    });
    redraw(handle, cx);
    (shell, handle)
}

fn open_page_browser(shell: &Entity<Shell>, handle: AnyWindowHandle, cx: &mut TestAppContext) {
    click_control(handle, "workbench-progress", cx);
    shell.update(cx, |shell, cx| shell.poll_focus(cx));
}

fn focused_outline(handle: AnyWindowHandle, cx: &mut TestAppContext) -> Option<usize> {
    handle
        .downcast::<Presenter>()
        .unwrap()
        .update(cx, |view, window, _| view.browser_focused_outline(window))
        .unwrap()
}

#[gpui::test]
fn outline_tabs_and_rows_preserve_page_until_one_committed_keyboard_jump(cx: &mut TestAppContext) {
    for (key, presenting) in [
        ("enter", false),
        ("space", false),
        ("enter", true),
        ("space", true),
    ] {
        let (shell, handle) = outline_fixture(cx);
        if presenting {
            shell.update(cx, |shell, cx| {
                shell
                    .state
                    .apply_at(
                        Command::StartPresentation {
                            audience_available: true,
                        },
                        Instant::now(),
                    )
                    .unwrap();
                cx.notify();
            });
            redraw(handle, cx);
        }
        open_page_browser(&shell, handle, cx);
        redraw(handle, cx);
        // Numeric intent remains separate from the current-section badge and focus.
        cx.simulate_keystrokes(handle, "3");
        let (before, old_id) = shell.read_with(cx, |shell, _| {
            (shell.state.clone(), shell.browser().unwrap().id)
        });
        click_control(handle, "page-view-outline", cx);
        redraw(handle, cx);
        assert_eq!(focused_outline(handle, cx), Some(0));
        shell.read_with(cx, |shell, _| {
            assert_eq!(shell.state, before);
            assert_eq!(shell.browser().unwrap().target, Some(PhysicalPage(3)));
            assert_eq!(
                shell
                    .workbench
                    .document
                    .as_ref()
                    .unwrap()
                    .document
                    .current_outline_entry(shell.state.current_page()),
                Some(0)
            );
        });
        cx.simulate_keystrokes(handle, "down home end up");
        redraw(handle, cx);
        assert_eq!(
            focused_outline(handle, cx),
            Some(1),
            "disabled rows must be skipped"
        );
        shell.read_with(cx, |shell, _| assert_eq!(shell.state, before));
        shell.update(cx, |shell, cx| {
            shell.dispatch(
                Intent::BrowserJump {
                    id: old_id,
                    page: PhysicalPage(3),
                },
                cx,
            );
            shell.dispatch(
                Intent::BrowserOutlineJump {
                    id: old_id,
                    entry_id: 5,
                },
                cx,
            );
            shell.dispatch(Intent::GridMove(16), cx);
            assert_eq!(shell.workbench.grid_start, 0);
            assert_eq!(shell.state, before);
        });
        cx.simulate_keystrokes(handle, key);
        shell.read_with(cx, |shell, _| {
            assert_eq!(shell.state.current_page(), PhysicalPage(2), "{key}");
            assert_eq!(
                shell.state.stage(),
                if presenting {
                    SessionStage::Presenting
                } else {
                    SessionStage::Ready
                }
            );
            assert!(
                shell.browser().is_none(),
                "one activation closes the browser"
            );
        });
        redraw(handle, cx);
    }
}

#[gpui::test]
fn outline_keyboard_tabs_and_numbering_do_not_commit_or_lose_the_editor(cx: &mut TestAppContext) {
    let (shell, handle) = outline_fixture(cx);
    open_page_browser(&shell, handle, cx);
    redraw(handle, cx);
    click_control(handle, "page-view-outline", cx);
    redraw(handle, cx);
    let before = shell.read_with(cx, |shell, _| shell.state.clone());
    // First row -> selected Outline tab -> Thumbnail tab. Raw Space is consumed
    // by the contextual button instead of the configurable Next binding.
    for key in ["shift-tab", "shift-tab", "space"] {
        cx.simulate_keystrokes(handle, key);
        redraw(handle, cx);
    }
    shell.read_with(cx, |shell, _| {
        assert_eq!(
            shell.workbench.browser_view,
            crate::workbench::BrowserView::Thumbnails
        );
        assert_eq!(shell.state, before);
    });
    redraw(handle, cx);
    click_control(handle, "page-view-outline", cx);
    redraw(handle, cx);
    click_control(handle, "page-mode-pdf", cx);
    redraw(handle, cx);
    cx.simulate_keystrokes(handle, "ctrl-a 2 enter");
    shell.read_with(cx, |shell, _| {
        assert_eq!(
            shell.workbench.browser_view,
            crate::workbench::BrowserView::Outline
        );
        assert_eq!(
            shell.workbench.page_number_mode,
            crate::workbench::PageNumberMode::Pdf
        );
        assert_eq!(shell.browser().unwrap().error, Some("pb-notes-page"));
        assert_eq!(shell.state, before);
    });
    cx.simulate_keystrokes(handle, "ctrl-a 3 enter");
    shell.read_with(cx, |shell, _| {
        assert_eq!(shell.state.current_page(), PhysicalPage(2));
        assert!(shell.browser().is_none());
    });
}

#[gpui::test]
fn outline_numeric_editor_cancelled_press_and_disabled_rows_are_inert(cx: &mut TestAppContext) {
    let (shell, handle) = outline_fixture(cx);
    open_page_browser(&shell, handle, cx);
    redraw(handle, cx);
    click_control(handle, "page-view-outline", cx);
    redraw(handle, cx);
    // An actual row hover and aborted press must never commit its destination.
    let before = shell.read_with(cx, |shell, _| shell.state.clone());
    let mut visual = gpui::VisualTestContext::from_window(handle, cx);
    let row = visual.debug_bounds("outline-1").unwrap();
    let header = visual.debug_bounds("page-target-context").unwrap();
    visual.simulate_mouse_move(row.center(), None, gpui::Modifiers::default());
    visual.simulate_mouse_down(
        row.center(),
        gpui::MouseButton::Left,
        gpui::Modifiers::default(),
    );
    visual.simulate_mouse_move(
        header.center(),
        Some(gpui::MouseButton::Left),
        gpui::Modifiers::default(),
    );
    visual.simulate_mouse_up(
        header.center(),
        gpui::MouseButton::Left,
        gpui::Modifiers::default(),
    );
    for entry_id in [2, 3, 4] {
        shell.update(cx, |shell, cx| {
            shell.dispatch(
                Intent::BrowserOutlineJump {
                    id: shell.browser().unwrap().id,
                    entry_id,
                },
                cx,
            );
            assert_eq!(shell.state, before);
        });
    }
    // Shared number editing keeps Home/End, selection and the active section unchanged.
    click_control(handle, "page-number-input", cx);
    cx.simulate_keystrokes(handle, "ctrl-a 3 home end left right");
    shell.read_with(cx, |shell, _| {
        assert_eq!(shell.state, before);
        assert_eq!(shell.browser().unwrap().target, Some(PhysicalPage(3)));
    });
    cx.simulate_keystrokes(handle, "escape");
    redraw(handle, cx);
    shell.read_with(cx, |shell, _| assert!(shell.contextual.is_none()));
    // Reopening retains the view but takes its numeric target from committed state.
    open_page_browser(&shell, handle, cx);
    redraw(handle, cx);
    shell.read_with(cx, |shell, _| {
        assert_eq!(
            shell.workbench.browser_view,
            crate::workbench::BrowserView::Outline
        );
        assert_eq!(shell.browser().unwrap().target, Some(PhysicalPage(0)));
    });
    click_control(handle, "outline-1", cx);
    shell.read_with(cx, |shell, _| {
        assert_eq!(shell.state.current_page(), PhysicalPage(2));
        assert!(shell.contextual.is_none());
    });
}

#[gpui::test]
fn outline_switch_retires_thumbnail_work_and_ignores_stale_view_generation(
    cx: &mut TestAppContext,
) {
    use crate::workbench::{BrowserView, Surface};
    let fixture = SaveFixture::new();
    fixture.unblock();
    let (shell, handle) = outline_fixture(cx);
    shell.update(cx, |s, _| s.store = fixture.store());
    let worker = ferrostage_render::RenderHandle::with_backend(
        || Box::new(SettingsBackend),
        ferrostage_render::CacheBudgets::default(),
    )
    .unwrap();
    shell.update(cx, |shell, cx| {
        shell.open_browser();
        shell
            .workbench
            .schedule(&worker, &shell.state, &shell.settings.settings.presenter)
            .unwrap();
        assert!(
            shell
                .workbench
                .requested_surfaces()
                .iter()
                .any(|s| matches!(s, Surface::Thumbnail(_)))
        );
        cx.notify();
    });
    redraw(handle, cx);
    click_control(handle, "page-view-outline", cx);
    redraw(handle, cx);
    shell.update(cx, |shell, cx| {
        let current = shell.state.clone();
        let id = shell.browser().unwrap().id;
        shell.dispatch(
            Intent::GridDensity {
                column: true,
                value: 1,
            },
            cx,
        );
        shell.dispatch(
            Intent::BrowserJump {
                id,
                page: PhysicalPage(3),
            },
            cx,
        );
        shell
            .workbench
            .schedule(&worker, &shell.state, &shell.settings.settings.presenter)
            .unwrap();
        assert!(!shell.workbench.grid);
        assert!(
            shell.browser().is_some(),
            "directory is still a contextual surface"
        );
        assert!(
            !shell
                .workbench
                .requested_surfaces()
                .iter()
                .any(|s| matches!(s, Surface::Thumbnail(_)))
        );
        assert_eq!(shell.state, current);
        shell.dispatch(
            Intent::BrowserView {
                id,
                view: BrowserView::Thumbnails,
            },
            cx,
        );
        shell.dispatch(Intent::BrowserOutlineJump { id, entry_id: 5 }, cx);
        assert_eq!(shell.state, current);
        shell
            .workbench
            .schedule(&worker, &shell.state, &shell.settings.settings.presenter)
            .unwrap();
        assert!(
            (1..=16).contains(
                &shell
                    .workbench
                    .requested_surfaces()
                    .iter()
                    .filter(|s| matches!(s, Surface::Thumbnail(_)))
                    .count()
            )
        );
        let id = shell.browser().unwrap().id;
        shell.dispatch(
            Intent::BrowserView {
                id,
                view: BrowserView::Outline,
            },
            cx,
        );
        let stale_id = shell.browser().unwrap().id;
        let session = shell.workbench.document.clone().unwrap();
        shell.finish_import(
            PathBuf::from("replacement.pdf"),
            ferrostage_core::DocumentGeneration(3),
            session,
            cx,
        );
        shell.open_browser();
        let id = shell.browser().unwrap().id;
        shell.dispatch(
            Intent::BrowserView {
                id,
                view: BrowserView::Outline,
            },
            cx,
        );
        shell.dispatch(
            Intent::BrowserOutlineJump {
                id: stale_id,
                entry_id: 5,
            },
            cx,
        );
        assert_eq!(shell.state.current_page(), PhysicalPage(0));
    });
}

#[gpui::test]
fn outline_reveals_current_wrapped_title_and_restores_original_invoker(cx: &mut TestAppContext) {
    let (shell, handle) = outline_fixture(cx);
    cx.test_window(handle)
        .simulate_resize(size(px(1100.), px(760.)));
    shell.update(cx, |shell, cx| {
        let session = shell.workbench.document.as_mut().unwrap();
        session.structure.outline.entries = (0..39).map(|id| ferrostage_pdf::PdfOutlineEntry {
            id, parent: (id != 0).then_some(0), depth: if id == 0 { 1 } else { 2 },
            title: Some(if id == 38 { "完整标题不能截断，运行优化框架中的多速率数据融合与熟料质量指标建模及其可靠性验证。".repeat(3) } else { format!("章节 {id}") }),
            page: Some(PhysicalPage(if id == 38 { 2 } else { 0 })), issue: None,
        }).collect();
        session.document = ferrostage_document::LoadedDocument::load(&session.metadata, &session.structure).unwrap();
        shell.apply(Command::JumpTo(PhysicalPage(2)), cx);
        cx.notify();
    });
    redraw(handle, cx);
    // Mouse-down focuses the real invoker; only release opens the browser.
    let mut visual = gpui::VisualTestContext::from_window(handle, cx);
    let progress = visual.debug_bounds("workbench-progress").unwrap();
    visual.simulate_mouse_down(
        progress.center(),
        gpui::MouseButton::Left,
        gpui::Modifiers::default(),
    );
    let invoker = handle
        .update(cx, |_, window, cx| window.focused(cx))
        .unwrap();
    visual.simulate_mouse_up(
        progress.center(),
        gpui::MouseButton::Left,
        gpui::Modifiers::default(),
    );
    shell.update(cx, |shell, cx| shell.poll_focus(cx));
    redraw(handle, cx);
    click_control(handle, "page-view-outline", cx);
    redraw(handle, cx);
    cx.run_until_parked();
    cx.test_window(handle).simulate_scheduled_frame();
    redraw(handle, cx);
    assert_eq!(focused_outline(handle, cx), Some(38));
    let mut visual = gpui::VisualTestContext::from_window(handle, cx);
    let list = visual.debug_bounds("page-browser-outline").unwrap();
    let title = visual.debug_bounds("outline-title-38").unwrap();
    assert!(
        title.top() >= list.top() && title.bottom() <= list.bottom(),
        "current title {title:?} must be revealed in {list:?}"
    );
    assert!(
        title.size.height > px(44.),
        "full title should wrap rather than ellipsize"
    );
    assert!(visual.debug_bounds("thumbnail-grid").is_none());
    cx.simulate_keystrokes(handle, "escape");
    redraw(handle, cx);
    assert_eq!(
        handle
            .update(cx, |_, window, cx| window.focused(cx))
            .unwrap(),
        invoker
    );
}
