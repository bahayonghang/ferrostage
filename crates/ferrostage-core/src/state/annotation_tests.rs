use super::*;
use crate::{LaserStyle, PresentationCommand as C};
#[test]
fn annotation_presence_includes_off_page_ink_but_not_empty_strokes() {
    let now = Instant::now();
    let mut state = PresentationState::new(2).unwrap();
    assert!(!state.has_annotations());
    state
        .apply_at(
            C::BeginStroke {
                color: [0, 0, 255, 255],
                width: 0.01,
                is_highlighter: false,
            },
            now,
        )
        .unwrap();
    assert!(!state.has_annotations());
    state
        .apply_at(C::AppendStrokePoint((0.2, 0.3)), now)
        .unwrap();
    state.apply_at(C::EndStroke, now).unwrap();
    assert!(state.has_annotations());
    state.apply_at(C::NextStep, now).unwrap();
    assert!(state.audience_snapshot().strokes.is_empty());
    assert!(state.has_annotations());
    let before = state.clone();
    assert!(state.has_annotations());
    assert_eq!(state, before, "presence query must be read-only");
    state.apply_at(C::PreviousStep, now).unwrap();
    state.apply_at(C::EraseStrokeAt((0.2, 0.3)), now).unwrap();
    assert!(
        !state.has_annotations(),
        "an empty retained page entry is not ink"
    );
    state
        .apply_at(C::UpdateLaserPointer(Some((0.4, 0.5))), now)
        .unwrap();
    assert!(state.has_annotations());
    state.apply_at(C::UpdateLaserPointer(None), now).unwrap();
    assert!(!state.has_annotations());
}
#[test]
fn reload_applies_to_latest_timer_blank_and_ink_without_reset() {
    let mut idle = PresentationState::idle();
    let before_idle = idle.clone();
    assert_eq!(
        idle.commit_reload(
            DocumentDescriptor::identity(DocumentGeneration(1), 1).unwrap(),
            PhysicalPage(0),
            &[]
        ),
        Err(CoreError::NotReady {
            stage: SessionStage::Idle
        })
    );
    assert_eq!(idle, before_idle);
    let now = Instant::now();
    let mut state = PresentationState::new(3).unwrap();
    state
        .apply_at(
            C::StartPresentation {
                audience_available: true,
            },
            now,
        )
        .unwrap();
    state
        .apply_at(C::PauseTimer, now + std::time::Duration::from_secs(7))
        .unwrap();
    state
        .apply_at(C::ToggleBlank(BlankMode::White), now)
        .unwrap();
    state
        .apply_at(C::AppendStrokePoint((0.2, 0.3)), now)
        .unwrap();
    let elapsed = state.elapsed_at(now + std::time::Duration::from_secs(20));
    let descriptor = DocumentDescriptor::identity(DocumentGeneration(2), 2).unwrap();
    let before = state.clone();
    assert!(
        state
            .commit_reload(
                descriptor.clone(),
                PhysicalPage(0),
                &[(PhysicalPage(0), PhysicalPage(9))]
            )
            .is_err()
    );
    assert_eq!(state, before);
    assert_eq!(
        state
            .commit_reload(
                descriptor,
                PhysicalPage(1),
                &[(PhysicalPage(0), PhysicalPage(1))]
            )
            .unwrap(),
        0
    );
    assert_eq!(state.stage(), SessionStage::Presenting);
    assert_eq!(
        state.elapsed_at(now + std::time::Duration::from_secs(40)),
        elapsed
    );
    assert_eq!(state.audience_snapshot().blank_mode, Some(BlankMode::White));
    assert_eq!(state.audience_snapshot().strokes[0].points, [(0.2, 0.3)]);
}
#[test]
fn strokes_are_separate_page_local_and_eraser_hits_segments() {
    let now = Instant::now();
    let mut state = PresentationState::new(3).unwrap();
    state
        .apply_at(
            C::BeginStroke {
                color: [10, 20, 30, 128],
                width: 0.02,
                is_highlighter: true,
            },
            now,
        )
        .unwrap();
    state
        .apply_at(C::AppendStrokePoint((0.1, 0.2)), now)
        .unwrap();
    state
        .apply_at(C::AppendStrokePoint((0.9, 0.2)), now)
        .unwrap();
    state.apply_at(C::EndStroke, now).unwrap();
    state
        .apply_at(
            C::BeginStroke {
                color: [0, 0, 255, 255],
                width: 0.01,
                is_highlighter: false,
            },
            now,
        )
        .unwrap();
    state
        .apply_at(C::AppendStrokePoint((0.4, 0.7)), now)
        .unwrap();
    state.apply_at(C::EndStroke, now).unwrap();
    assert_eq!(state.audience_snapshot().strokes.len(), 2);
    state.apply_at(C::NextStep, now).unwrap();
    assert!(state.audience_snapshot().strokes.is_empty());
    state.apply_at(C::PreviousStep, now).unwrap();
    assert_eq!(state.audience_snapshot().strokes.len(), 2);
    state.apply_at(C::EraseStrokeAt((0.5, 0.2)), now).unwrap();
    assert_eq!(state.audience_snapshot().strokes.len(), 1);
    assert!(!state.audience_snapshot().strokes[0].is_highlighter);
    state.apply_at(C::ClearStrokes, now).unwrap();
    assert!(state.audience_snapshot().strokes.is_empty());
}
#[test]
fn invalid_pointer_values_are_atomic_and_settings_bounds_match() {
    let now = Instant::now();
    let mut state = PresentationState::new(2).unwrap();
    for command in [
        C::UpdateLaserPointer(Some((f32::NAN, 0.))),
        C::AppendStrokePoint((1.1, 0.)),
        C::EraseStrokeAt((-0.1, 0.)),
        C::BeginStroke {
            color: [0; 4],
            width: f32::INFINITY,
            is_highlighter: false,
        },
        C::SetLaserStyle(LaserStyle {
            color_bgra: [0; 4],
            radius_norm: 0.26,
        }),
    ] {
        let before = state.clone();
        assert_eq!(state.apply_at(command, now), Err(CoreError::InvalidPointer));
        assert_eq!(state, before);
    }
    for radius in [0.0001, 0.25] {
        state
            .apply_at(
                C::SetLaserStyle(LaserStyle {
                    color_bgra: [1, 2, 3, 4],
                    radius_norm: radius,
                }),
                now,
            )
            .unwrap();
        assert_eq!(state.audience_snapshot().laser_style.radius_norm, radius);
    }
}
#[test]
fn navigation_ends_gesture_and_reload_cannot_reuse_old_annotations() {
    let now = Instant::now();
    let mut state = PresentationState::new(3).unwrap();
    state
        .apply_at(C::AppendStrokePoint((0.1, 0.1)), now)
        .unwrap();
    state
        .apply_at(C::UpdateLaserPointer(Some((0.2, 0.2))), now)
        .unwrap();
    state.apply_at(C::NextStep, now).unwrap();
    assert!(state.audience_snapshot().laser_pos_norm.is_none());
    state
        .apply_at(C::AppendStrokePoint((0.5, 0.5)), now)
        .unwrap();
    assert_eq!(state.audience_snapshot().strokes[0].points, [(0.5, 0.5)]);
    state
        .apply_at(
            C::CommitDocument(DocumentDescriptor::identity(DocumentGeneration(2), 3).unwrap()),
            now,
        )
        .unwrap();
    assert!(state.audience_snapshot().strokes.is_empty());
}
