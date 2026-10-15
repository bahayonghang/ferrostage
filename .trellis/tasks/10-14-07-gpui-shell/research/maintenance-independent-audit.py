"""Read-only independent audit of the frozen maintenance handoff and product baseline."""
import hashlib
import json
from pathlib import Path
import sys
import tarfile
import tomllib

ROOT = Path(__file__).resolve().parents[4]
RESEARCH = Path(__file__).resolve().parent
phase = sys.argv[1] if len(sys.argv) > 1 else "initial"
assert phase in ("initial", "final")


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest() if path.is_file() else None


manifest_path = RESEARCH / "maintenance-evidence-manifest.json"
manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
records = []
for group in ("candidate_source_files", "maintenance_evidence_files", "generated_cache_artifacts"):
    for item in manifest[group]:
        path = ROOT / item["path"]
        records.append({**item, "group": group, "actual_sha256": sha(path),
                        "matches": sha(path) == item["sha256"] and path.stat().st_size == item["bytes"]})
candidate = ROOT / "cache/gpui-maintenance-candidate"
candidate_paths = {p.relative_to(ROOT).as_posix() for folder in
                   ("vendor", "validation", "src", "tests", "assets", "maintenance-fixtures")
                   for p in (candidate / folder).rglob("*") if p.is_file()}
candidate_paths |= {(candidate / n).relative_to(ROOT).as_posix() for n in ("Cargo.toml", "Cargo.lock")}
expected_paths = {item["path"] for item in manifest["candidate_source_files"]}
validation = json.loads((RESEARCH / "maintenance-source-validation.json").read_text(encoding="utf-8"))
archives = []
for item in validation["registry_inputs"]:
    archive = Path(item["archive"])
    label = f'{item["package"]}-{item["version"]}'
    checksum_lock = ROOT / item["checksum_lock"]
    if checksum_lock.is_relative_to(candidate):
        checksum_lock = RESEARCH / 'maintenance-retained' / checksum_lock.relative_to(candidate)
    packages = tomllib.loads(checksum_lock.read_text(encoding="utf-8"))["package"]
    expected = next(p["checksum"] for p in packages
                    if p["name"] == item["package"] and p["version"] == item["version"] and "checksum" in p)
    members = {}
    differences = []
    with tarfile.open(archive, "r:gz") as tar:
        for entry in tar.getmembers():
            if entry.isfile():
                rel = Path(entry.name).relative_to(label).as_posix()
                digest = hashlib.sha256(tar.extractfile(entry).read()).hexdigest()
                members[rel] = digest
                if sha(Path(item["source"]) / rel) != digest:
                    differences.append(rel)
    archives.append({"package": label, "archive_sha256": sha(archive), "expected_sha256": expected,
                     "archive_matches": sha(archive) == expected, "file_count": len(members),
                     "cached_file_mismatches": differences,
                     "recorded_members_match": members == item["archive_file_hashes"]})
baseline_path = ROOT / ".trellis/tasks/10-14-06-beamer-document-model/research/independent-check-manifest.json"
baseline = json.loads(baseline_path.read_text(encoding="utf-8"))
product = {p: {"expected": digest, "actual": sha(ROOT / p)} for p, digest in baseline["source_files"].items()}
prior = json.loads((RESEARCH / "product-input-comparison.json").read_text(encoding="utf-8"))
product_paths = {item["path"] for item in prior["files"] if item["product_input"]}
crate_paths = {p.relative_to(ROOT).as_posix() for p in (ROOT / "crates").rglob("*") if p.is_file()}
result = {
    "handoff_manifest_sha256": sha(manifest_path),
    "handoff_patch_sha256": sha(RESEARCH / "maintenance-source.patch"),
    "records": records, "record_mismatches": [r for r in records if not r["matches"]],
    "candidate_extra_paths": sorted(candidate_paths - expected_paths),
    "candidate_missing_paths": sorted(expected_paths - candidate_paths),
    "registry_archives": archives, "archive_file_count": sum(a["file_count"] for a in archives),
    "baseline_manifest_sha256": sha(baseline_path), "baseline_files": product,
    "baseline_mismatches": [p for p, h in product.items() if h["actual"] != h["expected"]],
    "classified_product_count": len(product_paths),
    "classified_product_unchanged": sum(product[p]["actual"] == product[p]["expected"] for p in product_paths),
    "crate_file_count": len(crate_paths), "unlisted_crate_files": sorted(crate_paths - set(product)),
    "reviewable_font_binaries": [p.relative_to(ROOT).as_posix() for p in RESEARCH.rglob("*")
                                if p.suffix.lower() in {".ttf", ".otf", ".ttc"}],
}
if phase == 'final':
    final_snapshot = json.loads((RESEARCH / 'maintenance-independent-final-source.json').read_text())['files']
    result['final_candidate_mismatches'] = [p for p, digest in final_snapshot.items() if sha(candidate / p) != digest]
    result['final_candidate_count'] = len(final_snapshot)
    result['frozen_maintenance_evidence_mismatches'] = [r for r in records if r['group'] == 'maintenance_evidence_files' and not r['matches']]
    result['baseline_evidence_mismatches'] = [p for section in ('evidence_files','verified_external_inputs') for p,digest in baseline[section].items() if sha(ROOT/p) != digest]
    def old_records(value):
        if isinstance(value, dict):
            if 'path' in value and 'sha256' in value:
                yield value
            else:
                for child in value.values(): yield from old_records(child)
        elif isinstance(value, list):
            for child in value: yield from old_records(child)
    old = list(old_records(json.loads((RESEARCH/'evidence-manifest.json').read_text())))
    preserved = {str(RESEARCH/'gpui-display-api.md'):RESEARCH/'independent-original-gpui-display-api.md',
                 str(RESEARCH.parent/'implement.md'):RESEARCH/'independent-original-implement.md'}
    checked=[]
    for item in old:
        path=ROOT/Path(item['path'])
        retained=preserved.get(str(path))
        checked.append({'path':item['path'],'matches_current':sha(path)==item['sha256'],
                        'matches_preserved_original': bool(retained and sha(retained)==item['sha256'])})
    result['original_spike_records']=checked
    result['original_spike_unexpected_mismatches']=[x for x in checked if not x['matches_current'] and not x['matches_preserved_original']]
out = RESEARCH / f"maintenance-independent-{phase}-audit.json"
out.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
print(json.dumps({k: result[k] for k in ("handoff_manifest_sha256", "handoff_patch_sha256", "archive_file_count",
                 "baseline_mismatches", "classified_product_count", "classified_product_unchanged",
                 "crate_file_count", "unlisted_crate_files", "reviewable_font_binaries")}, indent=2))
if phase == 'final':
    checks=['final_candidate_mismatches','frozen_maintenance_evidence_mismatches','baseline_evidence_mismatches','original_spike_unexpected_mismatches']
    print(json.dumps({key:result[key] for key in checks}))
    assert not any(result[key] for key in checks)
assert not result['baseline_mismatches'] and not result['unlisted_crate_files']
assert all(x['archive_matches'] and not x['cached_file_mismatches'] and x['recorded_members_match'] for x in archives)
