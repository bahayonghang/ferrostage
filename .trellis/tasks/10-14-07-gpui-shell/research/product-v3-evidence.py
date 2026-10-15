"""Reconstruct the retained v2 source and record the focused v3 delta."""
from pathlib import Path
import difflib
import hashlib
import json
import re

root = Path.cwd()
research = root / '.trellis/tasks/10-14-07-gpui-shell/research'
sha = lambda data: hashlib.sha256(data).hexdigest()
v2 = json.loads((research / 'product-native-freeze-v2.json').read_text(encoding='utf-8'))
v3 = json.loads((research / 'product-native-freeze-v3.json').read_text(encoding='utf-8'))
relative = 'crates/ferrostage-ui/src/shell.rs'
before = (research / 'startup-red-source/shell.rs').read_text(encoding='utf-8').splitlines(keepends=True)
patch = (research / 'product-startup-repair.patch').read_text(encoding='utf-8')
patch = patch.split('--- startup-red/shell.rs\n+++ startup-fixed/shell.rs\n', 1)[1]
lines = patch.splitlines(keepends=True)
output, cursor, index = [], 0, 0
while index < len(lines):
    match = re.match(r'@@ -(\d+)(?:,(\d+))? \+(\d+)(?:,(\d+))? @@', lines[index])
    assert match, lines[index]
    start = int(match[1]) - 1
    output.extend(before[cursor:start])
    cursor = start
    index += 1
    while index < len(lines) and not lines[index].startswith('@@ '):
        line = lines[index]
        if line.startswith((' ', '-')):
            assert before[cursor] == line[1:], (cursor, before[cursor], line)
            cursor += 1
        if line.startswith((' ', '+')):
            output.append(line[1:])
        index += 1
output.extend(before[cursor:])
text = ''.join(output)
candidates = [text.encode(), text.replace('\n', '\r\n').encode()]
source = next(data for data in candidates if sha(data) == v2['source_files'][relative])
(research / 'product-v3-before-shell.rs').write_bytes(source)
after = (root / relative).read_bytes()
assert sha(after) == v3['source_files'][relative]
(research / 'product-v3-after-shell.rs').write_bytes(after)
delta = ''.join(difflib.unified_diff(text.splitlines(keepends=True), after.decode().splitlines(keepends=True), fromfile='v2/shell.rs', tofile='v3/shell.rs'))
(research / 'product-v3-close-repair.patch').write_text(delta, encoding='utf-8', newline='\n')
changes = {path: {'v2': digest, 'v3': v3['source_files'].get(path)} for path, digest in v2['source_files'].items() if digest != v3['source_files'].get(path)}
assert set(changes) == {relative}, changes
record = {'v2_source_reconstructed_and_hash_verified': True, 'changes': changes, 'unchanged_source_count': len(v2['source_files']) - len(changes), 'new_or_removed_files': sorted(set(v2['source_files']) ^ set(v3['source_files'])), 'lock_unchanged': v2['cargo_lock']['sha256'] == v3['cargo_lock']['sha256'], 'focused_patch_sha256': sha((research / 'product-v3-close-repair.patch').read_bytes())}
(research / 'product-v3-source-delta.json').write_text(json.dumps(record, indent=2) + '\n', encoding='utf-8')
print(json.dumps(record, indent=2))
