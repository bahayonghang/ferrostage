# Implementation plan

Approved on 2026-10-18. D1 is resolved: notes-page input produces local feedback, no jump and no redirection. Execute the reviewed combined plan.

## 1. Review and activate

- [x] Resolve D1 and receive explicit approval of the combined plan.
- [x] Recheck current diff, task state, frontend specs, and source locations.
- [x] Read the relevant document/session-reconfiguration sections of `.trellis/spec/frontend/state-management.md` directly as needed. The full file exceeds the native context injection file-size limit and is intentionally not included in the manifests.
- [x] Start the task only after the planning review gate; dispatch implementation/check using the project Trellis workflow.

## 2. Make the bounded visual change

- [x] Split wall-clock rendering from presentation timing and place it first in the existing loaded header. Preserve conditional visibility and Idle behavior (AC2).
- [x] Apply the approved numeric hierarchy and semantic labels with existing fonts, scaled sizes and palettes. Keep stable numeric allocation and complete values (AC3, AC4, AC6).
- [x] Preserve existing timer calculations, warning cues, pause/resume/reset dispatch, action IDs and focus order; add only the captured label discriminator needed by the view (AC5).
- [x] Add matching zh-CN/en-US strings; synchronize `DESIGN.md` with the implemented timer role (AC6).

## 3. Simplify controls and make document defaults safe

- [x] Remove the visible Navigate toolbar entry while retaining the neutral `Tool::Navigate` and existing active-tool toggle-off path (AC8).
- [x] Reduce the layout menu to Automatic, No split and independent-note recognition. Remove its old physical-navigation checkbox, keep the underlying grouping contract, and update focus/anchor counts, radio/checkbox roles and all keyboard index arithmetic together (AC9).
- [x] Decouple detected-layout descriptions from selectable layout rows, preserving complete labels for explicit Automatic without missing-row unwraps (AC9, AC11).
- [x] Set fresh-document default layout to explicit No split while retaining `identify_independent_notes=true`. Keep Automatic explicit and same-file reload option reuse (AC10, AC11).
- [x] Update directly contradicted design/spec text and only orphaned locale keys. Preserve document crop types and internal capabilities.

## 4. Implement the mode-aware page browser

- [x] Follow `page-browser-design.md`; use a typed numbering mode, session lifetime and guarded mode-change intent, independent of `DocumentOverrides.physical_pages` (AC12, AC16).
- [x] Build one coherent entry projection for content and original-PDF modes. Use it for input resolution, target conversion, captions/counts/ranges, reveal and group movement; no page-count halving (AC13).
- [x] Place the two-option selector above the input using existing native activation/focus styling. Keep Enter in the numeric field as submit; mode switching must not navigate (AC12, AC14).
- [x] Update scheduling and surface ownership so original-PDF notes slots are marked placeholders and never requested as thumbnails. Preserve the 16-slot cap, exact-key validity and closed-grid retirement (AC15, AC16).
- [x] Apply the approved D1 notes rejection; retain range errors, stale-event guards and no implicit notes projection (AC14, AC15).

## 5. Verify the changed contracts

- [x] Add or extend focused GPUI regressions for actual clock placement/absence, Ready/Presenting/Idle visibility, and preserved controls. Check rendered element order/bounds rather than only asserting a data boolean (AC2, AC6).
- [x] Cover timing-label selection for count-up/countdown/overtime; reuse existing state-machine tests for unchanged calculations. Exercise actual pause/resume/reset input and verify wall clock independence (AC5).
- [x] Reuse `settings_v09_values_reach_workbench_timer_layout_pointer_clock_and_diagnostics`, locale catalog parity, and existing workbench/session-strip regressions. Do not write tests that merely mirror styling constants.
- [x] Run `just ci` and `just check-shell`. The latter includes the actual GPUI/PDFium build, strict lint and `test-support` UI tests. Record failures as observed; no advisory/lint waivers.
- [x] Run normal-window native Ready/Presenting and paused/resumed/reset checks in Chinese/dark, plus representative English/light header/browser inspection. Warning/overtime edge colors and clock-off were verified by focused automated tests rather than an additional physical state matrix; see verification.md (AC3-AC6).
- [x] Check 09:59 to 10:00, 99:59 to 100:00, and signed overtime readouts for stable neighbors and complete values. Compare slide/notes geometry and default header height before/after (AC3, AC4).
- [x] Focused GPUI bounds assertions establish leading clock order, stable digits at minute transitions, and unchanged header height when clock visibility changes. Actual window observations are recorded separately; no identical-viewport native pre-change pixel comparison is claimed.
- [x] Test real-input removal of Navigate, re-click-to-exit for all four remaining tools, stroke/laser cleanup, normal slide clicks and previous/next/jump (AC8).
- [x] Update existing layout-menu tests from six layout radios/eight total rows to two/three. Exercise every remaining row, no-op/failure paths, annotation warning, keyboard wrap/Home/End, dismissal and focus return. Test Automatic detecting a directional result without a rendering panic (AC9, AC11).
- [x] Add fresh-load full-page regression cases for aspect ratios formerly auto-cropped; keep explicit Automatic detector cases. Verify labeled notes mapping/privacy, plain PDF pages, explicit No split, same-file reload and new-file reset (AC10, AC11).
- [x] Extend the existing focused document suite and run `cargo test -p ferrostage-document --locked` for the changed defaults, then the existing `just ci` / `just check-shell` gates once the final integrated diff is ready. Avoid repeated full suites without new changes or failures.
- [x] In the same normal-window native smoke, confirm the shortened toolbar/menu, tool exit and default independent-note flow (AC8-AC11).
- [x] Add mapping tests for the 95/190 example, odd and nonalternating notes, multiple/orphan notes, plain PDFs and repeated-label overlay steps. Verify both numbering ranges and D1 (AC12, AC13, AC15).
- [x] Add actual input tests for mode switch, valid-target conversion, invalid-input reset, stale mode/edit/click events, keyboard focus, no navigation before submit, reopening/session reset and same-file reload (AC12, AC14, AC16).
- [x] Verify mode changes with annotations and read-only settings never dispatch document reconfiguration, reset ink/timing/grouping or save settings. Confirm PDF notes placeholders consume grid slots without note thumbnail requests; old index mappings cannot display stale content after switching (AC15, AC16).
- [x] Extend the existing real-PDF scheduling/retention regression for both modes and closed-browser cleanup, then run the integrated product gates once. Native smoke checks both mode labels, ranges, selected state and notes feedback (AC12-AC16).

## 6. Review and handoff

- [x] Check only requested surface changed; preserve content pixels and audience privacy.
- [x] Re-read the visual thesis and reject unnecessary decoration or attention-grabbing motion.
- [x] Summarize acceptance evidence, any NOT_RUN items, and changed files. Commit only with the user's authorization.

Implementation and review are authorized. Record actual verification evidence, and do not commit or push without a user request.

## Outcome

Integrated implementation, independent review, final product gates and representative native smoke completed. The user has now authorized local Chinese commits and Trellis archive/session closeout; no push is requested. `verification.md` records the observed failed concurrent native attempt, isolated retests and remaining non-matrix visual limits.
