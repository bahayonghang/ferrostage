"""Pad an owned reload fixture so 64KiB/50ms writes remain observable.

The inert catalog stream does not alter pages or render content. This script
never opens an original user/repository PDF for writing.
"""
from pathlib import Path
import hashlib
import json

from pypdf import PdfReader, PdfWriter
from pypdf.generic import DecodedStreamObject, NameObject

root = Path(__file__).resolve().parent / "inputs"
source = root / "reload-green.pdf"
before = source.read_bytes()
assert hashlib.sha256(before).hexdigest() == "2d1571176a037b5ad526a4c91c742014aaf9fa3856986f8d810871b411110a50"
reader = PdfReader(source, strict=True)
writer = PdfWriter(clone_from=reader)
padding = DecodedStreamObject()
padding.set_data(b"0" * (4 * 1024 * 1024))
writer.root_object[NameObject("/FerroStageSmokePadding")] = writer._add_object(padding)
target = root / "reload-green-chunked.pdf"
with target.open("wb") as output:
    writer.write(output)
check = PdfReader(target, strict=True)
assert len(check.pages) == 5 and check.page_labels == reader.page_labels
for original, copied in zip(reader.pages, check.pages, strict=True):
    assert original.get_contents().get_data() == copied.get_contents().get_data()
assert source.read_bytes() == before
data = target.read_bytes()
receipt = {"file": target.name, "bytes": len(data),
           "sha256": hashlib.sha256(data).hexdigest(),
           "chunk_bytes": 65536, "delay_per_chunk_ms": 50,
           "chunks": (len(data) + 65535) // 65536,
           "source_unchanged": True, "pages_and_content_readback": "PASS",
           "product_parse_render_reload": "NOT_RUN"}
(root / "chunked-identity.json").write_text(json.dumps(receipt, indent=2) + "\n", encoding="utf-8")
print(json.dumps(receipt))
