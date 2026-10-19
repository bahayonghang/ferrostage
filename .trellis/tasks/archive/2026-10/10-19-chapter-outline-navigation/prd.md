# Two-level chapter outline navigation

## Goal

Let a presenter jump to a remembered chapter or section title inside the existing page browser, without knowing its page number. The user approved this proposal and implementation on 2026-10-19, including core functional verification. Do not commit.

## Background and Verified Evidence

- The screenshots show the current numeric/thumbnail browser and a PDF-reader outline. The missing capability is semantic navigation.
- A read-only probe finds 190 physical pages, 95 recognized notes and 83 bookmarks: 10 at depth 1, 29 at depth 2, 44 at depth 3. Retaining two levels yields 39 rows, all targeting content. See `research/pdf-outline.json`.
- Chapter 2 starts at content 12 / PDF 23; chapter 3 at content 22 / PDF 43; `3.1 引言` at content 23 / PDF 45; acknowledgements at content 95 / PDF 189. The source has `2.2` but no `2.1`; do not invent missing sections.
- Current numbering and guarded submission: `crates/ferrostage-ui/src/shell/page_browser.rs:21`, `:48`, `:64`, `:132`. Notes recognition: `crates/ferrostage-document/src/builder.rs:135`.

## Requirements

| ID | Observable requirement |
| --- | --- |
| R1 | Offer a directory inside the existing page browser in Ready and Presenting. Preserve numeric submission and thumbnails. |
| R2 | Show PDF outline depths 1 and 2 in source order, retaining unnumbered entries. Never infer depth from title text, renumber, deduplicate targets or expose deeper headings. |
| R3 | A valid title click or focused-row Enter/Space jumps exactly once and closes the browser. Scroll, tabs, focus, hover and cancelled presses never navigate. |
| R4 | Derive physical/content captions from actual content mapping. Notes, unresolved/out-of-range targets and external actions cannot navigate; show a disabled explanation rather than redirect. Invalid parents cannot hide valid children. |
| R5 | Mark the current section independently of focus and numeric target. Reveal it when entering the directory. Wrap full long titles and expose full accessible names. |
| R6 | Missing/partially malformed outlines never block PDF opening. Show an empty/partial explanation and retain other navigation. Read PDFs only; never execute actions or write the source. |
| R7 | Keep directory data presenter-owned, support themes/locales/font preferences, and exclude it from audience snapshots. |
| R8 | Replacement, reconfiguration, close, Stop/Home and stale callbacks cannot commit old targets. Stop invisible thumbnail scheduling; preserve ink, grouping, playback and notes boundaries. |

## Proposed UX for Final Review

Use `缩略图 / 章节目录` tabs below the shared title, numbering controls, editor and helper. Thumbnail mode retains density/grid/group footer. Directory mode replaces those areas with one scrollable, fully expanded two-level list and a quiet count/footer. Retain the 880px preferred panel width and contextual host. New documents default to thumbnails; remember view for the document session, independently from numbering mode.

The user approved this tabbed design on 2026-10-19. See `design.md` and the rendered layout study in `research/`.

## Acceptance Criteria

- **AC1 (R1, R2):** Sample data has exactly 39 rows: 10 top-level, 29 second-level. No depth-3 title, synthesized `2.1` or source-order change appears. Cover/contents/acknowledgements remain available.
- **AC2 (R3, R4):** `3.1 引言` reaches `PhysicalPage(44)`, content 23 / PDF 45, and closes once in Ready/Presenting. Chapter 2, chapter 3 and acknowledgements reach documented targets. All 39 sample destinations match the probe.
- **AC3 (R3, R5):** Current page, numeric target and focus remain distinct. Tab, arrows, Home/End, scroll, view/number switches, hover and cancelled presses emit zero navigation commands. Enter/Space activates one valid row. Escape/outside dismissal consumes the event and restores invoker focus.
- **AC4 (R4, R6):** Synthetic tests cover Unicode, direct/local GoTo/named destinations, invalid/missing targets, external/JavaScript actions, notes, duplicates, invalid parents/valid children, cycles and bounded traversal. Pages remain usable with missing/partially unreadable outlines.
- **AC5 (R5, R7):** At content 23, reveal/mark `3.1 引言`. Current-section rules are deterministic for duplicate/out-of-order targets. Normal-window native inspection covers wrapped titles, page alignment, focus, disabled/empty states and relevant light/dark, zh-CN/en-US states. PDF titles stay untranslated.
- **AC6 (R7, R8):** No audience directory fields. Old id/view/generation callbacks cannot commit after switch/replacement/reconfiguration. Directory view requests zero thumbnails; returning restores the <=16 visible-slot budget. Numbering, notes rejection, annotations and overlay/timing regressions stay green.
- **AC7 (all):** Implementation passes `just ci`, `just check-shell` and maintained all-feature dependency checks. Report native evidence separately. This planning delivery verifies artifacts/layout only, not product acceptance.

## Out of Scope

OCR, inferred headings, generated bookmarks, PDF edits, search, permanent sidebar, configurable depth, collapsible trees, new global shortcuts/settings/dependencies, timer/transition/layout redesign and release. Additional size/DPI/performance matrices are not blockers.

## Review Status

Evidence and proposal complete; no blocking technical unknown changes the proposed MVP. Implementation approved; task is in progress, with no commits authorized. Existing 10-19 defense-experience work is a coordination dependency, not part of this authorization.
