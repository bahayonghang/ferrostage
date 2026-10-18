"""Check side-specific text canaries in the compiled, original Beamer PDF."""
import hashlib
import json
from datetime import datetime, timezone
from pathlib import Path

import pdfplumber
from pypdf import PdfReader

folder = Path(__file__).resolve().parent
pdf_path = folder / "pympress-notes.pdf"
public = ["FS-SLIDE-01", "FS-SLIDE-02", "FS-SLIDE-03-STEP-1", "FS-SLIDE-03-STEP-2", "FS-SLIDE-04"]
private = ["FS-NOTE-ONLY-01", "FS-NOTE-ONLY-02", "FS-NOTE-ONLY-03A", "FS-NOTE-ONLY-03B", "FS-NOTE-ONLY-04"]
reader = PdfReader(pdf_path)
assert len(reader.pages) == 5
assert reader.page_labels == ["1", "2", "3", "3", "4"], reader.page_labels
pages = []
with pdfplumber.open(pdf_path) as document:
    for index, page in enumerate(document.pages):
        assert abs(page.width / page.height - 32 / 9) < 0.001
        left = page.crop((0, 0, page.width / 2, page.height)).extract_text() or ""
        right = page.crop((page.width / 2, 0, page.width, page.height)).extract_text() or ""
        assert public[index] in left, (index, left)
        assert private[index] in right, (index, right)
        assert "FS-NOTE-ONLY" not in left, (index, left)
        assert "蓝色纸鹤" not in left
        assert all(marker not in right for marker in private if marker != private[index])
        pages.append({"physical_page": index + 1, "label": reader.page_labels[index], "width_pt": page.width, "height_pt": page.height, "left_public_marker": public[index], "right_private_marker": private[index], "audience_text_isolation": "PASS"})

def item(path):
    return {"path": str(path.relative_to(folder)).replace("\\", "/"), "bytes": path.stat().st_size, "sha256": hashlib.sha256(path.read_bytes()).hexdigest()}

result = {"utc": datetime.now(timezone.utc).isoformat(), "compiler": "XeLaTeX / TeX Live 2025, -no-shell-escape", "pdf": item(pdf_path), "source": item(folder / "pympress-notes.tex"), "pages": pages, "preview_files": [item(path) for path in sorted((folder / "preview").glob("page-*.png"))], "visual_review": "Root inspected all five final Poppler pages: readable Chinese, code, public markers and private notes, no clipping", "limits": "File/crop evidence only; actual FerroStage PDFium regression and native notes UI acceptance are separate. Task07 UI placeholders do not prove notes display."}
(folder / "verification.json").write_text(json.dumps(result, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
print(json.dumps({"pages": 5, "labels": reader.page_labels, "sha256": result["pdf"]["sha256"], "side_specific_text_canaries": "PASS"}))
