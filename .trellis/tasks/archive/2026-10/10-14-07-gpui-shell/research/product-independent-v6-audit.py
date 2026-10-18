"""Read-only v6 identity/evidence verification; never runs Cargo or a UI."""
from pathlib import Path
import datetime
import hashlib
import json
import re

research = Path(__file__).resolve().parent
root = research.parents[3]


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def read(name):
    return json.loads((research / name).read_text(encoding="utf-8-sig"))


v5 = read("product-native-freeze-v5.json")
v6 = read("product-native-freeze-v6.json")
native = read("native-v5-evidence-manifest.json")
vendor = read("product-vendor-final-comparison.json")
implementation = read("product-v6-evidence-manifest.json")
assert sha(research / "product-native-freeze-v6.json") == "b7df3a62ff2b1b7212924e47f79d72468ff86356265a1aaf650c0694adb6d7ea"
assert sha(research / "native-v5-evidence-manifest.json") == "ddfec0154c3dfe6731eb08b2c5dcf4b9b27c2a49ccfd54589a05f2ed295d73d0"
assert sha(research / "product-v6-evidence-manifest.json") == "dac5512c8887bbb3278e0b9c68fc8a622731fbc40eea187aa7dfef090a40912c"
sources = v6["source_files"]
aggregate = hashlib.sha256(json.dumps(sources, sort_keys=True, separators=(",", ":")).encode()).hexdigest()
assert aggregate == v6["source_snapshot_sha256"] == "b71eb54d7da71f9b30ac5ab3bdb58fc1ada637bf2991adde10041f0a172170be"
source_bad = [p for p, h in sources.items() if sha(root / p) != h]
vendor_bad = [p for p, h in vendor["final_files"].items() if sha(root / p) != h]
native_bad = [r["path"] for r in native["records"] if sha(root / r["path"]) != r["sha256"] or (root / r["path"]).stat().st_size != r["bytes"]]
implementation_bad = [p for p, r in implementation["files"].items() if sha(root / p) != r["sha256"] or (root / p).stat().st_size != r["bytes"]]
assert not source_bad and not vendor_bad and not native_bad and not implementation_bad
binary = sha(Path(v6["binary"]["path"]))
assert binary == v6["binary"]["sha256"] == "d4f2e653ee793176b384034797ad99e1ad665aa0da6535a454ba88a36b055074"
delta = {p: {"v5": v5["source_files"].get(p), "v6": h} for p, h in sources.items() if h != v5["source_files"].get(p)}
before = {n: sha(research / f"product-v6-before-{n}.rs") == v5["source_files"][f"crates/ferrostage-ui/src/{n}.rs"] for n in ("presenter", "shell")}
assert all(before.values())
crate_files = [p.relative_to(root).as_posix() for p in (root / "crates").rglob("*") if p.is_file()]
unlisted = sorted(set(crate_files) - sources.keys())
assert not unlisted
counts = [int(n) for n in re.findall(r"test result: ok\. (\d+) passed", (research / "product-v6-just-ci.log").read_text(encoding="utf-8-sig"))]
assert sum(counts) == 145
record = {
    "captured_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
    "command": "python " + Path(__file__).relative_to(root).as_posix(),
    "scope": "Integrity and retained-evidence review only. No native launch or Cargo rerun; no Gate B acceptance.",
    "freeze_sha256": sha(research / "product-native-freeze-v6.json"),
    "source_snapshot_sha256": aggregate,
    "source_files_verified": len(sources),
    "source_mismatches": source_bad,
    "v5_to_v6_delta": delta,
    "v5_source_baselines_match": before,
    "current_crate_files": len(crate_files),
    "unlisted_crate_files": unlisted,
    "binary_sha256": binary,
    "native_v5_manifest_sha256": sha(research / "native-v5-evidence-manifest.json"),
    "native_v5_records_verified": len(native["records"]),
    "native_v5_mismatches": native_bad,
    "implementation_manifest_sha256": sha(research / "product-v6-evidence-manifest.json"),
    "implementation_records_verified": len(implementation["files"]),
    "implementation_mismatches": implementation_bad,
    "vendor_records_verified": len(vendor["final_files"]),
    "vendor_mismatches": vendor_bad,
    "default_test_counts": counts,
    "default_test_total": sum(counts),
    "freeze_bound_build_log": v6["final_build_log"],
    "freeze_bound_clippy_log": v6["final_native_clippy_log"],
    "exit_code": 0,
}
for key in ("final_build_log", "final_native_clippy_log", "cargo_lock", "pdfium", "fixture"):
    assert sha(Path(v6[key]["path"])) == v6[key]["sha256"]
(research / "product-independent-v6-audit.json").write_text(json.dumps(record, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
print(json.dumps({k: v for k, v in record.items() if not k.startswith("freeze_bound")}, ensure_ascii=False, indent=2))
