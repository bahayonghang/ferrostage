pub(super) fn timed_fixture(
    cx: &mut TestAppContext,
) -> (Entity<Shell>, AnyWindowHandle, AnyWindowHandle) {
    let (shell, presenter) = workbench_fixture(cx);
    let audience = shell.update(cx, |s, cx| {
        let session = s.workbench.document.as_ref().unwrap();
        let mut structure = session.structure.clone();
        for page in &mut structure.pages {
            page.presentation = ferrostage_core::PagePresentation {
                transition: Some(ferrostage_core::PageTransition {
                    style: ferrostage_core::TransitionStyle::Fade,
                    duration_seconds: 0.,
                    ..Default::default()
                }),
                dwell_seconds: Some(0.),
            };
        }
        s.workbench.document = Some(
            crate::workbench::DocumentSession::build(session.metadata.clone(), structure).unwrap(),
        );
        let audience = test_window(s, true, cx);
        let handle = audience.handle;
        s.audience = Some(audience);
        s.apply(
            Command::StartPresentation {
                audience_available: true,
            },
            cx,
        );
        handle
    });
    presenter
        .update(cx, |_, window, _| window.activate_window())
        .unwrap();
    (shell, presenter, audience)
}

pub(super) fn timed_frame(s: &mut Shell, cx: &mut Context<Shell>) {
    use crate::workbench::{ReadyFrame, Surface};
    let target = ferrostage_pdf::PixelSize {
        width: 4,
        height: 3,
    };
    s.workbench.audience_target = Some(target);
    let mut request = crate::workbench::request_for(
        s.state.generation(),
        s.state.current_page(),
        s.state.slide_crop(),
    );
    request.target_px = target;
    request.priority = ferrostage_render::RenderPriority::Audience;
    s.workbench
        .frames
        .retain(|(surface, _)| *surface != Surface::Audience);
    s.workbench.frames.push((
        Surface::Audience,
        ReadyFrame {
            request,
            frame: ferrostage_core::BgraFrame::new(4, 3, 16, vec![220; 48]).unwrap(),
        },
    ));
    s.publish(false, cx);
}

#[gpui::test]
fn timed_navigation_identity_waits_for_exact_destination_pixels(cx: &mut TestAppContext) {
    let (shell, _, _) = timed_fixture(cx);
    shell.update(cx, |s, cx| {
        timed_frame(s, cx);
        let first = s.snapshot.read(cx).playback.unwrap();
        assert!(first.transition.is_none());
        s.apply(Command::NextStep, cx);
        assert_eq!(s.state.current_page(), PhysicalPage(2));
        assert_eq!(s.snapshot.read(cx).current_page, PhysicalPage(0));
        assert_eq!(s.snapshot.read(cx).playback, Some(first));
        // New page, wrong target: it cannot authorize incoming playback.
        s.workbench.frames[0].1.request.page = PhysicalPage(2);
        s.workbench.frames[0].1.request.target_px.width = 99;
        s.publish(false, cx);
        assert_eq!(s.snapshot.read(cx).playback, Some(first));
        timed_frame(s, cx);
        let next = s.snapshot.read(cx).playback.unwrap();
        assert_ne!(first.identity, next.identity);
        assert_eq!(next.identity.page, PhysicalPage(2));
        assert!(next.transition.is_some());
        s.apply(Command::JumpTo(PhysicalPage(2)), cx);
        let jump = s.snapshot.read(cx).playback.unwrap();
        assert_ne!(next.identity, jump.identity);
        assert!(jump.transition.is_none());
        s.apply(Command::PreviousStep, cx);
        timed_frame(s, cx);
        assert_eq!(s.snapshot.read(cx).current_page, PhysicalPage(0));
        assert!(s.snapshot.read(cx).playback.unwrap().transition.is_some());
    });
}

#[gpui::test]
fn timed_zero_uses_real_audience_paint_events_once_and_skips_notes(cx: &mut TestAppContext) {
    let (shell, _, audience) = timed_fixture(cx);
    for (page, next) in [(0, 2), (2, 3), (3, 3)] {
        shell.update(cx, |s, cx| {
            assert_eq!(s.state.current_page(), PhysicalPage(page));
            s.poll_timed_playback(cx);
            assert_eq!(
                s.state.current_page(),
                PhysicalPage(page),
                "unpainted steps never catch up"
            );
            timed_frame(s, cx);
        });
        redraw(audience, cx);
        cx.run_until_parked();
        shell.update(cx, |s, cx| {
            assert_eq!(
                s.state.current_page(),
                PhysicalPage(page),
                "paint events do not navigate recursively"
            );
            s.poll_timed_playback(cx);
            assert_eq!(s.state.current_page(), PhysicalPage(next));
            s.poll_timed_playback(cx);
            assert_eq!(s.state.current_page(), PhysicalPage(next));
        });
    }
}

#[gpui::test]
fn timed_pause_button_is_independent_and_preserved_during_manual_navigation(
    cx: &mut TestAppContext,
) {
    let (shell, presenter, audience) = timed_fixture(cx);
    shell.update(cx, timed_frame);
    redraw(presenter, cx);
    click_control(presenter, "automatic-playback", cx);
    redraw(audience, cx);
    cx.run_until_parked();
    shell.update(cx, |s, cx| {
        assert!(s.autoplay_paused());
        assert!(!s.state.is_timer_paused());
        s.poll_timed_playback(cx);
        assert_eq!(s.state.current_page(), PhysicalPage(0));
        s.apply(Command::NextStep, cx);
        timed_frame(s, cx);
        assert!(s.autoplay_paused());
    });
    redraw(audience, cx);
    cx.run_until_parked();
    shell.update(cx, |s, cx| {
        s.poll_timed_playback(cx);
        assert_eq!(s.state.current_page(), PhysicalPage(2));
    });
    redraw(presenter, cx);
    click_control(presenter, "automatic-playback", cx);
    shell.update(cx, |s, cx| {
        s.poll_timed_playback(cx);
        assert_eq!(s.state.current_page(), PhysicalPage(3));
    });
}

#[gpui::test]
fn timed_blank_modal_focus_error_annotation_and_stop_prevent_navigation(cx: &mut TestAppContext) {
    let (shell, presenter, audience) = timed_fixture(cx);
    shell.update(cx, timed_frame);
    redraw(audience, cx);
    cx.run_until_parked();
    shell.update(cx, |s, cx| {
        s.dispatch(Intent::Black, cx);
        s.poll_timed_playback(cx);
        assert_eq!(s.state.current_page(), PhysicalPage(0));
        s.dispatch(Intent::Black, cx);
        s.modal = Some(Modal::Settings);
        s.poll_timed_playback(cx);
        assert_eq!(s.state.current_page(), PhysicalPage(0));
        s.modal = None;
        s.graphics_error = Some("failed upload".into());
        s.poll_timed_playback(cx);
        assert_eq!(s.state.current_page(), PhysicalPage(0));
        s.graphics_error = None;
    });
    audience
        .update(cx, |_, window, _| window.activate_window())
        .unwrap();
    shell.update(cx, |s, cx| {
        s.poll_timed_playback(cx);
        assert_eq!(s.state.current_page(), PhysicalPage(0));
    });
    presenter
        .update(cx, |_, window, _| window.activate_window())
        .unwrap();
    shell.update(cx, |s, cx| {
        s.apply(Command::UpdateLaserPointer(Some((0.5, 0.5))), cx);
        s.poll_timed_playback(cx);
        assert!(s.autoplay_paused());
        assert!(s.snapshot.read(cx).playback.unwrap().transition.is_none());
        let event = crate::audience::PlaybackEvent {
            identity: s.snapshot.read(cx).playback.unwrap().identity,
            phase: crate::audience::PlaybackPhase::Settled,
            at: Instant::now(),
        };
        s.stop(cx);
        s.playback_event(audience.window_id(), event, cx);
        s.poll_timed_playback(cx);
        assert_eq!(s.state.stage(), SessionStage::Ready);
        assert_eq!(s.state.current_page(), PhysicalPage(0));
    });
}

#[gpui::test]
fn timed_reload_and_retired_window_events_cannot_advance(cx: &mut TestAppContext) {
    let (shell, _, audience) = timed_fixture(cx);
    shell.update(cx, |s, cx| {
        timed_frame(s, cx);
        let old = s.snapshot.read(cx).playback.unwrap().identity;
        s.invalidate_playback();
        s.publish(false, cx);
        let current = s.snapshot.read(cx).playback.unwrap().identity;
        assert_ne!(old, current);
        assert!(s.snapshot.read(cx).playback.unwrap().transition.is_none());
        let retired = test_window(s, true, cx);
        for (source, identity) in [
            (audience.window_id(), old),
            (retired.handle.window_id(), current),
        ] {
            for phase in [
                crate::audience::PlaybackPhase::FirstVisible,
                crate::audience::PlaybackPhase::Settled,
            ] {
                s.playback_event(
                    source,
                    crate::audience::PlaybackEvent {
                        identity,
                        phase,
                        at: Instant::now(),
                    },
                    cx,
                );
            }
        }
        s.poll_timed_playback(cx);
        assert_eq!(s.state.current_page(), PhysicalPage(0));
        Shell::close(retired, cx);
    });
}
