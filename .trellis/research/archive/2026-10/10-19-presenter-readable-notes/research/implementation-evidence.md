# Original Notes implementation evidence

Date: 2026-10-19. Scope: external thesis repository only; no FerroStage product source changed by this worker. Native presenter and standing acceptance remain separate.

## Changed files

Repository: `D:/Documents/LYH/200-Learning/00博士毕业/毕业论文/thesis`.

- `src/thesis_analysis/defense_notes_sync.py`: durable Notes template uses 16 TeX pt type, 20 pt baseline spacing, 0.4em paragraph spacing, zero paragraph indentation and 0.5cm side margins. Minipage grouping keeps typography local; balanced side offsets preserve the outer Beamer box width. Label hook stays after the template definition.
- `slides/deck-notes/defense.tex`: synchronized generated template.
- `tests/test_defense_notes_sync.py`: actual XeLaTeX regression with Chinese density, inline mathematics, complete final sentence, no overflow, matching note labels, rendered font size and unchanged audience body size.
- `slides/AGENTS.md`: source-template and future density-check instructions.

Live repository was clean before edits. No chapter, note wording, audience style, notes.md or speech.md changes. No commit or installation performed.

## Validation

- Initial and final `uv run thesis-sync-defense-notes --dry-run`: already in sync, 9 chapters, 95 notes; actual sync applied only the generated driver.
- `uv run python -B -m unittest tests.test_defense_notes_sync -v`: 7 PASS, including two XeLaTeX regression passes.
- `uv run python -m compileall -q src`: PASS.
- `git diff --check`: PASS.
- Full `just slides-notes` builds: initial trial 130.8 seconds; final two passes 123.2 and 124.5 seconds. These are complete XeLaTeX command durations including its xdvipdfmx child; the child conversion was not separately timed. Each invocation had a 300-second subprocess deadline and completed within it.
- Final PDF: 190 physical pages; all labels exactly alternate content `N` and `notes:N`, 95 matching pairs.
- All 95 audience pages have identical RGB pixel buffers to the preserved original PDF, using the same PyMuPDF default 72dpi renderer/settings on both files.
- All 95 current source note payload SHA256 values unchanged. Extracted note text, normalized only for whitespace, matches a newly compiled reference containing the same current note payloads with the old template on all 95 pages.
- The pre-existing PDF was stale on physical Notes pages 4 and 6. Their current committed source wording was retained; this was not a typography edit. The source-equivalent old-template Notes reference avoids treating stale PDF prose as authority.
- All 95 Notes rendered at 2x and visually reviewed in 12 contact sheets. Poppler independently rendered physical pages 8, 32, 66, 156, 178 and 184; representative dense/formula output was inspected. No clipping, overlapping paragraphs or missing glyphs observed.
- Body type changed from approximately 11.9552 to 15.9402 PDF points, a 33.3% increase at equal page/pane width. Densest final text bottom is 233.8448 on a 255.12pt page, leaving at least 21.2752pt bottom clearance; left edge is approximately 14.173pt.
- A 17pt/21.25pt source-only trial produced a 9.93597pt vertical overflow and lost the final extracted character on physical page 156. The 16pt setting was retained to preserve complete text.
- Final layout warnings match the baseline's 37 warnings after normalizing shifted source line numbers. No new Notes overfull warning remains. Existing font-substitution patterns also occur at the new 16pt/math-script sizes (CMR chooses 17.28pt, Latin Modern bold italic uses slanted); rendered mathematics was inspected, without introducing a document-wide font-package change that could alter audience slides.

## Local artifacts and handoff

Final PDF: `D:/Documents/LYH/200-Learning/00博士毕业/毕业论文/thesis/slides/deck-notes/defense.pdf`.

- Bytes: 71,901,127.
- SHA256: `3f7246010bfbd5e6352d4abcdeb119c73093f693d588042aa8f49398d5d24f45`.
- Local ignored evidence directory: thesis `data/analysis/presenter-notes-20261019/`.
- `baseline.pdf`, `baseline.log`, `baseline.json`: preserved pre-change artifacts and source hashes.
- `source-baseline-notes.pdf`: current source payloads with the old template, compiled in 6.9 seconds.
- `analysis.json`, `verify.py`: page labels, same-renderer audience comparison, note text comparison, bounds and font measurements.
- `sheet-01.jpg` through `sheet-12.jpg`: complete Notes visual review.
- `note-156.png`, `note-066.png`, `note-178.png`, `note-184.png`: final dense Notes; `note-032.png`, `note-086.png`, `note-088.png`, `note-120.png`: formula examples.
- `poppler-8.png`, `poppler-32.png`, `poppler-66.png`, `poppler-156.png`, `poppler-178.png`, `poppler-184.png`: independent final renders.
- `trial-17pt.json`, `trial-17pt-note156.png`: rejected larger-size evidence.

No private PDF, extracted speech or page images were copied into the FerroStage repository. Remaining acceptance: parent must verify original raster sizing/scrolling, audience isolation, identical-pane glyph improvement and actual standing readability in the integrated native presenter. PDF rendering does not prove those runtime or human checks.
