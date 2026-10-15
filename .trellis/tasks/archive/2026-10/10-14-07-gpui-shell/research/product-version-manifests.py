"""Close v3-v5 implementation evidence without modifying any frozen input."""
from datetime import datetime, timezone
from pathlib import Path
import hashlib
import json

root = Path.cwd()
research = root / '.trellis/tasks/10-14-07-gpui-shell/research'


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def write_new(path, text):
    with path.open('x', encoding='utf-8', newline='\n') as output:
        output.write(text)


current_path = research / 'product-native-freeze-v5.json'
current = json.loads(current_path.read_text(encoding='utf-8'))
differences = [path for path, digest in current['source_files'].items()
               if sha(root / path) != digest]
assert not differences, differences

status = (research / 'product-status.txt').read_text(encoding='utf-8')
starts = {
    3: 'RUNNING: cargo test -p ferrostage-ui --features test-support --lib close_hook',
    4: 'RUNNING: cargo test -p ferrostage-ui --features test-support --lib input_route',
    5: 'RUNNING: cargo test -p ferrostage-ui --features test-support --lib v5_',
}
results = []
for version in (3, 4, 5):
    freeze_path = research / f'product-native-freeze-v{version}.json'
    freeze = json.loads(freeze_path.read_text(encoding='utf-8'))
    start = status.index(starts[version])
    final_exit = f'V{version} NATIVE BUILD EXIT: 0'
    end = status.index(final_exit, start) + len(final_exit)
    write_new(research / f'product-v{version}-command-status.txt',
              status[start:end] + '\n')
    paths = sorted(research.glob(f'product-v{version}-*'))
    paths += [freeze_path, research / f'product-native-freeze-v{version - 1}.json',
              research / 'product-freeze.py', Path(__file__).resolve()]
    files = {path.relative_to(root).as_posix():
             {'bytes': path.stat().st_size, 'sha256': sha(path)}
             for path in sorted(set(paths)) if path.is_file()}
    manifest = {
        'captured_utc': datetime.now(timezone.utc).isoformat(),
        'scope': f'Immutable v{version} implementation receipts, finalized after v5 freeze. '
                 'Root native captures and review reports are separate. '
                 'Command status is extracted from the existing chronological ledger; '
                 'capture time is not command execution time.',
        'freeze_sha256': sha(freeze_path),
        'source_snapshot_sha256': freeze['source_snapshot_sha256'],
        'files': files,
    }
    target = research / f'product-v{version}-evidence-manifest.json'
    write_new(target, json.dumps(manifest, indent=2) + '\n')
    digest = sha(target)
    write_new(target.with_suffix('.sha256'), digest + '  ' + target.name + '\n')
    reread = json.loads(target.read_text(encoding='utf-8'))
    assert all(sha(root / path) == record['sha256']
               for path, record in reread['files'].items())
    results.append({'version': version, 'records': len(files), 'sha256': digest})

print(json.dumps({'frozen_source_checked': len(current['source_files']),
                  'differences': differences, 'manifests': results}, indent=2))
