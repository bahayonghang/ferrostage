# FerroStage 1.1.0 defense experience

## Goal
Stand in front of the laptop and clearly read the authored PDF Notes while the assigned audience display plays Beamer slides and common animation mechanisms. Complete the integrated application version as 1.1.0.

## Confirmed scope
The user explicitly rejected extracted/reflowed text, Text/Original switching and extra Notes features. Original PDF appearance is authoritative. Improve the actual LaTeX notes source/template in the thesis repository and the existing original-image pane. Initial request's repeated upper-right Next phrase means one Next pane. Evidence is in research/findings.md and research/scope-revision.md; the latter supersedes the initial text-mode proposal.

| ID | Required outcome | Owner |
|---|---|---|
| R1 | Start enters presenter fullscreen; Stop restores previous window state | ../10-19-presenter-fullscreen-layout |
| R2 | Current slide left, Next upper-right, original Notes lower-right, compact controls | ../10-19-presenter-fullscreen-layout |
| R3 | Larger, well-spaced original PDF Notes; durable source change survives sync; complete text and formulas retained | ../10-19-presenter-readable-notes |
| R4 | All 12 standard PDF transition styles and Beamer-emitted parameter variants, not only the example's three styles | ../10-19-beamer-page-transitions |
| R5 | Manual overlay stepping and authored page-duration animation work with independent Notes pages | ../10-19-beamer-overlay-timing |
| R6 | Privacy, monitor roles, hot reload, timer, budgets and interruption remain coherent | Parent integration |
| R7 | Workspace/application/distribution version is 1.1.0 after functional integration | ../10-19-release-version-1-1-0 |

## Acceptance criteria
- AC1 (R1/R2): Start reaches confirmed native presenter fullscreen and assigned audience output. Failed Start restores a usable prior session/window. Stop restores normal/maximized/fullscreen state. Requested panes and keyboard-reachable essentials remain visible without distorting slide pixels.
- AC2 (R3): All 95 existing Notes retain complete original content, correct formulas, labels and pairing; source typography is visibly larger at the same pane size. Densest and formula-bearing notes have no clipping. Repeated source synchronization retains the typography. No new reading modes, text extraction, independent font panel or zoom toolbox is added.
- AC3 (R4): Replace, Split, Blinds, Box, Wipe, Dissolve, Glitter, Fly, Push, Cover, Uncover and Fade pass authored fixture checks, including applicable direction/orientation/in-out and Fly parameters. Unrecognized/invalid metadata uses truthful immediate fallback. The three original example effects remain covered.
- AC4 (R5): pause/only/uncover/visible/onslide/alt/temporal and incremental lists retain all compiled steps; notes never enter navigation or auto-advance. Positive/zero/missing /Dur, Beamer animate/animatevalue, manual interruption and blanking behave as defined in the timing child.
- AC5 (R6): No Notes enter audience snapshots, retained frames or diagnostic payloads. No stale callback/frame survives Stop, reload or display replacement. Fullscreen and page playback do not reset or couple to the talk timer. The application only reads PDFs; the thesis compiler may regenerate its own defense.pdf under this explicit source-edit request.
- AC6 (R7): Local Cargo metadata/lock entries and generated Windows release artifacts consistently identify 1.1.0; unrelated dependency versions and settings schema versions are unchanged.
- AC7: Required code gates, thesis sync/compile checks, actual full-screen dual-display smoke and transition recordings have explicit PASS/FAIL/NOT_RUN. Source PDF inspection is not native acceptance. Standing readability is judged at the actual laptop posture, not claimed from a font number alone.

## Decisions
Use the current warm-neutral UI theme and MonitorId assignments. Fresh layout starts 60/40 horizontally, Next/Notes 35/65 vertically; honor persisted custom ratios. Original Notes are fitted to pane width with natural aspect height and existing scrolling, not divided into equally shrinking thumbnails. Stable presenter panes accompany audience transitions. Honor authored metadata without a new effects picker or transitions preference. Automatic animation has only the pause/resume affordance needed to control authored playback, independent of talk timing.

## Out of scope
Acrobat JavaScript/animate-package animategraphics or animateinline runtime, 3D/Flash, OCR/reflow/AI rewriting, new media engine, audience-content restyling, unrelated thesis changes, generic animation editor, exhaustive size/DPI certification, installation/publishing/push. Existing media behavior is preserved. A compatibility warning for known unsupported active-content mechanisms prevents silently claiming they work.

## Execution status
The user approved the complete revised plan on 2026-10-19 and requested implementation, validation, and version 1.1.0 last. Implementation, independent review and local 1.1.0 packaging are complete. Functional tasks remain in review for explicit native qualification limits and human standing-distance judgment; see research/validation.md. Original-only Notes remains the explicit decision. Installation and publication are outside the request.
