"""Retain exact V10 check output without changing previous receipts."""
from datetime import datetime, timezone
from pathlib import Path
import json
import subprocess
import sys

research = Path(__file__).resolve().parent
commands = {
    'placement': ['cargo', 'test', '-p', 'ferrostage-ui', '--features', 'test-support', '--lib', '--locked', '--offline', 'placement_v10', '--', '--test-threads=1'],
    'narrow': ['cargo', 'test', '-p', 'ferrostage-ui', '--features', 'test-support', '--lib', '--locked', '--offline', 'narrow_v10', '--', '--test-threads=1'],
    'headless': ['cargo', 'test', '-p', 'ferrostage-ui', '--features', 'test-support', '--lib', '--locked', '--offline'],
    'fmt': ['cargo', 'fmt', '--all', '--', '--check'],
    'just-ci': ['just', 'ci'],
    'clippy': ['cargo', 'clippy', '-p', 'ferrostage-app', '-p', 'ferrostage-ui', '--all-features', '--all-targets', '--locked', '--offline', '--', '-D', 'warnings'],
    'build': ['cargo', 'build', '-p', 'ferrostage-app', '--features', 'gpui,pdfium', '--locked', '--offline'],
    'deny': ['cargo', 'deny', '--all-features', '--locked', '--offline', 'check'],
}
name = sys.argv[1]
suffix = '-' + sys.argv[2] if len(sys.argv) > 2 else ''
log = research / f'product-v10-{name}{suffix}.log'
receipt = log.with_suffix('.json')
assert not log.exists() and not receipt.exists(), 'Preserve previous command receipts'
record = {'started_utc': datetime.now(timezone.utc).isoformat(), 'argv': commands[name], 'status': 'RUNNING', 'log': log.name}
receipt.write_text(json.dumps(record, indent=2) + '\n', encoding='utf-8')
with log.open('x', encoding='utf-8') as stream:
    completed = subprocess.run(commands[name], stdout=stream, stderr=subprocess.STDOUT, check=False)
record.update(status='RETURNED', exit_code=completed.returncode, ended_utc=datetime.now(timezone.utc).isoformat())
receipt.write_text(json.dumps(record, indent=2) + '\n', encoding='utf-8')
print(json.dumps(record, indent=2))
print('\n'.join(log.read_text(encoding='utf-8', errors='replace').splitlines()[-32:]))
sys.exit(completed.returncode)
