"""Immutable V16 dependency adoption and product command receipts."""
from datetime import datetime, timezone
from pathlib import Path
import json
import hashlib
import os
import subprocess
import sys

research = Path(__file__).resolve().parent
commands = {
    'lock': ['cargo', 'update', '-p', 'accesskit_windows', '--offline'],
    'reconstruction': ['python', str(research / 'product-v16-accesskit.py'), 'verify'],
    'a11y': ['cargo', 'test', '--manifest-path', str(research / 'product-v16-accesskit-harness/Cargo.toml'), '--lib', '--offline', 'v16_regression_tests', '--', '--test-threads=1', '--nocapture'],
    'hidden-compile': ['cargo', 'test', '--manifest-path', str(research / 'product-v16-accesskit-harness/Cargo.toml'), '--lib', '--locked', '--offline', '--features', 'root-native-window', '--no-run'],
    'root-hidden': ['cargo', 'test', '--manifest-path', str(research / 'product-v16-accesskit-harness/Cargo.toml'), '--lib', '--locked', '--offline', '--features', 'root-native-window', 'v16_regression_tests::root_only_owned_hidden_hwnd_destroy_with_retained_context', '--', '--exact', '--test-threads=1', '--nocapture'],
    'getdc': ['cargo', 'test', '-p', 'ferrostage-platform', '--lib', '--locked', '--offline', 'getdc_v15', '--', '--test-threads=1', '--nocapture'],
    'platform': ['cargo', 'test', '-p', 'ferrostage-platform', '--locked', '--offline'],
    'headless': ['cargo', 'test', '-p', 'ferrostage-ui', '--features', 'test-support', '--lib', '--locked', '--offline'],
    'fmt': ['cargo', 'fmt', '--all', '--', '--check'],
    'just-ci': ['just', 'ci'],
    'document': ['cargo', 'test', '-p', 'ferrostage-document', '--features', 'pdfium', '--test', 'native', '--test', 'notes_native', '--locked', '--offline', '--', '--test-threads=1', '--nocapture'],
    'typecheck': ['cargo', 'check', '-p', 'ferrostage-app', '-p', 'ferrostage-ui', '-p', 'ferrostage-document', '--all-features', '--all-targets', '--locked', '--offline'],
    'clippy': ['cargo', 'clippy', '-p', 'ferrostage-app', '-p', 'ferrostage-ui', '-p', 'ferrostage-document', '--all-features', '--all-targets', '--locked', '--offline', '--', '-D', 'warnings'],
    'build': ['cargo', 'build', '-p', 'ferrostage-app', '--features', 'gpui,pdfium', '--locked', '--offline'],
    'deny': ['cargo', 'deny', '--all-features', '--locked', '--offline', 'check'],
}
name = sys.argv[1]
suffix = '-' + sys.argv[2] if len(sys.argv) > 2 else ''
log = research / f'product-v16-{name}{suffix}.log'
receipt = log.with_suffix('.json')
assert not log.exists() and not receipt.exists(), 'Preserve previous command receipts'
record = {'started_utc': datetime.now(timezone.utc).isoformat(), 'argv': commands[name], 'status': 'RUNNING', 'log': log.name}
env = os.environ.copy()
if name in ('a11y', 'hidden-compile', 'root-hidden'):
    env['CARGO_TARGET_DIR'] = str(Path.cwd() / 'target')
if name == 'document':
    runtime = json.loads((research / 'product-native-freeze-v14.json').read_text())['pdfium']
    env['FERROSTAGE_TEST_PDFIUM'] = runtime['path']
    paths = [Path(runtime['path']), Path('fixtures/notes-functional/pympress-notes.pdf'), Path('fixtures/notes-functional/pympress-notes.tex')]
    record['inputs'] = {str(path): {'sha256': hashlib.sha256(path.read_bytes()).hexdigest(), 'bytes': path.stat().st_size} for path in paths}
receipt.write_text(json.dumps(record, indent=2) + '\n', encoding='utf-8')
with log.open('x', encoding='utf-8') as stream:
    completed = subprocess.run(commands[name], stdout=stream, stderr=subprocess.STDOUT, check=False, env=env)
record.update(status='RETURNED', exit_code=completed.returncode, ended_utc=datetime.now(timezone.utc).isoformat())
receipt.write_text(json.dumps(record, indent=2) + '\n', encoding='utf-8')
print(json.dumps(record, indent=2))
print('\n'.join(log.read_text(encoding='utf-8', errors='replace').splitlines()[-45:]))
sys.exit(completed.returncode)
