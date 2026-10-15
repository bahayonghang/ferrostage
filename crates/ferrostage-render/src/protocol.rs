use ferrostage_core::{BgraFrame, DocumentGeneration, NormalizedRect, PhysicalPage};
use ferrostage_pdf::{PdfError, PixelSize};
use std::time::Duration;
use thiserror::Error;

/// Lower values dispatch first. An executing PDFium call is not preempted.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RenderPriority {
    Audience,
    Presenter,
    Prefetch,
    Preview,
    Thumbnail,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderRequest {
    pub generation: DocumentGeneration,
    pub page: PhysicalPage,
    /// Top-left origin in the visible (rotated and CropBox-clipped) page.
    pub crop: NormalizedRect,
    /// Bounding box; raster preserves crop aspect ratio within it.
    pub target_px: PixelSize,
    pub priority: RenderPriority,
    pub epoch: u64,
}
impl RenderRequest {
    /// Construct a fixed-height L3 request from visible-page metadata.
    pub fn thumbnail(
        generation: DocumentGeneration,
        page: &ferrostage_pdf::PageMetadata,
        crop: NormalizedRect,
        epoch: u64,
    ) -> Result<Self, RenderError> {
        if !crop.is_inside_unit_square() {
            return Err(PdfError::InvalidCrop.into());
        }
        let ratio = (page.width_points * crop.w) / (page.height_points * crop.h);
        if !ratio.is_finite() || ratio <= 0.0 {
            return Err(PdfError::InvalidPageGeometry.into());
        }
        // The width is a bounding box: rounding down would constrain the height.
        let width = (ratio * crate::THUMBNAIL_HEIGHT as f32).ceil().max(1.0) as u32;
        if width > 32768 {
            return Err(PdfError::InvalidTarget.into());
        }
        Ok(Self {
            generation,
            page: page.page,
            crop,
            target_px: PixelSize {
                width,
                height: crate::THUMBNAIL_HEIGHT,
            },
            priority: RenderPriority::Thumbnail,
            epoch,
        })
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RequestId(pub u64);
#[derive(Clone, Debug)]
pub struct RenderEvent {
    pub id: RequestId,
    pub request: RenderRequest,
    pub outcome: Result<RenderResult, RenderError>,
}
#[derive(Clone, Debug)]
pub struct RenderResult {
    pub id: RequestId,
    pub request: RenderRequest,
    pub frame: BgraFrame,
    /// Real backend render + owned BGRA export, excluding queue/upload; None on hits.
    pub render_duration: Option<Duration>,
    pub cache_hit: bool,
}
/// Valid only for its issuing worker, until epoch change/abort/reprepare/shutdown.
#[derive(Clone, Debug)]
pub struct CandidateReady {
    pub(crate) worker_identity: std::sync::Arc<()>,
    pub(crate) generation: DocumentGeneration,
    pub(crate) epoch: u64,
    pub(crate) nonce: u64,
    pub frames: Vec<RenderResult>,
}
impl CandidateReady {
    pub fn generation(&self) -> DocumentGeneration {
        self.generation
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GenerationAck {
    pub generation: DocumentGeneration,
}
#[derive(Clone, Copy, Debug)]
pub struct CacheBudgets {
    pub l2_bytes: usize,
    pub l3_bytes: usize,
}
impl Default for CacheBudgets {
    fn default() -> Self {
        Self {
            l2_bytes: crate::L2_CPU_BUDGET_MIB as usize * 1024 * 1024,
            l3_bytes: crate::L3_THUMB_BUDGET_MIB as usize * 1024 * 1024,
        }
    }
}
#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum RenderError {
    #[error(transparent)]
    Pdf(#[from] PdfError),
    #[error("render worker is unavailable")]
    WorkerUnavailable,
    #[error("native PDFium feature is disabled")]
    FeatureDisabled,
    #[error("generation must be strictly newer and may not be reused")]
    StaleGeneration,
    #[error("request generation is not the active document")]
    NotActive,
    #[error("candidate generation does not match")]
    NotCandidate,
    #[error("candidate readiness token is stale or invalid")]
    NotReady,
    #[error("render request was cancelled before publication")]
    Cancelled,
    #[error("cannot retire the active or candidate generation")]
    GenerationInUse,
    #[error("candidate prerender must contain requests from one epoch and generation")]
    InvalidCandidateRequests,
    #[error("thumbnail requests must have target height 180")]
    InvalidThumbnail,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn thumbnail_target_keeps_180_height_even_for_narrow_pages() {
        let page = ferrostage_pdf::PageMetadata {
            page: PhysicalPage(0),
            width_points: 0.6,
            height_points: 100.0,
            rotation_degrees: 0,
            label: None,
        };
        let request =
            RenderRequest::thumbnail(DocumentGeneration(1), &page, NormalizedRect::FULL, 0)
                .unwrap();
        let size = ferrostage_pdf::raster::fitted_size(
            page.width_points,
            page.height_points,
            request.crop,
            request.target_px,
        )
        .unwrap();
        assert_eq!(size.height, crate::THUMBNAIL_HEIGHT);
    }
}
