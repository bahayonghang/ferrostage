//! Pure media control, bounded process protocol and Rust animation decoding.
//! The native backend is a separate executable, never a GUI dependency.
pub mod animation;
pub mod protocol;
mod supervisor;
use ferrostage_core::{BgraFrame, DocumentGeneration, PhysicalPage};
pub use supervisor::{Controller, Runtime, Source, Start, Update};

pub const MAX_WIDTH: u32 = 1920;
pub const MAX_HEIGHT: u32 = 2160;
pub const MAX_SOURCE_PIXELS: u64 = 64 * 1024 * 1024;
pub const MAX_SOURCE_BYTES: u64 = 256 * 1024 * 1024;
pub const MAX_ANIMATION_BYTES: usize = 256 * 1024 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Identity {
    pub generation: DocumentGeneration,
    pub page: PhysicalPage,
    pub slot: u64,
    pub epoch: u64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SharedFrame {
    pub identity: Identity,
    pub sequence: u64,
    pub pixels: BgraFrame,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum MediaError {
    #[error("media runtime is unavailable")]
    RuntimeUnavailable,
    #[error("media source is missing or unreadable")]
    SourceUnavailable,
    #[error("media format or codec is unsupported")]
    Unsupported,
    #[error("media geometry or memory limit exceeded")]
    Limit,
    #[error("invalid media stream protocol")]
    Protocol,
    #[error("media process failed")]
    Process,
    #[error("media operation timed out")]
    Timeout,
    #[error("media operation was cancelled")]
    Cancelled,
}
pub fn output_size(width: u32, height: u32, max_height: u32) -> Result<(u32, u32), MediaError> {
    if width == 0
        || height == 0
        || u64::from(width) * u64::from(height) > MAX_SOURCE_PIXELS
        || !(1..=MAX_HEIGHT).contains(&max_height)
    {
        return Err(MediaError::Limit);
    }
    let scale = (f64::from(MAX_WIDTH) / f64::from(width))
        .min(f64::from(max_height) / f64::from(height))
        .min(1.);
    Ok((
        (f64::from(width) * scale).floor().max(1.) as u32,
        (f64::from(height) * scale).floor().max(1.) as u32,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn limits_keep_proportion_and_reject_unbounded_input() {
        assert_eq!(output_size(3840, 2160, 1080), Ok((1920, 1080)));
        assert_eq!(output_size(100, 1000, 200), Ok((20, 200)));
        assert_eq!(
            output_size(u32::MAX, u32::MAX, 1080),
            Err(MediaError::Limit)
        );
        assert_eq!(output_size(0, 1, 1080), Err(MediaError::Limit));
    }
}
