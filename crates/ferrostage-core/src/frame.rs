use std::{fmt, sync::Arc};

use crate::CoreError;

/// Immutable, tightly packed BGRA8 pixels, shared without a channel conversion.
#[derive(Clone, PartialEq, Eq)]
pub struct BgraFrame {
    width: u32,
    height: u32,
    stride: usize,
    pixels_bgra: Arc<Vec<u8>>,
}

impl BgraFrame {
    /// Validate geometry before accepting pixels from a rasterizer or decoder.
    pub fn new(
        width: u32,
        height: u32,
        stride: usize,
        pixels_bgra: Vec<u8>,
    ) -> Result<Self, CoreError> {
        if width == 0 || height == 0 {
            return Err(CoreError::InvalidFrameDimensions);
        }
        let packed_stride = usize::try_from(width)
            .ok()
            .and_then(|width| width.checked_mul(4))
            .ok_or(CoreError::InvalidFrameDimensions)?;
        let expected_len = usize::try_from(height)
            .ok()
            .and_then(|height| packed_stride.checked_mul(height))
            .ok_or(CoreError::InvalidFrameDimensions)?;
        if stride != packed_stride {
            return Err(CoreError::InvalidFrameStride {
                expected: packed_stride,
                actual: stride,
            });
        }
        if pixels_bgra.len() != expected_len {
            return Err(CoreError::InvalidFrameLength {
                expected: expected_len,
                actual: pixels_bgra.len(),
            });
        }
        Ok(Self {
            width,
            height,
            stride,
            pixels_bgra: Arc::new(pixels_bgra),
        })
    }

    #[must_use]
    pub fn width(&self) -> u32 {
        self.width
    }

    #[must_use]
    pub fn height(&self) -> u32 {
        self.height
    }

    #[must_use]
    pub fn stride(&self) -> usize {
        self.stride
    }

    #[must_use]
    pub fn pixels_bgra(&self) -> &[u8] {
        &self.pixels_bgra
    }

    /// Pixel allocation size for cache budgets, including unused Vec capacity.
    #[must_use]
    pub fn allocation_bytes(&self) -> usize {
        self.pixels_bgra.capacity()
    }
}

impl fmt::Debug for BgraFrame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BgraFrame")
            .field("width", &self.width)
            .field("height", &self.height)
            .field("stride", &self.stride)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packed_frame_shares_bgra_without_conversion() {
        let pixels = vec![1, 2, 3, 255, 4, 5, 6, 255];
        let original = pixels.as_ptr();
        let frame = BgraFrame::new(2, 1, 8, pixels).unwrap();
        assert_eq!((frame.width(), frame.height(), frame.stride()), (2, 1, 8));
        assert_eq!(frame.pixels_bgra().as_ptr(), original);
        let cloned = frame.clone();
        assert_eq!(cloned.pixels_bgra().as_ptr(), original);
        drop(frame);
        assert_eq!(cloned.pixels_bgra(), &[1, 2, 3, 255, 4, 5, 6, 255]);
        assert!(!format!("{cloned:?}").contains("255"));
    }

    #[test]
    fn invalid_geometry_stride_and_length_are_rejected() {
        for (width, height) in [(0, 1), (1, 0), (u32::MAX, u32::MAX)] {
            assert_eq!(
                BgraFrame::new(width, height, 0, vec![]),
                Err(CoreError::InvalidFrameDimensions)
            );
        }
        assert_eq!(
            BgraFrame::new(1, 1, 8, vec![0; 8]),
            Err(CoreError::InvalidFrameStride {
                expected: 4,
                actual: 8,
            })
        );
        for length in [3, 5] {
            assert_eq!(
                BgraFrame::new(1, 1, 4, vec![0; length]),
                Err(CoreError::InvalidFrameLength {
                    expected: 4,
                    actual: length,
                })
            );
        }
    }
}
