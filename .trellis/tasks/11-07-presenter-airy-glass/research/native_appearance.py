"""Run the presenter in isolated settings and capture only its own client pixels (Windows)."""
import argparse
import ctypes as c
from ctypes import wintypes as w
from contextlib import contextmanager
import json
import os
from pathlib import Path
import subprocess
import sys
import time
import tkinter as tk
import tomllib

from PIL import ImageChops, ImageGrab, ImageStat

ROOT = next(p.parent for p in Path(__file__).resolve().parents if p.name == ".trellis")
sys.path.insert(0, str(ROOT / ".trellis/tasks/archive/2026-11/11-07-presenter-performance/research"))
from native_benchmark import CONFIG, DLL, click_control, client, key, list_windows, presenter, win

position = win("SetWindowPos", w.BOOL, w.HWND, w.HWND, c.c_int, c.c_int, c.c_int, c.c_int, w.UINT)
to_screen = win("ClientToScreen", w.BOOL, w.HWND, c.POINTER(w.POINT))


def wait(fn, desc):
    deadline = time.monotonic() + 12
    while time.monotonic() < deadline:
        value = fn()
        if value:
            return value
        time.sleep(.05)
    raise RuntimeError("Timed out waiting for " + desc)


@contextmanager
def launch(args, config, label, pdf=None):
    log = args.run / (label + ".log")
    command = [str(args.exe.resolve()), "--config", str(config), "--pdfium-library", str(DLL)]
    if pdf:
        command.append(str(pdf))
    with log.open("wb") as stream:
        proc = subprocess.Popen(command, cwd=ROOT, stdout=stream, stderr=subprocess.STDOUT,
                                creationflags=subprocess.CREATE_NO_WINDOW,
                                env=dict(os.environ, RUST_LOG="warn,ferrostage_perf=info"))
        try:
            wait(lambda: list_windows(proc.pid), "presenter window")
            hwnd = presenter(proc.pid)
            if not position(hwnd, -1, 120, 100, 1440, 940, 0x10):
                raise c.WinError()
            if pdf:
                wait(lambda: "perf_preview " in log.read_text(encoding="utf8", errors="replace"), "PDF preview")
            time.sleep(.5)
            yield proc.pid, hwnd
        finally:
            if proc.poll() is None:
                proc.terminate()
            proc.wait(timeout=5)


def screenshot(args, hwnd, name=None):
    # Keep the measured client above the controlled backdrop after a modal closes.
    if not position(hwnd, -1, 120, 100, 0, 0, 0x11):
        raise c.WinError()
    point = w.POINT()
    if not to_screen(hwnd, c.byref(point)):
        raise c.WinError()
    width, height = client(hwnd)
    c.windll.dwmapi.DwmFlush()
    time.sleep(.25)
    im = ImageGrab.grab(bbox=(point.x, point.y, point.x + width, point.y + height),
                        all_screens=True).convert("RGB")
    if all(lo == hi for lo, hi in im.getextrema()):
        raise RuntimeError(f"Captured only a blank surface for {name}")
    if name:
        im.save(args.out / (name + ".png"))
    return im


def native_accent(hwnd):
    class Policy(c.Structure):
        _fields_ = [(k, w.DWORD) for k in ("state", "flags", "color", "animation")]

    class Attribute(c.Structure):
        _fields_ = [("kind", c.c_int), ("data", c.c_void_p), ("size", c.c_size_t)]

    try:
        getter = win("GetWindowCompositionAttribute", w.BOOL, w.HWND, c.POINTER(Attribute))
    except AttributeError:
        return None
    value = Policy()
    data = Attribute(19, c.addressof(value), c.sizeof(value))
    return value.state if getter(hwnd, c.byref(data)) else None


def edge_response(args, backing, hwnd):
    """Normalize a hard backing edge against two uniform backing colors."""
    colors = ("#101010", "#f0f0f0")
    uniform = []
    for color in colors:
        backing.configure(bg=color)
        backing.update()
        uniform.append(screenshot(args, hwnd))
    point = w.POINT()
    to_screen(hwnd, c.byref(point))
    boundary = uniform[0].height // 2
    edge = point.y + boundary - backing.winfo_rooty()
    canvas = tk.Canvas(backing, bg=colors[0], borderwidth=0, highlightthickness=0)
    canvas.pack(fill="both", expand=True)
    canvas.create_rectangle(-2, edge, 1600, 1200, fill=colors[1], outline="")
    backing.update()
    frame = screenshot(args, hwnd)
    # The first 12 physical pixels are inside the exposed field at desktop width.
    def level(im, y):
        return sum(ImageStat.Stat(im.crop((6, y, 13, y+2))).mean) / 3
    profile = []
    for offset in range(-64, 65, 4):
        y = boundary + offset
        lo, hi = [level(im, y) for im in uniform]
        response = (level(frame, y) - lo) / (hi - lo) if hi - lo > 1 else None
        profile.append([offset, round(hi-lo, 2), None if response is None else round(response, 3)])
    canvas.destroy()
    backing.configure(bg="#536f71")
    backing.update()
    return profile


def reads(config, field, val):
    def read():
        try:
            return tomllib.loads(config.read_text(encoding="utf8"))["general"].get(field) == val
        except (FileNotFoundError, KeyError, tomllib.TOMLDecodeError):
            return False
    wait(read, f"saved {field}={val}")


def run(args):
    config = args.run / "settings.toml"
    config.write_text(CONFIG.replace('en-US', 'zh-CN'), encoding="utf8")
    # Cover the area behind the presenter with a controlled background for
    # composited screen captures. No other application's contents are sampled.
    backing = tk.Tk()
    backing.overrideredirect(True)
    backing.geometry("1530x1030+85+65")
    backing.configure(bg="#536f71")
    backing.attributes("-topmost", True)
    backing.update()
    report = {}
    try:
        with launch(args, config, "home") as (pid, hwnd):
            def against_backing(color):
                backing.configure(bg=color)
                backing.update()
                return screenshot(args, hwnd)
            def backdrop_delta():
                first, second = against_backing("#314b70"), against_backing("#a98962")
                backing.configure(bg="#536f71")
                backing.update()
                return [round(x, 3) for x in ImageStat.Stat(ImageChops.difference(first, second)).mean]
            screenshot(args, hwnd, "light-on-home")
            report["window_accent_enabled"] = native_accent(hwnd)
            report["on_backdrop_mean_rgb_delta"] = backdrop_delta()
            report["field_edge_response_on"] = edge_response(args, backing, hwnd)
            click_control(pid, hwnd, "settings")
            screenshot(args, hwnd, "light-on-settings")
            click_control(pid, hwnd, "setting-general.glass_effect")
            reads(config, "glass_effect", False)
            click_control(pid, hwnd, "close-settings")
            screenshot(args, hwnd, "light-off-home")
            report["window_accent_disabled"] = native_accent(hwnd)
            report["off_backdrop_mean_rgb_delta"] = backdrop_delta()
            report["field_edge_response_off"] = edge_response(args, backing, hwnd)
            click_control(pid, hwnd, "settings")
            click_control(pid, hwnd, "setting-general.theme-dark")
            reads(config, "theme", "dark")
            click_control(pid, hwnd, "close-settings")
            screenshot(args, hwnd, "dark-off-home")
            click_control(pid, hwnd, "settings")
            click_control(pid, hwnd, "setting-general.glass_effect")
            reads(config, "glass_effect", True)
            screenshot(args, hwnd, "dark-on-settings")
            click_control(pid, hwnd, "close-settings")
            screenshot(args, hwnd, "dark-on-home")
            click_control(pid, hwnd, "settings")
            click_control(pid, hwnd, "setting-general.theme-system")
            reads(config, "theme", "system")
            click_control(pid, hwnd, "close-settings")
            key(pid, hwnd, 0x09)
            screenshot(args, hwnd, "system-keyboard-home")
            position(hwnd, -1, 120, 100, 860, 830, 0x10)
            time.sleep(.3)
            screenshot(args, hwnd, "narrow-home")
            report["narrow_backdrop_mean_rgb_delta"] = backdrop_delta()
            report["window_accent_narrow"] = native_accent(hwnd)
        pdf = ROOT / "fixtures/pdf/perf-100.pdf"
        audiences = {}
        for mode in ("system", "light"):
            with launch(args, config, mode, pdf) as (pid, hwnd):
                reads(config, "glass_effect", True)
                if mode == "light":
                    click_control(pid, hwnd, "settings")
                    click_control(pid, hwnd, "setting-general.theme-light")
                    reads(config, "theme", "light")
                    click_control(pid, hwnd, "close-settings")
                else:
                    reads(config, "theme", "system")
                screenshot(args, hwnd, mode + "-workbench")
                key(pid, hwnd, 0x74)
                log = args.run / (mode + ".log")
                wait(lambda: "phase=Settled" in log.read_text(encoding="utf8", errors="replace"), "audience settled")
                audience = next(h for h,t,cls in list_windows(pid) if h != hwnd and min(client(h)) > 250)
                position(audience, -1, 130, 105, 1050, 740, 0x10)
                time.sleep(.7)
                audiences[mode] = screenshot(args, audience, mode + "-audience")
        if audiences["system"].size != audiences["light"].size:
            raise ValueError("Audience dimensions changed")
        diff = ImageChops.difference(audiences["system"], audiences["light"])
        report["audience_client_difference_bbox"] = diff.getbbox()
        # Exclude the DWM activation border and rounded outer corners.
        report["audience_inner_difference_bbox"] = diff.crop((20, 20, diff.width-20, diff.height-20)).getbbox()
        if report["audience_inner_difference_bbox"]:
            raise RuntimeError("Audience content changed between the two presenter themes")
    finally:
        backing.destroy()
    (args.out / "native-check.json").write_text(json.dumps(report, indent=2), encoding="utf8")
    print(json.dumps(report))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--exe", required=True, type=Path)
    parser.add_argument("--run", required=True, type=Path)
    parser.add_argument("--out", required=True, type=Path)
    a = parser.parse_args()
    a.run = a.run.resolve()
    a.run.mkdir(parents=True, exist_ok=False)
    a.out.mkdir(parents=True, exist_ok=True)
    c.windll.user32.SetProcessDpiAwarenessContext(c.c_void_p(-4))
    run(a)
