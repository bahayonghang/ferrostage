from pathlib import Path

ROOT = Path.cwd()
R = ROOT / '.trellis/tasks/10-14-07-gpui-shell/research'
proof = ROOT / 'cache/gpui-maintenance-cmap-proof'
manifest = (proof / 'Cargo.toml').read_text(encoding='utf-8')
if 'fontdb =' not in manifest:
    manifest += '\nfontdb = "=0.24.0"\n'
if 'resvg_old =' not in manifest:
    manifest += '\nresvg_old = { package = "resvg", version = "=0.46.0", default-features = false, features = ["text"] }\nresvg_new = { package = "resvg", version = "=0.48.1", default-features = false, features = ["text"] }\n'
if '[patch.crates-io]' not in manifest:
    manifest += '\n[patch.crates-io]\nharfrust = { path = "../gpui-maintenance-candidate/vendor/harfrust" }\n'
(proof / 'Cargo.toml').write_text(manifest, encoding='utf-8')
original = (R / 'maintenance-independent-cmap-probe.rs').read_text(encoding='utf-8')
original = original.replace('use skrifa::MetadataProvider;', 'use skrifa::raw::TableProvider;\nmod usvg { pub use fontdb; }')
original = original.replace('let candidate = skrifa::FontRef::from_index(&data, 0).unwrap().charmap().map(ch).is_some();', '''let mut db = fontdb::Database::new();
        db.load_font_data(data);
        let id = db.faces().next().unwrap().id;
        let candidate = font_has_char(&db, id, ch);''')
actual = (ROOT / 'cache/gpui-maintenance-candidate/vendor/gpui/src/svg_renderer.rs').read_text(encoding='utf-8')
helper = actual[actual.index('fn font_has_char('):actual.index('\nfn select_emoji_font(')]
tests = (R / 'maintenance-independent-cmap-tests.rs').read_text(encoding='utf-8')
first = tests.index('    #[test]')
second = tests.index('    #[test]', first + 1)
tests = tests[:first] + tests[second:]
tests = tests.replace('    fn range(', '    pub(super) fn range(').replace('    fn cmap2(', '    pub(super) fn cmap2(').replace('    fn font(', '    pub(super) fn font(')
tests = tests.replace('include_bytes!("../../../assets/fonts/ibm-plex-sans/IBMPlexSans-Regular.ttf")',
                      'include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../gpui-maintenance-candidate/assets/fonts/ibm-plex-sans/IBMPlexSans-Regular.ttf"))')
position = tests.rindex('\n}')
tests = tests[:position] + '''
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
''' + tests[position:]
(proof / 'src/main.rs').write_text(original + '\n' + helper + '\n' + tests + '\n' + (R / 'maintenance-independent-render-probe.rs').read_text(), encoding='utf-8')
print('Prepared original five-case red probe plus expanded old/repaired edge comparisons.')
