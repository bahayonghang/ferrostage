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
    click_control(handle, "setting-general.theme-light", cx);
    cx.run_until_parked();
    shell.read_with(cx, |shell, _| {
        assert_eq!(
            shell.settings.settings.general.theme,
            ferrostage_settings::Theme::Light
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
            crate::locale::Locale::new("zh-CN").text("settings"),
            "language keeps current catalog until restart"
        );
        assert_eq!(
            crate::locale::Locale::new(&shell.store.load().unwrap().settings.general.language)
                .text("settings"),
            "Settings"
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
        shell.dispatch(Intent::EditSetting("general.theme", "dark".into()), cx);
        shell.dispatch(Intent::ResetPreferences(true), cx);
        shell.dispatch(Intent::ClearRecent, cx);
        assert_eq!(shell.settings.settings, before);
    });
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
        shell.dispatch(Intent::EditSetting("general.theme", "light".into()), cx);
        assert_eq!(
            shell.settings.settings.general.theme,
            ferrostage_settings::Theme::Light
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
        shell.dispatch(Intent::EditSetting("general.theme", "light".into()), cx);
        assert_eq!(shell.settings.settings.cache.l2_cpu_mib, old);
        let layout = shell.settings.settings.presenter.clone();
        shell.dispatch(Intent::Escape, cx);
        assert!(shell.modal.is_none());
        shell.dispatch(Intent::Split { next: 0.3, notes: 0.4, save: true }, cx);
        assert_eq!(shell.settings.settings.presenter, layout);
        assert!(!shell.settings.dirty);
    });
    settle_preferences(&shell, cx);
    shell.update(cx, |shell, cx| {
        assert_eq!(shell.settings.settings.cache.l2_cpu_mib, 17);
        assert!(!shell.settings.dirty);
        shell.dispatch(Intent::Split { next: 0.3, notes: 0.4, save: true }, cx);
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
        shell.dispatch(Intent::Split { next: 0.3, notes: 0.4, save: true }, cx);
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
    for (row_id, help_id) in [
        ("setting-row-general.language", "setting-help-general.language"),
        (
            "setting-row-general.reopen_last_document",
            "setting-help-general.reopen_last_document",
        ),
    ] {
        let row = visual.debug_bounds(row_id).unwrap();
        let help = visual.debug_bounds(help_id).unwrap();
        assert!(
            help.top() >= row.top() && help.bottom() <= row.bottom(),
            "{help_id} must fit inside its row: help={help:?}, row={row:?}"
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
        shell.ui_fonts = std::sync::Arc::new(vec!["Arial".into(), "Microsoft YaHei UI".into()]);
        shell.settings.settings.general.recent_files =
            vec!["alpha/same.pdf".into(), "beta/same.pdf".into()];
        shell.settings.settings.general.recent_opened_at = BTreeMap::from([
            ("alpha/same.pdf".into(), 123), ("beta/same.pdf".into(), 456),
        ]);
        shell.ui_fonts_loaded = true;
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
    cx.simulate_keystrokes(handle, "enter");
    cx.run_until_parked();
    redraw(handle, cx);
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
    cx.simulate_keystrokes(handle, "ctrl-a ctrl-c alt-x ctrl-z");
    cx.run_until_parked();
    assert!(shell.read_with(cx, |shell, _| shell.preferences.font_picker.as_ref().unwrap().filter.is_empty()));
    cx.simulate_keystrokes(handle, "y a h e i down enter");
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
    }
    click_control(handle, "start-mode-row-1", cx);
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
    }
    assert_eq!(crate::about::VERSION, "1.2.0");
    assert!(!crate::about::AUTHORS.is_empty());
    assert_eq!(crate::about::REPOSITORY, "https://github.com/bahayonghang/ferrostage");
    let time = chrono::DateTime::from_timestamp(1_792_467_301, 0)
        .unwrap()
        .with_timezone(&chrono::FixedOffset::west_opt(4 * 3600).unwrap());
    for (language, expected) in [("zh-CN", "2026年10月19日 23:35"), ("en-US", "2026-10-19 23:35")] {
        let format = crate::locale::Locale::new(language).text("about-build-time-format");
        assert_eq!(time.format(format).to_string(), expected);
    }
    assert_eq!(time.format("UTC%:z").to_string(), "UTC-04:00");
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
