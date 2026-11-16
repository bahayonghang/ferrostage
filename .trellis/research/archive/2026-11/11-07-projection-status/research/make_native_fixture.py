"""Create a self-authored three-page PDF with a slow second-page fade for native QA."""
from pathlib import Path
import sys


def make_pdf(path):
    objects = [b'', b'', b'<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>']
    kids = []
    colors = [(0.12, 0.27, 0.44), (0.18, 0.43, 0.31), (0.43, 0.24, 0.4)]
    for index, color in enumerate(colors):
        commands = f'{color[0]} {color[1]} {color[2]} rg 0 0 640 360 re f 1 1 1 rg BT /F1 42 Tf 50 180 Td (QA slide {index + 1}) Tj ET'.encode('ascii')
        objects.append(f'<< /Length {len(commands)} >>\nstream\n'.encode() + commands + b'\nendstream')
        stream_id = len(objects)
        transition = ' /Trans << /S /Fade /D 3 >>' if index == 1 else ''
        objects.append(f'<< /Type /Page /Parent 2 0 R /MediaBox [0 0 640 360] /Resources << /Font << /F1 3 0 R >> >> /Contents {stream_id} 0 R{transition} >>'.encode())
        kids.append(len(objects))
    objects[0] = b'<< /Type /Catalog /Pages 2 0 R >>'
    objects[1] = f'<< /Type /Pages /Count 3 /Kids [{" ".join(f"{n} 0 R" for n in kids)}] >>'.encode()
    out = bytearray(b'%PDF-1.7\n')
    offsets = [0]
    for number, obj in enumerate(objects, 1):
        offsets.append(len(out))
        out += f'{number} 0 obj\n'.encode() + obj + b'\nendobj\n'
    xref = len(out)
    out += f'xref\n0 {len(offsets)}\n0000000000 65535 f \n'.encode()
    for offset in offsets[1:]:
        out += f'{offset:010d} 00000 n \n'.encode()
    out += f'trailer\n<< /Size {len(offsets)} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n'.encode()
    path.write_bytes(out)


if __name__ == '__main__':
    make_pdf(Path(sys.argv[1]))
