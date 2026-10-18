use crate::{CoreError, DocumentGeneration, NormalizedRect, SlideMap};

/// Loading a document prepares the presenter; only an explicit start goes live.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SessionStage {
    Idle,
    Ready,
    Presenting,
}

/// Validated document metadata supplied by the document/render coordinator.
///
/// Core checks page count and generation before applying it. PDF bytes, handles,
/// speaker notes and file paths are deliberately absent.
#[derive(Clone, Debug, PartialEq)]
pub struct DocumentDescriptor {
    pub generation: DocumentGeneration,
    slide_map: SlideMap,
    slide_crop: NormalizedRect,
}

impl DocumentDescriptor {
    /// Explicit compatibility path for metadata-only consumers.
    pub fn identity(generation: DocumentGeneration, page_count: usize) -> Result<Self, CoreError> {
        Self::mapped(
            generation,
            SlideMap::identity(page_count)?,
            NormalizedRect::FULL,
        )
    }
    pub fn mapped(
        generation: DocumentGeneration,
        slide_map: SlideMap,
        slide_crop: NormalizedRect,
    ) -> Result<Self, CoreError> {
        if !slide_crop.is_inside_unit_square() {
            return Err(CoreError::InvalidDocumentCrop);
        }
        Ok(Self {
            generation,
            slide_map,
            slide_crop,
        })
    }
    pub fn page_count(&self) -> usize {
        self.slide_map.page_count()
    }
    pub fn slide_map(&self) -> &SlideMap {
        &self.slide_map
    }
    pub fn slide_crop(&self) -> NormalizedRect {
        self.slide_crop
    }
}
