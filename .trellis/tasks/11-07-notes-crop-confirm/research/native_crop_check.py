"""Windows native acceptance: color-coded PDFs, real controls and creation logs."""
import argparse
from contextlib import contextmanager
import ctypes as c
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import time
import tkinter as tk
from reportlab.pdfgen.canvas import Canvas

ROOT = next(p.parent for p in Path(__file__).resolve().parents if p.name == ".trellis")
ARCHIVE = ROOT / ".trellis/tasks/archive/2026-11"
sys.path.insert(0, str(ARCHIVE / "11-07-presenter-airy-glass/research"))
from native_appearance import screenshot, wait, position, to_screen, w
from native_benchmark import CONFIG, DLL, click_control, key, list_windows, presenter, client

PUBLIC = (26, 143, 84)
PRIVATE = (190, 46, 112)


def make_pdf(path, side, public_color=PUBLIC):
    horizontal = side in ("left", "right")
    w, h = (640, 180) if horizontal else (320, 360)
    output = Canvas(str(path), pagesize=(w, h), invariant=True)
    for page in range(2):
        for private in (False, True):
            if horizontal:
                x, y, width, height = w / 2 * (private == (side == "right")), 0, w / 2, h
            else:
                x, y, width, height = 0, h / 2 * (private == (side == "top")), w, h / 2
            color = PRIVATE if private else public_color
            output.setFillColorRGB(*(n / 255 for n in color))
            output.rect(x, y, width, height, stroke=0, fill=1)
            output.setFillColorRGB(1, 1, 1)
            output.setFont("Helvetica", 20)
            output.drawString(x+18, y+height/2, ("Private notes" if private else "Audience slide") + f" {page+1}")
        output.showPage()
    output.save()


def log_text(path):
    return path.read_text(encoding="utf8", errors="replace")


def audience_creates(log):
    # This INFO event follows each successful GPUI open_window call, before
    # placement or presentation commits. A later close cannot erase the event.
    return sum("window created behind confirmation gate" in line and "audience=true" in line
               for line in log_text(log).splitlines())


def panel_elements(hwnd, *ids):
    raw = subprocess.check_output(
        ["powershell.exe", "-NoProfile", "-ExecutionPolicy", "Bypass", "-File",
         str(Path(__file__).with_name("inspect_panel.ps1")), "-WindowHandle", str(hwnd), "-Ids", ",".join(ids)],
        creationflags=subprocess.CREATE_NO_WINDOW, timeout=12)
    return json.loads(raw.decode("utf-8-sig"))


@contextmanager
def launched(args, tag, pdf, final_hash=None):
    folder = args.run / tag
    folder.mkdir()
    config = folder / "settings.toml"
    config.write_text(CONFIG.replace("en-US", "zh-CN"), encoding="utf8")
    log = folder / "product.log"
    source_hash = hashlib.sha256(pdf.read_bytes()).hexdigest()
    with log.open("wb") as stream:
        proc = subprocess.Popen([args.exe, "--config", config, "--pdfium-library", DLL, pdf], cwd=ROOT,
                                stdout=stream, stderr=subprocess.STDOUT, creationflags=subprocess.CREATE_NO_WINDOW,
                                env=dict(os.environ, RUST_LOG="warn,ferrostage_ui::shell=info,ferrostage_perf=info"))
        try:
            wait(lambda: "perf_preview " in log_text(log), "first document preview")
            wait(lambda: "window transaction committed" in log_text(log), "initial presenter placement")
            hwnd = presenter(proc.pid)
            position(hwnd, -1, 120, 100, 1440, 940, 0x10)
            yield proc.pid, hwnd, log
        finally:
            proc.terminate()
            proc.wait(timeout=6)
    assert hashlib.sha256(pdf.read_bytes()).hexdigest() == (final_hash or source_hash)


def sole_presenter(pid, hwnd, log):
    time.sleep(.25)
    assert any(h == hwnd for h,_,_ in list_windows(pid))
    assert not any(h != hwnd and min(client(h)) > 200 for h,_,_ in list_windows(pid))
    assert audience_creates(log) == 0, "An audience window was created before approval"


def color_counts(im, public_color=PUBLIC):
    rgb = im.convert("RGB")
    pixels = rgb.get_flattened_data() if hasattr(rgb, "get_flattened_data") else rgb.getdata()
    from collections import Counter
    counts = Counter(pixels)
    return {key: sum(n for c,n in counts.items() if max(abs(c[i]-color[i]) for i in range(3)) <= 2)
            for key,color in (("public", public_color), ("private", PRIVATE))}


def audience_frame(args, pid, hwnd, log, tag, creates=1, generation=1):
    wait(lambda: any(f"generation={generation} " in line and "phase=Settled" in line
                     for line in log_text(log).splitlines()), "audience settled")
    assert audience_creates(log) == creates
    aud = next(h for h, _, _ in list_windows(pid) if h != hwnd and min(client(h)) > 200)
    return screenshot(args, aud, tag)


def verify(args):
    results = {}
    for side in ("right", "left", "top", "bottom"):
        if args.reload_only:
            break
        pdf = args.run / f"{side}.pdf"
        make_pdf(pdf, side)
        with launched(args, side, pdf) as (pid, hwnd, log):
            if args.baseline:
                key(pid, hwnd, 0x74)
                frame = audience_frame(args, pid, hwnd, log, "before-audience")
                counts = color_counts(frame)
                assert counts["public"] > 1000 and counts["private"] > 1000
                return {"baseline": counts}
            if side == "right":
                for method in ("button", "escape"):
                    key(pid, hwnd, 0x74)  # normal F5/Start path
                    click_control(pid, hwnd, "crop-layout-right")
                    click_control(pid, hwnd, "crop-sample-next")
                    sole_presenter(pid, hwnd, log)
                    key(pid, hwnd, 0x74)
                    if method == "button":
                        click_control(pid, hwnd, "crop-cancel")
                    else:
                        key(pid, hwnd, 0x1b)
                    sole_presenter(pid, hwnd, log)
                    assert not panel_elements(hwnd, "crop-confirm"), "Cancellation did not dismiss the panel"
                    results["cancel_"+method] = {"created_audience_windows": audience_creates(log)}
            key(pid, hwnd, 0x74)
            click_control(pid, hwnd, "crop-layout-"+side)
            bounds = panel_elements(hwnd, "crop-public-preview", "crop-notes-preview")
            screen_origin = w.POINT()
            assert to_screen(hwnd, c.byref(screen_origin))
            def region(image, id):
                x1,y1,x2,y2 = bounds[id]["rect"]
                return color_counts(image.crop((x1-screen_origin.x,y1-screen_origin.y,
                                                x2-screen_origin.x,y2-screen_origin.y)))
            # Both colors must be painted inside the real confirmation UI.
            def preview():
                image = screenshot(args, hwnd)
                public = region(image, "crop-public-preview")
                notes = region(image, "crop-notes-preview")
                return image if public["public"] > 1000 and not public["private"] and notes["private"] > 1000 and not notes["public"] else None
            paired = wait(preview, "paired sample rasters")
            if side == "right":
                paired.save(args.out / "paired-preview.png")
            sole_presenter(pid, hwnd, log)
            click_control(pid, hwnd, "crop-confirm")
            projected = audience_frame(args, pid, hwnd, log, "audience-"+side)
            counts = color_counts(projected)
            assert counts["public"] > 1000 and counts["private"] == 0, (side, counts)
            results[side] = {"public_preview_pixels": region(paired, "crop-public-preview"),
                             "private_preview_pixels": region(paired, "crop-notes-preview"), "audience_pixels": counts,
                             "sha256": hashlib.sha256(pdf.read_bytes()).hexdigest()}
    for fixture in (() if args.reload_only else ("ordinary", "independent")):
        pdf = ROOT / f"fixtures/beamer/{fixture}.pdf"
        with launched(args, fixture, pdf) as (pid, hwnd, log):
            key(pid, hwnd, 0x74)
            audience_frame(args, pid, hwnd, log, "direct-"+fixture)
            results[fixture] = {"direct_start_audience_windows": audience_creates(log)}
    if not args.reload_only:
        with launched(args, "whole-page", args.run / "right.pdf") as (pid, hwnd, log):
            key(pid, hwnd, 0x74)
            click_control(pid, hwnd, "crop-layout-whole")
            sole_presenter(pid, hwnd, log)
            click_control(pid, hwnd, "crop-confirm")
            counts = color_counts(audience_frame(args, pid, hwnd, log, "explicit-whole"))
            assert counts["public"] > 1000 and counts["private"] > 1000
            results["whole_page_by_choice"] = counts
        results["real_beamer"] = real_beamer(args)
    results["replacement"] = replacement_trial(args)
    return results


def real_beamer(args):
    with launched(args, "actual-beamer", ROOT / "fixtures/beamer/notes-right.pdf") as (pid, hwnd, log):
        key(pid, hwnd, 0x74)
        click_control(pid, hwnd, "crop-layout-right")
        wait(lambda: panel_elements(hwnd, "crop-confirm").get("crop-confirm", {}).get("enabled"), "real Beamer start preview")
        screenshot(args, hwnd, "beamer-start-preview")
        key(pid, hwnd, 0x1b)
        sole_presenter(pid, hwnd, log)
        click_control(pid, hwnd, "document-options")
        click_control(pid, hwnd, "crop-check")
        click_control(pid, hwnd, "crop-layout-right")
        click_control(pid, hwnd, "crop-sample-next")
        wait(lambda: panel_elements(hwnd, "crop-confirm").get("crop-confirm", {}).get("enabled"), "real Beamer previews")
        screenshot(args, hwnd, "beamer-confirmation")
        click_control(pid, hwnd, "crop-confirm")
        sole_presenter(pid, hwnd, log)
        key(pid, hwnd, 0x74)
        audience_frame(args, pid, hwnd, log, "beamer-projection")
        assert any("page=0 " in line and "phase=Settled" in line for line in log_text(log).splitlines())
        # Core offers Resume only after at least one second of presenting.
        time.sleep(1.1)
        key(pid, hwnd, 0x1b)
        wait(lambda: not any(h != hwnd and min(client(h)) > 200 for h, _, _ in list_windows(pid)), "stop")
        click_control(pid, hwnd, "start-resume")
        wait(lambda: audience_creates(log) == 2, "resume with same accepted crop")
        return {"menu_confirm_audience_windows": 0, "sample_browsing_preserved_page": 0,
                "starts_including_resume": audience_creates(log)}


def replacement_trial(args):
    active = args.run / "live.pdf"
    replacement = args.run / "new-structure.pdf"
    new_color = (35, 95, 225)
    make_pdf(active, "right")
    make_pdf(replacement, "top", new_color)
    final_hash = hashlib.sha256(replacement.read_bytes()).hexdigest()
    with launched(args, "replacement", active, final_hash) as (pid, hwnd, log):
        key(pid, hwnd, 0x74)
        click_control(pid, hwnd, "crop-layout-right")
        click_control(pid, hwnd, "crop-confirm")
        wait(lambda: "phase=Settled" in log_text(log), "initial public frame")
        time.sleep(.6)
        before = log_text(log).count("perf_submit")
        time.sleep(.6)
        idle_submits = log_text(log).count("perf_submit") - before
        assert idle_submits == 0, f"Static note panes kept submitting raster jobs: {idle_submits}"
        # The writer updates only this test's generated input.
        os.replace(replacement, active)
        wait(lambda: "replacement awaiting crop confirmation" in log_text(log), "held replacement")
        held_log = next(line for line in reversed(log_text(log).splitlines()) if "replacement awaiting crop confirmation" in line)
        held_generation = int(re.search(r"\bgeneration=(\d+)", held_log)[1])
        assert held_generation > 1
        old = audience_frame(args, pid, hwnd, log, "reload-retained")
        assert color_counts(old)["public"] > 1000 and color_counts(old)["private"] == 0
        key(pid, hwnd, 0x1b)
        wait(lambda: not any(h != hwnd and min(client(h)) > 200 for h, _, _ in list_windows(pid)), "stop pending reload")
        click_control(pid, hwnd, "start-resume")
        click_control(pid, hwnd, "crop-layout-top")
        assert audience_creates(log) == 1
        click_control(pid, hwnd, "crop-confirm")
        new = audience_frame(args, pid, hwnd, log, "reload-applied", creates=2, generation=held_generation)
        counts = color_counts(new, new_color)
        assert counts["public"] > 1000 and counts["private"] == 0
        return {"old_generation_retained": True, "idle_raster_submissions_in_600ms": idle_submits,
                "new_audience_pixels": counts, "new_pdf_sha256": final_hash}


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--exe", type=Path, required=True)
    parser.add_argument("--run", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--baseline", action="store_true")
    parser.add_argument("--reload-only", action="store_true")
    args = parser.parse_args()
    args.exe = args.exe.resolve(strict=True)
    args.run = args.run.resolve()
    args.run.mkdir(parents=True, exist_ok=False)
    args.out.mkdir(parents=True, exist_ok=True)
    c.windll.user32.SetProcessDpiAwarenessContext(c.c_void_p(-4))
    backing = tk.Tk()
    backing.overrideredirect(True)
    backing.geometry("1530x1030+85+65")
    backing.configure(bg="#536f71")
    backing.attributes("-topmost", True)
    backing.update()
    try:
        result = verify(args)
    finally:
        backing.destroy()
    result["executable_sha256"] = hashlib.sha256(args.exe.read_bytes()).hexdigest()
    result["pdfium_sha256"] = hashlib.sha256(DLL.read_bytes()).hexdigest()
    result_name = "baseline.json" if args.baseline else "reload-check.json" if args.reload_only else "native-check.json"
    (args.out / result_name).write_text(
        json.dumps(result, indent=2)+"\n", encoding="utf8")
    print(json.dumps(result))
