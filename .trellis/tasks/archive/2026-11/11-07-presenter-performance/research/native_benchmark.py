"""Native product-path benchmark. Run on the interactive Windows desktop, in a fresh result directory."""
import argparse
import ctypes as c
from ctypes import wintypes as w
import hashlib
import json
import math
import os
from pathlib import Path
import platform
import re
import statistics
import subprocess
import time

ROOT = next(p.parent for p in Path(__file__).resolve().parents if p.name == ".trellis")
PDF = ROOT / "fixtures/pdf/perf-100.pdf"
DLL = ROOT / "cache/pdfium-7881-win-x64/bin/pdfium.dll"
CONFIG = """schema_version = 1
[general]
language = "en-US"
reopen_last_document = false
[display]
start_mode = "dual"
single_monitor_mode = "windowed_audience"
presenter_start_fullscreen = false
audience_fullscreen = false
[cache]
l1_gpu_mib = 256
l2_cpu_mib = 512
l3_thumb_mib = 64
"""
ANSI = re.compile(r"\x1b\[[0-9;]*[A-Za-z]")
USER = c.WinDLL("user32", use_last_error=True)
CALLBACK = c.WINFUNCTYPE(w.BOOL, w.HWND, w.LPARAM)


def win(name, result, *args):
    fn = getattr(USER, name)
    fn.argtypes, fn.restype = args, result
    return fn


enum = win("EnumWindows", w.BOOL, CALLBACK, w.LPARAM)
visible = win("IsWindowVisible", w.BOOL, w.HWND)
pid_of = win("GetWindowThreadProcessId", w.DWORD, w.HWND, c.POINTER(w.DWORD))
title_of = win("GetWindowTextW", c.c_int, w.HWND, w.LPWSTR, c.c_int)
class_of = win("GetClassNameW", c.c_int, w.HWND, w.LPWSTR, c.c_int)
client_of = win("GetClientRect", w.BOOL, w.HWND, c.POINTER(w.RECT))
post = win("PostMessageW", w.BOOL, w.HWND, w.UINT, w.WPARAM, w.LPARAM)
scan = win("MapVirtualKeyW", w.UINT, w.UINT, w.UINT)
screen_to_client = win("ScreenToClient", w.BOOL, w.HWND, c.POINTER(w.POINT))


def owner(hwnd):
    pid = w.DWORD()
    pid_of(hwnd, c.byref(pid))
    return pid.value


def list_windows(pid):
    found = []

    @CALLBACK
    def visit(hwnd, _):
        if owner(hwnd) == pid and visible(hwnd):
            title, cls = c.create_unicode_buffer(512), c.create_unicode_buffer(256)
            title_of(hwnd, title, len(title))
            class_of(hwnd, cls, len(cls))
            found.append((hwnd, title.value, cls.value))
        return True

    enum(visit, 0)
    return found


def key(pid, hwnd, vk):
    if owner(hwnd) != pid:
        raise RuntimeError("Native input target no longer belongs to the test process")
    bits = 1 | (scan(vk, 0) << 16)
    if vk in (0x22, 0x21):
        bits |= 1 << 24
    for msg, flags in ((0x100, bits), (0x101, bits | (3 << 30))):
        if not post(hwnd, msg, vk, flags):
            raise c.WinError(c.get_last_error())


def click_control(pid, hwnd, automation_id):
    # Resolve the control in our PID. Posted clicks do not move the user's cursor.
    if owner(hwnd) != pid:
        raise RuntimeError("Presenter window has been replaced")
    helper = Path(__file__).with_name("locate_controls.ps1")
    cmd = ["powershell.exe", "-NoProfile", "-ExecutionPolicy", "Bypass", "-File", helper,
           "-WindowHandle", str(hwnd), "-AutomationId", automation_id]
    found = json.loads(subprocess.check_output(cmd, timeout=15, creationflags=subprocess.CREATE_NO_WINDOW))
    point = w.POINT(found["x"], found["y"])
    if not screen_to_client(hwnd, c.byref(point)):
        raise c.WinError(c.get_last_error())
    width, height = client(hwnd)
    if owner(hwnd) != pid or not (0 <= point.x < width and 0 <= point.y < height):
        raise RuntimeError("Control bounds did not resolve into the owned window")
    coords = (point.y << 16) | point.x
    for message, flags in ((0x0200, 0), (0x0201, 1), (0x0202, 0)):
        if not post(hwnd, message, flags, coords):
            raise c.WinError(c.get_last_error())


def parse(line):
    row = ANSI.sub("", line)
    name = re.search(r"\bperf_\w+\b", row)
    if not name:
        return None
    out = {"event": name.group()}
    for key_, val in re.findall(r'(\w+)=("[^"]*"|[^ ,\s]+)', row[name.end():]):
        val = val.strip('"')
        out[key_] = int(val) if val.isdecimal() else val
    return out


class Trace:
    def __init__(self, path, proc):
        self.reader = path.open("rb")
        self.proc = proc
        self.events = []
        self.trailing = b""

    def read(self):
        pieces = (self.trailing + self.reader.read()).split(b"\n")
        self.trailing = pieces.pop()
        for line in pieces:
            event = parse(line.decode("utf-8", "replace"))
            if event:
                self.events.append(event)

    def wait(self, pos, name, **fields):
        deadline = time.monotonic() + 12
        while time.monotonic() < deadline:
            self.read()
            for event in self.events[pos:]:
                if event["event"] == name and all(event.get(k) == v for k, v in fields.items()):
                    return event
            if self.proc.poll() is not None:
                raise RuntimeError(f"Process exited {self.proc.returncode} waiting for {name}")
            time.sleep(.003)
        raise TimeoutError(f"Timeout waiting for {name} {fields} from event {pos}")


def elapsed(first, last):
    us = last["us"] - first["us"]
    if us < 0:
        raise ValueError("Reversed trace boundary")
    return us / 1000


def sample_open(trace, offset, label, reports):
    opened = trace.wait(offset, "perf_open")
    candidate = trace.wait(offset, "perf_candidate", token=opened["token"])
    preview = trace.wait(offset, "perf_preview", generation=candidate["generation"], page=0)
    reports.append({"op": label, "token": opened["token"], "generation": candidate["generation"],
                    "ms": elapsed(opened, preview), "before_candidate_ms": elapsed(opened, candidate),
                    "width": preview["width"], "height": preview["height"]})
    return candidate["generation"]


def sample_audience(trace, offset, source, generation, target):
    selected = trace.wait(offset, "perf_selected", generation=generation, page=target)
    ident = {k: selected[k] for k in ("generation", "navigation", "page", "window")}
    visible_ = trace.wait(offset, "perf_audience", phase="FirstVisible", **ident)
    settled = trace.wait(offset, "perf_audience", phase="Settled", **ident)
    return {**ident, "ms": elapsed(source, visible_), "settled_ms": elapsed(source, settled),
            "accept_to_select_ms": elapsed(source, selected)}


def navigate(trace, proc, hwnd, generation, source_page, delta, label, reports):
    pos = len(trace.events)
    key(proc.pid, hwnd, 0x22 if delta == 1 else 0x21)
    direction = "next" if delta == 1 else "previous"
    entered = trace.wait(pos, "perf_input", generation=generation, page=source_page, direction=direction)
    target = source_page + delta
    reports.append({"op": label, "direction": direction,
                    **sample_audience(trace, pos, entered, generation, target)})
    return target


def client(hwnd):
    rect = w.RECT()
    if not client_of(hwnd, c.byref(rect)):
        raise c.WinError(c.get_last_error())
    return [rect.right, rect.bottom]


def presenter(pid):
    for hwnd, title, _ in list_windows(pid):
        if title == "FerroStage" or title.endswith("— FerroStage"):
            return hwnd
    raise RuntimeError("Could not identify the test process presenter window")


def run_process(exe, directory, n_open, first_touch, repeat, warmups, burst, output):
    config, log = directory / "settings.toml", directory / "product.log"
    config.write_text(CONFIG, encoding="utf8")
    env = dict(os.environ, RUST_LOG="warn,ferrostage_perf=info")
    with log.open("wb") as stream:
        proc = subprocess.Popen([exe, "--config", config, "--pdfium-library", DLL, PDF],
                                stdout=stream, stderr=subprocess.STDOUT, env=env,
                                cwd=ROOT, creationflags=subprocess.CREATE_NO_WINDOW)
        trace = Trace(log, proc)
        try:
            rows = output["samples"]
            generation = sample_open(trace, 0, "open_fresh_process", rows)
            hwnd = presenter(proc.pid)
            output["presenter_client_px"] = client(hwnd)
            for _ in range(n_open):
                pos = len(trace.events)
                click_control(proc.pid, hwnd, "home")
                click_control(proc.pid, hwnd, "recent-0")
                generation = sample_open(trace, pos, "open_same_process", rows)
            if not first_touch and not repeat and not burst:
                return
            pos = len(trace.events)
            key(proc.pid, hwnd, 0x74)
            started = trace.wait(pos, "perf_start", generation=generation, page=0)
            rows.append({"op": "start", **sample_audience(trace, pos, started, generation, 0)})
            output["window_clients_px"] = [client(h) for h, _, _ in list_windows(proc.pid)
                                           if min(client(h)) >= 200]
            page = 0
            for _ in range(warmups):
                page = navigate(trace, proc, hwnd, generation, page, 1 if page == 0 else -1,
                                "warmup", rows)
            for _ in range(first_touch):
                page = navigate(trace, proc, hwnd, generation, page, 1, "forward_traversal", rows)
            # Revisit this pair before sampling repeated navigation.
            delta = -1 if page else 1
            for i in range(repeat + (2 if repeat else 0)):
                page = navigate(trace, proc, hwnd, generation, page, delta,
                                "repeat" if i >= 2 else "warmup", rows)
                delta = -delta
            if burst:
                pos = len(trace.events)
                source_page = page
                for _ in range(burst):
                    key(proc.pid, hwnd, 0x22)
                for p in range(source_page, source_page + burst):
                    trace.wait(pos, "perf_input", generation=generation, page=p, direction="next")
                final = trace.wait(pos, "perf_selected", generation=generation, page=source_page + burst)
                trace.wait(pos, "perf_audience", generation=generation,
                           navigation=final["navigation"], page=final["page"], phase="Settled")
                for p in range(source_page, source_page + burst):
                    events = trace.events[pos:]
                    entered = next(e for e in events if e["event"] == "perf_input" and e["page"] == p)
                    selected = next(e for e in events if e["event"] == "perf_selected" and e["page"] == p + 1)
                    drawn = next((e for e in events if e["event"] == "perf_audience"
                                  and e["generation"] == generation and e["navigation"] == selected["navigation"]
                                  and e["phase"] == "FirstVisible"), None)
                    rows.append({"op": "burst", "page": p + 1, "navigation": selected["navigation"],
                                 "status": "drawn" if drawn else "superseded",
                                 "ms": elapsed(entered, drawn) if drawn else None})
        finally:
            trace.reader.close()
            if proc.poll() is None:
                try:
                    post(presenter(proc.pid), 0x10, 0, 0)  # WM_CLOSE
                    proc.wait(4)
                except (RuntimeError, subprocess.TimeoutExpired):
                    proc.terminate()
                    proc.wait(4)


def sha(path):
    with path.open("rb") as file:
        return hashlib.file_digest(file, "sha256").hexdigest()


def main():
    args = argparse.ArgumentParser(description=__doc__)
    args.add_argument("--exe", type=Path, required=True)
    args.add_argument("--out", type=Path, required=True)
    args.add_argument("--processes", type=int, default=1)
    args.add_argument("--reopens", type=int, default=1, help="per process")
    args.add_argument("--forward", type=int, default=4, help="total first-traversal steps across processes")
    args.add_argument("--repeat", type=int, default=4, help="total repeated-page steps across processes")
    args.add_argument("--warmups", type=int, default=2, help="per process, use an even number")
    args.add_argument("--burst", type=int, default=3, help="rapid steps per process")
    a = args.parse_args()
    if (a.processes < 1 or min(a.reopens, a.forward, a.repeat, a.warmups, a.burst) < 0
            or a.warmups % 2 or math.ceil(a.forward / a.processes) + a.burst + 2 >= 100):
        args.error("Counts must stay within the 100-page document; warmups must be even")
    win("SetProcessDPIAware", w.BOOL)()
    exe = a.exe.resolve(strict=True)
    a.out.mkdir(parents=True, exist_ok=False)
    dll_hash = sha(DLL)
    if dll_hash != json.loads((ROOT / "docs/pdfium-runtime.json").read_text())["dll_sha256"].lower():
        raise RuntimeError("Pinned PDFium differs from the project manifest")
    report = {"scope": "Native app input dispatch to GPUI draw; excludes OS Present and hardware input.",
              "exe_sha256": sha(exe), "pdf_sha256": sha(PDF), "dll_sha256": dll_hash,
              "host": platform.platform(), "cpu": platform.processor(), "config": CONFIG,
              "processes": [], "summary_ms": {}}
    try:
        for i in range(a.processes):
            out = {"index": i, "samples": []}
            report["processes"].append(out)
            directory = a.out / f"process-{i:02}"
            directory.mkdir()
            first_touch = a.forward // a.processes + (i < a.forward % a.processes)
            revisits = a.repeat // a.processes + (i < a.repeat % a.processes)
            try:
                run_process(exe, directory, a.reopens, first_touch, revisits,
                            a.warmups, a.burst, out)
            except Exception as error:
                out["error"] = str(error)
                raise
    finally:
        values = {}
        for process in report["processes"]:
            for row in process["samples"]:
                if row["ms"] is not None:
                    values.setdefault(row["op"], []).append(row["ms"])
        for op, samples in values.items():
            report["summary_ms"][op] = {"n": len(samples), "median": statistics.median(samples),
                                       "p95": sorted(samples)[math.ceil(.95 * len(samples)) - 1]}
        (a.out / "results.json").write_text(json.dumps(report, indent=2), encoding="utf8")
        print(json.dumps(report["summary_ms"]))


if __name__ == "__main__":
    main()
