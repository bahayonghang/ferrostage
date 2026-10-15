use std::sync::Arc;
use std::time::{Duration, Instant};

use tracing::debug;

use crate::command::PresentationCommand;
use crate::error::CoreError;
use crate::geometry::{NormalizedRect, NormalizedStroke};
use crate::ids::{DocumentGeneration, LogicalSlide, PhysicalPage};
use crate::snapshot::{AudienceProjectionSnapshot, AudienceStage, BlankMode};
use crate::timer::TimerState;
use crate::{DocumentDescriptor, SessionStage, SlideMap};

const DEFAULT_STROKE_COLOR_BGRA: [u8; 4] = [0, 0, 255, 255];
const DEFAULT_STROKE_WIDTH_NORM: f32 = 0.004;

/// Presenter-side state. Fields stay private so the audience window can only
/// observe [`AudienceProjectionSnapshot`].
#[derive(Clone, Debug, PartialEq)]
pub struct PresentationState {
    stage: SessionStage,
    timer: TimerState,
    generation: DocumentGeneration,
    page_count: usize,
    slide_map: Option<SlideMap>,
    current_page: PhysicalPage,
    logical_slide: LogicalSlide,
    slide_crop: NormalizedRect,
    blank_mode: Option<BlankMode>,
    laser_pos_norm: Option<(f32, f32)>,
    strokes: Vec<NormalizedStroke>,
    jump_preview: Option<PhysicalPage>,
}

impl PresentationState {
    /// Prepare the first page without starting a presentation or a timer.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::EmptyDocument`] when `page_count` is zero.
    pub fn new(page_count: usize) -> Result<Self, CoreError> {
        let mut state = Self::idle();
        state.load_document(DocumentGeneration(1), page_count)?;
        Ok(state)
    }

    #[must_use]
    pub fn idle() -> Self {
        Self {
            stage: SessionStage::Idle,
            timer: TimerState::default(),
            generation: DocumentGeneration(0),
            page_count: 0,
            slide_map: None,
            current_page: PhysicalPage(0),
            logical_slide: LogicalSlide(0),
            slide_crop: NormalizedRect::FULL,
            blank_mode: None,
            laser_pos_norm: None,
            strokes: Vec::new(),
            jump_preview: None,
        }
    }

    #[must_use]
    pub fn stage(&self) -> SessionStage {
        self.stage
    }

    /// Compute elapsed time from an app-supplied monotonic timestamp.
    #[must_use]
    pub fn elapsed_at(&self, now: Instant) -> Duration {
        self.timer.elapsed_at(now)
    }

    #[must_use]
    pub fn is_timer_paused(&self) -> bool {
        self.stage == SessionStage::Presenting && self.timer.is_paused()
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
    pub fn jump_preview(&self) -> Option<PhysicalPage> {
        self.jump_preview
    }

    #[must_use]
    pub fn slide_crop(&self) -> NormalizedRect {
        self.slide_crop
    }

    /// Set the audience crop. Beamer notes pages use [`NormalizedRect::LEFT_HALF`].
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::InvalidDocumentCrop`] without mutation for invalid geometry.
    pub fn set_slide_crop(&mut self, crop: NormalizedRect) -> Result<(), CoreError> {
        if !crop.is_inside_unit_square() {
            return Err(CoreError::InvalidDocumentCrop);
        }
        self.slide_crop = crop;
        Ok(())
    }

    /// Copy the audience-visible projection. Speaker notes are not readable here.
    #[must_use]
    pub fn audience_snapshot(&self) -> AudienceProjectionSnapshot {
        AudienceProjectionSnapshot {
            stage: if self.stage == SessionStage::Presenting {
                AudienceStage::Live
            } else {
                AudienceStage::Curtain
            },
            generation: self.generation,
            current_page: self.current_page,
            slide_crop: self.slide_crop,
            blank_mode: self.blank_mode,
            laser_pos_norm: self.laser_pos_norm,
            strokes: Arc::from(self.strokes.as_slice()),
            visual: None,
        }
    }

    /// Apply one presenter command using the app's monotonic clock sample.
    ///
    /// # Errors
    ///
    /// Invalid stages, missing audience, pages and descriptors return a typed
    /// error without changing state. `SwapScreens` must be handled by the app.
    pub fn apply_at(
        &mut self,
        command: PresentationCommand,
        now: Instant,
    ) -> Result<(), CoreError> {
        debug!(op = command.name(), "apply presentation command");
        let allowed = match &command {
            PresentationCommand::LoadDocument { .. }
            | PresentationCommand::LoadMappedDocument(_) => self.stage != SessionStage::Presenting,
            PresentationCommand::StartPresentation { .. } => self.stage == SessionStage::Ready,
            PresentationCommand::StopPresentation
            | PresentationCommand::PauseTimer
            | PresentationCommand::ResumeTimer
            | PresentationCommand::ResetTimer => self.stage == SessionStage::Presenting,
            PresentationCommand::CloseDocument | PresentationCommand::SwapScreens => true,
            _ => self.stage != SessionStage::Idle,
        };
        if !allowed {
            return Err(CoreError::NotReady { stage: self.stage });
        }
        match command {
            PresentationCommand::LoadMappedDocument(descriptor) => self.load_mapped(descriptor),
            PresentationCommand::ReconfigureDocument(descriptor) => self.reconfigure(descriptor)?,
            PresentationCommand::LoadDocument {
                generation,
                page_count,
            } => {
                self.load_document(generation, page_count)?;
            }
            PresentationCommand::StartPresentation { audience_available } => {
                if !audience_available {
                    return Err(CoreError::NoAudienceDisplay);
                }
                self.timer.start(now);
                self.stage = SessionStage::Presenting;
            }
            PresentationCommand::StopPresentation => {
                self.timer.pause(now);
                self.stage = SessionStage::Ready;
            }
            PresentationCommand::CloseDocument => *self = Self::idle(),
            PresentationCommand::PauseTimer => self.timer.pause(now),
            PresentationCommand::ResumeTimer => self.timer.resume(now),
            PresentationCommand::ResetTimer => self.timer.reset(now),
            PresentationCommand::NextStep => {
                self.step(1, false);
            }
            PresentationCommand::PreviousStep => {
                self.step(-1, false);
            }
            PresentationCommand::NextLogicalSlide => self.step(1, true),
            PresentationCommand::PreviousLogicalSlide => self.step(-1, true),
            PresentationCommand::JumpPreview(page) => self.preview(page)?,
            PresentationCommand::CommitJump => self.commit_jump(),
            PresentationCommand::CancelJump => self.jump_preview = None,
            PresentationCommand::ToggleBlank(mode) => self.toggle_blank(mode),
            PresentationCommand::UpdateLaserPointer(pos) => self.laser_pos_norm = pos,
            PresentationCommand::AppendStrokePoint(point) => self.append_point(point),
            PresentationCommand::ClearStrokes => self.strokes.clear(),
            PresentationCommand::SwapScreens => {
                return Err(CoreError::ScreenSwapRequiresCoordinator);
            }
            PresentationCommand::CommitDocument(descriptor) => self.commit_document(descriptor)?,
            PresentationCommand::StageShadowDocument {
                new_generation,
                bytes,
            } => self.stage_shadow(new_generation, &bytes)?,
        }
        Ok(())
    }

    pub fn slide_map(&self) -> Option<&SlideMap> {
        self.slide_map.as_ref()
    }

    fn step(&mut self, delta: isize, logical: bool) {
        let page = self.jump_preview.unwrap_or(self.current_page);
        let Some(map) = &self.slide_map else {
            return;
        };
        let next = match (logical, delta > 0) {
            (false, true) => map.next_step(page),
            (false, false) => map.previous_step(page),
            (true, true) => map.next_logical(page),
            (true, false) => map.previous_logical(page),
        };
        if let Some(next) = next {
            if self.jump_preview.is_some() {
                self.jump_preview = Some(next);
            } else {
                self.current_page = next;
                self.logical_slide = map.logical_for(next).unwrap_or(LogicalSlide(0));
            }
        }
    }

    fn preview(&mut self, page: PhysicalPage) -> Result<(), CoreError> {
        if page.0 >= self.page_count {
            return Err(CoreError::PageOutOfRange {
                page: page.0,
                len: self.page_count,
            });
        }
        if self
            .slide_map
            .as_ref()
            .and_then(|m| m.logical_for(page))
            .is_none()
        {
            return Err(CoreError::NotContentPage { page: page.0 });
        }
        self.jump_preview = Some(page);
        Ok(())
    }

    fn commit_jump(&mut self) {
        if let Some(page) = self.jump_preview.take() {
            self.current_page = page;
            self.logical_slide = self
                .slide_map
                .as_ref()
                .and_then(|m| m.logical_for(page))
                .unwrap_or(LogicalSlide(0));
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
        let map = self.slide_map.clone().ok_or(CoreError::EmptyDocument)?;
        self.commit_document(DocumentDescriptor::mapped(
            new_generation,
            map,
            self.slide_crop,
        )?)
    }

    fn load_document(
        &mut self,
        generation: DocumentGeneration,
        page_count: usize,
    ) -> Result<(), CoreError> {
        self.load_mapped(DocumentDescriptor::identity(generation, page_count)?);
        Ok(())
    }

    fn load_mapped(&mut self, descriptor: DocumentDescriptor) {
        *self = Self::idle();
        self.stage = SessionStage::Ready;
        self.install(descriptor, PhysicalPage(0));
    }

    fn install(&mut self, descriptor: DocumentDescriptor, position: PhysicalPage) {
        self.generation = descriptor.generation;
        self.page_count = descriptor.page_count();
        self.current_page = descriptor
            .slide_map()
            .closest_content_at_or_before(position);
        self.logical_slide = descriptor
            .slide_map()
            .logical_for(self.current_page)
            .unwrap_or(LogicalSlide(0));
        self.slide_crop = descriptor.slide_crop();
        self.slide_map = Some(descriptor.slide_map().clone());
        self.jump_preview = None;
    }

    fn reconfigure(&mut self, descriptor: DocumentDescriptor) -> Result<(), CoreError> {
        if descriptor.generation != self.generation || descriptor.page_count() != self.page_count {
            return Err(CoreError::InvalidReconfiguration);
        }
        self.install(descriptor, self.current_page);
        Ok(())
    }

    fn commit_document(&mut self, descriptor: DocumentDescriptor) -> Result<(), CoreError> {
        if descriptor.generation <= self.generation {
            return Err(CoreError::StaleGeneration {
                current: self.generation.0,
                next: descriptor.generation.0,
            });
        }
        tracing::info!(
            generation = descriptor.generation.0,
            pages = descriptor.page_count(),
            "committed document descriptor"
        );
        self.install(descriptor, self.current_page);
        Ok(())
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod lifecycle_tests;

#[cfg(test)]
mod property_tests;

#[cfg(test)]
mod mapping_tests;
