use crate::presenter::TimerPhase;

/// Ready deck whose Start can create a real (Single-mode) Start transaction.
fn startable_fixture(cx: &mut TestAppContext) -> (Entity<Shell>, AnyWindowHandle) {
    let (shell, handle) = workbench_fixture(cx);
    shell.update(cx, |shell, cx| {
        shell.test_topology = true;
        shell.monitors = vec![test_monitor()];
        shell.settings.settings.display.start_mode = ferrostage_settings::StartMode::Single;
        shell.settings.settings.timer.mode = ferrostage_settings::TimerMode::CountUp;
        assert_eq!(shell.start_reason(), None);
        cx.notify();
    });
    cx.test_window(handle)
        .simulate_resize(size(px(1600.), px(900.)));
    redraw(handle, cx);
    (shell, handle)
}

/// Run and stop a session that lasted `seconds`, leaving it resumable.
fn interrupt(shell: &Entity<Shell>, seconds: u64, cx: &mut TestAppContext) {
    shell.update(cx, |shell, cx| {
        let now = Instant::now();
        shell
            .state
            .apply_at(
                Command::StartPresentation {
                    audience_available: true,
                },
                now - Duration::from_secs(seconds),
            )
            .unwrap();
        shell.state.apply_at(Command::StopPresentation, now).unwrap();
        cx.notify();
    });
}

fn capsule(
    handle: AnyWindowHandle,
    cx: &mut TestAppContext,
) -> (
    TimerPhase,
    &'static str,
    Option<String>,
    Option<StartTimer>,
) {
    handle
        .downcast::<Presenter>()
        .unwrap()
        .update(cx, |view, _, _| {
            let (phase, label, resumable, pending) = view.capsule_probe();
            (phase, label, resumable.map(str::to_owned), pending)
        })
        .unwrap()
}

/// Commit the pending Start transaction the input created. TestPlatform cannot
/// confirm a fullscreen audience, so swap in a windowed one of the same kind.
fn commit_pending_start(shell: &Entity<Shell>, cx: &mut TestAppContext) {
    shell.update(cx, |shell, cx| {
        let pending = shell.pending.take().expect("input created a Start transaction");
        if let Some(window) = pending.audience {
            Shell::close(window, cx);
        }
        let audience = test_window(shell, true, cx);
        shell.pending = Some(Transaction {
            kind: pending.kind,
            presenter: None,
            audience: Some(audience),
            deadline: Instant::now() + CONFIRM_TIMEOUT,
        });
        shell.poll_transaction(cx);
        assert!(shell.pending.is_none());
        assert_eq!(shell.state.stage(), SessionStage::Presenting);
    });
}

#[gpui::test]
fn resumable_session_splits_start_and_dispatches_resume_or_restart(cx: &mut TestAppContext) {
    let (shell, handle) = startable_fixture(cx);
    {
        let mut visual = gpui::VisualTestContext::from_window(handle, cx);
        assert!(visual.debug_bounds("start").is_some());
        assert!(visual.debug_bounds("session-start-split").is_none());
        assert!(visual.debug_bounds("timer-mark-fresh").is_some());
    }
    assert_eq!(capsule(handle, cx).0, TimerPhase::Fresh);
    interrupt(&shell, 754, cx);
    redraw(handle, cx);
    let page = shell.read_with(cx, |shell, _| shell.state.current_page());
    {
        let mut visual = gpui::VisualTestContext::from_window(handle, cx);
        assert!(visual.debug_bounds("start").is_none());
        let resume = visual.debug_bounds("start-resume").unwrap();
        let restart = visual.debug_bounds("start-restart").unwrap();
        let mode = visual.debug_bounds("start-mode").unwrap();
        assert!(same_row(resume.center().y, restart.center().y));
        assert!(resume.right() <= restart.left() + px(4.) && restart.right() <= mode.left());
        assert!(resume.size.height >= px(crate::theme::SESSION_CONTROL));
        assert!(resume.size.width >= px(crate::theme::SESSION_WIDTH));
        assert!(restart.size.width >= px(crate::theme::RESTART_WIDTH));
        assert!(visual.debug_bounds("start-resume-detail").is_some());
        assert!(visual.debug_bounds("start-restart-detail").is_some());
        assert!(visual.debug_bounds("timer-mark-stopped").is_some());
    }
    assert_eq!(
        capsule(handle, cx),
        (
            TimerPhase::Stopped,
            "wb-timer-stopped",
            Some("12:34".into()),
            None
        )
    );

    // Resume through a real click: one transaction, busy on that segment only.
    click_control(handle, "start-resume", cx);
    cx.run_until_parked();
    redraw(handle, cx);
    assert_eq!(capsule(handle, cx).3, Some(StartTimer::Resume));
    click_control(handle, "start-restart", cx);
    cx.run_until_parked();
    shell.read_with(cx, |shell, _| {
        assert_eq!(shell.pending_start(), Some(StartTimer::Resume));
        assert_eq!(shell.state.stage(), SessionStage::Ready);
        assert!(shell.resumable_elapsed().is_some());
    });
    commit_pending_start(&shell, cx);
    shell.read_with(cx, |shell, _| {
        assert!(shell.state.elapsed_at(Instant::now()) >= Duration::from_secs(754));
        assert_eq!(shell.state.current_page(), page);
    });
    redraw(handle, cx);
    {
        let mut visual = gpui::VisualTestContext::from_window(handle, cx);
        assert!(visual.debug_bounds("stop").is_some());
        assert!(visual.debug_bounds("session-start-split").is_none());
        assert!(visual.debug_bounds("timer-mark-running").is_some());
    }

    // Stop again; Restart zeroes the timer through its own segment.
    shell.update(cx, |shell, cx| shell.dispatch(Intent::Stop, cx));
    redraw(handle, cx);
    assert_eq!(capsule(handle, cx).0, TimerPhase::Stopped);
    click_control(handle, "start-restart", cx);
    cx.run_until_parked();
    redraw(handle, cx);
    assert_eq!(capsule(handle, cx).3, Some(StartTimer::Restart));
    commit_pending_start(&shell, cx);
    shell.read_with(cx, |shell, _| {
        assert!(shell.state.elapsed_at(Instant::now()) < Duration::from_secs(5));
        assert_eq!(shell.resumable_elapsed(), None);
        assert_eq!(shell.state.current_page(), page);
    });

    // Home discards the resumable session.
    shell.update(cx, |shell, cx| shell.dispatch(Intent::Stop, cx));
    interrupt_live(&shell, cx);
    shell.update(cx, |shell, cx| {
        assert!(shell.resumable_elapsed().is_some());
        shell.dispatch(Intent::Home, cx);
        assert_eq!(shell.state.stage(), SessionStage::Idle);
        assert_eq!(shell.resumable_elapsed(), None);
    });
}

/// Stop the current live session after letting it run for a few seconds.
fn interrupt_live(shell: &Entity<Shell>, cx: &mut TestAppContext) {
    shell.update(cx, |shell, cx| {
        shell.state = PresentationState::idle();
        let descriptor = shell
            .workbench
            .document
            .as_ref()
            .unwrap()
            .document
            .descriptor(ferrostage_core::DocumentGeneration(2))
            .unwrap();
        shell
            .state
            .apply_at(Command::LoadMappedDocument(descriptor), Instant::now())
            .unwrap();
        cx.notify();
    });
    interrupt(shell, 5, cx);
}

#[gpui::test]
fn start_shortcut_resumes_interrupted_session_and_starts_fresh_otherwise(
    cx: &mut TestAppContext,
) {
    let (shell, handle) = startable_fixture(cx);
    cx.simulate_keystrokes(handle, "f5");
    cx.run_until_parked();
    shell.update(cx, |shell, cx| {
        assert_eq!(shell.pending_start(), Some(StartTimer::Restart));
        shell.cancel_transaction("test", cx);
    });
    interrupt(&shell, 90, cx);
    redraw(handle, cx);
    cx.simulate_keystrokes(handle, "f5");
    cx.run_until_parked();
    shell.update(cx, |shell, cx| {
        assert_eq!(shell.pending_start(), Some(StartTimer::Resume));
        // A cancelled or failed Resume never touched core.
        shell.cancel_transaction("test", cx);
        assert_eq!(shell.state.stage(), SessionStage::Ready);
        assert_eq!(shell.resumable_elapsed(), Some(Duration::from_secs(90)));
    });
    // A short session is not resumable, so F5 restarts it.
    shell.update(cx, |shell, cx| {
        let now = Instant::now();
        shell.state = PresentationState::idle();
        let descriptor = shell
            .workbench
            .document
            .as_ref()
            .unwrap()
            .document
            .descriptor(ferrostage_core::DocumentGeneration(2))
            .unwrap();
        shell
            .state
            .apply_at(Command::LoadMappedDocument(descriptor), now)
            .unwrap();
        shell
            .state
            .apply_at(
                Command::StartPresentation {
                    audience_available: true,
                },
                now - Duration::from_millis(400),
            )
            .unwrap();
        shell.state.apply_at(Command::StopPresentation, now).unwrap();
        cx.notify();
    });
    redraw(handle, cx);
    {
        let mut visual = gpui::VisualTestContext::from_window(handle, cx);
        assert!(visual.debug_bounds("start").is_some());
        assert!(visual.debug_bounds("session-start-split").is_none());
    }
    cx.simulate_keystrokes(handle, "f5");
    cx.run_until_parked();
    shell.read_with(cx, |shell, _| {
        assert_eq!(shell.pending_start(), Some(StartTimer::Restart))
    });
}

#[gpui::test]
fn presenting_header_hosts_layout_menu_and_timer_states(cx: &mut TestAppContext) {
    let (shell, handle) = workbench_fixture(cx);
    cx.test_window(handle)
        .simulate_resize(size(px(1280.), px(800.)));
    shell.update(cx, |shell, cx| {
        shell.settings.settings.timer.mode = ferrostage_settings::TimerMode::CountUp;
        shell
            .state
            .apply_at(
                Command::StartPresentation {
                    audience_available: true,
                },
                Instant::now() - Duration::from_secs(30),
            )
            .unwrap();
        cx.notify();
    });
    redraw(handle, cx);
    {
        let mut visual = gpui::VisualTestContext::from_window(handle, cx);
        let header = visual.debug_bounds("presenter-bar").unwrap();
        let actions = visual.debug_bounds("presenter-actions").unwrap();
        let strip = visual.debug_bounds("workbench-controls").unwrap();
        let trigger = visual.debug_bounds("document-options").unwrap();
        let more = visual.debug_bounds("more").unwrap();
        assert!(header.contains(&trigger.center()) && actions.contains(&trigger.center()));
        assert!(!strip.contains(&trigger.center()));
        assert!(trigger.right() <= more.left(), "layout trigger leads the actions");
        assert!(visual.debug_bounds("timer-mark-running").is_some());
        assert!(visual.debug_bounds("timer-pause").is_some());
        assert!(visual.debug_bounds("timer-reset").is_some());
    }
    assert_eq!(capsule(handle, cx).1, "wb-elapsed");
    click_control(handle, "timer-pause", cx);
    redraw(handle, cx);
    let paused = capsule(handle, cx);
    assert_eq!((paused.0, paused.1), (TimerPhase::Paused, "wb-timer-paused"));
    {
        let mut visual = gpui::VisualTestContext::from_window(handle, cx);
        assert!(visual.debug_bounds("timer-mark-paused").is_some());
    }
    click_control(handle, "document-options", cx);
    redraw(handle, cx);
    {
        let mut visual = gpui::VisualTestContext::from_window(handle, cx);
        let trigger = visual.debug_bounds("document-options").unwrap();
        let panel = visual.debug_bounds("layout-navigation-menu").unwrap();
        assert!(panel.top() > trigger.bottom(), "menu opens below the header");
        assert!(visual.debug_bounds("document-options-chevron-open").is_some());
    }
    cx.simulate_keystrokes(handle, "escape");
    cx.run_until_parked();
    redraw(handle, cx);
    assert_eq!(layout_focus(handle, cx), (None, true));
    shell.read_with(cx, |shell, _| {
        assert!(shell.contextual.is_none());
        assert_eq!(shell.state.stage(), SessionStage::Presenting);
    });
}

#[gpui::test]
fn header_timer_centers_and_ignores_filename_length(cx: &mut TestAppContext) {
    let (shell, handle) = workbench_fixture(cx);
    let start = |shell: &mut Shell| {
        shell
            .state
            .apply_at(
                Command::StartPresentation {
                    audience_available: true,
                },
                Instant::now() - Duration::from_secs(65),
            )
            .unwrap();
    };
    shell.update(cx, |shell, _| start(shell));
    // Centered while each side's actions fit in half the free width; below
    // that (Ready < ~1100, Presenting < ~1000) the row degrades without overlap.
    for (width, presenting, centered) in [
        (1600., false, true),
        (1280., false, true),
        (1100., false, true),
        (900., false, false),
        (1600., true, true),
        (1280., true, true),
        (1000., true, true),
        (800., true, false),
    ] {
        shell.update(cx, |shell, cx| {
            if !presenting && shell.state.stage() == SessionStage::Presenting {
                shell.state.apply_at(Command::StopPresentation, Instant::now()).unwrap();
            } else if presenting && shell.state.stage() == SessionStage::Ready {
                start(shell);
            }
            cx.notify();
        });
        cx.test_window(handle)
            .simulate_resize(size(px(width), px(800.)));
        let mut digits = None;
        for name in ["a.pdf", &format!("{}.pdf", "very-long-defense-title-".repeat(12))] {
            shell.update(cx, |shell, cx| {
                shell.path = Some(name.into());
                cx.notify();
            });
            redraw(handle, cx);
            let mut visual = gpui::VisualTestContext::from_window(handle, cx);
            let timer = visual.debug_bounds("presenter-timer").unwrap();
            let actual = visual.debug_bounds("presenter-timer-digits").unwrap();
            let title = visual.debug_bounds("document-title").unwrap();
            let actions = visual.debug_bounds("presenter-actions").unwrap();
            assert!(title.right() <= timer.left(), "{width}: title overlaps timer");
            assert!(timer.right() <= actions.left(), "{width}: actions overlap timer");
            assert!(actions.right() <= px(width), "{width}: actions overflow");
            assert_eq!(
                (f32::from(timer.center().x) - width / 2.).abs() <= 1.,
                centered,
                "{width}/{presenting}: timer {timer:?}"
            );
            if let Some(expected) = digits {
                assert_eq!(actual, expected, "{width}: filename moved the digits");
            }
            digits = Some(actual);
        }
    }
    // Narrow windows stack the capsule on its own centered row.
    cx.test_window(handle)
        .simulate_resize(size(px(600.), px(800.)));
    redraw(handle, cx);
    let mut visual = gpui::VisualTestContext::from_window(handle, cx);
    let timer = visual.debug_bounds("presenter-timer").unwrap();
    let title = visual.debug_bounds("document-title").unwrap();
    let digits = visual.debug_bounds("presenter-timer-digits").unwrap();
    assert!(timer.top() >= title.bottom());
    assert!((f32::from(timer.center().x) - 300.).abs() <= 1.);
    assert!(digits.size.width >= px(30. * 0.6 * 7.));
}

#[gpui::test]
fn split_session_control_stays_visible_in_narrow_windows(cx: &mut TestAppContext) {
    let (shell, handle) = workbench_fixture(cx);
    interrupt(&shell, 75, cx);
    for width in [1100., 800., 390., 320.] {
        cx.test_window(handle)
            .simulate_resize(size(px(width), px(760.)));
        redraw(handle, cx);
        let mut visual = gpui::VisualTestContext::from_window(handle, cx);
        let resume = visual.debug_bounds("start-resume").unwrap();
        let restart = visual.debug_bounds("start-restart").unwrap();
        // At 320 the bottom strip already clips its trailing controls (as for
        // Previous/Next); Resume, like Start, must remain fully visible.
        let visible: &[(&str, gpui::Bounds<gpui::Pixels>)] = if width < 390. {
            &[("resume", resume)]
        } else {
            &[("resume", resume), ("restart", restart)]
        };
        for &(name, bounds) in visible {
            assert!(
                bounds.left() >= px(0.) && bounds.right() <= px(width),
                "{width}: {name} {bounds:?}"
            );
            assert!(bounds.size.height >= px(crate::theme::CONTROL));
        }
    }
}
