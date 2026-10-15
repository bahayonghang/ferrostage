"""Read-only identity/evidence audit; output only a new independent receipt."""
from pathlib import Path
from datetime import datetime, timezone
import difflib
import hashlib
import json
import sys

root = Path.cwd()
research = Path(__file__).resolve().parent
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
read = lambda p: json.loads(p.read_text(encoding='utf-8-sig'))
freeze_path = research / 'product-native-freeze-v15.json'
freeze = read(freeze_path)
suffix = '-' + sys.argv[1] if len(sys.argv) > 1 else ''
output = research / f'product-independent-v15-identity{suffix}.json'
assert not output.exists()
mismatches = []
sources = {p: sha(root / p) for p in freeze['source_files']}
mismatches.extend(p for p, digest in freeze['source_files'].items() if sources[p] != digest)
aggregate = hashlib.sha256(json.dumps(sources, sort_keys=True, separators=(',', ':')).encode()).hexdigest()
assert aggregate == freeze['source_snapshot_sha256']

def check_record(value):
    if isinstance(value, dict):
        if 'path' in value and 'sha256' in value:
            p = Path(value['path'])
            if sha(p) != value['sha256'] or p.stat().st_size != value['bytes']:
                mismatches.append(str(p))
        else:
            for v in value.values():
                check_record(v)

check_record(freeze)
manifests = {}
for name in ['product-v15-evidence-manifest.json', 'native-v14-evidence-manifest.json', 'product-independent-v14-manifest.json']:
    manifest = read(research / name)
    entries = manifest['files']
    entries = entries.items() if isinstance(entries, dict) else ((record['path'], record) for record in entries)
    for filename, record in entries:
        p = root / filename
        if not p.exists():
            p = research / filename
        digest = record if isinstance(record, str) else record['sha256']
        if sha(p) != digest or (isinstance(record, dict) and p.stat().st_size != record['bytes']):
            mismatches.append(str(p))
    manifests[name] = {'count': len(manifest['files']), 'sha256': sha(research / name)}

previous = read(research / 'product-native-freeze-v14.json')['source_files']
delta = read(research / 'product-v15-source-delta.json')
actual_changes = {p: {'v14': previous[p], 'v15': digest} for p, digest in sources.items() if p in previous and previous[p] != digest}
actual_added = {p: digest for p, digest in sources.items() if p not in previous}
assert actual_changes == delta['changes'] and actual_added == delta['added']
assert all(sources[p] == digest for p, digest in previous.items() if p.startswith('vendor/') or p.endswith('Cargo.toml') or p == 'Cargo.lock')

upstream_record = read(research / 'product-v15-a11y-upstream.json')
upstream = Path(upstream_record['path'])
candidate = research / 'product-v15-accesskit-candidate'
for p, digest in upstream_record['files'].items():
    assert sha(upstream / p) == digest, p
patch = ''
candidate_files = {}
for name in ['src/util.rs', 'src/context.rs', 'src/node.rs', 'src/text.rs']:
    patch += ''.join(difflib.unified_diff((upstream / name).read_text(encoding='utf-8').splitlines(True), (candidate / name).read_text(encoding='utf-8').splitlines(True), fromfile='a/' + name, tofile='b/' + name))
    candidate_files[name] = sha(candidate / name)
assert patch == (research / 'product-v15-accesskit-proposal.patch').read_text(encoding='utf-8')
assert sha(candidate / 'src/v15_research_tests.rs') == sha(research / 'product-v15-a11y-tests.rs')
root_hidden = read(research / 'product-v15-a11y-root-hidden.json')
assert root_hidden['status'] == 'RETURNED' and root_hidden['exit_code'] == 0
root_hidden_log = (research / 'product-v15-a11y-root-hidden.log').read_text(encoding='utf-8')
assert root_hidden_log.count('native destroy then four retained-context COM errors returned normally') == 16
assert not mismatches, mismatches
record = {
    'at_utc': datetime.now(timezone.utc).isoformat(), 'status': 'PASS',
    'source_count': len(sources), 'source_sha256': aggregate,
    'binary_sha256': sha(Path(freeze['binary']['path'])), 'freeze_sha256': sha(freeze_path),
    'fixture_sha256': sha(root / 'fixtures/notes-functional/pympress-notes.pdf'),
    'delta': {'changed': len(actual_changes), 'added': len(actual_added), 'unchanged': len(previous) - len(actual_changes)},
    'product_dependencies_and_vendor_unchanged_from_v14': True,
    'manifests': manifests, 'candidate_patch_sha256': sha(research / 'product-v15-accesskit-proposal.patch'),
    'candidate_files': candidate_files, 'candidate_harness_sha256': sha(candidate / 'src/v15_research_tests.rs'),
    'root_hidden': {'receipt_sha256': sha(research / 'product-v15-a11y-root-hidden.json'), 'log_sha256': sha(research / 'product-v15-a11y-root-hidden.log'), 'cycles': 16, 'com_calls': 64, 'scope': 'after synchronized destruction; not concurrent/handle reuse/product acceptance'},
    'mismatches': mismatches,
}
with output.open('x', encoding='utf-8') as stream:
    json.dump(record, stream, indent=2)
    stream.write('\n')
print(json.dumps(record, indent=2))
