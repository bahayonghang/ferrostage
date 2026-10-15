use ferrostage_core::PhysicalPage;

/// Owned metadata in visible, intrinsically rotated page coordinates.
#[derive(Clone, Debug, PartialEq)]
pub struct PageMetadata {
    pub page: PhysicalPage,
    pub width_points: f32,
    pub height_points: f32,
    pub rotation_degrees: u16,
    pub label: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DocumentMetadata {
    pub pages: Vec<PageMetadata>,
}
