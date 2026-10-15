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
