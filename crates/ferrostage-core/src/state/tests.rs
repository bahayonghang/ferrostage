use super::*;

fn presenting_state() -> PresentationState {
    let mut state = PresentationState::new(4).unwrap();
    state
        .apply_at(
            PresentationCommand::StartPresentation {
                audience_available: true,
            },
            Instant::now(),
        )
        .unwrap();
    state
}

#[test]
fn empty_document_is_rejected() {
    assert_eq!(
        PresentationState::new(0).unwrap_err(),
        CoreError::EmptyDocument
    );
}

#[test]
fn preview_does_not_change_the_audience_page() {
    let mut presentation = presenting_state();
    presentation
        .apply_at(
            PresentationCommand::JumpPreview(PhysicalPage(2)),
            Instant::now(),
        )
        .unwrap();
    assert_eq!(
        presentation.audience_snapshot().current_page,
        PhysicalPage(0)
    );
    presentation
        .apply_at(PresentationCommand::CancelJump, Instant::now())
        .unwrap();
    assert_eq!(presentation.current_page(), PhysicalPage(0));
}

#[test]
fn preview_navigation_requires_explicit_commit() {
    let mut presentation = presenting_state();
    presentation
        .apply_at(
            PresentationCommand::JumpPreview(PhysicalPage(1)),
            Instant::now(),
        )
        .unwrap();
    presentation
        .apply_at(PresentationCommand::NextStep, Instant::now())
        .unwrap();
    assert_eq!(presentation.current_page(), PhysicalPage(0));
    presentation
        .apply_at(PresentationCommand::CommitJump, Instant::now())
        .unwrap();
    assert_eq!(presentation.current_page(), PhysicalPage(2));
}

#[test]
fn preview_steps_clamp_in_both_directions_and_cancel_keeps_committed_page() {
    let now = Instant::now();
    let mut state = presenting_state();
    state
        .apply_at(PresentationCommand::JumpPreview(PhysicalPage(1)), now)
        .unwrap();
    for _ in 0..6 {
        state
            .apply_at(PresentationCommand::NextLogicalSlide, now)
            .unwrap();
    }
    assert_eq!(state.jump_preview(), Some(PhysicalPage(3)));
    for _ in 0..6 {
        state
            .apply_at(PresentationCommand::PreviousLogicalSlide, now)
            .unwrap();
    }
    assert_eq!(state.jump_preview(), Some(PhysicalPage(0)));
    state.apply_at(PresentationCommand::NextStep, now).unwrap();
    state
        .apply_at(PresentationCommand::CancelJump, now)
        .unwrap();
    assert_eq!(state.current_page(), PhysicalPage(0));
    assert_eq!(state.jump_preview(), None);
}

#[test]
fn audience_snapshot_tracks_generation_and_has_no_notes_field() {
    let mut presentation = presenting_state();
    let before = presentation.audience_snapshot();
    assert_eq!(before.generation, DocumentGeneration(1));
    assert!(before.laser_pos_norm.is_none());
    assert!(before.blank_mode.is_none());

    presentation
        .apply_at(
            PresentationCommand::StageShadowDocument {
                new_generation: DocumentGeneration(2),
                bytes: Arc::new(b"%PDF".to_vec()),
            },
            Instant::now(),
        )
        .unwrap();
    let after = presentation.audience_snapshot();
    assert_eq!(after.generation, DocumentGeneration(2));
    let rendered = format!("{after:?}");
    assert!(!rendered.contains("notes"));
}

#[test]
fn blank_toggle_is_shown_on_the_snapshot_and_clears_on_repeat() {
    let mut presentation = presenting_state();
    presentation
        .apply_at(
            PresentationCommand::ToggleBlank(BlankMode::Black),
            Instant::now(),
        )
        .unwrap();
    assert_eq!(
        presentation.audience_snapshot().blank_mode,
        Some(BlankMode::Black)
    );
    presentation
        .apply_at(
            PresentationCommand::ToggleBlank(BlankMode::Black),
            Instant::now(),
        )
        .unwrap();
    assert_eq!(presentation.audience_snapshot().blank_mode, None);
}

#[test]
fn swap_screens_does_not_change_the_audience_page() {
    let mut presentation = presenting_state();
    presentation
        .apply_at(PresentationCommand::NextStep, Instant::now())
        .unwrap();
    let page = presentation.audience_snapshot().current_page;
    let error = presentation
        .apply_at(PresentationCommand::SwapScreens, Instant::now())
        .unwrap_err();
    assert_eq!(error, CoreError::ScreenSwapRequiresCoordinator);
    assert_eq!(presentation.audience_snapshot().current_page, page);
    assert_eq!(
        presentation.audience_snapshot().slide_crop,
        NormalizedRect::FULL
    );
}

#[test]
fn preview_out_of_range_is_an_error_and_stale_generation_is_rejected() {
    let mut presentation = presenting_state();
    let error = presentation
        .apply_at(
            PresentationCommand::JumpPreview(PhysicalPage(9)),
            Instant::now(),
        )
        .unwrap_err();
    assert_eq!(error, CoreError::PageOutOfRange { page: 9, len: 4 });
    let stale = presentation
        .apply_at(
            PresentationCommand::StageShadowDocument {
                new_generation: DocumentGeneration(1),
                bytes: Arc::new(vec![1]),
            },
            Instant::now(),
        )
        .unwrap_err();
    assert_eq!(
        stale,
        CoreError::StaleGeneration {
            current: 1,
            next: 1
        }
    );
}

#[test]
fn step_clamps_at_both_ends() {
    let mut presentation = presenting_state();
    presentation
        .apply_at(PresentationCommand::PreviousStep, Instant::now())
        .unwrap();
    assert_eq!(presentation.current_page(), PhysicalPage(0));
    for _ in 0..10 {
        presentation
            .apply_at(PresentationCommand::NextStep, Instant::now())
            .unwrap();
    }
    assert_eq!(presentation.current_page(), PhysicalPage(3));
}

#[test]
fn halves_cover_the_unit_square_without_overlap() {
    assert!(NormalizedRect::FULL.is_inside_unit_square());
    assert!(NormalizedRect::LEFT_HALF.is_inside_unit_square());
    assert!(NormalizedRect::RIGHT_HALF.is_inside_unit_square());
    let left = std::hint::black_box(NormalizedRect::LEFT_HALF);
    let right = std::hint::black_box(NormalizedRect::RIGHT_HALF);
    assert!(left.x + left.w <= right.x + f32::EPSILON);
}
