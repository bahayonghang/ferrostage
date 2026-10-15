"""Read back and seal V8 implementation evidence without rewriting old receipts."""
from datetime import datetime, timezone
from pathlib import Path
import hashlib
import json
import subprocess

root = Path.cwd()
research = root / '.trellis/tasks/10-14-07-gpui-shell/research'
sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
freeze_path = research / 'product-native-freeze-v8.json'
freeze = json.loads(freeze_path.read_text(encoding='utf-8'))
assert all(sha(root / path) == digest for path, digest in freeze['source_files'].items())
for name in ['binary', 'pdfium', 'fixture', 'cargo_lock']:
    item = freeze[name]
    assert sha(Path(item['path'])) == item['sha256'], name

config_inputs = {}
for name, item in freeze['config_fixtures'].items():
    path = Path(item['path'])
    assert sha(path) == item['sha256']
    config_inputs[name] = {'original_path': str(path), 'sha256': sha(path),
                           'utf8_text': path.read_bytes().decode('utf-8')}
with (research / 'product-v8-config-inputs.json').open('x', encoding='utf-8') as output:
    json.dump(config_inputs, output, ensure_ascii=False, indent=2)
    output.write('\n')

diff = subprocess.run(['git', 'diff', '--check'], capture_output=True, text=True, check=False)
assert diff.returncode == 0, diff.stdout + diff.stderr
with (research / 'product-v8-readback.json').open('x', encoding='utf-8') as output:
    json.dump({'captured_utc': datetime.now(timezone.utc).isoformat(),
               'source_verified': len(freeze['source_files']),
               'binary_pdfium_fixture_lock_verified': True,
               'config_inputs_verified': len(config_inputs),
               'diff_check': {'argv': ['git', 'diff', '--check'],
                              'exit_code': diff.returncode, 'stdout': diff.stdout, 'stderr': diff.stderr}}, output, indent=2)
    output.write('\n')

paths = set(research.glob('product-v8-*')) | {freeze_path, research / 'resume-v8-baseline.json',
    root / '.trellis/tasks/10-14-presenter-v0.3/research/display-assignment-plan.md',
    root / '.trellis/tasks/10-14-presenter-v0.3/research/display-assignment-review.md'}
files = {path.relative_to(root).as_posix(): {'bytes': path.stat().st_size, 'sha256': sha(path)}
         for path in sorted(paths) if path.is_file()}
record = {'captured_utc': datetime.now(timezone.utc).isoformat(),
          'scope': 'V8 implementation and command receipts; native/check evidence remains separate. '
                   'Config input text is captured separately because runtime configs may change during Root acceptance.',
          'freeze_sha256': sha(freeze_path),
          'source_snapshot_sha256': freeze['source_snapshot_sha256'], 'files': files}
target = research / 'product-v8-evidence-manifest.json'
with target.open('x', encoding='utf-8') as output:
    json.dump(record, output, indent=2)
    output.write('\n')
with target.with_suffix('.sha256').open('x', encoding='utf-8') as output:
    output.write(sha(target) + '  ' + target.name + '\n')
assert all(sha(root / path) == item['sha256'] for path, item in files.items())
print(json.dumps({'records': len(files), 'manifest_sha256': sha(target),
                  'source_verified': len(freeze['source_files']), 'binary_verified': True}, indent=2))
