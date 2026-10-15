"""Read-only product identity/policy audit; writes only this review's receipt."""
import datetime
import hashlib
import json
import pathlib
import sys
import tomllib

ROOT = pathlib.Path(__file__).resolve().parents[4]
RESEARCH = pathlib.Path(__file__).resolve().parent


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def read(name):
    return json.loads((RESEARCH / name).read_text(encoding="utf-8-sig"))


def compare(files):
    changed = []
    for name, expected in files.items():
        path = pathlib.Path(name)
        if not path.is_absolute():
            path = ROOT / path
        actual = sha(path) if path.is_file() else None
        if actual != expected:
            changed.append({"path": name, "expected": expected, "actual": actual})
    return {"count": len(files), "matched": len(files) - len(changed), "differences": changed}


freeze_name = sys.argv[1] if len(sys.argv) > 1 else "product-native-freeze-v4.json"
receipt_name = sys.argv[2] if len(sys.argv) > 2 else "product-independent-v4-audit.json"
assert receipt_name.startswith("product-independent-") and "/" not in receipt_name and "\\" not in receipt_name
freeze = read(freeze_name)
files = freeze["source_files"]
canonical = hashlib.sha256(json.dumps(files, sort_keys=True, separators=(",", ":")).encode()).hexdigest()
vendor = read("product-vendor-final-comparison.json")
policy = tomllib.loads((ROOT / "deny.toml").read_text())
ui = tomllib.loads((ROOT / "crates/ferrostage-ui/Cargo.toml").read_text())
app = tomllib.loads((ROOT / "crates/ferrostage-app/Cargo.toml").read_text())
crate_files = sorted(p.relative_to(ROOT).as_posix() for p in (ROOT / "crates").rglob("*") if p.is_file())
extra_crate_files = [p for p in crate_files if p not in files]
receipts = {}
for pattern in ("product-v4-*.log", "product-v5-*.log", "product-v5-*.json", "product-v5-*.patch", "product-final-*.log", "product-vendor-*.json", "product-vendor-*.patch", "native/v4-*.json", "native/v4-*.log"):
    for p in sorted(RESEARCH.glob(pattern)):
        receipts[p.relative_to(ROOT).as_posix()] = sha(p)
record = {
    "captured_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
    "command": "python " + pathlib.Path(__file__).relative_to(ROOT).as_posix() + " " + freeze_name + " " + receipt_name,
    "scope": "Read-only SHA/policy/coverage audit, not Cargo tests or native acceptance. Concurrent implementation may intentionally differ from the frozen historical inputs.",
    "freeze": {"path": freeze_name, "sha256": sha(RESEARCH / freeze_name), "source_snapshot": canonical, "matches_declared_snapshot": canonical == freeze["source_snapshot_sha256"]},
    "current_source_vs_freeze": compare(files),
    "current_vendor_vs_adopted": compare(vendor["final_files"]),
    "crate_file_coverage": {"current_count": len(crate_files), "unlisted": extra_crate_files},
    "binary": {"path": freeze["binary"]["path"], "expected": freeze["binary"]["sha256"], "actual": sha(pathlib.Path(freeze["binary"]["path"]))},
    "licenses": policy["licenses"],
    "advisories": policy.get("advisories"),
    "sources": policy.get("sources"),
    "ui_features": ui["features"],
    "app_features": app["features"],
    "receipt_hashes": receipts,
    "exit_code": 0,
}
(RESEARCH / receipt_name).write_text(json.dumps(record, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
print(json.dumps({k: v for k, v in record.items() if k not in ("receipt_hashes",)}, ensure_ascii=False, indent=2))
