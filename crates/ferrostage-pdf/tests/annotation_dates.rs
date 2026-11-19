#![cfg(feature = "pdfium")]

use pdfium_render::prelude::*;

#[test]
fn annotation_dates_round_trip_through_pdfium() -> Result<(), PdfiumError> {
    let dll = std::env::var_os("FERROSTAGE_TEST_PDFIUM")
        .expect("native suite requires FERROSTAGE_TEST_PDFIUM");
    let pdfium = Pdfium::new(Pdfium::bind_to_library(dll)?);
    let mut document = pdfium.create_new_pdf()?;
    let mut page = document
        .pages_mut()
        .create_page_at_end(PdfPagePaperSize::a4())?;
    let mut note = page
        .annotations_mut()
        .create_text_annotation("Date regression")?;
    note.set_creation_date("2000-02-29T18:30:00+05:30".parse().unwrap())?;
    note.set_modification_date("2038-01-19T03:14:08.123Z".parse().unwrap())?;
    assert_eq!(
        note.creation_date().as_deref(),
        Some("D:20000229130000Z00'00'")
    );
    assert_eq!(
        note.modification_date().as_deref(),
        Some("D:20380119031408Z00'00'")
    );
    Ok(())
}
