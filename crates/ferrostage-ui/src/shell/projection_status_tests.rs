fn projection_fixture(cx: &mut TestAppContext) -> (Entity<Shell>, AnyWindowHandle, AnyWindowHandle) {
    let (shell, presenter) = workbench_fixture(cx);
    let audience = shell.update(cx, |s, cx| {
        s.monitors = vec![test_monitor()];
        let audience = test_window(s, true, cx);
        let handle = audience.handle;
        s.audience = Some(audience);
        s.apply(Command::StartPresentation { audience_available: true }, cx);
        handle
    });
    (shell, presenter, audience)
}

fn projection_ack(s: &mut Shell, source: WindowId, phase: crate::audience::PlaybackPhase, cx: &mut Context<Shell>) {
    s.playback_event(source, crate::audience::PlaybackEvent {
        identity: s.selected_playback().unwrap(), phase, at: Instant::now(),
    }, cx);
}

#[gpui::test]
fn projection_waits_for_settled_and_retains_last_draw_during_pending_or_failure(cx: &mut TestAppContext) {
    use crate::audience::PlaybackPhase;
    let (shell, _, audience) = projection_fixture(cx);
    shell.update(cx, |s, cx| {
        assert!(!s.has_authored_timing(), "ordinary pages also receive paint identities");
        timed_frame(s, cx);
        projection_ack(s, audience.window_id(), PlaybackPhase::FirstVisible, cx);
        assert!(s.settled_playback().is_none());
        projection_ack(s, audience.window_id(), PlaybackPhase::Settled, cx);
        let old = s.settled_playback().unwrap();
        s.apply(Command::JumpTo(PhysicalPage(2)), cx);
        assert!(!s.public_frame_ready());
        let pending = s.projection_status().unwrap();
        assert!(pending.summary.contains(s.locale.text("projection-drawn").split('{').next().unwrap()));
        assert!(pending.summary.contains(s.locale.text("projection-pending").split('{').next().unwrap()));
        assert_eq!(s.settled_playback(), Some(old));
        s.graphics_error = Some("failed upload".into());
        let failed = s.projection_status().unwrap();
        assert!(failed.failed);
        assert!(failed.summary.contains(s.locale.text("projection-failed")));
        assert_eq!(s.settled_playback(), Some(old));
        s.graphics_error = None;
        timed_frame(s, cx);
        let current = s.selected_playback().unwrap();
        s.playback_event(audience.window_id(), crate::audience::PlaybackEvent { identity: old, phase: PlaybackPhase::Settled, at: Instant::now() }, cx);
        assert_eq!(s.settled_playback(), Some(old));
        projection_ack(s, audience.window_id(), PlaybackPhase::FirstVisible, cx);
        assert_eq!(s.settled_playback(), Some(old));
        projection_ack(s, audience.window_id(), PlaybackPhase::Settled, cx);
        assert_eq!(s.settled_playback(), Some(current));
        assert!(!s.projection_status().unwrap().summary.contains(s.locale.text("projection-pending").split('{').next().unwrap()));
    });
}

#[gpui::test]
fn projection_real_audience_draw_emits_ordinary_page_receipt(cx: &mut TestAppContext) {
    let (shell, _, audience) = projection_fixture(cx);
    shell.update(cx, timed_frame);
    redraw(audience, cx);
    cx.run_until_parked();
    shell.read_with(cx, |s, _| {
        assert_eq!(s.settled_playback(), s.selected_playback());
        assert!(s.settled_playback().is_some());
    });
}

#[gpui::test]
fn projection_blank_restore_preserves_timer_pause_and_public_state_through_real_input(cx: &mut TestAppContext) {
    use crate::audience::PlaybackPhase;
    let (shell, presenter, audience) = projection_fixture(cx);
    for blank in [BlankMode::Black, BlankMode::White] {
        let before = shell.update(cx, |s, cx| {
            timed_frame(s, cx);
            projection_ack(s, audience.window_id(), PlaybackPhase::Settled, cx);
            s.timed.clock.pause(true, Instant::now());
            s.apply(Command::PauseTimer, cx);
            s.apply(Command::ToggleBlank(blank), cx);
            assert!(s.public_frame_ready(), "covered pixels remain ready, without a visible receipt");
            let old = s.settled_playback();
            s.apply(Command::JumpTo(if s.state.current_page() == PhysicalPage(0) { PhysicalPage(2) } else { PhysicalPage(0) }), cx);
            timed_frame(s, cx);
            let mut before = s.state.audience_snapshot();
            before.blank_mode = None;
            (before, s.state.is_timer_paused(), s.autoplay_paused(), old)
        });
        redraw(audience, cx);
        shell.read_with(cx, |s, _| {
            assert_eq!(s.settled_playback(), before.3, "a covered paint must not claim visible output");
        });
        redraw(presenter, cx);
        click_control(presenter, "projection-restore", cx);
        shell.read_with(cx, |s, _| {
            assert_eq!(s.state.audience_snapshot(), before.0);
            assert_eq!(s.state.is_timer_paused(), before.1);
            assert_eq!(s.autoplay_paused(), before.2);
            assert!(!s.projection_status().unwrap().restore);
        });
    }
}

#[gpui::test]
fn projection_rejects_retired_window_and_generation_receipts(cx: &mut TestAppContext) {
    use crate::audience::PlaybackPhase;
    let (shell, _, audience) = projection_fixture(cx);
    shell.update(cx, |s, cx| {
        timed_frame(s, cx);
        projection_ack(s, audience.window_id(), PlaybackPhase::Settled, cx);
        let old = s.settled_playback().unwrap();
        let replacement = test_window(s, true, cx);
        let current_window = replacement.handle.window_id();
        let retired = s.audience.replace(replacement).unwrap();
        s.publish(false, cx);
        timed_frame(s, cx);
        assert!(s.settled_playback().is_none());
        projection_ack(s, retired.handle.window_id(), PlaybackPhase::Settled, cx);
        assert!(s.settled_playback().is_none());
        let mut wrong_generation = s.selected_playback().unwrap();
        wrong_generation.generation.0 += 1;
        s.playback_event(current_window, crate::audience::PlaybackEvent { identity: wrong_generation, phase: PlaybackPhase::Settled, at: Instant::now() }, cx);
        assert!(s.settled_playback().is_none());
        projection_ack(s, current_window, PlaybackPhase::Settled, cx);
        assert_ne!(s.settled_playback(), Some(old));
        Shell::close(retired, cx);
        s.stop(cx);
        assert!(s.settled_playback().is_none());
        assert_eq!(s.projection_status().unwrap().summary, s.locale.text("projection-not-started"));
    });
}

#[gpui::test]
fn projection_reports_observation_unknown_and_configured_target_separately(cx: &mut TestAppContext) {
    let (shell, presenter, _) = projection_fixture(cx);
    shell.update(cx, |s, cx| {
        let actual = s.projection_status().unwrap();
        assert!(actual.location.contains("Headless display"));
        s.audience.as_ref().unwrap().observed_display.set(None);
        let unknown = s.projection_status().unwrap();
        assert!(unknown.location.contains(s.locale.text("displays-location-unknown")));
        assert!(!unknown.location.contains("Headless display"));
        s.stop(cx);
        let ready = s.projection_status().unwrap();
        assert!(ready.location.contains("Headless display"));
        assert!(ready.location.contains(s.locale.text("projection-configured").split('{').next().unwrap()));
        s.pending = Some(Transaction {
            kind: TransactionKind::Start(StartTimer::Restart),
            presenter: None,
            audience: None,
            deadline: Instant::now() + CONFIRM_TIMEOUT,
        });
        assert_eq!(s.projection_status().unwrap().summary, s.locale.text("projection-preparing"));
    });
    redraw(presenter, cx);
    let mut visual = gpui::VisualTestContext::from_window(presenter, cx);
    assert!(visual.debug_bounds("projection-status").is_some());
    assert!(visual.debug_bounds("projection-location").is_some());
}

#[gpui::test]
fn projection_public_readiness_rejects_recovery_loading_and_placement(cx: &mut TestAppContext) {
    let (shell, _, _) = projection_fixture(cx);
    shell.update(cx, |s, cx| {
        timed_frame(s, cx);
        s.apply(Command::ToggleBlank(BlankMode::Black), cx);
        assert!(s.public_frame_ready());
        s.loading = true;
        assert!(!s.public_frame_ready());
        s.loading = false;
        s.reload.failed_worker = true;
        assert!(!s.public_frame_ready());
        assert!(s.projection_status().unwrap().failed);
        s.reload.failed_worker = false;
        s.graphics_error = Some("failed upload".into());
        assert!(!s.public_frame_ready());
        s.graphics_error = None;
        s.pending = Some(Transaction { kind: TransactionKind::Assign(s.settings.settings.display.clone()), presenter: None, audience: None, deadline: Instant::now() + CONFIRM_TIMEOUT });
        assert!(!s.public_frame_ready());
        s.pending = None;
        assert!(s.public_frame_ready());
    });
}

#[gpui::test]
fn projection_blank_after_audience_delivery_keeps_receipts_for_unblank(cx: &mut TestAppContext) {
    use crate::audience::{Audience, PlaybackEvent, PlaybackPhase};
    let (shell, _, audience) = timed_fixture(cx);
    audience
        .downcast::<Audience>()
        .unwrap()
        .update(cx, |_, _, cx| {
            let mut blanked = false;
            cx.subscribe_self(move |view, event: &PlaybackEvent, cx| {
                if !blanked && event.phase == PlaybackPhase::FirstVisible {
                    blanked = true;
                    // Audience committed this delivery, but Shell consumes it
                    // on another deferred turn. A later mask cannot undo it.
                    view.snapshot.update(cx, |snapshot, _| {
                        snapshot.blank_mode = Some(BlankMode::Black);
                    });
                }
            })
            .detach();
        })
        .unwrap();
    shell.update(cx, timed_frame);
    redraw(audience, cx);
    cx.run_until_parked();
    shell.update(cx, |s, cx| {
        assert_eq!(s.snapshot.read(cx).blank_mode, Some(BlankMode::Black));
        // Restore the unblanked core projection after the late cover.
        s.publish(false, cx);
    });
    redraw(audience, cx);
    cx.run_until_parked();
    shell.update(cx, |s, cx| {
        assert_eq!(s.settled_playback(), s.selected_playback());
        s.poll_timed_playback(cx);
        s.poll_timed_playback(cx);
        assert_eq!(s.state.current_page(), PhysicalPage(2));
    });
}
