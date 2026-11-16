"""Bind V11 product checks without altering the failed V10 native evidence."""
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

previous = research / 'product-native-freeze-v10.json'
freeze = json.loads(previous.read_text(encoding='utf-8'))
sources = {path: sha(root / path) for path in freeze['source_files']}
changes = {path: {'v10': digest, 'v11': sources[path]} for path, digest in freeze['source_files'].items() if sources[path] != digest}
assert set(changes) == {'crates/ferrostage-ui/src/shell.rs', '.trellis/spec/frontend/state-management.md'}, changes
delta = research / 'product-v11-source-delta.json'
save(delta, {'changes': changes, 'added': {}, 'source_count': len(sources), 'unchanged_count': len(sources) - len(changes)})
checks = {}
for name in ['just-ci', 'headless-final', 'clippy', 'deny', 'build']:
    path = research / f'product-v11-{name}.json'
    assert json.loads(path.read_text(encoding='utf-8'))['exit_code'] == 0, name
    checks[name] = record(path)
for key in ['binary', 'pdfium', 'fixture', 'cargo_lock']:
    current = record(Path(freeze[key]['path']))
    if key != 'binary':
        assert current['sha256'] == freeze[key]['sha256'], key
    freeze[key] = current
freeze.update(
    captured_utc=datetime.now(timezone.utc).isoformat(),
    head=subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip(),
    status='FROZEN for serial independent check then Root native acceptance; release before source or executable changes',
    source_files=sources,
    source_snapshot_sha256=hashlib.sha256(json.dumps(sources, sort_keys=True, separators=(',', ':')).encode()).hexdigest(),
    final_build_log=record(research / 'product-v11-build.log'),
    final_native_clippy_log=record(research / 'product-v11-clippy.log'),
    checks=checks, previous_freeze=record(previous), implementation_delta=record(delta),
    default_test_count=150, ui_feature_test_count=70, ui_feature_breakdown={'pure': 23, 'actual_gpui': 47},
    native_runtime_status='V11 NOT_RUN by implementer; V10 native startup FAIL remains historical failure, not normal close PASS',
    config_fixtures={path.name: record(path) for path in sorted((research / 'native').glob('v8-*.toml'))},
    limits=['V11 frame correction and actual C startup require Root native verification',
            'V10 headless evidence did not model native frame error; new test models actual4x9 physical pixel loss through real GPUI callbacks',
            'Actual native narrow client sizes, Presenting migration/no-play, save retry/future settings matrix remain for Root',
            'Physical unplug/reconnect/ports and real single/zero remain NOT_RUN; Gate B incomplete; no08-11 implementation'],
)
freeze['observations'].append('restoring target client size after creation: immutable target, actual bounds, request and correction boolean; at most initial request plus one observed-delta correction')
path = research / 'product-native-freeze-v11.json'
save(path, freeze)
assert all(sha(root / path) == digest for path, digest in sources.items())
print(json.dumps({'source_count': len(sources), 'source_sha256': freeze['source_snapshot_sha256'], 'binary': freeze['binary'], 'freeze_sha256': sha(path)}, indent=2))
