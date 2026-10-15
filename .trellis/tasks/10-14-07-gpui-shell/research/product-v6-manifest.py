"""Finalize immutable V6 receipts without rewriting earlier manifests."""
from datetime import datetime, timezone
from pathlib import Path
import hashlib
import json

root = Path.cwd()
research = root / '.trellis/tasks/10-14-07-gpui-shell/research'
sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
freeze_path = research / 'product-native-freeze-v6.json'
freeze = json.loads(freeze_path.read_text(encoding='utf-8'))
assert all(sha(root / path) == digest for path, digest in freeze['source_files'].items())
assert sha(Path(freeze['binary']['resolved_path'])) == freeze['binary']['sha256']
status = (research / 'product-status.txt').read_text(encoding='utf-8')
start = status.index('RUNNING: cargo test -p ferrostage-ui --features test-support --lib v6_')
end_marker = 'V6 DIFF CHECK EXIT: 0'
end = status.index(end_marker, start) + len(end_marker)
with (research / 'product-v6-command-status.txt').open('x', encoding='utf-8', newline='\n') as output:
    output.write(status[start:end] + '\n')
paths = set(research.glob('product-v6-*')) | {freeze_path,
    research / 'product-native-freeze-v5.json', research / 'product-freeze.py'}
files = {path.relative_to(root).as_posix(): {'bytes': path.stat().st_size, 'sha256': sha(path)}
         for path in sorted(paths) if path.is_file()}
record = {'captured_utc': datetime.now(timezone.utc).isoformat(),
          'scope': 'V6 F7/F8 implementation receipts; Root native and independent review are separate. '
                   'The command ledger is extracted at closeout, not a new execution.',
          'freeze_sha256': sha(freeze_path),
          'source_snapshot_sha256': freeze['source_snapshot_sha256'], 'files': files}
target = research / 'product-v6-evidence-manifest.json'
with target.open('x', encoding='utf-8', newline='\n') as output:
    output.write(json.dumps(record, indent=2) + '\n')
digest = sha(target)
with target.with_suffix('.sha256').open('x', encoding='utf-8', newline='\n') as output:
    output.write(digest + '  ' + target.name + '\n')
assert all(sha(root / path) == item['sha256'] for path, item in
           json.loads(target.read_text(encoding='utf-8'))['files'].items())
print(json.dumps({'records': len(files), 'sha256': digest,
                  'source_verified': len(freeze['source_files']), 'binary_verified': True}, indent=2))
