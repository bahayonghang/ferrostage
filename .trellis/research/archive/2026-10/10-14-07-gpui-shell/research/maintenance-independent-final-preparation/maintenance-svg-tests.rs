// Isolated evaluation regressions. Never linked into the FerroStage product.
#[cfg(test)]
mod maintenance_tests {
    use super::*;

    const IBM: &[u8] =
        include_bytes!("../../../assets/fonts/ibm-plex-sans/IBMPlexSans-Regular.ttf");
    const LILEX: &[u8] = include_bytes!("../../../assets/fonts/lilex/Lilex-Regular.ttf");
    const COLLECTION: &[u8] = include_bytes!("../../../assets/fonts/evaluation.ttc");
    const SHAPE: &[u8] = include_bytes!("../../../maintenance-fixtures/shape.svg");
    const SHAPE_GZIP: &[u8] = include_bytes!("../../../maintenance-fixtures/shape.svgz");

    fn database() -> usvg::fontdb::Database {
        let mut db = usvg::fontdb::Database::new();
        db.load_font_data(IBM.to_vec());
        db.load_font_data(LILEX.to_vec());
        fix_generic_font_families(&mut db);
        db
    }

    fn family(db: &usvg::fontdb::Database, name: &str) -> usvg::fontdb::ID {
        db.query(&usvg::fontdb::Query {
            families: &[usvg::fontdb::Family::Name(name)],
            ..Default::default()
        })
        .unwrap()
    }

    fn renderer() -> SvgRenderer {
        SvgRenderer {
            asset_source: Arc::new(()),
            usvg_options: Arc::new(usvg::Options {
                fontdb: Arc::new(database()),
                ..Default::default()
            }),
        }
    }

    fn render(svg: &[u8]) -> Arc<RenderImage> {
        let renderer = renderer();
        let parsed = renderer.parse_svg(svg).unwrap();
        renderer
            .render_parsed(
                &parsed,
                SvgSize::ExactSize(Size::new(DevicePixels(128), DevicePixels(64))),
            )
            .unwrap()
    }

    #[test]
    fn glyph_lookup_checks_real_face_coverage_and_missing_id() {
        let db = database();
        let ibm = family(&db, "IBM Plex Sans");
        let lilex = family(&db, "Lilex");
        assert!(font_has_char(&db, ibm, 'A'));
        assert!(font_has_char(&db, lilex, 'A'));
        assert!(!font_has_char(&db, ibm, '│'));
        assert!(font_has_char(&db, lilex, '│'));
        assert!(!font_has_char(&db, ibm, '\u{10ffff}'));
        assert!(!font_has_char(&usvg::fontdb::Database::new(), ibm, 'A'));
    }

    #[test]
    fn collection_lookup_respects_both_face_indices() {
        let mut db = usvg::fontdb::Database::new();
        db.load_font_data(COLLECTION.to_vec());
        assert_eq!(db.len(), 2);
        let ibm = family(&db, "IBM Plex Sans");
        let lilex = family(&db, "Lilex");
        assert_eq!(db.face(ibm).unwrap().index, 0);
        assert_eq!(db.face(lilex).unwrap().index, 1);
        assert!(!font_has_char(&db, ibm, '│'));
        assert!(font_has_char(&db, lilex, '│'));
        assert!(skrifa::FontRef::from_index(COLLECTION, 2).is_err());
        assert!(skrifa::FontRef::from_index(b"invalid font", 0).is_err());
    }

    #[test]
    fn fallback_excludes_used_and_missing_glyph_faces() {
        let db = database();
        let lilex = family(&db, "Lilex");
        let families = &["Missing family", "IBM Plex Sans", "Lilex"];
        assert_eq!(select_emoji_font('│', &[], &db, families), Some(lilex));
        assert_eq!(select_emoji_font('│', &[lilex], &db, families), None);
        assert_eq!(select_emoji_font('\u{10ffff}', &[], &db, families), None);
    }

    #[test]
    fn shape_keeps_bgra_alpha_and_requested_size() {
        let image = render(SHAPE);
        assert_eq!(
            image.size(0),
            Size::new(DevicePixels(128), DevicePixels(64))
        );
        let bytes = image.as_bytes(0).unwrap();
        let at = |x: usize, y: usize| &bytes[(y * 128 + x) * 4..(y * 128 + x) * 4 + 4];
        assert_eq!(at(8, 8), &[0, 0, 255, 255]);
        assert_eq!(at(80, 8), &[255, 0, 0, 255]);
        assert_eq!(at(8, 48), &[0, 0, 0, 0]);
    }

    #[test]
    fn svgz_decodes_to_identical_shape_pixels() {
        assert_eq!(render(SHAPE).as_bytes(0), render(SHAPE_GZIP).as_bytes(0));
    }

    #[test]
    fn text_fallback_renders_the_available_glyph() {
        let svg = |font: &str| {
            format!(
                r#"<svg xmlns="http://www.w3.org/2000/svg" width="128" height="64"><text x="8" y="48" font-family="{font}" font-size="40">│</text></svg>"#
            )
        };
        let fallback = render(svg("IBM Plex Sans").as_bytes());
        let explicit = render(svg("Lilex").as_bytes());
        let pixels = fallback.as_bytes(0).unwrap();
        assert!(pixels.chunks_exact(4).filter(|pixel| pixel[3] != 0).count() > 40);
        assert_eq!(pixels, explicit.as_bytes(0).unwrap());
    }

    #[test]
    fn ligature_combining_and_mixed_font_text_produces_visible_output() {
        let image = render("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"128\" height=\"64\"><text x=\"2\" y=\"28\" font-family=\"IBM Plex Sans\" font-size=\"24\">office é</text><text x=\"2\" y=\"56\" font-family=\"Lilex\" font-size=\"24\">A│</text></svg>".as_bytes());
        let pixels = image.as_bytes(0).unwrap();
        for rows in [0..32, 32..64] {
            let painted = pixels[rows.start * 128 * 4..rows.end * 128 * 4]
                .chunks_exact(4)
                .filter(|p| p[3] > 0)
                .count();
            assert!(
                painted > 100,
                "expected shaped text in each row, got {painted}"
            );
        }
    }
}
