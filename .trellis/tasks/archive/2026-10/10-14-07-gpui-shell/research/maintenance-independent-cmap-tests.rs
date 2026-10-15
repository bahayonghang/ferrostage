#[cfg(test)]
mod maintenance_cmap_tests {
    use super::*;
    const IBM: &[u8] =
        include_bytes!("../../../assets/fonts/ibm-plex-sans/IBMPlexSans-Regular.ttf");

    fn word(out: &mut Vec<u8>, n: u16) {
        out.extend(n.to_be_bytes());
    }
    fn long(out: &mut Vec<u8>, n: u32) {
        out.extend(n.to_be_bytes());
    }

    fn range(format: u16, code: u32, glyph: u16) -> Vec<u8> {
        let mut out = Vec::new();
        for n in [format, 0] {
            word(&mut out, n);
        }
        for n in [28, 0, 1, code, code, u32::from(glyph)] {
            long(&mut out, n);
        }
        out
    }

    fn cmap2(code: u16, glyph: u16, delta: i16) -> Vec<u8> {
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
    fn font(records: Vec<(u16, u16, Vec<u8>)>) -> Vec<u8> {
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
    fn legacy_cmap_emoji_fallback_produces_the_same_real_outline() {
        use skrifa::MetadataProvider;
        use std::sync::atomic::{AtomicUsize, Ordering};
        let glyph = skrifa::FontRef::from_index(IBM, 0)
            .unwrap()
            .charmap()
            .map('A')
            .unwrap()
            .to_u32() as u16;
        let mut images = Vec::new();
        for (encoding, table) in [(6, range(13, 0x2705, glyph)), (0, cmap2(0x2705, glyph, 0))] {
            let mut db = usvg::fontdb::Database::new();
            db.load_font_data(
                include_bytes!("../../../assets/fonts/lilex/Lilex-Regular.ttf").to_vec(),
            );
            db.load_font_data(font(vec![(0, encoding, table)]));
            let calls = Arc::new(AtomicUsize::new(0));
            let observed = calls.clone();
            let renderer = SvgRenderer {
                asset_source: Arc::new(()),
                usvg_options: Arc::new(usvg::Options {
                    fontdb: Arc::new(db),
                    font_resolver: usvg::FontResolver {
                        select_font: usvg::FontResolver::default_font_selector(),
                        select_fallback: Box::new(move |ch, used, db| {
                            assert!(is_emoji_presentation(ch));
                            observed.fetch_add(1, Ordering::Relaxed);
                            select_emoji_font(ch, used, db, &["IBM Plex Sans", "Lilex"])
                        }),
                    },
                    ..Default::default()
                }),
            };
            let svg = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"128\" height=\"64\"><text x=\"8\" y=\"48\" font-family=\"Lilex\" font-size=\"40\">✅</text></svg>";
            let parsed = renderer.parse_svg(svg.as_bytes()).unwrap();
            let image = renderer
                .render_parsed(
                    &parsed,
                    SvgSize::ExactSize(Size::new(DevicePixels(128), DevicePixels(64))),
                )
                .unwrap();
            assert!(
                image
                    .as_bytes(0)
                    .unwrap()
                    .chunks_exact(4)
                    .filter(|p| p[3] != 0)
                    .count()
                    > 40
            );
            assert_eq!(calls.load(Ordering::Relaxed), 1);
            images.push(image.as_bytes(0).unwrap().to_vec());
        }
        assert_eq!(images[0], images[1]);
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
}
