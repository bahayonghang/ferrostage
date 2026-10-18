use crate::{
    AssetSource, DevicePixels, IsZero, RenderImage, Result, SharedString, Size,
    swap_rgba_pa_to_bgra,
};
use image::Frame;
use resvg::tiny_skia::Pixmap;
use skrifa::raw::TableProvider;
use smallvec::SmallVec;
use std::{
    hash::Hash,
    sync::{Arc, LazyLock, OnceLock},
};

#[cfg(target_os = "macos")]
const EMOJI_FONT_FAMILIES: &[&str] = &["Apple Color Emoji", ".AppleColorEmojiUI"];

#[cfg(target_os = "windows")]
const EMOJI_FONT_FAMILIES: &[&str] = &["Segoe UI Emoji", "Segoe UI Symbol"];

#[cfg(any(target_os = "linux", target_os = "freebsd"))]
const EMOJI_FONT_FAMILIES: &[&str] = &[
    "Noto Color Emoji",
    "Emoji One",
    "Twitter Color Emoji",
    "JoyPixels",
];

#[cfg(not(any(
    target_os = "macos",
    target_os = "windows",
    target_os = "linux",
    target_os = "freebsd",
)))]
const EMOJI_FONT_FAMILIES: &[&str] = &[];

fn is_emoji_presentation(c: char) -> bool {
    static EMOJI_PRESENTATION_REGEX: LazyLock<regex::Regex> =
        LazyLock::new(|| regex::Regex::new("\\p{Emoji_Presentation}").unwrap());
    let mut buf = [0u8; 4];
    EMOJI_PRESENTATION_REGEX.is_match(c.encode_utf8(&mut buf))
}

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

fn select_emoji_font(
    ch: char,
    fonts: &[usvg::fontdb::ID],
    db: &usvg::fontdb::Database,
    families: &[&str],
) -> Option<usvg::fontdb::ID> {
    for family_name in families {
        let query = usvg::fontdb::Query {
            families: &[usvg::fontdb::Family::Name(family_name)],
            weight: usvg::fontdb::Weight(400),
            stretch: usvg::fontdb::Stretch::Normal,
            style: usvg::fontdb::Style::Normal,
        };

        let Some(id) = db.query(&query) else {
            continue;
        };

        if fonts.contains(&id) || !font_has_char(db, id, ch) {
            continue;
        }

        return Some(id);
    }

    None
}

/// When rendering SVGs, we render them at twice the size to get a higher-quality result.
pub const SMOOTH_SVG_SCALE_FACTOR: f32 = 2.;

#[derive(Clone, PartialEq, Hash, Eq)]
#[expect(missing_docs)]
pub struct RenderSvgParams {
    pub path: SharedString,
    pub size: Size<DevicePixels>,
}

#[derive(Clone)]
/// A struct holding everything necessary to render SVGs.
pub struct SvgRenderer {
    asset_source: Arc<dyn AssetSource>,
    usvg_options: Arc<usvg::Options<'static>>,
}

/// A parsed SVG document that can be rasterized at any scale.
///
/// Produced by [`SvgRenderer::parse_svg`] and rasterized by
/// [`SvgRenderer::render_parsed`]. Parsing resolves fonts and converts text
/// to paths, so callers that need to rasterize the same SVG at multiple
/// scales should retain this value to avoid re-paying the parse cost.
pub struct ParsedSvg(usvg::Tree);

/// The size in which to rasterize the SVG.
#[derive(Clone, Copy)]
pub enum SvgSize {
    /// A width in device pixels. The SVG retains its aspect ratio.
    Size(Size<DevicePixels>),
    /// An exact width and height in device pixels.
    ExactSize(Size<DevicePixels>),
    /// A logical scaling factor to apply to the size provided by the SVG.
    ScaleFactor(f32),
}

impl From<f32> for SvgSize {
    fn from(scale_factor: f32) -> Self {
        Self::ScaleFactor(scale_factor)
    }
}

impl SvgRenderer {
    /// Creates a new SVG renderer with the provided asset source.
    pub fn new(asset_source: Arc<dyn AssetSource>) -> Self {
        static SYSTEM_FONT_DB: LazyLock<Arc<usvg::fontdb::Database>> = LazyLock::new(|| {
            let mut db = usvg::fontdb::Database::new();
            db.load_system_fonts();
            Arc::new(db)
        });

        // Build the enriched font DB lazily on first SVG render rather than
        // eagerly at construction time. This avoids the expensive deep-clone
        // of the system font database for code paths that never render SVGs
        // (e.g. tests).
        let enriched_fontdb: Arc<OnceLock<Arc<usvg::fontdb::Database>>> = Arc::new(OnceLock::new());

        let default_font_resolver = usvg::FontResolver::default_font_selector();
        let font_resolver = Box::new({
            let asset_source = asset_source.clone();
            move |font: &usvg::Font, db: &mut Arc<usvg::fontdb::Database>| {
                if db.is_empty() {
                    let fontdb = enriched_fontdb.get_or_init(|| {
                        let mut db = (**SYSTEM_FONT_DB).clone();
                        load_bundled_fonts(&*asset_source, &mut db);
                        fix_generic_font_families(&mut db);
                        Arc::new(db)
                    });
                    *db = fontdb.clone();
                }
                if let Some(id) = default_font_resolver(font, db) {
                    return Some(id);
                }
                // fontdb doesn't recognize CSS system font keywords like "system-ui"
                // or "ui-sans-serif", so fall back to sans-serif before any face.
                let sans_query = usvg::fontdb::Query {
                    families: &[usvg::fontdb::Family::SansSerif],
                    ..Default::default()
                };
                db.query(&sans_query)
                    .or_else(|| db.faces().next().map(|f| f.id))
            }
        });
        let default_fallback_selection = usvg::FontResolver::default_fallback_selector();
        let fallback_selection = Box::new(
            move |ch: char, fonts: &[usvg::fontdb::ID], db: &mut Arc<usvg::fontdb::Database>| {
                if is_emoji_presentation(ch) {
                    if let Some(id) = select_emoji_font(ch, fonts, db.as_ref(), EMOJI_FONT_FAMILIES)
                    {
                        return Some(id);
                    }
                }

                default_fallback_selection(ch, fonts, db)
            },
        );
        let options = usvg::Options {
            font_resolver: usvg::FontResolver {
                select_font: font_resolver,
                select_fallback: fallback_selection,
            },
            ..Default::default()
        };
        Self {
            asset_source,
            usvg_options: Arc::new(options),
        }
    }

    /// Parses SVG data into a [`ParsedSvg`] that can be rasterized at any scale.
    pub fn parse_svg(&self, bytes: &[u8]) -> Result<ParsedSvg, usvg::Error> {
        usvg::Tree::from_data(bytes, &self.usvg_options).map(ParsedSvg)
    }

    /// Rasterizes a previously parsed SVG into an image buffer.
    pub fn render_parsed(
        &self,
        svg: &ParsedSvg,
        size: impl Into<SvgSize>,
    ) -> Result<Arc<RenderImage>, usvg::Error> {
        let (size, image_scale_factor) = match size.into() {
            SvgSize::Size(size) => (SvgSize::Size(size), 1.0),
            SvgSize::ExactSize(size) => (SvgSize::ExactSize(size), 1.0),
            SvgSize::ScaleFactor(scale_factor) => (
                SvgSize::ScaleFactor(scale_factor * SMOOTH_SVG_SCALE_FACTOR),
                SMOOTH_SVG_SCALE_FACTOR,
            ),
        };
        let pixmap = rasterize_tree(&svg.0, size)?;
        let mut buffer =
            image::ImageBuffer::from_raw(pixmap.width(), pixmap.height(), pixmap.take()).unwrap();

        for pixel in buffer.chunks_exact_mut(4) {
            swap_rgba_pa_to_bgra(pixel);
        }

        let mut image = RenderImage::new(SmallVec::from_const([Frame::new(buffer)]));
        image.scale_factor = image_scale_factor;
        Ok(Arc::new(image))
    }

    /// Renders the given bytes into an image buffer.
    pub fn render_single_frame(
        &self,
        bytes: &[u8],
        scale_factor: f32,
    ) -> Result<Arc<RenderImage>, usvg::Error> {
        let svg = self.parse_svg(bytes)?;
        self.render_parsed(&svg, scale_factor)
    }

    pub(crate) fn render_alpha_mask(
        &self,
        params: &RenderSvgParams,
        bytes: Option<&[u8]>,
    ) -> Result<Option<(Size<DevicePixels>, Vec<u8>)>> {
        anyhow::ensure!(!params.size.is_zero(), "can't render at a zero size");

        let render_pixmap = |bytes| {
            let pixmap = self.render_pixmap(bytes, SvgSize::Size(params.size))?;

            // Convert the pixmap's pixels into an alpha mask.
            let size = Size::new(
                DevicePixels(pixmap.width() as i32),
                DevicePixels(pixmap.height() as i32),
            );
            let alpha_mask = pixmap
                .pixels()
                .iter()
                .map(|p| p.alpha())
                .collect::<Vec<_>>();

            Ok(Some((size, alpha_mask)))
        };

        if let Some(bytes) = bytes {
            render_pixmap(bytes)
        } else if let Some(bytes) = self.asset_source.load(&params.path)? {
            render_pixmap(&bytes)
        } else {
            Ok(None)
        }
    }

    fn render_pixmap(&self, bytes: &[u8], size: SvgSize) -> Result<Pixmap, usvg::Error> {
        let tree = usvg::Tree::from_data(bytes, &self.usvg_options)?;
        rasterize_tree(&tree, size)
    }
}

fn rasterize_tree(tree: &usvg::Tree, size: SvgSize) -> Result<Pixmap, usvg::Error> {
    // Cap the size of the rendered pixmap to avoid texture allocation panics
    // Related issue: #56466
    const MAX_SIZE: f32 = 8192.0;

    let svg_size = tree.size();
    let (mut width, mut height) = match size {
        SvgSize::Size(size) => {
            let scale = i32::from(size.width) as f32 / svg_size.width();
            (svg_size.width() * scale, svg_size.height() * scale)
        }
        SvgSize::ExactSize(size) => (i32::from(size.width) as f32, i32::from(size.height) as f32),
        SvgSize::ScaleFactor(scale) => (svg_size.width() * scale, svg_size.height() * scale),
    };

    if width > MAX_SIZE {
        log::warn!("Attempted to render pixmap where width ({width}) > MAX_SIZE ({MAX_SIZE})");
    }
    if height > MAX_SIZE {
        log::warn!("Attempted to render pixmap where height ({height}) > MAX_SIZE ({MAX_SIZE})");
    }
    let scale = (MAX_SIZE / width).min(MAX_SIZE / height).min(1.0);
    width *= scale;
    height *= scale;

    // Render the SVG to a pixmap with the specified width and height.
    let mut pixmap = resvg::tiny_skia::Pixmap::new(width as u32, height as u32)
        .ok_or(usvg::Error::InvalidSize)?;

    let transform = resvg::tiny_skia::Transform::from_scale(
        width / svg_size.width(),
        height / svg_size.height(),
    );

    resvg::render(tree, transform, &mut pixmap.as_mut());

    Ok(pixmap)
}

fn load_bundled_fonts(asset_source: &dyn AssetSource, db: &mut usvg::fontdb::Database) {
    let font_paths = [
        "fonts/ibm-plex-sans/IBMPlexSans-Regular.ttf",
        "fonts/lilex/Lilex-Regular.ttf",
    ];
    for path in font_paths {
        match asset_source.load(path) {
            Ok(Some(data)) => db.load_font_data(data.into_owned()),
            Ok(None) => log::warn!("Bundled font not found: {path}"),
            Err(error) => log::warn!("Failed to load bundled font {path}: {error}"),
        }
    }
}

// fontdb defaults generic families to Microsoft fonts ("Arial", "Times New Roman")
// which aren't installed on most Linux systems. fontconfig normally overrides these,
// but when it fails the defaults remain and all generic family queries return None.
fn fix_generic_font_families(db: &mut usvg::fontdb::Database) {
    use usvg::fontdb::{Family, Query};

    let families_and_fallbacks: &[(Family<'_>, &str)] = &[
        (Family::SansSerif, "IBM Plex Sans"),
        // No serif font bundled; use sans-serif as best available fallback.
        (Family::Serif, "IBM Plex Sans"),
        (Family::Monospace, "Lilex"),
        (Family::Cursive, "IBM Plex Sans"),
        (Family::Fantasy, "IBM Plex Sans"),
    ];

    for (family, fallback_name) in families_and_fallbacks {
        let query = Query {
            families: &[*family],
            ..Default::default()
        };
        if db.query(&query).is_none() {
            match family {
                Family::SansSerif => db.set_sans_serif_family(*fallback_name),
                Family::Serif => db.set_serif_family(*fallback_name),
                Family::Monospace => db.set_monospace_family(*fallback_name),
                Family::Cursive => db.set_cursive_family(*fallback_name),
                Family::Fantasy => db.set_fantasy_family(*fallback_name),
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use usvg::fontdb::{Database, Family, Query};

    const IBM_PLEX_REGULAR: &[u8] =
        include_bytes!("../../fixtures/fonts/ibm-plex-sans/IBMPlexSans-Regular.ttf");
    const LILEX_REGULAR: &[u8] = include_bytes!("../../fixtures/fonts/lilex/Lilex-Regular.ttf");

    #[test]
    fn renders_parsed_svg_at_requested_size() -> Result<()> {
        let renderer = SvgRenderer::new(Arc::new(()));
        let svg = renderer.parse_svg(
            br#"<svg xmlns="http://www.w3.org/2000/svg" width="24pt" height="12pt"></svg>"#,
        )?;
        let requested_size = Size::new(DevicePixels(24), DevicePixels(12));
        let image = renderer.render_parsed(&svg, SvgSize::ExactSize(requested_size))?;

        assert_eq!(image.size(0), requested_size);
        Ok(())
    }

    #[test]
    fn preserves_aspect_ratio_for_width_constrained_size() -> Result<()> {
        let renderer = SvgRenderer::new(Arc::new(()));
        let svg = renderer.parse_svg(
            br#"<svg xmlns="http://www.w3.org/2000/svg" width="24pt" height="12pt"></svg>"#,
        )?;
        let image = renderer.render_parsed(
            &svg,
            SvgSize::Size(Size::new(DevicePixels(24), DevicePixels(24))),
        )?;

        assert_eq!(image.size(0), Size::new(DevicePixels(24), DevicePixels(12)));
        Ok(())
    }

    fn db_with_bundled_fonts() -> Database {
        let mut db = Database::new();
        db.load_font_data(IBM_PLEX_REGULAR.to_vec());
        db.load_font_data(LILEX_REGULAR.to_vec());
        db
    }

    #[test]
    fn text_with_split_glyph_clusters_in_mixed_fonts_does_not_panic() {
        let mut db = Database::new();
        db.load_font_data(IBM_PLEX_REGULAR.to_vec());
        db.load_font_data(LILEX_REGULAR.to_vec());
        let options = usvg::Options {
            fontdb: std::sync::Arc::new(db),
            ..Default::default()
        };

        // A base letter followed by a stack of combining marks. Under HarfBuzz's
        // default cluster merging every mark glyph shares the base's byte index,
        // which is the "glyph splitting" condition that triggered the panic. The
        // chunk must use two different fonts so the buggy merge path runs.
        let zalgo = "e\u{0301}\u{0302}\u{0303}\u{0304}\u{0306}\u{0307}\u{0308}\u{030a}";
        let svg = format!(
            r#"<svg viewBox="0 0 200 200" xmlns="http://www.w3.org/2000/svg"><text font-family="Lilex" font-size="32">{zalgo}<tspan font-family="IBM Plex Sans">{zalgo}</tspan></text></svg>"#
        );

        // Before the fix this aborts via panic with a message like
        // "removal index (is 5) should be < len (is 5)".
        usvg::Tree::from_data(svg.as_bytes(), &options)
            .expect("SVG with mixed-font text should parse");
    }

    #[test]
    fn test_is_emoji_presentation() {
        let cases = [
            ("a", false),
            ("Z", false),
            ("1", false),
            ("#", false),
            ("*", false),
            ("漢", false),
            ("中", false),
            ("カ", false),
            ("©", false),
            ("♥", false),
            ("😀", true),
            ("✅", true),
            ("🇺🇸", true),
            // SVG fallback is not cluster-aware yet
            ("©️", false),
            ("♥️", false),
            ("1️⃣", false),
        ];
        for (s, expected) in cases {
            assert_eq!(
                is_emoji_presentation(s.chars().next().unwrap()),
                expected,
                "for char {:?}",
                s
            );
        }
    }

    #[test]
    fn fix_generic_font_families_sets_all_families() {
        let mut db = db_with_bundled_fonts();
        fix_generic_font_families(&mut db);

        let families = [
            Family::SansSerif,
            Family::Serif,
            Family::Monospace,
            Family::Cursive,
            Family::Fantasy,
        ];

        for family in families {
            let query = Query {
                families: &[family],
                ..Default::default()
            };
            assert!(
                db.query(&query).is_some(),
                "Expected generic family {family:?} to resolve after fix_generic_font_families"
            );
        }
    }

    #[test]
    fn test_select_emoji_font_skips_family_without_glyph() {
        let mut db = db_with_bundled_fonts();

        let ibm_plex_sans = db
            .query(&usvg::fontdb::Query {
                families: &[usvg::fontdb::Family::Name("IBM Plex Sans")],
                weight: usvg::fontdb::Weight(400),
                stretch: usvg::fontdb::Stretch::Normal,
                style: usvg::fontdb::Style::Normal,
            })
            .unwrap();
        let lilex = db
            .query(&usvg::fontdb::Query {
                families: &[usvg::fontdb::Family::Name("Lilex")],
                weight: usvg::fontdb::Weight(400),
                stretch: usvg::fontdb::Stretch::Normal,
                style: usvg::fontdb::Style::Normal,
            })
            .unwrap();
        let selected = select_emoji_font('│', &[], &db, &["IBM Plex Sans", "Lilex"]).unwrap();

        assert_eq!(selected, lilex);
        assert!(!font_has_char(&db, ibm_plex_sans, '│'));
        assert!(font_has_char(&db, selected, '│'));
    }

    #[test]
    fn fix_generic_font_families_monospace_resolves_to_lilex() {
        let mut db = db_with_bundled_fonts();
        fix_generic_font_families(&mut db);

        let query = Query {
            families: &[Family::Monospace],
            ..Default::default()
        };
        let id = db.query(&query).expect("Monospace should resolve");
        let face = db.face(id).expect("Face should exist");
        assert!(
            face.families.iter().any(|(name, _)| name.contains("Lilex")),
            "Monospace should map to Lilex, got {:?}",
            face.families
        );
    }
}

// Isolated evaluation regressions. Never linked into the FerroStage product.
#[cfg(test)]
mod maintenance_tests {
    use super::*;

    const IBM: &[u8] = include_bytes!("../../fixtures/fonts/ibm-plex-sans/IBMPlexSans-Regular.ttf");
    const LILEX: &[u8] = include_bytes!("../../fixtures/fonts/lilex/Lilex-Regular.ttf");
    static COLLECTION: LazyLock<Vec<u8>> = LazyLock::new(|| {
        // Test-only collection; retain original OFL fonts, never distribute a
        // modified RFN-bearing font file. Table offsets are collection-relative.
        let mut bytes = b"ttcf\0\x01\0\0\0\0\0\x02\0\0\0\0\0\0\0\0".to_vec();
        for (index, input) in [IBM, LILEX].into_iter().enumerate() {
            while bytes.len() % 4 != 0 {
                bytes.push(0);
            }
            let offset = bytes.len() as u32;
            bytes[12 + index * 4..16 + index * 4].copy_from_slice(&offset.to_be_bytes());
            let mut font = input.to_vec();
            let tables = u16::from_be_bytes([font[4], font[5]]) as usize;
            for table in 0..tables {
                let entry = 12 + table * 16 + 8;
                let old = u32::from_be_bytes(font[entry..entry + 4].try_into().unwrap());
                font[entry..entry + 4].copy_from_slice(&(old + offset).to_be_bytes());
            }
            bytes.extend(font);
        }
        bytes
    });
    const SHAPE: &[u8] = include_bytes!("../../fixtures/svg/shape.svg");
    const SHAPE_GZIP: &[u8] = include_bytes!("../../fixtures/svg/shape.svgz");

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
        assert!(skrifa::FontRef::from_index(&COLLECTION, 2).is_err());
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

#[cfg(test)]
mod maintenance_cmap_tests {
    use super::*;
    const IBM: &[u8] = include_bytes!("../../fixtures/fonts/ibm-plex-sans/IBMPlexSans-Regular.ttf");

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
                include_bytes!("../../fixtures/fonts/lilex/Lilex-Regular.ttf").to_vec(),
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
