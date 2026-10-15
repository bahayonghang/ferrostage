"""Freeze the first-party owned async-wait lifecycle repair and current isolated inputs."""
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

previous = research / 'product-native-freeze-v12.json'
freeze = json.loads(previous.read_text(encoding='utf-8'))
sources = {path: sha(root / path) for path in freeze['source_files']}
changes = {path: {'v12': digest, 'v13': sources[path]} for path, digest in freeze['source_files'].items() if sources[path] != digest}
assert set(changes) == {'crates/ferrostage-ui/src/shell.rs', '.trellis/spec/frontend/component-guidelines.md'}, changes
delta = research / 'product-v13-source-delta.json'
save(delta, {'changes': changes, 'added': {}, 'source_count': len(sources), 'unchanged_count': len(sources) - len(changes)})
checks = {}
for name in ['just-ci', 'headless', 'clippy', 'deny', 'build']:
    version = 'v12' if name in ['just-ci', 'deny'] else 'v13'
    path = research / f'product-{version}-{name}.json'
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
    status='FROZEN for completed independent review/self-fix then Root native acceptance; release before source/executable changes',
    source_files=sources,
    source_snapshot_sha256=hashlib.sha256(json.dumps(sources, sort_keys=True, separators=(',', ':')).encode()).hexdigest(),
    final_build_log=record(research / 'product-v13-build.log'),
    final_native_clippy_log=record(research / 'product-v13-clippy.log'),
    checks=checks, previous_freeze=record(previous), implementation_delta=record(delta),
    default_test_count=150, ui_feature_test_count=77, ui_feature_breakdown={'pure': 23, 'actual_gpui': 54},
    native_runtime_status='V13 NOT_RUN by implementer; historical V11 F11 dispatcher HRESULT remains FAIL despite exit0',
    config_fixtures={path.name: record(path) for path in sorted((research / 'native').glob('v8-*.toml'))},
    limits=['V13 F11 close/late dispatcher error requires Root native retest; GPUI tests prove task cancellation, not Windows HRESULT absence',
            'Independent review confirms scoped V9 Identify/V11 UI applicability; this is not V13 native evidence',
            'Root owns remaining native matrix; Gate B remains incomplete; physical unplug/reconnect/ports and real single/zero NOT_RUN; no08-11 work'],
)
freeze['observations'].append('Shell owns/cancels polling, import-read and path-prompt continuations before quit; dispatcher errors remain unchanged')
path = research / 'product-native-freeze-v13.json'
save(path, freeze)
assert all(sha(root / path) == digest for path, digest in sources.items())
print(json.dumps({'source_count': len(sources), 'source_sha256': freeze['source_snapshot_sha256'], 'binary': freeze['binary'], 'freeze_sha256': sha(path)}, indent=2))
