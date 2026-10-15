use crate::*;
use ferrostage_core::{
    AudienceProjectionSnapshot, CoreError, DocumentDescriptor, DocumentGeneration, LogicalSlide,
    PhysicalPage, PresentationState, SlideMap,
};
use ferrostage_pdf::{DocumentMetadata, DocumentStructure};
use thiserror::Error;
#[derive(Debug, Error, PartialEq, Eq)]
pub enum DocumentError {
    #[error("metadata and structure pages or visible geometry disagree")]
    MetadataMismatch,
    #[error(transparent)]
    Core(#[from] CoreError),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DocumentWarning {
    AmbiguousLayout,
    MixedGeometry,
    OverlayHeuristic,
    OrphanNotes {
        page: PhysicalPage,
    },
    MediaOutsideSlide {
        page: PhysicalPage,
        annotation: usize,
    },
}
#[derive(Clone, Debug, PartialEq)]
pub struct LoadedDocument {
    pub page_count: usize,
    pub labels: Vec<Option<String>>,
    pub layout: NotesLayout,
    pub layout_source: LayoutSource,
    pub slide_map: SlideMap,
    pub overrides: DocumentOverrides,
    pub notes: Vec<SlideNotes>,
    pub notes_mapping: Vec<NotesMapping>,
    pub media: Vec<MediaSlot>,
    pub warnings: Vec<DocumentWarning>,
    pub structure_diagnostics: Vec<StructureDiagnostic>,
}
impl LoadedDocument {
    /// A new load starts with fresh, document-local options.
    pub fn load(
        metadata: &DocumentMetadata,
        structure: &DocumentStructure,
    ) -> Result<Self, DocumentError> {
        Self::build(metadata, structure, DocumentOverrides::default())
    }
    /// The caller owns same-file identity; reuse this only for that document's replacement.
    pub fn reload(
        &self,
        metadata: &DocumentMetadata,
        structure: &DocumentStructure,
    ) -> Result<Self, DocumentError> {
        Self::build(metadata, structure, self.overrides)
    }
    pub fn build(
        metadata: &DocumentMetadata,
        structure: &DocumentStructure,
        overrides: DocumentOverrides,
    ) -> Result<Self, DocumentError> {
        if metadata.pages.len() != structure.pages.len() {
            return Err(DocumentError::MetadataMismatch);
        }
        if metadata.pages.is_empty() {
            return Err(CoreError::EmptyDocument.into());
        }
        for (index, (page, raw)) in metadata.pages.iter().zip(&structure.pages).enumerate() {
            let (width, height) = raw.geometry.visible_size();
            if !width.is_finite()
                || !height.is_finite()
                || width <= 0.0
                || height <= 0.0
                || raw.geometry.bounds.iter().any(|v| !v.is_finite())
                || page.page != PhysicalPage(index)
                || raw.page != page.page
                || !page.width_points.is_finite()
                || !page.height_points.is_finite()
                || page.width_points <= 0.0
                || page.height_points <= 0.0
                || !matches!(page.rotation_degrees, 0 | 90 | 180 | 270)
                || page.rotation_degrees != raw.geometry.rotation_degrees
                || (width - page.width_points).abs() > 0.05
                || (height - page.height_points).abs() > 0.05
            {
                return Err(DocumentError::MetadataMismatch);
            }
        }
        let mut warnings = Vec::new();
        let first = &metadata.pages[0];
        let uniform = metadata.pages.iter().all(|p| {
            (p.width_points - first.width_points).abs() < 0.05
                && (p.height_points - first.height_points).abs() < 0.05
                && p.rotation_degrees == first.rotation_degrees
        });
        let (layout, layout_source) = if let Some(layout) = overrides.layout {
            (layout, LayoutSource::Manual)
        } else if !uniform {
            warnings.push(DocumentWarning::MixedGeometry);
            (NotesLayout::None, LayoutSource::Unsplit)
        } else {
            let ratio = first.width_points / first.height_points;
            let layout = if (3.1..=3.6).contains(&ratio) || (2.4..=2.7).contains(&ratio) {
                NotesLayout::Right
            } else if (0.85..=0.95).contains(&ratio) || (0.63..=0.70).contains(&ratio) {
                NotesLayout::Bottom
            } else {
                NotesLayout::None
            };
            if layout == NotesLayout::None {
                (layout, LayoutSource::Unsplit)
            } else {
                warnings.push(DocumentWarning::AmbiguousLayout);
                (layout, LayoutSource::AspectRatio)
            }
        };
        let labels: Vec<_> = metadata
            .pages
            .iter()
            .map(|p| p.label.clone().filter(|s| !s.is_empty()))
            .collect();
        let mut groups: Vec<Vec<PhysicalPage>> = Vec::new();
        let mut notes_mapping: Vec<NotesMapping> = Vec::new();
        let mut previous: Option<usize> = None;
        for (index, label) in labels.iter().enumerate() {
            let page = PhysicalPage(index);
            if let Some(suffix) = label
                .as_deref()
                .filter(|_| overrides.identify_independent_notes)
                .and_then(|s| s.strip_prefix("notes:"))
            {
                if previous.is_some_and(|prev| labels[prev].as_deref() == Some(suffix)) {
                    if let Some(mapping) = notes_mapping.last_mut() {
                        mapping.notes_pages.push(page);
                    }
                } else {
                    warnings.push(DocumentWarning::OrphanNotes { page });
                }
                continue;
            }
            let grouped = !overrides.physical_pages
                && label.is_some()
                && previous.is_some_and(|prev| labels[prev] == *label);
            if grouped {
                if let Some(group) = groups.last_mut() {
                    group.push(page);
                }
            } else {
                groups.push(vec![page]);
            }
            notes_mapping.push(NotesMapping {
                content_page: page,
                notes_pages: Vec::new(),
            });
            previous = Some(index);
        }
        let slide_map = SlideMap::new(labels.len(), groups)?;
        if slide_map.groups().iter().any(|g| g.len() > 1) {
            warnings.push(DocumentWarning::OverlayHeuristic);
        }
        let mut media = Vec::new();
        for page in &structure.pages {
            for (annotation, entry) in page.media.iter().enumerate() {
                let slide_rect = entry
                    .rect
                    .and_then(|rect| crate::media_slots::in_crop(rect, slide_crop(layout)));
                if slide_rect.is_none() {
                    warnings.push(DocumentWarning::MediaOutsideSlide {
                        page: page.page,
                        annotation,
                    });
                }
                media.push(MediaSlot {
                    page: page.page,
                    annotation,
                    media: entry.clone(),
                    slide_rect,
                });
            }
        }
        Ok(Self {
            page_count: labels.len(),
            labels,
            layout,
            layout_source,
            slide_map,
            overrides,
            notes: Vec::new(),
            notes_mapping,
            media,
            warnings,
            structure_diagnostics: structure.diagnostics.clone(),
        })
    }
    pub fn descriptor(
        &self,
        generation: DocumentGeneration,
    ) -> Result<DocumentDescriptor, CoreError> {
        DocumentDescriptor::mapped(generation, self.slide_map.clone(), slide_crop(self.layout))
    }
    pub fn notes_for(&self, page: PhysicalPage) -> Option<&str> {
        self.notes
            .iter()
            .find(|n| n.page == page)
            .map(|n| n.markdown.as_str())
    }
    pub fn notes_pages_for(&self, page: PhysicalPage) -> &[PhysicalPage] {
        self.notes_mapping
            .iter()
            .find(|n| n.content_page == page)
            .map_or(&[], |n| n.notes_pages.as_slice())
    }
    pub fn label_candidates(&self, label: &str) -> Vec<LogicalSlide> {
        if label.is_empty() {
            return Vec::new();
        }
        self.slide_map
            .groups()
            .iter()
            .enumerate()
            .filter(|(_, g)| self.labels[g[0].0].as_deref() == Some(label))
            .map(|(index, _)| LogicalSlide(index))
            .collect()
    }
    pub fn project(
        state: &PresentationState,
        _notes: Option<&SlideNotes>,
    ) -> AudienceProjectionSnapshot {
        state.audience_snapshot()
    }
}
#[cfg(test)]
mod tests;
