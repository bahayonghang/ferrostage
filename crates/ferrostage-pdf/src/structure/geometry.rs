use super::*;
use lopdf::ObjectId;
use std::collections::HashSet;

/// Visible CropBox intersected with MediaBox, before intrinsic clockwise rotation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PageGeometry {
    pub bounds: [f32; 4],
    pub rotation_degrees: u16,
}
impl PageGeometry {
    pub fn visible_size(self) -> (f32, f32) {
        let [l, b, r, t] = self.bounds;
        if self.rotation_degrees == 90 || self.rotation_degrees == 270 {
            (t - b, r - l)
        } else {
            (r - l, t - b)
        }
    }
    pub fn normalize_rect(self, rect: [f32; 4]) -> Option<NormalizedRect> {
        if rect.iter().any(|v| !v.is_finite()) {
            return None;
        }
        let [l, b, r, t] = self.bounds;
        if self.bounds.iter().any(|v| !v.is_finite()) || r <= l || t <= b {
            return None;
        }
        let left = rect[0].min(rect[2]).max(l);
        let right = rect[0].max(rect[2]).min(r);
        let bottom = rect[1].min(rect[3]).max(b);
        let top = rect[1].max(rect[3]).min(t);
        if right <= left || top <= bottom {
            return None;
        }
        let x = (left - l) / (r - l);
        let y = (t - top) / (t - b);
        let w = (right - left) / (r - l);
        let h = (top - bottom) / (t - b);
        let rect = match self.rotation_degrees {
            0 => NormalizedRect { x, y, w, h },
            90 => NormalizedRect {
                x: 1.0 - y - h,
                y: x,
                w: h,
                h: w,
            },
            180 => NormalizedRect {
                x: 1.0 - x - w,
                y: 1.0 - y - h,
                w,
                h,
            },
            270 => NormalizedRect {
                x: y,
                y: 1.0 - x - w,
                w: h,
                h: w,
            },
            _ => return None,
        };
        Some(NormalizedRect {
            x: rect.x.max(0.0),
            y: rect.y.max(0.0),
            ..rect
        })
    }
}
fn inherited<'a>(
    document: &'a Document,
    mut id: ObjectId,
    key: &[u8],
) -> Result<Option<&'a Object>, PdfStructureError> {
    let mut seen = HashSet::new();
    loop {
        if !seen.insert(id) {
            return Err(PdfStructureError::InvalidPageGeometry);
        }
        let dict = document
            .get_dictionary(id)
            .map_err(|_| PdfStructureError::InvalidPageGeometry)?;
        if let Ok(object) = dict.get(key) {
            return resolve(document, object)
                .map(Some)
                .ok_or(PdfStructureError::InvalidPageGeometry);
        }
        match dict.get(b"Parent") {
            Ok(parent) => {
                id = parent
                    .as_reference()
                    .map_err(|_| PdfStructureError::InvalidPageGeometry)?
            }
            Err(_) => return Ok(None),
        }
    }
}
fn rectangle(object: &Object) -> Option<[f32; 4]> {
    let a = object.as_array().ok()?;
    if a.len() != 4 {
        return None;
    }
    let mut r = [0.0; 4];
    for (dst, src) in r.iter_mut().zip(a) {
        *dst = number(src)? as f32;
    }
    (r.iter().all(|v| v.is_finite()) && r[2] > r[0] && r[3] > r[1]).then_some(r)
}
pub(super) fn extract(
    document: &Document,
    id: ObjectId,
) -> Result<PageGeometry, PdfStructureError> {
    let error = || PdfStructureError::InvalidPageGeometry;
    let media = inherited(document, id, b"MediaBox")?
        .and_then(rectangle)
        .ok_or_else(error)?;
    let crop = match inherited(document, id, b"CropBox")? {
        Some(o) => rectangle(o).ok_or_else(error)?,
        None => media,
    };
    let bounds = [
        media[0].max(crop[0]),
        media[1].max(crop[1]),
        media[2].min(crop[2]),
        media[3].min(crop[3]),
    ];
    if bounds[2] <= bounds[0] || bounds[3] <= bounds[1] {
        return Err(error());
    }
    let rotate = match inherited(document, id, b"Rotate")? {
        Some(o) => o.as_i64().map_err(|_| error())?,
        None => 0,
    };
    if rotate % 90 != 0 {
        return Err(error());
    }
    // UserUnit support requires agreement with the native metadata contract.
    let dictionary = document.get_dictionary(id).map_err(|_| error())?;
    if let Ok(unit) = dictionary.get(b"UserUnit")
        && resolve(document, unit).and_then(number) != Some(1.0)
    {
        return Err(PdfStructureError::UnsupportedUserUnit);
    }
    Ok(PageGeometry {
        bounds,
        rotation_degrees: rotate.rem_euclid(360) as u16,
    })
}
