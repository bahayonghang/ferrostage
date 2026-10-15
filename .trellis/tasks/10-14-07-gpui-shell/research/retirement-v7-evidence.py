"""Read retained V6 errors and source identities; never replace an earlier index."""
from datetime import datetime, timezone
from pathlib import Path
import hashlib
import json

root = Path.cwd()
research = Path(__file__).resolve().parent
upstream = Path.home() / '.cargo/git/checkouts/zed-a70e2ad075855582/96a1b1b/crates'
sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
sources = [root / name for name in [
    'Cargo.lock', 'vendor/gpui/src/app.rs', 'vendor/gpui/src/window.rs',
    'vendor/gpui/src/platform/test/platform.rs', 'vendor/gpui/src/app/test_context.rs',
]] + [upstream / name for name in [
    'gpui_util/src/lib.rs', 'gpui_windows/src/window.rs', 'gpui_windows/src/events.rs',
    'gpui_windows/src/dispatcher.rs', 'gpui_windows/src/platform.rs',
]] + [research / f'product-v7-before-{name}.rs' for name in ['main', 'shell']]
logs = {}
for tag in ['v6-focus-save', 'v6-relaunch', 'resume-v6-light', 'resume-v6-dark']:
    path = research / 'native' / f'{tag}.stdout.log'
    lines = path.read_text(encoding='utf-8').splitlines()
    logs[path.name] = {
        'sha256': sha(path),
        'errors': [{'line': index, 'text': line} for index, line in enumerate(lines, 1)
                   if 'ERROR' in line],
        'exit_receipt': json.loads(path.with_name(f'{tag}.exit.json').read_text(encoding='utf-8')),
        'exit_receipt_sha256': sha(path.with_name(f'{tag}.exit.json')),
    }
record = {
    'captured_utc': datetime.now(timezone.utc).isoformat(),
    'scope': 'Frozen V6 diagnosis, source read only; no newly executed product regression',
    'logs': logs,
    'sources': {str(path): sha(path) for path in sources},
    'unknown_exit_trigger': 'resume-v6-light; exit0 is not a normal-close PASS',
    'external_git_check': 'NOT_VERIFIED: git -C external checkout failed permission denied',
}
with (research / 'retirement-v7-evidence.json').open('x', encoding='utf-8') as output:
    output.write(json.dumps(record, indent=2) + '\n')
print(json.dumps({'logs': len(logs), 'sources': len(sources),
                  'error_counts': {name: len(item['errors']) for name, item in logs.items()}}, indent=2))
