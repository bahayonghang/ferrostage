"""Run one diagnostic-build check, retaining its exact command and exit."""
from datetime import datetime, timezone
from pathlib import Path
import json
import subprocess
import sys

research = Path(__file__).resolve().parent
commands = {
    'fmt': ['cargo', 'fmt', '--all', '--', '--check'],
    'just-ci': ['just', 'ci'],
    'clippy': ['cargo', 'clippy', '-p', 'ferrostage-app', '-p', 'ferrostage-ui',
               '--all-features', '--all-targets', '--locked', '--offline', '--', '-D', 'warnings'],
    'headless': ['cargo', 'test', '-p', 'ferrostage-ui', '--features', 'test-support',
                 '--lib', '--locked', '--offline'],
    'build': ['cargo', 'build', '-p', 'ferrostage-app', '--features', 'gpui,pdfium',
              '--locked', '--offline'],
}
name = sys.argv[1]
suffix = '-' + sys.argv[2] if len(sys.argv) > 2 else ''
command = commands[name]
log = research / f'product-v7-{name}{suffix}.log'
receipt = research / f'product-v7-{name}{suffix}.json'
assert not log.exists() and not receipt.exists(), 'Preserve previous command receipts'
record = {'started_utc': datetime.now(timezone.utc).isoformat(), 'argv': command,
          'status': 'RUNNING', 'log': log.name}
receipt.write_text(json.dumps(record, indent=2) + '\n', encoding='utf-8')
with log.open('x', encoding='utf-8') as stream:
    completed = subprocess.run(command, stdout=stream, stderr=subprocess.STDOUT, check=False)
record.update(status='RETURNED', exit_code=completed.returncode,
              ended_utc=datetime.now(timezone.utc).isoformat())
receipt.write_text(json.dumps(record, indent=2) + '\n', encoding='utf-8')
print(json.dumps(record, indent=2))
print('\n'.join(log.read_text(encoding='utf-8', errors='replace').splitlines()[-24:]))
sys.exit(completed.returncode)
