use thiserror::Error;

/// Recoverable state-machine failures. Library code returns these; it does not panic.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum CoreError {
    #[error("document has no pages")]
    EmptyDocument,

    #[error("page {page} is outside document length {len}")]
    PageOutOfRange { page: usize, len: usize },

    #[error("shadow generation {next} is not newer than {current}")]
    StaleGeneration { current: u64, next: u64 },

    #[error("shadow document is empty")]
    EmptyShadow,
}
