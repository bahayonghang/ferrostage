"""Read frozen V16 evidence; write only a new resume audit receipt."""
from datetime import datetime, timezone
from pathlib import Path
import hashlib
import json
import re
import subprocess
import tomllib

root = Path.cwd()
research = root / '.trellis/tasks/10-14-07-gpui-shell/research'
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
read = lambda p: json.loads(p.read_text(encoding='utf-8-sig'))
freeze_path = research / 'product-native-freeze-v16.json'
freeze = read(freeze_path)
failures = []

def verify(path, digest, size=None):
    if not path.is_file():
        failures.append({'path': str(path), 'error': 'missing'})
        return
    actual = sha(path)
    if actual != digest.lower() or (size is not None and path.stat().st_size != size):
        failures.append({'path': str(path), 'expected': digest, 'actual': actual})

verify(freeze_path, '0e2b1ed8ef9299676ba249e7600bd8ab6734f00c6ae17f677467405ad60402d3')
for name, digest in freeze['source_files'].items():
    verify(root / name, digest)
aggregate = hashlib.sha256(json.dumps(freeze['source_files'], sort_keys=True, separators=(',', ':')).encode()).hexdigest()
assert aggregate == freeze['source_snapshot_sha256']
for key in ['binary', 'pdfium', 'fixture', 'cargo_lock']:
    entry = freeze[key]
    verify(Path(entry['path']), entry['sha256'], entry['bytes'])
for name, entry in freeze['accesskit_harness_files'].items():
    verify(root / name, entry['sha256'], entry['bytes'])

manifests = {}
for name in ['product-v16-evidence-manifest.json', 'product-independent-v16-evidence-manifest.json',
             'native-v16-evidence-manifest.json', 'native-v16-independent-evidence-manifest.json']:
    path = research / name
    data = read(path)
    entries = data.get('files', data)
    for item, entry in entries.items():
        verify(root / item, entry['sha256'], entry['bytes'])
    sidecar = path.with_suffix('.sha256')
    if sidecar.exists():
        assert sha(path) == sidecar.read_text().split()[0].lower()
    manifests[name] = {'count': len(entries), 'sha256': sha(path),
                       'jpg_count': sum(p.endswith('.jpg') for p in entries)}

checks = {}
for name in ['product-v16-reconstruction', 'product-v16-a11y-cache-access',
             'product-v16-hidden-compile', 'product-v16-root-hidden-cache-access',
             'product-v16-just-ci', 'product-v16-headless-final-lock',
             'product-v16-clippy', 'product-v16-typecheck', 'product-v16-deny', 'product-v16-build',
             'product-independent-v16-lint-cache-access', 'product-independent-v16-typecheck',
             'product-independent-v16-fmt']:
    receipt = read(research / (name + '.json'))
    assert receipt['exit_code'] == 0 and receipt['status'] == 'RETURNED', name
    log = research / receipt['log']
    text = log.read_text(encoding='utf-8-sig')
    results = re.findall(r'test result: ok\. (\d+) passed; (\d+) failed', text)
    checks[name] = {'receipt': receipt, 'log_sha256': sha(log),
                    'test_pass_sum': sum(int(p) for p, _ in results),
                    'test_fail_sum': sum(int(f) for _, f in results),
                    'log_tail': text.splitlines()[-5:], 'status': 'VERIFIED_REUSE_NOT_RERUN'}
assert checks['product-v16-just-ci']['test_pass_sum'] == 151
assert checks['product-v16-headless-final-lock']['test_pass_sum'] == 82
assert checks['product-v16-a11y-cache-access']['test_pass_sum'] == 8
hidden = (research / 'product-v16-root-hidden-cache-access.log').read_text()
assert len(re.findall('native destroy then four retained-context COM errors returned normally', hidden)) == 16

native = {}
for name in ['v16-core-main', 'v16-core-relaunch']:
    launch = read(research / 'native' / (name + '.launch.json'))
    exit_record = read(research / 'native' / (name + '.exit.json'))
    inventory = read(research / 'native' / (name + '-post-exit-windows.json'))
    stdout = (research / 'native' / (name + '.stdout.log')).read_text()
    stderr = (research / 'native' / (name + '.stderr.log')).read_text()
    assert launch['pid'] == exit_record['pid'] and exit_record['exit_code'] == 0
    assert launch['rust_backtrace'] == 'full' and not inventory['windows']
    assert datetime.fromisoformat(inventory['utc']) > datetime.fromisoformat(exit_record['utc'])
    for key, artifact in [('exe_sha256', 'binary'), ('dll_sha256', 'pdfium'), ('pdf_sha256', 'fixture')]:
        assert launch[key].lower() == freeze[artifact]['sha256'].lower()
    assert not stderr and not re.search(r'panic|\bERROR\b|failed|window not found|invalid window|cannot unwind', stdout, re.I)
    native[name] = {'launch': launch, 'exit': exit_record, 'post_exit_inventory': inventory,
                    'stdout_lines': len(stdout.splitlines()), 'stderr_bytes': len(stderr)}

old_lock = tomllib.loads(subprocess.check_output(['git', 'show', 'HEAD:Cargo.lock'], text=True))
new_lock = tomllib.loads((root / 'Cargo.lock').read_text())
accesskit = next(p for p in old_lock['package'] if p['name'] == 'accesskit_windows')
assert accesskit['version'] == '0.34.0'
accesskit.pop('source')
accesskit.pop('checksum')
assert old_lock == new_lock
diff = subprocess.run(['git', 'diff', '--check'], capture_output=True, text=True)
assert diff.returncode == 0
proc = subprocess.run(['powershell', '-NoProfile', '-Command',
    "$p = @(Get-Process | Where-Object { $_.ProcessName -match '^(ferrostage.*|cargo|rustc|clippy-driver|rustfmt|just|accesskit.*)$' } | Select-Object Id,ProcessName); ConvertTo-Json -InputObject $p"],
    capture_output=True, text=True)
assert proc.returncode == 0
processes = json.loads(proc.stdout)
result = {'captured_utc': datetime.now(timezone.utc).isoformat(),
          'branch': subprocess.check_output(['git', 'branch', '--show-current'], text=True).strip(),
          'head': subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip(),
          'source_count': len(freeze['source_files']), 'source_aggregate': aggregate,
          'freeze_sha256': sha(freeze_path), 'artifacts': {k: freeze[k] for k in ['binary', 'pdfium', 'fixture', 'cargo_lock']},
          'manifests': manifests, 'checks': checks, 'native': native,
          'lock_package_count': len(new_lock['package']), 'only_lock_change': 'same-version AccessKit registry source/checksum removal',
          'process_query': {'method': 'Get-Process', 'matches': processes,
                            'limit': 'Win32_Process CIM access denied; no command-line/parent inventory obtained'},
          'git_diff_check': diff.returncode, 'failures': failures,
          'status': 'PASS' if not failures else 'IDENTITY_MISMATCH',
          'native_or_build_executed': False}
with (research / 'resume-2026-10-15-v16-audit.json').open('x', encoding='utf-8') as stream:
    json.dump(result, stream, indent=2)
    stream.write('\n')
print(json.dumps({k: result[k] for k in ['status', 'source_count', 'source_aggregate', 'manifests', 'failures', 'process_query']}, indent=2))
