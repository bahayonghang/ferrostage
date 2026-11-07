"""PDFium CPU paging probe. Run with python -B from the repository root."""
import ctypes as c
import json
import math
import hashlib
import platform
from pathlib import Path
import statistics
from time import perf_counter_ns

from probe_metadata import (
    bind, close, close_page, count, destroy, digest, get_page,
    height, init, integer, load, ptr, width,
)

create = bind("FPDFBitmap_CreateEx", ptr, integer, integer, integer, ptr, integer)
release = bind("FPDFBitmap_Destroy", None, ptr)
clear = bind("FPDFBitmap_FillRect", None, ptr, integer, integer, integer, integer, c.c_ulong)
render = bind("FPDF_RenderPageBitmap", None, ptr, ptr, integer, integer, integer, integer,
              integer, integer)


def measure(path, target):
    data = path.read_bytes()
    memory = c.create_string_buffer(data)
    doc = load(memory, len(data), None)
    if not doc:
        raise ValueError("Cannot open fixture")
    try:
        pages = count(doc)
        samples = []
        for i in range(220):
            idx = i % pages
            start = perf_counter_ns()
            page = get_page(doc, idx)
            if not page:
                raise ValueError("Cannot load page")
            try:
                w, h = width(page), height(page)
                scale = min(target[0] / w, target[1] / h)
                w, h = round(w * scale), round(h * scale)
                image = create(w, h, 4, None, 0)  # FPDFBitmap_BGRA
                if not image:
                    raise MemoryError("Cannot allocate BGRA image")
                try:
                    clear(image, 0, 0, w, h, 0xffffffff)
                    render(image, page, 0, 0, w, h, 0, 0)
                finally:
                    release(image)
            finally:
                close_page(page)
            elapsed = (perf_counter_ns() - start) / 1e6
            if i >= 20:
                samples.append(elapsed)
        return {"fixture": str(path), "sha256": hashlib.sha256(data).hexdigest(),
                "pages": pages, "target": target, "median_ms": statistics.median(samples),
                "p95_ms": sorted(samples)[math.ceil(.95 * len(samples)) - 1],
                "samples_ms": samples}
    finally:
        close(doc)


if __name__ == "__main__":
    init()
    try:
        print(json.dumps({"scope": "Repeated sequential native page load, fitted BGRA allocation, clear, CPU paint and release. No FerroStage caches, BGRA export, queue, GPUI or OS present.",
                          "warmups": 20, "measured_per_case": 200, "dll_sha256": digest,
                          "os": platform.platform(), "cpu": platform.processor(),
                          "cases": [measure(Path("fixtures/beamer/overlays.pdf"), (1920, 1080)),
                                    measure(Path("fixtures/pdf/perf-100.pdf"), (1920, 1080))]},
                         indent=2))
    finally:
        destroy()
