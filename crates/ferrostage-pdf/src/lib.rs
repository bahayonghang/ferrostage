//! PDF backend boundary.
//!
//! The default build links `lopdf` 0.45 for structure checks only. PDFium
//! (`pdfium-render` 0.9.4) is optional and driven by the single render worker.
//! `hayro-syntax` 0.8.0 remains unlinked.
//! This crate does not open a window and does not convert BGRA to RGBA.

use thiserror::Error;

mod metadata;
pub mod structure;
pub use structure::{DocumentStructure, extract_structure};
pub mod raster;
pub use metadata::{DocumentMetadata, PageMetadata};
pub use raster::{BgraBitmap, PdfError, PixelSize, RuntimeConfig};
#[cfg(feature = "pdfium")]
mod native;
#[cfg(feature = "pdfium")]
#[doc(hidden)]
pub use native::with_native_session;

/// Versions the rest of the workspace should pin. Kept as constants so a docs
/// change and the crate graph cannot drift silently.
pub const LOPDF_VERSION: &str = "0.45.0";
pub const PDFIUM_RENDER_VERSION: &str = "0.9.4";
pub const HAYRO_SYNTAX_VERSION: &str = "0.8.0";

/// Pixel layout the raster worker will hand to GPUI. No channel swap.
pub const RASTER_PIXEL_FORMAT: &str = "BGRA8";

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PdfStructureError {
    #[error("lopdf could not parse the buffer")]
    Unreadable,
    #[error("invalid or cyclic page geometry")]
    InvalidPageGeometry,
    #[error("non-default UserUnit is outside the structure geometry subset")]
    UnsupportedUserUnit,
}

/// Returns `Ok(())` when `lopdf` accepts the buffer as a PDF.
///
/// # Errors
///
/// Returns [`PdfStructureError::Unreadable`] when the bytes are not a PDF
/// `lopdf` can load. This is a structure check, not a render.
pub fn parse_structure(bytes: &[u8]) -> Result<(), PdfStructureError> {
    lopdf::Document::load_mem(bytes)
        .map(|_| ())
        .map_err(|_| PdfStructureError::Unreadable)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_buffer_is_not_a_pdf() {
        assert_eq!(parse_structure(b""), Err(PdfStructureError::Unreadable));
    }

    #[test]
    fn version_pins_match_the_workspace_plan() {
        assert_eq!(LOPDF_VERSION, "0.45.0");
        assert_eq!(PDFIUM_RENDER_VERSION, "0.9.4");
        assert_eq!(HAYRO_SYNTAX_VERSION, "0.8.0");
        assert_eq!(RASTER_PIXEL_FORMAT, "BGRA8");
    }
}
