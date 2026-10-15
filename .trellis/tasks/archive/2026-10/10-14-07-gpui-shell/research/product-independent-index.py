"""Index immutable inputs/receipts used by the independent product review."""
import datetime
import hashlib
import json
from pathlib import Path

research = Path(__file__).resolve().parent
root = research.parents[3]
paths = set()
for pattern in (
    "product-independent-*.py",
    "product-independent-review.md",
    "product-independent-v*-audit.json",
    "product-native-freeze-v[45].json",
    "product-v5-*.log",
    "product-v5-before-*.rs",
    "product-v5-source-delta.json",
    "product-v5-repair.patch",
    "product-v5-processes.json",
    "product-vendor-final-*",
    "product-test-support-deny.log",
    "product-final-full-deny-retry.log",
    "native/v4-*",
):
    paths.update(p for p in research.glob(pattern) if p.is_file())
hashes = {
    p.relative_to(root).as_posix(): hashlib.sha256(p.read_bytes()).hexdigest()
    for p in sorted(paths)
}
manifest = research / "product-independent-manifest.json"
manifest.write_text(json.dumps({
    "captured_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
    "scope": "Independent v5 source/automated-evidence review. Native v5 belongs to Root and is deliberately not indexed while running. No overall task07 or Gate B acceptance.",
    "command": "python " + Path(__file__).relative_to(root).as_posix(),
    "hash_algorithm": "SHA256 raw file bytes",
    "files": hashes,
    "exit_code": 0,
}, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
digest = hashlib.sha256(manifest.read_bytes()).hexdigest()
(research / "product-independent-manifest.sha256").write_text(digest + "  " + manifest.name + "\n", encoding="utf-8")
print(json.dumps({"files": len(hashes), "manifest_sha256": digest}))
