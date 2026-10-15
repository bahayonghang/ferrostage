"""Reproduce the frozen handoff without modifying any original candidate/evidence."""
import hashlib
import json
from pathlib import Path
import shutil
import subprocess

ROOT = Path.cwd()
RESEARCH = ROOT / '.trellis/tasks/10-14-07-gpui-shell/research'
OUT = RESEARCH / 'maintenance-independent-reproduction'
DEST = ROOT / 'cache/gpui-maintenance-independent-reproduction'
OUT.mkdir(exist_ok=True)
for pattern in ('maintenance-svg-tests.rs', 'maintenance-regression-*.rs'):
    for file in RESEARCH.glob(pattern):
        shutil.copy2(file, OUT / file.name)
shutil.copytree(RESEARCH / 'maintenance-retained', OUT / 'maintenance-retained', dirs_exist_ok=True)
shutil.copytree(RESEARCH / 'maintenance-fixtures', OUT / 'maintenance-fixtures', dirs_exist_ok=True)
script = (RESEARCH / 'maintenance-prepare.py').read_text(encoding='utf-8')
script = script.replace("RESEARCH = ROOT / '.trellis/tasks/10-14-07-gpui-shell/research'",
                        "RESEARCH = ROOT / '.trellis/tasks/10-14-07-gpui-shell/research/maintenance-independent-reproduction'")
script = script.replace("DEST = ROOT / 'cache/gpui-maintenance-candidate'",
                        "DEST = ROOT / 'cache/gpui-maintenance-independent-reproduction'")
adapted = OUT / 'maintenance-prepare.py'
adapted.write_text(script, encoding='utf-8', newline='\n')
run = subprocess.run(['python', '-X', 'utf8', str(adapted)], cwd=ROOT, capture_output=True, text=True)
(OUT / 'prepare.log').write_text(run.stdout + run.stderr, encoding='utf-8')
sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest() if path.is_file() else None
manifest = json.loads((RESEARCH / 'maintenance-evidence-manifest.json').read_text(encoding='utf-8'))
records = []
for original in manifest['candidate_source_files']:
    rel = Path(original['path']).relative_to('cache/gpui-maintenance-candidate')
    records.append({'path': rel.as_posix(), 'expected': original['sha256'],
                    'actual': sha(DEST / rel), 'matches': sha(DEST / rel) == original['sha256']})
result = {'prepare_exit': run.returncode, 'patch_matches': sha(OUT / 'maintenance-source.patch') == sha(RESEARCH / 'maintenance-source.patch'),
          'original_patch_sha256': sha(RESEARCH / 'maintenance-source.patch'),
          'reproduced_patch_sha256': sha(OUT / 'maintenance-source.patch'),
          'records': records, 'mismatches': [r for r in records if not r['matches']]}
(RESEARCH / 'maintenance-independent-reproduction.json').write_text(json.dumps(result, indent=2) + '\n', encoding='utf-8')
print(json.dumps({k:v for k,v in result.items() if k != 'records'}))
raise SystemExit(0 if run.returncode == 0 and result['patch_matches'] and not result['mismatches'] else 1)
