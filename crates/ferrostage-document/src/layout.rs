use ferrostage_core::NormalizedRect;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NotesLayout {
    #[default]
    None,
    Left,
    Right,
    Top,
    Bottom,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayoutSource {
    Unsplit,
    AspectRatio,
    Manual,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DocumentOverrides {
    pub layout: Option<NotesLayout>,
    pub identify_independent_notes: bool,
    pub physical_pages: bool,
}
impl Default for DocumentOverrides {
    fn default() -> Self {
        Self {
            layout: None,
            identify_independent_notes: true,
            physical_pages: false,
        }
    }
}
pub fn slide_crop(layout: NotesLayout) -> NormalizedRect {
    match layout {
        NotesLayout::None => NormalizedRect::FULL,
        NotesLayout::Left => NormalizedRect::RIGHT_HALF,
        NotesLayout::Right => NormalizedRect::LEFT_HALF,
        NotesLayout::Top => NormalizedRect::BOTTOM_HALF,
        NotesLayout::Bottom => NormalizedRect::TOP_HALF,
    }
}
pub fn notes_crop(layout: NotesLayout) -> Option<NormalizedRect> {
    match layout {
        NotesLayout::None => None,
        NotesLayout::Left => Some(NormalizedRect::LEFT_HALF),
        NotesLayout::Right => Some(NormalizedRect::RIGHT_HALF),
        NotesLayout::Top => Some(NormalizedRect::TOP_HALF),
        NotesLayout::Bottom => Some(NormalizedRect::BOTTOM_HALF),
    }
}
