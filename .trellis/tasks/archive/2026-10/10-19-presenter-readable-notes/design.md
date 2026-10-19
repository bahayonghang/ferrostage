# Original-source layout design

## Source of truth
Thesis root: `D:/Documents/LYH/200-Learning/00博士毕业/毕业论文/thesis`.
Current `NOTES_BLOCK` uses `\large`, `\linespread{1.25}` and 0.65em paragraph spacing. Beamer renders notes in a fixed-height page box; raising font size can overflow without producing another page. Source changes must account for the complete available height. Preserve the note-page template definition followed by the appended `notes:N` page-label hook; changing their order can erase the labels.

## Typography
Retain existing Chinese font family, dark ink on light paper, left alignment and original mathematical typesetting. Start from explicit 16pt/20pt baseline, reduced but comfortable note-only margins and approximately 0.35-0.45em paragraph gaps. Measure actual usable height and the dense notes before fixing final values. The baseline inspection found 84-282 extracted characters per Notes page (mean 202); character count is a density signal, not a layout calculation.

Do not change the whole Beamer documentclass font size, which would alter audience slides. Apply note-specific settings inside the generated template with correct TeX grouping. Reduce wasted template space first, then refine paragraph spacing and only lossless line breaks/emphasis in existing note payloads where required. No shortening of speech to force a fit. If the largest common readable size cannot fit complete notes, report the actual density constraint and revise source layout deliberately; do not add app reflow as a fallback.

## Original rendering
The layout child gives Notes the lower-right pane. Each note raster is laid out at available width with height derived from its aspect ratio; the existing vertical scroll handles overflow. Avoid equal flex-grow shrinking across multiple notes images. Rendering requests track actual visible raster needs and the existing budgets; theme/UI typography never transforms PDF pixels.

## Synchronization and scope protection
Run a sync dry-run against the current dirty baseline first. Sync can overwrite chapter bodies and styles, so compare its proposed writes before applying; preserve unrelated differences and note payload hashes. Work from canonical sources if required, not an edited copy that the next sync erases. `notes.md` and `speech.md` are not sync outputs and need no regeneration for pure typography. Keep all PDFs/preview files local as required by thesis instructions.

## Validation and rollback
Run focused sync tests, compile twice for labels, inspect all final note labels and content/notes association. Render all affected Notes at consistent scale and compare every audience content page with pre-change output. Investigate all new overfull/clipping warnings. Capture native identical-pane before/after and standing evidence. Retain baseline source hunks and PDF hash locally; rollback restores only task-owned hunks and recompiles, preserving the user's existing changes.
