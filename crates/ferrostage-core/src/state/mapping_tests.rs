use super::*;
use PresentationCommand::*;

fn mapped(generation: u64) -> DocumentDescriptor {
    DocumentDescriptor::mapped(
        DocumentGeneration(generation),
        SlideMap::new(
            7,
            vec![
                vec![PhysicalPage(1), PhysicalPage(3)],
                vec![PhysicalPage(5)],
            ],
        )
        .unwrap(),
        NormalizedRect::LEFT_HALF,
    )
    .unwrap()
}
#[test]
fn mapped_navigation_preview_shadow_and_reconfiguration_preserve_lifecycle() {
    let now = Instant::now();
    let mut state = PresentationState::idle();
    state.apply_at(LoadMappedDocument(mapped(1)), now).unwrap();
    assert_eq!(state.current_page(), PhysicalPage(1));
    state
        .apply_at(
            StartPresentation {
                audience_available: true,
            },
            now,
        )
        .unwrap();
    state.apply_at(JumpPreview(PhysicalPage(3)), now).unwrap();
    state.apply_at(NextLogicalSlide, now).unwrap();
    assert_eq!(state.jump_preview(), Some(PhysicalPage(5)));
    assert_eq!(state.audience_snapshot().current_page, PhysicalPage(1));
    state.apply_at(CommitJump, now).unwrap();
    let before = state.clone();
    assert_eq!(
        state.apply_at(JumpPreview(PhysicalPage(4)), now),
        Err(CoreError::NotContentPage { page: 4 })
    );
    assert_eq!(state, before);
    assert!(state.apply_at(LoadMappedDocument(mapped(2)), now).is_err());
    assert_eq!(state, before);
    state
        .apply_at(
            StageShadowDocument {
                new_generation: DocumentGeneration(2),
                bytes: Arc::new(vec![1]),
            },
            now,
        )
        .unwrap();
    assert_eq!(state.slide_map(), before.slide_map());
    assert_eq!(state.slide_crop(), NormalizedRect::LEFT_HALF);
    let descriptor = DocumentDescriptor::mapped(
        DocumentGeneration(2),
        SlideMap::new(7, vec![vec![PhysicalPage(0)], vec![PhysicalPage(2)]]).unwrap(),
        NormalizedRect::FULL,
    )
    .unwrap();
    state
        .apply_at(ReconfigureDocument(descriptor), now)
        .unwrap();
    assert_eq!(state.current_page(), PhysicalPage(2));
    assert_eq!(state.stage(), SessionStage::Presenting);
    assert_eq!(
        state.elapsed_at(now + Duration::from_secs(5)),
        Duration::from_secs(5)
    );
    let before = state.clone();
    for descriptor in [
        mapped(1),
        DocumentDescriptor::identity(DocumentGeneration(2), 3).unwrap(),
    ] {
        assert_eq!(
            state.apply_at(ReconfigureDocument(descriptor), now),
            Err(CoreError::InvalidReconfiguration)
        );
        assert_eq!(state, before);
    }
    state.apply_at(CommitDocument(mapped(3)), now).unwrap();
    assert_eq!(state.current_page(), PhysicalPage(1));
    assert_eq!(
        state.elapsed_at(now + Duration::from_secs(6)),
        Duration::from_secs(6)
    );
}
#[test]
fn checked_descriptor_rejects_empty_and_invalid_crop() {
    assert_eq!(
        DocumentDescriptor::identity(DocumentGeneration(1), 0),
        Err(CoreError::EmptyDocument)
    );
    let crop = NormalizedRect {
        x: f32::NAN,
        ..NormalizedRect::FULL
    };
    assert_eq!(
        DocumentDescriptor::mapped(DocumentGeneration(1), SlideMap::identity(1).unwrap(), crop),
        Err(CoreError::InvalidDocumentCrop)
    );
}

#[test]
fn reconfigure_clears_preview_and_preserves_paused_or_ready_timer() {
    let now = Instant::now();
    let mut state = PresentationState::idle();
    state.apply_at(LoadMappedDocument(mapped(1)), now).unwrap();
    state
        .apply_at(
            StartPresentation {
                audience_available: true,
            },
            now,
        )
        .unwrap();
    state.apply_at(NextStep, now).unwrap();
    state
        .apply_at(PauseTimer, now + Duration::from_secs(3))
        .unwrap();
    for stop in [false, true] {
        if stop {
            state
                .apply_at(StopPresentation, now + Duration::from_secs(8))
                .unwrap();
        }
        state.apply_at(JumpPreview(PhysicalPage(5)), now).unwrap();
        let before = state.clone();
        assert_eq!(
            state.apply_at(ReconfigureDocument(mapped(2)), now),
            Err(CoreError::InvalidReconfiguration)
        );
        assert_eq!(state, before);
        state.apply_at(ReconfigureDocument(mapped(1)), now).unwrap();
        assert_eq!(state.current_page(), PhysicalPage(3));
        assert_eq!(state.jump_preview(), None);
        assert_eq!(state.stage(), before.stage());
        assert_eq!(state.is_timer_paused(), before.is_timer_paused());
        assert_eq!(
            state.elapsed_at(now + Duration::from_secs(20)),
            Duration::from_secs(3)
        );
    }
}

#[test]
fn legacy_crop_setter_rejects_invalid_geometry_without_mutation() {
    let mut state = PresentationState::new(2).unwrap();
    let before = state.clone();
    for crop in [
        NormalizedRect {
            x: f32::NAN,
            ..NormalizedRect::FULL
        },
        NormalizedRect {
            w: 0.0,
            ..NormalizedRect::FULL
        },
        NormalizedRect {
            h: f32::INFINITY,
            ..NormalizedRect::FULL
        },
        NormalizedRect {
            x: -0.1,
            ..NormalizedRect::FULL
        },
        NormalizedRect {
            w: 1.1,
            ..NormalizedRect::FULL
        },
    ] {
        assert_eq!(
            state.set_slide_crop(crop),
            Err(CoreError::InvalidDocumentCrop)
        );
        assert_eq!(state, before);
    }
    state.set_slide_crop(NormalizedRect::LEFT_HALF).unwrap();
    assert_eq!(state.slide_crop(), NormalizedRect::LEFT_HALF);
}
