use super::*;
use proptest::prelude::*;

proptest! {
    #[test]
    fn arbitrary_commands_preserve_stage_privacy_and_error_atomicity(
        commands in prop::collection::vec((0u8..23, 0usize..8, 0u64..16), 0..80)
    ) {
        let origin = Instant::now();
        let mut state = PresentationState::idle();
        for (index, (op, page, generation)) in commands.into_iter().enumerate() {
            use PresentationCommand::*;
            let descriptor = DocumentDescriptor::identity(DocumentGeneration(generation), page.max(1)).unwrap();
            let command = match op {
                0 => LoadDocument { generation: descriptor.generation, page_count: page },
                1 => StartPresentation { audience_available: true },
                2 => StartPresentation { audience_available: false },
                3 => StopPresentation,
                4 => CloseDocument,
                5 => PauseTimer,
                6 => ResumeTimer,
                7 => ResetTimer,
                8 => NextStep,
                9 => PreviousStep,
                10 => NextLogicalSlide,
                11 => PreviousLogicalSlide,
                12 => JumpTo(PhysicalPage(page)),
                13 => NextStep,
                14 => PreviousStep,
                15 => ToggleBlank(BlankMode::Black),
                16 => ToggleBlank(BlankMode::White),
                17 => UpdateLaserPointer(Some((0.25, 0.75))),
                18 => AppendStrokePoint((0.25, 0.75)),
                19 => ClearStrokes,
                20 => SwapScreens,
                21 => CommitDocument(descriptor),
                _ => StageShadowDocument { new_generation: descriptor.generation, bytes: Arc::new(vec![1; page]) },
            };
            let must_preserve_page = matches!(command, PauseTimer | ResumeTimer | ResetTimer | ToggleBlank(_) | ClearStrokes);
            let before = state.clone();
            let result = state.apply_at(command, origin + Duration::from_secs(index as u64));
            if result.is_err() {
                prop_assert_eq!(&state, &before);
            }
            let snapshot = state.audience_snapshot();
            if state.stage() != SessionStage::Presenting {
                prop_assert_eq!(snapshot.stage, AudienceStage::Curtain);
                prop_assert!(snapshot.visual.is_none());
            } else {
                prop_assert_eq!(snapshot.stage, AudienceStage::Live);
            }
            prop_assert_eq!(snapshot.current_page, state.current_page());
            if must_preserve_page {
                prop_assert_eq!(snapshot.current_page, before.current_page());
            }
        }
    }
}
