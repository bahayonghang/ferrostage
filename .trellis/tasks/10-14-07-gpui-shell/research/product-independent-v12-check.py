"""Independent V12 identity and check receipts; preserve all prior evidence."""
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
    'lint': ['cargo', 'clippy', '-p', 'ferrostage-app', '-p', 'ferrostage-ui', '--all-features', '--all-targets', '--locked', '--offline', '--', '-D', 'warnings'],
    'types': ['cargo', 'check', '-p', 'ferrostage-app', '-p', 'ferrostage-ui', '--all-features', '--all-targets', '--locked', '--offline'],
    'retirement': ['cargo', 'test', '-p', 'ferrostage-ui', '--features', 'test-support', '--lib', '--locked', '--offline', 'retirement_v', '--', '--test-threads=1'],
    'narrow': ['cargo', 'test', '-p', 'ferrostage-ui', '--features', 'test-support', '--lib', '--locked', '--offline', 'narrow_v10', '--', '--test-threads=1'],
    'diff': ['git', 'diff', '--check'],
}
name = sys.argv[1]
suffix = '-' + sys.argv[2] if len(sys.argv) > 2 else ''
prefix = research / f'product-independent-v12-{name}{suffix}'
receipt = prefix.with_suffix('.json')
assert not receipt.exists()
if name == 'identity':
    path = research / 'product-native-freeze-v12.json'
    freeze = json.loads(path.read_text())
    sources = {p: sha(root / p) for p in freeze['source_files']}
    mismatches = [p for p, h in freeze['source_files'].items() if sources[p] != h]
    aggregate = hashlib.sha256(json.dumps(sources, sort_keys=True, separators=(',', ':')).encode()).hexdigest()
    assert aggregate == freeze['source_snapshot_sha256']
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
    manifests = {}
    for name in ['product-v12-evidence-manifest.json', 'native-v11-evidence-manifest.json', 'product-independent-v11-manifest.json', 'native-v9-evidence-manifest.json']:
        manifest = json.loads((research / name).read_text())
        for p, value in manifest['files'].items():
            path = root / p
            path = path if path.exists() else research / p
            digest = value if isinstance(value, str) else value['sha256']
            if sha(path) != digest:
                mismatches.append(str(path))
            if isinstance(value, dict) and path.stat().st_size != value['bytes']:
                mismatches.append(str(path))
        manifests[name] = {'count': len(manifest['files']), 'sha256': sha(research / name)}
    assert not mismatches, mismatches
    value = {'at_utc': datetime.now(timezone.utc).isoformat(), 'source_count': len(sources),
             'source_sha256': aggregate, 'binary_sha256': sha(Path(freeze['binary']['path'])),
             'freeze_sha256': sha(research / 'product-native-freeze-v12.json'),
             'manifests': manifests, 'mismatches': mismatches, 'status': 'PASS'}
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
