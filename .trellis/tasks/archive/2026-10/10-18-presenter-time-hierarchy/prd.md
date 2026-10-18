# Presenter time hierarchy and control simplification

## Status and goal

Approved for implementation on 2026-10-18. The user accepted the complete combined plan and resolved D1: show 备注页不可投影 for a recognized notes-page input, with no jump or redirection.

Make presentation timing easier to identify at a glance without competing with slide content, move current local time to the upper left, remove redundant controls, make independent notes the safe default without implicit cropping, and support two clearly distinguished page-numbering modes inside the jump panel.

## Source requirements

- R1: Research online references for both time displays and propose their visual treatment.
- R2: Give presentation timing slightly stronger emphasis while keeping slides visually dominant.
- R3: Move current time to the upper left.
- R4: Deliver analysis and a Trellis task in this turn.
- R5: Remove the toolbar's standalone Navigate / 导航 button.
- R6: Remove Notes left/right/top/bottom / 备注在左侧、右侧、上方、下方 from Layout & navigation.
- R7: Default to independent notes pages, avoiding unwanted effects from page splitting.
- R8: Add these changes to the same task for combined implementation with the time-display changes.
- R9: Put a two-way jump-mode selector at the top of the existing page-browser panel: content-only numbering by default and original PDF numbering over the full page count.
- R10: Keep the mode, input label/range, thumbnails and counts consistent. The approximately half-sized content count in the supplied deck is an example, not a rule for arbitrary PDFs.

## Current evidence

The supplied screenshot shows elapsed time and wall clock adjacent to each other beside the upper-right actions. They lack persistent semantic labels. Current source already uses Consolas for both: presentation timing is 24px and wall clock 14px before the user's typography scale. Therefore, replacing the numeric font alone does not solve the hierarchy problem.

The current clock includes seconds, is controlled by `presenter.show_clock`, and uses the system local time. Presentation timing already supports count-up, countdown, negative overtime, warnings, pause/resume, and reset. Idle intentionally omits the document header.

The second supplied screenshot identifies the standalone navigation tool and four directional notes-layout rows. Navigate is the neutral internal tool state; clicking the currently active drawing/pointer tool already returns to that state. Removing the visible button must retain this exit path.

Independent notes recognition is already on by default and uses `notes:` page labels. However, the default layout is Automatic, which can infer a half-page crop from page aspect ratio. Removing directional menu rows alone would leave that implicit cropping behavior active.

The third screenshot shows 95 visible content pages, a physical total of 190, and tiles labeled PDF 1, 3, 5... . The browser currently accepts only original PDF numbers and rejects recognized notes pages. The existing physical-navigation checkbox only changes overlay grouping; it is not a numbering-mode switch and must not be repurposed silently.

## Approved user experience

- Keep the existing warm, restrained light/dark presenter theme.
- Place the current time at the leading edge of the existing header, before document filename and stage. Keep it quieter than the filename and presentation timer, with no independent card or button appearance.
- Keep presentation timing before the right-side actions. Separate it spatially from the clock and add a localized semantic label: Elapsed / 已用时, Remaining / 剩余, or Overtime / 超时 according to the existing timer mode and value.
- Increase the presentation digits moderately, with stable numeral alignment. Use neutral ink normally and the existing semantic warning colors only at existing thresholds.
- Keep the full local-time and presentation-time strings. Do not hide seconds, introduce countdown settings, or change timer semantics as part of visual polish.
- Keep pause/resume/reset available during Presenting, and make the paused state understandable without relying on color alone.
- Remove only the standalone Navigate button. Keep ordinary slide navigation and neutral tool behavior. Clicking the active laser/pen/highlighter/eraser again exits it, clears live tool state and returns to ordinary operation.
- Remove the four directional layout rows from rendering, keyboard traversal and accessibility. Keep Automatic, No split and Identify independent notes pages. Remove the confusing physical-navigation checkbox from this menu when the new local numbering selector is introduced. Retain its lower-level compatibility flag without binding the new selector to it.
- Recommend No split plus independent-notes recognition as the initial state for each newly opened document. Automatic remains an explicit opt-in choice. Existing same-document reload preserves deliberate session choices.
- Keep the existing labeled-note mapping and privacy boundary. Do not invent odd/even page pairing or notes detection from arbitrary unlabelled content.
- Add Content pages / 内容页 (default) and PDF original pages / PDF 原页码 as mutually exclusive, labeled choices above the page-number editor. Reuse existing opaque theme surfaces and selected/focus treatments; no new modal or settings screen.
- Content mode uses one-based consecutive indices over the current navigable content-page sequence, skipping recognized notes. In the example, content page 2 maps to original PDF page 3. Do not calculate this by multiplying/dividing page numbers by two. Existing overlay steps remain individual content pages; logical-slide groups and arbitrary PDF label strings are not a third numbering mode.
- PDF mode uses original one-based PDF positions and the full PDF total. Display recognized notes positions with an explicit notes marker; they do not become audience content merely because their numbers are visible.
- Switching modes changes browsing/number interpretation only. It does not navigate, reconfigure the document, clear annotations, change page grouping, change notes recognition, or write settings. Remember the mode within the document session, reset to Content for a different document, and preserve it through same-file reload.

## Scope boundaries

Scope includes loaded-presenter time displays, removing the standalone Navigate entry, reducing the existing layout menu, a safe document default, dual-mode page browsing/jumping and its bounded thumbnail scheduling, necessary locale/spec updates, and focused regressions. Preserve the slides/notes/next-slide arrangement, PDF pixels except explicitly requested crop changes, audience privacy, filename access, stage indication, remaining toolbar actions, keyboard activation, and settings persistence. No new dependencies, fonts, animation, alarms, progress rings, settings schema, or global theme migration. Do not delete internal neutral-tool, overlay-navigation or document crop capabilities merely because dedicated UI entries are removed.

## Acceptance criteria

- [x] AC1 (R1): Research records official sources, one concrete decision per reference, alternatives, and the distinction between source facts and design recommendations.
- [x] AC2 (R3): In Ready and Presenting with `show_clock=true`, exactly one full local clock appears before filename/stage at the upper-left of the header; it no longer appears in the right-side timing group. `show_clock=false` removes it without a reserved empty slot. Idle remains without the loaded header.
- [x] AC3 (R2): A normal-window native comparison demonstrates a readable but restrained presentation timer, a clear semantic label, and a quieter clock. Slides remain dominant and the header does not grow solely because of the default-size time treatment.
- [x] AC4 (R2): Digit updates do not shift neighboring controls at ordinary value transitions, including 09:59 to 10:00 and 99:59 to 100:00. Negative overtime and longer minute counts remain complete without ellipsis or clipping.
- [x] AC5 (R2): Count-up/countdown, overtime sign, warning thresholds and textual cues, pause/resume/reset, and local-time source/format retain their behavior. Pausing presentation timing does not pause the wall clock.
- [x] AC6 (R2, R3): Both catalogs and existing light/dark palettes support the changed surface; numbers respect existing UI size scaling while retaining the numeric font role. Relative order/focus behavior of remaining actions and content geometry remain intact.
- [x] AC7 (R4, R8): Expanded artifacts were validated in planning; the user explicitly approved implementation before task activation and product edits. Final implementation and verification evidence are persisted.
- [x] AC8 (R5): Navigate is absent from the toolbar and Tab/accessibility targets. Previous/next, page jump, and ordinary slide clicks retain their behavior. Every remaining drawing/pointer tool can be toggled off by reactivating it; stroke/laser state is cleared without unintended ink, navigation or stopping the presentation.
- [x] AC9 (R6, R9): No directional notes-layout row or duplicate physical-navigation checkbox is available in Layout & navigation. Its three retained rows have correct labels/checks, continuous traversal, Home/End selection and Enter/Space activation. Escape/outside dismissal, focus restoration and stale-generation guards remain intact.
- [x] AC10 (R7): A fresh document starts in No split with independent-notes recognition enabled, including aspect ratios that previously triggered automatic cropping. Content uses full-page crop; labeled notes remain presenter-only and excluded from normal/physical audience navigation. Plain PDFs keep all content pages; unlabelled notes are not guessed.
- [x] AC11 (R6, R7): Explicit Automatic still works, explicit No split restores full-page content, same-file hot reload preserves the selected options, and opening another document resets to the new default. Independent-note recognition retains its meaning, guarded reconfiguration and annotation-loss warning. No settings schema or user PDF writes are introduced.
- [x] AC12 (R9, R10): A clearly selected two-option control appears above the input in the existing panel and is keyboard/accessibility operable. Content is the fresh-document default; reopening remembers the document-session choice. Each mode updates the input's label, exact range, value, tile labels, target helper and range footer coherently.
- [x] AC13 (R9, R10): Content input n maps through the actual ordered content-page sequence; original-PDF input p maps to physical position p. The 95/190 example, plain PDFs, irregular/multiple/orphan notes and overlay steps prove neither mapping assumes odd/even pages, half totals, arbitrary page-label parsing or logical-group indices.
- [x] AC14 (R9): Typing, hover, cancelled press, mode switching and group browsing never change the committed page. Explicit valid submit/tile activation jumps once. Switching translates the selected valid target into the other numbering mode, or resets invalid input to the current content page, and prevents stale events from the previous interpretation from being applied.
- [x] AC15 (R9, R10): Original-PDF mode shows full-page positions with clearly marked, non-projectable notes entries. Entering a recognized notes-page number shows 备注页不可投影 (localized in English), disables Jump, and never redirects or changes the committed page. Notes remain outside the audience content map and are not rasterized into browser thumbnail slots. Ordinary content targets remain direct jumps.
- [x] AC16 (R9, R10): Grid entries, captions, range calculation, group movement, target reveal and thumbnail scheduling use one coherent mode-aware projection. Keep the existing maximum of 16 visible slots, request only content thumbnails in those slots, retire stale mode/generation work, and retain zero closed-browser thumbnail work. Mode changes preserve annotations, timer, document overrides, grouping and audience state, even with read-only settings.

## Verification boundary

Implementation requires focused behavioral regressions, existing product build/lint/tests, and normal-window native visual evidence. Headless tests are not evidence of visual quality. No exhaustive size/DPI/performance matrix is part of this task. Planning completion does not imply any implementation acceptance criterion has passed.

## Resolved product decision

D1, approved: entering a recognized notes-page number in original-PDF mode shows 备注页不可投影. Keep Jump disabled and the current page unchanged; never redirect to an owning content page or project notes. This applies equally to matched and orphan recognized notes pages.

The user explicitly approved this combined plan and requested implementation. Activate the task and carry out the integrated implementation and verification. No additional planning approval is needed unless the scope materially changes.

## Delivery status

Implementation and independent review are complete. See `verification.md` for actual gates, native observations and verification limits. The user accepted the implementation and requested local Chinese commits plus Trellis archive/session closeout. Preserve the existing verification scope; no push is requested.
