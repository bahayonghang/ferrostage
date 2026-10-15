#![forbid(unsafe_code)]

use cosmic_text::{Attrs, Buffer, Color, Family, FontSystem, Metrics, Shaping, SwashCache, fontdb};

const COLLECTION: &[u8] = include_bytes!("../assets/fonts/evaluation.ttc");
const IBM: &[u8] = include_bytes!("../assets/fonts/ibm-plex-sans/IBMPlexSans-Regular.ttf");

#[test]
fn maintained_metadata_parser_preserves_collection_face_metadata() {
    let mut db = fontdb::Database::new();
    db.load_font_data(COLLECTION.to_vec());
    assert_eq!(db.len(), 2);
    for (index, name, mono) in [(0, "IBM Plex Sans", false), (1, "Lilex", true)] {
        let id = db
            .query(&fontdb::Query {
                families: &[Family::Name(name)],
                ..Default::default()
            })
            .unwrap();
        let face = db.face(id).unwrap();
        assert_eq!(face.index, index);
        assert_eq!(face.monospaced, mono);
        assert_eq!(face.weight, fontdb::Weight::NORMAL);
        assert_eq!(face.style, fontdb::Style::Normal);
    }
}

#[test]
fn malformed_metadata_and_out_of_bounds_collection_offsets_do_not_load_faces() {
    for data in [
        &IBM[..0],
        &IBM[..4],
        &IBM[..12],
        &IBM[..64],
        &COLLECTION[..20],
        b"invalid font",
    ] {
        let mut db = fontdb::Database::new();
        db.load_font_data(data.to_vec());
        assert_eq!(db.len(), 0);
    }
    let mut bad = COLLECTION.to_vec();
    bad[12..16].copy_from_slice(&u32::MAX.to_be_bytes());
    bad[16..20].copy_from_slice(&u32::MAX.to_be_bytes());
    let mut db = fontdb::Database::new();
    db.load_font_data(bad);
    assert_eq!(db.len(), 0);
}

#[test]
fn cosmic_shaping_fallback_and_rasterization_use_both_collection_faces() {
    let mut db = fontdb::Database::new();
    db.load_font_data(COLLECTION.to_vec());
    db.set_sans_serif_family("IBM Plex Sans");
    db.set_monospace_family("Lilex");
    let lilex = db
        .query(&fontdb::Query {
            families: &[Family::Name("Lilex")],
            ..Default::default()
        })
        .unwrap();
    let mut fonts = FontSystem::new_with_locale_and_db("en-US".to_string(), db);
    let mut buffer = Buffer::new(&mut fonts, Metrics::new(24.0, 30.0));
    buffer.set_size(Some(320.0), Some(64.0));
    buffer.set_text(
        "A│office é",
        &Attrs::new().family(Family::Name("IBM Plex Sans")),
        Shaping::Advanced,
        None,
    );
    buffer.shape_until_scroll(&mut fonts, false);
    let glyphs: Vec<_> = buffer
        .layout_runs()
        .flat_map(|run| run.glyphs.iter())
        .collect();
    assert!(glyphs.len() >= 6);
    assert!(glyphs.iter().all(|glyph| glyph.glyph_id != 0));
    assert!(glyphs.iter().any(|glyph| glyph.font_id == lilex));
    let mut painted = 0;
    buffer.draw(
        &mut fonts,
        &mut SwashCache::new(),
        Color::rgb(255, 255, 255),
        |_, _, w, h, color| {
            if color.a() > 0 {
                painted += w * h;
            }
        },
    );
    assert!(painted > 100);
}
