//! Deterministic, bounded GPU composition geometry. No PDF rendering per tick.
use ferrostage_core::{
    BgraFrame, PageTransition, TransitionDimension as Dimension, TransitionDirection as Direction,
    TransitionMotion as Motion, TransitionStyle as Style,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Source {
    Old,
    New,
    Difference,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}
impl Rect {
    pub const FULL: Self = Self {
        x: 0.,
        y: 0.,
        w: 1.,
        h: 1.,
    };
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Layer {
    pub source: Source,
    pub clip: Rect,
    pub image: Rect,
    pub opacity: f32,
}
fn layer(source: Source) -> Layer {
    Layer {
        source,
        clip: Rect::FULL,
        image: Rect::FULL,
        opacity: 1.,
    }
}
fn vector(direction: Direction) -> (f32, f32) {
    match direction {
        Direction::Right => (1., 0.),
        Direction::Up => (0., -1.),
        Direction::Left => (-1., 0.),
        Direction::Down => (0., 1.),
        Direction::DownRight => (1., 1.),
        Direction::None => (0., 0.),
    }
}
fn reveal(rect: Rect) -> Layer {
    Layer {
        clip: rect,
        ..layer(Source::New)
    }
}

pub(crate) fn layers(t: PageTransition, progress: f32) -> Vec<Layer> {
    let p = progress.clamp(0., 1.);
    if p >= 1. || t.style == Style::Replace || t.duration_seconds <= 0. {
        return vec![layer(Source::New)];
    }
    if p <= 0. {
        return vec![layer(Source::Old)];
    }
    let mut out = vec![layer(Source::Old)];
    match t.style {
        Style::Fade => out.push(Layer {
            opacity: p,
            ..layer(Source::New)
        }),
        Style::Wipe => {
            let clip = match t.direction {
                Direction::Right => Rect { w: p, ..Rect::FULL },
                Direction::Left => Rect {
                    x: 1. - p,
                    w: p,
                    ..Rect::FULL
                },
                Direction::Up => Rect {
                    y: 1. - p,
                    h: p,
                    ..Rect::FULL
                },
                _ => Rect { h: p, ..Rect::FULL },
            };
            out.push(reveal(clip));
        }
        Style::Split => {
            let horizontal = t.dimension == Dimension::Horizontal;
            if t.motion == Motion::Outward {
                out.push(reveal(if horizontal {
                    Rect {
                        y: (1. - p) / 2.,
                        h: p,
                        ..Rect::FULL
                    }
                } else {
                    Rect {
                        x: (1. - p) / 2.,
                        w: p,
                        ..Rect::FULL
                    }
                }));
            } else {
                out.push(reveal(if horizontal {
                    Rect {
                        h: p / 2.,
                        ..Rect::FULL
                    }
                } else {
                    Rect {
                        w: p / 2.,
                        ..Rect::FULL
                    }
                }));
                out.push(reveal(if horizontal {
                    Rect {
                        y: 1. - p / 2.,
                        h: p / 2.,
                        ..Rect::FULL
                    }
                } else {
                    Rect {
                        x: 1. - p / 2.,
                        w: p / 2.,
                        ..Rect::FULL
                    }
                }));
            }
        }
        Style::Blinds => {
            for index in 0..8 {
                let start = index as f32 / 8.;
                out.push(reveal(if t.dimension == Dimension::Horizontal {
                    Rect {
                        y: start,
                        h: p / 8.,
                        ..Rect::FULL
                    }
                } else {
                    Rect {
                        x: start,
                        w: p / 8.,
                        ..Rect::FULL
                    }
                }));
            }
        }
        Style::Box => {
            if t.motion == Motion::Outward {
                out.push(reveal(Rect {
                    x: (1. - p) / 2.,
                    y: (1. - p) / 2.,
                    w: p,
                    h: p,
                }));
            } else {
                let edge = p / 2.;
                out.extend(
                    [
                        Rect {
                            x: 0.,
                            y: 0.,
                            w: 1.,
                            h: edge,
                        },
                        Rect {
                            x: 0.,
                            y: 1. - edge,
                            w: 1.,
                            h: edge,
                        },
                        Rect {
                            x: 0.,
                            y: edge,
                            w: edge,
                            h: 1. - p,
                        },
                        Rect {
                            x: 1. - edge,
                            y: edge,
                            w: edge,
                            h: 1. - p,
                        },
                    ]
                    .map(reveal),
                );
            }
        }
        Style::Dissolve | Style::Glitter => {
            // Spatial cells use a stable permutation, never per-frame randomness.
            for y in 0..24u32 {
                for x in 0..32u32 {
                    let hash = (y * 32 + x)
                        .wrapping_mul(747796405)
                        .wrapping_add(2891336453);
                    let hash = ((hash >> ((hash >> 28) + 4)) ^ hash).wrapping_mul(277803737);
                    let noise = ((hash >> 22) ^ hash) as f64 / u32::MAX as f64;
                    let threshold = if t.style == Style::Dissolve {
                        noise as f32
                    } else {
                        let u = (x as f32 + 0.5) / 32.;
                        let v = (y as f32 + 0.5) / 24.;
                        let sweep = match t.direction {
                            Direction::Left => 1. - u,
                            Direction::Up => 1. - v,
                            Direction::Down => v,
                            Direction::DownRight => (u + v) / 2.,
                            _ => u,
                        };
                        0.8 * sweep + 0.2 * noise as f32
                    };
                    if threshold < p {
                        out.push(reveal(Rect {
                            x: x as f32 / 32.,
                            y: y as f32 / 24.,
                            w: 1. / 32.,
                            h: 1. / 24.,
                        }));
                    }
                }
            }
        }
        Style::Push | Style::Cover | Style::Uncover => {
            let (dx, dy) = vector(t.direction);
            let mut old = layer(Source::Old);
            let mut new = layer(Source::New);
            if t.style != Style::Cover {
                old.image.x = dx * p;
                old.image.y = dy * p;
            }
            if t.style != Style::Uncover {
                new.image.x = -dx * (1. - p);
                new.image.y = -dy * (1. - p);
            }
            out = if t.style == Style::Uncover {
                vec![new, old]
            } else {
                vec![old, new]
            };
        }
        Style::Fly => {
            let inward = t.motion == Motion::Inward;
            let q = if inward { 1. - p } else { p };
            let scale = 1. + (t.scale - 1.) * q;
            let (dx, dy) = vector(t.direction);
            let sign = if inward { -1. } else { 1. };
            // Scaling is centered on the page. Enlarged changes need more
            // than one page of travel to reach a fully offscreen endpoint.
            let travel = ((1. + t.scale) / 2.).max(1.);
            out = vec![
                layer(if inward { Source::Old } else { Source::New }),
                Layer {
                    source: Source::Difference,
                    image: Rect {
                        x: (1. - scale) / 2. + sign * dx * travel * q,
                        y: (1. - scale) / 2. + sign * dy * travel * q,
                        w: scale,
                        h: scale,
                    },
                    ..layer(Source::Difference)
                },
            ];
        }
        Style::Replace => unreachable!(),
    }
    out
}

/// One preparation pass, only changed pixels travel. B=true fills their bounding rectangle.
/// Different raster sizes cannot be compared safely and use explicit replacement fallback.
pub(crate) fn difference(old: &BgraFrame, new: &BgraFrame, t: PageTransition) -> Option<BgraFrame> {
    if (old.width(), old.height()) != (new.width(), new.height()) {
        return None;
    }
    let width = old.width() as usize;
    let height = old.height() as usize;
    let mut bounds = (width, height, 0, 0);
    let source = if t.motion == Motion::Inward { new } else { old };
    let mut pixels = vec![0; source.pixels_bgra().len()];
    for (i, (a, b)) in old
        .pixels_bgra()
        .as_chunks::<4>()
        .0
        .iter()
        .zip(new.pixels_bgra().as_chunks::<4>().0.iter())
        .enumerate()
    {
        if a != b {
            bounds.0 = bounds.0.min(i % width);
            bounds.1 = bounds.1.min(i / width);
            bounds.2 = bounds.2.max(i % width + 1);
            bounds.3 = bounds.3.max(i / width + 1);
            pixels[i * 4..i * 4 + 4].copy_from_slice(&source.pixels_bgra()[i * 4..i * 4 + 4]);
        }
    }
    if t.opaque && bounds.0 < bounds.2 {
        for y in bounds.1..bounds.3 {
            let start = (y * width + bounds.0) * 4;
            let end = (y * width + bounds.2) * 4;
            pixels[start..end].copy_from_slice(&source.pixels_bgra()[start..end]);
        }
    }
    BgraFrame::new(old.width(), old.height(), old.stride(), pixels).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fly_scaled_cardinal_endpoints_stay_offscreen() {
        for scale in [0., 0.3, 1., 3.] {
            for direction in [
                Direction::Right,
                Direction::Up,
                Direction::Left,
                Direction::Down,
            ] {
                for motion in [Motion::Inward, Motion::Outward] {
                    let t = PageTransition {
                        style: Style::Fly,
                        scale,
                        direction,
                        motion,
                        ..Default::default()
                    };
                    // Inspect the moving layer just inside the endpoint. Exact
                    // endpoints replace it and can conceal a visible jump.
                    let p = if motion == Motion::Inward {
                        0.000001
                    } else {
                        0.999999
                    };
                    let image = layers(t, p)[1].image;
                    let visible_w = (image.x + image.w).min(1.) - image.x.max(0.);
                    let visible_h = (image.y + image.h).min(1.) - image.y.max(0.);
                    assert!(visible_w.min(visible_h) < 0.00001, "{t:?}: {image:?}");
                }
            }
        }
    }
    #[test]
    fn reveal_variants_preserve_direction_and_inward_outward_semantics() {
        let revealed = |t: PageTransition, x: f32, y: f32| {
            layers(t, 0.5).iter().any(|l| {
                l.source == Source::New
                    && x >= l.clip.x
                    && x < l.clip.x + l.clip.w
                    && y >= l.clip.y
                    && y < l.clip.y + l.clip.h
            })
        };
        for style in [Style::Split, Style::Box] {
            for dimension in [Dimension::Horizontal, Dimension::Vertical] {
                let t = PageTransition {
                    style,
                    dimension,
                    motion: Motion::Inward,
                    ..Default::default()
                };
                assert!(!revealed(t, 0.5, 0.5));
                let edge = if dimension == Dimension::Horizontal {
                    (0.5, 0.1)
                } else {
                    (0.1, 0.5)
                };
                assert!(revealed(t, edge.0, edge.1));
                let t = PageTransition {
                    motion: Motion::Outward,
                    ..t
                };
                assert!(revealed(t, 0.5, 0.5));
                assert!(!revealed(t, edge.0, edge.1));
            }
        }
        for (direction, yes, no) in [
            (Direction::Right, (0.1, 0.5), (0.9, 0.5)),
            (Direction::Left, (0.9, 0.5), (0.1, 0.5)),
            (Direction::Up, (0.5, 0.9), (0.5, 0.1)),
            (Direction::Down, (0.5, 0.1), (0.5, 0.9)),
        ] {
            let t = PageTransition {
                style: Style::Wipe,
                direction,
                ..Default::default()
            };
            assert!(revealed(t, yes.0, yes.1));
            assert!(!revealed(t, no.0, no.1));
        }
    }
    #[test]
    fn every_style_has_exact_endpoints_and_distinct_midpoint_geometry() {
        for style in [
            Style::Split,
            Style::Blinds,
            Style::Box,
            Style::Wipe,
            Style::Dissolve,
            Style::Glitter,
            Style::Fly,
            Style::Push,
            Style::Cover,
            Style::Uncover,
            Style::Fade,
        ] {
            let t = PageTransition {
                style,
                ..Default::default()
            };
            assert_eq!(layers(t, 0.), vec![layer(Source::Old)]);
            assert_eq!(layers(t, 1.), vec![layer(Source::New)]);
            assert!(layers(t, 0.5).len() >= 2);
        }
        let t = PageTransition {
            style: Style::Push,
            direction: Direction::Left,
            ..Default::default()
        };
        assert_eq!(
            (layers(t, 0.5)[0].image.x, layers(t, 0.5)[1].image.x),
            (-0.5, 0.5)
        );
        assert_eq!(
            layers(
                PageTransition {
                    style: Style::Cover,
                    ..t
                },
                0.5
            )[0]
            .image,
            Rect::FULL
        );
        assert_eq!(
            layers(
                PageTransition {
                    style: Style::Uncover,
                    ..t
                },
                0.5
            )[0]
            .source,
            Source::New
        );
        let dissolve = layers(
            PageTransition {
                style: Style::Dissolve,
                ..t
            },
            0.5,
        );
        assert!(dissolve.len() > 200 && dissolve.len() < 600);
        assert!(dissolve.iter().all(|l| l.opacity == 1.));
        assert_ne!(
            dissolve,
            layers(
                PageTransition {
                    style: Style::Glitter,
                    ..t
                },
                0.5
            )
        );
    }
    #[test]
    fn fly_moves_only_changed_pixels_and_opaque_mode_fills_their_rectangle() {
        let old = BgraFrame::new(3, 1, 12, vec![255; 12]).unwrap();
        let new = BgraFrame::new(
            3,
            1,
            12,
            vec![0, 0, 0, 255, 255, 255, 255, 255, 0, 0, 0, 255],
        )
        .unwrap();
        let t = PageTransition {
            style: Style::Fly,
            direction: Direction::None,
            scale: 0.2,
            ..Default::default()
        };
        assert_eq!(difference(&old, &new, t).unwrap().pixels_bgra()[7], 0);
        assert_eq!(
            difference(&old, &new, PageTransition { opaque: true, ..t })
                .unwrap()
                .pixels_bgra()[7],
            255
        );
        assert!((layers(t, 0.5)[1].image.w - 0.6).abs() < 0.0001);
        assert_eq!(
            layers(
                PageTransition {
                    motion: Motion::Outward,
                    ..t
                },
                0.5
            )[0]
            .source,
            Source::New
        );
    }
}
