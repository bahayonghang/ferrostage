"""Bind the checked V8 source, native executable and isolated input fixtures."""
from datetime import datetime, timezone
from pathlib import Path
import hashlib
import json
import subprocess

root = Path.cwd()
research = root / '.trellis/tasks/10-14-07-gpui-shell/research'
target = research / 'product-native-freeze-v8.json'
assert not target.exists(), 'Preserve previous freezes'
sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()

def record(path):
    return {'path': str(path), 'resolved_path': str(path.resolve()),
            'bytes': path.stat().st_size, 'sha256': sha(path)}

files = []
for folder in ['crates', 'vendor', 'assets/locales']:
    files.extend(path for path in (root / folder).rglob('*') if path.is_file())
for name in ['Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml', 'deny.toml', 'justfile',
             '.github/workflows/ci.yml', 'DESIGN.md', 'docs/packaging.md',
             'docs/adr/ADR-008-gpui-dependency-gate.md',
             '.trellis/spec/frontend/index.md',
             '.trellis/spec/frontend/component-guidelines.md',
             '.trellis/spec/frontend/state-management.md',
             '.trellis/spec/frontend/maintained-dependencies.md']:
    files.append(root / name)
sources = {path.relative_to(root).as_posix(): sha(path) for path in sorted(set(files))}
source_hash = hashlib.sha256(json.dumps(sources, sort_keys=True, separators=(',', ':')).encode()).hexdigest()
exe = root / 'target/debug/ferrostage.exe'
dll = root / 'cache/pdfium-7881-win-x64/bin/pdfium.dll'
assert sha(dll) == '79d4676b656cfb1abcea88f9ade3b4b0826c5200382db5f4ec72a636c598c118'
gates = {}
for name in ['just-ci', 'headless-final', 'clippy-final', 'deny', 'build']:
    receipt = research / f'product-v8-{name}.json'
    value = json.loads(receipt.read_text(encoding='utf-8'))
    assert value['exit_code'] == 0, name
    gates[name] = record(receipt)
fixture_configs = {path.name: record(path) for path in sorted((research / 'native').glob('v8-*.toml'))}
assert len(fixture_configs) == 5
freeze = {
    'captured_utc': datetime.now(timezone.utc).isoformat(),
    'status': 'FROZEN for independent review and Root native acceptance; no further source/executable changes without a new identity',
    'head': subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip(),
    'source_algorithm': 'SHA256 of UTF-8 JSON(source_files,sort_keys=True,separators=(comma,colon))',
    'source_snapshot_sha256': source_hash,
    'source_files': sources,
    'binary': record(exe), 'pdfium': record(dll), 'cargo_lock': record(root / 'Cargo.lock'),
    'fixture': record(root / 'fixtures/pdf/raster.pdf'),
    'final_build_log': record(research / 'product-v8-build.log'),
    'final_native_clippy_log': record(research / 'product-v8-clippy-final.log'),
    'checks': gates, 'config_fixtures': fixture_configs,
    'native_runtime_status': 'NOT_RUN by implementer; Root exclusively controls desktop',
    'default_test_count': 148, 'ui_feature_test_count': 62,
    'ui_feature_breakdown': {'pure': 21, 'actual_gpui': 41},
    'launch_argv': [str(exe), '--config', str(research / 'native/v8-light.toml'),
                    '--pdfium-library', str(dll), str(root / 'fixtures/pdf/raster.pdf')],
    'observations': [
        'display settings opened: stage, zero-based page, elapsed_ms, topology revision',
        'display assignment preparing: stage, zero-based page, elapsed_ms and candidate DisplaySettings',
        'identify display: shared number, MonitorId, target GPUI display id and WindowId',
        'window bounds changed: WindowId, bounds, viewport, scale and current display id',
        'window placement observation: requested/actual monitor, bounds and fullscreen',
        'window transaction committed: stage, zero-based page, elapsed_ms and current handles',
    ],
    'limits': ['No physical unplug/reconnect/port tests', 'Real single/zero display NOT_RUN',
               'Windows HRESULT cleanup and native new-panel acceptance require Root retest',
               'No task08-11 implementation; Gate B remains incomplete'],
}
target.write_text(json.dumps(freeze, indent=2, ensure_ascii=False) + '\n', encoding='utf-8')
old = json.loads((research / 'product-native-freeze-v7.json').read_text(encoding='utf-8'))
delta = {path: {'v7': digest, 'v8': sources.get(path)}
         for path, digest in old['source_files'].items() if sources.get(path) != digest}
added = {path: digest for path, digest in sources.items() if path not in old['source_files']}
with (research / 'product-v8-source-delta.json').open('x', encoding='utf-8') as output:
    json.dump({'changes': delta, 'added': added, 'previous_count': len(old['source_files']),
               'current_count': len(sources), 'unchanged_count': len(old['source_files']) - len(delta)}, output, indent=2)
    output.write('\n')
assert all(sha(root / path) == digest for path, digest in sources.items())
print(json.dumps({'source_count': len(sources), 'source_sha256': source_hash,
                  'binary': freeze['binary'], 'freeze_sha256': sha(target)}, indent=2))
