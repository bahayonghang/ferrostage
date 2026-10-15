"""Independent review commands; never overwrite an existing receipt."""
from datetime import datetime, timezone
from pathlib import Path
import hashlib
import json
import subprocess
import sys

research = Path(__file__).resolve().parent
root = Path.cwd()
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
commands = {
    'fmt': ['cargo', 'fmt', '--all', '--', '--check'],
    'clippy': ['cargo', 'clippy', '-p', 'ferrostage-app', '-p', 'ferrostage-ui', '--all-features', '--all-targets', '--locked', '--offline', '--', '-D', 'warnings'],
    'typecheck': ['cargo', 'check', '-p', 'ferrostage-app', '-p', 'ferrostage-ui', '--all-features', '--all-targets', '--locked', '--offline'],
    'tests': ['cargo', 'test', '-p', 'ferrostage-ui', '--features', 'test-support', '--lib', '--locked', '--offline'],
    'build': ['cargo', 'build', '-p', 'ferrostage-app', '--features', 'gpui,pdfium', '--locked', '--offline'],
    'diff': ['git', 'diff', '--check'],
}
name = sys.argv[1]
suffix = '-' + sys.argv[2] if len(sys.argv) > 2 else ''
prefix = research / f'product-independent-v8-{name}{suffix}'
receipt = prefix.with_suffix('.json')
assert not receipt.exists()
if name == 'identity':
    freeze_path = research / 'product-native-freeze-v8.json'
    freeze = json.loads(freeze_path.read_text())
    mismatches = []
    sources = {p: sha(root / p) for p in freeze['source_files']}
    for p, h in freeze['source_files'].items():
        if sources[p] != h:
            mismatches.append(p)
    actual_source_hash = hashlib.sha256(json.dumps(sources, sort_keys=True, separators=(',', ':')).encode()).hexdigest()
    assert actual_source_hash == freeze['source_snapshot_sha256']
    def check_record(value):
        if isinstance(value, dict):
            if 'path' in value and 'sha256' in value:
                p = Path(value['path'])
                if sha(p) != value['sha256'] or p.stat().st_size != value['bytes']:
                    mismatches.append(str(p))
            else:
                for v in value.values():
                    check_record(v)
    check_record(freeze)
    manifest = json.loads((research / 'product-v8-evidence-manifest.json').read_text())
    for p, record in manifest['files'].items():
        path = root / p
        if sha(path) != record['sha256'] or path.stat().st_size != record['bytes']:
            mismatches.append(p)
    assert not mismatches, mismatches
    value = {'checked_utc': datetime.now(timezone.utc).isoformat(), 'status': 'PASS',
             'source_count': len(sources), 'source_hash': actual_source_hash,
             'binary_sha256': sha(Path(freeze['binary']['path'])),
             'freeze_sha256': sha(freeze_path), 'evidence_count': len(manifest['files']),
             'evidence_manifest_sha256': sha(research / 'product-v8-evidence-manifest.json'),
             'mismatches': mismatches}
    receipt.write_text(json.dumps(value, indent=2) + '\n')
    print(json.dumps(value, indent=2))
else:
    log = prefix.with_suffix('.log')
    assert not log.exists()
    value = {'started_utc': datetime.now(timezone.utc).isoformat(), 'argv': commands[name], 'status': 'RUNNING', 'log': log.name}
    receipt.write_text(json.dumps(value, indent=2) + '\n')
    with log.open('x', encoding='utf-8') as output:
        result = subprocess.run(commands[name], stdout=output, stderr=subprocess.STDOUT, check=False)
    value.update(status='RETURNED', exit_code=result.returncode, ended_utc=datetime.now(timezone.utc).isoformat())
    receipt.write_text(json.dumps(value, indent=2) + '\n')
    print(json.dumps(value, indent=2))
    print('\n'.join(log.read_text(encoding='utf-8', errors='replace').splitlines()[-12:]))
    sys.exit(result.returncode)
