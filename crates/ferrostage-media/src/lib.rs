//! Media playback boundary.
//!
//! Video and GIF frames will be decoded offscreen (libVLC or GStreamer) into
//! BGRA buffers. One `Arc` of the latest frame is shown in both windows.
//! Child window handles are forbidden: they break GPU composition (Airspace).

use std::sync::Arc;

/// One decoded video or animation frame.
///
/// `pixels` is tightly packed BGRA8, top-left origin, `width * height * 4` bytes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SharedFrame {
    pub width: u32,
    pub height: u32,
    pub pixels_bgra: Arc<[u8]>,
}

impl SharedFrame {
    /// Build a frame when the buffer length matches the geometry.
    #[must_use]
    pub fn try_new(width: u32, height: u32, pixels_bgra: Arc<[u8]>) -> Option<Self> {
        let expected = (width as usize)
            .checked_mul(height as usize)?
            .checked_mul(4)?;
        if pixels_bgra.len() != expected {
            return None;
        }
        Some(Self {
            width,
            height,
            pixels_bgra,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_length_must_match_bgra8() {
        let pixels: Arc<[u8]> = Arc::from([1_u8, 2, 3, 4].as_slice());
        assert!(SharedFrame::try_new(1, 1, Arc::clone(&pixels)).is_some());
        assert!(SharedFrame::try_new(2, 1, pixels).is_none());
    }
}
