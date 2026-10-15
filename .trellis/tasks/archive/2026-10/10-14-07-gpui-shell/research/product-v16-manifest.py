"""Verify V16 handoff and seal its new evidence without rewriting prior receipts."""
from datetime import datetime, timezone
from pathlib import Path
import hashlib
import json
import subprocess

root = Path.cwd()
research = root / '.trellis/tasks/10-14-07-gpui-shell/research'
sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
freeze_path = research / 'product-native-freeze-v16.json'
freeze = json.loads(freeze_path.read_text())
assert all(sha(root / name) == digest for name, digest in freeze['source_files'].items())
for key in ['binary', 'pdfium', 'fixture', 'cargo_lock']:
    assert sha(Path(freeze[key]['path'])) == freeze[key]['sha256'], key
for name, item in freeze['accesskit_harness_files'].items():
    assert sha(root / name) == item['sha256'], name
vendor = root / 'vendor/accesskit_windows'
harness = research / 'product-v16-accesskit-harness'
for path in (vendor / 'src').rglob('*'):
    if path.is_file() and path.name != 'lib.rs':
        assert sha(path) == sha(harness / path.relative_to(vendor)), str(path)
assert (harness / 'src/lib.rs').read_text() == (vendor / 'src/lib.rs').read_text().replace('mod tests;', 'mod v16_regression_tests;')
assert sha(harness / 'src/v16_regression_tests.rs') == sha(research / 'product-v16-a11y-tests.rs')
inputs = {}
for name, item in freeze['config_fixtures'].items():
    path = Path(item['path'])
    assert sha(path) == item['sha256']
    inputs[name] = {'original_path': str(path), 'sha256': sha(path), 'utf8_text': path.read_bytes().decode('utf-8')}

def save(path, value):
    with path.open('x', encoding='utf-8') as stream:
        json.dump(value, stream, indent=2, ensure_ascii=False)
        stream.write('\n')

save(research / 'product-v16-config-inputs.json', inputs)
diff = subprocess.run(['git', 'diff', '--check'], capture_output=True, text=True)
assert diff.returncode == 0, diff.stdout + diff.stderr
save(research / 'product-v16-readback.json', {
    'captured_utc': datetime.now(timezone.utc).isoformat(),
    'source_verified': len(freeze['source_files']), 'binary_pdfium_fixture_lock_verified': True,
    'harness_product_source_equivalence': 'All production source byte-identical except lib.rs test-only module redirection; narrow harness source matches frozen test file.',
    'config_inputs_verified': len(inputs), 'diff_check': {'exit_code': diff.returncode, 'stdout': diff.stdout, 'stderr': diff.stderr},
})
paths = set(research.glob('product-v16-*')) | set(harness.rglob('*')) | {freeze_path}
records = {path.relative_to(root).as_posix(): {'bytes': path.stat().st_size, 'sha256': sha(path)} for path in sorted(paths) if path.is_file()}
target = research / 'product-v16-evidence-manifest.json'
save(target, {'captured_utc': datetime.now(timezone.utc).isoformat(),
              'scope': 'Authorized V16 sixth-package AccessKit adoption and automated checks. Root-only hidden HWND compiled, product native NOT_RUN at handoff.',
              'freeze_sha256': sha(freeze_path), 'source_snapshot_sha256': freeze['source_snapshot_sha256'], 'files': records})
with target.with_suffix('.sha256').open('x', encoding='utf-8') as stream:
    stream.write(sha(target) + '  ' + target.name + '\n')
assert all(sha(root / name) == record['sha256'] for name, record in records.items())
print(json.dumps({'records': len(records), 'manifest_sha256': sha(target), 'source_verified': len(freeze['source_files']), 'binary_verified': True}, indent=2))
