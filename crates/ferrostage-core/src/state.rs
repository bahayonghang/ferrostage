use std::sync::Arc;

use tracing::debug;

use crate::command::PresentationCommand;
use crate::error::CoreError;
use crate::geometry::{NormalizedRect, NormalizedStroke};
use crate::ids::{DocumentGeneration, LogicalSlide, PhysicalPage};
use crate::snapshot::{AudienceProjectionSnapshot, BlankMode};

const DEFAULT_STROKE_COLOR_BGRA: [u8; 4] = [0, 0, 255, 255];
const DEFAULT_STROKE_WIDTH_NORM: f32 = 0.004;

/// Presenter-side state. Fields stay private so the audience window can only
/// observe [`AudienceProjectionSnapshot`].
#[derive(Clone, Debug)]
pub struct PresentationState {
    generation: DocumentGeneration,
    page_count: usize,
    current_page: PhysicalPage,
    logical_slide: LogicalSlide,
    slide_crop: NormalizedRect,
    blank_mode: Option<BlankMode>,
    laser_pos_norm: Option<(f32, f32)>,
    strokes: Vec<NormalizedStroke>,
    screens_swapped: bool,
    jump_preview: Option<PhysicalPage>,
}

impl PresentationState {
    /// Start on the first page of a document that already has `page_count` pages.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::EmptyDocument`] when `page_count` is zero.
    pub fn new(page_count: usize) -> Result<Self, CoreError> {
        if page_count == 0 {
            return Err(CoreError::EmptyDocument);
        }
        Ok(Self {
            generation: DocumentGeneration(1),
            page_count,
            current_page: PhysicalPage(0),
            logical_slide: LogicalSlide(0),
            slide_crop: NormalizedRect::FULL,
            blank_mode: None,
            laser_pos_norm: None,
            strokes: Vec::new(),
            screens_swapped: false,
            jump_preview: None,
        })
    }

    #[must_use]
    pub fn generation(&self) -> DocumentGeneration {
        self.generation
    }

    #[must_use]
    pub fn page_count(&self) -> usize {
        self.page_count
    }

    #[must_use]
    pub fn current_page(&self) -> PhysicalPage {
        self.current_page
    }

    #[must_use]
    pub fn screens_swapped(&self) -> bool {
        self.screens_swapped
    }

    #[must_use]
    pub fn slide_crop(&self) -> NormalizedRect {
        self.slide_crop
    }

    /// Set the audience crop. Beamer notes pages use [`NormalizedRect::LEFT_HALF`].
    pub fn set_slide_crop(&mut self, crop: NormalizedRect) {
        self.slide_crop = crop;
    }

    /// Copy the audience-visible projection. Speaker notes are not readable here.
    #[must_use]
    pub fn audience_snapshot(&self) -> AudienceProjectionSnapshot {
        AudienceProjectionSnapshot {
            generation: self.generation,
            current_page: self.jump_preview.unwrap_or(self.current_page),
            slide_crop: self.slide_crop,
            blank_mode: self.blank_mode,
            laser_pos_norm: self.laser_pos_norm,
            strokes: Arc::from(self.strokes.as_slice()),
        }
    }

    /// Apply one presenter command.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::PageOutOfRange`] for a preview outside the document,
    /// [`CoreError::StaleGeneration`] when a shadow generation is not strictly
    /// newer, and [`CoreError::EmptyShadow`] when the shadow buffer is empty.
    pub fn apply(&mut self, command: PresentationCommand) -> Result<(), CoreError> {
        debug!(op = command.name(), "apply presentation command");
        match command {
            PresentationCommand::NextStep | PresentationCommand::NextLogicalSlide => {
                self.step(1);
            }
            PresentationCommand::PreviousStep | PresentationCommand::PreviousLogicalSlide => {
                self.step(-1);
            }
            PresentationCommand::JumpPreview(page) => self.preview(page)?,
            PresentationCommand::CommitJump => self.commit_jump(),
            PresentationCommand::CancelJump => self.jump_preview = None,
            PresentationCommand::ToggleBlank(mode) => self.toggle_blank(mode),
            PresentationCommand::UpdateLaserPointer(pos) => self.laser_pos_norm = pos,
            PresentationCommand::AppendStrokePoint(point) => self.append_point(point),
            PresentationCommand::ClearStrokes => self.strokes.clear(),
            PresentationCommand::SwapScreens => self.screens_swapped = !self.screens_swapped,
            PresentationCommand::StageShadowDocument {
                new_generation,
                bytes,
            } => self.stage_shadow(new_generation, &bytes)?,
        }
        Ok(())
    }

    fn step(&mut self, delta: isize) {
        if let Some(preview) = self.jump_preview.take() {
            self.current_page = preview;
            self.logical_slide = LogicalSlide(preview.0);
            return;
        }
        let next = self.current_page.0 as isize + delta;
        let last = self.page_count.saturating_sub(1) as isize;
        let clamped = next.clamp(0, last) as usize;
        self.current_page = PhysicalPage(clamped);
        self.logical_slide = LogicalSlide(clamped);
    }

    fn preview(&mut self, page: PhysicalPage) -> Result<(), CoreError> {
        if page.0 >= self.page_count {
            return Err(CoreError::PageOutOfRange {
                page: page.0,
                len: self.page_count,
            });
        }
        self.jump_preview = Some(page);
        Ok(())
    }

    fn commit_jump(&mut self) {
        if let Some(page) = self.jump_preview.take() {
            self.current_page = page;
            self.logical_slide = LogicalSlide(page.0);
        }
    }

    fn toggle_blank(&mut self, mode: BlankMode) {
        self.blank_mode = if self.blank_mode == Some(mode) {
            None
        } else {
            Some(mode)
        };
    }

    fn append_point(&mut self, point: (f32, f32)) {
        if let Some(stroke) = self.strokes.last_mut() {
            stroke.points.push(point);
            return;
        }
        self.strokes.push(NormalizedStroke {
            points: vec![point],
            color_bgra: DEFAULT_STROKE_COLOR_BGRA,
            width_norm: DEFAULT_STROKE_WIDTH_NORM,
            is_highlighter: false,
        });
    }

    fn stage_shadow(
        &mut self,
        new_generation: DocumentGeneration,
        bytes: &[u8],
    ) -> Result<(), CoreError> {
        if bytes.is_empty() {
            return Err(CoreError::EmptyShadow);
        }
        if new_generation <= self.generation {
            return Err(CoreError::StaleGeneration {
                current: self.generation.0,
                next: new_generation.0,
            });
        }
        tracing::info!(
            generation = new_generation.0,
            bytes = bytes.len(),
            "staged shadow document"
        );
        self.generation = new_generation;
        self.jump_preview = None;
        if self.current_page.0 >= self.page_count {
            self.current_page = PhysicalPage(0);
            self.logical_slide = LogicalSlide(0);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state() -> PresentationState {
        PresentationState::new(4).unwrap()
    }

    #[test]
    fn empty_document_is_rejected() {
        assert_eq!(
            PresentationState::new(0).unwrap_err(),
            CoreError::EmptyDocument
        );
    }

    #[test]
    fn audience_snapshot_tracks_generation_and_has_no_notes_field() {
        let mut presentation = state();
        let before = presentation.audience_snapshot();
        assert_eq!(before.generation, DocumentGeneration(1));
        assert!(before.laser_pos_norm.is_none());
        assert!(before.blank_mode.is_none());

        presentation
            .apply(PresentationCommand::StageShadowDocument {
                new_generation: DocumentGeneration(2),
                bytes: Arc::new(b"%PDF".to_vec()),
            })
            .unwrap();
        let after = presentation.audience_snapshot();
        assert_eq!(after.generation, DocumentGeneration(2));
        let rendered = format!("{after:?}");
        assert!(!rendered.contains("notes"));
    }

    #[test]
    fn blank_toggle_is_shown_on_the_snapshot_and_clears_on_repeat() {
        let mut presentation = state();
        presentation
            .apply(PresentationCommand::ToggleBlank(BlankMode::Black))
            .unwrap();
        assert_eq!(
            presentation.audience_snapshot().blank_mode,
            Some(BlankMode::Black)
        );
        presentation
            .apply(PresentationCommand::ToggleBlank(BlankMode::Black))
            .unwrap();
        assert_eq!(presentation.audience_snapshot().blank_mode, None);
    }

    #[test]
    fn swap_screens_does_not_change_the_audience_page() {
        let mut presentation = state();
        presentation.apply(PresentationCommand::NextStep).unwrap();
        let page = presentation.audience_snapshot().current_page;
        presentation
            .apply(PresentationCommand::SwapScreens)
            .unwrap();
        assert!(presentation.screens_swapped());
        assert_eq!(presentation.audience_snapshot().current_page, page);
        assert_eq!(
            presentation.audience_snapshot().slide_crop,
            NormalizedRect::FULL
        );
    }

    #[test]
    fn preview_out_of_range_is_an_error_and_stale_generation_is_rejected() {
        let mut presentation = state();
        let error = presentation
            .apply(PresentationCommand::JumpPreview(PhysicalPage(9)))
            .unwrap_err();
        assert_eq!(error, CoreError::PageOutOfRange { page: 9, len: 4 });
        let stale = presentation
            .apply(PresentationCommand::StageShadowDocument {
                new_generation: DocumentGeneration(1),
                bytes: Arc::new(vec![1]),
            })
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
        let mut presentation = state();
        presentation
            .apply(PresentationCommand::PreviousStep)
            .unwrap();
        assert_eq!(presentation.current_page(), PhysicalPage(0));
        for _ in 0..10 {
            presentation.apply(PresentationCommand::NextStep).unwrap();
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
}
