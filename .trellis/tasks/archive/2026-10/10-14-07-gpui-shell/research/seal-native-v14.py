"""Seal only closed V14 receipts; never rewrite prior evidence."""
import hashlib
import json
from datetime import datetime, timezone
from pathlib import Path

root = Path(__file__).resolve().parents[4]
research = Path(__file__).resolve().parent
native = research / "native"
freeze = json.loads((research / "product-native-freeze-v14.json").read_text())

def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

for name, expected in freeze["source_files"].items():
    assert digest(root / name) == expected, name
for name in ("binary", "pdfium", "fixture", "cargo_lock"):
    item = freeze[name]
    assert digest(Path(item["path"])) == item["sha256"], name

screenshots = 0
for path in native.glob("v14-*.json"):
    data = json.loads(path.read_text(encoding="utf-8-sig"))
    if isinstance(data, dict):
        for shot in data.get("screenshots", []):
            assert digest(Path(shot["path"])) == shot["sha256"], str(path)
            screenshots += 1
for tag in ("v14-light-small", "v14-dark-matrix", "v14-core-relaunch"):
    data = json.loads((native / f"{tag}-post-exit-windows.json").read_text())
    assert not (data.get("windows", []) if isinstance(data, dict) else data), tag

paths = sorted(native.glob("v14-*")) + [research / "native-v14-checkpoint.md", research / "native-v14-core-results.md", research / "native-v14-plan.md"]
items = [{"path": str(p.relative_to(root)).replace("\\", "/"), "bytes": p.stat().st_size, "sha256": digest(p)} for p in paths if p.is_file()]
result = {"utc": datetime.now(timezone.utc).isoformat(), "source_files_verified": len(freeze["source_files"]), "source_sha256": freeze["source_snapshot_sha256"], "exe_sha256": freeze["binary"]["sha256"], "original_screenshots_verified": screenshots, "result": "Scoped core PASS; F13 DC leak and F14 AccessKit close FAIL; Gate B INCOMPLETE", "files": items}
output = research / "native-v14-evidence-manifest.json"
with output.open("x", encoding="utf-8", newline="\n") as handle:
    json.dump(result, handle, indent=2)
    handle.write("\n")
print(json.dumps({"files": len(items), "screenshots": screenshots, "manifest_sha256": digest(output), "source_inputs_verified": len(freeze["source_files"])}))
