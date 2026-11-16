"""Measure first preview including process launch to catch font discovery moved into startup."""
import argparse
import json
import os
from pathlib import Path
import statistics
import subprocess
import sys
import time

ROOT = next(p.parent for p in Path(__file__).resolve().parents if p.name == ".trellis")
sys.path.insert(0, str(ROOT / ".trellis/tasks/archive/2026-11/11-07-presenter-performance/research"))
from native_benchmark import CONFIG, DLL, PDF, Trace, elapsed, sha


def sample(exe, folder, glass):
    folder.mkdir()
    config = folder / "settings.toml"
    config.write_text(CONFIG.replace("[general]", f'[general]\ntheme = "light"\nglass_effect = {str(glass).lower()}'), encoding="utf8")
    log = folder / "product.log"
    with log.open("wb") as stream:
        start = time.monotonic()
        proc = subprocess.Popen([exe, "--config", config, "--pdfium-library", DLL, PDF], cwd=ROOT,
                                stdout=stream, stderr=subprocess.STDOUT, creationflags=subprocess.CREATE_NO_WINDOW,
                                env=dict(os.environ, RUST_LOG="warn,ferrostage_perf=info"))
        trace = Trace(log, proc)
        try:
            accepted = trace.wait(0, "perf_open")
            preview = trace.wait(0, "perf_preview", page=0)
            return {"launch_to_observed_preview_ms": round((time.monotonic()-start)*1000, 2),
                    "accept_to_preview_ms": elapsed(accepted, preview),
                    "actual_px": [preview["width"], preview["height"]]}
        finally:
            trace.reader.close()
            proc.terminate()
            proc.wait(timeout=6)


if __name__ == "__main__":
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--reference", type=Path, required=True)
    ap.add_argument("--candidate", type=Path, required=True)
    ap.add_argument("--run", type=Path, required=True)
    ap.add_argument("--out", type=Path, required=True)
    a = ap.parse_args()
    folder = a.run.resolve()
    folder.mkdir(parents=True, exist_ok=False)
    cases = [("reference", a.reference.resolve(), False), ("glass-on", a.candidate.resolve(), True),
             ("glass-off", a.candidate.resolve(), False)]
    data = {name: [] for name, _, _ in cases}
    for i in range(6):
        for name, exe, glass in (cases if i % 2 == 0 else list(reversed(cases))):
            data[name].append(sample(exe, folder / f"{i}-{name}", glass))
    sizes = {tuple(row["actual_px"]) for rows in data.values() for row in rows}
    if len(sizes) != 1:
        raise ValueError(f"Preview dimensions differ: {sizes}")
    report = {"fixture": sha(PDF), "pdfium": sha(DLL), "reference_exe": sha(a.reference),
              "candidate_exe": sha(a.candidate), "samples": data,
              "medians": {name: {k: round(statistics.median(r[k] for r in rows), 2)
                          for k in rows[0] if k.endswith("_ms")} for name, rows in data.items()}}
    a.out.write_text(json.dumps(report, indent=2)+"\n", encoding="utf8")
    print(json.dumps(report["medians"]))
