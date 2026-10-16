use super::*;
use ferrostage_core::{BlankMode, PresentationCommand as C};
use std::time::Instant;
fn frame(state: &PresentationState) -> ReadyFrame {
    let mut request = request_for(state.generation(), state.current_page(), state.slide_crop());
    request.priority = RenderPriority::Audience;
    ReadyFrame {
        request,
        frame: BgraFrame::new(2, 1, 8, vec![12, 34, 56, 255, 12, 34, 56, 255]).unwrap(),
    }
}
#[test]
fn projection_rejects_every_mismatched_key_and_presenter_notes_priority() {
    let mut state = PresentationState::new(3).unwrap();
    state
        .apply_at(
            C::StartPresentation {
                audience_available: true,
            },
            Instant::now(),
        )
        .unwrap();
    let empty = state.audience_snapshot();
    let good = frame(&state);
    let target = Some(good.request.target_px);
    assert!(
        assemble(empty.clone(), &empty, Some(&good), target, 64 * 1024 * 1024)
            .visual
            .is_some()
    );
    for kind in 0..5 {
        let mut wrong = good.clone();
        match kind {
            0 => wrong.request.generation = DocumentGeneration(9),
            1 => wrong.request.page = PhysicalPage(1),
            2 => wrong.request.crop = NormalizedRect::RIGHT_HALF,
            3 => wrong.request.target_px.width += 1,
            _ => wrong.request.priority = RenderPriority::Presenter,
        };
        assert!(
            assemble(
                empty.clone(),
                &empty,
                Some(&wrong),
                target,
                64 * 1024 * 1024
            )
            .visual
            .is_none()
        );
    }
}
#[test]
fn awaiting_frame_keeps_complete_old_identity_and_immediate_covers() {
    let mut state = PresentationState::new(3).unwrap();
    let now = Instant::now();
    state
        .apply_at(
            C::StartPresentation {
                audience_available: true,
            },
            now,
        )
        .unwrap();
    let good = frame(&state);
    let target = Some(good.request.target_px);
    let old = assemble(
        state.audience_snapshot(),
        &state.audience_snapshot(),
        Some(&good),
        target,
        64 * 1024 * 1024,
    );
    state.apply_at(C::NextStep, now).unwrap();
    state
        .apply_at(C::UpdateLaserPointer(Some((0.4, 0.4))), now)
        .unwrap();
    state
        .apply_at(C::AppendStrokePoint((0.2, 0.3)), now)
        .unwrap();
    let held = assemble(
        state.audience_snapshot(),
        &old,
        None,
        target,
        64 * 1024 * 1024,
    );
    assert_eq!(held.current_page, old.current_page);
    assert_eq!(held.generation, old.generation);
    assert_eq!(held.slide_crop, old.slide_crop);
    assert_eq!(held.visual, old.visual);
    assert!(held.strokes.is_empty());
    assert!(held.laser_pos_norm.is_none());
    state
        .apply_at(C::ToggleBlank(BlankMode::White), now)
        .unwrap();
    assert_eq!(
        assemble(
            state.audience_snapshot(),
            &old,
            None,
            target,
            64 * 1024 * 1024
        )
        .blank_mode,
        Some(BlankMode::White)
    );
    state.apply_at(C::StopPresentation, now).unwrap();
    let curtain = assemble(
        state.audience_snapshot(),
        &old,
        None,
        target,
        64 * 1024 * 1024,
    );
    assert_eq!(curtain.stage, AudienceStage::Curtain);
    assert!(curtain.visual.is_none());
}
#[test]
fn ten_previews_do_not_change_authorized_projection() {
    let now = Instant::now();
    let mut state = PresentationState::new(20).unwrap();
    state
        .apply_at(
            C::StartPresentation {
                audience_available: true,
            },
            now,
        )
        .unwrap();
    let good = frame(&state);
    let target = Some(good.request.target_px);
    let old = assemble(
        state.audience_snapshot(),
        &state.audience_snapshot(),
        Some(&good),
        target,
        64 * 1024 * 1024,
    );
    for page in 1..=10 {
        state
            .apply_at(C::JumpPreview(PhysicalPage(page)), now)
            .unwrap();
        assert_eq!(
            assemble(
                state.audience_snapshot(),
                &old,
                Some(&good),
                target,
                64 * 1024 * 1024
            ),
            old
        );
    }
    state.apply_at(C::CommitJump, now).unwrap();
    assert_eq!(
        assemble(
            state.audience_snapshot(),
            &old,
            Some(&good),
            target,
            64 * 1024 * 1024
        )
        .current_page,
        PhysicalPage(0)
    );
    let committed = frame(&state);
    assert_eq!(
        assemble(
            state.audience_snapshot(),
            &old,
            Some(&committed),
            target,
            64 * 1024 * 1024
        )
        .current_page,
        PhysicalPage(10)
    );
}

#[test]
fn audience_target_respects_its_half_of_the_gpu_budget() {
    let budget = 8 * 1024 * 1024;
    let target = budgeted_target(
        PixelSize {
            width: 3840,
            height: 2160,
        },
        budget,
    );
    assert!(target.width as usize * target.height as usize * 4 <= budget);
    assert!((target.width as f64 / target.height as f64 - 16. / 9.).abs() < 0.002);
}
