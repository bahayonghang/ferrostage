#![cfg(feature = "pdfium")]
use ferrostage_pdf::{PdfError, RuntimeConfig};
use ferrostage_render::{CacheBudgets, RenderError, RenderHandle};
#[test]
fn missing_library_is_typed_before_any_successful_initialization() {
    let config = RuntimeConfig {
        library_path: std::path::PathBuf::from("/__ferrostage_nonexistent__/missing-pdfium.dll"),
        font_directories: vec![],
    };
    assert!(matches!(
        RenderHandle::spawn(config, CacheBudgets::default()),
        Err(RenderError::Pdf(PdfError::LibraryUnavailable))
    ));
    let valid = RuntimeConfig {
        library_path: std::env::var_os("FERROSTAGE_TEST_PDFIUM")
            .expect("native suite requires FERROSTAGE_TEST_PDFIUM")
            .into(),
        font_directories: vec![
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/pdf"),
        ],
    };
    let bad_font = RuntimeConfig {
        font_directories: vec![std::path::PathBuf::from(
            "/__ferrostage_nonexistent__/fonts",
        )],
        ..valid.clone()
    };
    assert!(matches!(
        RenderHandle::spawn(bad_font, CacheBudgets::default()),
        Err(RenderError::Pdf(PdfError::InvalidFontDirectory))
    ));
    RenderHandle::spawn(valid, CacheBudgets::default())
        .unwrap()
        .shutdown()
        .unwrap();
}
