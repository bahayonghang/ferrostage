"""Freeze the authorized F14 adoption without changing any V15 evidence."""
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

previous = research / 'product-native-freeze-v15.json'
freeze = json.loads(previous.read_text(encoding='utf-8'))
sources = {path: sha(root / path) for path in freeze['source_files']}
changes = {path: {'v15': digest, 'v16': sources[path]} for path, digest in freeze['source_files'].items() if sources[path] != digest}
assert set(changes) == {
    'Cargo.toml', 'Cargo.lock', 'vendor/README.md',
    '.trellis/spec/frontend/maintained-dependencies.md',
    'docs/adr/ADR-008-gpui-dependency-gate.md',
}, changes
additional = list((root / 'vendor/accesskit_windows').rglob('*')) + [
    root / 'AGENTS.md', root / 'docs/adr/ADR-001-gpui-from-zed-tag.md',
    root / 'docs/PROJECT_PLAN.md', research / 'native-v16-plan.md',
    research / 'native/run-native-v16.ps1',
]
added = {path.relative_to(root).as_posix(): sha(path) for path in additional if path.is_file() and path.relative_to(root).as_posix() not in sources}
sources.update(added)
sources = dict(sorted(sources.items()))
delta = research / 'product-v16-source-delta.json'
save(delta, {'changes': changes, 'added': added, 'source_count': len(sources), 'unchanged_count': len(freeze['source_files']) - len(changes)})
checks = {}
for name in ['reconstruction', 'a11y-cache-access', 'hidden-compile', 'just-ci', 'headless-final-lock', 'typecheck', 'clippy', 'deny', 'build']:
    path = research / f'product-v16-{name}.json'
    assert json.loads(path.read_text(encoding='utf-8'))['exit_code'] == 0, name
    checks[name] = record(path)
for key in ['binary', 'pdfium', 'fixture', 'cargo_lock']:
    current = record(Path(freeze[key]['path']))
    if key in ['pdfium', 'fixture']:
        assert current['sha256'] == freeze[key]['sha256'], key
    freeze[key] = current
freeze.update(
    captured_utc=datetime.now(timezone.utc).isoformat(),
    head=subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip(),
    status='FROZEN for serial independent check then Root native F14 acceptance; six-package scope explicitly approved',
    source_files=sources,
    source_snapshot_sha256=hashlib.sha256(json.dumps(sources, sort_keys=True, separators=(',', ':')).encode()).hexdigest(),
    final_build_log=record(research / 'product-v16-build.log'),
    final_native_clippy_log=record(research / 'product-v16-clippy.log'),
    checks=checks, previous_freeze=record(previous), implementation_delta=record(delta),
    default_test_count=151, ui_feature_test_count=82, ui_feature_breakdown={'pure': 23, 'actual_gpui': 59},
    accesskit_regression_test_count=8,
    document_native_test_count=2,
    document_native_evidence={'status': 'REUSED source-bound V15 PASS, not rerun', 'receipt': record(research / 'product-v15-document-isolated.json'), 'reason': 'All first-party sources, document/PDFium dependencies, DLL and fixture unchanged; final lock only replaces same-version AccessKit source.'},
    native_runtime_status='V16 NOT_RUN at implementer handoff; Root-only hidden HWND test compiled, not run',
    config_fixtures={'v16-core.toml': record(research / 'native/v16-core.toml')},
    accesskit_provenance=record(research / 'product-v16-accesskit-provenance.json'),
    accesskit_harness_files={path.relative_to(root).as_posix(): record(path) for path in sorted((research / 'product-v16-accesskit-harness').rglob('*')) if path.is_file()},
    limits=['F14 patch converts actual invalid-HWND ClientToScreen error only; no complete retirement/drain or HWND reuse guarantee',
            'Eight real COM tests create no HWND; Root-only 16-cycle hidden HWND test is compile-only at this handoff',
            'Actual product titlebar/Presenting shutdown and external UIA behavior remain Root-owned NOT_RUN',
            'V15 F13/resource and notes/document evidence is retained;07 notes UI remains placeholders until08',
            'User deferred size matrix; physical unplug/reconnect/ports and real single/zero NOT_RUN; Gate B incomplete;08-11 gated'],
)
freeze['observations'].append('V16 adopts approved four-file AccessKit0.34.0 patch as sixth maintained package; all other product runtime source unchanged.')
target = research / 'product-native-freeze-v16.json'
save(target, freeze)
assert all(sha(root / path) == digest for path, digest in sources.items())
print(json.dumps({'source_count': len(sources), 'source_sha256': freeze['source_snapshot_sha256'], 'binary': freeze['binary'], 'freeze_sha256': sha(target)}, indent=2))
