use std::sync::Arc;

use crate::geometry::{NormalizedRect, NormalizedStroke};
use crate::ids::{DocumentGeneration, PhysicalPage};
use crate::{BgraFrame, CoreError};

/// The audience window is absent outside Presenting; its projection is Curtain.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AudienceStage {
    Curtain,
    Live,
}

/// Pixels authorized for one media slot, without a source path or descriptor.
#[derive(Clone, Debug, PartialEq)]
pub struct AudienceMediaFrame {
    // Public opaque playback-instance identity, not a PDF annotation index.
    slot_id: u64,
    rect_norm: NormalizedRect,
    pixels: BgraFrame,
}

impl AudienceMediaFrame {
    pub fn new(
        slot_id: u64,
        rect_norm: NormalizedRect,
        pixels: BgraFrame,
    ) -> Result<Self, CoreError> {
        if !rect_norm.is_inside_unit_square() {
            return Err(CoreError::InvalidMediaRect);
        }
        Ok(Self {
            slot_id,
            rect_norm,
            pixels,
        })
    }

    #[must_use]
    pub fn slot_id(&self) -> u64 {
        self.slot_id
    }

    #[must_use]
    pub fn rect_norm(&self) -> NormalizedRect {
        self.rect_norm
    }

    #[must_use]
    pub fn pixels(&self) -> &BgraFrame {
        &self.pixels
    }
}

/// Only the projection assembler may select page/crop/generation-matched pixels.
/// These fields cannot carry notes, file handles, media sources or cache access.
#[derive(Clone, Debug, PartialEq)]
pub struct AudienceVisual {
    /// UI-authorized total GPU bytes, including retained and candidate images.
    pub texture_budget_bytes: usize,
    pub page: BgraFrame,
    pub media: Vec<AudienceMediaFrame>,
}
/// Public candidate pixels authorized only for invisible pre-upload. This is
/// never a new committed projection until the assembler publishes its identity.
#[derive(Clone, Debug, PartialEq)]
pub struct AudienceStagingFrame {
    pub attempt: u64,
    pub generation: DocumentGeneration,
    pub current_page: PhysicalPage,
    pub slide_crop: NormalizedRect,
    pub pixels: BgraFrame,
    pub texture_budget_bytes: usize,
}

/// Audience-window blank color. `None` on the snapshot means the slide is visible.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlankMode {
    Black,
    White,
}

/// Everything the audience window is allowed to draw.
///
/// This type has no speaker-notes field. Presenter-only data must not be copied
/// into it. Both windows observe one snapshot; they do not share a mutable
/// document handle.
#[derive(Clone, Debug, PartialEq)]
pub struct AudienceProjectionSnapshot {
    pub playback: Option<crate::AudiencePlayback>,
    pub stage: AudienceStage,
    pub generation: DocumentGeneration,
    pub current_page: PhysicalPage,
    pub slide_crop: NormalizedRect,
    pub blank_mode: Option<BlankMode>,
    pub laser_style: crate::LaserStyle,
    pub laser_pos_norm: Option<(f32, f32)>,
    pub strokes: Arc<[NormalizedStroke]>,
    pub visual: Option<AudienceVisual>,
    pub staging: Option<AudienceStagingFrame>,
}
