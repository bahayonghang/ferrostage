"""Clean-room standard PDF transition fixtures. MIT OR Apache-2.0.

No third-party presenter source or private deck content. Writes only these two
named fixture PDFs; native variants last 2 seconds for recording.
"""
from pathlib import Path

ROOT = Path(__file__).resolve().parent
VARIANTS = [("R", ""), ("Fade", ""), ("Dissolve", "")]
VARIANTS += [("Split", f"/Dm /{d} /M /{m}") for d in ("H", "V") for m in ("I", "O")]
VARIANTS += [("Blinds", f"/Dm /{d}") for d in ("H", "V")]
VARIANTS += [("Box", f"/M /{m}") for m in ("I", "O")]
VARIANTS += [(s, f"/Di {d}") for s in ("Wipe", "Push", "Cover", "Uncover") for d in (0, 90, 180, 270)]
VARIANTS += [("Glitter", f"/Di {d}") for d in (0, 270, 315)]
VARIANTS += [("Fly", f"/Di {d} /M /{m} /SS {scale} /B {opaque}")
             for d, scale in (("0", "1"), ("180", "0.3"), ("/None", "0.3"))
             for m in ("I", "O") for opaque in ("false", "true")]
# Append enlarged Fly cases without renumbering the original coverage.
VARIANTS += [("Fly", f"/Di {d} /M /{m} /SS 3 /B false")
             for d in (0, 90, 180, 270) for m in ("I", "O")]


def generate(name, duration):
    objects = [b"", b"", b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>"]

    def obj(value):
        objects.append(value.encode("ascii") if isinstance(value, str) else value)
        return len(objects)

    kids = []
    for index, (style, parameters) in enumerate([("R", "")] + VARIANTS):
        title = f"{index:02d} {style} {parameters}".replace("/", "")
        # Shared grid/background and two disjoint changed rectangles expose Fly B.
        color = "0.05 0.35 0.8" if index % 2 else "0.8 0.2 0.05"
        x = 80 if index % 2 else 360
        content = ("0.96 0.96 0.94 rg 0 0 640 360 re f\n"
                   "0.72 0.72 0.72 RG 1 w\n" +
                   "".join(f"{i} 0 m {i} 360 l S\n" for i in range(0, 641, 40)) +
                   "".join(f"0 {i} m 640 {i} l S\n" for i in range(0, 361, 40)) +
                   f"{color} rg {x} 80 140 70 re f {x} 200 140 70 re f\n"
                   f"0 0 0 rg BT /F1 14 Tf 18 332 Td ({title}) Tj ET\n"
                   "BT /F1 12 Tf 18 16 Td (Shared grid stays still during Fly; public fixture only) Tj ET\n").encode("ascii")
        stream = obj(f"<< /Length {len(content)} >>\nstream\n".encode() + content + b"endstream")
        transition = obj(f"<< /S /{style} /D {duration} {parameters} >>")
        kids.append(obj(f"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 640 360] /Resources << /Font << /F1 3 0 R >> >> /Contents {stream} 0 R /Trans {transition} 0 R >>"))
    objects[0] = b"<< /Type /Catalog /Pages 2 0 R >>"
    objects[1] = f'<< /Type /Pages /Count {len(kids)} /Kids [{" ".join(f"{k} 0 R" for k in kids)}] >>'.encode()
    data = bytearray(b"%PDF-1.7\n")
    offsets = [0]
    for index, value in enumerate(objects, 1):
        offsets.append(len(data))
        data += f"{index} 0 obj\n".encode() + value + b"\nendobj\n"
    xref = len(data)
    data += f"xref\n0 {len(offsets)}\n0000000000 65535 f \n".encode()
    for offset in offsets[1:]:
        data += f"{offset:010d} 00000 n \n".encode()
    data += f"trailer\n<< /Size {len(offsets)} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n".encode()
    (ROOT / name).write_bytes(data)


if __name__ == "__main__":
    generate("transitions.pdf", "0.1")
    generate("transitions-native.pdf", "2")
