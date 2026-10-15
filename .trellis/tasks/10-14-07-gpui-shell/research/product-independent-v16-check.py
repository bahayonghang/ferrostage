"""Serial independent checks; never starts an application or creates HWNDs."""
from datetime import datetime, timezone
from pathlib import Path
import json
import os
import subprocess
import sys

research = Path(__file__).resolve().parent
commands = {
    'reconstruction': ['python', str(research / 'product-v16-accesskit.py'), 'verify'],
    'lint': ['cargo', 'clippy', '-p', 'ferrostage-app', '-p', 'ferrostage-ui', '-p', 'ferrostage-document', '-p', 'ferrostage-platform', '--all-features', '--all-targets', '--locked', '--offline', '--', '-D', 'warnings'],
    'typecheck': ['cargo', 'check', '-p', 'ferrostage-app', '-p', 'ferrostage-ui', '-p', 'ferrostage-document', '-p', 'ferrostage-platform', '--all-features', '--all-targets', '--locked', '--offline'],
    'fmt': ['cargo', 'fmt', '--all', '--', '--check'],
}
name = sys.argv[1]
suffix = '-' + sys.argv[2] if len(sys.argv) > 2 else ''
log = research / f'product-independent-v16-{name}{suffix}.log'
receipt = log.with_suffix('.json')
assert not log.exists() and not receipt.exists()
record = {'started_utc': datetime.now(timezone.utc).isoformat(), 'argv': commands[name], 'status': 'RUNNING', 'log': log.name}
receipt.write_text(json.dumps(record, indent=2) + '\n', encoding='utf-8')
with log.open('x', encoding='utf-8') as stream:
    result = subprocess.run(commands[name], stdout=stream, stderr=subprocess.STDOUT, env=os.environ.copy())
record.update(exit_code=result.returncode, status='RETURNED', ended_utc=datetime.now(timezone.utc).isoformat())
receipt.write_text(json.dumps(record, indent=2) + '\n', encoding='utf-8')
print(json.dumps(record, indent=2))
print('\n'.join(log.read_text(encoding='utf-8', errors='replace').splitlines()[-35:]))
sys.exit(result.returncode)
