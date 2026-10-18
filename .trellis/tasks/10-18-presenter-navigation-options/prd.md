# Presenter navigation and contextual options

## Goal

Let presenters find and jump to a page confidently, and adjust layout/navigation while seeing the actual workbench result. Replace the two full-content utility screens in the user's screenshots with contextual surfaces. This parent owns requirements and integration acceptance; implementation belongs to its children.

## Background and confirmed facts

- Jump renders typed digits inside prose without an input, replacing the workbench (`crates/ferrostage-ui/src/presenter.rs:835-890`).
- The separate grid already uses navigable content steps, cropped thumbnails and direct selection (`crates/ferrostage-ui/src/presenter/workbench_view.rs:978-1060`).
- Document options mixes six layout choices, two boolean actions and raw diagnostics (`crates/ferrostage-ui/src/presenter/workbench_view.rs:1062-1124`).
- Existing theme, configured sans typography and preview composition remain the foundation, as defined in root `DESIGN.md`.
- User confirmed on 2026-10-18: maximum 4 columns by 4 rows, default 4 by 4. No thumbnail-budget increase.

## Requirements and task map

| ID | User outcome | Child |
| --- | --- | --- |
| R1 | Conspicuous editable page-number field with label, range, caret/selection, ordinary numeric editing and adjacent errors | `10-18-page-browser` |
| R2 | Page input and real slide thumbnails share one contextual browser, accessible from progress, Jump shortcut and renamed grid action | `10-18-page-browser` |
| R3 | Independently select 1-4 columns and 1-4 rows, default 4x4; update layout without navigating | `10-18-page-browser` |
| R4 | Enter/Go or thumbnail activation jumps exactly once and closes; editing, hovering and dismissal never jump | `10-18-page-browser` |
| R5 | Rename Document options to Layout & navigation / 布局与导航; show upward menu above toolbar trigger with live workbench context | `10-18-layout-navigation-popover` |
| R6 | Separate six exclusive layout choices from two independent check items; apply immediately, close, and reflect effective state when reopened | `10-18-layout-navigation-popover` |
| R7 | Preserve notes isolation, settings compatibility, direct navigation, focus, presenter/audience boundaries and bounded on-demand rendering | Both |

## Scope and behavior boundaries

Includes the two contextual surfaces, their toolbar entries, localized copy, keyboard/accessibility handling and grid preference persistence. Existing slide/notes split, timer, Start/Stop, PDF pixels and tools retain their composition.

Page input remains explicitly **PDF 页码 / PDF page**, preserving existing physical-page semantics. Show logical slide/step context beside it. A physical page excluded as independent notes is rejected with a specific message; never silently interpret logical page 35 as PDF page 35. Thumbnails follow the current slide map's content steps.

Document overrides keep their session lifetime. Grid dimensions remain application preferences. Ordinary menu text uses check states and localized descriptions, not booleans or debug dumps.

## Acceptance criteria

| ID | Observable outcome | Requirements |
| --- | --- | --- |
| A1 | Workbench remains rendered beneath both surfaces; no replacement page/new native window; only one contextual surface open | R2, R5 |
| A2 | Opening browser focuses/selects current PDF number in a visible bordered field; valid edits update target context only; range/notes-only errors stay local | R1, R4, R7 |
| A3 | 2 columns x 3 rows shows up to 6 thumbnails; 4x4 shows up to 16. Dimension changes preserve committed page and visible-group anchor | R2, R3 |
| A4 | Real pointer/keyboard events prove one jump per explicit activation; hover, cancelled press, edits, dimension changes and dismissal leave projection unchanged | R4, R7 |
| A5 | Grid save/readback preserves unknown TOML leaves; save failures retain effective values and Unsaved/Retry; read-only settings do not mutate | R3, R7 |
| A6 | Menu opens upward without moving workbench; successful choices apply, close and restore focus; reopening shows one layout check and both toggle states | R5, R6 |
| A7 | Selecting current layout closes without resetting frames/annotations. Failure retains previous effective state and menu with local error. Existing annotations get truthful contextual notice | R6, R7 |
| A8 | Escape closes once without ending presentation; outside dismissal does not click through; contextual editing/navigation does not leak to global shortcuts | R4, R6, R7 |
| A9 | Closed browser schedules/retains no UI thumbnails; repeated group/dimension changes never exceed 16 visible thumbnail requests | R3, R7 |
| A10 | Default checks and GPUI/PDFium build/lint/input regressions pass; normal-window native visual/core smoke is recorded separately with relevant themes/locales | All |

## Out of scope

Whole-app redesign, new engines/dependencies, display reassignment, new page-numbering modes, changing core annotation-removal policy, global settings redesign, larger grids, exhaustive size/DPI matrices, CPU/FPS claims and PDF writes.

## Artifact status

Both children were implemented and independently checked in the approved order on 2026-10-18. Integration and normal-window native acceptance are complete; see `research/verification.md`. Changes remain uncommitted and tasks are retained for handoff.
