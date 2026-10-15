"""Index immutable review evidence without rewriting earlier manifests."""
from pathlib import Path
import hashlib
import json
from datetime import datetime, timezone

ROOT = Path(__file__).resolve().parents[4]
RESEARCH = Path(__file__).resolve().parent
paths = set()
for name in ["native-v6-evidence-manifest.json", "native-v5-evidence-manifest.json",
             "product-v6-evidence-manifest.json", "product-independent-v6-manifest.json"]:
    p = RESEARCH / name
    paths.add(p.relative_to(ROOT).as_posix())
    data = json.loads(p.read_text(encoding="utf-8-sig"))
    rows = data.get("records", data.get("files"))
    paths.update(rows if isinstance(rows, dict) else (r["path"] for r in rows))
for name in ["native-v6-independent-closeout.md", "native-v6-independent-audit.py",
             "native-v6-independent-audit.json", "native-v6-independent-index.py",
             "poc-1-report.md", "product-independent-review.md", "product-native-freeze-v4.json"]:
    paths.add((RESEARCH / name).relative_to(ROOT).as_posix())
for stem in ["v4-a-cli.launch.json", "v4-a-cli.stdout.log", "v4-a-cli.exit.json",
             "v4-a-presenting.json", "v4-a-presenting-0.jpg", "v4-a-audience-page1.json",
             "v4-a-audience-page1-0.jpg", "v4-escape-ready-page2.json", "v4-escape-ready-page2-0.jpg"]:
    paths.add((RESEARCH / "native" / stem).relative_to(ROOT).as_posix())
paths.update(["crates/ferrostage-ui/src/audience.rs", "crates/ferrostage-core/src/snapshot.rs"])
records = {p: {"bytes": (ROOT / p).stat().st_size,
               "sha256": hashlib.sha256((ROOT / p).read_bytes()).hexdigest()} for p in sorted(paths)}
out = {"utc": datetime.now(timezone.utc).isoformat(),
       "command": "python .trellis/tasks/10-14-07-gpui-shell/research/native-v6-independent-index.py",
       "scope": "Independent native closeout review; referenced prior records retained verbatim; not Gate B acceptance",
       "records": records, "exit_code": 0}
target = RESEARCH / "native-v6-independent-manifest.json"
target.write_text(json.dumps(out, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
digest = hashlib.sha256(target.read_bytes()).hexdigest()
target.with_suffix(".sha256").write_text(digest + "  " + target.name + "\n", encoding="utf-8")
print(json.dumps({"records": len(records), "manifest_sha256": digest, "exit_code": 0}))
