/// Rectangle in normalized page space. Origin is the top-left of the page.
///
/// `x` and `y` are the origin. `w` and `h` are the size. Each component is a
/// fraction of the full page, so [`NormalizedRect::FULL`] is the unit square.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NormalizedRect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl NormalizedRect {
    pub const TOP_HALF: Self = Self {
        x: 0.0,
        y: 0.0,
        w: 1.0,
        h: 0.5,
    };
    pub const BOTTOM_HALF: Self = Self {
        x: 0.0,
        y: 0.5,
        w: 1.0,
        h: 0.5,
    };
    pub const FULL: Self = Self {
        x: 0.0,
        y: 0.0,
        w: 1.0,
        h: 1.0,
    };

    pub const LEFT_HALF: Self = Self {
        x: 0.0,
        y: 0.0,
        w: 0.5,
        h: 1.0,
    };

    pub const RIGHT_HALF: Self = Self {
        x: 0.5,
        y: 0.0,
        w: 0.5,
        h: 1.0,
    };

    /// True when every component is finite and the rectangle lies inside the unit square.
    #[must_use]
    pub fn is_inside_unit_square(self) -> bool {
        self.x.is_finite()
            && self.y.is_finite()
            && self.w.is_finite()
            && self.h.is_finite()
            && self.w > 0.0
            && self.h > 0.0
            && self.x >= 0.0
            && self.y >= 0.0
            && self.x + self.w <= 1.0 + f32::EPSILON
            && self.y + self.h <= 1.0 + f32::EPSILON
    }
}

/// One pointer stroke in normalized page space.
///
/// `color_bgra` is B, G, R, A so it matches the PDFium bitmap layout without
/// a channel swap on the audience path.
#[derive(Clone, Debug, PartialEq)]
pub struct NormalizedStroke {
    pub points: Vec<(f32, f32)>,
    pub color_bgra: [u8; 4],
    pub width_norm: f32,
    pub is_highlighter: bool,
}

/// Hit test whole strokes, including the space between sampled points.
pub(crate) fn stroke_hit(stroke: &NormalizedStroke, point: (f32, f32)) -> bool {
    let radius = (stroke.width_norm * 0.5).max(0.015);
    let distance = |a: (f32, f32), b: (f32, f32)| (a.0 - b.0).hypot(a.1 - b.1);
    stroke.points.iter().any(|p| distance(*p, point) <= radius)
        || stroke.points.windows(2).any(|segment| {
            let (a, b) = (segment[0], segment[1]);
            let (dx, dy) = (b.0 - a.0, b.1 - a.1);
            let length = dx * dx + dy * dy;
            let t = if length > 0.0 {
                ((point.0 - a.0) * dx + (point.1 - a.1) * dy) / length
            } else {
                0.0
            }
            .clamp(0.0, 1.0);
            distance((a.0 + t * dx, a.1 + t * dy), point) <= radius
        })
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LaserStyle {
    pub color_bgra: [u8; 4],
    pub radius_norm: f32,
}
impl Default for LaserStyle {
    fn default() -> Self {
        Self {
            color_bgra: [0, 0, 255, 255],
            radius_norm: 0.008,
        }
    }
}
