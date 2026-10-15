"""Read-back of immutable implementation evidence and independent review inputs."""
from datetime import datetime, timezone
from pathlib import Path
import hashlib
import json
import subprocess
import tomllib

root = Path.cwd()
research = root / '.trellis/tasks/10-14-07-gpui-shell/research'
sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
freeze_path = research / 'product-native-freeze-v16.json'
freeze = json.loads(freeze_path.read_text())
assert sha(freeze_path) == '0e2b1ed8ef9299676ba249e7600bd8ab6734f00c6ae17f677467405ad60402d3'
assert len(freeze['source_files']) == 648
for name, digest in freeze['source_files'].items():
    assert sha(root / name) == digest, name
aggregate = hashlib.sha256(json.dumps(freeze['source_files'], sort_keys=True, separators=(',', ':')).encode()).hexdigest()
assert aggregate == freeze['source_snapshot_sha256'] == '3e208cf586dcf37ab0420228e8eb3fba8ee07c0b73a8597c318a7f6a219c35b8'
for key in ['binary', 'fixture', 'pdfium', 'cargo_lock']:
    assert sha(Path(freeze[key]['path'])) == freeze[key]['sha256'], key
assert freeze['binary']['sha256'] == 'cf228c03655ddd45d25520bd88dab025c219af9bec9195314ce541dbb1cd89d1'
manifest_path = research / 'product-v16-evidence-manifest.json'
assert sha(manifest_path) == '1e004a195f04725d2fe511036c1013d6be4bd5346ffc01e93dae76b4bac71f63'
manifest = json.loads(manifest_path.read_text())
assert len(manifest['files']) == 62
for name, record in manifest['files'].items():
    assert sha(root / name) == record['sha256'], name
    assert (root / name).stat().st_size == record['bytes'], name
for name, record in freeze['accesskit_harness_files'].items():
    assert sha(root / name) == record['sha256'], name
vendor = root / 'vendor/accesskit_windows'
harness = research / 'product-v16-accesskit-harness'
for path in vendor.rglob('*'):
    if path.is_file() and path.relative_to(vendor).as_posix() not in ['Cargo.toml', 'Cargo.lock', 'src/lib.rs']:
        assert sha(path) == sha(harness / path.relative_to(vendor)), str(path)
assert (harness / 'src/lib.rs').read_text() == (vendor / 'src/lib.rs').read_text().replace('mod tests;', 'mod v16_regression_tests;')
assert sha(harness / 'src/v16_regression_tests.rs') == sha(research / 'product-v16-a11y-tests.rs')
upstream_manifest = (vendor / 'Cargo.toml').read_text()
expected_manifest = upstream_manifest[:upstream_manifest.index('[dev-dependencies.once_cell]')]
expected_manifest = expected_manifest.replace('[[example]]\nname = "hello_world"\npath = "examples/hello_world.rs"\n\n', '')
expected_manifest += '\n[workspace]\n\n[features]\nroot-native-window = []\n'
assert (harness / 'Cargo.toml').read_text() == expected_manifest
old_lock = tomllib.loads(subprocess.check_output(['git', 'show', 'HEAD:Cargo.lock'], text=True))
new_lock = tomllib.loads((root / 'Cargo.lock').read_text())
accesskit = next(p for p in old_lock['package'] if p['name'] == 'accesskit_windows')
assert accesskit['version'] == '0.34.0'
accesskit.pop('source')
assert accesskit.pop('checksum') == '106c2b961215864d1c2e703ee63269c25c4e80a577ffb2c1017b9c17dcdf83a1'
assert old_lock == new_lock
previous = json.loads((research / 'product-native-freeze-v15.json').read_text())
unchanged_first_party = [name for name in previous['source_files'] if name.startswith('crates/')]
assert all(previous['source_files'][name] == freeze['source_files'][name] for name in unchanged_first_party)
reused = {}
for label in ['a11y-cache-access', 'hidden-compile', 'just-ci', 'headless-final-lock', 'typecheck', 'clippy', 'deny', 'build']:
    receipt_path = research / f'product-v16-{label}.json'
    receipt = json.loads(receipt_path.read_text())
    assert receipt['exit_code'] == 0 and receipt['status'] == 'RETURNED'
    reused[label] = {'receipt_sha256': sha(receipt_path), 'log_sha256': sha(research / receipt['log']), 'status': 'VERIFIED_REUSE_NOT_RERUN'}
diff = subprocess.run(['git', 'diff', '--check'], capture_output=True, text=True)
assert diff.returncode == 0
baseline = json.loads((research / 'product-v16-baseline-and-lock.json').read_text())
result = {
    'captured_utc': datetime.now(timezone.utc).isoformat(), 'freeze_sha256': sha(freeze_path),
    'source_count': len(freeze['source_files']), 'aggregate': aggregate,
    'binary': freeze['binary'], 'pdfium': freeze['pdfium'], 'fixture': freeze['fixture'],
    'implementation_manifest_count': len(manifest['files']), 'implementation_manifest_sha256': sha(manifest_path),
    'all_harness_source_and_manifest_adaptations_verified': True,
    'lock_only_same_version_accesskit_source_removal': True,
    'unchanged_first_party_files_from_v15': len(unchanged_first_party),
    'reused_checks': reused, 'diff_check': {'exit_code': diff.returncode, 'stdout': diff.stdout, 'stderr': diff.stderr},
    'baseline_receipt_clarification': {
        'sealed_receipt_observed_binary': baseline['unchanged_v15_artifacts']['binary'],
        'authoritative_v15_frozen_binary': previous['binary']['sha256'],
        'status': 'MISMATCH: do not use the baseline receipt binary field as a V15 frozen identity or acceptance receipt. Original preserved; current V16 identity independently verified.'},
    'native_executed_by_reviewer': False,
}
target = research / 'product-independent-v16-readback.json'
with target.open('x', encoding='utf-8') as stream:
    json.dump(result, stream, indent=2)
    stream.write('\n')
print(json.dumps({'sources': len(freeze['source_files']), 'manifest_records': len(manifest['files']), 'first_party_unchanged': len(unchanged_first_party), 'binary': freeze['binary']['sha256'], 'status': 'PASS_WITH_BASELINE_RECEIPT_CLARIFICATION'}, indent=2))
