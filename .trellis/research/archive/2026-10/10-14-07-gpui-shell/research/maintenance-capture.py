"""Retain reproduction inputs and hash the final isolated evaluation evidence."""
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import shutil
import subprocess

ROOT = Path.cwd()
RESEARCH = ROOT / '.trellis/tasks/10-14-07-gpui-shell/research'
CANDIDATE = ROOT / 'cache/gpui-maintenance-candidate'
RETAINED = RESEARCH / 'maintenance-retained'


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def record(path):
    try:
        name = path.relative_to(ROOT).as_posix()
    except ValueError:
        name = str(path)
    return {'path': name, 'bytes': path.stat().st_size, 'sha256': sha(path)}


for relative in ['Cargo.toml', 'Cargo.lock', 'src/main.rs',
                 'vendor/gpui/Cargo.toml', 'vendor/gpui/Cargo.lock',
                 'vendor/pulp/Cargo.toml', 'vendor/pulp/Cargo.lock',
                 'vendor/metal/Cargo.toml',
                 'vendor/cosmic-text/Cargo.toml', 'vendor/cosmic-text/Cargo.lock',
                 'validation/fontdb/Cargo.toml', 'validation/fontdb/Cargo.lock',
                 'validation/wgpu-consumer/Cargo.toml', 'validation/wgpu-consumer/Cargo.lock',
                 'validation/wgpu-consumer/src/main.rs']:
    target = RETAINED / relative
    target.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(CANDIDATE / relative, target)

baseline_path = ROOT / '.trellis/tasks/10-14-06-beamer-document-model/research/independent-check-manifest.json'
baseline = json.loads(baseline_path.read_text(encoding='utf-8-sig'))
prior_comparison = json.loads((RESEARCH / 'product-input-comparison.json').read_text(encoding='utf-8-sig'))
product_paths = {item['path'] for item in prior_comparison['files'] if item['product_input']}
comparison = []
for name, expected in baseline['source_files'].items():
    path = ROOT / name
    actual = sha(path) if path.is_file() else None
    comparison.append({'path': name, 'product_input': name in product_paths,
                       'baseline_sha256': expected, 'current_sha256': actual,
                       'result': 'UNCHANGED' if actual == expected else 'DIFFERENT_OR_MISSING'})
unlisted_crates = sorted(path.relative_to(ROOT).as_posix() for path in (ROOT / 'crates').rglob('*')
                        if path.is_file() and path.relative_to(ROOT).as_posix() not in baseline['source_files'])
product_result = {'captured_utc': datetime.now(timezone.utc).isoformat(), 'baseline': record(baseline_path),
                  'baseline_head': baseline['head'], 'total_files': len(comparison),
                  'unchanged': sum(item['result'] == 'UNCHANGED' for item in comparison),
                  'product_input_count': len(product_paths),
                  'product_inputs_unchanged': sum(item['product_input'] and item['result'] == 'UNCHANGED' for item in comparison),
                  'unlisted_crate_files': unlisted_crates, 'files': comparison}
(RESEARCH / 'maintenance-product-input-comparison.json').write_text(json.dumps(product_result, indent=2) + '\n', encoding='utf-8')
assert product_result['unchanged'] == len(comparison) and not unlisted_crates

candidate_files = []
for relative in ['vendor', 'validation', 'src', 'tests', 'assets', 'maintenance-fixtures']:
    candidate_files.extend(path for path in (CANDIDATE / relative).rglob('*') if path.is_file())
candidate_files.extend([CANDIDATE / 'Cargo.toml', CANDIDATE / 'Cargo.lock'])
candidate_records = [record(path) for path in sorted(set(candidate_files))]
candidate_hash = hashlib.sha256(json.dumps(candidate_records, sort_keys=True, separators=(',', ':')).encode()).hexdigest()
research_files = []
for path in RESEARCH.glob('maintenance-*'):
    if path.name == 'maintenance-evidence-manifest.json':
        continue
    research_files.extend([path] if path.is_file() else [item for item in path.rglob('*') if item.is_file()])
artifacts = list((CANDIDATE / 'artifacts').glob('*'))
binary = CANDIDATE / 'target/debug/ferrostage-gpui-maintenance-candidate.exe'
if binary.exists():
    artifacts.append(binary)
manifest = {'captured_utc': datetime.now(timezone.utc).isoformat(),
            'scope': 'Isolated maintenance candidate only; no product pin, policy, UI or feature change.',
            'candidate': str(CANDIDATE), 'candidate_source_snapshot_sha256': candidate_hash,
            'candidate_source_files': candidate_records,
            'maintenance_evidence_files': [record(path) for path in sorted(research_files)],
            'generated_cache_artifacts': [record(path) for path in sorted(artifacts) if path.is_file()],
            'rustc': subprocess.check_output(['rustc', '-Vv'], text=True).strip(),
            'cargo': subprocess.check_output(['cargo', '-V'], text=True).strip(),
            'policy': record(ROOT / 'deny.toml'),
            'product_comparison': record(RESEARCH / 'maintenance-product-input-comparison.json')}
(RESEARCH / 'maintenance-evidence-manifest.json').write_text(json.dumps(manifest, indent=2) + '\n', encoding='utf-8')
print(json.dumps({'source_files': len(candidate_records), 'evidence_files': len(research_files),
                  'source_snapshot_sha256': candidate_hash,
                  'product_baseline_unchanged': product_result['unchanged'],
                  'product_inputs_unchanged': product_result['product_inputs_unchanged']}))
