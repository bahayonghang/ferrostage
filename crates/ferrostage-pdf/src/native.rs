use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
};

use ferrostage_core::{BgraFrame, DocumentGeneration, NormalizedRect, PhysicalPage};
use pdfium_render::prelude::*;

use crate::{
    DocumentMetadata, PageMetadata, PdfError, PixelSize, RuntimeConfig,
    raster::{RasterBackend, fitted_size},
};

// pdfium-render 0.9.4 retains its bindings in a process-wide OnceCell. Do not
// pretend that constructing a second Pdfium changes the library or font paths.
static SESSION: Mutex<()> = Mutex::new(());
static CONFIG: OnceLock<RuntimeConfig> = OnceLock::new();

/// Infrastructure entry point for ferrostage-render, called inside its sole worker.
pub fn with_native_session<T>(
    mut config: RuntimeConfig,
    run: impl FnOnce(&mut dyn RasterBackend) -> T,
) -> Result<T, PdfError> {
    let _guard = SESSION.try_lock().map_err(|_| PdfError::RuntimeBusy)?;
    // Windows DLL loading must not reinterpret a relative slash-separated path
    // as a library search name. Normalize explicit paths before binding.
    config.library_path = config
        .library_path
        .canonicalize()
        .map_err(|_| PdfError::LibraryUnavailable)?;
    config.font_directories = config
        .font_directories
        .iter()
        .map(|path| {
            path.canonicalize()
                .map_err(|_| PdfError::InvalidFontDirectory)
        })
        .collect::<Result<_, _>>()?;
    if CONFIG.get().is_some_and(|current| current != &config) {
        return Err(PdfError::RuntimeConfigurationChanged);
    }
    let paths = config
        .font_directories
        .iter()
        .map(|path| {
            if !path.is_dir() {
                return Err(PdfError::InvalidFontDirectory);
            }
            path.to_str().ok_or(PdfError::InvalidFontDirectory)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let options = PdfiumLibraryConfig::new()
        .set_renderer_anti_grain_geometry()
        .set_user_font_paths(&paths)
        .map_err(|_| PdfError::InvalidFontDirectory)?;
    let bindings =
        Pdfium::bind_to_library(&config.library_path).map_err(|_| PdfError::LibraryUnavailable)?;
    let pdfium = Pdfium::new_with_config(bindings, options);
    CONFIG.get_or_init(|| config);
    let mut backend = NativeBackend {
        pdfium: &pdfium,
        documents: HashMap::new(),
    };
    Ok(run(&mut backend))
}

struct NativeBackend<'a> {
    pdfium: &'a Pdfium,
    documents: HashMap<DocumentGeneration, PdfDocument<'a>>,
}

fn metadata(document: &PdfDocument<'_>) -> Result<DocumentMetadata, PdfError> {
    if document.pages().is_empty() {
        return Err(PdfError::EmptyDocument);
    }
    let pages = (0..document.pages().len())
        .map(|index| {
            let page = document
                .pages()
                .get(index)
                .map_err(|_| PdfError::Unreadable)?;
            let width_points = page.width().value;
            let height_points = page.height().value;
            if !width_points.is_finite()
                || !height_points.is_finite()
                || width_points <= 0.0
                || height_points <= 0.0
            {
                return Err(PdfError::InvalidPageGeometry);
            }
            Ok(PageMetadata {
                page: PhysicalPage(index as usize),
                width_points,
                height_points,
                rotation_degrees: page
                    .rotation()
                    .map_err(|_| PdfError::InvalidPageGeometry)?
                    .as_degrees() as u16,
                label: page.label().map(str::to_owned),
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(DocumentMetadata { pages })
}

impl RasterBackend for NativeBackend<'_> {
    fn probe(&mut self, bytes: Vec<u8>) -> Result<DocumentMetadata, PdfError> {
        let document = self
            .pdfium
            .load_pdf_from_byte_vec(bytes, None)
            .map_err(|_| PdfError::Unreadable)?;
        metadata(&document)
    }

    fn load(
        &mut self,
        generation: DocumentGeneration,
        bytes: Vec<u8>,
    ) -> Result<DocumentMetadata, PdfError> {
        let document = self
            .pdfium
            .load_pdf_from_byte_vec(bytes, None)
            .map_err(|_| PdfError::Unreadable)?;
        let metadata = metadata(&document)?;
        self.documents.insert(generation, document);
        Ok(metadata)
    }

    fn render(
        &mut self,
        generation: DocumentGeneration,
        page: PhysicalPage,
        crop: NormalizedRect,
        target: PixelSize,
    ) -> Result<BgraFrame, PdfError> {
        let document = self
            .documents
            .get(&generation)
            .ok_or(PdfError::UnknownGeneration)?;
        let index = i32::try_from(page.0).map_err(|_| PdfError::PageOutOfRange)?;
        let page = document
            .pages()
            .get(index)
            .map_err(|_| PdfError::PageOutOfRange)?;
        let size = fitted_size(page.width().value, page.height().value, crop, target)?;
        let scale_x = size.width as f32 / (page.width().value * crop.w);
        let scale_y = size.height as f32 / (page.height().value * crop.h);
        // WithMatrix applies a device-space transform to PDFium's intrinsic
        // CropBox/rotation/top-left page transform. Never combine set_origin here.
        let config = PdfRenderConfig::new()
            .set_fixed_size(size.width as i32, size.height as i32)
            .set_format(PdfBitmapFormat::BGRA)
            .set_reverse_byte_order(false)
            .clear_before_rendering(false)
            .transform(
                scale_x,
                0.0,
                0.0,
                scale_y,
                -(crop.x * page.width().value * scale_x),
                -(crop.y * page.height().value * scale_y),
            )
            .map_err(|_| PdfError::InvalidCrop)?
            .clip(0, 0, size.width as i32, size.height as i32);
        let stride = size.width as usize * 4;
        let rows = size.height as usize;
        // The checked safe wrapper borrows this owned BGRA Vec only until the
        // bitmap is dropped. Moving the Vec into BgraFrame then preserves its
        // pixel allocation without a full-frame export or Vec-to-slice copy.
        let mut pixels = vec![255; stride * rows];
        {
            let mut bitmap = PdfBitmap::from_bytes(
                size.width as i32,
                size.height as i32,
                PdfBitmapFormat::BGRA,
                &mut pixels,
            )
            .map_err(|_| PdfError::InvalidBitmap)?;
            page.render_into_bitmap_with_config(&mut bitmap, &config)
                .map_err(|_| PdfError::RenderFailed)?;
            if bitmap.format().map_err(|_| PdfError::InvalidBitmap)? != PdfBitmapFormat::BGRA {
                return Err(PdfError::InvalidBitmap);
            }
        }
        BgraFrame::new(size.width, size.height, stride, pixels).map_err(|_| PdfError::InvalidBitmap)
    }

    fn remove(&mut self, generation: DocumentGeneration) {
        self.documents.remove(&generation);
    }
}
