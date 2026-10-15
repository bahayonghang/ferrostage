"""Index the completed independent v6 review without changing earlier records."""
from pathlib import Path
import datetime
import hashlib
import json

research = Path(__file__).resolve().parent
root = research.parents[3]
implementation = json.loads((research / "product-v6-evidence-manifest.json").read_text(encoding="utf-8"))
native = json.loads((research / "native-v5-evidence-manifest.json").read_text(encoding="utf-8"))
paths = {root / p for p in implementation["files"]}
paths.update(root / r["path"] for r in native["records"])
for name in (
    "product-independent-v6-review.md",
    "product-independent-v6-audit.py",
    "product-independent-v6-audit.json",
    "product-independent-v6-index.py",
    "product-v6-evidence-manifest.json",
    "native-v5-evidence-manifest.json",
    "native-v5-checkpoint.md",
):
    paths.add(research / name)
hashes = {p.relative_to(root).as_posix(): hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(paths)}
manifest = research / "product-independent-v6-manifest.json"
manifest.write_text(json.dumps({
    "captured_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
    "scope": "Independent v6 F7/F8 source/automated-evidence review; ready only for Root targeted native retest. Native v6 evidence is separately owned and not indexed here. No task07/Gate B acceptance.",
    "command": "python " + Path(__file__).relative_to(root).as_posix(),
    "hash_algorithm": "SHA256 raw file bytes",
    "files": hashes,
    "exit_code": 0,
}, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
digest = hashlib.sha256(manifest.read_bytes()).hexdigest()
(research / "product-independent-v6-manifest.sha256").write_text(digest + "  " + manifest.name + "\n", encoding="utf-8")
bad = [p for p, h in hashes.items() if hashlib.sha256((root / p).read_bytes()).hexdigest() != h]
assert not bad
print(json.dumps({"files": len(hashes), "mismatches": bad, "manifest_sha256": digest}))
