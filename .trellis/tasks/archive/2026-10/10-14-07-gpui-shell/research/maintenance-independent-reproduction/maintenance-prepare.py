"""Recreate the bounded candidate from unchanged, hash-recorded Cargo sources.

Run from the FerroStage root. Writes the isolated candidate and maintenance
patch/preparation evidence only. No network, Cargo-cache mutation, product
mutation, or deletion is performed.
"""
import copy
import difflib
import hashlib
import gzip
import json
from pathlib import Path
import shutil
import struct
import tomllib

ROOT = Path.cwd()
RESEARCH = ROOT / '.trellis/tasks/10-14-07-gpui-shell/research/maintenance-independent-reproduction'
DEST = ROOT / 'cache/gpui-maintenance-independent-reproduction'
TAG = 'v1.23.3'
REV = '96a1b1b5389efff473fc2306f7f8c690f2e1a417'
URL = 'https://github.com/zed-industries/zed'
SOURCE = Path('C:/Users/lyh/.cargo/git/checkouts/zed-a70e2ad075855582/96a1b1b')
REGISTRY = Path('C:/Users/lyh/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f')
RETAINED = RESEARCH / 'maintenance-retained'

git_directory = SOURCE / '.git'
git_head = (git_directory / 'HEAD').read_text(encoding='utf-8').strip()
if git_head.startswith('ref: '):
    ref_name = git_head.removeprefix('ref: ')
    git_head = (git_directory / ref_name).read_text(encoding='utf-8').strip()
assert git_head == REV


def read(path):
    return path.read_text(encoding='utf-8')


def write(path, text):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding='utf-8', newline='\n')


def value(item):
    if isinstance(item, dict):
        return '{ ' + ', '.join(f'{json.dumps(k)} = {value(v)}' for k, v in item.items()) + ' }'
    if isinstance(item, list):
        return '[' + ', '.join(value(v) for v in item) + ']'
    return json.dumps(item, ensure_ascii=False)


workspace = tomllib.loads(read(SOURCE / 'Cargo.toml'))['workspace']
original_gpui = read(SOURCE / 'crates/gpui/Cargo.toml')
gpui = tomllib.loads(original_gpui)
assert gpui['package']['license'] == 'Apache-2.0'
needed = set()


def find_inherited(data):
    if isinstance(data, dict):
        for name, child in data.items():
            if isinstance(child, dict) and child.get('workspace') is True and name in workspace['dependencies']:
                needed.add(name)
            find_inherited(child)


find_inherited(gpui)
dependencies = {name: copy.deepcopy(workspace['dependencies'][name]) for name in sorted(needed)}
path_dependencies = []
for name, dependency in dependencies.items():
    if isinstance(dependency, dict) and 'path' in dependency:
        source_manifest = SOURCE / dependency.pop('path') / 'Cargo.toml'
        package = tomllib.loads(read(source_manifest))['package']
        assert package['license'] in ('Apache-2.0', 'MIT', 'MIT OR Apache-2.0'), (name, package)
        path_dependencies.append({'name': name, 'license': package['license'], 'source_manifest': str(source_manifest)})
        dependency.update(git=URL, tag=TAG)

# Keep all existing resvg behavior, including SVGZ decoding, now an explicit gate.
dependencies['resvg'] = {'version': '=0.48.1', 'default-features': False,
                         'features': ['text', 'system-fonts', 'memmap-fonts', 'raster-images', 'svgz']}
dependencies['usvg'] = {'version': '=0.48.1', 'default-features': False}
dependencies['skrifa'] = {'version': '=0.44.0', 'default-features': False, 'features': ['std']}

shutil.copytree(SOURCE / 'crates/gpui', DEST / 'vendor/gpui', dirs_exist_ok=True)
for family, files in {
    'ibm-plex-sans': ['IBMPlexSans-Regular.ttf', 'license.txt'],
    'lilex': ['Lilex-Regular.ttf', 'OFL.txt'],
}.items():
    for name in files:
        target = DEST / 'assets/fonts' / family / name
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(SOURCE / 'assets/fonts' / family / name, target)

# A deterministic TTC containing the two unchanged OFL test fonts. SFNT table
# offsets in a collection are relative to the collection, not the face start.
font_paths = [DEST / 'assets/fonts/ibm-plex-sans/IBMPlexSans-Regular.ttf',
              DEST / 'assets/fonts/lilex/Lilex-Regular.ttf']
collection = bytearray(b'ttcf' + struct.pack('>II', 0x10000, 2) + bytes(8))
for index, path in enumerate(font_paths):
    collection.extend(bytes((-len(collection)) % 4))
    offset = len(collection)
    struct.pack_into('>I', collection, 12 + index * 4, offset)
    font = bytearray(path.read_bytes())
    table_count = struct.unpack_from('>H', font, 4)[0]
    for table in range(table_count):
        entry = 12 + table * 16 + 8
        table_offset = struct.unpack_from('>I', font, entry)[0]
        struct.pack_into('>I', font, entry, table_offset + offset)
    collection.extend(font)
(DEST / 'assets/fonts/evaluation.ttc').write_bytes(collection)
fixture_source = RESEARCH / 'maintenance-fixtures'
if fixture_source.exists():
    shutil.copytree(fixture_source, DEST / 'maintenance-fixtures', dirs_exist_ok=True)
    shape = (fixture_source / 'shape.svg').read_bytes()
    (DEST / 'maintenance-fixtures/shape.svgz').write_bytes(gzip.compress(shape, mtime=0))

patches = []


def patch_file(label, old, new, target):
    write(target, new)
    patches.extend(difflib.unified_diff(old.splitlines(True), new.splitlines(True),
                                      fromfile=f'a/{label}', tofile=f'b/{label}'))


# All inherited dependencies remain at their original versions/features except
# the three explicitly listed text-stack dependencies. Git paths retain the tag.
inheritance = '\n[workspace]\n\n[workspace.package]\n'
inheritance += '\n'.join(f'{name} = {value(item)}' for name, item in workspace['package'].items())
inheritance += '\n\n[workspace.dependencies]\n'
inheritance += '\n'.join(f'{json.dumps(name)} = {value(item)}' for name, item in dependencies.items())
for category, rules in workspace['lints'].items():
    inheritance += f'\n\n[workspace.lints.{category}]\n'
    inheritance += '\n'.join(f'{json.dumps(name)} = {value(item)}' for name, item in rules.items())
inheritance += '\n\n[patch.crates-io]\npulp = { path = "../pulp" }\nmetal = { path = "../metal" }\n'
inheritance += 'cosmic-text = { path = "../cosmic-text" }\n'
inheritance += f'\n[patch."{URL}"]\ngpui = {{ path = "." }}\n'
patch_file('gpui/Cargo.toml', original_gpui,
           original_gpui.replace('ttf-parser = "0.25"', 'skrifa.workspace = true') + inheritance,
           DEST / 'vendor/gpui/Cargo.toml')

svg = read(SOURCE / 'crates/gpui/src/svg_renderer.rs')
patched_svg = svg.replace('use resvg::tiny_skia::Pixmap;',
                          'use resvg::tiny_skia::Pixmap;\nuse skrifa::MetadataProvider;')
patched_svg = patched_svg.replace('ttf_parser::Face::parse(font_data, face_index)',
                                  'skrifa::FontRef::from_index(font_data, face_index)')
patched_svg = patched_svg.replace('face.glyph_index(ch)', 'face.charmap().map(ch)')
assert patched_svg != svg and 'ttf_parser' not in patched_svg
test_source = RESEARCH / 'maintenance-svg-tests.rs'
if test_source.exists():
    patched_svg += '\n' + read(test_source)
patch_file('gpui/src/svg_renderer.rs', svg, patched_svg, DEST / 'vendor/gpui/src/svg_renderer.rs')

# Rust 1.99's lint now recognizes this pre-existing literal. The one-shot
# bit proof records both f32 representations as 0x3fcf1bbd; behavior is unchanged.
geometry = read(SOURCE / 'crates/gpui/src/geometry.rs')
assert geometry.count('relative(1.618_034)') == 1
patch_file('gpui/src/geometry.rs', geometry,
           geometry.replace('relative(1.618_034)', 'relative(std::f32::consts::GOLDEN_RATIO)'),
           DEST / 'vendor/gpui/src/geometry.rs')

for package, version in [('pulp', '0.22.3'), ('metal', '0.33.0')]:
    source = REGISTRY / f'{package}-{version}'
    target = DEST / 'vendor' / package
    shutil.copytree(source, target, dirs_exist_ok=True)
    original = read(source / 'Cargo.toml')
    replacement = original.replace('[dependencies.paste]\nversion = "1"',
                                   '[dependencies.paste]\nversion = "=0.2.3"\npackage = "pastey"')
    assert replacement != original
    replacement += '\n[workspace]\n'
    patch_file(f'{package}/Cargo.toml', original, replacement, target / 'Cargo.toml')
    write(target / 'FERROSTAGE-MAINTENANCE.txt',
          f'Isolated evaluation based on {package} {version}. Original notices retained.\n'
          'The only source-package change is a dependency on published pastey 0.2.3\n'
          'under the existing paste alias; no upstream Rust source is renamed or edited.\n')

cosmic_source = REGISTRY / 'cosmic-text-0.19.0'
cosmic_target = DEST / 'vendor/cosmic-text'
shutil.copytree(cosmic_source, cosmic_target, dirs_exist_ok=True)
cosmic_original = read(cosmic_source / 'Cargo.toml')
cosmic_patched = cosmic_original.replace('[dependencies.fontdb]\nversion = "0.23"',
                                        '[dependencies.fontdb]\nversion = "=0.24.0"')
assert cosmic_patched != cosmic_original
cosmic_patched += '\n[workspace]\n'
patch_file('cosmic-text/Cargo.toml', cosmic_original, cosmic_patched, cosmic_target / 'Cargo.toml')
write(cosmic_target / 'FERROSTAGE-MAINTENANCE.txt',
      'Isolated MIT OR Apache-2.0 cosmic-text0.19.0 dependency-only evaluation.\n'
      'fontdb0.24 replaces0.23; original Rust source and feature set are unchanged.\n'
      'fontdb0.24 itself maintains an inlined ttf-parser metadata subset; see report.\n')
shutil.copytree(REGISTRY / 'fontdb-0.24.0', DEST / 'validation/fontdb', dirs_exist_ok=True)
write(DEST / 'validation/fontdb/Cargo.toml', read(REGISTRY / 'fontdb-0.24.0/Cargo.toml') + '\n[workspace]\n')

image_dependency = copy.deepcopy(workspace['dependencies']['image'])
image_dependency['version'] = '=0.25.10'
manifest = f'''[package]
name = "ferrostage-gpui-maintenance-candidate"
version = "0.0.0"
edition = "2024"
publish = false
license = "MIT OR Apache-2.0"

[workspace]
exclude = ["vendor/gpui", "vendor/pulp", "vendor/metal", "vendor/cosmic-text", "validation/fontdb"]

[dependencies]
gpui = {{ path = "vendor/gpui", default-features = false }}
gpui_platform = {{ git = "{URL}", tag = "{TAG}", default-features = false }}

[dev-dependencies]
image = {value(image_dependency)}
exr = "=1.74.2"
paste = {{ package = "pastey", version = "=0.2.3" }}
cosmic-text = "=0.19.0"

[patch."{URL}"]
gpui = {{ path = "vendor/gpui" }}

[patch.crates-io]
pulp = {{ path = "vendor/pulp" }}
metal = {{ path = "vendor/metal" }}
cosmic-text = {{ path = "vendor/cosmic-text" }}
'''
write(DEST / 'Cargo.toml', manifest)
write(DEST / 'src/main.rs', '#![forbid(unsafe_code)]\n\nfn main() {\n    let _application = gpui_platform::application();\n}\n')
# Directly activate the WGPU consumer on the Windows host for an API check.
# It is a separate test workspace, not a feature or graph change to the candidate.
write(DEST / 'validation/wgpu-consumer/Cargo.toml', f'''[package]
name = "ferrostage-gpui-wgpu-api-check"
version = "0.0.0"
edition = "2024"
publish = false
license = "MIT OR Apache-2.0"

[workspace]

[dependencies]
gpui_wgpu = {{ git = "{URL}", tag = "{TAG}" }}

[patch."{URL}"]
gpui = {{ path = "../../vendor/gpui" }}

[patch.crates-io]
pulp = {{ path = "../../vendor/pulp" }}
metal = {{ path = "../../vendor/metal" }}
cosmic-text = {{ path = "../../vendor/cosmic-text" }}
''')
write(DEST / 'validation/wgpu-consumer/src/main.rs',
      '#![forbid(unsafe_code)]\n\nfn main() {}\n')
write(DEST / 'vendor/gpui/FERROSTAGE-MAINTENANCE.txt',
      f'Isolated evaluation of Apache-2.0 GPUI from Zed {TAG}, {REV}.\n'
      'Original notices retained. SVG text dependencies and glyph lookup use the maintained\n'
      'resvg/usvg0.48.1 and skrifa0.44 stack. A bit-identical GOLDEN_RATIO constant\n'
      'replaces the original literal for Rust1.99 lint compatibility.\n'
      'Display/input/platform source is unchanged.\n')
for file in RESEARCH.glob('maintenance-regression-*.rs'):
    write(DEST / 'tests' / file.name.removeprefix('maintenance-regression-'), read(file))
if RETAINED.exists():
    for lock in RETAINED.rglob('Cargo.lock'):
        target = DEST / lock.relative_to(RETAINED)
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(lock, target)
write(RESEARCH / 'maintenance-source.patch', ''.join(patches))
write(RESEARCH / 'maintenance-preparation.json', json.dumps({
    'tag': TAG, 'revision': REV, 'source': str(SOURCE), 'candidate': str(DEST),
    'path_dependencies': path_dependencies, 'inherited_dependencies': sorted(needed),
    'changes': ['resvg/usvg0.48.1 + explicit svgz', 'skrifa0.44 glyph lookup', 'pulp/metal real pastey0.2.3 dependency', 'cosmic-text0.19 fontdb0.24 dependency', 'bit-identical GOLDEN_RATIO for Rust1.99 lint'],
    'gpui_original_manifest_sha256': hashlib.sha256((SOURCE / 'crates/gpui/Cargo.toml').read_bytes()).hexdigest(),
}, indent=2) + '\n')
print(json.dumps({'prepared': str(DEST), 'inherited_dependencies': len(needed), 'copied_gpui_license': 'Apache-2.0'}))
