"""Copy only the five approved maintained packages; preserve review evidence."""
from pathlib import Path
import difflib
import hashlib
import json
import shutil

root = Path.cwd()
r = root / '.trellis/tasks/10-14-07-gpui-shell/research'
source = root / 'cache/gpui-maintenance-candidate'
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
assert sha(r / 'maintenance-independent-manifest.json') == '4cc87ec3a4f2fe7bbad8757eefd78869be8ae62d7f73e3a6e027e3201ea23bba'
expected = json.loads((r / 'maintenance-independent-final-source.json').read_text(encoding="utf-8"))['files']
assert all(sha(source / name) == value for name, value in expected.items())
vendor = root / 'vendor'
assert not vendor.exists(), 'Adoption is one-shot; preserve existing product edits.'
for name in ['gpui', 'pulp', 'metal', 'cosmic-text', 'harfrust']:
    shutil.copytree(source / 'vendor' / name, vendor / name)
for family, files in {'ibm-plex-sans': ['IBMPlexSans-Regular.ttf', 'license.txt'],
                      'lilex': ['Lilex-Regular.ttf', 'OFL.txt']}.items():
    target = vendor / 'fixtures/fonts' / family
    target.mkdir(parents=True)
    for name in files:
        shutil.copy2(source / 'assets/fonts' / family / name, target / name)
shutil.copytree(source / 'maintenance-fixtures', vendor / 'fixtures/svg')
ttc = '''static COLLECTION: LazyLock<Vec<u8>> = LazyLock::new(|| {
        // Test-only collection; retain original OFL fonts, never distribute a
        // modified RFN-bearing font file. Table offsets are collection-relative.
        let mut bytes = b"ttcf\\0\\x01\\0\\0\\0\\0\\0\\x02\\0\\0\\0\\0\\0\\0\\0\\0".to_vec();
        for (index, input) in [IBM, LILEX].into_iter().enumerate() {
            while bytes.len() % 4 != 0 { bytes.push(0); }
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
    });'''
patch = []
for name, replacements in {
    'gpui/src/svg_renderer.rs': [
        ('const COLLECTION: &[u8] = include_bytes!("../../../assets/fonts/evaluation.ttc");', ttc),
        ('FontRef::from_index(COLLECTION,', 'FontRef::from_index(&COLLECTION,'),
        ('../../../assets/fonts/', '../../fixtures/fonts/'),
        ('../../../maintenance-fixtures/', '../../fixtures/svg/')],
    'harfrust/src/hb/charmap.rs': [('../../../../assets/fonts/', '../../../fixtures/fonts/')],
}.items():
    path = vendor / name
    old = path.read_text(encoding="utf-8")
    new = old
    for before, after in replacements:
        assert before in new, (name, before)
        new = new.replace(before, after)
    path.write_text(new, encoding='utf-8', newline='\n')
    patch.extend(difflib.unified_diff(old.splitlines(True), new.splitlines(True),
                                     fromfile='reviewed/' + name, tofile='product/' + name))
(r / 'product-vendor-adaptation.patch').write_text(''.join(patch), encoding='utf-8')
files = {p.relative_to(root).as_posix(): sha(p) for p in vendor.rglob('*') if p.is_file()}
(r / 'product-vendor-adoption.json').write_text(json.dumps({
    'reviewed_files_verified': len(expected), 'source': str(source),
    'reviewed_snapshot': 'cc9a8ded6bfeeed1c0286ba5bf2e3ecf750b6ce34987f11d4b6e23d85ec8a960',
    'scope': 'Five approved packages and original OFL regression fonts; test path/in-memory TTC adaptation only. No evaluation app or derived font file.',
    'files': files}, indent=2) + '\n', encoding='utf-8')
print(json.dumps({'reviewed_files_verified': len(expected), 'product_vendor_files': len(files)}))
