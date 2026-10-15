"""Record the source-bound v4 action repair without altering earlier receipts."""
from pathlib import Path
import difflib
import hashlib
import json
import re
import tomllib

root = Path.cwd()
r = root / '.trellis/tasks/10-14-07-gpui-shell/research'
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
v3 = json.loads((r / 'product-native-freeze-v3.json').read_text(encoding='utf-8'))
v4 = json.loads((r / 'product-native-freeze-v4.json').read_text(encoding='utf-8'))
before = {
    'crates/ferrostage-ui/src/actions.rs': 'product-v4-before-actions.rs',
    'crates/ferrostage-ui/src/shell.rs': 'product-v3-after-shell.rs',
    'crates/ferrostage-ui/src/presenter.rs': 'product-v4-before-presenter.rs',
    'crates/ferrostage-ui/src/theme.rs': 'product-v4-before-theme.rs',
    'assets/locales/en-US/app.toml': 'product-v4-before-en-US.toml',
    'assets/locales/zh-CN/app.toml': 'product-v4-before-zh-CN.toml',
}
patch = []
for path, old in before.items():
    assert sha(r / old) == v3['source_files'][path], path
    assert sha(root / path) == v4['source_files'][path], path
    patch.extend(difflib.unified_diff((r / old).read_text(encoding='utf-8').splitlines(True), (root / path).read_text(encoding='utf-8').splitlines(True), fromfile='v3/' + path, tofile='v4/' + path))
target = r / 'product-v4-input-repair.patch'
target.write_text(''.join(patch), encoding='utf-8', newline='\n')
changes = {path: {'v3': digest, 'v4': v4['source_files'].get(path)} for path, digest in v3['source_files'].items() if digest != v4['source_files'].get(path)}
assert set(changes) <= set(before) | {'DESIGN.md'}, changes
delta = {'source_changes': changes, 'unchanged_count': len(v3['source_files']) - len(changes), 'implementer_changes': sorted(before), 'root_owned_document_changes': sorted(set(changes) - set(before)), 'lock_unchanged': v3['cargo_lock']['sha256'] == v4['cargo_lock']['sha256'], 'patch_sha256': sha(target)}
(r / 'product-v4-source-delta.json').write_text(json.dumps(delta, indent=2) + '\n', encoding='utf-8')
locales = {lang: tomllib.loads((root / 'assets/locales' / lang / 'app.toml').read_text(encoding='utf-8')) for lang in ['en-US', 'zh-CN']}
used = set()
for path in (root / 'crates/ferrostage-ui/src').glob('*.rs'):
    used.update(re.findall(r'\.(?:text|format|error)\("([a-z][a-z-]+)"', path.read_text(encoding='utf-8')))
assert used <= locales['en-US'].keys()
assert locales['en-US'].keys() == locales['zh-CN'].keys()
(r / 'product-v4-locale-contract.json').write_text(json.dumps({'catalog_keys': len(locales['en-US']), 'direct_call_keys': sorted(used), 'catalog_sha256': {lang: sha(root / 'assets/locales' / lang / 'app.toml') for lang in locales}, 'status': 'PASS; actual default unsupported warning output verified in both languages by GPUI test'}, indent=2) + '\n', encoding='utf-8')
print(json.dumps(delta, indent=2))
