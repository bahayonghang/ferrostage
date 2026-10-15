"""Recompute task06 applicability and task07 evidence without editing either manifest."""

import hashlib
import json
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
RESEARCH = Path(__file__).resolve().parent


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest() if path.is_file() else None


def compare(path, expected, category, expected_bytes=None):
    file = Path(path)
    if not file.is_absolute():
        file = ROOT / file
    actual = digest(file)
    size = file.stat().st_size if file.is_file() else None
    return {
        "category": category, "path": str(path), "expected_sha256": expected,
        "actual_sha256": actual, "actual_bytes": size,
        "match": actual == expected and (expected_bytes is None or size == expected_bytes),
    }


def records(value, category):
    if isinstance(value, dict):
        if "path" in value and "sha256" in value:
            yield compare(value["path"], value["sha256"], category, value.get("bytes"))
        else:
            for key, child in value.items():
                yield from records(child, f"{category}/{key}")
    elif isinstance(value, list):
        for index, child in enumerate(value):
            yield from records(child, f"{category}/{index}")


baseline_path = ROOT / ".trellis/tasks/10-14-06-beamer-document-model/research/independent-check-manifest.json"
baseline = json.loads(baseline_path.read_text(encoding="utf-8"))
checkpoint_path = RESEARCH / "evidence-manifest.json"
checkpoint = json.loads(checkpoint_path.read_text(encoding="utf-8"))
baseline_records = [compare(path, sha, section) for section in
                    ("source_files", "evidence_files", "verified_external_inputs")
                    for path, sha in baseline[section].items()]
checkpoint_records = list(records(checkpoint, "task07_checkpoint"))
retained_originals = {
    str(RESEARCH / "gpui-display-api.md"): RESEARCH / "independent-original-gpui-display-api.md",
    str(RESEARCH.parent / "implement.md"): RESEARCH / "independent-original-implement.md",
}
for item in checkpoint_records:
    retained = retained_originals.get(str(Path(item["path"])))
    if retained is not None and not item["match"]:
        item["retained_original"] = str(retained)
        item["retained_original_sha256"] = digest(retained)
        item["review_change_with_original_preserved"] = digest(retained) == item["expected_sha256"]
source_paths = set(baseline["source_files"])
crate_paths = {str(path.relative_to(ROOT)).replace("\\", "/")
               for path in (ROOT / "crates").rglob("*") if path.is_file()}
product_paths = {path for path in source_paths if path.startswith("crates/") or path in {
    "Cargo.toml", "Cargo.lock", ".github/workflows/ci.yml", "justfile", "deny.toml", "rust-toolchain.toml"}}
source_current = {path: digest(ROOT / path) for path in sorted(source_paths)}
supplement = {}
for path in sorted(RESEARCH.glob("independent-*")):
    if path.is_file() and path.name != "independent-hash-audit.json":
        supplement[str(path.relative_to(ROOT)).replace("\\", "/")] = digest(path)
for name in ("dependency-gate.md", "gpui-display-api.md"):
    supplement[str((RESEARCH / name).relative_to(ROOT)).replace("\\", "/")] = digest(RESEARCH / name)
supplement[str((RESEARCH.parent / "implement.md").relative_to(ROOT)).replace("\\", "/")] = digest(RESEARCH.parent / "implement.md")
for rel in ("docs/adr/ADR-001-gpui-from-zed-tag.md", "docs/adr/ADR-008-gpui-dependency-gate.md"):
    supplement[rel] = digest(ROOT / rel)
result = {
    "captured_utc": datetime.now(timezone.utc).isoformat(),
    "command": "python .trellis/tasks/10-14-07-gpui-shell/research/independent-hash-audit.py",
    "baseline_manifest_sha256": digest(baseline_path),
    "baseline_manifest_expected_sha256": "6d00530903376138990099497bf797ee847aac6e6b233905a7da76dbfdcadf47",
    "checkpoint_manifest_sha256": digest(checkpoint_path),
    "baseline_counts": {section: sum(item["category"] == section for item in baseline_records)
                        for section in ("source_files", "evidence_files", "verified_external_inputs")},
    "product_input_count": len(product_paths), "crate_file_count": len(crate_paths),
    "unlisted_crate_files": sorted(crate_paths - source_paths),
    "missing_crate_files": sorted({p for p in source_paths if p.startswith("crates/")} - crate_paths),
    "product_inputs_unchanged": all(source_current[p] == baseline["source_files"][p] for p in product_paths),
    "source_snapshot_sha256": hashlib.sha256(json.dumps(source_current, sort_keys=True, separators=(",", ":")).encode()).hexdigest(),
    "baseline_records": baseline_records, "checkpoint_records": checkpoint_records,
    "mismatches": [item for item in baseline_records + checkpoint_records if not item["match"]],
    "unexpected_mismatches": [item for item in baseline_records + checkpoint_records
                              if not item["match"] and not item.get("review_change_with_original_preserved")],
    "supplement_sha256": supplement,
}
(RESEARCH / "independent-hash-audit.json").write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
print(json.dumps({key: result[key] for key in ("baseline_manifest_sha256", "checkpoint_manifest_sha256", "baseline_counts", "product_input_count", "crate_file_count", "product_inputs_unchanged", "unlisted_crate_files", "missing_crate_files", "source_snapshot_sha256", "mismatches")}, indent=2))
