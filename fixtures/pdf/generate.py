"""Deterministic self-authored PDF fixtures; no external PDF or compiler input.

The embedded Type3 glyph outlines below are original geometric test glyphs.
They and the fixture are licensed MIT OR Apache-2.0 with this repository.
"""
from pathlib import Path
import hashlib
import json

ROOT = Path(__file__).resolve().parent
objects = []


def obj(data):
    objects.append(data.encode("ascii") if isinstance(data, str) else data)
    return len(objects)


def stream(data):
    data = data.encode("ascii")
    return obj(f"<< /Length {len(data)} >>\nstream\n".encode() + data + b"\nendstream")


catalog = obj("")
pages = obj("")
# Distinct E, equals, m, c and superscript 2 from simple strokes.
glyphs = {
    "E": "60 50 70 650 re f 60 50 450 70 re f 60 340 350 70 re f 60 630 450 70 re f",
    "equal": "60 250 450 60 re f 60 430 450 60 re f",
    "m": "60 50 60 500 re f 270 50 60 500 re f 480 50 60 500 re f 60 490 480 60 re f",
    "c": "80 50 60 500 re f 80 50 420 60 re f 80 490 420 60 re f",
    "two": "70 640 m 500 640 l 500 390 l 70 100 l 500 100 l 45 w S",
}
charprocs = {name: stream("600 0 0 0 600 750 d1\n" + shape) for name, shape in glyphs.items()}
font = obj("<< /Type /Font /Subtype /Type3 /FontBBox [0 0 600 750] /FontMatrix [.001 0 0 .001 0 0] "
           "/CharProcs << " + " ".join(f"/{name} {ref} 0 R" for name, ref in charprocs.items()) +
           " >> /Encoding << /Type /Encoding /Differences [65 /E /equal /m /c /two] >> "
           "/FirstChar 65 /LastChar 69 /Widths [600 600 600 600 600] /Resources << >> >>")
alpha = obj("<< /Type /ExtGState /ca .5 /CA .5 >>")
resources = f"<< /Font << /F1 {font} 0 R >> /ExtGState << /A {alpha} 0 R >> >>"


def art(w, h):
    parts = [f"1 1 1 rg 0 0 {w} {h} re f", f"1 0 0 rg 0 {h/2} {w/2} {h/2} re f",
             f"0 1 0 rg {w/2} {h/2} {w/2} {h/2} re f", f"0 0 1 rg 0 0 {w/2} {h/2} re f",
             f"1 1 0 rg {w/2} 0 {w/2} {h/2} re f"]
    # Text and thin curves on both halves, plus transparent red over blue.
    for x, y in [(15, 22), (w / 2 + 15, 22), (15, h / 2 + 22), (w / 2 + 15, h / 2 + 22)]:
        parts += [f"0 0 0 rg BT /F1 18 Tf {x} {y} Td (ABCD) Tj 0 7 Td /F1 10 Tf (E) Tj ET",
                  f"0 0 0 RG .35 w {x} {y+30} m {x+15} {y+60} {x+40} {y+10} {x+60} {y+30} c S"]
    parts += ["q /A gs 1 0 0 rg 120 10 20 20 re f Q"]
    return "\n".join(parts)


specs = [
    (320, 180, "1 0 0 rg 0 0 320 180 re f", ""),
    (640, 180, art(640, 180), ""),
    (320, 360, art(320, 360), ""),
    (180, 640, art(180, 640), "/Rotate 90"),
    (700, 240, "q 1 0 0 1 30 30 cm\n" + art(640, 180) + "\nQ", "/CropBox [30 30 670 210]"),
    (320, 180, "q /A gs 1 0 0 rg 0 0 160 180 re f Q", ""),
    (640, 180, art(640, 180), "/Rotate 180"),
    (180, 640, art(180, 640), "/Rotate 270"),
    (320, 180, art(320, 180), ""),
]
page_ids = []
for w, h, commands, extra in specs:
    content = stream(commands)
    page_ids.append(obj(f"<< /Type /Page /Parent {pages} 0 R /MediaBox [0 0 {w} {h}] {extra} /Resources {resources} /Contents {content} 0 R >>"))
objects[pages - 1] = f"<< /Type /Pages /Kids [{' '.join(f'{i} 0 R' for i in page_ids)}] /Count {len(page_ids)} >>".encode()
labels = "/Nums [0 << /P (red) >> 1 << /P (1) >> 2 << /P (notes:1) >> 3 << /P (rotated) >>]"
objects[catalog - 1] = f"<< /Type /Catalog /Pages {pages} 0 R /PageLabels << {labels} >> >>".encode()
pdf = bytearray(b"%PDF-1.7\n%\xe2\xe3\xcf\xd3\n")
offsets = [0]
for index, data in enumerate(objects, 1):
    offsets.append(len(pdf))
    pdf += f"{index} 0 obj\n".encode() + data + b"\nendobj\n"
xref = len(pdf)
pdf += f"xref\n0 {len(offsets)}\n0000000000 65535 f \n".encode()
pdf += b"".join(f"{offset:010} 00000 n \n".encode() for offset in offsets[1:])
pdf += f"trailer\n<< /Size {len(offsets)} /Root {catalog} 0 R >>\nstartxref\n{xref}\n%%EOF\n".encode()
(ROOT / "raster.pdf").write_bytes(pdf)
manifest = {
    "raster.pdf": hashlib.sha256(pdf).hexdigest(),
    "embedded_type3_glyphs_sha256": hashlib.sha256(json.dumps(glyphs, sort_keys=True).encode()).hexdigest(),
    "fonts": "original embedded Type3 geometric E=mc^2; no system font dependency",
}
performance = ROOT / "perf-100.pdf"
if performance.is_file():
    manifest["perf-100.pdf"] = hashlib.sha256(performance.read_bytes()).hexdigest()
(ROOT / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
