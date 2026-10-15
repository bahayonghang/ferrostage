"""Read back frozen V15 sources and seal only newly created evidence."""
from datetime import datetime, timezone
from pathlib import Path
import hashlib
import json
import subprocess

root = Path.cwd()
research = root / '.trellis/tasks/10-14-07-gpui-shell/research'
sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
freeze_path = research / 'product-native-freeze-v15.json'
freeze = json.loads(freeze_path.read_text(encoding='utf-8'))
assert all(sha(root / path) == digest for path, digest in freeze['source_files'].items())
for key in ['binary', 'pdfium', 'fixture', 'cargo_lock']:
    assert sha(Path(freeze[key]['path'])) == freeze[key]['sha256'], key
inputs = {}
for name, item in freeze['config_fixtures'].items():
    path = Path(item['path'])
    assert sha(path) == item['sha256']
    inputs[name] = {'original_path': str(path), 'sha256': sha(path), 'utf8_text': path.read_bytes().decode('utf-8')}
with (research / 'product-v15-config-inputs.json').open('x', encoding='utf-8') as stream:
    json.dump(inputs, stream, ensure_ascii=False, indent=2)
    stream.write('\n')
diff = subprocess.run(['git', 'diff', '--check'], capture_output=True, text=True, check=False)
assert diff.returncode == 0, diff.stdout + diff.stderr
with (research / 'product-v15-readback.json').open('x', encoding='utf-8') as stream:
    json.dump({'captured_utc': datetime.now(timezone.utc).isoformat(),
               'source_verified': len(freeze['source_files']), 'binary_pdfium_fixture_lock_verified': True,
               'config_inputs_verified': len(inputs), 'diff_check': {'exit_code': diff.returncode, 'stdout': diff.stdout, 'stderr': diff.stderr}}, stream, indent=2)
    stream.write('\n')
paths = set(research.glob('product-v15-*')) | set((research / 'product-v15-accesskit-candidate').rglob('*')) | {freeze_path}
files = {path.relative_to(root).as_posix(): {'bytes': path.stat().st_size, 'sha256': sha(path)}
         for path in sorted(paths) if path.is_file()}
target = research / 'product-v15-evidence-manifest.json'
with target.open('x', encoding='utf-8') as stream:
    json.dump({'captured_utc': datetime.now(timezone.utc).isoformat(),
               'scope': 'V15 authorized F13 product implementation plus research-only unadopted F14 candidate. Immutable config inputs preserved for Root. No native acceptance claimed; Root-only hidden-window harness NOT_RUN at handoff.',
               'freeze_sha256': sha(freeze_path), 'source_snapshot_sha256': freeze['source_snapshot_sha256'], 'files': files}, stream, indent=2)
    stream.write('\n')
with target.with_suffix('.sha256').open('x', encoding='utf-8') as stream:
    stream.write(sha(target) + '  ' + target.name + '\n')
assert all(sha(root / path) == item['sha256'] for path, item in files.items())
print(json.dumps({'records': len(files), 'manifest_sha256': sha(target), 'source_verified': len(freeze['source_files']), 'binary_verified': True}, indent=2))


