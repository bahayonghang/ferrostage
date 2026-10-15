"""Preserve V8 and freeze the independently reviewed, test-corrected V9."""
from datetime import datetime, timezone
from pathlib import Path
import hashlib
import json

root = Path.cwd()
research = root / '.trellis/tasks/10-14-07-gpui-shell/research'
sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()

def record(path):
    return {'path': str(path), 'resolved_path': str(path.resolve()),
            'bytes': path.stat().st_size, 'sha256': sha(path)}

previous = research / 'product-native-freeze-v8.json'
freeze = json.loads(previous.read_text(encoding='utf-8'))
assert sha(previous) == '4bd0a8840a4eeb6ca8d3be1976f8fe527eadd9c8c5851a61c4a2bf101f2a4130'
sources = {p: sha(root / p) for p in freeze['source_files']}
assert [p for p in sources if sources[p] != freeze['source_files'][p]] == ['crates/ferrostage-ui/src/shell.rs']
freeze.update(
    captured_utc=datetime.now(timezone.utc).isoformat(),
    status='FROZEN V9 after independent review; only added a real GPUI conflict/offline input regression to V8; Root native acceptance pending',
    source_files=sources,
    source_snapshot_sha256=hashlib.sha256(json.dumps(sources, sort_keys=True, separators=(',', ':')).encode()).hexdigest(),
    binary=record(Path(freeze['binary']['path'])),
    ui_feature_test_count=63,
    ui_feature_breakdown={'pure': 21, 'actual_gpui': 42},
    final_build_log=record(research / 'product-independent-v8-build-v9.log'),
    final_native_clippy_log=record(research / 'product-independent-v8-clippy-v9.log'),
    native_runtime_status='NOT_RUN by reviewer; Root exclusively controls desktop',
    previous_freeze=record(previous),
    review_delta=record(research / 'product-independent-v8-v9-source-delta.json'),
    reused_checks_reason='V8 just ci148 and full-graph deny receipts remain applicable: sole source delta is a cfg(all(test, feature = test-support)) regression. Dependency graph and all runtime code are unchanged. V9 reruns full feature tests, strict lint, typecheck, native build, fmt and diff checks.',
)
for key, name in [('headless-final', 'tests'), ('clippy-final', 'clippy'), ('typecheck', 'typecheck'), ('build', 'build'), ('fmt', 'fmt'), ('diff', 'diff')]:
    path = research / f'product-independent-v8-{name}-v9.json'
    assert json.loads(path.read_text())['exit_code'] == 0
    freeze['checks'][key] = record(path)
for value in freeze['config_fixtures'].values():
    assert sha(Path(value['path'])) == value['sha256']
path = research / 'product-native-freeze-v9.json'
with path.open('x', encoding='utf-8') as output:
    json.dump(freeze, output, indent=2, ensure_ascii=False)
    output.write('\n')
print(json.dumps({'source_count': len(sources), 'source_hash': freeze['source_snapshot_sha256'],
                  'exe_hash': freeze['binary']['sha256'], 'freeze_hash': sha(path)}, indent=2))
