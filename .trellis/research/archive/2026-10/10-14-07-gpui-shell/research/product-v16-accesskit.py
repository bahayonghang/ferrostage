"""Reconstruct the approved AccessKit package and prepare a non-GUI regression copy."""
from pathlib import Path
import hashlib
import json
import os
import shutil
import subprocess
import sys
import tarfile
import tempfile

root = Path(__file__).resolve().parents[4]
research = Path(__file__).resolve().parent
vendor = root / 'vendor/accesskit_windows'
archive = Path.home() / '.cargo/registry/cache/index.crates.io-1949cf8c6b5b557f/accesskit_windows-0.34.0.crate'
patch = research / 'product-v15-accesskit-proposal.patch'
checksum = '106c2b961215864d1c2e703ee63269c25c4e80a577ffb2c1017b9c17dcdf83a1'
patch_sha = '93563d304909e2bca657dc0136382ba9179e638b71799d9c6823918ad2a9f2f0'
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
assert sha(archive) == checksum
assert sha(patch) == patch_sha

def files(directory):
    return {p.relative_to(directory).as_posix(): sha(p) for p in sorted(directory.rglob('*')) if p.is_file()}

def reconstruct(destination):
    with tarfile.open(archive) as source:
        source.extractall(destination, filter='data')
    package = destination / 'accesskit_windows-0.34.0'
    original = files(package)
    environment = dict(os.environ, GIT_CEILING_DIRECTORIES=str(destination))
    patch_bytes = patch.read_bytes().replace(b'\r\n', b'\n')
    subprocess.run(['git', 'apply', '--check', '-'], input=patch_bytes, cwd=package, env=environment, check=True)
    subprocess.run(['git', 'apply', '-'], input=patch_bytes, cwd=package, env=environment, check=True)
    patched = files(package)
    changed = [name for name in original if original[name] != patched[name]]
    assert set(changed) == {'src/util.rs', 'src/context.rs', 'src/node.rs', 'src/text.rs'}
    for name in ['LICENSE-MIT', 'LICENSE-APACHE']:
        shutil.copy2(research / ('product-v16-' + name), package / name)
    return package, original, changed

mode = sys.argv[1]
if mode in ('adopt', 'verify'):
    with tempfile.TemporaryDirectory(prefix='product-v16-reconstruct-', dir=research) as temporary:
        assert Path(temporary).resolve().is_relative_to(research.resolve())
        package, original, changed = reconstruct(Path(temporary))
        expected = files(package)
        if mode == 'adopt':
            assert not vendor.exists()
            shutil.copytree(package, vendor)
            record = {
                'package': 'accesskit_windows 0.34.0', 'license': 'MIT OR Apache-2.0',
                'upstream_vcs': json.loads((package / '.cargo_vcs_info.json').read_text()),
                'archive_url': 'https://static.crates.io/crates/accesskit_windows/accesskit_windows-0.34.0.crate',
                'archive_sha256': checksum, 'patch': patch.name, 'patch_sha256': patch_sha,
                'original_files': original, 'product_files': expected, 'runtime_changes': changed,
                'manifest_adaptation': 'None; normalized/original manifest, tests, examples and package lock preserved byte-for-byte.',
                'patch_line_endings': 'Verify original approved patch SHA first, then normalize its CRLF to LF for git apply against LF registry archive. No textual hunk changes.',
                'additional_notices': {name: {'url': 'https://raw.githubusercontent.com/AccessKit/accesskit/c88605b96d04431f9c3c792464a0f2f253480e94/' + name, 'sha256': expected[name]} for name in ['LICENSE-MIT', 'LICENSE-APACHE']},
                'tracking': 'Local F14 tracking: ADR-008 and task07 product-v16-implementation.md. No upstream issue/PR submitted.',
                'removal_condition': 'Remove local patch only after a separately approved upstream version handles the actual ClientToScreen failure for all four callers and passes retained-context positive/negative/empty controls plus product native close checks. Rebase must compare all four callers and original manifest; no version update authorized here.',
                'limits': 'Invalid HWND error conversion only; not HWND reuse, simultaneous race elimination, full provider retirement/drain or native product acceptance.',
            }
            with (research / 'product-v16-accesskit-provenance.json').open('x') as output:
                json.dump(record, output, indent=2)
                output.write('\n')
        actual = files(vendor)
        assert actual == expected, {'changed': [n for n in expected if actual.get(n) != expected[n]], 'extra': sorted(actual.keys() - expected.keys())}
        print(json.dumps({'status': 'PASS', 'files': len(expected), 'runtime_changes': changed, 'manifest_sha256': expected['Cargo.toml'], 'patch_sha256': patch_sha}, indent=2))
elif mode == 'prepare-tests':
    harness = research / 'product-v16-accesskit-harness'
    assert not harness.exists()
    shutil.copytree(vendor, harness)
    # Test-only adaptation, never applied to the production package manifest.
    manifest = (harness / 'Cargo.toml').read_text()
    manifest = manifest[:manifest.index('[dev-dependencies.once_cell]')]
    manifest = manifest.replace('[[example]]\nname = "hello_world"\npath = "examples/hello_world.rs"\n\n', '')
    (harness / 'Cargo.toml').write_text(manifest + '\n[workspace]\n\n[features]\nroot-native-window = []\n')
    shutil.copy2(root / 'Cargo.lock', harness / 'Cargo.lock')
    lib = harness / 'src/lib.rs'
    lib.write_text(lib.read_text().replace('mod tests;', 'mod v16_regression_tests;'))
    shutil.copy2(research / 'product-v16-a11y-tests.rs', harness / 'src/v16_regression_tests.rs')
    print('Prepared separate test harness from exact product vendor source; no GUI test executed.')
else:
    raise ValueError(mode)
