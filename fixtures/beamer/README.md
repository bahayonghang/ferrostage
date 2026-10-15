# Beamer and document-model fixtures

All TeX sources and synthetic PDF content here are authored for FerroStage (MIT OR Apache-2.0). No Pympress or other viewer code, assets, or sample PDFs were used.

| Fixture | Provenance and expected model |
| --- | --- |
| overlays.tex/pdf | Actual Beamer16:9, two frames, one pause: three physical pages, labels1/1/2, two logical groups. |
| notes-right.tex/pdf | Actual Beamer second-screen right: two physical pages907.087x255.118pt, slide LEFT_HALF/notes RIGHT_HALF. Actual producer labels1/1 are preserved, so automatic grouping gives one group; physical_pages restores two. |
| movie.tex/pdf | Actual multimedia command with autostart/loop/start2s/duration8s. Source is a deliberately absent external filename; tests only extract metadata. |
| independent.pdf | Explicitly authored PageLabels, multiple notes per content step, leading/mismatched orphans, nonnumeric/repeated labels and01 versus1. Last empty prefixes become absent labels in PDFium. |
| media.pdf | Hand-authored Movie/Screen/RichMedia dictionaries, scaled/unscaled/invalid times, inherited CropBox and90-degree rotation, embedded metadata bytes (not a playable movie), unsupported Flash case. |
| bottom-*.pdf, top-rotated.pdf | Independent synthetic visible-geometry expectations for8:9/2:3 and rotation. Auto cannot know top versus bottom; manual override supplies the intended side. |
| *counterexample.pdf, ordinary.pdf, tall-ordinary.pdf, mixed.pdf | Ratio false positives, ordinary16:9/portrait and mixed-size rejection. Explicit None restores full page. |

`expected.json` records declared expectations; default unit tests compare the actual media dictionaries and model edge cases. The explicit document `pdfium` test obtains owned metadata through task05's real render worker, builds all13 models and verifies actual Beamer left/right crops against equal-density full-page pixels (one-byte channel tolerance, nonempty content on both halves). No compiler is required to run tests.

Development-only generation used pdfTeX3.141592653-2.6-1.40.27 (TeX Live2025), Beamer3.72 (2025-02-04), pgfpages0.02 and multimedia0.02. Compile logs and source/PDF/tool/font SHA256 values are under task06/research; PDFs were not postprocessed to change labels. Recompile only into an ignored directory, for example `pdflatex -interaction=nonstopmode -halt-on-error -output-directory=cache/beamer-build fixtures/beamer/overlays.tex`, then copy the intended PDF. TeX timestamps/IDs make byte-identical rebuilds unnecessary; checked PDFs define the tested artifact. `python fixtures/beamer/generate.py` deterministically regenerates only synthetic PDFs and expected.json.

The actual TeX PDFs embed subset Computer Modern Sans Type1 fonts: CMSS10/CMSS12, plus CMSS9 in notes-right. They come from TeX Live's `fonts/type1/public/amsfonts/cm/` and carry the American Mathematical Society1997/2009 copyright under SIL OFL1.1. The installed AMSFonts OFL notice explicitly lists these reserved names; its document exception covers documents created using the fonts. TeX package/macros are not redistributed, no standalone font files are added, and these embedded fonts are not claimed as FerroStage-authored Type3 glyphs. The exact source-font and notice hashes and pdffonts inventory are preserved in provenance.

No native UI, font-substitution quality, media playback, timing performance, manual display actions or non-Windows runtime acceptance is implied by these fixtures.
