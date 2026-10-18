"""Create owned, visibly distinguishable reload inputs without touching source.

These inputs are for the later native smoke; generation is not acceptance.
"""
from pathlib import Path
import hashlib
import json
import pypdf
from pypdf.generic import DecodedStreamObject

ROOT = Path(__file__).resolve().parents[5]
OUTPUT = Path(__file__).resolve().parent / "inputs"
SOURCE = ROOT / "fixtures/notes-functional/pympress-notes.pdf"
EXPECTED = "e939e35f931c184fd93b2e9b95e560db73ba6a6f6dfccbb0963262f0b359dfeb"
before = hashlib.sha256(SOURCE.read_bytes()).hexdigest()
assert before == EXPECTED
OUTPUT.mkdir(exist_ok=True)
records = []
for name, color, bars in [("reload-blue.pdf", "0.05 0.35 0.95", 1),
                           ("reload-green.pdf", "0.05 0.65 0.25", 2),
                           ("reload-orange.pdf", "0.95 0.4 0.05", 3)]:
    writer = pypdf.PdfWriter(clone_from=SOURCE)
    for page in writer.pages:
        content = page.get_contents().get_data()
        marker = f"q {color} rg " + " ".join(f"{14 + index * 16} 12 12 18 re f" for index in range(bars)) + " Q"
        replacement = DecodedStreamObject()
        replacement.set_data(b"q\n" + content + b"\nQ\n" + marker.encode("ascii") + b"\n")
        page.replace_contents(replacement)
    output = OUTPUT / name
    writer.write(output)
    reader = pypdf.PdfReader(output, strict=True)
    assert len(reader.pages) == 5
    assert reader.page_labels == ["1", "2", "3", "3", "4"]
    records.append({"name": name, "bytes": output.stat().st_size,
                    "sha256": hashlib.sha256(output.read_bytes()).hexdigest(),
                    "labels": reader.page_labels, "public_marker_bars": bars,
                    "public_marker_rgb": color})
after = hashlib.sha256(SOURCE.read_bytes()).hexdigest()
assert after == before
receipt = {"source": str(SOURCE), "source_sha256_before": before,
           "source_sha256_after": after, "generator_reader": f"pypdf {pypdf.__version__}",
           "outputs": records,
           "scope": "Owned fixture preparation only; actual product/native reload NOT_RUN."}
(OUTPUT / "identities.json").write_text(json.dumps(receipt, indent=2) + "\n", encoding="utf-8")
print(json.dumps(receipt))
