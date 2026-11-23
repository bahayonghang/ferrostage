#[gpui::test]
fn audience_click_toggle_is_live_durable_and_does_not_replace_windows(cx: &mut TestAppContext) {
    let fixture = SaveFixture::new();
    let (shell, presenter, audience) = audience_input_fixture(cx);
    let before = shell.update(cx, |s, cx| {
        s.store = fixture.store();
        s.modal = Some(Modal::Settings);
        s.preferences.category = crate::settings_panel::Category::Displays;
        cx.notify();
        (s.state.clone(), s.settings.settings.display.clone())
    });
    redraw(presenter, cx);
    click_control(presenter, "setting-display.audience_click_to_advance", cx);
    cx.run_until_parked();
    shell.read_with(cx, |s, _| {
        assert!(!s.settings.settings.display.audience_click_to_advance);
        assert!(s.settings.dirty && s.settings.save_error.is_some());
        assert!(s.pending.is_none() && s.preference_change.is_none());
        assert_eq!(s.state, before.0);
        assert_eq!(s.settings.settings.display.monitors, before.1.monitors);
        assert_eq!(s.presenter.as_ref().unwrap().handle, presenter);
        assert_eq!(s.audience.as_ref().unwrap().handle, audience);
    });
    shell.update(cx, |s, _| s.modal = None);
    click_audience(audience, cx);
    assert_eq!(shell.read_with(cx, |s, _| s.state.current_page()), PhysicalPage(0));
    fixture.unblock();
    shell.update(cx, |s, cx| {
        s.dispatch(Intent::RetrySave, cx);
        assert!(!s.settings.dirty);
        assert!(!s.store.load().unwrap().settings.display.audience_click_to_advance);
        s.modal = Some(Modal::Settings);
        cx.notify();
    });
    redraw(presenter, cx);
    // The original toggle retains focus across live preference updates.
    cx.simulate_keystrokes(presenter, "space");
    cx.run_until_parked();
    shell.update(cx, |s, _| {
        assert!(s.settings.settings.display.audience_click_to_advance);
        let fresh = ferrostage_settings::SettingsSession::from_outcome(s.store.load().unwrap());
        assert!(fresh.settings.display.audience_click_to_advance);
        assert_eq!(s.presenter.as_ref().unwrap().handle, presenter);
        assert_eq!(s.audience.as_ref().unwrap().handle, audience);
        assert_eq!(s.state, before.0);
        s.modal = None;
    });
    click_audience(audience, cx);
    assert_eq!(shell.read_with(cx, |s, _| s.state.current_page()), PhysicalPage(1));

    // A future-version session renders the same control as read-only.
    let bytes = std::fs::read(fixture.store().path()).unwrap();
    shell.update(cx, |s, cx| {
        s.settings.write_policy = ferrostage_settings::from_toml("schema_version=999").unwrap().write_policy;
        s.modal = Some(Modal::Settings);
        cx.notify();
    });
    redraw(presenter, cx);
    click_control(presenter, "setting-display.audience_click_to_advance", cx);
    cx.simulate_keystrokes(presenter, "space");
    cx.run_until_parked();
    assert!(shell.read_with(cx, |s, _| s.settings.settings.display.audience_click_to_advance));
    assert_eq!(std::fs::read(fixture.store().path()).unwrap(), bytes);
}

#[gpui::test]
fn audience_click_combined_placement_edit_keeps_display_transaction(cx: &mut TestAppContext) {
    let fixture = SaveFixture::new();
    fixture.unblock();
    let (shell, _) = display_fixture(cx);
    shell.update(cx, |s, cx| {
        s.store = fixture.store();
        set_playing(s, cx, Instant::now());
        s.modal = Some(Modal::Settings);
        let mut next = s.settings.settings.clone();
        next.display.audience_click_to_advance = false;
        next.display.audience_fullscreen = !next.display.audience_fullscreen;
        next.display.single_monitor_mode = ferrostage_settings::SingleMonitorMode::WindowedAudience;
        s.apply_preferences(next, cx);
        assert!(s.pending.is_some() && s.preference_change.is_some());
        assert!(s.settings.settings.display.audience_click_to_advance);
        s.poll_transaction(cx);
        assert!(s.pending.is_none() && s.preference_change.is_none());
        assert!(!s.settings.settings.display.audience_click_to_advance);
        assert!(!s.store.load().unwrap().settings.display.audience_click_to_advance);
    });
}

#[gpui::test]
fn theme_and_glass_controls_persist_independently_during_presentation(cx: &mut TestAppContext) {
    use ferrostage_settings::Theme;
    let fixture = SaveFixture::new();
    fixture.unblock();
    let (shell, presenter, audience) = audience_input_fixture(cx);
    let before = shell.update(cx, |s, cx| {
        s.store = fixture.store();
        s.modal = Some(Modal::Settings);
        s.preferences.category = crate::settings_panel::Category::General;
        cx.notify();
        s.state.clone()
    });
    redraw(presenter, cx);
    click_control(presenter, "setting-general.theme-dark", cx);
    cx.run_until_parked();
    redraw(presenter, cx);
    click_control(presenter, "setting-general.glass_effect", cx);
    cx.run_until_parked();
    shell.read_with(cx, |s, _| {
        let saved = s.store.load().unwrap().settings.general;
        assert_eq!(saved.theme, Theme::Dark);
        assert!(!saved.glass_effect);
        assert_eq!(s.state, before);
        assert_eq!(s.audience.as_ref().unwrap().handle, audience);
    });
    redraw(presenter, cx);
    cx.simulate_keystrokes(presenter, "space");
    cx.run_until_parked();
    redraw(presenter, cx);
    click_control(presenter, "setting-general.theme-system", cx);
    cx.run_until_parked();
    shell.read_with(cx, |s, _| {
        let saved = s.store.load().unwrap().settings.general;
        assert_eq!(saved.theme, Theme::System);
        assert!(saved.glass_effect);
        assert_eq!(s.settings.settings.general, saved);
        assert_eq!(s.state, before);
        assert_eq!(s.presenter.as_ref().unwrap().handle, presenter);
    });
}

#[gpui::test]
fn settings_v09_edit_save_retry_readonly_and_runtime_consumers(cx: &mut TestAppContext) {
    let fixture = SaveFixture::new();
    let (shell, handle) = input_presenter(cx);
    shell.update(cx, |shell, cx| {
        shell.store = fixture.store();
        load_ready(shell);
        shell.modal = Some(Modal::Settings);
        cx.notify();
    });
    redraw(handle, cx);
    click_control(handle, "setting-general.theme-dark", cx);
    cx.run_until_parked();
    shell.read_with(cx, |shell, _| {
        assert_eq!(
            shell.settings.settings.general.theme,
            ferrostage_settings::Theme::Dark
        );
        assert!(shell.settings.dirty);
        assert!(shell.settings.save_error.is_some());
    });
    fixture.unblock();
    redraw(handle, cx);
    click_control(handle, "settings-retry", cx);
    cx.run_until_parked();
    assert!(!shell.read_with(cx, |shell, _| shell.settings.dirty));
    shell.update(cx, |shell, cx| {
        for (id, value) in [
            ("presenter.notes_font_scale", "2"),
            ("presenter.grid_columns", "4"),
            ("presenter.grid_visible_pages", "13"),
            ("timer.target_minutes", "42"),
            ("pointer.laser_color_bgra", "112233FF"),
            ("general.language", "en-US"),
        ] {
            shell.dispatch(Intent::EditSetting(id, value.into()), cx);
        }
        assert_eq!(shell.settings.settings.presenter.notes_font_scale, 2.);
        assert_eq!(shell.settings.settings.presenter.grid_columns, 4);
        assert_eq!(shell.settings.settings.presenter.grid_visible_pages, 13);
        assert_eq!(shell.settings.settings.timer.target_minutes, 42);
        assert_eq!(
            shell.state.audience_snapshot().laser_style.color_bgra,
            [0x33, 0x22, 0x11, 255]
        );
        assert_eq!(
            shell.locale.text("settings"),
            crate::locale::Locale::new("en-US").text("settings"),
            "language switches catalog immediately without restart"
        );
        assert_eq!(
            shell.store.load().unwrap().settings.general.language,
            "en-US"
        );
        let before = shell.settings.settings.clone();
        shell.dispatch(
            Intent::EditSetting("timer.warn_red_remaining_min", "999".into()),
            cx,
        );
        assert_eq!(shell.settings.settings, before);
        assert!(shell.preferences.error.is_some());
        shell.settings.write_policy = ferrostage_settings::from_toml("schema_version=999")
            .unwrap()
            .write_policy;
        shell.dispatch(Intent::EditSetting("general.theme", "light".into()), cx);
        shell.dispatch(Intent::ResetPreferences(true), cx);
        shell.dispatch(Intent::ClearRecent, cx);
        assert_eq!(shell.settings.settings, before);
    });
    redraw(handle, cx);
    let mut visual = gpui::VisualTestContext::from_window(handle, cx);
    assert_eq!(
        handle
            .downcast::<Presenter>()
            .unwrap()
            .read_with(cx, |view, _| view.titlebar_probe().unwrap().title)
            .unwrap(),
        "FerroStage — Ready — FerroStage"
    );
    assert_eq!(
        visual.window_title(),
        Some("FerroStage — Ready — FerroStage".into())
    );
}

#[gpui::test]
fn settings_v09_record_replaces_keymap_and_capture_precedes_actions(cx: &mut TestAppContext) {
    let fixture = SaveFixture::new();
    fixture.unblock();
    let (shell, handle) = input_presenter(cx);
    shell.update(cx, |shell, cx| {
        shell.store = fixture.store();
        load_ready(shell);
        shell.modal = Some(Modal::Settings);
        shell.preferences.category = crate::settings_panel::Category::Keys;
        cx.notify();
    });
    redraw(handle, cx);
    click_control(handle, "record-start_presentation", cx);
    cx.run_until_parked();
    redraw(handle, cx);
    cx.simulate_keystrokes(handle, "f6");
    cx.run_until_parked();
    shell.read_with(cx, |shell, _| {
        assert!(
            shell
                .settings
                .settings
                .keybindings
                .start_presentation
                .contains(&"f6".into())
        );
        assert_eq!(shell.state.stage(), SessionStage::Ready);
        assert!(shell.preferences.recording.is_none());
    });
    // Capture an existing global binding; it must become input instead of starting.
    redraw(handle, cx);
    click_control(handle, "record-next", cx);
    cx.run_until_parked();
    redraw(handle, cx);
    cx.simulate_keystrokes(handle, "f6");
    cx.run_until_parked();
    shell.read_with(cx, |shell, _| {
        let (keys, warning) = policy::bindings(&shell.settings.settings.keybindings, shell.locale);
        assert!(!keys.iter().any(|(key, _)| key == "f6"));
        assert!(!warning.is_empty());
    });
    shell.update(cx, |shell, cx| {
        shell.dispatch(Intent::RemoveBinding("start_presentation", 1), cx);
        shell.dispatch(Intent::Escape, cx);
    });
    redraw(handle, cx);
    cx.simulate_keystrokes(handle, "f6");
    cx.run_until_parked();
    assert_eq!(
        shell.read_with(cx, |shell, _| shell.state.current_page()),
        PhysicalPage(1)
    );
    shell.update(cx, |shell, cx| {
        shell.modal = Some(Modal::Settings);
        shell.dispatch(Intent::ResetBindings, cx);
        shell.dispatch(Intent::Escape, cx);
    });
    redraw(handle, cx);
    cx.simulate_keystrokes(handle, "f6");
    cx.run_until_parked();
    assert_eq!(
        shell.read_with(cx, |shell, _| shell.state.current_page()),
        PhysicalPage(1),
        "removed F6 must not remain in keymap"
    );
}

#[gpui::test]
fn settings_v09_numeric_editor_and_slider_use_real_keyboard_events(cx: &mut TestAppContext) {
    let fixture = SaveFixture::new();
    fixture.unblock();
    let (shell, handle) = input_presenter(cx);
    shell.update(cx, |shell, cx| {
        shell.store = fixture.store();
        load_ready(shell);
        shell.modal = Some(Modal::Settings);
        shell.preferences.category = crate::settings_panel::Category::Presenter;
        cx.notify();
    });
    redraw(handle, cx);
    click_control(handle, "setting-presenter.notes_font_scale-value", cx);
    cx.run_until_parked();
    redraw(handle, cx);
    cx.simulate_keystrokes(handle, "ctrl-a 2 enter");
    cx.run_until_parked();
    assert_eq!(
        shell.read_with(cx, |shell, _| shell
            .settings
            .settings
            .presenter
            .notes_font_scale),
        2.
    );
    redraw(handle, cx);
    let mut visual = gpui::VisualTestContext::from_window(handle, cx);
    let bounds = visual
        .debug_bounds("slider-presenter.next_slide_ratio")
        .unwrap();
    visual.simulate_click(bounds.center(), gpui::Modifiers::default());
    cx.run_until_parked();
    let ratio = shell.read_with(cx, |shell, _| {
        shell.settings.settings.presenter.next_slide_ratio
    });
    assert!((ratio - 0.53).abs() < 0.02, "{ratio}");
    redraw(handle, cx);
    cx.simulate_keystrokes(handle, "right");
    cx.run_until_parked();
    assert!(
        shell.read_with(cx, |shell, _| shell
            .settings
            .settings
            .presenter
            .next_slide_ratio)
            > ratio
    );
    assert_eq!(
        shell.read_with(cx, |shell, _| shell.state.current_page()),
        PhysicalPage(0)
    );
}

#[cfg(target_os = "windows")]
#[test]
fn display_settings_both_hosts_fit_windows_main_thread_stack() {
    // Rust's default test threads have 2 MiB, while the Windows executable has
    // a 1 MiB main stack. Isolate overflow (which aborts) from the test suite.
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "shell::tests::displays_design_both_hosts_fit_three_rows_and_cancel_only_allocation",
            "--nocapture",
        ])
        .env("RUST_MIN_STACK", "1048576")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "display hosts exceeded the native stack budget:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("1 passed"));
}

#[gpui::test]
fn settings_category_text_stays_centered_when_selection_changes(cx: &mut TestAppContext) {
    let (shell, handle) = input_presenter(cx);
    shell.update(cx, |shell, cx| {
        shell.modal = Some(Modal::Settings);
        shell.settings.settings.general.ui_font_size = 18;
        cx.notify();
    });
    for width in [1280., 710.] {
        cx.simulate_window_resize(handle, size(px(width), px(780.)));
        let mut previous = Vec::new();
        for selected in ["settings-category-general", "settings-category-pointer"] {
            redraw(handle, cx);
            click_control(handle, selected, cx);
            redraw(handle, cx);
            let mut visual = gpui::VisualTestContext::from_window(handle, cx);
            let mut centers = Vec::new();
            for (id, text, mark) in [
                (
                    "settings-category-general",
                    "settings-category-general-label",
                    "settings-category-general-selected",
                ),
                (
                    "settings-category-pointer",
                    "settings-category-pointer-label",
                    "settings-category-pointer-selected",
                ),
            ] {
                let row = visual.debug_bounds(id).unwrap();
                let label = visual.debug_bounds(text).unwrap();
                assert!(label.size.width < row.size.width);
                let delta = label.center() - row.center();
                assert!(
                    f32::from(delta.x).abs() <= 1. && f32::from(delta.y).abs() <= 1.,
                    "{width}/{id}/{selected}: {delta:?}"
                );
                assert_eq!(visual.debug_bounds(mark).is_some(), selected == id);
                centers.push(label.center());
            }
            if !previous.is_empty() {
                assert_eq!(previous, centers);
            }
            previous = centers;
        }
        cx.simulate_keystrokes(handle, "shift-tab enter");
        cx.run_until_parked();
        assert_eq!(
            shell.read_with(cx, |s, _| s.preferences.category),
            crate::settings_panel::Category::Timer
        );
        redraw(handle, cx);
        cx.simulate_keystrokes(handle, "tab space");
        cx.run_until_parked();
        assert_eq!(
            shell.read_with(cx, |s, _| s.preferences.category),
            crate::settings_panel::Category::Pointer
        );
    }
}

#[gpui::test]
fn settings_centers_navigation_and_category_body_with_aligned_header_and_feedback(
    cx: &mut TestAppContext,
) {
    let (shell, handle) = input_presenter(cx);
    shell.update(cx, |shell, cx| {
        shell.modal = Some(Modal::Settings);
        cx.notify();
    });
    cx.simulate_window_resize(handle, size(px(1600.), px(900.)));
    redraw(handle, cx);
    for (category, body_width) in [
        ("settings-category-general", 960.),
        ("settings-category-displays", 1120.),
    ] {
        click_control(handle, category, cx);
        redraw(handle, cx);
        let mut visual = gpui::VisualTestContext::from_window(handle, cx);
        let rail = visual.debug_bounds(category).unwrap();
        let body = visual.debug_bounds("settings-content").unwrap();
        let left = rail.left();
        let right = px(1600.) - body.right();
        assert!(f32::from(left - right).abs() <= 1., "{category}: {left:?} vs {right:?}");
        assert_eq!(body.size.width, px(body_width), "{category}");
        assert_eq!(visual.debug_bounds("settings-title").unwrap().left(), rail.left());
        assert_eq!(visual.debug_bounds("close-settings").unwrap().right(), body.right());
        assert_eq!(visual.debug_bounds("settings-feedback").unwrap().left(), body.left());
    }
}

#[gpui::test]
fn narrow_settings_wraps_categories_and_scrolls_to_the_last_display_control(
    cx: &mut TestAppContext,
) {
    let (shell, handle) = input_presenter(cx);
    shell.update(cx, |shell, cx| {
        shell.settings.settings.general.ui_font_size = 24;
        shell.modal = Some(Modal::Settings);
        cx.notify();
    });
    cx.simulate_window_resize(handle, size(px(390.), px(640.)));
    redraw(handle, cx);
    click_control(handle, "settings-category-displays", cx);
    redraw(handle, cx);
    let mut visual = gpui::VisualTestContext::from_window(handle, cx);
    let body = visual.debug_bounds("settings-content").unwrap();
    let first = visual.debug_bounds("settings-category-general").unwrap();
    let last = visual.debug_bounds("settings-category-about").unwrap();
    assert!(first.bottom() <= last.top() && last.bottom() <= body.top());
    assert!(body.size.height >= px(44.));
    visual.simulate_event(gpui::ScrollWheelEvent {
        position: body.center(),
        delta: gpui::ScrollDelta::Lines(gpui::point(0., -100.)),
        modifiers: gpui::Modifiers::default(),
        touch_phase: gpui::TouchPhase::Moved,
    });
    redraw(handle, cx);
    let toggle = visual.debug_bounds("setting-display.presenter_start_fullscreen").unwrap();
    assert!(toggle.left() >= body.left() && toggle.right() <= body.right()
        && toggle.top() >= body.top() && toggle.bottom() <= body.bottom(),
        "bottom display control {toggle:?} must fit in the scrolling body {body:?}");
    click_control(handle, "close-settings", cx);
    assert!(shell.read_with(cx, |shell, _| shell.modal.is_none()));
}

#[gpui::test]
fn switching_settings_category_opens_its_first_controls_after_scrolling(
    cx: &mut TestAppContext,
) {
    let (shell, handle) = input_presenter(cx);
    shell.update(cx, |shell, cx| {
        shell.settings.settings.general.ui_font_size = 20;
        shell.modal = Some(Modal::Settings);
        cx.notify();
    });
    cx.simulate_window_resize(handle, size(px(710.), px(640.)));
    redraw(handle, cx);
    let mut visual = gpui::VisualTestContext::from_window(handle, cx);
    let body = visual.debug_bounds("settings-content").unwrap();
    visual.simulate_event(gpui::ScrollWheelEvent {
        position: body.center(),
        delta: gpui::ScrollDelta::Lines(gpui::point(0., -100.)),
        modifiers: gpui::Modifiers::default(),
        touch_phase: gpui::TouchPhase::Moved,
    });
    redraw(handle, cx);
    assert!(visual.debug_bounds("setting-row-general.language").unwrap().top() < body.top());
    click_control(handle, "settings-category-displays", cx);
    redraw(handle, cx);
    let body = visual.debug_bounds("settings-content").unwrap();
    let refresh = visual.debug_bounds("refresh-displays").unwrap();
    assert!(refresh.top() >= body.top() && refresh.bottom() <= body.bottom(),
        "refresh {refresh:?} must be visible on entering Displays, inside {body:?}");
    click_control(handle, "settings-category-general", cx);
    redraw(handle, cx);
    let language = visual.debug_bounds("setting-row-general.language").unwrap();
    let body = visual.debug_bounds("settings-content").unwrap();
    assert!(language.top() >= body.top() && language.bottom() <= body.bottom());
}

#[gpui::test]
fn settings_v09_all_categories_render_and_footer_stays_reachable(cx: &mut TestAppContext) {
    let (shell, handle) = input_presenter(cx);
    for category in crate::settings_panel::Category::ALL {
        shell.update(cx, |shell, cx| {
            shell.modal = Some(Modal::Settings);
            shell.preferences.category = category;
            shell.preferences.display = Some(DisplayDraft::new(
                shell.topology_revision,
                &shell.monitors,
                &shell.settings.settings.display,
            ));
            cx.notify();
        });
        for (w, h) in [(1000., 760.), (390., 844.)] {
            cx.simulate_window_resize(handle, size(px(w), px(h)));
            redraw(handle, cx);
            let mut visual = gpui::VisualTestContext::from_window(handle, cx);
            let bounds = visual.debug_bounds("close-settings").unwrap();
            assert!(
                bounds.bottom() <= px(h) && bounds.right() <= px(w),
                "{category:?}: {bounds:?}"
            );
        }
    }
}

#[gpui::test]
fn settings_v09_dual_gpu_preflight_never_applies_half_a_transaction(cx: &mut TestAppContext) {
    use ferrostage_core::{AudienceVisual, BgraFrame};
    let fixture = SaveFixture::new();
    fixture.unblock();
    let (shell, handle) = input_presenter(cx);
    shell.update(cx, |shell, cx| {
        shell.store = fixture.store();
        load_ready(shell);
        set_playing(shell, cx, Instant::now());
        shell.modal = Some(Modal::Settings);
        let frame = BgraFrame::new(1024, 512, 4096, vec![7; 1024 * 512 * 4]).unwrap();
        let budget = shell.settings.settings.cache.l1_gpu_mib as usize * 1024 * 1024 / 2;
        shell.snapshot.update(cx, |snapshot, _| {
            snapshot.visual = Some(AudienceVisual {
                page: frame.clone(),
                texture_budget_bytes: budget,
                media: Vec::new(),
            })
        });
        let presenter = handle.downcast::<Presenter>().unwrap();
        presenter
            .update(cx, |view, _, _| view.apply_texture_budget(budget).unwrap())
            .unwrap();
        let audience = shell
            .audience
            .as_ref()
            .unwrap()
            .handle
            .downcast::<Audience>()
            .unwrap();
        audience
            .update(cx, |view, window, cx| {
                view.seed_budget_image(frame.clone(), budget, window, cx)
            })
            .unwrap();
        let before = shell.settings.settings.clone();
        shell.dispatch(Intent::EditSetting("cache.l1_gpu_mib", "1".into()), cx);
        assert_eq!(shell.settings.settings, before);
        assert!(shell.preferences.error.is_some());
        assert!(!shell.settings.dirty);
        assert_eq!(
            presenter.read(cx).unwrap().effective_texture_budget(),
            budget
        );
        assert_eq!(
            audience.read(cx).unwrap().effective_texture_budget(),
            budget
        );
        assert_eq!(
            shell
                .snapshot
                .read(cx)
                .visual
                .as_ref()
                .unwrap()
                .texture_budget_bytes,
            budget
        );
        // A cover with no visual does not erase the retained image or authorize a shrink.
        shell.snapshot.update(cx, |snapshot, _| {
            snapshot.visual = None;
            snapshot.stage = AudienceStage::Curtain;
        });
        shell.dispatch(Intent::EditSetting("cache.l1_gpu_mib", "512".into()), cx);
        assert_eq!(shell.settings.settings, before);
        assert_eq!(
            audience.read(cx).unwrap().effective_texture_budget(),
            budget
        );
        // Unrelated settings remain editable behind the safety cover.
        shell.dispatch(Intent::EditSetting("general.theme", "dark".into()), cx);
        assert_eq!(
            shell.settings.settings.general.theme,
            ferrostage_settings::Theme::Dark
        );
        shell.snapshot.update(cx, |snapshot, _| {
            snapshot.visual = Some(AudienceVisual {
                page: frame,
                texture_budget_bytes: budget,
                media: Vec::new(),
            })
        });
        shell.dispatch(Intent::EditSetting("cache.l1_gpu_mib", "512".into()), cx);
        assert_eq!(shell.settings.settings.cache.l1_gpu_mib, 512);
        assert_eq!(
            presenter.read(cx).unwrap().effective_texture_budget(),
            256 * 1024 * 1024
        );
        assert_eq!(
            audience.read(cx).unwrap().effective_texture_budget(),
            256 * 1024 * 1024
        );
        let frame = BgraFrame::new(1024, 512, 4096, vec![9; 1024 * 512 * 4]).unwrap();
        presenter
            .update(cx, |view, window, cx| {
                view.seed_budget_image(frame, 256 * 1024 * 1024, window, cx)
            })
            .unwrap();
        shell.dispatch(Intent::EditSetting("cache.l1_gpu_mib", "1".into()), cx);
        assert_eq!(shell.settings.settings.cache.l1_gpu_mib, 512);
    });
}

struct SettingsBackend;
impl ferrostage_pdf::raster::RasterBackend for SettingsBackend {
    fn probe(
        &mut self,
        _: Vec<u8>,
    ) -> Result<ferrostage_pdf::DocumentMetadata, ferrostage_pdf::PdfError> {
        Err(ferrostage_pdf::PdfError::Unreadable)
    }
    fn load(
        &mut self,
        _: ferrostage_core::DocumentGeneration,
        bytes: Vec<u8>,
    ) -> Result<ferrostage_pdf::DocumentMetadata, ferrostage_pdf::PdfError> {
        self.probe(bytes)
    }
    fn render(
        &mut self,
        _: ferrostage_core::DocumentGeneration,
        _: PhysicalPage,
        _: ferrostage_core::NormalizedRect,
        _: ferrostage_pdf::PixelSize,
    ) -> Result<ferrostage_core::BgraFrame, ferrostage_pdf::PdfError> {
        Err(ferrostage_pdf::PdfError::Unreadable)
    }
    fn remove(&mut self, _: ferrostage_core::DocumentGeneration) {}
}
fn settle_preferences(shell: &Entity<Shell>, cx: &mut TestAppContext) {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        shell.update(cx, |shell, cx| shell.poll_preferences(cx));
        if shell.read_with(cx, |shell, _| shell.preference_change.is_none()) {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "worker acknowledgement timed out"
        );
        std::thread::sleep(Duration::from_millis(1));
    }
}
#[gpui::test]
fn settings_v09_worker_ack_serializes_changes_and_failed_display_rolls_budget_back(
    cx: &mut TestAppContext,
) {
    let fixture = SaveFixture::new();
    fixture.unblock();
    let (shell, _) = input_presenter(cx);
    let worker = ferrostage_render::RenderHandle::with_backend(
        || Box::new(SettingsBackend),
        ferrostage_render::CacheBudgets::default(),
    )
    .unwrap();
    shell.update(cx, |shell, cx| {
        shell.store = fixture.store();
        shell.renderer = Ok(worker);
        shell.modal = Some(Modal::Settings);
        let old = shell.settings.settings.cache.l2_cpu_mib;
        shell.dispatch(Intent::EditSetting("cache.l2_cpu_mib", "17".into()), cx);
        assert!(shell.busy());
        assert_eq!(shell.settings.settings.cache.l2_cpu_mib, old);
        shell.dispatch(Intent::EditSetting("cache.l2_cpu_mib", "19".into()), cx);
        shell.dispatch(Intent::EditSetting("general.theme", "dark".into()), cx);
        assert_eq!(shell.settings.settings.general.theme, ferrostage_settings::Theme::Light);
        assert_eq!(shell.settings.settings.cache.l2_cpu_mib, old);
        let layout = shell.settings.settings.presenter.clone();
        shell.dispatch(Intent::Escape, cx);
        assert!(shell.modal.is_none());
        shell.dispatch(Intent::Split { layout: crate::workbench::WorkbenchLayout::Standard, primary: 0.3, secondary: 0.4, save: true }, cx);
        assert_eq!(shell.settings.settings.presenter, layout);
        assert!(!shell.settings.dirty);
    });
    settle_preferences(&shell, cx);
    shell.update(cx, |shell, cx| {
        assert_eq!(shell.settings.settings.cache.l2_cpu_mib, 17);
        assert!(!shell.settings.dirty);
        shell.dispatch(Intent::Split { layout: crate::workbench::WorkbenchLayout::Standard, primary: 0.3, secondary: 0.4, save: true }, cx);
        assert_eq!(shell.settings.settings.presenter.next_slide_ratio, 0.3);
        assert_eq!(shell.settings.settings.presenter.notes_ratio, 0.55);
        assert_eq!(shell.store.load().unwrap().settings.presenter, shell.settings.settings.presenter);
        shell.modal = Some(Modal::Settings);
        let mut proposed = shell.settings.settings.clone();
        proposed.cache.l2_cpu_mib = 23;
        proposed.display.audience_fullscreen = !proposed.display.audience_fullscreen;
        // No monitors: the role transaction cannot prepare; worker must acknowledge rollback.
        shell.monitors.clear();
        shell.apply_preferences(proposed, cx);
    });
    settle_preferences(&shell, cx);
    shell.read_with(cx, |shell, _| {
        assert_eq!(shell.settings.settings.cache.l2_cpu_mib, 17);
        assert!(shell.preferences.error.is_some());
        assert_eq!(shell.modal, Some(Modal::Settings));
        assert_eq!(shell.store.load().unwrap().settings.cache.l2_cpu_mib, 17);
    });
}

#[gpui::test]
fn settings_v09_closed_panel_split_waits_for_apply_and_rollback_ack(cx: &mut TestAppContext) {
    fn drag(handle: AnyWindowHandle, cx: &mut TestAppContext) {
        redraw(handle, cx);
        let mut visual = gpui::VisualTestContext::from_window(handle, cx);
        let split = visual.debug_bounds("horizontal-split").unwrap();
        visual.simulate_mouse_down(split.center(), gpui::MouseButton::Left, gpui::Modifiers::default());
        visual.simulate_mouse_move(split.center() - gpui::point(px(100.), px(0.)), Some(gpui::MouseButton::Left), gpui::Modifiers::default());
        visual.simulate_mouse_up(gpui::point(px(1.), px(1.)), gpui::MouseButton::Left, gpui::Modifiers::default());
    }
    let fixture = SaveFixture::new();
    fixture.unblock();
    let (shell, handle) = workbench_fixture(cx);
    let worker = ferrostage_render::RenderHandle::with_backend(|| Box::new(SettingsBackend), ferrostage_render::CacheBudgets::default()).unwrap();
    shell.update(cx, |shell, cx| {
        shell.store = fixture.store();
        shell.renderer = Ok(worker);
        shell.save(cx);
        shell.modal = Some(Modal::Settings);
        shell.dispatch(Intent::EditSetting("cache.l2_cpu_mib", "17".into()), cx);
    });
    redraw(handle, cx);
    cx.simulate_keystrokes(handle, "escape");
    let layout = shell.read_with(cx, |shell, _| {
        assert!(shell.modal.is_none() && shell.busy());
        shell.settings.settings.presenter.clone()
    });
    let path = fixture.0.join("blocked/settings.toml");
    let saved = std::fs::read(&path).unwrap();
    drag(handle, cx);
    shell.read_with(cx, |shell, _| assert_eq!(shell.settings.settings.presenter, layout));
    assert_eq!(std::fs::read(&path).unwrap(), saved);
    shell.update(cx, |shell, cx| {
        shell.rollback_preferences("test rollback".into(), cx);
        assert!(shell.preference_change.is_some() && shell.busy());
        shell.dispatch(Intent::Split { layout: crate::workbench::WorkbenchLayout::Standard, primary: 0.3, secondary: 0.4, save: true }, cx);
    });
    drag(handle, cx);
    shell.read_with(cx, |shell, _| assert_eq!(shell.settings.settings.presenter, layout));
    assert_eq!(std::fs::read(&path).unwrap(), saved);
    settle_preferences(&shell, cx);
    drag(handle, cx);
    shell.read_with(cx, |shell, _| {
        assert!(!shell.busy() && !shell.settings.dirty);
        assert!(shell.settings.settings.presenter.next_slide_ratio > layout.next_slide_ratio);
        assert_eq!(shell.store.load().unwrap().settings.presenter, shell.settings.settings.presenter);
    });
}

#[gpui::test]
fn settings_v09_shared_display_apply_cancel_timeout_and_topology(cx: &mut TestAppContext) {
    let fixture = SaveFixture::new();
    fixture.unblock();
    let (shell, old) = display_fixture(cx);
    shell.update(cx, |shell, cx| {
        shell.store = fixture.store();
        set_playing(shell, cx, Instant::now() - Duration::from_secs(10));
        shell.modal = Some(Modal::Settings);
        shell.preferences.category = crate::settings_panel::Category::Displays;
        shell.new_display_draft();
        let before = shell.settings.settings.clone();
        let state = shell.state.clone();
        shell.dispatch(Intent::SelectPdf(PdfTarget::Windowed), cx);
        shell.apply_display_draft(cx);
        assert!(shell.preference_change.is_some());
        assert_eq!(shell.settings.settings, before);
        shell.pending.as_mut().unwrap().deadline = Instant::now() - Duration::from_secs(1);
        shell
            .pending
            .as_mut()
            .unwrap()
            .presenter
            .as_mut()
            .unwrap()
            .display = u64::MAX;
        shell.poll_transaction(cx);
        assert!(shell.pending.is_none() && shell.preference_change.is_none());
        assert_eq!(shell.settings.settings, before);
        assert_eq!(shell.state, state);
        assert_eq!(shell.modal, Some(Modal::Settings));
        assert!(shell.preferences.error.is_some());
        assert_eq!(shell.presenter.as_ref().unwrap().handle, old);
        shell.preferences.error = None;
        shell.apply_display_draft(cx);
        shell.poll_transaction(cx);
        assert_eq!(shell.state, state);
        assert_eq!(shell.modal, Some(Modal::Settings));
        assert_eq!(
            shell.settings.settings.display.single_monitor_mode,
            ferrostage_settings::SingleMonitorMode::WindowedAudience
        );
        assert_eq!(
            shell.store.load().unwrap().settings.display,
            shell.settings.settings.display
        );
        add_spare_monitor(shell);
        shell.new_display_draft();
        shell.dispatch(Intent::SelectPdf(PdfTarget::None), cx);
        shell.apply_display_draft(cx);
        shell.poll_transaction(cx);
        assert_eq!(shell.state.stage(), SessionStage::Ready);
        assert_eq!(shell.state.current_page(), PhysicalPage(1));
        assert!(shell.audience.is_none());
        let elapsed = shell.state.elapsed_at(Instant::now());
        assert_eq!(
            shell
                .state
                .elapsed_at(Instant::now() + Duration::from_secs(100)),
            elapsed
        );
        let mut monitor = test_monitor();
        monitor.native_size = (1280, 720);
        shell.topology_changed(TopologyEvent::Changed(vec![monitor]), cx);
        assert_eq!(shell.modal, Some(Modal::Settings));
        assert_ne!(
            shell.display_draft().unwrap().revision,
            shell.topology_revision
        );
    });
}

#[gpui::test]
fn settings_v09_values_reach_workbench_timer_layout_pointer_clock_and_diagnostics(
    cx: &mut TestAppContext,
) {
    let fixture = SaveFixture::new();
    fixture.unblock();
    let (shell, handle) = input_presenter(cx);
    shell.update(cx, |shell, cx| {
        shell.store = fixture.store();
        load_ready(shell);
        shell.modal = Some(Modal::Settings);
        for (id, value) in [
            ("presenter.notes_font_scale", "2"),
            ("presenter.next_slide_ratio", "0.6"),
            ("presenter.notes_ratio", "0.65"),
            ("presenter.show_clock", "false"),
            ("presenter.grid_columns", "4"),
            ("presenter.grid_visible_pages", "13"),
            ("timer.mode", "count_down"),
            ("timer.target_minutes", "3"),
            ("timer.warn_red_remaining_min", "1"),
            ("timer.warn_yellow_remaining_min", "4"),
            ("pointer.pen_color_bgra", "AABBCCDD"),
            ("pointer.pen_width_norm", "0.02"),
            ("pointer.highlighter_alpha", "0.7"),
            ("general.debug_overlay", "true"),
        ] {
            shell.dispatch(Intent::EditSetting(id, value.into()), cx);
            assert!(
                shell.preferences.error.is_none(),
                "{id}: {:?}",
                shell.preferences.error
            );
        }
    });
    cx.run_until_parked();
    redraw(handle, cx);
    let typed = handle.downcast::<Presenter>().unwrap();
    typed
        .read_with(cx, |view, _| {
            let actual = view.preferences_probe();
            assert_eq!(actual.layout.notes_font_scale, 2.);
            assert_eq!(actual.layout.next_slide_ratio, 0.6);
            assert_eq!(actual.layout.notes_ratio, 0.65);
            assert_eq!(
                (actual.layout.grid_columns, actual.layout.grid_visible_pages),
                (4, 13)
            );
            assert!(!actual.clock && !actual.layout.show_thumbnail_strip);
            assert!(actual.debug);
            assert_eq!(actual.pointer.pen_color_bgra, [0xCC, 0xBB, 0xAA, 0xDD]);
            assert_eq!(actual.pointer.pen_width_norm, 0.02);
            assert_eq!(actual.pointer.highlighter_alpha, 0.7);
            assert_eq!(actual.timer, "03:00");
            assert_eq!(actual.warning, 1);
        })
        .unwrap();
    shell.update(cx, |shell, cx| {
        shell.dispatch(
            Intent::EditSetting("timer.warn_red_remaining_min", "3".into()),
            cx,
        )
    });
    cx.run_until_parked();
    typed
        .read_with(cx, |view, _| {
            assert_eq!(view.preferences_probe().warning, 2)
        })
        .unwrap();
}

#[gpui::test]
fn settings_general_help_is_contained_by_its_content_driven_row(cx: &mut TestAppContext) {
    let (shell, handle) = input_presenter(cx);
    shell.update(cx, |shell, cx| {
        load_ready(shell);
        shell.ui_fonts = std::sync::Arc::new(vec!["Consolas".into()]);
        shell.settings.settings.general.ui_font_family = "Consolas".into();
        shell.settings.settings.general.ui_font_size = 18;
        shell.settings.settings.general.recent_files = vec![
            "fixtures/pdf/perf-100.pdf".into(),
            "fixtures/pdf/raster.pdf".into(),
            "fixtures/media/media-smoke.pdf".into(),
        ];
        shell.modal = Some(Modal::Settings);
        cx.notify();
    });
    redraw(handle, cx);
    let mut visual = gpui::VisualTestContext::from_window(handle, cx);
    assert!(visual.debug_bounds("setting-row-general.language").is_some());
    assert!(visual.debug_bounds("setting-help-general.language").is_none());
    {
        let row = visual
            .debug_bounds("setting-row-general.reopen_last_document")
            .unwrap();
        let help = visual
            .debug_bounds("setting-help-general.reopen_last_document")
            .unwrap();
        assert!(
            help.top() >= row.top() && help.bottom() <= row.bottom(),
            "setting-help-general.reopen_last_document must fit inside its row: help={help:?}, row={row:?}"
        );
    }
    let content = visual.debug_bounds("settings-content").unwrap();
    visual.simulate_event(gpui::ScrollWheelEvent {
        position: content.center(),
        delta: gpui::ScrollDelta::Lines(gpui::point(0., -100.)),
        modifiers: gpui::Modifiers::default(),
        touch_phase: gpui::TouchPhase::Moved,
    });
    redraw(handle, cx);
    for (row_id, parent_id) in [
        ("settings-recent-0", "settings-recent-parent-0"),
        ("settings-recent-1", "settings-recent-parent-1"),
        ("settings-recent-2", "settings-recent-parent-2"),
    ] {
        let row = visual.debug_bounds(row_id).unwrap();
        let parent = visual.debug_bounds(parent_id).unwrap();
        assert!(
            parent.size.height >= px(13.5),
            "{parent_id} must have at least its 13.5px font height at base18: {parent:?}"
        );
        assert!(
            parent.top() >= row.top() && parent.bottom() <= row.bottom(),
            "{parent_id} exceeds its row: parent={parent:?}, row={row:?}"
        );
        assert!(parent.top() >= content.top() && parent.bottom() <= content.bottom());
    }
    click_control(handle, "settings-category-displays", cx);
    redraw(handle, cx);
    let content = visual.debug_bounds("settings-content").unwrap();
    visual.simulate_event(gpui::ScrollWheelEvent {
        position: content.center(),
        delta: gpui::ScrollDelta::Lines(gpui::point(0., -100.)),
        modifiers: gpui::Modifiers::default(),
        touch_phase: gpui::TouchPhase::Moved,
    });
    redraw(handle, cx);
    let row = visual
        .debug_bounds("setting-row-display.presenter_start_fullscreen")
        .unwrap();
    let help = visual
        .debug_bounds("setting-help-display.presenter_start_fullscreen")
        .unwrap();
    assert!(
        help.top() >= row.top() && help.bottom() <= row.bottom(),
        "setting-help-display.presenter_start_fullscreen must fit inside its row: help={help:?}, row={row:?}"
    );
}

#[gpui::test]
fn settings_fonts_filter_apply_size_restore_and_history_through_real_input(
    cx: &mut TestAppContext,
) {
    let fixture = SaveFixture::new();
    fixture.unblock();
    let (shell, handle) = input_presenter(cx);
    shell.update(cx, |shell, cx| {
        shell.store = fixture.store();
        load_ready(shell);
        shell.ui_fonts = std::sync::Arc::new(vec!["Arial".into(), "Microsoft YaHei UI".into(), "微软雅黑".into()]);
        shell.settings.settings.general.recent_files =
            vec!["alpha/same.pdf".into(), "beta/same.pdf".into()];
        shell.settings.settings.general.recent_opened_at = BTreeMap::from([
            ("alpha/same.pdf".into(), 123), ("beta/same.pdf".into(), 456),
        ]);
        shell.modal = Some(Modal::Settings);
        cx.notify();
    });
    redraw(handle, cx);
    // Reach the family selector through the actual tab order, not a mouse click.
    let mut reached_family = false;
    for _ in 0..24 {
        cx.simulate_keystrokes(handle, "tab");
        cx.run_until_parked();
        redraw(handle, cx);
        if handle
            .downcast::<Presenter>()
            .unwrap()
            .update(cx, |view, window, _| view.font_picker_focused(window))
            .unwrap()
        {
            reached_family = true;
            break;
        }
    }
    assert!(reached_family, "font family must be reachable by Tab");
    let family = gpui::VisualTestContext::from_window(handle, cx)
        .debug_bounds("setting-font-family")
        .unwrap();
    cx.simulate_keystrokes(handle, "enter");
    cx.run_until_parked();
    redraw(handle, cx);
    // Opening the picker marks its trigger without changing the trigger's width.
    let mut visual = gpui::VisualTestContext::from_window(handle, cx);
    assert!(visual.debug_bounds("setting-font-family-selected").is_some());
    assert_eq!(visual.debug_bounds("setting-font-family").unwrap().size, family.size);
    assert!(visual.debug_bounds("settings-font-search").is_some(), "opening the picker shows the text input");
    handle.update(cx, |_, window, cx| {
        cx.write_to_clipboard(gpui::ClipboardItem::new_string("Arial".into()));
        window.dispatch_keystroke(gpui::Keystroke::parse("ctrl-v").unwrap(), cx);
        // A paste provider can restore its clipboard before deferred Shell dispatch.
        cx.write_to_clipboard(gpui::ClipboardItem::new_string("restored clipboard".into()));
    }).unwrap();
    cx.run_until_parked();
    assert_eq!(
        shell.read_with(cx, |shell, _| shell.preferences.font_picker.as_ref().unwrap().filter.clone()),
        "Arial",
        "paste must capture event-time clipboard text before deferred dispatch"
    );
    cx.write_to_clipboard(gpui::ClipboardItem::new_string("bad\nfilter".into()));
    cx.simulate_keystrokes(handle, "ctrl-v");
    cx.run_until_parked();
    assert_eq!(
        shell.read_with(cx, |shell, _| shell.preferences.font_picker.as_ref().unwrap().filter.clone()),
        "Arial",
        "a single-line font filter must reject pasted control characters"
    );
    cx.simulate_keystrokes(handle, "ctrl-a");
    let mut platform = cx.test_window(handle);
    let mut input = gpui::PlatformWindow::take_input_handler(&mut platform).expect("focused font search input");
    assert_eq!(input.selected_text_range(false).unwrap().range, 0..5);
    input.replace_and_mark_text_in_range(None, "雅黑", Some(2..2));
    assert_eq!(input.marked_text_range(), Some(0..2));
    gpui::PlatformWindow::set_input_handler(&mut platform, input);
    cx.simulate_keystrokes(handle, "enter");
    cx.run_until_parked();
    shell.read_with(cx, |shell, _| {
        assert_eq!(shell.preferences.font_picker.as_ref().unwrap().filter, "雅黑");
        assert!(shell.settings.settings.general.ui_font_family.is_empty());
    });
    let mut input = gpui::PlatformWindow::take_input_handler(&mut platform).unwrap();
    input.replace_text_in_range(None, "雅黑");
    assert_eq!(input.marked_text_range(), None);
    gpui::PlatformWindow::set_input_handler(&mut platform, input);
    cx.simulate_keystrokes(handle, "x");
    redraw(handle, cx);
    assert!(gpui::VisualTestContext::from_window(handle, cx).debug_bounds("settings-font-empty").is_some());
    cx.simulate_keystrokes(handle, "down up enter");
    assert!(shell.read_with(cx, |shell, _| shell.preferences.font_picker.is_some()));
    cx.simulate_keystrokes(handle, "ctrl-a backspace");
    assert!(shell.read_with(cx, |shell, _| shell.preferences.font_picker.as_ref().unwrap().filter.is_empty()));
    cx.simulate_keystrokes(handle, "shift-y a shift-h e i enter");
    cx.run_until_parked();
    shell.read_with(cx, |shell, _| {
        assert_eq!(
            shell.settings.settings.general.ui_font_family,
            "Microsoft YaHei UI"
        );
        assert!(shell.preferences.font_picker.is_none());
        assert_eq!(
            shell.store.load().unwrap().settings.general.ui_font_family,
            "Microsoft YaHei UI"
        );
        assert_eq!(shell.state.stage(), SessionStage::Ready);
        assert_eq!(shell.state.current_page(), PhysicalPage(0));
    });
    redraw(handle, cx);
    click_control(handle, "setting-general.ui_font_size-value", cx);
    cx.run_until_parked();
    redraw(handle, cx);
    cx.simulate_keystrokes(handle, "ctrl-a 2 0 enter");
    cx.run_until_parked();
    shell.read_with(cx, |shell, _| {
        assert_eq!(shell.settings.settings.general.ui_font_size, 20);
        assert_eq!(shell.settings.settings.presenter.notes_font_scale, 1.);
        assert!(shell.audience.is_none());
        assert!(shell.state.audience_snapshot().visual.is_none());
    });
    redraw(handle, cx);
    handle.downcast::<Presenter>().unwrap().read_with(cx, |view, _| {
        assert_eq!(view.typography_probe(), ("Microsoft YaHei UI".into(), 17.5, false));
    }).unwrap();
    click_control(handle, "setting-font-size-default", cx);
    cx.run_until_parked();
    redraw(handle, cx);
    click_control(handle, "setting-font-default", cx);
    cx.run_until_parked();
    shell.read_with(cx, |shell, _| {
        assert_eq!(shell.settings.settings.general.ui_font_size, 16);
        assert!(shell.settings.settings.general.ui_font_family.is_empty());
    });
    redraw(handle, cx);
    click_control(handle, "setting-font-family", cx);
    cx.run_until_parked();
    redraw(handle, cx);
    cx.simulate_keystrokes(handle, "a escape");
    cx.run_until_parked();
    redraw(handle, cx);
    shell.read_with(cx, |shell, _| {
        assert!(shell.preferences.font_picker.is_none());
        assert_eq!(shell.modal, Some(Modal::Settings));
        assert_eq!(
            shell.settings.settings.general.recent_files,
            ["alpha/same.pdf", "beta/same.pdf"]
        );
    });
    assert!(handle.downcast::<Presenter>().unwrap().update(cx, |view, window, _| view.font_picker_focused(window)).unwrap(), "Escape restores the trigger");
    // The existing history action still clears the source seen by both settings and home.
    shell.update(cx, |shell, cx| {
        shell.dispatch(Intent::ClearRecent, cx);
        assert!(shell.settings.settings.general.recent_files.is_empty());
        assert!(shell.settings.settings.general.recent_opened_at.is_empty());
        assert!(shell.store.load().unwrap().settings.general.recent_opened_at.is_empty());
        assert!(
            shell
                .store
                .load()
                .unwrap()
                .settings
                .general
                .recent_files
                .is_empty()
        );
    });
}

#[gpui::test]
fn settings_fonts_scroll_click_and_tab_follow_the_displayed_order(cx: &mut TestAppContext) {
    let fixture = SaveFixture::new();
    fixture.unblock();
    let (shell, handle) = input_presenter(cx);
    let last = "Family Eight With A Long Display Name";
    shell.update(cx, |shell, cx| {
        shell.store = fixture.store();
        let mut families = (0..8).map(|i| format!("Family {i}")).collect::<Vec<_>>();
        families.push(last.into());
        shell.ui_fonts = std::sync::Arc::new(families);
        shell.modal = Some(Modal::Settings);
        cx.notify();
    });
    redraw(handle, cx);
    click_control(handle, "setting-font-family", cx);
    cx.run_until_parked();
    redraw(handle, cx);
    cx.simulate_keystrokes(handle, "pagedown down down");
    redraw(handle, cx);
    let mut visual = gpui::VisualTestContext::from_window(handle, cx);
    let viewport = visual.debug_bounds("settings-font-results").unwrap();
    let last_row = visual.debug_bounds("font-choice-Family Eight With A Long Display Name").unwrap();
    assert!(last_row.top() >= viewport.top() && last_row.bottom() <= viewport.bottom(),
        "keyboard candidate must scroll into view: {last_row:?} in {viewport:?}");
    assert!(shell.read_with(cx, |s, _| s.store.load().unwrap().settings.general.ui_font_family.is_empty()));
    // Choose the preceding visible row with the mouse while the last one is highlighted.
    click_control(handle, "font-choice-Family 7", cx);
    cx.run_until_parked();
    shell.read_with(cx, |s, _| {
        assert_eq!(s.store.load().unwrap().settings.general.ui_font_family, "Family 7");
        assert!(s.preferences.font_picker.is_none());
    });
    redraw(handle, cx);
    click_control(handle, "setting-font-family", cx);
    cx.run_until_parked();
    redraw(handle, cx);
    assert_eq!(shell.read_with(cx, |s, _| s.preferences.font_picker.as_ref().unwrap().selected), 7);
    cx.simulate_keystrokes(handle, "shift-tab enter");
    cx.run_until_parked();
    shell.read_with(cx, |s, _| {
        assert!(s.preferences.font_picker.is_none());
        // Shift+Tab left the search on the preceding explicit Use default control.
        assert!(s.settings.settings.general.ui_font_family.is_empty());
    });
    redraw(handle, cx);
    click_control(handle, "setting-font-family", cx);
    cx.run_until_parked();
    redraw(handle, cx);
    cx.simulate_keystrokes(handle, "tab enter");
    cx.run_until_parked();
    shell.read_with(cx, |s, _| {
        // Tab left the search on the next setting, the interface-size minus button.
        assert!(s.preferences.font_picker.is_none());
        assert_eq!(s.settings.settings.general.ui_font_size, 15);
    });
}

#[gpui::test]
fn start_mode_menu_selects_single_persists_and_hides_while_presenting(cx: &mut TestAppContext) {
    let fixture = SaveFixture::new();
    fixture.unblock();
    let (shell, handle) = input_presenter(cx);
    shell.update(cx, |shell, cx| {
        load_ready(shell);
        shell.store = fixture.store();
        assert_eq!(shell.settings.settings.display.start_mode, ferrostage_settings::StartMode::Dual);
        cx.notify();
    });
    redraw(handle, cx);
    click_control(handle, "start-mode", cx);
    redraw(handle, cx);
    {
        let mut visual = gpui::VisualTestContext::from_window(handle, cx);
        assert!(visual.debug_bounds("start-mode-menu").is_some());
        assert!(visual.debug_bounds("start-mode-check-0").is_some());
        assert!(visual.debug_bounds("start-mode-check-1").is_none());
        assert!(visual.debug_bounds("start-mode-row-2").is_some());
    }
    click_control(handle, "start-mode-row-2", cx);
    cx.run_until_parked();
    shell.update(cx, |shell, cx| {
        assert!(shell.contextual.is_none());
        assert_eq!(shell.settings.settings.display.start_mode, ferrostage_settings::StartMode::Single);
        assert!(!shell.settings.dirty);
        assert_eq!(shell.store.load().unwrap().settings.display.start_mode, ferrostage_settings::StartMode::Single);
        shell
            .state
            .apply_at(Command::StartPresentation { audience_available: true }, Instant::now())
            .unwrap();
        shell.dispatch(Intent::StartModeMenu, cx);
        assert!(shell.contextual.is_none());
        shell.dispatch(Intent::SetStartMode(ferrostage_settings::StartMode::Dual), cx);
        assert_eq!(shell.settings.settings.display.start_mode, ferrostage_settings::StartMode::Single);
        cx.notify();
    });
    redraw(handle, cx);
    let mut visual = gpui::VisualTestContext::from_window(handle, cx);
    assert!(visual.debug_bounds("start-mode").is_none());
}

#[gpui::test]
fn about_tab_shows_version_author_build_time_license_and_repository(cx: &mut TestAppContext) {
    let (shell, handle) = input_presenter(cx);
    shell.update(cx, |shell, cx| {
        load_ready(shell);
        shell.modal = Some(Modal::Settings);
        shell.preferences.category = crate::settings_panel::Category::About;
        cx.notify();
    });
    redraw(handle, cx);
    {
        let mut visual = gpui::VisualTestContext::from_window(handle, cx);
        for key in [
            "about-version",
            "about-author",
            "about-build-time",
            "about-license",
            "about-repository",
        ] {
            assert!(visual.debug_bounds(key).is_some(), "{key} row is missing");
        }
        assert!(visual.debug_bounds("about-commit").is_none());
        assert!(visual.debug_bounds("about-changelog").is_some());
        assert!(visual.debug_bounds("changelog-1.4.0").is_none());
        assert!(visual.debug_bounds("changelog-1.3.1").is_none());
    }
    click_control(handle, "about-changelog", cx);
    cx.run_until_parked();
    redraw(handle, cx);
    {
        let mut visual = gpui::VisualTestContext::from_window(handle, cx);
        let mut previous_top = None;
        for id in [
            "changelog-1.4.0",
            "changelog-1.3.1",
            "changelog-1.3.0",
            "changelog-1.2.1",
            "changelog-1.2.0",
            "changelog-1.1.1",
            "changelog-1.1.0",
            "changelog-1.0.0",
        ] {
            let bounds = visual.debug_bounds(id).unwrap_or_else(|| panic!("{id}"));
            if let Some(prev) = previous_top {
                assert!(bounds.top() > prev, "{id} should appear below previous entry");
            }
            previous_top = Some(bounds.top());
        }
    }
    assert!(cx.opened_url().is_none());
    click_control(handle, "about-changelog", cx);
    cx.run_until_parked();
    redraw(handle, cx);
    {
        let mut visual = gpui::VisualTestContext::from_window(handle, cx);
        assert!(visual.debug_bounds("changelog-1.4.0").is_none());
        assert!(visual.debug_bounds("changelog-1.3.1").is_none());
    }
    click_control(handle, "about-changelog", cx);
    cx.run_until_parked();
    click_control(handle, "settings-category-general", cx);
    cx.run_until_parked();
    redraw(handle, cx);
    {
        let mut visual = gpui::VisualTestContext::from_window(handle, cx);
        assert!(visual.debug_bounds("changelog-1.4.0").is_none());
        assert!(visual.debug_bounds("changelog-1.3.1").is_none());
        assert!(visual.debug_bounds("about-changelog").is_none());
    }
    shell.read_with(cx, |shell, _| assert!(!shell.preferences.changelog_open));
    click_control(handle, "settings-category-about", cx);
    cx.run_until_parked();
    redraw(handle, cx);
    assert_eq!(crate::about::VERSION, "1.4.0");
    assert_eq!(crate::about::LICENSE, "MIT");
    assert_eq!(crate::about::CHANGELOG[0].0, crate::about::VERSION);
    assert_eq!(crate::about::CHANGELOG[0].1, "about-changelog-1-4-0");
    for (language, v140, v131, v130, v100) in [
        (
            "zh-CN",
            "校准浅色与深色暖橙毛玻璃主题；讲者窗口改用与当前明暗主题一致的一体化透明标题栏；Windows 10 自动回退实色表面以消除窗口拖拽粘滞。",
            "软件许可证改为 MIT。",
            "讲者界面改为可关闭的暖橙毛玻璃。翻页笔方向键可以翻页，旧的默认绑定会自动升级。放映前可以预览并确认备注裁切。没有备注的文稿使用稳定的双预览布局。讲者可以看到真实绘制进度和观众窗口位置。答疑跳转后可以精确返回原来的讲述位置。PDF 首张预览和 Beamer 翻页更快。",
            "首次发布：双窗口 PDF / Beamer 演讲、演讲者备注、批注，以及外部 PDF 热重载。",
        ),
        (
            "en-US",
            "Calibrated the light and dark warm-orange frosted glass themes. The presenter window now uses an integrated transparent title bar that matches the active light or dark theme. Windows 10 automatically falls back to solid surfaces so dragging the window stays smooth.",
            "The software license is MIT.",
            "The presenter uses warm-orange frosted glass, which can be turned off. Clicker arrow keys turn pages, and old default bindings upgrade automatically. Before a talk, notes cropping can be previewed and confirmed. Decks without notes use a stable dual-preview layout. The presenter shows real paint progress and the audience window position. After a Q&A jump, you can return to the exact previous speaking position. The first PDF preview and Beamer page turns are faster.",
            "First release: dual-window PDF and Beamer presentation, speaker notes, annotations, and external PDF hot reload.",
        ),
    ] {
        let locale = crate::locale::Locale::new(language);
        assert_eq!(locale.text("about-changelog-1-4-0"), v140);
        assert_eq!(locale.text("about-changelog-1-3-1"), v131);
        assert_eq!(locale.text("about-changelog-1-3-0"), v130);
        assert_eq!(locale.text("about-changelog-1-0-0"), v100);
    }
    assert!(!crate::about::AUTHORS.is_empty());
    assert_eq!(crate::about::REPOSITORY, "https://github.com/bahayonghang/ferrostage");
    let zone = jiff::tz::TimeZone::get("America/New_York").unwrap();
    for built_at in [0, -1, i64::MAX] {
        assert!(crate::about::BuildInfo { built_at }.local_time(zone.clone()).is_none());
    }
    let time = crate::about::BuildInfo { built_at: 1_792_467_301 }.local_time(zone).unwrap();
    for (language, expected) in [("zh-CN", "2026年10月19日 23:35"), ("en-US", "2026-10-19 23:35")] {
        let format = crate::locale::Locale::new(language).text("about-build-time-format");
        assert_eq!(time.strftime(format).to_string(), expected);
    }
    assert_eq!(time.strftime("UTC%:z").to_string(), "UTC-04:00");
    click_control(handle, "about-repository-link", cx);
    assert_eq!(cx.opened_url().as_deref(), Some(crate::about::REPOSITORY));
}

#[gpui::test]
fn event_error_flag_closes_and_persistent_monitor_error_has_no_close_button(
    cx: &mut TestAppContext,
) {
    let (shell, handle) = input_presenter(cx);
    let timeout = shell.update(cx, |shell, cx| {
        load_ready(shell);
        let timeout = ShellError::PlacementTimeout.localized(shell.locale);
        shell.error = Some(timeout.clone());
        cx.notify();
        timeout
    });
    redraw(handle, cx);
    let presented = |cx: &mut TestAppContext| {
        handle
            .update(cx, |root, _, cx| {
                root.downcast::<Presenter>()
                    .unwrap()
                    .read(cx)
                    .presented_error()
                    .map(str::to_owned)
            })
            .unwrap()
    };
    assert_eq!(presented(cx), Some(timeout));
    click_control(handle, "dismiss-error", cx);
    redraw(handle, cx);
    assert!(shell.read_with(cx, |shell, _| shell.error.is_none() && shell.graphics_error.is_none()));
    // The headless shell has no topology watcher. That persistent condition
    // stays visible and has no close button.
    let monitor = shell.read_with(cx, |shell, _| shell.monitor_error_text());
    assert!(monitor.is_some());
    assert_eq!(presented(cx), monitor);
    let mut visual = gpui::VisualTestContext::from_window(handle, cx);
    assert!(visual.debug_bounds("presenter-error").is_some());
    assert!(visual.debug_bounds("dismiss-error").is_none());
}

fn open_shell_from_store(
    store: SettingsStore,
    cx: &mut TestAppContext,
) -> (Entity<Shell>, AnyWindowHandle) {
    let settings = SettingsSession::from_outcome(store.load().unwrap());
    let (_, topology) = std::sync::mpsc::channel();
    let shell = cx.new(|cx| {
        Shell::new(
            ShellInputs {
                settings,
                store,
                renderer: Err(RenderError::FeatureDisabled),
                topology,
                watcher: Err(ferrostage_platform::PlatformError::UnsupportedPlatform),
                initial_monitors: Ok(Vec::new()),
                initial_pdf: None,
            },
            cx,
        )
    });
    cx.update(|cx| install_close_handler(&shell, cx));
    let warnings = cx.update(|cx| {
        let model = shell.read(cx);
        let keys = model.settings.settings.keybindings.clone();
        let locale = model.locale;
        actions::install(shell.downgrade(), &keys, locale, cx)
    });
    let handle = shell.update(cx, |shell, cx| {
        let id = MonitorId("headless-test-display".into());
        let display = u64::from(cx.displays().first().unwrap().id());
        shell.mapped = Ok([(id, display)].into());
        shell.shortcut_warnings = warnings;
        shell.error = None;
        shell.presenter = Some(test_window(shell, false, cx));
        cx.notify();
        shell.presenter.as_ref().unwrap().handle
    });
    (shell, handle)
}

#[gpui::test]
fn language_hot_switch_updates_settings_home_titlebar_and_window_title(cx: &mut TestAppContext) {
    let fixture = SaveFixture::new();
    fixture.unblock();
    let (shell, handle) = input_presenter(cx);
    shell.update(cx, |shell, cx| {
        shell.store = fixture.store();
        shell.monitors = vec![test_monitor()];
        shell.settings.settings.general.recent_files = vec![r"C:\slides\talk.pdf".into()];
        cx.notify();
    });
    redraw(handle, cx);
    let presenter = handle.downcast::<Presenter>().unwrap();
    let mut visual = gpui::VisualTestContext::from_window(handle, cx);
    assert_eq!(
        presenter.read_with(cx, |view, _| view.recent_probe()[0].1.to_owned()).unwrap(),
        "暂无记录"
    );
    assert_eq!(
        shell.read_with(cx, |shell, _| shell.configured_projection_target()),
        "屏幕 1 Headless display · 同屏全屏"
    );
    assert_eq!(visual.window_title(), Some("FerroStage".into()));

    click_control(handle, "settings", cx);
    shell.update(cx, |shell, cx| shell.poll_focus(cx));
    cx.run_until_parked();
    redraw(handle, cx);
    assert!(visual.debug_bounds("setting-help-general.language").is_none());

    click_control(handle, "setting-general.language-en-US", cx);
    cx.run_until_parked();
    redraw(handle, cx);
    shell.read_with(cx, |shell, _| {
        assert_eq!(shell.settings.settings.general.language, "en-US");
        assert_eq!(shell.locale.text("settings"), "Settings");
        assert_eq!(
            shell.locale.text("settings-font-preview"),
            "Presentation ready · 演示就绪 · 123 Aa"
        );
        assert_eq!(shell.store.load().unwrap().settings.general.language, "en-US");
    });

    click_control(handle, "close-settings", cx);
    cx.run_until_parked();
    redraw(handle, cx);
    assert_eq!(
        presenter.read_with(cx, |view, _| view.recent_probe()[0].1.to_owned()).unwrap(),
        "Not recorded"
    );
    assert_eq!(
        shell.read_with(cx, |shell, _| shell.configured_projection_target()),
        "Display 1 Headless display · Same-screen fullscreen"
    );

    shell.update(cx, |shell, cx| {
        load_ready(shell);
        shell.path = Some(PathBuf::from("slides/talk.pdf"));
        cx.notify();
    });
    redraw(handle, cx);
    assert_eq!(
        presenter.read_with(cx, |view, _| view.titlebar_probe().unwrap().title).unwrap(),
        "talk.pdf — Ready — FerroStage"
    );
    assert_eq!(
        visual.window_title(),
        Some("talk.pdf — Ready — FerroStage".into())
    );
    assert_eq!(
        shell.read_with(cx, |shell, _| shell.projection_status().unwrap().summary),
        "Not presenting"
    );

    click_control(handle, "settings", cx);
    shell.update(cx, |shell, cx| shell.poll_focus(cx));
    cx.run_until_parked();
    redraw(handle, cx);
    click_control(handle, "setting-general.language-zh-CN", cx);
    cx.run_until_parked();
    redraw(handle, cx);
    assert_eq!(
        presenter.read_with(cx, |view, _| view.titlebar_probe().unwrap().title).unwrap(),
        "talk.pdf — 准备就绪 — FerroStage"
    );
    assert_eq!(
        visual.window_title(),
        Some("talk.pdf — 准备就绪 — FerroStage".into())
    );
    assert_eq!(
        shell.read_with(cx, |shell, _| shell.projection_status().unwrap().summary),
        "尚未放映"
    );
}

#[gpui::test]
fn language_hot_switch_during_presentation_updates_workbench_and_preserves_session(
    cx: &mut TestAppContext,
) {
    let fixture = SaveFixture::new();
    fixture.unblock();
    let (shell, presenter) = workbench_fixture(cx);
    let audience = shell.update(cx, |shell, cx| {
        shell.store = fixture.store();
        shell.monitors = vec![test_monitor()];
        shell.path = Some(PathBuf::from("slides/demo.pdf"));
        let started = Instant::now() - Duration::from_secs(15);
        shell
            .state
            .apply_at(
                Command::StartPresentation {
                    audience_available: true,
                },
                started,
            )
            .unwrap();
        shell.apply(Command::JumpTo(PhysicalPage(2)), cx);
        shell.record_return_position(true, shell.state.generation(), PhysicalPage(0));
        shell.apply(
            Command::BeginStroke {
                color: [0x11, 0x22, 0x33, 0xFF],
                width: 0.01,
                is_highlighter: false,
            },
            cx,
        );
        shell.apply(Command::AppendStrokePoint((0.25, 0.35)), cx);
        shell.apply(Command::EndStroke, cx);
        let audience = test_window(shell, true, cx);
        let handle = audience.handle;
        shell.audience = Some(audience);
        shell.publish(false, cx);
        handle
    });
    redraw(presenter, cx);
    redraw(audience, cx);

    let before = shell.read_with(cx, |shell, cx| {
        (
            shell.state.stage(),
            shell.state.current_page(),
            shell.state.has_annotations(),
            shell.state.elapsed_at(Instant::now()),
            shell.snapshot.read(cx).clone(),
            shell.return_label(),
        )
    });
    assert_eq!(before.0, SessionStage::Presenting);
    assert_eq!(before.1, PhysicalPage(2));
    assert!(before.2);
    assert!(before.5.as_deref().is_some_and(|label| label.contains("返回")));
    assert_eq!(
        shell.read_with(cx, |shell, _| shell.projection_status().unwrap().summary),
        "等待首幅完整画面 · 待切换：第 2/3 页"
    );

    let mut visual = gpui::VisualTestContext::from_window(presenter, cx);
    let presenter_entity = presenter.downcast::<Presenter>().unwrap();
    assert_eq!(
        presenter_entity
            .read_with(cx, |view, _| view.titlebar_probe().unwrap().title)
            .unwrap(),
        "demo.pdf — 演示中 — FerroStage"
    );

    shell.update(cx, |shell, cx| {
        shell.modal = Some(Modal::Settings);
        cx.notify();
    });
    redraw(presenter, cx);
    click_control(presenter, "setting-general.language-en-US", cx);
    cx.run_until_parked();
    redraw(presenter, cx);

    assert_eq!(
        presenter_entity
            .read_with(cx, |view, _| view.titlebar_probe().unwrap().title)
            .unwrap(),
        "demo.pdf — Presenting — FerroStage"
    );
    assert_eq!(
        visual.window_title(),
        Some("demo.pdf — Presenting — FerroStage".into())
    );
    click_control(presenter, "close-settings", cx);
    cx.run_until_parked();
    redraw(presenter, cx);

    shell.read_with(cx, |shell, cx| {
        assert_eq!(shell.state.stage(), SessionStage::Presenting);
        assert_eq!(shell.state.current_page(), before.1);
        assert_eq!(shell.state.has_annotations(), before.2);
        assert!(shell.state.elapsed_at(Instant::now()) >= before.3);
        assert_eq!(shell.presenter.as_ref().unwrap().handle, presenter);
        assert_eq!(shell.audience.as_ref().unwrap().handle, audience);
        let snapshot = shell.snapshot.read(cx);
        assert_eq!(snapshot.stage, AudienceStage::Live);
        assert_eq!(snapshot.current_page, before.4.current_page);
        assert_eq!(snapshot.strokes, before.4.strokes);
        assert_eq!(
            shell.return_label().as_deref(),
            Some("Return to Slide 1/3")
        );
        assert_eq!(
            shell.projection_status().unwrap().summary,
            "Awaiting first complete draw · Pending: Slide 2/3"
        );
    });

    shell.update(cx, |shell, cx| {
        shell.stop(cx);
        shell.open_browser();
        cx.notify();
    });
    redraw(presenter, cx);
    assert!(visual.debug_bounds("page-browser").is_some());
    assert_eq!(
        shell.read_with(cx, |shell, _| shell.projection_status().unwrap().summary),
        "Not presenting"
    );
    shell.update(cx, |shell, cx| {
        shell.close_contextual();
        shell.new_display_draft();
        cx.notify();
    });
    redraw(presenter, cx);
    assert!(visual.debug_bounds("display-panel-content").is_some());
    shell.update(cx, |shell, cx| {
        shell.modal = None;
        shell.open_crop(None);
        cx.notify();
    });
    redraw(presenter, cx);
    assert!(visual.debug_bounds("crop-panel").is_some());
}

#[gpui::test]
fn reset_preferences_hot_switches_english_to_zh_cn_and_relaunch_uses_saved_language(
    cx: &mut TestAppContext,
) {
    let fixture = SaveFixture::new();
    fixture.unblock();
    let (shell, handle) = input_presenter(cx);
    shell.update(cx, |shell, cx| {
        shell.store = fixture.store();
        shell.modal = Some(Modal::Settings);
        cx.notify();
    });
    redraw(handle, cx);
    click_control(handle, "setting-general.language-en-US", cx);
    cx.run_until_parked();
    redraw(handle, cx);
    assert_eq!(
        shell.read_with(cx, |shell, _| shell.locale.text("settings")),
        "Settings"
    );

    let (relaunched, relaunched_handle) = open_shell_from_store(fixture.store(), cx);
    shell.update(cx, |shell, _| shell.stop_tasks("end first session"));
    let _ = handle.update(cx, |_, window, _| window.remove_window());
    cx.run_until_parked();

    relaunched.update(cx, |shell, cx| {
        load_ready(shell);
        shell.path = Some(PathBuf::from("slides/relaunch.pdf"));
        shell.modal = Some(Modal::Settings);
        cx.notify();
    });
    redraw(relaunched_handle, cx);
    let mut visual = gpui::VisualTestContext::from_window(relaunched_handle, cx);
    assert_eq!(
        relaunched.read_with(cx, |shell, _| shell.locale.text("settings")),
        "Settings"
    );
    assert_eq!(
        visual.window_title(),
        Some("relaunch.pdf — Ready — FerroStage".into())
    );

    click_control(relaunched_handle, "settings-category-advanced", cx);
    redraw(relaunched_handle, cx);
    let content = visual.debug_bounds("settings-content").unwrap();
    visual.simulate_event(gpui::ScrollWheelEvent {
        position: content.center(),
        delta: gpui::ScrollDelta::Lines(gpui::point(0., -100.)),
        modifiers: gpui::Modifiers::default(),
        touch_phase: gpui::TouchPhase::Moved,
    });
    redraw(relaunched_handle, cx);
    click_control(relaunched_handle, "settings-reset-all", cx);
    cx.run_until_parked();
    assert!(relaunched.read_with(cx, |shell, _| shell.preferences.confirm_reset));
    redraw(relaunched_handle, cx);
    click_control(relaunched_handle, "settings-reset-all", cx);
    cx.run_until_parked();
    redraw(relaunched_handle, cx);

    relaunched.read_with(cx, |shell, _| {
        assert_eq!(shell.settings.settings.general.language, "zh-CN");
        assert_eq!(shell.locale.text("settings"), "设置");
        assert_eq!(shell.store.load().unwrap().settings.general.language, "zh-CN");
    });
    assert_eq!(
        visual.window_title(),
        Some("relaunch.pdf — 准备就绪 — FerroStage".into())
    );
}

#[gpui::test]
fn readonly_policy_disables_language_switch_and_preserves_config(cx: &mut TestAppContext) {
    let fixture = SaveFixture::new();
    fixture.unblock();
    let config_path = fixture.store().path().to_path_buf();
    std::fs::create_dir_all(config_path.parent().unwrap()).unwrap();
    std::fs::write(
        &config_path,
        "schema_version = 999\n[general]\nlanguage = \"zh-CN\"\n",
    )
    .unwrap();
    let bytes = std::fs::read(fixture.store().path()).unwrap();
    let (shell, handle) = open_shell_from_store(fixture.store(), cx);
    shell.update(cx, |shell, cx| {
        shell.modal = Some(Modal::Settings);
        cx.notify();
    });
    redraw(handle, cx);
    click_control(handle, "setting-general.language-en-US", cx);
    cx.run_until_parked();
    redraw(handle, cx);
    shell.read_with(cx, |shell, _| {
        assert_eq!(shell.settings.settings.general.language, "zh-CN");
        assert_eq!(shell.locale.text("settings"), "设置");
    });
    assert_eq!(std::fs::read(fixture.store().path()).unwrap(), bytes);
}

#[gpui::test]
fn shortcut_warnings_recompute_on_language_switch_and_preserve_escape_stops(
    cx: &mut TestAppContext,
) {
    let fixture = SaveFixture::new();
    fixture.unblock();
    let config_path = fixture.store().path().to_path_buf();
    std::fs::create_dir_all(config_path.parent().unwrap()).unwrap();
    std::fs::write(
        &config_path,
        "schema_version = 1\n[general]\nlanguage = \"zh-CN\"\n\n[keybindings]\nnext = [\"bad-key\"]\nprevious = [\"ctrl-o\"]\nstop_presentation = [\"escape\"]\n",
    )
    .unwrap();
    let (shell, handle) = open_shell_from_store(fixture.store(), cx);
    shell.update(cx, |shell, cx| {
        shell.warnings.push("startup-warning".into());
        cx.notify();
    });
    redraw(handle, cx);
    let mut visual = gpui::VisualTestContext::from_window(handle, cx);
    shell.read_with(cx, |shell, _| {
        assert_eq!(shell.warnings, vec!["startup-warning"]);
        assert_eq!(shell.shortcut_warnings.len(), 2);
        assert!(shell.shortcut_warnings.iter().all(|w| w.contains("已禁用")));
        assert!(shell.escape_stops);
    });
    assert!(visual.debug_bounds("shell-warning-0").is_some());
    assert!(visual.debug_bounds("shell-warning-1").is_some());
    assert!(visual.debug_bounds("shell-warning-2").is_some());
    assert!(visual.debug_bounds("shell-warning-3").is_none());

    shell.update(cx, |shell, cx| {
        shell.modal = Some(Modal::Settings);
        cx.notify();
    });
    redraw(handle, cx);
    click_control(handle, "setting-general.language-en-US", cx);
    cx.run_until_parked();
    click_control(handle, "close-settings", cx);
    cx.run_until_parked();
    redraw(handle, cx);

    shell.read_with(cx, |shell, _| {
        assert_eq!(shell.warnings, vec!["startup-warning"]);
        assert_eq!(shell.shortcut_warnings.len(), 2);
        assert!(
            shell
                .shortcut_warnings
                .iter()
                .any(|w| w.contains("Disabled invalid shortcut")),
            "{:?}",
            shell.shortcut_warnings
        );
        assert!(
            shell
                .shortcut_warnings
                .iter()
                .any(|w| w.contains("Disabled conflicting shortcut")),
            "{:?}",
            shell.shortcut_warnings
        );
        assert!(
            shell.shortcut_warnings.iter().all(|w| !w.contains("已禁用")),
            "stale zh-CN shortcut warning remained: {:?}",
            shell.shortcut_warnings
        );
        assert!(shell.escape_stops);
    });
    assert!(visual.debug_bounds("shell-warning-0").is_some());
    assert!(visual.debug_bounds("shell-warning-1").is_some());
    assert!(visual.debug_bounds("shell-warning-2").is_some());
    assert!(visual.debug_bounds("shell-warning-3").is_none());

    // Also verify escape_stops == false stays false across language switch.
    shell.update(cx, |shell, cx| {
        let mut next = shell.settings.settings.clone();
        next.keybindings.stop_presentation.clear();
        shell.apply_preferences(next, cx);
        assert!(!shell.escape_stops);
        shell.dispatch(Intent::EditSetting("general.language", "zh-CN".into()), cx);
        assert!(!shell.escape_stops);
        assert!(shell.shortcut_warnings.iter().all(|w| w.contains("已禁用")));
    });
}
