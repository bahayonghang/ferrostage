#[gpui::test]
fn crop_menu_keyboard_changes_only_the_draft_and_escape_restores_focus(cx: &mut TestAppContext) {
    let (shell, handle) = workbench_fixture(cx);
    click_control(handle, "document-options", cx);
    redraw(handle, cx);
    // The inspection action precedes the final layout-reset action.
    cx.simulate_keystrokes(handle, "end up enter");
    redraw(handle, cx);
    let mut visual = gpui::VisualTestContext::from_window(handle, cx);
    for selector in [
        "crop-panel",
        "crop-public-preview",
        "crop-notes-preview",
        "crop-cancel",
    ] {
        assert!(visual.debug_bounds(selector).is_some(), "{selector}");
    }
    cx.simulate_keystrokes(handle, "right pagedown");
    redraw(handle, cx);
    shell.read_with(cx, |s, _| {
        let crop = s.crop_preview().expect("active crop draft");
        assert_eq!(crop.layout, ferrostage_document::NotesLayout::Right);
        assert_eq!(crop.sample_index, 1);
        assert!(!crop.can_confirm, "fixture has no loaded renderer");
        assert_eq!(s.state.current_page(), PhysicalPage(0));
        assert_eq!(
            s.workbench.document.as_ref().unwrap().document.layout,
            ferrostage_document::NotesLayout::None
        );
    });
    click_control(handle, "crop-confirm", cx);
    redraw(handle, cx);
    shell.read_with(cx, |s, _| assert!(s.crop_preview().is_some()));
    cx.simulate_keystrokes(handle, "escape");
    redraw(handle, cx);
    assert!(layout_focus(handle, cx).1, "return to the layout trigger");
    shell.read_with(cx, |s, _| {
        assert!(s.crop_preview().is_none());
        assert!(s.audience.is_none());
        assert_eq!(s.state.stage(), SessionStage::Ready);
    });
}

#[gpui::test]
fn presenting_crop_review_is_read_only_and_cancel_keeps_presenting(cx: &mut TestAppContext) {
    let (shell, handle) = workbench_fixture(cx);
    shell.update(cx, |s, cx| {
        s.apply(
            Command::StartPresentation {
                audience_available: true,
            },
            cx,
        )
    });
    redraw(handle, cx);
    click_control(handle, "document-options", cx);
    redraw(handle, cx);
    click_control(handle, "layout-row-2", cx);
    redraw(handle, cx);
    shell.read_with(cx, |s, _| {
        assert!(s.layout_menu_open());
        assert!(
            s.workbench
                .document
                .as_ref()
                .unwrap()
                .document
                .overrides
                .identify_independent_notes
        );
    });
    click_control(handle, "crop-check", cx);
    redraw(handle, cx);
    shell.read_with(cx, |s, _| assert!(s.crop_preview().unwrap().read_only));
    let mut visual = gpui::VisualTestContext::from_window(handle, cx);
    assert!(visual.debug_bounds("crop-confirm").is_none());
    click_control(handle, "crop-cancel", cx);
    redraw(handle, cx);
    shell.read_with(cx, |s, _| {
        assert!(s.crop_preview().is_none());
        assert_eq!(s.state.stage(), SessionStage::Presenting);
    });
}
