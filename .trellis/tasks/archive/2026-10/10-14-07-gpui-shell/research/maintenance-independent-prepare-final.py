"""Authoritative reproduction entry for the reviewed candidate, not product adoption.

Default output is a fresh isolated reproduction. No shared sources or original
handoff evidence are edited. Requires the recorded local Cargo cache inputs.
"""
from pathlib import Path
import hashlib
import json
import shutil
import subprocess
import sys

ROOT = Path.cwd()
R = ROOT / '.trellis/tasks/10-14-07-gpui-shell/research'
OUT = R / 'maintenance-independent-final-preparation'
DEST = ROOT / (sys.argv[1] if len(sys.argv) > 1 else 'cache/gpui-maintenance-independent-final-reproduction')
assert DEST.resolve().is_relative_to((ROOT / 'cache').resolve()) and DEST.name.startswith('gpui-maintenance-')
assert DEST.name not in ('gpui-maintenance-candidate', 'gpui-maintenance-cmap-proof'), 'Do not overwrite the reviewed/tested candidate or probe.'
OUT.mkdir(exist_ok=True)
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest() if p.is_file() else None
assert sha(R / 'maintenance-evidence-manifest.json') == 'a0316c9af7a46dfd4f8bb331520c1fb3d4938b1468b02c27de77b2b470e81b0f'
assert sha(R / 'maintenance-source.patch') == 'd907642badb418203c3ca97472d2acd464fd4025e9b756b7c3f4fef0d98f9440'
for pattern in ('maintenance-svg-tests.rs', 'maintenance-regression-*.rs'):
    for file in R.glob(pattern):
        shutil.copy2(file, OUT / file.name)
for folder in ('maintenance-retained', 'maintenance-fixtures'):
    shutil.copytree(R / folder, OUT / folder, dirs_exist_ok=True)
prepare = (R / 'maintenance-prepare.py').read_text()
prepare = prepare.replace("RESEARCH = ROOT / '.trellis/tasks/10-14-07-gpui-shell/research'", f"RESEARCH = ROOT / {OUT.relative_to(ROOT).as_posix()!r}")
prepare = prepare.replace("DEST = ROOT / 'cache/gpui-maintenance-candidate'", f"DEST = ROOT / {DEST.relative_to(ROOT).as_posix()!r}")
(OUT / 'maintenance-prepare.py').write_text(prepare, encoding='utf-8', newline='\n')
commands = []
def run(script, *args):
    command = ['python', '-X', 'utf8', str(script), *args]
    result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True)
    commands.append({'command': command, 'exit': result.returncode, 'output': result.stdout + result.stderr})
    (OUT / 'commands.json').write_text(json.dumps(commands, indent=2) + '\n', encoding='utf-8')
    result.check_returncode()
run(OUT / 'maintenance-prepare.py')
handoff = json.loads((R / 'maintenance-evidence-manifest.json').read_text())
assert all(sha(DEST / Path(x['path']).relative_to('cache/gpui-maintenance-candidate')) == x['sha256'] for x in handoff['candidate_source_files'])
run(R / 'maintenance-independent-repair.py', DEST.relative_to(ROOT).as_posix(), OUT.relative_to(ROOT).as_posix())
run(R / 'maintenance-independent-harfrust-repair.py', DEST.relative_to(ROOT).as_posix(), OUT.relative_to(ROOT).as_posix())
for lock in (R / 'maintenance-independent-retained/final-locks').rglob('Cargo.lock'):
    target = DEST / lock.relative_to(R / 'maintenance-independent-retained/final-locks')
    target.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(lock, target)
expected = json.loads((R / 'maintenance-independent-final-source.json').read_text())['files']
observed = {p.relative_to(DEST).as_posix(): sha(p) for folder in ('vendor', 'validation', 'src', 'tests', 'assets', 'maintenance-fixtures')
            for p in (DEST / folder).rglob('*') if p.is_file()}
observed |= {name: sha(DEST / name) for name in ('Cargo.toml', 'Cargo.lock')}
diff = {name: {'expected': expected.get(name), 'actual': observed.get(name)} for name in expected.keys() | observed.keys() if expected.get(name) != observed.get(name)}
result = {'candidate': str(DEST), 'base_files_verified': len(handoff['candidate_source_files']), 'final_files_verified': len(expected),
          'commands': commands, 'differences': diff, 'matches': not diff}
(R / 'maintenance-independent-final-reproduction.json').write_text(json.dumps(result, indent=2) + '\n', encoding='utf-8')
print(json.dumps({'base_files_verified':result['base_files_verified'], 'final_files_verified':len(expected), 'matches':not diff, 'differences':diff}))
raise SystemExit(1 if diff else 0)
