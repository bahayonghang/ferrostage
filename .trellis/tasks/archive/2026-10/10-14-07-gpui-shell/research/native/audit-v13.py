"""Audit completed V13 native receipts without touching product inputs."""
import datetime
import hashlib
import json
import re
import tomllib
from pathlib import Path

root = Path(__file__).resolve().parents[5]
research = Path(__file__).resolve().parent.parent
native = research / "native"
digest = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
freeze = json.loads((research / "product-native-freeze-v13.json").read_text())
runs = []
for tag in ("v13-main", "v13-dark", "v13-en", "v13-open", "v13-light-small"):
    launch = json.loads((native / f"{tag}.launch.json").read_text(encoding="utf-8-sig"))
    end = json.loads((native / f"{tag}.exit.json").read_text(encoding="utf-8-sig"))
    windows = json.loads((native / f"{tag}-final-windows.json").read_text())
    out = (native / f"{tag}.stdout.log").read_text()
    err = (native / f"{tag}.stderr.log").read_text()
    before, after = [native / f"{tag}.config-{phase}.toml" for phase in ("before", "after")]
    before_obj, after_obj = [tomllib.loads(p.read_text(encoding="utf-8-sig")) for p in (before, after)]
    findings = [dict(line=i, text=line) for i, line in enumerate(out.splitlines(), 1)
                if re.search(r"\b(ERROR|WARN)\b|0x80070578|window not found", line)]
    assert launch["pid"] == end["pid"] and end["exit_code"] == 0
    assert not windows["windows"]
    assert launch["exe_sha256"] == freeze["binary"]["sha256"]
    assert launch["source_sha256"] == freeze["source_snapshot_sha256"]
    assert not err and before_obj == after_obj
    runs.append(dict(tag=tag, pid=end["pid"], started=launch["utc"], exited=end["utc"],
                     exit_code=end["exit_code"], empty_final_windows=True,
                     config_before_sha256=digest(before), config_after_sha256=digest(after),
                     config_bytes_equal=before.read_bytes() == after.read_bytes(),
                     config_parsed_equal=True, final_display=after_obj["display"],
                     final_general=after_obj["general"], findings=findings,
                     stderr_empty=True, poll_cancel_before_quit="shell polling task cancelled before quit" in out,
                     close_result="FAIL_F12" if findings else "PASS_SCOPED"))
screenshots = []
for path in sorted(native.glob("v13-*.json")):
    data = json.loads(path.read_text(encoding="utf-8-sig"))
    if not isinstance(data, dict):
        continue
    for shot in data.get("screenshots", []):
        image_path = Path(shot["path"])
        assert digest(image_path) == shot["sha256"], image_path
        screenshots.append(dict(metadata=path.name, image=image_path.name, sha256=shot["sha256"]))
result = dict(utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),
              scope="Completed V13 native runs; first four scoped clean, fifth F12 FAIL. No current-source assertion after release.",
              freeze_sha256=digest(research / "product-native-freeze-v13.json"),
              pre_release_identity=json.loads((native / "v13-pre-release-identity.json").read_text()),
              runs=runs, screenshots=screenshots)
target = native / "v13-completed-runs-audit.json"
with target.open("x", encoding="utf-8", newline="\n") as out:
    json.dump(result, out, indent=2, ensure_ascii=False)
    out.write("\n")
print(json.dumps(dict(runs=len(runs), screenshots=len(screenshots), result=[r["close_result"] for r in runs], sha256=digest(target))))
