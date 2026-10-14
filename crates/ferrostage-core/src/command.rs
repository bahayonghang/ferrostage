use std::sync::Arc;

use crate::ids::{DocumentGeneration, PhysicalPage};
use crate::snapshot::BlankMode;

/// Inputs the presenter sends into [`crate::PresentationState`].
///
/// `StageShadowDocument` carries bytes so the caller can hand a validated
/// shadow file to the document layer. The state machine only checks that the
/// generation increased. It does not parse or retain the bytes.
#[derive(Clone, Debug)]
pub enum PresentationCommand {
    NextStep,
    PreviousStep,
    NextLogicalSlide,
    PreviousLogicalSlide,
    JumpPreview(PhysicalPage),
    CommitJump,
    CancelJump,
    ToggleBlank(BlankMode),
    UpdateLaserPointer(Option<(f32, f32)>),
    AppendStrokePoint((f32, f32)),
    ClearStrokes,
    SwapScreens,
    StageShadowDocument {
        new_generation: DocumentGeneration,
        bytes: Arc<Vec<u8>>,
    },
}

impl PresentationCommand {
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::NextStep => "next_step",
            Self::PreviousStep => "previous_step",
            Self::NextLogicalSlide => "next_logical_slide",
            Self::PreviousLogicalSlide => "previous_logical_slide",
            Self::JumpPreview(_) => "jump_preview",
            Self::CommitJump => "commit_jump",
            Self::CancelJump => "cancel_jump",
            Self::ToggleBlank(_) => "toggle_blank",
            Self::UpdateLaserPointer(_) => "update_laser_pointer",
            Self::AppendStrokePoint(_) => "append_stroke_point",
            Self::ClearStrokes => "clear_strokes",
            Self::SwapScreens => "swap_screens",
            Self::StageShadowDocument { .. } => "stage_shadow_document",
        }
    }
}
