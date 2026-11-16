"""Compare completed native benchmark runs, including the segments of the UI path.

Usage: python compare_runs.py --before <dirs...> --after <dirs...> --output comparison.json
"""
import argparse
from collections import defaultdict
import json
import math
from pathlib import Path
import statistics

from native_benchmark import parse


def summarize(values):
    values = sorted(values)
    return {"n": len(values), "median_ms": round(statistics.median(values), 3),
            "p95_ms": round(values[math.ceil(len(values) * .95) - 1], 3)}


def collect(paths):
    groups = defaultdict(list)
    blocks = {}
    identity = set()
    dimensions = defaultdict(set)
    binary = set()
    burst = defaultdict(int)
    for path in paths:
        report = json.loads((path / "results.json").read_text(encoding="utf8"))
        identity.add((report["pdf_sha256"], report["dll_sha256"], report["config"]))
        binary.add(report["exe_sha256"])
        block = defaultdict(list)
        for proc in report["processes"]:
            if "error" in proc:
                raise ValueError(f"Incomplete sample: {path} process {proc['index']}")
            dimensions["presenter_at_open"].add(tuple(proc["presenter_client_px"]))
            dimensions["clients_after_start"].add(tuple(sorted(tuple(x) for x in proc["window_clients_px"])))
            log = (path / f"process-{proc['index']:02}" / "product.log").read_text(encoding="utf8")
            events = [event for line in log.splitlines() if (event := parse(line))]
            for sample in proc["samples"]:
                op = sample["op"]
                if op == "burst":
                    burst[sample["status"]] += 1
                    continue
                if op == "warmup":
                    continue
                block[op].append(sample["ms"])
                if "settled_ms" in sample:
                    block[op + "_settled"].append(sample["settled_ms"])
                if op.startswith("open_"):
                    dimensions[op].add((sample["width"], sample["height"]))
                    gen = sample["generation"]
                    begin = next(e for e in events if e["event"] == "perf_open" and e["token"] == sample["token"])
                    prev = begin["us"]
                    for name in ("candidate", "metadata", "candidate_raster", "commit", "preview"):
                        phase = next(e for e in events if e["event"] == "perf_" + name and e["generation"] == gen)
                        block[f"{op}_to_{name}"].append((phase["us"] - prev) / 1000)
                        prev = phase["us"]
                if op not in ("forward_traversal", "repeat"):
                    continue
                selected = next(e for e in events if e["event"] == "perf_selected"
                                and e["navigation"] == sample["navigation"] and e["generation"] == sample["generation"])
                visible = next(e for e in events if e["event"] == "perf_audience" and e["phase"] == "FirstVisible"
                               and e["navigation"] == sample["navigation"] and e["generation"] == sample["generation"])
                matching = [e for e in events if e["event"] == "perf_submit" and e["surface"] == "Audience"
                            and e["page"] == sample["page"] and e["generation"] == sample["generation"]
                            and selected["us"] <= e["us"] <= visible["us"]]
                if not matching:
                    continue
                end = next((e for e in events if e["event"] == "perf_result" and e["surface"] == "Audience"
                            and e["id"] in {m["id"] for m in matching} and e["us"] <= visible["us"]), None)
                if end:
                    submit = next(m for m in matching if m["id"] == end["id"])
                    dimensions[op + "_audience_target"].add((submit["width"], submit["height"]))
                    block[f"{op}_select_to_submit"].append((submit["us"] - selected["us"]) / 1000)
                    block[f"{op}_submit_to_consume"].append((end["us"] - submit["us"]) / 1000)
                    block[f"{op}_consume_to_draw"].append((visible["us"] - end["us"]) / 1000)
                    layer = "worker_hit" if end["cache_hit"] == "true" else "worker_miss"
                    block[layer].append(sample["ms"])
        blocks[path.name] = {k: summarize(v) for k, v in block.items()}
        for k, v in block.items():
            groups[k].extend(v)
    if len(identity) != 1 or len(binary) != 1:
        raise ValueError("Mixed input, settings, runtime or binary in one variant")
    return {"summary": {k: summarize(v) for k, v in groups.items()}, "blocks": blocks,
            "burst": dict(burst), "binary_sha256": binary.pop(),
            "input_runtime_settings": identity.pop(), "dimensions": {k: sorted(v) for k, v in dimensions.items()}}


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--before", nargs="+", type=Path, required=True)
    parser.add_argument("--after", nargs="+", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    before, after = collect(args.before), collect(args.after)
    if before["input_runtime_settings"] != after["input_runtime_settings"]:
        raise ValueError("A/B input, runtime or settings mismatch")
    if before["dimensions"] != after["dimensions"]:
        raise ValueError(f"A/B dimensions differ: {before['dimensions']} vs {after['dimensions']}")
    for op in ("open_fresh_process", "open_same_process", "start", "forward_traversal", "repeat"):
        if before["summary"][op]["n"] != after["summary"][op]["n"]:
            raise ValueError(f"A/B sample counts differ for {op}")
    if sum(before["burst"].values()) != sum(after["burst"].values()):
        raise ValueError("A/B burst input counts differ")
    args.output.write_text(json.dumps({"before": before, "after": after}, indent=2) + "\n", encoding="utf8")
