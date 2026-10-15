# Raster fixtures

`generate.py` is a deterministic, standalone developer fixture writer. It uses
only Python's standard library and writes files in this directory. No user PDF,
external presentation source, TeX compiler, or product writer is involved.
The PDF and original embedded Type3 geometric glyphs are MIT OR Apache-2.0.

`raster.pdf` has nine pages: red sentinel, 32:9 color/text/formula composition,
8:9 vertical composition, intrinsic 90-degree rotation, offset CropBox, transparent
red over the white output background, 180/270-degree rotations, and a 16:9
composition used for full-4K timing. The glyphs render E=mc with a superscript 2;
they are deterministic test outlines, not a claim of real-font fallback quality.
The fixture's PageLabels include `red`, `1`, `notes:1`, and `rotated`.

`manifest.json` records the PDF hash and embedded glyph-source hash. Regenerate
with `python fixtures/pdf/generate.py`; CI checks that committed files are unchanged.
Native tests independently assert color/orientation sentinels, alpha composition,
black glyph pixels and tightly packed BGRA. Left/right crops at 640x360 compare
to 1280x360 full pages; top/bottom crops at 640x360 compare to 640x720 full pages.
Only the outer 2 pixels of each crop are excluded; every other channel differs
by at most 1. A retained red frame must remain red after rendering other pages.

The performance example uses the same fixture at 4K with 20 warmups and 200
uncached renders per case. It separately keeps the preceding frame alive for a
full-4K case. Original samples, source/runtime hashes and hardware context are
recorded in task05 research; these synthetic CPU fixtures are not GPU acceptance
or universal performance guarantees for arbitrary PDFs.
