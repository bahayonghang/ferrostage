"""Freeze the bounded GPUI callback lifetime repair, preserving V13 evidence."""
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

previous = research / 'product-native-freeze-v13.json'
freeze = json.loads(previous.read_text(encoding='utf-8'))
sources = {path: sha(root / path) for path in freeze['source_files']}
changes = {path: {'v13': digest, 'v14': sources[path]} for path, digest in freeze['source_files'].items() if sources[path] != digest}
assert set(changes) == {
    'crates/ferrostage-ui/src/shell.rs',
    'vendor/gpui/src/window.rs',
    'vendor/gpui/src/app/test_context.rs',
    'vendor/gpui/src/platform/test/window.rs',
    '.trellis/spec/frontend/component-guidelines.md',
    '.trellis/spec/frontend/maintained-dependencies.md',
    'docs/adr/ADR-008-gpui-dependency-gate.md',
}, changes
delta = research / 'product-v14-source-delta.json'
save(delta, {'changes': changes, 'added': {}, 'source_count': len(sources), 'unchanged_count': len(sources) - len(changes)})

v12 = json.loads((research / 'product-native-freeze-v12.json').read_text(encoding='utf-8'))
default_inputs = {path: digest for path, digest in v12['source_files'].items()
                  if (path.startswith('crates/') and path != 'crates/ferrostage-ui/src/shell.rs')
                  or path in {'Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml', 'justfile', 'deny.toml', '.github/workflows/ci.yml'}}
assert all(sources[path] == digest for path, digest in default_inputs.items())
assert '#[cfg(feature = "gpui")]\nmod shell;' in (root / 'crates/ferrostage-ui/src/lib.rs').read_text()
reused = research / 'product-v14-default-reuse.json'
save(reused, {
    'status': 'REUSED verified default-only source scope; not rerun in V14',
    'previous_freeze': record(research / 'product-native-freeze-v12.json'),
    'just_ci_receipt': record(research / 'product-v12-just-ci.json'),
    'just_ci_log': record(research / 'product-v12-just-ci.log'),
    'default_test_count': 150, 'verified_unchanged_inputs': default_inputs,
    'reason': 'All default crate inputs/manifests/toolchain/CI/policy unchanged. Shell is gpui-gated; V14 Shell edits are tests only. GPUI is optional. V14 reruns current fmt, feature UI, strict consumers, typecheck, full all-feature deny and native build.',
    'not_reused': 'V12 all-feature deny is not reused; current V14 full graph receipt is required.',
})
checks = {'just-ci-default-reused': record(reused)}
assert json.loads((research / 'product-v12-just-ci.json').read_text())['exit_code'] == 0
for name in ['retirement-green', 'headless', 'fmt', 'vendor-fmt', 'typecheck', 'clippy', 'deny', 'build']:
    path = research / f'product-v14-{name}.json'
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
    status='FROZEN for serial independent check then Root native acceptance; release before source/executable changes',
    source_files=sources,
    source_snapshot_sha256=hashlib.sha256(json.dumps(sources, sort_keys=True, separators=(',', ':')).encode()).hexdigest(),
    final_build_log=record(research / 'product-v14-build.log'),
    final_native_clippy_log=record(research / 'product-v14-clippy.log'),
    checks=checks, previous_freeze=record(previous), implementation_delta=record(delta),
    default_test_count=150, ui_feature_test_count=82, ui_feature_breakdown={'pure': 23, 'actual_gpui': 59},
    native_runtime_status='V14 NOT_RUN; V13 F12 hover window-not-found remains historical FAIL despite exit0',
    config_fixtures={path.name: record(path) for path in sorted((research / 'native').glob('v8-*.toml'))},
    limits=['V14 GPUI tests prove owner-retirement cancellation for the17 event registrations; native F12 absence requires Root titlebar-close retest/full post-exit logs',
            'A11y Send/channel receivers are separately scheduled and not repaired or claimed tested by this callback regression',
            'on_close idempotent teardown/tab cleanup and first-party on_should_close interception retain their existing semantics',
            'Root owns remaining native matrix; Gate B incomplete; physical unplug/reconnect/ports and real single/zero NOT_RUN; no08-11 work'],
)
freeze['observations'].append('All17 Window-access event registrations share the Window-owned weak lifetime; deferred appearance rechecks at execution; live update errors remain logged')
path = research / 'product-native-freeze-v14.json'
save(path, freeze)
assert all(sha(root / path) == digest for path, digest in sources.items())
print(json.dumps({'source_count': len(sources), 'source_sha256': freeze['source_snapshot_sha256'], 'binary': freeze['binary'], 'freeze_sha256': sha(path)}, indent=2))
