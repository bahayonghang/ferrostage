"""Capture the focused v5-to-v6 repair; do not overwrite prior evidence."""
from pathlib import Path
import difflib
import hashlib
import json

root = Path.cwd()
research = root / '.trellis/tasks/10-14-07-gpui-shell/research'
sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
previous = json.loads((research / 'product-native-freeze-v5.json').read_text(encoding='utf-8'))
current = json.loads((research / 'product-native-freeze-v6.json').read_text(encoding='utf-8'))
owned = {'crates/ferrostage-ui/src/presenter.rs', 'crates/ferrostage-ui/src/shell.rs'}
patch = []
for path in sorted(owned):
    old = research / f'product-v6-before-{Path(path).name}'
    new = root / path
    assert sha(old) == previous['source_files'][path]
    assert sha(new) == current['source_files'][path]
    patch.extend(difflib.unified_diff(old.read_text(encoding='utf-8').splitlines(True),
                                    new.read_text(encoding='utf-8').splitlines(True),
                                    fromfile='v5/' + path, tofile='v6/' + path))
patch_path = research / 'product-v6-repair.patch'
with patch_path.open('x', encoding='utf-8', newline='\n') as output:
    output.write(''.join(patch))
changes = {path: {'v5': digest, 'v6': current['source_files'].get(path)}
           for path, digest in previous['source_files'].items()
           if digest != current['source_files'].get(path)}
assert set(changes) == owned, changes
assert previous['source_files'].keys() == current['source_files'].keys()
assert all(sha(root / path) == digest for path, digest in current['source_files'].items())
record = {'source_changes': changes,
          'unchanged_count': len(previous['source_files']) - len(changes),
          'new_or_removed': [],
          'lock_unchanged': previous['cargo_lock']['sha256'] == current['cargo_lock']['sha256'],
          'all_core_vendor_inputs_unchanged': all(previous['source_files'][path] == digest
              for path, digest in current['source_files'].items()
              if path.startswith(('crates/ferrostage-core/', 'vendor/'))),
          'patch_sha256': sha(patch_path)}
with (research / 'product-v6-source-delta.json').open('x', encoding='utf-8') as output:
    output.write(json.dumps(record, indent=2) + '\n')
print(json.dumps(record, indent=2))
