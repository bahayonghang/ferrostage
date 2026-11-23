#[gpui::test]
fn titlebar_renders_across_idle_settings_ready_and_presenting_with_synced_title(
    cx: &mut TestAppContext,
) {
    let (shell, handle) = input_presenter(cx);
    redraw(handle, cx);

    let presenter = handle.downcast::<Presenter>().unwrap();
    let assert_titlebar = |cx: &mut TestAppContext, expected_title: &str| {
        redraw(handle, cx);
        let mut visual = gpui::VisualTestContext::from_window(handle, cx);
        let bar = visual
            .debug_bounds("presenter-titlebar")
            .expect("presenter titlebar rendered");
        let title_bounds = visual
            .debug_bounds("titlebar-title")
            .expect("titlebar title rendered");
        assert!((f32::from(bar.size.height) - crate::theme::TITLEBAR_HEIGHT).abs() < 0.5);
        assert!(bar.contains(&title_bounds.center()));
        let probe = presenter
            .read_with(cx, |view, _| view.titlebar_probe().unwrap())
            .unwrap();
        assert_eq!(probe.title, expected_title);
    };

    assert_titlebar(cx, "FerroStage");

    shell.update(cx, |shell, cx| {
        shell.modal = Some(Modal::Settings);
        cx.notify();
    });
    assert_titlebar(cx, "FerroStage");
    {
        let mut visual = gpui::VisualTestContext::from_window(handle, cx);
        let bar = visual.debug_bounds("presenter-titlebar").unwrap();
        let settings_title = visual.debug_bounds("settings-title").unwrap();
        assert!(bar.bottom() <= settings_title.top());
    }

    shell.update(cx, |shell, cx| {
        shell.monitors = vec![test_monitor()];
        shell.new_display_draft();
        shell.modal = Some(Modal::Displays(shell.preferences.display.clone().unwrap()));
        cx.notify();
    });
    assert_titlebar(cx, "FerroStage");

    shell.update(cx, |shell, cx| {
        shell.modal = None;
        load_ready(shell);
        shell.path = Some(PathBuf::from("slides/talk.pdf"));
        cx.notify();
    });
    let ready_expected = shell.read_with(cx, |shell, _| {
        format!("talk.pdf — {} — FerroStage", shell.locale.text("stage-ready"))
    });
    assert_titlebar(cx, &ready_expected);

    let (crop_shell, crop_handle) = workbench_fixture(cx);
    crop_shell.update(cx, |shell, cx| {
        shell.path = Some(PathBuf::from("slides/crop.pdf"));
        shell.open_crop(None);
        cx.notify();
    });
    redraw(crop_handle, cx);
    {
        let mut visual = gpui::VisualTestContext::from_window(crop_handle, cx);
        assert!(visual.debug_bounds("presenter-titlebar").is_some());
        assert!(visual.debug_bounds("crop-panel").is_some());
    }

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
        shell.audience = Some(test_window(shell, true, cx));
        shell.publish(false, cx);
        cx.notify();
    });
    let presenting_expected = shell.read_with(cx, |shell, _| {
        format!(
            "talk.pdf — {} — FerroStage",
            shell.locale.text("stage-presenting")
        )
    });
    assert_titlebar(cx, &presenting_expected);
}

#[gpui::test]
fn titlebar_window_control_area_hit_testing_and_close_interception(cx: &mut TestAppContext) {
    let (shell, presenter, _audience) = audience_input_fixture(cx);
    redraw(presenter, cx);
    let mut visual = gpui::VisualTestContext::from_window(presenter, cx);

    for (selector, expected) in [
        ("titlebar-drag", gpui::WindowControlArea::Drag),
        ("titlebar-minimize", gpui::WindowControlArea::Min),
        ("titlebar-maximize", gpui::WindowControlArea::Max),
        ("titlebar-close", gpui::WindowControlArea::Close),
    ] {
        let bounds = visual
            .debug_bounds(selector)
            .unwrap_or_else(|| panic!("missing {selector}"));
        visual.simulate_mouse_move(bounds.center(), None, gpui::Modifiers::default());
        let hit = cx.test_window(presenter).simulate_window_control_hit_test();
        assert_eq!(hit, Some(expected), "hit test for {selector}");
    }

    assert!(!visual.simulate_close());
    cx.run_until_parked();
    assert!(cx.windows().is_empty());
    shell.read_with(cx, |shell, _| {
        assert!(shell.presenter.is_none());
        assert!(shell.audience.is_none());
    });
}

#[gpui::test]
fn titlebar_hides_in_fullscreen_and_restores_on_exit(cx: &mut TestAppContext) {
    let (shell, handle) = workbench_fixture(cx);
    redraw(handle, cx);
    assert!(
        gpui::VisualTestContext::from_window(handle, cx)
            .debug_bounds("presenter-titlebar")
            .is_some()
    );

    handle
        .update(cx, |_, window, _| window.toggle_fullscreen())
        .unwrap();
    redraw(handle, cx);
    assert!(
        gpui::VisualTestContext::from_window(handle, cx)
            .debug_bounds("presenter-titlebar")
            .is_none()
    );

    handle
        .update(cx, |_, window, _| window.toggle_fullscreen())
        .unwrap();
    redraw(handle, cx);
    assert!(
        gpui::VisualTestContext::from_window(handle, cx)
            .debug_bounds("presenter-titlebar")
            .is_some()
    );

    shell.update(cx, |shell, cx| {
        shell.monitors = vec![test_monitor()];
        shell.mapped = Ok(BTreeMap::from([(
            test_monitor().id,
            u64::from(cx.displays().first().unwrap().id()),
        )]));
        shell.enter_presenter_fullscreen(cx).unwrap();
    });
    redraw(handle, cx);
    assert!(
        gpui::VisualTestContext::from_window(handle, cx)
            .debug_bounds("presenter-titlebar")
            .is_none()
    );

    shell.update(cx, |shell, cx| shell.stop(cx));
    redraw(handle, cx);
    assert!(
        gpui::VisualTestContext::from_window(handle, cx)
            .debug_bounds("presenter-titlebar")
            .is_some()
    );
}

#[gpui::test]
fn titlebar_activation_changes_update_contrast_via_redraw(cx: &mut TestAppContext) {
    let (_shell, handle) = input_presenter(cx);
    let presenter = handle.downcast::<Presenter>().unwrap();

    let read_probe = |cx: &TestAppContext| {
        presenter
            .read_with(cx, |view, _| view.titlebar_probe().unwrap())
            .unwrap()
    };

    redraw(handle, cx);
    cx.test_window(handle).simulate_scheduled_frame();

    handle
        .update(cx, |_, window, _| window.activate_window())
        .unwrap();
    cx.run_until_parked();
    cx.test_window(handle).simulate_scheduled_frame();
    let active_probe = read_probe(cx);
    assert!(active_probe.active);
    assert_eq!(active_probe.colors.title_ink, crate::theme::LIGHT.titlebar_ink);
    assert_eq!(active_probe.colors.icon_ink, crate::theme::LIGHT.titlebar_icon);

    gpui::VisualTestContext::from_window(handle, cx).deactivate_window();
    cx.run_until_parked();
    cx.test_window(handle).simulate_scheduled_frame();
    let inactive_probe = read_probe(cx);
    assert!(!inactive_probe.active);
    assert_eq!(
        inactive_probe.colors.title_ink,
        crate::theme::LIGHT.titlebar_inactive_ink
    );
    assert_eq!(
        inactive_probe.colors.icon_ink,
        crate::theme::LIGHT.titlebar_inactive_icon
    );

    handle
        .update(cx, |_, window, _| window.activate_window())
        .unwrap();
    cx.run_until_parked();
    cx.test_window(handle).simulate_scheduled_frame();
    let reactivated_probe = read_probe(cx);
    assert!(reactivated_probe.active);
    assert_eq!(
        reactivated_probe.colors.title_ink,
        crate::theme::LIGHT.titlebar_ink
    );
    assert_eq!(
        reactivated_probe.colors.icon_ink,
        crate::theme::LIGHT.titlebar_icon
    );
}

#[gpui::test]
fn titlebar_tracks_runtime_theme_and_glass_changes_preserving_presenting_session(
    cx: &mut TestAppContext,
) {
    use ferrostage_settings::Theme;
    let fixture = SaveFixture::new();
    fixture.unblock();
    let (shell, presenter_handle, audience_handle) = audience_input_fixture(cx);
    let presenter = presenter_handle.downcast::<Presenter>().unwrap();
    let read_probe = |cx: &TestAppContext| {
        presenter
            .read_with(cx, |view, _| view.titlebar_probe().unwrap())
            .unwrap()
    };

    let snapshot_before = shell.update(cx, |shell, cx| {
        shell.store = fixture.store();
        shell
            .state
            .apply_at(Command::NextStep, Instant::now())
            .unwrap();
        shell
            .state
            .apply_at(
                Command::BeginStroke {
                    color: [0, 0, 255, 255],
                    width: 0.01,
                    is_highlighter: false,
                },
                Instant::now(),
            )
            .unwrap();
        shell
            .state
            .apply_at(Command::AppendStrokePoint((0.1, 0.1)), Instant::now())
            .unwrap();
        shell.state.apply_at(Command::EndStroke, Instant::now()).unwrap();
        shell.modal = Some(Modal::Settings);
        shell.preferences.category = crate::settings_panel::Category::General;
        cx.notify();
        (
            shell.state.current_page(),
            shell.state.has_annotations(),
            shell.state.elapsed_at(Instant::now()),
            shell.settings.settings.display.monitors.clone(),
        )
    });
    presenter_handle
        .update(cx, |_, window, _| window.activate_window())
        .unwrap();
    cx.run_until_parked();
    redraw(presenter_handle, cx);

    click_control(presenter_handle, "setting-general.theme-dark", cx);
    cx.run_until_parked();
    redraw(presenter_handle, cx);
    let dark_probe = read_probe(cx);
    assert_eq!(dark_probe.colors.title_ink, crate::theme::DARK.titlebar_ink);
    assert_eq!(dark_probe.field_top, crate::theme::DARK.field_top);

    click_control(presenter_handle, "setting-general.glass_effect", cx);
    cx.run_until_parked();
    redraw(presenter_handle, cx);
    assert!(!shell.read_with(cx, |shell, _| shell.settings.settings.general.glass_effect));

    click_control(presenter_handle, "setting-general.theme-system", cx);
    cx.run_until_parked();
    redraw(presenter_handle, cx);
    let system_light = read_probe(cx);
    assert_eq!(system_light.colors.title_ink, crate::theme::LIGHT.titlebar_ink);

    cx.test_window(presenter_handle)
        .simulate_appearance_change(gpui::WindowAppearance::Dark);
    cx.run_until_parked();
    redraw(presenter_handle, cx);
    let system_dark = read_probe(cx);
    assert_eq!(system_dark.colors.title_ink, crate::theme::DARK.titlebar_ink);

    shell.read_with(cx, |shell, _| {
        assert_eq!(shell.settings.settings.general.theme, Theme::System);
        assert_eq!(shell.presenter.as_ref().unwrap().handle, presenter_handle);
        assert_eq!(shell.audience.as_ref().unwrap().handle, audience_handle);
        assert_eq!(shell.state.current_page(), snapshot_before.0);
        assert_eq!(shell.state.has_annotations(), snapshot_before.1);
        assert_eq!(shell.state.elapsed_at(Instant::now()), snapshot_before.2);
        assert_eq!(shell.settings.settings.display.monitors, snapshot_before.3);
    });
}

#[gpui::test]
fn audience_and_identify_windows_isolate_titlebar_and_presenter_theme(cx: &mut TestAppContext) {
    let (shell, presenter, audience) = audience_input_fixture(cx);
    redraw(presenter, cx);
    redraw(audience, cx);

    let mut aud_visual = gpui::VisualTestContext::from_window(audience, cx);
    assert!(aud_visual.debug_bounds("presenter-titlebar").is_none());
    assert!(aud_visual.debug_bounds("titlebar-drag").is_none());
    assert!(aud_visual.debug_bounds("titlebar-close").is_none());

    shell.update(cx, |shell, cx| {
        shell.stop(cx);
        shell.monitors = vec![test_monitor()];
        shell.mapped = Ok(BTreeMap::from([(
            test_monitor().id,
            u64::from(cx.displays().first().unwrap().id()),
        )]));
        shell.open_identify(cx);
    });
    let identify = shell.read_with(cx, |shell, _| *shell.identify.first().unwrap());
    redraw(identify, cx);
    let mut id_visual = gpui::VisualTestContext::from_window(identify, cx);
    assert!(id_visual.debug_bounds("presenter-titlebar").is_none());
}
