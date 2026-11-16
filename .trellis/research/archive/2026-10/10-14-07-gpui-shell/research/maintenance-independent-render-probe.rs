#[test]
fn compare_real_svg_fallback_across_text_stacks() {
    fn old_probe(data: &[u8], index: u32, ch: char) -> bool {
        ttf_parser::Face::parse(data, index).ok().and_then(|f| f.glyph_index(ch)).is_some()
    }
    fn repaired_probe(data: &[u8], _index: u32, ch: char) -> bool {
        let mut db = fontdb::Database::new();
        db.load_font_data(data.to_vec());
        let id = db.faces().next().unwrap().id;
        font_has_char(&db, id, ch)
    }
    macro_rules! render {
        ($engine:ident, $probe:ident, $data:expr) => {{
            use $engine as engine;
            let mut db = engine::usvg::fontdb::Database::new();
            db.load_font_data(include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../gpui-maintenance-candidate/assets/fonts/lilex/Lilex-Regular.ttf")).to_vec());
            db.load_font_data($data.clone());
            let fallback_calls = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
            let calls = fallback_calls.clone();
            let options = engine::usvg::Options {
                fontdb: std::sync::Arc::new(db),
                font_resolver: engine::usvg::FontResolver {
                    select_font: engine::usvg::FontResolver::default_font_selector(),
                    select_fallback: Box::new(move |ch, used, db| {
                        calls.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        db.faces().filter(|f| !used.contains(&f.id)).find(|f| {
                            db.with_face_data(f.id, |data, index| $probe(data, index, ch)).unwrap_or(false)
                        }).map(|f| f.id)
                    }),
                },
                ..Default::default()
            };
            let svg = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"128\" height=\"64\"><text x=\"8\" y=\"48\" font-family=\"Lilex\" font-size=\"40\">✅</text></svg>";
            let tree = engine::usvg::Tree::from_data(svg.as_bytes(), &options).unwrap();
            let mut pixmap = engine::tiny_skia::Pixmap::new(128, 64).unwrap();
            engine::render(&tree, engine::tiny_skia::Transform::identity(), &mut pixmap.as_mut());
            (pixmap.data().chunks_exact(4).filter(|p| p[3] != 0).count(),
             fallback_calls.load(std::sync::atomic::Ordering::Relaxed), pixmap.take())
        }};
    }
    let glyph = ttf_parser::Face::parse(IBM, 0).unwrap().glyph_index('A').unwrap().0;
    let mut failures = Vec::new();
    for (label, table, encoding) in [
        ("format13", maintenance_cmap_tests::range(13, 0x2705, glyph), 6),
        ("format2", maintenance_cmap_tests::cmap2(0x2705, glyph, 0), 0),
    ] {
        let data = maintenance_cmap_tests::font(vec![(0, encoding, table)]);
        let old = render!(resvg_old, old_probe, data);
        let repaired = render!(resvg_new, repaired_probe, data);
        println!("SVG fallback {label}: old painted={} calls={}; repaired painted={} calls={}", old.0, old.1, repaired.0, repaired.1);
        assert!(old.0 > 40 && old.1 > 0, "old stack baseline must paint real outline through fallback");
        println!("SVG fallback {label}: exact pixels equal={}", old.2 == repaired.2);
        if repaired.0 <= 40 || repaired.1 == 0 || old.2 != repaired.2 { failures.push(label); }
    }
    assert!(failures.is_empty(), "old supported rendering lost: {failures:?}");
}
