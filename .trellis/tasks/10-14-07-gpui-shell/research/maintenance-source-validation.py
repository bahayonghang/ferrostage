"""Read-only verification of published inputs; retain exact license texts."""
import hashlib
import json
from pathlib import Path
import shutil
import tarfile
import tomllib

ROOT = Path.cwd()
RESEARCH = ROOT / '.trellis/tasks/10-14-07-gpui-shell/research'
CANDIDATE = ROOT / 'cache/gpui-maintenance-candidate'
REGISTRY = Path('C:/Users/lyh/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f')
ARCHIVES = Path('C:/Users/lyh/.cargo/registry/cache/index.crates.io-1949cf8c6b5b557f')
ZED = Path('C:/Users/lyh/.cargo/git/checkouts/zed-a70e2ad075855582/96a1b1b')
NOTICES = RESEARCH / 'maintenance-retained/notices'


def digest(data):
    return hashlib.sha256(data).hexdigest()


def hashes(directory):
    return {path.relative_to(directory).as_posix(): digest(path.read_bytes())
            for path in sorted(directory.rglob('*')) if path.is_file()}


def retain(path, relative):
    target = NOTICES / relative
    target.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(path, target)
    return {'source': str(path), 'retained': target.relative_to(ROOT).as_posix(),
            'sha256': digest(path.read_bytes())}


checksum_sources = {}
for lock in [ROOT / 'cache/gpui-route-spike/Cargo.lock', CANDIDATE / 'Cargo.lock']:
    for package in tomllib.loads(lock.read_text(encoding='utf-8'))['package']:
        if 'checksum' in package:
            checksum_sources[(package['name'], package['version'])] = (
                package['checksum'], lock.relative_to(ROOT).as_posix())

packages = [('pulp', '0.22.3'), ('metal', '0.33.0'), ('cosmic-text', '0.19.0'),
            ('fontdb', '0.24.0'), ('resvg', '0.48.1'), ('usvg', '0.48.1'),
            ('skrifa', '0.44.0'), ('harfrust', '0.12.0'), ('pastey', '0.2.3'),
            ('arrayref', '0.3.9'), ('libbz2-rs-sys', '0.2.5'), ('cbindgen', '0.28.0')]
records = []
notices = []
for name, version in packages:
    label = f'{name}-{version}'
    source = REGISTRY / label
    archive = ARCHIVES / f'{label}.crate'
    expected, lock = checksum_sources[(name, version)]
    actual = digest(archive.read_bytes())
    assert actual == expected, (label, actual, expected)
    file_hashes = {}
    differences = []
    with tarfile.open(archive, 'r:gz') as contents:
        for member in contents.getmembers():
            if not member.isfile():
                continue
            relative = Path(member.name).relative_to(label)
            entry_hash = digest(contents.extractfile(member).read())
            file_hashes[relative.as_posix()] = entry_hash
            cached = source / relative
            if not cached.is_file() or digest(cached.read_bytes()) != entry_hash:
                differences.append(relative.as_posix())
    assert not differences, (label, differences)
    package = tomllib.loads((source / 'Cargo.toml').read_text(encoding='utf-8'))['package']
    vcs_path = source / '.cargo_vcs_info.json'
    vcs = json.loads(vcs_path.read_text(encoding='utf-8')) if vcs_path.exists() else None
    records.append({'package': name, 'version': version, 'license': package['license'],
                    'repository': package.get('repository'), 'vcs': vcs,
                    'archive': str(archive), 'archive_sha256': actual,
                    'checksum_lock': lock, 'source': str(source),
                    'verified_files': len(file_hashes), 'mismatches': differences,
                    'archive_file_hashes': file_hashes})
    for path in source.iterdir():
        if path.is_file() and any(word in path.name.upper() for word in ('LICENSE', 'COPYING', 'NOTICE')):
            notices.append(retain(path, f'{label}/{path.name}'))

copy_checks = []
for name, source, target, expected_changes in [
    ('gpui', ZED / 'crates/gpui', CANDIDATE / 'vendor/gpui',
     {'Cargo.toml', 'src/svg_renderer.rs', 'src/geometry.rs'}),
    ('pulp', REGISTRY / 'pulp-0.22.3', CANDIDATE / 'vendor/pulp', {'Cargo.toml', 'Cargo.lock'}),
    ('metal', REGISTRY / 'metal-0.33.0', CANDIDATE / 'vendor/metal', {'Cargo.toml'}),
    ('cosmic-text', REGISTRY / 'cosmic-text-0.19.0', CANDIDATE / 'vendor/cosmic-text', {'Cargo.toml', 'Cargo.lock'}),
    ('fontdb-validation', REGISTRY / 'fontdb-0.24.0', CANDIDATE / 'validation/fontdb', {'Cargo.toml', 'Cargo.lock'}),
]:
    source_hashes = hashes(source)
    target_hashes = hashes(target)
    changed = sorted(path for path, value in source_hashes.items() if target_hashes.get(path) != value)
    extra = sorted(set(target_hashes) - set(source_hashes))
    assert set(changed) <= expected_changes, (name, changed)
    assert set(extra) <= {'Cargo.lock', 'FERROSTAGE-MAINTENANCE.txt'}, (name, extra)
    copy_checks.append({'package': name, 'source': str(source), 'copy': str(target),
                        'source_files': source_hashes, 'changed': changed, 'extra': extra})

notices.append(retain(ZED / 'crates/gpui/LICENSE-APACHE', 'gpui/LICENSE-APACHE'))
for family, license_name in [('ibm-plex-sans', 'license.txt'), ('lilex', 'OFL.txt')]:
    notices.append(retain(ZED / 'assets/fonts' / family / license_name,
                          f'fonts/{family}/{license_name}'))
notices.append(retain(REGISTRY / 'fontdb-0.24.0/tests/fonts/LICENSE.txt', 'fonts/tuffy/LICENSE.txt'))
fonts = []
for relative in ['assets/fonts/ibm-plex-sans/IBMPlexSans-Regular.ttf', 'assets/fonts/lilex/Lilex-Regular.ttf']:
    original = ZED / relative
    copied = CANDIDATE / relative
    assert original.read_bytes() == copied.read_bytes()
    fonts.append({'source': str(original), 'copy': str(copied), 'sha256': digest(original.read_bytes())})
fonts.append({'source': str(REGISTRY / 'fontdb-0.24.0/tests/fonts/Tuffy.ttf'),
              'sha256': digest((REGISTRY / 'fontdb-0.24.0/tests/fonts/Tuffy.ttf').read_bytes())})
fonts.append({'generated_ignored_cache_only': str(CANDIDATE / 'assets/fonts/evaluation.ttc'),
              'sha256': digest((CANDIDATE / 'assets/fonts/evaluation.ttc').read_bytes()),
              'reproduction': 'maintenance-prepare.py deterministic TTC assembly; not distributed'})
result = {'method': 'Lock checksum -> original .crate SHA256 -> every archive member matches registry source; exact-tag diff/status separately; copied package differences constrained to the patch/locks/notices.',
          'registry_inputs': records, 'copy_checks': copy_checks, 'notices': notices, 'fonts': fonts}
(RESEARCH / 'maintenance-source-validation.json').write_text(json.dumps(result, indent=2) + '\n', encoding='utf-8')
print(json.dumps({'verified_archives': len(records), 'verified_archive_files': sum(item['verified_files'] for item in records),
                  'copied_packages': len(copy_checks), 'retained_notices': len(notices), 'mismatches': 0}))
