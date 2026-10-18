use crate::workbench::PageNumberMode;

#[gpui::test]
fn numbering_switch_retires_cached_thumbnail_pixels_before_new_rasters(cx: &mut TestAppContext) {
    use crate::workbench::{ReadyFrame, Surface};
    let (shell, handle) = workbench_fixture(cx);
    let worker = ferrostage_render::RenderHandle::with_backend(
        || Box::new(SettingsBackend),
        ferrostage_render::CacheBudgets::default(),
    ).unwrap();
    shell.update(cx, |shell, cx| {
        shell.open_browser();
        shell.workbench.schedule(&worker, &shell.state, &shell.settings.settings.presenter).unwrap();
        let session = shell.workbench.document.as_ref().unwrap();
        // Content slot 2 is physical page 3; PDF slot 2 will be physical page 2.
        let request = ferrostage_render::RenderRequest::thumbnail(
            shell.state.generation(),
            &session.metadata.pages[3],
            shell.state.slide_crop(),
            0,
        ).unwrap();
        shell.workbench.frames = vec![(Surface::Thumbnail(2), ReadyFrame {
            request,
            frame: ferrostage_core::BgraFrame::new(2, 1, 8, vec![255; 8]).unwrap(),
        })];
        cx.notify();
    });
    redraw(handle, cx);
    let mut visual = gpui::VisualTestContext::from_window(handle, cx);
    assert!(visual.debug_bounds("surface-Thumbnail(2)").is_some());
    // Leave the old desired surface in place, just as it remains until the next tick.
    click_control(handle, "page-mode-pdf", cx);
    redraw(handle, cx);
    assert!(visual.debug_bounds("thumbnail-image-2").is_some());
    assert!(visual.debug_bounds("surface-Thumbnail(2)").is_none(),
        "a remapped slot must not fall back to the old page's GPU image");
}

#[gpui::test]
fn browser_modes_translate_targets_reject_notes_and_retire_queued_events(cx: &mut TestAppContext) {
    let (shell, handle) = workbench_fixture(cx);
    shell.update(cx, |shell, cx| {
        shell.settings = SettingsSession::from_outcome(
            ferrostage_settings::from_toml("schema_version = 999").unwrap(),
        );
        shell.apply(Command::AppendStrokePoint((0.2, 0.3)), cx);
        shell.apply(Command::EndStroke, cx);
        shell.open_browser();
        cx.notify();
    });
    redraw(handle, cx);
    cx.simulate_keystrokes(handle, "2");
    let (original, old_id, settings) = shell.read_with(cx, |shell, _| {
        assert_eq!(shell.browser().unwrap().target, Some(PhysicalPage(2)));
        (
            shell.state.clone(),
            shell.browser().unwrap().id,
            shell.settings.settings.clone(),
        )
    });
    click_control(handle, "page-mode-pdf", cx);
    redraw(handle, cx);
    shell.update(cx, |shell, cx| {
        assert_eq!(shell.workbench.page_number_mode, PageNumberMode::Pdf);
        assert_eq!(shell.browser().unwrap().edit.text, "3");
        for intent in [
            Intent::BrowserEdit {
                id: old_id,
                edit: crate::page_browser::NumberEdit::new("4".into()),
            },
            Intent::BrowserSubmit { id: old_id },
            Intent::BrowserJump {
                id: old_id,
                page: PhysicalPage(3),
            },
            Intent::BrowserMode {
                id: old_id,
                mode: PageNumberMode::Content,
            },
        ] {
            shell.dispatch(intent, cx);
        }
        assert_eq!(shell.browser().unwrap().edit.text, "3");
        assert_eq!(shell.state, original);
        assert_eq!(shell.settings.settings, settings);
        assert!(!shell.settings.dirty);
        assert!(
            !shell
                .workbench
                .document
                .as_ref()
                .unwrap()
                .document
                .overrides
                .physical_pages
        );
    });
    // Mode switching refocuses and selects the converted number for replacement.
    cx.simulate_keystrokes(handle, "2 enter");
    shell.read_with(cx, |shell, _| {
        assert_eq!(shell.browser().unwrap().error, Some("pb-notes-page"));
        assert_eq!(shell.locale.text("pb-notes-page"), "备注页不可投影");
        assert_eq!(shell.state, original);
    });
    click_control(handle, "commit-jump", cx); // Disabled mouse submit is inert.
    shell.read_with(cx, |shell, _| assert_eq!(shell.state, original));
    click_control(handle, "page-mode-content", cx);
    redraw(handle, cx);
    shell.read_with(cx, |shell, _| {
        assert_eq!(
            shell.browser().unwrap().edit.text,
            "1",
            "invalid notes input resets to current content"
        );
        assert_eq!(shell.state, original);
    });
    cx.simulate_keystrokes(handle, "2 enter");
    shell.read_with(cx, |shell, _| {
        assert_eq!(shell.state.current_page(), PhysicalPage(2));
        assert!(shell.contextual.is_none());
    });
    shell.update(cx, |shell, cx| {
        shell.open_browser();
        cx.notify();
    });
    redraw(handle, cx);
    click_control(handle, "page-mode-pdf", cx);
    redraw(handle, cx);
    let mut visual = gpui::VisualTestContext::from_window(handle, cx);
    assert!(visual.debug_bounds("thumbnail-image-1").is_some());
    assert!(
        visual.debug_bounds("surface-Thumbnail(1)").is_none(),
        "notes slot has no raster"
    );
    cx.simulate_keystrokes(handle, "escape");
    shell.update(cx, |shell, cx| {
        shell.open_browser();
        cx.notify();
    });
    shell.read_with(cx, |shell, _| {
        assert_eq!(shell.workbench.page_number_mode, PageNumberMode::Pdf);
        assert_eq!(shell.browser().unwrap().edit.text, "3");
    });
    shell.update(cx, |shell, cx| {
        let session = shell.workbench.document.clone().unwrap();
        shell.finish_import(
            PathBuf::from("new-document.pdf"),
            ferrostage_core::DocumentGeneration(3),
            session,
            cx,
        );
        assert_eq!(shell.workbench.page_number_mode, PageNumberMode::Content);
        assert!(shell.contextual.is_none());
    });
}

#[gpui::test]
fn remaining_tools_toggle_off_and_finish_live_gestures(cx: &mut TestAppContext) {
    let (shell, handle) = workbench_fixture(cx);
    let mut visual = gpui::VisualTestContext::from_window(handle, cx);
    assert!(visual.debug_bounds("navigate").is_none());
    for control in ["laser", "pen", "highlighter", "eraser"] {
        click_control(handle, control, cx);
        shell.update(cx, |shell, cx| {
            shell.apply(Command::ClearStrokes, cx);
            shell.apply(Command::AppendStrokePoint((0.2, 0.3)), cx);
            shell.apply(Command::UpdateLaserPointer(Some((0.4, 0.5))), cx);
        });
        redraw(handle, cx);
        click_control(handle, control, cx);
        shell.update(cx, |shell, cx| {
            assert_eq!(shell.workbench.tool, crate::workbench::Tool::Navigate);
            assert_eq!(shell.state.audience_snapshot().laser_pos_norm, None);
            shell.apply(Command::AppendStrokePoint((0.5, 0.6)), cx);
            assert_eq!(
                shell.state.audience_snapshot().strokes.len(),
                2,
                "toggle ends rather than extends live stroke"
            );
            assert_eq!(shell.state.current_page(), PhysicalPage(0));
            assert_eq!(shell.state.stage(), SessionStage::Ready);
        });
        redraw(handle, cx);
    }
}

#[gpui::test]
fn timing_header_uses_left_clock_and_stable_labeled_readout(cx: &mut TestAppContext) {
    let (shell, handle) = workbench_fixture(cx);
    cx.test_window(handle)
        .simulate_resize(size(px(1600.), px(900.)));
    redraw(handle, cx);
    let mut visual = gpui::VisualTestContext::from_window(handle, cx);
    let clock = visual.debug_bounds("presenter-clock").unwrap();
    let title = visual.debug_bounds("document-title").unwrap();
    let header = visual.debug_bounds("presenter-bar").unwrap();
    assert!(clock.right() <= title.left());
    assert!(clock.left() < visual.debug_bounds("presenter-timer").unwrap().left());
    shell.update(cx, |shell, cx| {
        shell.settings.settings.presenter.show_clock = false;
        cx.notify();
    });
    redraw(handle, cx);
    assert!(visual.debug_bounds("presenter-clock").is_none());
    assert!(visual.debug_bounds("document-title").unwrap().left() < title.left());
    assert_eq!(
        visual.debug_bounds("presenter-bar").unwrap().size.height,
        header.size.height
    );
    let mut timer_bounds = None;
    for elapsed in [599, 600, 5999, 6000] {
        shell.update(cx, |shell, cx| {
            shell.state = PresentationState::new(3).unwrap();
            shell
                .state
                .apply_at(
                    Command::StartPresentation {
                        audience_available: true,
                    },
                    Instant::now() - Duration::from_secs(elapsed),
                )
                .unwrap();
            shell.settings.settings.timer.mode = ferrostage_settings::TimerMode::CountUp;
            shell.settings.settings.presenter.show_clock = true;
            cx.notify();
        });
        redraw(handle, cx);
        let actual = visual.debug_bounds("presenter-timer-digits").unwrap();
        if let Some(expected) = timer_bounds {
            assert_eq!(actual, expected);
        } else {
            timer_bounds = Some(actual);
        }
        handle
            .downcast::<Presenter>()
            .unwrap()
            .update(cx, |view, _, _| {
                assert_eq!(view.timing_probe().1, "wb-elapsed");
                assert!(!view.timing_probe().0.contains('…'));
            })
            .unwrap();
    }
    shell.update(cx, |shell, cx| {
        shell.settings.settings.timer.mode = ferrostage_settings::TimerMode::CountDown;
        shell.settings.settings.timer.target_minutes = 1;
        cx.notify();
    });
    redraw(handle, cx);
    handle
        .downcast::<Presenter>()
        .unwrap()
        .update(cx, |view, _, _| {
            assert_eq!(view.timing_probe().1, "wb-overtime");
            assert!(view.timing_probe().0.starts_with('−'));
        })
        .unwrap();
    click_control(handle, "timer-pause", cx);
    redraw(handle, cx);
    handle
        .downcast::<Presenter>()
        .unwrap()
        .update(cx, |view, _, _| assert!(view.timing_probe().2))
        .unwrap();
    assert!(visual.debug_bounds("presenter-clock").is_some());
    click_control(handle, "timer-pause", cx);
    redraw(handle, cx);
    handle
        .downcast::<Presenter>()
        .unwrap()
        .update(cx, |view, _, _| assert!(!view.timing_probe().2))
        .unwrap();
    click_control(handle, "timer-reset", cx);
    redraw(handle, cx);
    handle
        .downcast::<Presenter>()
        .unwrap()
        .update(cx, |view, _, _| {
            assert_eq!(view.timing_probe().1, "wb-remaining");
            assert_eq!(view.timing_probe().0, "01:00");
        })
        .unwrap();
}

#[gpui::test]
fn explicit_automatic_keeps_detected_direction_labels_without_hidden_menu_rows(
    cx: &mut TestAppContext,
) {
    let (shell, handle) = workbench_fixture(cx);
    shell.update(cx, |shell, cx| {
        shell.settings = SettingsSession::from_outcome(
            ferrostage_settings::from_toml("schema_version = 999").unwrap(),
        );
        let mut session = shell.workbench.document.clone().unwrap();
        for page in &mut session.metadata.pages {
            page.width_points = 640.;
            page.height_points = 180.;
        }
        for page in &mut session.structure.pages {
            page.geometry.bounds = [0., 0., 640., 180.];
        }
        session.document =
            ferrostage_document::LoadedDocument::load(&session.metadata, &session.structure)
                .unwrap();
        shell.finish_import(
            PathBuf::from("wide.pdf"),
            ferrostage_core::DocumentGeneration(3),
            session,
            cx,
        );
        assert_eq!(
            shell.state.slide_crop(),
            ferrostage_core::NormalizedRect::FULL
        );
        shell.dispatch(Intent::Document, cx);
    });
    redraw(handle, cx);
    click_control(handle, "layout-row-0", cx);
    shell.read_with(cx, |shell, _| {
        assert_eq!(
            shell.workbench.document.as_ref().unwrap().document.layout,
            ferrostage_document::NotesLayout::Right
        );
        assert_eq!(
            shell.state.slide_crop(),
            ferrostage_core::NormalizedRect::LEFT_HALF
        );
    });
    redraw(handle, cx);
    click_control(handle, "document-options", cx);
    redraw(handle, cx); // Rendering explicit Auto/Right must not search a removed selectable row.
    let mut visual = gpui::VisualTestContext::from_window(handle, cx);
    for row in LAYOUT_ROWS {
        assert!(visual.debug_bounds(row).is_some());
    }
    assert!(visual.debug_bounds("layout-row-3").is_none());
    assert!(visual.debug_bounds("layout-check-0").is_some());
    click_control(handle, "layout-row-1", cx);
    shell.read_with(cx, |shell, _| {
        assert_eq!(
            shell.state.slide_crop(),
            ferrostage_core::NormalizedRect::FULL
        )
    });
}
