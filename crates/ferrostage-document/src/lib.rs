//! Presenter-owned document model. No notes, source paths, or mappings enter the audience snapshot.
mod builder;
mod layout;
mod media_slots;
mod notes;
mod reload;
pub use builder::{DocumentError, DocumentWarning, LoadedDocument};
pub use ferrostage_pdf::structure::{
    DiagnosticKind, MediaKind, MediaRepeat, MediaSource, MediaTime, StructureDiagnostic,
};
pub use layout::{DocumentOverrides, LayoutSource, NotesLayout, notes_crop, slide_crop};
pub use media_slots::MediaSlot;
pub use notes::{NotesMapping, SlideNotes};
pub use reload::ReloadMapping;
