"""Read current task locations and validate context without changing task status."""
from datetime import datetime, timezone
from pathlib import Path
import hashlib
import json
import subprocess
import sys

root = Path.cwd()
parent = root / '.trellis/tasks/10-14-presenter-v0.3'
research = parent / 'research'
result = {'started_utc': datetime.now(timezone.utc).isoformat(), 'status': 'RUNNING'}
receipt = research / 'resume-2026-10-15-context-audit.json'
if receipt.exists():
    raise SystemExit('Preserve the existing receipt; use a new audit name.')
receipt.write_text(json.dumps(result, indent=2) + '\n', encoding='utf-8')


def run(args):
    completed = subprocess.run(args, cwd=root, capture_output=True,
                               encoding='utf-8', errors='replace')
    return {'command': args, 'exit_code': completed.returncode,
            'stdout': completed.stdout, 'stderr': completed.stderr}


result['branch'] = run(['git', 'branch', '--show-current'])
result['head'] = run(['git', 'rev-parse', 'HEAD'])
result['current'] = run([sys.executable, '.trellis/scripts/task.py', 'current'])
result['worktree'] = run(['git', 'status', '--short'])
result['diff_stat'] = run(['git', 'diff', '--stat'])
parent_record = json.loads((parent / 'task.json').read_text(encoding='utf-8'))
paths = list((root / '.trellis/tasks').rglob('task.json'))
tasks = [parent]
for name in parent_record['children']:
    matches = [p.parent for p in paths if p.parent.name == name]
    assert len(matches) == 1, (name, matches)
    tasks.extend(matches)
assert len(tasks) == 12
result['tasks'] = []
for task in tasks:
    data = json.loads((task / 'task.json').read_text(encoding='utf-8'))
    if task != parent:
        assert data['parent'] == parent.name
    artifacts = {}
    for name in ['task.json', 'prd.md', 'design.md', 'implement.md',
                 'implement.jsonl', 'check.jsonl']:
        path = task / name
        if path.exists():
            raw = path.read_bytes()
            content = raw.decode('utf-8')
            artifacts[name] = {'sha256': hashlib.sha256(raw).hexdigest(),
                               'bytes': len(raw), 'checked': content.count('[x]'),
                               'unchecked': content.count('[ ]')}
        else:
            artifacts[name] = {'absent': True}
    assert not artifacts['prd.md'].get('absent')
    if task.name != '10-14-01-docs-sync':
        assert not artifacts['design.md'].get('absent')
        assert not artifacts['implement.md'].get('absent')
    relative = task.relative_to(root).as_posix()
    validation = run([sys.executable, '.trellis/scripts/task.py', 'validate', relative])
    result['tasks'].append({'path': relative, 'id': data['id'],
                            'status': data['status'], 'artifacts': artifacts,
                            'validation': validation})
result['diff_check'] = run(['git', 'diff', '--check'])
result['finished_utc'] = datetime.now(timezone.utc).isoformat()
result['status'] = ('PASS' if all(t['validation']['exit_code'] == 0 for t in result['tasks'])
                    and result['diff_check']['exit_code'] == 0 else 'FAIL')
result['boundary'] = 'Context validity and diff whitespace only; not product/native acceptance.'
receipt.write_text(json.dumps(result, indent=2, ensure_ascii=False) + '\n', encoding='utf-8')
print(json.dumps({'status': result['status'], 'tasks': len(tasks),
                  'validation_pass': sum(t['validation']['exit_code'] == 0 for t in result['tasks']),
                  'diff_exit': result['diff_check']['exit_code'],
                  'receipt': receipt.relative_to(root).as_posix()}))
raise SystemExit(0 if result['status'] == 'PASS' else 1)
