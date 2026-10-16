"""Original media smoke inputs; writes only this research fixture directory.

MIT OR Apache-2.0. This is an authored PDF, not TeX/Beamer output. Movie.pdf in
the repository remains the separate actual-Beamer acceptance input.
"""
from pathlib import Path
import hashlib
import json

ROOT = Path(__file__).resolve().parent


class Pdf:
    def __init__(self):
        self.objects = []

    def add(self, value):
        self.objects.append(value.encode("ascii") if isinstance(value, str) else value)
        return len(self.objects)

    def stream(self, data, extra=""):
        return self.add(f"<< /Length {len(data)} {extra} >>\nstream\n".encode()
                        + data + b"\nendstream")

    def save(self, name):
        result = bytearray(b"%PDF-1.7\n")
        offsets = [0]
        for number, item in enumerate(self.objects, 1):
            offsets.append(len(result))
            result += f"{number} 0 obj\n".encode() + item + b"\nendobj\n"
        xref = len(result)
        result += f"xref\n0 {len(offsets)}\n0000000000 65535 f \n".encode()
        for offset in offsets[1:]:
            result += f"{offset:010d} 00000 n \n".encode()
        result += (f"trailer\n<< /Size {len(offsets)} /Root 1 0 R >>\n"
                   f"startxref\n{xref}\n%%EOF\n").encode()
        (ROOT / name).write_bytes(result)
        return {"file": name, "bytes": len(result),
                "sha256": hashlib.sha256(result).hexdigest()}


pdf = Pdf()
pdf.add("")
pdf.add("")
font = pdf.add("<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>")
cases = [
    ("GIF", "synthetic-three-frames.gif", True, False, 0, None, True),
    ("WebP", "synthetic-three-frames.webp", True, False, 0, None, True),
    ("MP4 start 2 duration 8 loop", "synthetic-ball-tone-12s.mp4", True, False, 2, 8, True),
    ("Audio only", "synthetic-ball-tone-12s.ogg", True, False, 0, None, False),
    ("Embedded MP4 - click to play", "synthetic-ball-tone-12s.mp4", False, True, 2, 8, False),
    ("Missing external media", "intentionally-missing.mp4", True, False, 0, None, False),
]
pages = []
expected = []
for title, filename, auto, embedded, start, duration, repeat in cases:
    source = f"({filename})"
    if embedded:
        stream = pdf.stream((ROOT / filename).read_bytes(), "/Type /EmbeddedFile")
        source = f"{pdf.add(f'<< /Type /Filespec /F ({filename}) /EF << /F {stream} 0 R >> >>')} 0 R"
    options = f"/Start [{start * 1000} 1000] /Mode /{'Repeat' if repeat else 'Once'}"
    if duration is not None:
        options += f" /Duration [{duration * 1000} 1000]"
    annotation = pdf.add(f"<< /Type /Annot /Subtype /Movie /Rect [24 22 296 146] "
                         f"/Movie << /F {source} >> /A << {options} >> >>")
    activation = f"/AA << /O << /S /Movie /Annotation {annotation} 0 R /Operation /Play >> >>" if auto else ""
    content = pdf.stream((f"0.95 0.94 0.91 rg 0 0 320 180 re f "
                          f"0.15 0.20 0.25 rg 24 22 272 124 re f "
                          f"0.1 0.1 0.1 rg BT /F1 10 Tf 24 160 Td ({title}) Tj ET").encode())
    page = pdf.add(f"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 320 180] "
                   f"/Resources << /Font << /F1 {font} 0 R >> >> /Contents {content} 0 R "
                   f"/Annots [{annotation} 0 R] {activation} >>")
    pages.append(page)
    expected.append({"page": len(pages), "title": title, "source": filename,
                     "embedded": embedded, "autostart": auto, "start_seconds": start,
                     "duration_seconds": duration, "repeat": repeat})
pdf.objects[0] = b"<< /Type /Catalog /Pages 2 0 R >>"
pdf.objects[1] = (f"<< /Type /Pages /Count {len(pages)} /Kids ["
                  + " ".join(f"{page} 0 R" for page in pages) + "] >>").encode()
identity = pdf.save("media-smoke.pdf")
(ROOT / "media-smoke-identity.json").write_text(json.dumps({
    **identity, "pages": expected,
    "limitations": ["Authored research input, not product acceptance or Beamer output.",
                    "Helvetica is a standard PDF font; rendering is checked separately.",
                    "Original user/repository PDFs were not modified."],
}, indent=2) + "\n", encoding="utf-8")
print(json.dumps(identity))
