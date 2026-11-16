"""Read-only fixture probe; bundled pypdf is not a product dependency."""
import hashlib
import json
import sys
from collections import Counter
from pathlib import Path

from pypdf import PdfReader

sys.stdout.reconfigure(encoding="utf-8")
source = Path(sys.argv[1])
reader = PdfReader(source)
labels = reader.page_labels
content_pages = []
notes_pages = []
previous_label = None
for index, label in enumerate(labels):
    if label.startswith("notes:"):
        assert label[6:] == previous_label
        notes_pages.append(index)
    else:
        content_pages.append(index)
        previous_label = label
content_number = {page: index + 1 for index, page in enumerate(content_pages)}
counts = Counter()
retained = []
all_entries = []


def walk(items, depth=1, parent=None):
    last = parent
    for item in items:
        if isinstance(item, list):
            walk(item, depth + 1, last)
            continue
        counts[depth] += 1
        page = reader.get_destination_page_number(item)
        last = len(all_entries)
        row = {
            "source_id": last,
            "parent_id": parent,
            "depth": depth,
            "title": item.title,
            "physical_page_zero_based": page,
            "pdf_page": page + 1 if page >= 0 else None,
            "content_number": content_number.get(page),
            "label": labels[page] if page >= 0 else None,
        }
        all_entries.append(row)
        if depth <= 2:
            retained.append(row)


walk(reader.outline)
with source.open("rb") as stream:
    digest = hashlib.file_digest(stream, "sha256").hexdigest()
result = {
    "source_path": str(source),
    "source_bytes": source.stat().st_size,
    "sha256": digest,
    "physical_pages": len(reader.pages),
    "content_pages": len(content_pages),
    "recognized_note_pages": len(notes_pages),
    "outline_total": len(all_entries),
    "outline_depth_counts": dict(counts),
    "retained_count": len(retained),
    "all_retained_targets_content": all(r["content_number"] is not None for r in retained),
    "named_destinations": len(reader.named_destinations),
    "label_sample": labels[:8],
    "retained": retained,
    "excluded_depth3_examples": [r for r in all_entries if r["depth"] == 3][:8],
}
if len(sys.argv) > 2:
    Path(sys.argv[2]).write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
print(json.dumps({k: v for k, v in result.items() if k not in ("retained", "excluded_depth3_examples", "source_path")}, ensure_ascii=False, indent=2))
