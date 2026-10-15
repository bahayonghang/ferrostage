"""Read-only verification of retained native closeout inputs; writes this review's audit only."""
from pathlib import Path
import hashlib
import json
import platform
import sys
from datetime import datetime, timezone

ROOT = Path(__file__).resolve().parents[4]
RESEARCH = Path(__file__).resolve().parent


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def read(path):
    return json.loads(path.read_text(encoding="utf-8-sig"))


def validate_manifest(name, expected, count):
    path = RESEARCH / name
    assert sha(path) == expected, name
    data = read(path)
    records = data.get("records", data.get("files"))
    if isinstance(records, dict):
        records = [dict(path=k, **(v if isinstance(v, dict) else {"sha256": v}))
                   for k, v in records.items()]
    assert len(records) == count, name
    for record in records:
        actual = ROOT / record["path"]
        assert sha(actual) == record["sha256"], record["path"]
        if "bytes" in record:
            assert actual.stat().st_size == record["bytes"], record["path"]
    return {"sha256": expected, "records_matched": count}


def main():
    out = {"utc": datetime.now(timezone.utc).isoformat(),
           "command": "python .trellis/tasks/10-14-07-gpui-shell/research/native-v6-independent-audit.py",
           "environment": {"platform": platform.platform(), "python": sys.version},
           "scope": "Retained native evidence and post-native documentation delta; no Cargo or UI run."}
    out["manifests"] = {
        n: validate_manifest(n, h, c) for n, h, c in [
            ("native-v6-evidence-manifest.json", "6c7551fa9c38890252c0500951c90597500b2f7a690d1e52aefe89986a6ac1a0", 54),
            ("native-v5-evidence-manifest.json", "ddfec0154c3dfe6731eb08b2c5dcf4b9b27c2a49ccfd54589a05f2ed295d73d0", 108),
            ("product-v6-evidence-manifest.json", "dac5512c8887bbb3278e0b9c68fc8a622731fbc40eea187aa7dfef090a40912c", 26),
            ("product-independent-v6-manifest.json", "252a22e5ab5173b7c9a03da0e6a15768edd2d904b6cfd5404c42c9f72c101fb5", 141),
        ]}
    freeze_path = RESEARCH / "product-native-freeze-v6.json"
    assert sha(freeze_path) == "b7df3a62ff2b1b7212924e47f79d72468ff86356265a1aaf650c0694adb6d7ea"
    freeze = read(freeze_path)
    files = freeze["source_files"]
    aggregate = hashlib.sha256(json.dumps(files, sort_keys=True, separators=(",", ":")).encode()).hexdigest()
    assert aggregate == freeze["source_snapshot_sha256"]
    assert len(files) == 602
    deltas = {p: {"frozen": h, "current": sha(ROOT / p)}
              for p, h in files.items() if sha(ROOT / p) != h}
    assert set(deltas) == {"DESIGN.md"}, deltas
    original = RESEARCH / "native-v6-design-before-closeout.md"
    assert sha(original) == files["DESIGN.md"]
    before = original.read_text(encoding="utf-8").splitlines()
    after = (ROOT / "DESIGN.md").read_text(encoding="utf-8").splitlines()
    assert len(before) == len(after)
    lines = [i + 1 for i, (a, b) in enumerate(zip(before, after)) if a != b]
    assert lines == [151], lines
    crates = sorted(p.relative_to(ROOT).as_posix() for p in (ROOT / "crates").rglob("*") if p.is_file())
    assert len(crates) == 94
    assert all(p in files and sha(ROOT / p) == files[p] for p in crates)
    artifacts = {}
    for key in ["binary", "pdfium", "cargo_lock", "fixture"]:
        record = freeze[key]
        p = Path(record["path"])
        assert sha(p) == record["sha256"] and p.stat().st_size == record["bytes"], key
        artifacts[key] = record
    out["identity"] = {"freeze_sha256": sha(freeze_path), "frozen_count": len(files),
                       "frozen_aggregate": aggregate, "current_match_count": len(files) - len(deltas),
                       "delta": deltas, "changed_design_lines": lines,
                       "original_design_sha256": sha(original), "crate_files_matched": len(crates),
                       "artifacts": artifacts}
    native = RESEARCH / "native"
    runs = {}
    for name in ["v6-focus-save", "v6-relaunch"]:
        launch, end = read(native / (name + ".launch.json")), read(native / (name + ".exit.json"))
        assert launch["pid"] == end["pid"] and end["exit_code"] == 0
        assert launch["exe_sha256"] == artifacts["binary"]["sha256"]
        assert launch["source_sha256"] == aggregate
        assert sha(native / (name + ".config-before.toml")) == launch["config_sha256"].lower()
        runs[name] = {"launch": launch, "exit": end,
                      "config_after_sha256": sha(native / (name + ".config-after.toml"))}
    assert runs["v6-focus-save"]["exit"]["utc"] < runs["v6-relaunch"]["launch"]["utc"]
    readback = read(native / "v6-settings-readback.json")
    expected_saved = "b96b8708fcb7207f7673c322be62d1f2f68d1c9ff382fe3607fbf7d0d7249ab9"
    assert runs["v6-focus-save"]["config_after_sha256"] == expected_saved
    assert runs["v6-relaunch"]["config_after_sha256"] == expected_saved
    assert runs["v6-relaunch"]["launch"]["config_sha256"].lower() == expected_saved
    out["runs"] = runs
    out["settings_readback"] = readback
    out["lock_release"] = read(native / "v6-dark-lock.unlock.json")
    out["lock_release_boundary"] = "unlock receipt is written after FileStream.Dispose; helper exit0 is Root's recorded command observation, not a field in this receipt"
    final_windows = read(native / "v6-final-window-inventory.json")
    assert final_windows["ferrostageWindows"] == []
    out["final_window_inventory"] = final_windows
    out["screenshots_checked"] = []
    for path in sorted(native.glob("v6-*.json")):
        data = read(path)
        for shot in data.get("screenshots", []):
            assert sha(native / shot["file"]) == shot["sha256"]
            out["screenshots_checked"].append(shot["file"])
    out["document_read_versions"] = {}
    for rel in [".trellis/tasks/10-14-07-gpui-shell/prd.md", ".trellis/tasks/10-14-presenter-v0.3/prd.md",
                ".trellis/tasks/10-14-presenter-v0.3/design.md", ".trellis/tasks/10-14-presenter-v0.3/traceability.md",
                ".trellis/tasks/10-14-presenter-v0.3/execution-state.md", ".trellis/tasks/10-14-presenter-v0.3/research/gate-b.md",
                "docs/feature-parity.md", "docs/PROJECT_PLAN.md", "DESIGN.md"]:
        out["document_read_versions"][rel] = sha(ROOT / rel)
    out["exit_code"] = 0
    dest = RESEARCH / "native-v6-independent-audit.json"
    dest.write_text(json.dumps(out, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"result": "PASS", "manifests": out["manifests"], "current_inputs": "601/602; DESIGN line151 only",
                      "crates": len(crates), "output": str(dest)}, ensure_ascii=True))


if __name__ == "__main__":
    main()
