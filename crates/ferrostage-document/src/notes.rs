use ferrostage_core::PhysicalPage;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SlideNotes {
    pub page: PhysicalPage,
    pub markdown: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NotesMapping {
    pub content_page: PhysicalPage,
    pub notes_pages: Vec<PhysicalPage>,
}
