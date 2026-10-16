use std::sync::Arc;

use crate::DocumentDescriptor;
use crate::ids::{DocumentGeneration, PhysicalPage};
use crate::snapshot::BlankMode;

/// Inputs the presenter sends into [`crate::PresentationState`].
///
/// `StageShadowDocument` carries bytes so the caller can hand a validated
/// shadow file to the document layer. The state machine only checks that the
/// generation increased. It does not parse or retain the bytes.
#[derive(Clone, Debug)]
pub enum PresentationCommand {
    LoadMappedDocument(DocumentDescriptor),
    ReconfigureDocument(DocumentDescriptor),
    LoadDocument {
        generation: DocumentGeneration,
        page_count: usize,
    },
    StartPresentation {
        audience_available: bool,
    },
    StopPresentation,
    CloseDocument,
    PauseTimer,
    ResumeTimer,
    ResetTimer,
    NextStep,
    PreviousStep,
    NextLogicalSlide,
    PreviousLogicalSlide,
    JumpPreview(PhysicalPage),
    CommitJump,
    CancelJump,
    ToggleBlank(BlankMode),
    SetLaserStyle(crate::LaserStyle),
    UpdateLaserPointer(Option<(f32, f32)>),
    BeginStroke {
        color: [u8; 4],
        width: f32,
        is_highlighter: bool,
    },
    EndStroke,
    EraseStrokeAt((f32, f32)),
    AppendStrokePoint((f32, f32)),
    ClearStrokes,
    /// App intent: the role coordinator handles this before calling core.
    SwapScreens,
    CommitDocument(DocumentDescriptor),
    StageShadowDocument {
        new_generation: DocumentGeneration,
        bytes: Arc<Vec<u8>>,
    },
}

impl PresentationCommand {
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::LoadMappedDocument(_) => "load_mapped_document",
            Self::ReconfigureDocument(_) => "reconfigure_document",
            Self::LoadDocument { .. } => "load_document",
            Self::StartPresentation { .. } => "start_presentation",
            Self::StopPresentation => "stop_presentation",
            Self::CloseDocument => "close_document",
            Self::PauseTimer => "pause_timer",
            Self::ResumeTimer => "resume_timer",
            Self::ResetTimer => "reset_timer",
            Self::NextStep => "next_step",
            Self::PreviousStep => "previous_step",
            Self::NextLogicalSlide => "next_logical_slide",
            Self::PreviousLogicalSlide => "previous_logical_slide",
            Self::JumpPreview(_) => "jump_preview",
            Self::CommitJump => "commit_jump",
            Self::CancelJump => "cancel_jump",
            Self::ToggleBlank(_) => "toggle_blank",
            Self::SetLaserStyle(_) => "set_laser_style",
            Self::UpdateLaserPointer(_) => "update_laser_pointer",
            Self::BeginStroke { .. } => "begin_stroke",
            Self::EndStroke => "end_stroke",
            Self::EraseStrokeAt(_) => "erase_stroke_at",
            Self::AppendStrokePoint(_) => "append_stroke_point",
            Self::ClearStrokes => "clear_strokes",
            Self::SwapScreens => "swap_screens",
            Self::CommitDocument(_) => "commit_document",
            Self::StageShadowDocument { .. } => "stage_shadow_document",
        }
    }
}
