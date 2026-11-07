"""Measure PDFium page metadata calls only. Run from the repository root on Windows."""
import ctypes as c
import hashlib
import json
import math
import platform
import statistics
import sys
from pathlib import Path
from time import perf_counter_ns

root = Path.cwd()
manifest = json.loads((root / "docs/pdfium-runtime.json").read_text("utf-8"))
dll = root / "cache/pdfium-7881-win-x64/bin/pdfium.dll"
digest = hashlib.sha256(dll.read_bytes()).hexdigest()
if digest.lower() != manifest["dll_sha256"].lower():
    raise RuntimeError("Unexpected PDFium hash")
api = c.CDLL(str(dll))


def bind(name, result, *args):
    fn = getattr(api, name)
    fn.argtypes = args
    fn.restype = result
    return fn


ptr, integer = c.c_void_p, c.c_int
init = bind("FPDF_InitLibrary", None)
destroy = bind("FPDF_DestroyLibrary", None)
load = bind("FPDF_LoadMemDocument64", ptr, ptr, c.c_size_t, c.c_char_p)
close = bind("FPDF_CloseDocument", None, ptr)
count = bind("FPDF_GetPageCount", integer, ptr)
get_page = bind("FPDF_LoadPage", ptr, ptr, integer)
close_page = bind("FPDF_ClosePage", None, ptr)
width = bind("FPDF_GetPageWidthF", c.c_float, ptr)
height = bind("FPDF_GetPageHeightF", c.c_float, ptr)
rotation = bind("FPDFPage_GetRotation", integer, ptr)
label = bind("FPDF_GetPageLabel", c.c_ulong, ptr, integer, ptr, c.c_ulong)


def run(path):
    data = path.read_bytes()
    memory = c.create_string_buffer(data)
    samples = []
    for i in range(35):
        start = perf_counter_ns()
        doc = load(memory, len(data), None)
        if not doc:
            raise RuntimeError("Unable to load fixture")
        opened = perf_counter_ns()
        try:
            pages = count(doc)
            if pages <= 0:
                raise ValueError("Empty document")
            for idx in range(pages):
                page = get_page(doc, idx)
                if not page:
                    raise RuntimeError("Unable to read page")
                try:
                    if width(page) <= 0 or height(page) <= 0 or rotation(page) < 0:
                        raise ValueError("Invalid page geometry")
                    n = label(doc, idx, None, 0)
                    if n:
                        label(doc, idx, c.create_string_buffer(n), n)
                finally:
                    close_page(page)
            end = perf_counter_ns()
        finally:
            close(doc)
        if i >= 5:
            samples.append([(opened-start)/1e6, (end-opened)/1e6])

    def summary(values):
        ordered = sorted(values)
        return {"median": statistics.median(ordered),
                "p95": ordered[math.ceil(.95 * len(ordered)) - 1]}

    return {"fixture": str(path), "sha256": hashlib.sha256(data).hexdigest(),
            "pages": pages, "document_open_ms": summary([a for a, b in samples]),
            "page_metadata_ms": summary([b for a, b in samples]), "samples_ms": samples}


if __name__ == "__main__":
    init()
    try:
        report = {"scope": "In-memory PDFium C calls through Python; excludes disk, lopdf, Rust wrapping, queueing, raster, upload and paint",
                  "warmups": 5, "measured_per_file": 30, "os": platform.platform(),
                  "cpu": platform.processor(), "python": sys.version.split()[0],
                  "dll_sha256": digest, "fixtures": [run(Path(p)) for p in
                  ("fixtures/beamer/overlays.pdf", "fixtures/pdf/perf-100.pdf")]}
        print(json.dumps(report, indent=2))
    finally:
        destroy()
