"""Original crop/privacy smoke fixtures; MIT OR Apache-2.0.

Writes only new research files beside this script. Select Notes Right in the
product for this authored PDF; it is not a Beamer output or acceptance result.
"""
from pathlib import Path
import hashlib
import json

from PIL import Image, ImageDraw, __version__ as pillow_version
from pypdf import PdfReader, PdfWriter, __version__ as pypdf_version
from pypdf.generic import (
    ArrayObject, DecodedStreamObject, DictionaryObject, FloatObject, NameObject,
    NumberObject, TextStringObject,
)

ROOT = Path(__file__).resolve().parent
gif = ROOT / "synthetic-public-private.gif"
frames = []
for left, right in [((220, 35, 30), (0, 220, 60)), ((255, 130, 0), (0, 220, 230))]:
    frame = Image.new("RGB", (160, 100), right)
    ImageDraw.Draw(frame).rectangle((0, 0, 79, 99), fill=left)
    frames.append(frame)
frames[0].save(gif, save_all=True, append_images=frames[1:],
               duration=[250, 250], loop=0, optimize=False)

writer = PdfWriter()
font = writer._add_object(DictionaryObject({
    NameObject("/Type"): NameObject("/Font"),
    NameObject("/Subtype"): NameObject("/Type1"),
    NameObject("/BaseFont"): NameObject("/Helvetica"),
}))
cases = [
    ("Public crop: red or orange only", "PRIVATE: green/cyan source half",
     gif.name, [240, 30, 400, 130], [0, 0, 0.5, 1]),
    ("Notes-only media: audio stays stopped", "PRIVATE audio annotation",
     "synthetic-ball-tone-12s.ogg", [350, 30, 600, 130], None),
    ("Public audio slot: playback allowed", "PRIVATE NOTES",
     "synthetic-ball-tone-12s.ogg", [30, 30, 250, 130], [0, 0, 1, 1]),
]
expected = []
for title, private, source, rect, source_crop in cases:
    page = writer.add_blank_page(640, 180)
    content = DecodedStreamObject()
    content.set_data((
        "0.96 0.94 0.90 rg 0 0 320 180 re f "
        "0.83 0.88 0.96 rg 320 0 320 180 re f "
        "0.25 0.25 0.25 rg "
        f"{rect[0]} {rect[1]} {rect[2]-rect[0]} {rect[3]-rect[1]} re f "
        f"0.1 0.1 0.1 rg BT /F1 10 Tf 12 160 Td ({title}) Tj ET "
        f"BT /F1 10 Tf 332 160 Td ({private}) Tj ET"
    ).encode("ascii"))
    page[NameObject("/Contents")] = writer._add_object(content)
    page[NameObject("/Resources")] = DictionaryObject({
        NameObject("/Font"): DictionaryObject({NameObject("/F1"): font}),
    })
    annotation = writer._add_object(DictionaryObject({
        NameObject("/Type"): NameObject("/Annot"),
        NameObject("/Subtype"): NameObject("/Movie"),
        NameObject("/Rect"): ArrayObject([FloatObject(x) for x in rect]),
        NameObject("/Movie"): DictionaryObject({NameObject("/F"): TextStringObject(source)}),
        NameObject("/A"): DictionaryObject({
            NameObject("/Start"): ArrayObject([NumberObject(0), NumberObject(1000)]),
            NameObject("/Mode"): NameObject("/Repeat"),
        }),
    }))
    page[NameObject("/Annots")] = ArrayObject([annotation])
    page[NameObject("/AA")] = DictionaryObject({
        NameObject("/O"): DictionaryObject({
            NameObject("/S"): NameObject("/Movie"),
            NameObject("/Annotation"): annotation,
            NameObject("/Operation"): NameObject("/Play"),
        }),
    })
    expected.append({"page": len(expected) + 1, "source": source, "rect": rect,
                     "source_crop_with_notes_right": source_crop})

pdf = ROOT / "media-crop-smoke.pdf"
with pdf.open("wb") as output:
    writer.write(output)

# Independent strict parse/readback of the serialized input, not a product test.
reader = PdfReader(pdf, strict=True)
assert len(reader.pages) == 3
for page, case in zip(reader.pages, expected, strict=True):
    annotation = page["/Annots"][0].get_object()
    assert list(annotation["/Rect"]) == case["rect"]
    assert annotation["/Movie"]["/F"] == case["source"]
    assert page["/AA"]["/O"]["/Annotation"] == annotation
    assert (ROOT / case["source"]).is_file()
with Image.open(gif) as check:
    assert check.n_frames == 2
    colors = []
    for index in range(2):
        check.seek(index)
        assert check.info["duration"] == 250
        rgb = check.convert("RGB")
        colors.append({"left": rgb.getpixel((20, 50)), "right": rgb.getpixel((140, 50))})

def identity(path):
    data = path.read_bytes()
    return {"file": path.name, "bytes": len(data), "sha256": hashlib.sha256(data).hexdigest()}

receipt = {"inputs": [identity(pdf), identity(gif)], "pages": expected,
           "animation_colors": colors, "pypdf": pypdf_version, "pillow": pillow_version,
           "structure_readback": "PASS", "product_parse_render_playback": "NOT_RUN",
           "required_document_override": "Notes Right",
           "limitations": "Authored research fixtures, not Beamer output or product acceptance."}
(ROOT / "media-crop-identity.json").write_text(json.dumps(receipt, indent=2) + "\n", encoding="utf-8")
print(json.dumps(receipt["inputs"]))
