use std::time::{Duration, Instant};

use super::*;
use PresentationCommand::*;

fn ready(now: Instant) -> PresentationState {
    let mut state = PresentationState::idle();
    state
        .apply_at(
            LoadDocument {
                generation: DocumentGeneration(3),
                page_count: 4,
            },
            now,
        )
        .unwrap();
    state
}

fn live(now: Instant) -> PresentationState {
    let mut state = ready(now);
    state
        .apply_at(
            StartPresentation {
                audience_available: true,
            },
            now,
        )
        .unwrap();
    state
}

#[test]
fn legal_lifecycle_edges_control_projection_and_document_resets() {
    let now = Instant::now();
    let idle = PresentationState::idle();
    assert_eq!(idle.stage(), SessionStage::Idle);
    assert_eq!(idle.audience_snapshot().stage, AudienceStage::Curtain);
    let mut state = ready(now);
    assert_eq!(state.stage(), SessionStage::Ready);
    assert_eq!(
        PresentationState::new(2).unwrap().stage(),
        SessionStage::Ready
    );
    assert_eq!(state.audience_snapshot().stage, AudienceStage::Curtain);
    state
        .apply_at(
            StartPresentation {
                audience_available: true,
            },
            now,
        )
        .unwrap();
    assert_eq!(state.stage(), SessionStage::Presenting);
    assert_eq!(state.audience_snapshot().stage, AudienceStage::Live);
    state.apply_at(NextStep, now).unwrap();
    state.apply_at(AppendStrokePoint((0.2, 0.3)), now).unwrap();
    state
        .apply_at(StopPresentation, now + Duration::from_secs(5))
        .unwrap();
    assert_eq!(state.stage(), SessionStage::Ready);
    assert_eq!(state.current_page(), PhysicalPage(1));
    assert_eq!(state.audience_snapshot().strokes.len(), 1);
    assert_eq!(state.audience_snapshot().stage, AudienceStage::Curtain);
    state.apply_at(JumpTo(PhysicalPage(2)), now).unwrap();
    state.apply_at(ToggleBlank(BlankMode::Black), now).unwrap();
    state
        .apply_at(UpdateLaserPointer(Some((0.2, 0.3))), now)
        .unwrap();
    state.set_slide_crop(NormalizedRect::LEFT_HALF).unwrap();
    state
        .apply_at(
            LoadDocument {
                generation: DocumentGeneration(3),
                page_count: 4,
            },
            now,
        )
        .unwrap();
    assert_eq!(state, ready(now));

    for mut state in [idle, ready(now), live(now)] {
        state
            .apply_at(CloseDocument, now + Duration::from_secs(10))
            .unwrap();
        assert_eq!(state, PresentationState::idle());
        assert_eq!(
            state.elapsed_at(now + Duration::from_secs(20)),
            Duration::ZERO
        );
    }
}

#[test]
fn illegal_lifecycle_edges_leave_every_field_unchanged() {
    let now = Instant::now();
    for state in [PresentationState::idle(), ready(now), live(now)] {
        let commands = match state.stage() {
            SessionStage::Idle => vec![
                StartPresentation {
                    audience_available: true,
                },
                ResumePresentation {
                    audience_available: true,
                },
                StopPresentation,
                PauseTimer,
                ResumeTimer,
                ResetTimer,
            ],
            SessionStage::Ready => vec![
                ResumePresentation {
                    audience_available: true,
                },
                StopPresentation,
                PauseTimer,
                ResumeTimer,
                ResetTimer,
            ],
            SessionStage::Presenting => vec![
                StartPresentation {
                    audience_available: true,
                },
                ResumePresentation {
                    audience_available: true,
                },
                LoadDocument {
                    generation: DocumentGeneration(4),
                    page_count: 1,
                },
            ],
        };
        for command in commands {
            let mut actual = state.clone();
            assert_eq!(
                actual.apply_at(command, now),
                Err(CoreError::NotReady {
                    stage: state.stage()
                })
            );
            assert_eq!(actual, state);
        }
    }
    for mut state in [PresentationState::idle(), ready(now)] {
        let before = state.clone();
        assert_eq!(
            state.apply_at(
                LoadDocument {
                    generation: DocumentGeneration(5),
                    page_count: 0
                },
                now
            ),
            Err(CoreError::EmptyDocument)
        );
        assert_eq!(state, before);
    }
    let mut state = ready(now);
    let before = state.clone();
    assert_eq!(
        state.apply_at(
            StartPresentation {
                audience_available: false
            },
            now
        ),
        Err(CoreError::NoAudienceDisplay)
    );
    assert_eq!(state, before);
}

#[test]
fn timer_pause_resume_stop_and_restart_use_only_injected_time() {
    let origin = Instant::now();
    let at = |seconds| origin + Duration::from_secs(seconds);
    let mut state = live(at(0));
    state.apply_at(PauseTimer, at(5)).unwrap();
    assert!(state.is_timer_paused());
    assert_eq!(state.elapsed_at(at(10)), Duration::from_secs(5));
    state.apply_at(PauseTimer, at(10)).unwrap();
    state.apply_at(ResumeTimer, at(10)).unwrap();
    state.apply_at(ResumeTimer, at(11)).unwrap();
    assert!(!state.is_timer_paused());
    state.apply_at(StopPresentation, at(12)).unwrap();
    assert_eq!(state.elapsed_at(at(17)), Duration::from_secs(7));
    state
        .apply_at(
            StartPresentation {
                audience_available: true,
            },
            at(17),
        )
        .unwrap();
    assert_eq!(state.elapsed_at(at(17)), Duration::ZERO);
    state.apply_at(ResetTimer, at(20)).unwrap();
    assert!(!state.is_timer_paused());
    assert_eq!(state.elapsed_at(at(22)), Duration::from_secs(2));
    state.apply_at(PauseTimer, at(22)).unwrap();
    state.apply_at(ResetTimer, at(23)).unwrap();
    assert!(state.is_timer_paused());
    assert_eq!(state.elapsed_at(at(30)), Duration::ZERO);
    state.apply_at(ResumeTimer, at(30)).unwrap();
    state.apply_at(StopPresentation, at(33)).unwrap();
    state
        .apply_at(
            LoadDocument {
                generation: DocumentGeneration(8),
                page_count: 2,
            },
            at(40),
        )
        .unwrap();
    assert_eq!(state.elapsed_at(at(50)), Duration::ZERO);
}

#[test]
fn navigation_and_tools_require_a_document_but_not_a_live_session() {
    let now = Instant::now();
    let commands = [
        NextStep,
        PreviousStep,
        NextLogicalSlide,
        PreviousLogicalSlide,
        JumpTo(PhysicalPage(1)),
        ToggleBlank(BlankMode::White),
        UpdateLaserPointer(Some((0.1, 0.2))),
        AppendStrokePoint((0.1, 0.2)),
        ClearStrokes,
    ];
    for command in commands {
        let mut idle = PresentationState::idle();
        assert_eq!(
            idle.apply_at(command.clone(), now),
            Err(CoreError::NotReady {
                stage: SessionStage::Idle
            })
        );
        assert_eq!(idle, PresentationState::idle());
        for mut state in [ready(now), live(now)] {
            state.apply_at(command.clone(), now).unwrap();
        }
    }
}

#[test]
fn descriptor_commit_is_atomic_clamps_position_and_preserves_session_timer() {
    let now = Instant::now();
    for mut state in [ready(now), live(now)] {
        state.apply_at(JumpTo(PhysicalPage(3)), now).unwrap();
        state.apply_at(JumpTo(PhysicalPage(2)), now).unwrap();
        let before = state.clone();
        for descriptor in [
            DocumentDescriptor::identity(DocumentGeneration(3), 2).unwrap(),
            DocumentDescriptor::identity(DocumentGeneration(2), 2).unwrap(),
        ] {
            assert!(state.apply_at(CommitDocument(descriptor), now).is_err());
            assert_eq!(state, before);
        }
        state
            .apply_at(
                CommitDocument(DocumentDescriptor::identity(DocumentGeneration(4), 2).unwrap()),
                now + Duration::from_secs(5),
            )
            .unwrap();
        assert_eq!(state.generation(), DocumentGeneration(4));
        assert_eq!(state.page_count(), 2);
        assert_eq!(state.current_page(), PhysicalPage(1));
        assert_eq!(state.stage(), before.stage());
        assert_eq!(
            state.elapsed_at(now + Duration::from_secs(10)),
            before.elapsed_at(now + Duration::from_secs(10))
        );
    }
    let mut idle = PresentationState::idle();
    for command in [
        CommitDocument(DocumentDescriptor::identity(DocumentGeneration(1), 2).unwrap()),
        StageShadowDocument {
            new_generation: DocumentGeneration(1),
            bytes: Arc::new(vec![1]),
        },
    ] {
        assert_eq!(
            idle.apply_at(command, now),
            Err(CoreError::NotReady {
                stage: SessionStage::Idle
            })
        );
        assert_eq!(idle, PresentationState::idle());
    }
}

fn resumable(origin: Instant, seconds: u64) -> PresentationState {
    let mut state = live(origin);
    state.apply_at(NextStep, origin).unwrap();
    state
        .apply_at(StopPresentation, origin + Duration::from_secs(seconds))
        .unwrap();
    state
}

#[test]
fn stopped_session_resumes_from_frozen_time_or_restarts_at_zero() {
    let origin = Instant::now();
    let at = |seconds| origin + Duration::from_secs(seconds);
    let mut state = resumable(origin, 12);
    assert_eq!(state.stage(), SessionStage::Ready);
    assert_eq!(
        state.resumable_elapsed(at(40)),
        Some(Duration::from_secs(12))
    );
    assert_eq!(state.elapsed_at(at(40)), Duration::from_secs(12));
    let page = state.current_page();

    let before = state.clone();
    assert_eq!(
        state.apply_at(
            ResumePresentation {
                audience_available: false
            },
            at(40)
        ),
        Err(CoreError::NoAudienceDisplay)
    );
    assert_eq!(state, before);

    state
        .apply_at(
            ResumePresentation {
                audience_available: true,
            },
            at(40),
        )
        .unwrap();
    assert_eq!(state.stage(), SessionStage::Presenting);
    assert_eq!(state.current_page(), page);
    assert_eq!(state.resumable_elapsed(at(45)), None);
    assert!(!state.is_timer_paused());
    assert_eq!(state.elapsed_at(at(45)), Duration::from_secs(17));

    // A pause before stopping does not carry over: resume always runs.
    state.apply_at(PauseTimer, at(45)).unwrap();
    state.apply_at(StopPresentation, at(50)).unwrap();
    assert_eq!(
        state.resumable_elapsed(at(60)),
        Some(Duration::from_secs(17))
    );
    state
        .apply_at(
            ResumePresentation {
                audience_available: true,
            },
            at(60),
        )
        .unwrap();
    assert!(!state.is_timer_paused());
    assert_eq!(state.elapsed_at(at(61)), Duration::from_secs(18));

    state.apply_at(StopPresentation, at(61)).unwrap();
    state
        .apply_at(
            StartPresentation {
                audience_available: true,
            },
            at(70),
        )
        .unwrap();
    assert_eq!(state.current_page(), page);
    assert_eq!(state.resumable_elapsed(at(70)), None);
    assert_eq!(state.elapsed_at(at(72)), Duration::from_secs(2));
}

#[test]
fn short_sessions_and_document_boundaries_clear_resumable_time() {
    let origin = Instant::now();
    let mut short = live(origin);
    short
        .apply_at(StopPresentation, origin + Duration::from_millis(999))
        .unwrap();
    assert_eq!(short.resumable_elapsed(origin), None);
    let before = short.clone();
    assert_eq!(
        short.apply_at(
            ResumePresentation {
                audience_available: true
            },
            origin
        ),
        Err(CoreError::NotReady {
            stage: SessionStage::Ready
        })
    );
    assert_eq!(short, before);

    let mut exact = live(origin);
    exact
        .apply_at(StopPresentation, origin + Duration::from_secs(1))
        .unwrap();
    assert_eq!(
        exact.resumable_elapsed(origin),
        Some(Duration::from_secs(1))
    );

    for command in [
        LoadDocument {
            generation: DocumentGeneration(9),
            page_count: 2,
        },
        LoadMappedDocument(DocumentDescriptor::identity(DocumentGeneration(9), 2).unwrap()),
        CloseDocument,
    ] {
        let mut state = resumable(origin, 12);
        state.apply_at(command, origin).unwrap();
        assert_eq!(state.resumable_elapsed(origin), None);
        assert_eq!(state.elapsed_at(origin), Duration::ZERO);
    }

    let mut reloaded = resumable(origin, 12);
    reloaded
        .apply_at(
            CommitDocument(DocumentDescriptor::identity(DocumentGeneration(4), 4).unwrap()),
            origin + Duration::from_secs(20),
        )
        .unwrap();
    assert_eq!(
        reloaded.resumable_elapsed(origin + Duration::from_secs(30)),
        Some(Duration::from_secs(12))
    );
    let mut checked = resumable(origin, 12);
    checked
        .commit_reload(
            DocumentDescriptor::identity(DocumentGeneration(4), 4).unwrap(),
            PhysicalPage(1),
            &[],
        )
        .unwrap();
    assert_eq!(
        checked.resumable_elapsed(origin),
        Some(Duration::from_secs(12))
    );
}
