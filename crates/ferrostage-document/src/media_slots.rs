use ferrostage_core::{NormalizedRect, PhysicalPage};
use ferrostage_pdf::structure::MediaAnnotation;
#[derive(Clone, Debug, PartialEq)]
pub struct MediaSlot {
    pub page: PhysicalPage,
    pub annotation: usize,
    pub media: MediaAnnotation,
    /// Rectangle relative to the selected slide crop; None when entirely outside it.
    pub slide_rect: Option<NormalizedRect>,
    /// Public intersection in source-media coordinates. Decode/crop off the UI thread.
    pub source_crop: Option<NormalizedRect>,
    /// Physical annotation aspect ratio before the slide/notes intersection.
    pub aspect_ratio: Option<f32>,
}
pub(crate) fn in_crop(rect: NormalizedRect, crop: NormalizedRect) -> Option<NormalizedRect> {
    let x = rect.x.max(crop.x);
    let y = rect.y.max(crop.y);
    let right = (rect.x + rect.w).min(crop.x + crop.w);
    let bottom = (rect.y + rect.h).min(crop.y + crop.h);
    if right <= x || bottom <= y {
        return None;
    }
    Some(NormalizedRect {
        x: (x - crop.x) / crop.w,
        y: (y - crop.y) / crop.h,
        w: (right - x) / crop.w,
        h: (bottom - y) / crop.h,
    })
}

pub(crate) fn source_crop(rect: NormalizedRect, crop: NormalizedRect) -> Option<NormalizedRect> {
    in_crop(crop, rect)
}
