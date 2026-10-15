"""Freeze V10 without rewriting V6-V9 inputs or native receipts."""
from datetime import datetime, timezone
from pathlib import Path
import hashlib
import json
import subprocess

root = Path.cwd()
research = root / '.trellis/tasks/10-14-07-gpui-shell/research'
sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()

def record(path):
    return {'path': str(path), 'resolved_path': str(path.resolve()),
            'bytes': path.stat().st_size, 'sha256': sha(path)}

def save(path, data):
    with path.open('x', encoding='utf-8') as stream:
        json.dump(data, stream, indent=2, ensure_ascii=False)
        stream.write('\n')

previous = research / 'product-native-freeze-v9.json'
freeze = json.loads(previous.read_text(encoding='utf-8'))
sources = {path: sha(root / path) for path in freeze['source_files']}
changes = {path: {'v9': digest, 'v10': sources[path]}
           for path, digest in freeze['source_files'].items() if sources[path] != digest}
assert set(changes) == {'crates/ferrostage-ui/src/shell.rs', 'crates/ferrostage-ui/src/policy.rs',
                        '.trellis/spec/frontend/state-management.md'}, changes
delta_path = research / 'product-v10-source-delta.json'
save(delta_path, {'changes': changes, 'added': {}, 'source_count': len(sources),
                 'unchanged_count': len(sources) - len(changes)})
gates = {}
for name in ['just-ci', 'headless', 'clippy', 'deny', 'build']:
    path = research / f'product-v10-{name}.json'
    assert json.loads(path.read_text(encoding='utf-8'))['exit_code'] == 0, name
    gates[name] = record(path)
for key in ['binary', 'pdfium', 'fixture', 'cargo_lock']:
    old = freeze[key]
    current = record(Path(old['path']))
    if key != 'binary':
        assert old['sha256'] == current['sha256'], key
    freeze[key] = current
freeze.update(
    captured_utc=datetime.now(timezone.utc).isoformat(),
    head=subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip(),
    status='FROZEN for serial independent review then Root native acceptance; release before any source or binary change',
    source_files=sources,
    source_snapshot_sha256=hashlib.sha256(json.dumps(sources, sort_keys=True, separators=(',', ':')).encode()).hexdigest(),
    final_build_log=record(research / 'product-v10-build.log'),
    final_native_clippy_log=record(research / 'product-v10-clippy.log'),
    checks=gates, previous_freeze=record(previous), implementation_delta=record(delta_path),
    default_test_count=150, ui_feature_test_count=68,
    ui_feature_breakdown={'pure': 23, 'actual_gpui': 45},
    native_runtime_status='NOT_RUN for V10 by implementer; Root exclusively controls desktop',
    config_fixtures={path.name: record(path) for path in sorted((research / 'native').glob('v8-*.toml'))},
    limits=['V10 mixed-DPI fit and titlebar visibility still require Root native read-back and screenshots',
            '800x600, 390x844, 320x640 client layout/scroll are headless tested, not native acceptance',
            'Physical unplug/reconnect/port tests and real single/zero remain NOT_RUN',
            'Gate B incomplete; no08-11 implementation'],
)
freeze.pop('reused_checks_reason', None)
freeze.pop('review_delta', None)
freeze['observations'].extend([
    'restoring target client size after creation: actual client bounds and requested logical size',
    'window placement observation additionally records actual_usable_bounds and requested_client_size',
])
path = research / 'product-native-freeze-v10.json'
save(path, freeze)
assert all(sha(root / path) == digest for path, digest in sources.items())
print(json.dumps({'source_count': len(sources), 'source_sha256': freeze['source_snapshot_sha256'],
                  'binary': freeze['binary'], 'freeze_sha256': sha(path)}, indent=2))
