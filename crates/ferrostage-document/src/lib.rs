//! Document model.
//!
//! Speaker notes and media descriptors live in this crate. They are never
//! copied into [`ferrostage_core::AudienceProjectionSnapshot`].

use ferrostage_core::{AudienceProjectionSnapshot, PhysicalPage, PresentationState};

/// Presenter-only notes for one physical page. Markdown is stored, not rendered.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SlideNotes {
    pub page: PhysicalPage,
    pub markdown: String,
}

/// A media rectangle discovered in the PDF, in normalized page space.
///
/// Playback itself belongs to `ferrostage-media`. This crate only records where
/// the rectangle is and which bytes or path feed it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MediaSlot {
    pub page: PhysicalPage,
    pub label: String,
}

/// Loaded document metadata. PDF bytes stay in the PDF crate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoadedDocument {
    pub page_count: usize,
    pub notes: Vec<SlideNotes>,
    pub media: Vec<MediaSlot>,
}

impl LoadedDocument {
    #[must_use]
    pub fn notes_for(&self, page: PhysicalPage) -> Option<&str> {
        self.notes
            .iter()
            .find(|notes| notes.page == page)
            .map(|notes| notes.markdown.as_str())
    }

    /// Build the audience snapshot from presenter state.
    ///
    /// `notes` is accepted so callers can see the data is intentionally unused
    /// on this path.
    #[must_use]
    pub fn project(
        state: &PresentationState,
        _notes: Option<&SlideNotes>,
    ) -> AudienceProjectionSnapshot {
        state.audience_snapshot()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn projecting_drops_speaker_notes() {
        let notes = SlideNotes {
            page: PhysicalPage(0),
            markdown: "secret speaker note".to_owned(),
        };
        let document = LoadedDocument {
            page_count: 1,
            notes: vec![notes.clone()],
            media: Vec::new(),
        };
        let state = PresentationState::new(document.page_count).unwrap();
        let snapshot = LoadedDocument::project(&state, Some(&notes));
        let rendered = format!("{snapshot:?}");
        assert!(!rendered.contains("secret speaker note"));
        assert_eq!(
            document.notes_for(PhysicalPage(0)),
            Some("secret speaker note")
        );
    }
}
