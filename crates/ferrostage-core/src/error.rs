use thiserror::Error;

use crate::SessionStage;

/// Recoverable state-machine failures. Library code returns these; it does not panic.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum CoreError {
    #[error("pointer coordinates or stroke width are invalid")]
    InvalidPointer,
    #[error("document has no pages")]
    EmptyDocument,

    #[error("reconfiguration must retain generation and physical page count")]
    InvalidReconfiguration,

    #[error("document has no content pages")]
    EmptyContent,
    #[error("content map must contain nonempty groups of strictly increasing in-range pages")]
    InvalidSlideMap,
    #[error("page {page} is not a content page")]
    NotContentPage { page: usize },
    #[error("document crop must lie inside the normalized page")]
    InvalidDocumentCrop,

    #[error("page {page} is outside document length {len}")]
    PageOutOfRange { page: usize, len: usize },

    #[error("shadow generation {next} is not newer than {current}")]
    StaleGeneration { current: u64, next: u64 },

    #[error("shadow document is empty")]
    EmptyShadow,

    #[error("command is not available in session stage {stage:?}")]
    NotReady { stage: SessionStage },

    #[error("no audience display is available")]
    NoAudienceDisplay,

    #[error("screen swapping requires the app role coordinator")]
    ScreenSwapRequiresCoordinator,

    #[error("BGRA frame dimensions must be nonzero and fit in memory")]
    InvalidFrameDimensions,

    #[error("BGRA stride {actual} does not match packed stride {expected}")]
    InvalidFrameStride { expected: usize, actual: usize },

    #[error("BGRA buffer length {actual} does not match expected length {expected}")]
    InvalidFrameLength { expected: usize, actual: usize },

    #[error("media rectangle must lie inside the normalized page")]
    InvalidMediaRect,
}
