"""Freeze authorized F13 product fix and actual notes fixture, excluding F14 proposal."""
from datetime import datetime, timezone
from pathlib import Path
import hashlib
import json
import subprocess

root = Path.cwd()
research = root / '.trellis/tasks/10-14-07-gpui-shell/research'
sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()

def record(path):
    return {'path': str(path), 'resolved_path': str(path.resolve()), 'bytes': path.stat().st_size, 'sha256': sha(path)}

def save(path, value):
    with path.open('x', encoding='utf-8') as stream:
        json.dump(value, stream, indent=2, ensure_ascii=False)
        stream.write('\n')

previous = research / 'product-native-freeze-v14.json'
freeze = json.loads(previous.read_text(encoding='utf-8'))
sources = {path: sha(root / path) for path in freeze['source_files']}
changes = {path: {'v14': digest, 'v15': sources[path]} for path, digest in freeze['source_files'].items() if sources[path] != digest}
assert set(changes) == {
    'crates/ferrostage-platform/src/windows.rs',
    'crates/ferrostage-platform/src/windows/tests.rs',
    '.trellis/spec/frontend/maintained-dependencies.md',
}, changes
fixture_dir = root / 'fixtures/notes-functional'
fixture_inputs = [path for path in fixture_dir.rglob('*') if path.is_file() and 'build' not in path.relative_to(fixture_dir).parts]
additional = [root / '.trellis/spec/backend/quality-guidelines.md', root / 'crates/ferrostage-document/tests/notes_native.rs'] + fixture_inputs
added = {path.relative_to(root).as_posix(): sha(path) for path in additional if path.relative_to(root).as_posix() not in sources}
sources.update(added)
sources = dict(sorted(sources.items()))
delta = research / 'product-v15-source-delta.json'
save(delta, {'changes': changes, 'added': added, 'source_count': len(sources), 'unchanged_count': len(freeze['source_files']) - len(changes),
             'excluded': 'research-only AccessKit candidate/proposal are not product sources or dependencies'})
checks = {}
for name in ['getdc-green', 'just-ci-final', 'headless', 'document-isolated', 'typecheck', 'clippy', 'deny', 'build']:
    path = research / f'product-v15-{name}.json'
    assert json.loads(path.read_text(encoding='utf-8'))['exit_code'] == 0, name
    checks[name] = record(path)
for key in ['binary', 'pdfium', 'cargo_lock']:
    current = record(Path(freeze[key]['path']))
    if key != 'binary':
        assert current['sha256'] == freeze[key]['sha256'], key
    freeze[key] = current
freeze['previous_fixture'] = freeze['fixture']
freeze['fixture'] = record(fixture_dir / 'pympress-notes.pdf')
assert freeze['fixture']['sha256'] == 'e939e35f931c184fd93b2e9b95e560db73ba6a6f6dfccbb0963262f0b359dfeb'
freeze.update(
    captured_utc=datetime.now(timezone.utc).isoformat(),
    head=subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip(),
    status='FROZEN for serial independent check then Root native acceptance; F14 remains unresolved, candidate is research-only',
    source_files=sources,
    source_snapshot_sha256=hashlib.sha256(json.dumps(sources, sort_keys=True, separators=(',', ':')).encode()).hexdigest(),
    final_build_log=record(research / 'product-v15-build.log'),
    final_native_clippy_log=record(research / 'product-v15-clippy.log'),
    checks=checks, previous_freeze=record(previous), implementation_delta=record(delta),
    default_test_count=151, ui_feature_test_count=82, ui_feature_breakdown={'pure': 23, 'actual_gpui': 59},
    document_native_test_count=2,
    fixture_inputs={path.relative_to(fixture_dir).as_posix(): record(path) for path in sorted(fixture_inputs)},
    native_runtime_status='V15 NOT_RUN; F13 real API regression PASS; F14 native close panic unresolved in this product build',
    config_fixtures={path.name: record(path) for path in sorted((research / 'native').glob('v8-*.toml'))},
    limits=['Only authorized product runtime delta is the safe non-NULL desktop DC owner; no AccessKit or gpui_windows product patch',
            'V15 resource test is real three-monitor enumeration and GDI counts; long-running GPUI/DPI native acceptance remains Root-owned',
            'New compiled notes PDF proves document/PDFium crop/overlay contracts in tests;07 real notes UI is NOT_IMPLEMENTED/NOT_RUN until08',
            'F14 research-only candidate6 tests and compile-only hidden-window harness are not linked and do not establish F14 product PASS',
            'User deferred further native width/size testing; physical unplug/reconnect/ports and real single/zero NOT_RUN; GateB incomplete; no08-11'],
)
freeze['observations'].append('V15 replaces NULL DC owner with real desktop HWND; new notes fixture replaces raster.pdf for future native runs; all prior frozen evidence retained')
path = research / 'product-native-freeze-v15.json'
save(path, freeze)
assert all(sha(root / path) == digest for path, digest in sources.items())
print(json.dumps({'source_count': len(sources), 'source_sha256': freeze['source_snapshot_sha256'], 'binary': freeze['binary'], 'fixture': freeze['fixture'], 'freeze_sha256': sha(path)}, indent=2))
