# Original PDF Notes typography

Parent R3. User decision: original only. The historical directory slug is retained for stable task links; no extracted-text feature remains.

## Goal
Improve actual authored Notes in the thesis deck so the speaker can read them while standing, with the original PDF as the only app representation.

## Requirements and acceptance
- N1: Change the durable Notes template owned by `src/thesis_analysis/defense_notes_sync.py::NOTES_BLOCK`, synchronize to deck-notes, and compile. A second sync preserves the result; editing only generated defense.tex is insufficient.
- N2: Increase source type size, set explicit leading, useful margins and paragraph spacing. Verify short, median, dense and formula-bearing Notes; inspect every page affected by the shared template. Preserve complete wording, numbers, formulas and scientific meaning. No auto-shrink hiding overflow, clipping, missing sentences or synthetic summaries.
- N3: At identical app pane bounds, the new original PDF has demonstrably larger displayed glyphs than the baseline and usable paragraph separation. Verify the actual standing view with the final fullscreen layout. Do not equate a nominal point size with human readability.
- N4: All content-page visuals remain unchanged by note-only typography. Every `notes:N` label matches its preceding content label. For the current 95-frame no-overlay deck, preserve 95 content + 95 Notes pages; future overlay fixtures use identity-aware mapping, not parity/count assumptions.
- N5: Repeated synchronization preserves every pre-existing note by frame id. No unmatched/missing notes are accepted silently. Preserve unrelated dirty work, notes.md/speech.md and source material; paragraph-only changes must not change canonical spoken text.
- N6: FerroStage displays original note images at pane width with their natural aspect height in the existing scroll container. Multiple note pages remain ordered and readable rather than sharing one shrinking row. No text extraction, mode selector, font editor, zoom toolbox or OCR is added.

## Scope
Thesis note-template generator, generated deck-notes driver and strictly necessary note formatting/source documentation/tests. UI original image sizing belongs to layout child. Audience-slide style changes and thesis rewriting are out of scope. If a shared style is necessary, edit canonical deck/ysu-defense-modern.sty before syncing and prove audience content is unchanged.

## Dependencies
Final native acceptance follows the fullscreen/layout child. Repository constraints and current dirty-file baseline are in parent research/scope-revision.md. Product decision is resolved; implementation was approved by the user on 2026-10-19; final results and qualification limits are recorded in the parent research/validation.md.
