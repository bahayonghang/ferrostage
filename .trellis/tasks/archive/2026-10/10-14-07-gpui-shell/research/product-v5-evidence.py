"""Capture the focused v4-to-v5 runtime/test delta, retaining prior freezes."""
from pathlib import Path
import difflib
import hashlib
import json

root = Path.cwd()
r = root / '.trellis/tasks/10-14-07-gpui-shell/research'
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
previous = json.loads((r / 'product-native-freeze-v4.json').read_text(encoding='utf-8'))
current = json.loads((r / 'product-native-freeze-v5.json').read_text(encoding='utf-8'))
patch = []
owned = set()
for name in ['actions', 'presenter', 'shell']:
    path = f'crates/ferrostage-ui/src/{name}.rs'
    owned.add(path)
    old = r / f'product-v5-before-{name}.rs'
    new = root / path
    assert sha(old) == previous['source_files'][path]
    assert sha(new) == current['source_files'][path]
    patch.extend(difflib.unified_diff(old.read_text(encoding='utf-8').splitlines(True), new.read_text(encoding='utf-8').splitlines(True), fromfile='v4/' + path, tofile='v5/' + path))
target = r / 'product-v5-repair.patch'
target.write_text(''.join(patch), encoding='utf-8', newline='\n')
changes = {path: {'v4': digest, 'v5': current['source_files'].get(path)} for path, digest in previous['source_files'].items() if digest != current['source_files'].get(path)}
assert set(changes) <= owned | {'DESIGN.md'}, changes
record = {'source_changes': changes, 'unchanged_count': len(previous['source_files']) - len(changes), 'new_or_removed': sorted(set(previous['source_files']) ^ set(current['source_files'])), 'lock_unchanged': previous['cargo_lock']['sha256'] == current['cargo_lock']['sha256'], 'all_core_vendor_inputs_unchanged': all(previous['source_files'][path] == digest for path, digest in current['source_files'].items() if path.startswith(('crates/ferrostage-core/', 'vendor/'))), 'patch_sha256': sha(target)}
(r / 'product-v5-source-delta.json').write_text(json.dumps(record, indent=2) + '\n', encoding='utf-8')
print(json.dumps(record, indent=2))
