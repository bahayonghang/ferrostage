# Visual page browser

## Goal
Find a destination using an obvious PDF page field and real slide thumbnails, without leaving the workbench.

## Parent contract
Parent: `10-18-presenter-navigation-options`. Owns R1-R4 and its R7 obligations, contributes A1-A5 and A8-A10. Shared visual/behavior contracts: parent `design.md`.

## Requirements
- PB1: Progress, Jump shortcut and 页面浏览 action open the same contextual panel. Opening reveals current content page and focuses/selects its PDF number.
- PB2: Visible labeled numeric field supports caret/selection, replacement, Delete/Backspace, Home/End and plain-text paste. Interpret one-based PDF physical page, show range and logical/step context. Reject excluded notes pages specifically.
- PB3: Real cropped thumbnails, current badge, separate typed target/focus; one whole-tile activation path.
- PB4: Independently choose 1-4 columns and 1-4 rows, default 4x4. Changes update arrangement/capacity without a jump. Settings save through the existing store with honest feedback.
- PB5: Enter/Go or tile activates exactly once and closes. Typing, hover, cancelled press and dismissal are inert for projection.
- PB6: Previous/next group moves one capacity; wheel one row; short final group has no duplicates. Browser closes on Stop/Home/Swap/import/replacement.
- PB7: Preserve the 16-thumbnail limit, single worker, generation/map/crop freshness, legacy settings, no closed-grid work and notes isolation.

## Acceptance
1. Actual GPUI input demonstrates field focus/caret, select/replace/paste, local errors and immunity to conflicting global digit/Backspace/Space bindings (PB1, PB2).
2. On a mixed deck with notes/overlays, typing 69 shows its linked logical/step context without changing core page; submit changes page once. A private notes page is rejected without projection (PB2, PB5, PB7).
3. 2 columns x 3 rows yields up to 6 distinct real thumbnails; 4x4 up to 16. Current page, viewed anchor and keyboard focus remain well defined after changes (PB3, PB4, PB6).
4. Mouse and keyboard tile activation share the same result. Hover and press-drag-release outside do not jump. Escape closes without Stop, restores trigger focus, and outside dismissal cannot draw/click through (PB3, PB5).
5. Existing 4-column/13-page and 1-column/16-page configs open as custom without mutation. First explicit dimension edit uses parent conversion rule; save/readback preserves unknown leaves. Failed saves retain effective values + Retry; future-version config remains read-only (PB4, PB7).
6. Repeated open/resize/group/close cycles and hot replacement retain no obsolete UI thumbnails, schedule none when closed and never exceed 16 visible requests (PB6, PB7).

## Out of scope
Logical-number input mode, larger grids, new settings schema keys, new PDF renderer and all layout-menu behavior beyond the shared contextual host.

## Status
Implemented and independently checked on 2026-10-18 with the user-confirmed maximum 4x4. Native browser acceptance and combined integration are complete; see the parent `research/verification.md`. Changes remain uncommitted.
