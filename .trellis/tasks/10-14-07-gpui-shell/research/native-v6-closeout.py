"""Bind completed native receipts to V6; never launch an app or run Cargo."""
from pathlib import Path
import datetime
import hashlib
import json
import subprocess

research = Path(__file__).resolve().parent
root = research.parents[3]


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def read(path):
    return json.loads(path.read_text(encoding="utf-8-sig"))


def write_new(path, value):
    assert not path.exists(), f"Preserve previous receipt: {path}"
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


freeze = read(research / "product-native-freeze-v6.json")
delta = {p: {"frozen": h, "current": sha(root / p)} for p, h in freeze["source_files"].items() if sha(root / p) != h}
assert set(delta) == {"DESIGN.md"}, delta
design_before = research / "native-v6-design-before-closeout.md"
assert sha(design_before) == freeze["source_files"]["DESIGN.md"]
before = design_before.read_text(encoding="utf-8-sig").splitlines()
after = (root / "DESIGN.md").read_text(encoding="utf-8-sig").splitlines()
assert len(before) == len(after)
changed_lines = [i + 1 for i, (a, b) in enumerate(zip(before, after)) if a != b]
assert changed_lines == [151], changed_lines
for key in ("binary", "pdfium", "fixture", "cargo_lock", "final_build_log", "final_native_clippy_log"):
    assert sha(Path(freeze[key]["path"])) == freeze[key]["sha256"], key
verified = {}
for name in ("native-v5-evidence-manifest.json", "product-v6-evidence-manifest.json", "product-independent-v6-manifest.json"):
    manifest = read(research / name)
    entries = manifest.get("records")
    if entries is not None:
        pairs = [(v["path"], v["sha256"]) for v in entries]
    else:
        pairs = [(p, v if isinstance(v, str) else v["sha256"]) for p, v in manifest["files"].items()]
    assert all(sha(root / p) == h for p, h in pairs), name
    verified[name] = {"sha256": sha(research / name), "records_matched": len(pairs)}
for run in ("v6-focus-save", "v6-relaunch"):
    launch = read(research / "native" / f"{run}.launch.json")
    result = read(research / "native" / f"{run}.exit.json")
    assert result["exit_code"] == 0 and result["pid"] == launch["pid"]
    assert launch["exe_sha256"] == freeze["binary"]["sha256"]
    assert launch["source_sha256"] == freeze["source_snapshot_sha256"]
assert read(research / "native/v6-final-window-inventory.json")["ferrostageWindows"] == []
current_crates = {p.relative_to(root).as_posix() for p in (root / "crates").rglob("*") if p.is_file()}
assert not current_crates.difference(freeze["source_files"])
head = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=root, text=True).strip()
assert head == freeze["head"]
status = subprocess.check_output(["git", "status", "--porcelain=v1"], cwd=root)
(research / "native-v6-closeout-git-status.txt").write_bytes(status)
diff = subprocess.run(["git", "diff", "--check"], cwd=root, capture_output=True)
(research / "native-v6-closeout-diff-check.log").write_bytes(diff.stdout + diff.stderr)
assert diff.returncode == 0
identity = {
    "utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
    "command": "python " + Path(__file__).relative_to(root).as_posix(),
    "scope": "Native receipt integrity, unchanged compiled inputs and documentation-only closeout. Not task07/Gate B acceptance.",
    "head": head,
    "frozen_source_count": len(freeze["source_files"]),
    "unchanged_source_count": len(freeze["source_files"]) - len(delta),
    "post_native_documentation_delta": delta,
    "design_changed_lines": changed_lines,
    "crate_files_matched": len(current_crates),
    "binary_sha256": freeze["binary"]["sha256"],
    "fixture_sha256": freeze["fixture"]["sha256"],
    "prior_manifests_verified": verified,
    "owned_native_runs": {run: read(research / "native" / f"{run}.exit.json") for run in ("v6-focus-save", "v6-relaunch")},
    "git_diff_check_exit": diff.returncode,
    "working_tree_status_entries": len(status.splitlines()),
}
write_new(research / "native-v6-closeout-identity.json", identity)
files = sorted(p for p in (research / "native").glob("v6*") if p.is_file())
files += [research / name for name in (
    "native-v6-checkpoint.md", "native-v6-closeout.py", "native-v6-closeout-identity.json",
    "native-v6-closeout-git-status.txt", "native-v6-closeout-diff-check.log",
    "native-v6-design-before-closeout.md", "native-v6-retest-plan.md",
    "product-native-freeze-v6.json", "product-independent-v6-manifest.json",
    "native/run-native.ps1", "native/hold-settings-read-lock.ps1",
)]
records = [{"path": p.relative_to(root).as_posix(), "bytes": p.stat().st_size, "sha256": sha(p)} for p in files]
write_new(research / "native-v6-evidence-manifest.json", {
    "utc": identity["utc"], "scope": identity["scope"], "records": records,
    "limitations": "Physical unplug/ports/single/zero-display, full logical-client size/state matrix, drag/drop, live OS theme event and retirement diagnostics remain unresolved or unverified. Earlier failures are retained, not promoted to passes.",
})
print(json.dumps({"source_unchanged": identity["unchanged_source_count"], "documentation_delta": list(delta), "prior_manifests": verified, "native_records": len(records), "native_manifest_sha256": sha(research / "native-v6-evidence-manifest.json"), "exit_code": 0}, indent=2))
