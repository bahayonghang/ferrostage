//! Owned boundary used by the render worker; no native handle crosses it.
use std::path::PathBuf;

use ferrostage_core::{BgraFrame, DocumentGeneration, NormalizedRect, PhysicalPage};
use thiserror::Error;

use crate::DocumentMetadata;

pub type BgraBitmap = BgraFrame;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeConfig {
    /// Explicit library file, not a search directory or build-time linker variable.
    pub library_path: PathBuf,
    /// Extra font search directories; empty by default. No fonts are bundled.
    pub font_directories: Vec<PathBuf>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PixelSize {
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum PdfError {
    #[error("PDFium library could not be loaded from the configured path")]
    LibraryUnavailable,
    #[error("a native PDF session is already running")]
    RuntimeBusy,
    #[error("PDFium's process-wide runtime configuration cannot be changed")]
    RuntimeConfigurationChanged,
    #[error("invalid PDFium font directory")]
    InvalidFontDirectory,
    #[error("PDFium could not read this PDF (encrypted or unsupported files are not accepted)")]
    Unreadable,
    #[error("PDF has no pages")]
    EmptyDocument,
    #[error("document generation is not loaded")]
    UnknownGeneration,
    #[error("page index is outside the document")]
    PageOutOfRange,
    #[error("invalid page geometry")]
    InvalidPageGeometry,
    #[error("invalid normalized crop rectangle")]
    InvalidCrop,
    #[error("invalid or excessive raster dimensions")]
    InvalidTarget,
    #[error("PDFium rasterization failed")]
    RenderFailed,
    #[error("PDFium returned an invalid BGRA buffer")]
    InvalidBitmap,
}

/// Worker-only adapter interface. Inputs and outputs are owned, including probe results.
/// The native implementation is constructed only by `with_native_session` on the worker.
pub trait RasterBackend {
    fn probe(&mut self, bytes: Vec<u8>) -> Result<DocumentMetadata, PdfError>;
    fn load(
        &mut self,
        generation: DocumentGeneration,
        bytes: Vec<u8>,
    ) -> Result<DocumentMetadata, PdfError>;
    fn render(
        &mut self,
        generation: DocumentGeneration,
        page: PhysicalPage,
        crop: NormalizedRect,
        target: PixelSize,
    ) -> Result<BgraFrame, PdfError>;
    fn remove(&mut self, generation: DocumentGeneration);
}

/// Fit the crop inside a pixel bounding box while preserving the visible aspect ratio.
pub fn fitted_size(
    width: f32,
    height: f32,
    crop: NormalizedRect,
    target: PixelSize,
) -> Result<PixelSize, PdfError> {
    if !crop.is_inside_unit_square() {
        return Err(PdfError::InvalidCrop);
    }
    if !width.is_finite() || !height.is_finite() || width <= 0.0 || height <= 0.0 {
        return Err(PdfError::InvalidPageGeometry);
    }
    // Bound a single allocation to 256 MiB, including unreasonable caller input.
    if target.width == 0 || target.height == 0 || target.width > 32768 || target.height > 32768 {
        return Err(PdfError::InvalidTarget);
    }
    let w = f64::from(width) * f64::from(crop.w);
    let h = f64::from(height) * f64::from(crop.h);
    let scale = (f64::from(target.width) / w).min(f64::from(target.height) / h);
    let size = PixelSize {
        width: (w * scale).round().max(1.0) as u32,
        height: (h * scale).round().max(1.0) as u32,
    };
    if u64::from(size.width) * u64::from(size.height) * 4 > 256 * 1024 * 1024 {
        return Err(PdfError::InvalidTarget);
    }
    Ok(size)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fits_crop_aspect_and_rejects_invalid_input() {
        let target = PixelSize {
            width: 3840,
            height: 2160,
        };
        assert_eq!(
            fitted_size(640.0, 180.0, NormalizedRect::LEFT_HALF, target).unwrap(),
            target
        );
        assert_eq!(
            fitted_size(100.0, 100.0, NormalizedRect::FULL, target).unwrap(),
            PixelSize {
                width: 2160,
                height: 2160
            }
        );
        assert_eq!(
            fitted_size(
                100.0,
                100.0,
                NormalizedRect {
                    x: f32::NAN,
                    ..NormalizedRect::FULL
                },
                target
            ),
            Err(PdfError::InvalidCrop)
        );
        assert_eq!(
            fitted_size(0.0, 100.0, NormalizedRect::FULL, target),
            Err(PdfError::InvalidPageGeometry)
        );
    }
}
