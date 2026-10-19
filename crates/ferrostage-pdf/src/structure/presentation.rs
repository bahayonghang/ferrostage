use super::{DiagnosticKind, Parser};
use ferrostage_core::{
    PagePresentation, PageTransition, TransitionDimension as Dimension,
    TransitionDirection as Direction, TransitionMotion as Motion, TransitionStyle as Style,
};
use lopdf::{Dictionary, Object};

pub(super) fn extract(parser: &mut Parser<'_>, page: &Dictionary) -> PagePresentation {
    let dwell_seconds = if page.has(b"Dur") {
        match parser
            .number(page, b"Dur")
            .filter(|n| std::time::Duration::try_from_secs_f64(*n).is_ok())
        {
            Some(value) => Some(value),
            None => {
                parser.warn("Dur", DiagnosticKind::Malformed);
                None
            }
        }
    } else {
        None
    };
    let transition = if page.has(b"Trans") {
        match parser.dict(page, b"Trans") {
            Some(dict) => match transition(parser, dict) {
                Ok(value) => Some(value),
                Err((field, kind)) => {
                    parser.warn(field, kind);
                    None
                }
            },
            None => {
                parser.warn("Trans", DiagnosticKind::Malformed);
                None
            }
        }
    } else {
        None
    };
    PagePresentation {
        transition,
        dwell_seconds,
    }
}

fn transition(
    parser: &Parser<'_>,
    dict: &Dictionary,
) -> Result<PageTransition, (&'static str, DiagnosticKind)> {
    let malformed = |field| (field, DiagnosticKind::Malformed);
    let mut t = PageTransition::default();
    if dict.has(b"S") {
        t.style = match parser.name(dict, b"S").ok_or(malformed("Trans/S"))? {
            b"R" => Style::Replace,
            b"Split" => Style::Split,
            b"Blinds" => Style::Blinds,
            b"Box" => Style::Box,
            b"Wipe" => Style::Wipe,
            b"Dissolve" => Style::Dissolve,
            b"Glitter" => Style::Glitter,
            b"Fly" => Style::Fly,
            b"Push" => Style::Push,
            b"Cover" => Style::Cover,
            b"Uncover" => Style::Uncover,
            b"Fade" => Style::Fade,
            _ => return Err(("Trans/S", DiagnosticKind::Unsupported)),
        };
    }
    if dict.has(b"D") {
        t.duration_seconds = parser
            .number(dict, b"D")
            .filter(|n| *n >= 0.)
            .ok_or(malformed("Trans/D"))?;
    }
    if matches!(t.style, Style::Split | Style::Blinds) && dict.has(b"Dm") {
        t.dimension = match parser.name(dict, b"Dm") {
            Some(b"H") => Dimension::Horizontal,
            Some(b"V") => Dimension::Vertical,
            _ => return Err(malformed("Trans/Dm")),
        };
    }
    if matches!(t.style, Style::Split | Style::Box | Style::Fly) && dict.has(b"M") {
        t.motion = match parser.name(dict, b"M") {
            Some(b"I") => Motion::Inward,
            Some(b"O") => Motion::Outward,
            _ => return Err(malformed("Trans/M")),
        };
    }
    if matches!(
        t.style,
        Style::Wipe | Style::Glitter | Style::Fly | Style::Push | Style::Cover | Style::Uncover
    ) && dict.has(b"Di")
    {
        t.direction = match parser.field(dict, b"Di") {
            Some(Object::Name(name)) if name == b"None" && t.style == Style::Fly => Direction::None,
            Some(value) => match super::number(value) {
                Some(0.) => Direction::Right,
                Some(90.) => Direction::Up,
                Some(180.) => Direction::Left,
                Some(270.) => Direction::Down,
                Some(315.) if t.style == Style::Glitter => Direction::DownRight,
                _ => return Err(malformed("Trans/Di")),
            },
            _ => return Err(malformed("Trans/Di")),
        };
    }
    if t.style == Style::Fly {
        if dict.has(b"SS") {
            t.scale = parser
                .number(dict, b"SS")
                .filter(|n| (0.0..=f64::from(f32::MAX)).contains(n))
                .ok_or(malformed("Trans/SS"))? as f32;
        }
        if dict.has(b"B") {
            t.opaque = match parser.field(dict, b"B") {
                Some(Object::Boolean(value)) => *value,
                _ => return Err(malformed("Trans/B")),
            };
        }
    }
    Ok(t)
}

#[cfg(test)]
mod tests {
    use super::*;
    use lopdf::{Document, dictionary};
    #[test]
    fn dwell_absent_zero_positive_indirect_and_unrepresentable_are_distinct() {
        let mut document = Document::new();
        let duration = document.add_object(Object::Real(1.5));
        for (value, expected) in [
            (None, None),
            (Some(Object::Integer(0)), Some(0.)),
            (Some(Object::Reference(duration)), Some(1.5)),
            (Some(Object::Integer(-1)), None),
            (Some(Object::Real(f32::MAX)), None),
            (Some(Object::Real(f32::NAN)), None),
        ] {
            let mut diagnostics = vec![];
            let mut parser = Parser {
                document: &document,
                page: ferrostage_core::PhysicalPage(0),
                annotation: None,
                diagnostics: &mut diagnostics,
            };
            let mut page = Dictionary::new();
            if let Some(value) = value.clone() {
                page.set("Dur", value);
            }
            assert_eq!(extract(&mut parser, &page).dwell_seconds, expected);
            assert_eq!(
                !diagnostics.is_empty(),
                value.is_some() && expected.is_none()
            );
        }
    }

    #[test]
    fn standard_styles_indirect_values_defaults_and_dwell_are_distinct() {
        let mut document = Document::new();
        let duration = document.add_object(Object::Real(0.3));
        for (name, style) in [
            ("R", Style::Replace),
            ("Split", Style::Split),
            ("Blinds", Style::Blinds),
            ("Box", Style::Box),
            ("Wipe", Style::Wipe),
            ("Dissolve", Style::Dissolve),
            ("Glitter", Style::Glitter),
            ("Fly", Style::Fly),
            ("Push", Style::Push),
            ("Cover", Style::Cover),
            ("Uncover", Style::Uncover),
            ("Fade", Style::Fade),
        ] {
            let id = document.add_object(
                dictionary! { "S" => Object::Name(name.as_bytes().to_vec()), "D" => duration },
            );
            let page = dictionary! { "Trans" => id, "Dur" => 0 };
            let mut diagnostics = vec![];
            let mut parser = Parser {
                document: &document,
                page: ferrostage_core::PhysicalPage(0),
                annotation: None,
                diagnostics: &mut diagnostics,
            };
            let value = extract(&mut parser, &page);
            assert_eq!(value.transition.unwrap().style, style);
            assert!((value.transition.unwrap().duration_seconds - 0.3).abs() < 0.0001);
            assert_eq!(value.dwell_seconds, Some(0.));
            assert!(diagnostics.is_empty());
        }
        let mut diagnostics = vec![];
        let mut parser = Parser {
            document: &document,
            page: ferrostage_core::PhysicalPage(0),
            annotation: None,
            diagnostics: &mut diagnostics,
        };
        assert_eq!(
            extract(&mut parser, &dictionary! {"Trans" => Dictionary::new()}).transition,
            Some(PageTransition::default())
        );
        assert_eq!(
            extract(&mut parser, &Dictionary::new()),
            PagePresentation::default()
        );
        for dict in [
            dictionary! {"S" => "Unknown"},
            dictionary! {"D" => -1},
            dictionary! {"S" => "Fly", "SS" => -1},
            dictionary! {"S" => "Push", "Di" => 45},
        ] {
            assert!(
                extract(&mut parser, &dictionary! {"Trans" => dict})
                    .transition
                    .is_none()
            );
        }
        assert_eq!(diagnostics.len(), 4);
    }
    #[test]
    fn fixture_covers_parameters_and_indirect_transition_dictionaries() {
        let document = super::super::extract_structure(include_bytes!(
            "../../../../fixtures/beamer/transitions.pdf"
        ))
        .unwrap();
        assert_eq!(document.pages.len(), 51);
        assert!(document.diagnostics.is_empty());
        let effects: Vec<_> = document
            .pages
            .iter()
            .map(|p| p.presentation.transition.unwrap())
            .collect();
        for dimension in [Dimension::Horizontal, Dimension::Vertical] {
            for motion in [Motion::Inward, Motion::Outward] {
                assert!(effects.iter().any(|t| t.style == Style::Split
                    && t.dimension == dimension
                    && t.motion == motion));
            }
        }
        assert!(
            effects
                .iter()
                .any(|t| t.style == Style::Glitter && t.direction == Direction::DownRight)
        );
        for motion in [Motion::Inward, Motion::Outward] {
            for opaque in [false, true] {
                assert!(effects.iter().any(|t| t.style == Style::Fly
                    && t.direction == Direction::None
                    && t.motion == motion
                    && t.opaque == opaque
                    && (t.scale - 0.3).abs() < 0.00001));
            }
        }
        for style in [Style::Wipe, Style::Push, Style::Cover, Style::Uncover] {
            for direction in [
                Direction::Right,
                Direction::Up,
                Direction::Left,
                Direction::Down,
            ] {
                assert!(
                    effects
                        .iter()
                        .any(|t| t.style == style && t.direction == direction)
                );
            }
        }
        for motion in [Motion::Inward, Motion::Outward] {
            for direction in [
                Direction::Right,
                Direction::Up,
                Direction::Left,
                Direction::Down,
            ] {
                assert!(effects.iter().any(|t| t.style == Style::Fly
                    && t.motion == motion
                    && t.direction == direction
                    && t.scale == 3.));
            }
        }
    }
}
