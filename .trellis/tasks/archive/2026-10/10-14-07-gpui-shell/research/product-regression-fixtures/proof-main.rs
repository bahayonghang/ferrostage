use skrifa::raw::TableProvider;
mod usvg { pub use fontdb; }

const IBM: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../vendor/fixtures/fonts/ibm-plex-sans/IBMPlexSans-Regular.ttf"));

fn u16be(bytes: &mut Vec<u8>, value: u16) { bytes.extend(value.to_be_bytes()); }
fn u32be(bytes: &mut Vec<u8>, value: u32) { bytes.extend(value.to_be_bytes()); }

fn range(format: u16, codepoint: u32, glyph: u16) -> Vec<u8> {
    let mut table = Vec::new();
    u16be(&mut table, format);
    u16be(&mut table, 0);
    u32be(&mut table, 28);
    u32be(&mut table, 0);
    u32be(&mut table, 1);
    u32be(&mut table, codepoint);
    u32be(&mut table, codepoint);
    u32be(&mut table, glyph as u32);
    table
}

fn format2(glyph: u16) -> Vec<u8> {
    let mut table = Vec::new();
    u16be(&mut table, 2);
    u16be(&mut table, 528);
    u16be(&mut table, 0);
    table.resize(518, 0); // All high bytes use the single-byte subheader.
    for value in [65, 1, 0, 2, glyph] { u16be(&mut table, value); }
    table
}

fn font(records: Vec<(u16, u16, Vec<u8>)>) -> Vec<u8> {
    let mut cmap = Vec::new();
    u16be(&mut cmap, 0);
    u16be(&mut cmap, records.len() as u16);
    let mut offset = 4 + 8 * records.len();
    for (platform, encoding, table) in &records {
        u16be(&mut cmap, *platform);
        u16be(&mut cmap, *encoding);
        u32be(&mut cmap, offset as u32);
        offset += table.len();
    }
    for (_, _, table) in records { cmap.extend(table); }
    let mut data = IBM.to_vec();
    data.resize((data.len() + 3) & !3, 0);
    let offset = data.len() as u32;
    let count = u16::from_be_bytes(data[4..6].try_into().unwrap()) as usize;
    let entry = (0..count).map(|n| 12 + 16 * n).find(|&n| &data[n..n+4] == b"cmap").unwrap();
    data[entry+8..entry+12].copy_from_slice(&offset.to_be_bytes());
    data[entry+12..entry+16].copy_from_slice(&(cmap.len() as u32).to_be_bytes());
    data.extend(cmap);
    data
}

#[test]
fn candidate_lookup_preserves_old_unicode_coverage() {
    let glyph = ttf_parser::Face::parse(IBM, 0).unwrap().glyph_index('A').unwrap().0;
    let cases = [
        ("format13", font(vec![(0, 4, range(13, 0x2502, glyph))]), '│'),
        ("format2", font(vec![(0, 0, format2(glyph))]), 'A'),
        ("multiple_unicode", font(vec![(0, 4, range(12, 65, glyph)), (0, 6, range(12, 0x2502, glyph))]), 'A'),
        ("symbol_priority", font(vec![(3, 0, range(12, 65, glyph)), (0, 4, range(12, 0x2502, glyph))]), '│'),
        ("symbol_only", font(vec![(3, 0, range(12, 65, glyph))]), 'A'),
    ];
    let mut differences = Vec::new();
    for (label, data, ch) in cases {
        let old = ttf_parser::Face::parse(&data, 0).unwrap().glyph_index(ch).is_some();
        let mut db = fontdb::Database::new();
        db.load_font_data(data);
        let id = db.faces().next().unwrap().id;
        let candidate = font_has_char(&db, id, ch);
        println!("{label}: old={old}, candidate={candidate}");
        if old != candidate { differences.push(label); }
    }
    assert!(differences.is_empty(), "behavior changed for {differences:?}");
}

fn main() {}

fn font_has_char(db: &usvg::fontdb::Database, id: usvg::fontdb::ID, ch: char) -> bool {
    // Preserve GPUI's coverage probe across Unicode records. This is not the
    // renderer's preferred-charmap selection policy (which may prefer symbols).
    db.with_face_data(id, |font_data, face_index| {
        let face = skrifa::FontRef::from_index(font_data, face_index).ok()?;
        let cmap = face.cmap().ok()?;
        cmap.encoding_records().iter().find_map(|record| {
            use skrifa::raw::tables::cmap::{CmapSubtable, PlatformId};
            let subtable = record.subtable(cmap.offset_data()).ok()?;
            let unicode = match (record.platform_id(), record.encoding_id()) {
                (PlatformId::Unicode, _) | (PlatformId::Windows, 1) => true,
                (PlatformId::Windows, 10) => matches!(
                    subtable,
                    CmapSubtable::Format12(_) | CmapSubtable::Format13(_)
                ),
                _ => false,
            };
            if !unicode {
                return None;
            }
            if let CmapSubtable::Format2(table) = &subtable {
                return cmap2_glyph(table, ch);
            }
            let glyph = subtable.map_codepoint(ch)?.to_u32();
            // The original parser rejects zero in format 0; other formats
            // preserve their optional glyph result, including an explicit 0.
            if matches!(subtable, CmapSubtable::Format0(_)) && glyph == 0 {
                return None;
            }
            u16::try_from(glyph).ok()
        })
    })
    .flatten()
    .is_some()
}

// read-fonts exposes checked format-2 records but has no map_codepoint for it.
// OpenType cmap format 2 specifies offsets relative to the idRangeOffset word:
// https://learn.microsoft.com/en-us/typography/opentype/spec/cmap#format-2-high-byte-mapping-through-table
fn cmap2_glyph(table: &skrifa::raw::tables::cmap::Cmap2<'_>, ch: char) -> Option<u16> {
    use skrifa::raw::tables::cmap::SubHeader;
    let code = u16::try_from(u32::from(ch)).ok()?;
    let key = if code < 0xff {
        0
    } else {
        table.sub_header_keys().get(usize::from(code >> 8))?.get() / 8
    };
    let data = table.offset_data().slice(..usize::from(table.length()))?;
    let header_offset = table.sub_header_keys_byte_range().end + usize::from(key) * 8;
    let header = data.read_ref_at::<SubHeader>(header_offset).ok()?;
    let low = code & 0xff;
    let end = header.first_code().checked_add(header.entry_count())?;
    if low < header.first_code() || low >= end {
        return None;
    }
    let offset = header_offset
        + 6
        + usize::from(header.id_range_offset())
        + usize::from(low - header.first_code()) * 2;
    let glyph = data.read_at::<u16>(offset).ok()?;
    if glyph == 0 {
        return None;
    }
    // Retain the former coverage probe's optional result, including zero after
    // delta and its rejection of a negative remainder. No shaping is done here.
    u16::try_from((i32::from(glyph) + i32::from(header.id_delta())) % 65536).ok()
}

#[cfg(test)]
mod maintenance_cmap_tests {
    use super::*;
    const IBM: &[u8] =
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../vendor/fixtures/fonts/ibm-plex-sans/IBMPlexSans-Regular.ttf"));

    fn word(out: &mut Vec<u8>, n: u16) {
        out.extend(n.to_be_bytes());
    }
    fn long(out: &mut Vec<u8>, n: u32) {
        out.extend(n.to_be_bytes());
    }

    pub(super) fn range(format: u16, code: u32, glyph: u16) -> Vec<u8> {
        let mut out = Vec::new();
        for n in [format, 0] {
            word(&mut out, n);
        }
        for n in [28, 0, 1, code, code, u32::from(glyph)] {
            long(&mut out, n);
        }
        out
    }

    pub(super) fn cmap2(code: u16, glyph: u16, delta: i16) -> Vec<u8> {
        let double = code > 255;
        let mut out = Vec::new();
        for n in [2, if double { 536 } else { 528 }, 0] {
            word(&mut out, n);
        }
        out.resize(518, 0);
        if double {
            let key = 6 + usize::from(code >> 8) * 2;
            out[key..key + 2].copy_from_slice(&8_u16.to_be_bytes());
            for n in [0, 0, 0, 0] {
                word(&mut out, n);
            }
        }
        for n in [code & 255, 1, delta as u16, 2, glyph] {
            word(&mut out, n);
        }
        out
    }

    fn simple(format: u16, glyph: u16) -> Vec<u8> {
        let mut out = Vec::new();
        match format {
            0 => {
                for n in [0, 262, 0] {
                    word(&mut out, n);
                }
                out.resize(262, 0);
                out[6 + 65] = glyph as u8;
            }
            4 => {
                for n in [
                    4,
                    32,
                    0,
                    4,
                    4,
                    1,
                    0,
                    65,
                    65535,
                    0,
                    65,
                    65535,
                    glyph.wrapping_sub(65),
                    1,
                    0,
                    0,
                ] {
                    word(&mut out, n);
                }
            }
            6 => {
                for n in [6, 12, 0, 65, 1, glyph] {
                    word(&mut out, n);
                }
            }
            10 => {
                for n in [10, 0] {
                    word(&mut out, n);
                }
                for n in [22, 0, 65, 1] {
                    long(&mut out, n);
                }
                word(&mut out, glyph);
            }
            _ => unreachable!(),
        }
        out
    }

    // Derive only in memory from a real OFL font. No modified/RFN font is saved.
    pub(super) fn font(records: Vec<(u16, u16, Vec<u8>)>) -> Vec<u8> {
        let mut cmap = Vec::new();
        word(&mut cmap, 0);
        word(&mut cmap, records.len() as u16);
        let mut offset = 4 + 8 * records.len();
        for (platform, encoding, table) in &records {
            word(&mut cmap, *platform);
            word(&mut cmap, *encoding);
            long(&mut cmap, offset as u32);
            offset += table.len();
        }
        for (_, _, table) in records {
            cmap.extend(table);
        }
        let mut data = IBM.to_vec();
        data.resize((data.len() + 3) & !3, 0);
        let offset = data.len() as u32;
        let count = u16::from_be_bytes(data[4..6].try_into().unwrap()) as usize;
        let entry = (0..count)
            .map(|n| 12 + 16 * n)
            .find(|&n| &data[n..n + 4] == b"cmap")
            .unwrap();
        data[entry + 8..entry + 12].copy_from_slice(&offset.to_be_bytes());
        data[entry + 12..entry + 16].copy_from_slice(&(cmap.len() as u32).to_be_bytes());
        data.extend(cmap);
        data
    }

    fn cases() -> Vec<(&'static str, Vec<u8>, char, bool)> {
        let mut cases = vec![
            (
                "format2_single",
                font(vec![(0, 0, cmap2(65, 1, 0))]),
                'A',
                true,
            ),
            (
                "format2_double",
                font(vec![(0, 0, cmap2(0x2541, 1, 0))]),
                '\u{2541}',
                true,
            ),
            (
                "format2_missing",
                font(vec![(0, 0, cmap2(65, 1, 0))]),
                'B',
                false,
            ),
            (
                "format2_non_bmp",
                font(vec![(0, 0, cmap2(65, 1, 0))]),
                '\u{10041}',
                false,
            ),
            (
                "format2_zero_before_delta",
                font(vec![(0, 0, cmap2(65, 0, 2))]),
                'A',
                false,
            ),
            (
                "format2_zero_after_delta",
                font(vec![(0, 0, cmap2(65, 1, -1))]),
                'A',
                true,
            ),
            (
                "format2_negative_delta",
                font(vec![(0, 0, cmap2(65, 1, -2))]),
                'A',
                false,
            ),
            (
                "format2_wrap_delta",
                font(vec![(0, 0, cmap2(65, 65535, 1))]),
                'A',
                true,
            ),
            ("format13", font(vec![(0, 6, range(13, 65, 1))]), 'A', true),
            (
                "format12_non_bmp",
                font(vec![(3, 10, range(12, 0x10041, 1))]),
                '\u{10041}',
                true,
            ),
            (
                "second_unicode_record",
                font(vec![(0, 4, range(12, 66, 1)), (3, 10, range(12, 65, 1))]),
                'A',
                true,
            ),
            (
                "symbol_priority",
                font(vec![(0, 4, range(12, 65, 1)), (3, 0, simple(4, 0))]),
                'A',
                true,
            ),
            ("symbol_only", font(vec![(3, 0, simple(4, 1))]), 'A', false),
            (
                "windows_unicode_bmp",
                font(vec![(3, 1, simple(4, 1))]),
                'A',
                true,
            ),
            (
                "windows_full_wrong_format",
                font(vec![(3, 10, simple(4, 1))]),
                'A',
                false,
            ),
            ("mac_only", font(vec![(1, 0, simple(0, 1))]), 'A', false),
        ];
        for (format, label) in [
            (0, "format0"),
            (4, "format4"),
            (6, "format6"),
            (10, "format10"),
        ] {
            cases.push((label, font(vec![(0, 3, simple(format, 1))]), 'A', true));
        }
        for (format, label) in [
            (0, "format0_zero"),
            (6, "format6_zero"),
            (10, "format10_zero"),
        ] {
            cases.push((
                label,
                font(vec![(0, 3, simple(format, 0))]),
                'A',
                format != 0,
            ));
        }
        for (format, label) in [(12, "format12_zero"), (13, "format13_zero")] {
            cases.push((label, font(vec![(0, 4, range(format, 65, 0))]), 'A', true));
        }
        let mut truncated = cmap2(65, 1, 0);
        truncated.truncate(527);
        cases.push((
            "format2_truncated",
            font(vec![(0, 0, truncated)]),
            'A',
            false,
        ));
        let mut out_of_bounds = cmap2(65, 1, 0);
        out_of_bounds[524..526].copy_from_slice(&65534_u16.to_be_bytes());
        cases.push((
            "format2_bad_offset",
            font(vec![(0, 0, out_of_bounds)]),
            'A',
            false,
        ));
        cases.push((
            "zero_then_later_record",
            font(vec![(0, 0, cmap2(65, 0, 0)), (3, 10, range(12, 65, 1))]),
            'A',
            true,
        ));
        cases
    }

    #[test]
    fn unicode_coverage_preserves_formats_records_and_legacy_edge_results() {
        for (label, data, ch, expected) in cases() {
            let mut db = usvg::fontdb::Database::new();
            db.load_font_data(data);
            let id = db.faces().next().unwrap().id;
            assert_eq!(font_has_char(&db, id, ch), expected, "{label}");
        }
    }
    #[test]
    fn compare_all_edges_with_original_parser() {
        for (label, data, ch, expected) in cases() {
            let old = ttf_parser::Face::parse(&data, 0).unwrap().glyph_index(ch).is_some();
            let mut db = usvg::fontdb::Database::new();
            db.load_font_data(data);
            let id = db.faces().next().unwrap().id;
            let repaired = font_has_char(&db, id, ch);
            println!("{label}: old={old}, repaired={repaired}, expected={expected}");
            assert_eq!((old, repaired), (expected, expected), "{label}");
        }
    }

}

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
            db.load_font_data(include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../vendor/fixtures/fonts/lilex/Lilex-Regular.ttf")).to_vec());
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
