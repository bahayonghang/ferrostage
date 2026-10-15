use crate::hb::tables::{find_best_cmap_subtable, SelectedCmapSubtable, TableRanges};

use super::cache::hb_cache_t;
use read_fonts::{
    tables::cmap::{Cmap, Cmap14, CmapSubtable, MapVariant, PlatformId},
    types::GlyphId,
    FontRef, TableProvider,
};

pub type cache_t = hb_cache_t<21, 19, 256, 32>;

#[derive(Clone)]
pub struct Charmap<'a> {
    subtable: Option<(SelectedCmapSubtable, CmapSubtable<'a>)>,
    vs_subtable: Option<Cmap14<'a>>,
}

impl<'a> Charmap<'a> {
    pub fn new(font: &FontRef<'a>, table_ranges: &TableRanges) -> Self {
        if let Some(cmap) = table_ranges.cmap.resolve_table::<Cmap>(font) {
            let data = cmap.offset_data();
            let records = cmap.encoding_records();
            let subtable = table_ranges
                .cmap_subtable
                .and_then(|s| Some((s, records.get(s.index as usize)?.subtable(data).ok()?)));
            let vs_subtable = table_ranges
                .cmap_vs_subtable
                .and_then(|index| records.get(index as usize))
                .and_then(|rec| rec.subtable(data).ok())
                .and_then(|subtable| match subtable {
                    CmapSubtable::Format14(table) => Some(table),
                    _ => None,
                });
            Self {
                subtable,
                vs_subtable,
            }
        } else {
            Self {
                subtable: None,
                vs_subtable: None,
            }
        }
    }

    pub fn from_tables(font: &impl TableProvider<'a>) -> Self {
        if let Ok(cmap) = font.cmap() {
            let subtable = if let Some((index, platform, encoding, subtable)) =
                find_best_cmap_subtable(&cmap)
            {
                Some((
                    SelectedCmapSubtable {
                        index,
                        is_mac_roman: platform == PlatformId::Macintosh,
                        is_symbol: platform == PlatformId::Windows && encoding == 0,
                    },
                    subtable,
                ))
            } else {
                None
            };
            Self {
                subtable,
                vs_subtable: cmap.uvs_subtable().map(|(_, subtable)| subtable),
            }
        } else {
            Self {
                subtable: None,
                vs_subtable: None,
            }
        }
    }

    pub fn map(&self, mut c: u32) -> Option<GlyphId> {
        let subtable = self.subtable.as_ref()?;
        if subtable.0.is_mac_roman && c > 0x7F {
            c = unicode_to_macroman(c);
        }
        let result = match &subtable.1 {
            CmapSubtable::Format0(table) => table.map_codepoint(c),
            CmapSubtable::Format2(table) => cmap2_glyph(table, c),
            CmapSubtable::Format6(table) => table.map_codepoint(c),
            CmapSubtable::Format10(table) => {
                if let Some(index) = c.checked_sub(table.start_char_code()) {
                    if index < table.num_chars() {
                        table
                            .glyph_id_array()
                            .get(index as usize)
                            .map(|gid| GlyphId::from(gid.get()))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            CmapSubtable::Format4(table) => table.map_codepoint(c),
            CmapSubtable::Format12(table) => table.map_codepoint(c),
            CmapSubtable::Format13(table) => table.map_codepoint(c),
            _ => None,
        };
        if result.is_none() && subtable.0.is_symbol && c <= 0x00FF {
            // For symbol-encoded OpenType fonts, we duplicate the
            // U+F000..F0FF range at U+0000..U+00FF.  That's what
            // Windows seems to do, and that's hinted about at:
            // https://docs.microsoft.com/en-us/typography/opentype/spec/recom
            // under "Non-Standard (Symbol) Fonts".
            return self.map(0xF000 + c);
        }
        result
    }

    pub fn map_variant(&self, c: u32, vs: u32) -> Option<GlyphId> {
        let subtable = self.vs_subtable.as_ref()?;
        match subtable.map_variant(c, vs)? {
            MapVariant::UseDefault => self.map(c),
            MapVariant::Variant(gid) => Some(gid),
        }
    }
}

#[rustfmt::skip]
static UNICODE_TO_MACROMAN: &[u16] = &[
    0x00C4, 0x00C5, 0x00C7, 0x00C9, 0x00D1, 0x00D6, 0x00DC, 0x00E1,
    0x00E0, 0x00E2, 0x00E4, 0x00E3, 0x00E5, 0x00E7, 0x00E9, 0x00E8,
    0x00EA, 0x00EB, 0x00ED, 0x00EC, 0x00EE, 0x00EF, 0x00F1, 0x00F3,
    0x00F2, 0x00F4, 0x00F6, 0x00F5, 0x00FA, 0x00F9, 0x00FB, 0x00FC,
    0x2020, 0x00B0, 0x00A2, 0x00A3, 0x00A7, 0x2022, 0x00B6, 0x00DF,
    0x00AE, 0x00A9, 0x2122, 0x00B4, 0x00A8, 0x2260, 0x00C6, 0x00D8,
    0x221E, 0x00B1, 0x2264, 0x2265, 0x00A5, 0x00B5, 0x2202, 0x2211,
    0x220F, 0x03C0, 0x222B, 0x00AA, 0x00BA, 0x03A9, 0x00E6, 0x00F8,
    0x00BF, 0x00A1, 0x00AC, 0x221A, 0x0192, 0x2248, 0x2206, 0x00AB,
    0x00BB, 0x2026, 0x00A0, 0x00C0, 0x00C3, 0x00D5, 0x0152, 0x0153,
    0x2013, 0x2014, 0x201C, 0x201D, 0x2018, 0x2019, 0x00F7, 0x25CA,
    0x00FF, 0x0178, 0x2044, 0x20AC, 0x2039, 0x203A, 0xFB01, 0xFB02,
    0x2021, 0x00B7, 0x201A, 0x201E, 0x2030, 0x00C2, 0x00CA, 0x00C1,
    0x00CB, 0x00C8, 0x00CD, 0x00CE, 0x00CF, 0x00CC, 0x00D3, 0x00D4,
    0xF8FF, 0x00D2, 0x00DA, 0x00DB, 0x00D9, 0x0131, 0x02C6, 0x02DC,
    0x00AF, 0x02D8, 0x02D9, 0x02DA, 0x00B8, 0x02DD, 0x02DB, 0x02C7,
];

fn unicode_to_macroman(c: u32) -> u32 {
    let u = c as u16;
    let Some(index) = UNICODE_TO_MACROMAN.iter().position(|m| *m == u) else {
        return 0;
    };
    (0x80 + index) as u32
}

// read-fonts exposes checked format-2 records but has no map_codepoint for it.
// OpenType cmap format 2 specifies offsets relative to the idRangeOffset word:
// https://learn.microsoft.com/en-us/typography/opentype/spec/cmap#format-2-high-byte-mapping-through-table
fn cmap2_glyph(table: &read_fonts::tables::cmap::Cmap2<'_>, ch: u32) -> Option<GlyphId> {
    use read_fonts::tables::cmap::SubHeader;
    let code = u16::try_from(ch).ok()?;
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
    u16::try_from((i32::from(glyph) + i32::from(header.id_delta())) % 65536)
        .ok()
        .map(GlyphId::from)
}

#[cfg(test)]
mod maintenance_cmap2_tests {
    use super::*;
    const IBM: &[u8] =
        include_bytes!("../../../fixtures/fonts/ibm-plex-sans/IBMPlexSans-Regular.ttf");

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
    fn both_constructors_keep_format2_glyph_and_failure_results() {
        for (label, data, ch, expected) in
            cases().into_iter().filter(|c| c.0.starts_with("format2"))
        {
            let font = FontRef::from_index(&data, 0).unwrap();
            let ranges = TableRanges::new(&font);
            let cached = Charmap::new(&font, &ranges).map(u32::from(ch));
            let alternate = Charmap::from_tables(&font).map(u32::from(ch));
            let expected_gid = match label {
                "format2_zero_after_delta" | "format2_wrap_delta" => Some(GlyphId::new(0)),
                _ if expected => Some(GlyphId::new(1)),
                _ => None,
            };
            assert_eq!(cached, expected_gid, "cached {label}");
            assert_eq!(alternate, expected_gid, "alternate {label}");
        }
    }
}
