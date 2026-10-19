"""Read-only planning probe. Writes an inventory, never PDF bytes or note text."""
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path

from pypdf import PdfReader
from pypdf.generic import IndirectObject

parser = argparse.ArgumentParser()
parser.add_argument("pdf", type=Path)
parser.add_argument("output", type=Path)
args = parser.parse_args()
reader = PdfReader(args.pdf)
labels = reader.page_labels
transitions = {}
annotations = Counter()
flags = Counter()
object_flags = Counter()
errors = 0
note_stats = []
for index, page in enumerate(reader.pages):
    is_note = labels[index].startswith("notes:")
    transition = page.get("/Trans")
    if transition is not None:
        value = transition.get_object()
        key = str(value)
        entry = transitions.setdefault(key, {"dictionary": key, "physical_pages": [], "content_count": 0, "notes_count": 0})
        entry["physical_pages"].append(index + 1)
        entry["notes_count" if is_note else "content_count"] += 1
    for key in ["/AA", "/Dur"]:
        flags[key] += int(key in page)
    for ref in page.get("/Annots", []):
        annotations[str(ref.get_object().get("/Subtype"))] += 1
    if is_note:
        text = page.extract_text()
        note_stats.append({"page": index + 1, "characters": len(text), "replacement_characters": text.count("\ufffd")})
for generation, objects in reader.xref.items():
    for object_id in objects:
        if object_id == 0:
            continue
        try:
            obj = reader.get_object(IndirectObject(object_id, generation, reader))
            if isinstance(obj, dict):
                for key in ["/JS", "/OCProperties", "/RichMediaContent", "/EmbeddedFiles"]:
                    object_flags[key] += int(key in obj)
        except Exception:
            errors += 1
result = {
    "path": str(args.pdf.resolve()),
    "sha256": hashlib.sha256(args.pdf.read_bytes()).hexdigest(),
    "bytes": args.pdf.stat().st_size,
    "pages": len(reader.pages),
    "content_pages": sum(not label.startswith("notes:") for label in labels),
    "notes_pages": len(note_stats),
    "all_pairs_match": all(labels[i + 1] == "notes:" + labels[i] for i in range(0, len(labels) - 1, 2)),
    "catalog": list(reader.trailer["/Root"].keys()),
    "transitions": list(transitions.values()),
    "page_flags": dict(flags),
    "annotation_subtypes": dict(annotations),
    "object_flags": dict(object_flags),
    "xref_scan_errors": errors,
    "note_text_stats": note_stats,
    "sizes": sorted(set(str(list(page.mediabox)) for page in reader.pages)),
}
args.output.write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
print(json.dumps({"pages": result["pages"], "content": result["content_pages"], "notes": result["notes_pages"], "pairs_match": result["all_pairs_match"], "text_nonempty": sum(n["characters"] > 0 for n in note_stats), "replacement_characters": sum(n["replacement_characters"] for n in note_stats), "scan_errors": errors, "sha256": result["sha256"]}))
