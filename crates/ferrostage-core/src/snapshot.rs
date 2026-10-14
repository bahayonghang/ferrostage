use std::sync::Arc;

use crate::geometry::{NormalizedRect, NormalizedStroke};
use crate::ids::{DocumentGeneration, PhysicalPage};

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
    pub generation: DocumentGeneration,
    pub current_page: PhysicalPage,
    pub slide_crop: NormalizedRect,
    pub blank_mode: Option<BlankMode>,
    pub laser_pos_norm: Option<(f32, f32)>,
    pub strokes: Arc<[NormalizedStroke]>,
}
