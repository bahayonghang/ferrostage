use skrifa::MetadataProvider;

const IBM: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../gpui-maintenance-candidate/assets/fonts/ibm-plex-sans/IBMPlexSans-Regular.ttf"));

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
        let candidate = skrifa::FontRef::from_index(&data, 0).unwrap().charmap().map(ch).is_some();
        println!("{label}: old={old}, candidate={candidate}");
        if old != candidate { differences.push(label); }
    }
    assert!(differences.is_empty(), "behavior changed for {differences:?}");
}

fn main() {}
