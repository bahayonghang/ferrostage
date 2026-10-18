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
