use crate::PdfStructureError;
use ferrostage_core::{NormalizedRect, PhysicalPage};
use lopdf::{Dictionary, Document, Object};
mod active_content;
mod geometry;
mod media;
mod outline;
mod presentation;
mod source;
pub use geometry::PageGeometry;
pub use outline::{PdfOutline, PdfOutlineEntry, PdfOutlineIssue};

#[derive(Clone, Debug, PartialEq)]
pub struct DocumentStructure {
    pub pages: Vec<PageStructure>,
    pub diagnostics: Vec<StructureDiagnostic>,
    pub outline: PdfOutline,
}
#[derive(Clone, Debug, PartialEq)]
pub struct PageStructure {
    pub presentation: ferrostage_core::PagePresentation,
    pub page: PhysicalPage,
    pub geometry: PageGeometry,
    pub media: Vec<MediaAnnotation>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MediaSource {
    External(String),
    Embedded {
        name: Option<String>,
        bytes: Vec<u8>,
    },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MediaKind {
    Movie,
    Screen,
    RichMedia,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MediaTime {
    Seconds(f64),
    Unscaled(i64),
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MediaRepeat {
    Count(f64),
    Forever,
}
#[derive(Clone, Debug, PartialEq)]
pub struct MediaAnnotation {
    pub kind: MediaKind,
    pub rect: Option<NormalizedRect>,
    pub source: Option<MediaSource>,
    pub start: Option<MediaTime>,
    pub duration: Option<MediaTime>,
    pub repeat: Option<MediaRepeat>,
    /// Page-open/visible activation, not merely play-on-click.
    pub autostart: Option<bool>,
    pub play_on_activation: Option<bool>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiagnosticKind {
    Missing,
    Malformed,
    Unsupported,
    UnscaledTime,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StructureDiagnostic {
    pub page: PhysicalPage,
    pub annotation: Option<usize>,
    pub field: String,
    pub kind: DiagnosticKind,
}

/// Parse owned metadata only. Never follows media paths, executes actions, or writes a PDF.
pub fn extract_structure(bytes: &[u8]) -> Result<DocumentStructure, PdfStructureError> {
    let document = Document::load_mem(bytes).map_err(|_| PdfStructureError::Unreadable)?;
    let mut result = DocumentStructure {
        pages: Vec::new(),
        diagnostics: Vec::new(),
        outline: outline::extract(&document),
    };
    active_content::document(&document, &mut result.diagnostics);
    for (index, (_, id)) in document.get_pages().into_iter().enumerate() {
        let page = PhysicalPage(index);
        let dictionary = document
            .get_dictionary(id)
            .map_err(|_| PdfStructureError::Unreadable)?;
        let geometry = geometry::extract(&document, id)?;
        active_content::page(&document, page, dictionary, &mut result.diagnostics);
        let mut entries = Vec::new();
        if let Ok(annots) = dictionary.get(b"Annots") {
            if let Some(array) = resolve(&document, annots).and_then(|o| o.as_array().ok()) {
                for (annotation, object) in array.iter().enumerate() {
                    let mut parser = Parser {
                        document: &document,
                        page,
                        annotation: Some(annotation),
                        diagnostics: &mut result.diagnostics,
                    };
                    let Some(dict) = resolve(&document, object).and_then(|o| o.as_dict().ok())
                    else {
                        parser.warn("Annots", DiagnosticKind::Malformed);
                        continue;
                    };
                    if let Some(item) =
                        media::extract(&mut parser, dict, object, dictionary, geometry)
                    {
                        entries.push(item);
                    }
                }
            } else {
                result.diagnostics.push(StructureDiagnostic {
                    page,
                    annotation: None,
                    field: "Annots".into(),
                    kind: DiagnosticKind::Malformed,
                });
            }
        }
        result.pages.push(PageStructure {
            presentation: presentation::extract(
                &mut Parser {
                    document: &document,
                    page,
                    annotation: None,
                    diagnostics: &mut result.diagnostics,
                },
                dictionary,
            ),
            page,
            geometry,
            media: entries,
        });
    }
    Ok(result)
}
fn resolve<'a>(document: &'a Document, object: &'a Object) -> Option<&'a Object> {
    document.dereference(object).ok().map(|(_, o)| o)
}
struct Parser<'a> {
    document: &'a Document,
    page: PhysicalPage,
    annotation: Option<usize>,
    diagnostics: &'a mut Vec<StructureDiagnostic>,
}
impl<'a> Parser<'a> {
    fn warn(&mut self, field: &str, kind: DiagnosticKind) {
        self.diagnostics.push(StructureDiagnostic {
            page: self.page,
            annotation: self.annotation,
            field: field.into(),
            kind,
        });
    }
    fn field(&self, dict: &'a Dictionary, key: &[u8]) -> Option<&'a Object> {
        resolve(self.document, dict.get(key).ok()?)
    }
    fn dict(&self, dict: &'a Dictionary, key: &[u8]) -> Option<&'a Dictionary> {
        self.field(dict, key)?.as_dict().ok()
    }
    fn name(&self, dict: &'a Dictionary, key: &[u8]) -> Option<&'a [u8]> {
        self.field(dict, key)?.as_name().ok()
    }
    fn number(&self, dict: &'a Dictionary, key: &[u8]) -> Option<f64> {
        number(self.field(dict, key)?)
    }
}
fn number(object: &Object) -> Option<f64> {
    let value = match object {
        Object::Integer(v) => *v as f64,
        Object::Real(v) => f64::from(*v),
        _ => return None,
    };
    value.is_finite().then_some(value)
}
fn text(object: &Object) -> Option<String> {
    lopdf::decode_text_string(object).ok()
}

#[cfg(test)]
mod tests;
