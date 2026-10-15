"""Apply the separately approved two-file harfrust0.12 compatibility repair."""
from pathlib import Path
import difflib
import json
import shutil
import sys
import subprocess

ROOT = Path.cwd()
R = ROOT / '.trellis/tasks/10-14-07-gpui-shell/research'
DEST = ROOT / (sys.argv[1] if len(sys.argv) > 1 else 'cache/gpui-maintenance-candidate')
assert DEST.resolve().is_relative_to((ROOT / 'cache').resolve()) and DEST.name.startswith('gpui-maintenance-')
SOURCE = Path('C:/Users/lyh/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/harfrust-0.12.0')
target = DEST / 'vendor/harfrust'
shutil.copytree(SOURCE, target, dirs_exist_ok=True)
(target / 'Cargo.toml').write_text((SOURCE / 'Cargo.toml').read_text() + '\n[workspace]\n', encoding='utf-8', newline='\n')
patches = []
tables = (SOURCE / 'src/hb/tables.rs').read_text()
updated_tables = tables.replace("fn find_best_cmap_subtable<'a>(", "pub(super) fn find_best_cmap_subtable<'a>(")
assert 'pub(super) fn find_best_cmap_subtable' in updated_tables
updated_tables = updated_tables.replace('CmapSubtable::Format0(_)\n', 'CmapSubtable::Format0(_)\n                | CmapSubtable::Format2(_)\n')
assert updated_tables != tables
charmap = (SOURCE / 'src/hb/charmap.rs').read_text()
updated_charmap = charmap.replace('use crate::hb::tables::{SelectedCmapSubtable, TableRanges};',
    'use crate::hb::tables::{find_best_cmap_subtable, SelectedCmapSubtable, TableRanges};')
updated_charmap = updated_charmap.replace('cmap::{Cmap, Cmap14, CmapSubtable, MapVariant}',
    'cmap::{Cmap, Cmap14, CmapSubtable, MapVariant, PlatformId}')
updated_charmap = updated_charmap.replace('Some((index, record, subtable)) = cmap.best_subtable()',
    'Some((index, platform, encoding, subtable)) = find_best_cmap_subtable(&cmap)')
updated_charmap = updated_charmap.replace('is_mac_roman: record.is_mac_roman(),', 'is_mac_roman: platform == PlatformId::Macintosh,')
updated_charmap = updated_charmap.replace('is_symbol: record.is_symbol(),', 'is_symbol: platform == PlatformId::Windows && encoding == 0,')
updated_charmap = updated_charmap.replace('CmapSubtable::Format0(table) => table.map_codepoint(c),',
    'CmapSubtable::Format0(table) => table.map_codepoint(c),\n            CmapSubtable::Format2(table) => cmap2_glyph(table, c),')
helper = (R / 'maintenance-independent-font-helper.rs').read_text()
adapter = helper[helper.index('// read-fonts exposes checked format-2'):]
adapter = adapter.replace('skrifa::raw::', 'read_fonts::').replace('ch: char) -> Option<u16>', 'ch: u32) -> Option<GlyphId>')
adapter = adapter.replace('u16::try_from(u32::from(ch))', 'u16::try_from(ch)')
adapter = adapter.replace('% 65536).ok()', '% 65536).ok().map(GlyphId::from)')
updated_charmap += '\n' + adapter
fixtures = (R / 'maintenance-independent-cmap-tests.rs').read_text()
fixtures = fixtures[:fixtures.index('    #[test]')]
fixtures = fixtures.replace('mod maintenance_cmap_tests', 'mod maintenance_cmap2_tests')
fixtures = fixtures.replace('../../../assets/', '../../../../assets/')
fixtures += '''    #[test]
    fn both_constructors_keep_format2_glyph_and_failure_results() {
        for (label, data, ch, expected) in cases().into_iter().filter(|c| c.0.starts_with("format2")) {
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
'''
updated_charmap += '\n' + fixtures
for rel, before, after in [('src/hb/tables.rs', tables, updated_tables), ('src/hb/charmap.rs', charmap, updated_charmap)]:
    (target / rel).write_text(after, encoding='utf-8', newline='\n')
    subprocess.run(['rustfmt', '--edition', '2021', str(target / rel)], check=True)
    formatted = (target / rel).read_text()
    patches.extend(difflib.unified_diff(before.splitlines(True), formatted.splitlines(True), fromfile=f'a/harfrust/{rel}', tofile=f'b/harfrust/{rel}'))
(target / 'FERROSTAGE-MAINTENANCE.txt').write_text(
    'Isolated MIT harfrust0.12.0 compatibility evaluation. Original license retained.\n'
    'Two internal files add checked format2 cmap selection/mapping to both constructors.\n'
    'No feature, shaping algorithm, public API, or encoding-priority change.\n', encoding='utf-8')
for relative, package_path in [('Cargo.toml', 'vendor/harfrust'), ('vendor/gpui/Cargo.toml', '../harfrust'), ('validation/wgpu-consumer/Cargo.toml', '../../vendor/harfrust')]:
    path = DEST / relative
    content = path.read_text()
    line = f'harfrust = {{ path = "{package_path}" }}\n'
    if line not in content:
        path.write_text(content.replace('[patch.crates-io]\n', '[patch.crates-io]\n' + line), encoding='utf-8', newline='\n')
patch_out = ROOT / sys.argv[2] if len(sys.argv) > 2 else R
(patch_out / 'maintenance-independent-harfrust-repair.patch').write_text(''.join(patches), encoding='utf-8', newline='\n')
print(json.dumps({'harfrust_copy': str(target), 'source_files_modified': ['src/hb/tables.rs', 'src/hb/charmap.rs']}))
