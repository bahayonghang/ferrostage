"""Independent read-only syntax/source check of authored research PDFs."""
from pathlib import Path
import hashlib
import json
import pypdf

root = Path(__file__).resolve().parent
reader = pypdf.PdfReader(root / "media-smoke.pdf", strict=True)
assert len(reader.pages) == 6
observed = []
for index, page in enumerate(reader.pages):
    annotations = page["/Annots"]
    assert len(annotations) == 1
    annotation = annotations[0].get_object()
    assert annotation["/Subtype"] == "/Movie"
    source = annotation["/Movie"]["/F"]
    if isinstance(source, str):
        source_kind = "external"
        filename = source
    else:
        source_kind = "embedded"
        filename = str(source["/F"])
        data = source["/EF"]["/F"].get_data()
        assert data == (root / filename).read_bytes()
    auto = "/AA" in page
    if auto:
        target = page["/AA"]["/O"]["/Annotation"]
        assert target is annotation
    observed.append({"page": index + 1, "title": page.extract_text(),
                     "source_kind": source_kind, "source": filename,
                     "autostart": auto})
assert [item["autostart"] for item in observed] == [True, True, True, True, False, True]
result = {"reader": "pypdf", "version": pypdf.__version__, "pages": observed,
          "sha256": hashlib.sha256((root / "media-smoke.pdf").read_bytes()).hexdigest(),
          "scope": "syntax and source bytes only; no FerroStage or native rendering acceptance"}
(root / "media-smoke-structure-check.json").write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
print(json.dumps(result))
