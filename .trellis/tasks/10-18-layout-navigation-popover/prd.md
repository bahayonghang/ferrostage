# Layout and navigation upward menu

## Goal
Change slide/notes layout or navigation, then immediately see the effect on the workbench.

## Parent and dependency
Parent: `10-18-presenter-navigation-options`. Own R5-R6 and R7 obligations, contribute A1 and A6-A10. Depends on page-browser child's shared contextual host; tree position alone does not establish this dependency.

## Requirements
- LN1: Rename 文档选项 to 布局与导航 / Layout & navigation and open upward from its toolbar trigger, retaining live previews.
- LN2: Six exclusive layout rows; independent recognition and physical-navigation check items. A check reflects effective requested choice, Auto also shows detected outcome.
- LN3: Every successful change immediately applies and closes; reopening reflects resulting state. Same-layout selection closes without any reconfiguration.
- LN4: Reconfiguration failure preserves old state and keeps menu/error visible. Existing annotations receive the truthful clear-on-change notice.
- LN5: Pointer/keyboard/AX share selection behavior, menu navigation cannot advance presentation, outside/Escape dismiss safely and restore trigger focus.
- LN6: Use localized user language, remove true/false and internal debug dump, preserve real warnings. Preserve notes isolation and session-only override lifetime.

## Acceptance
1. At a normal window, menu opens above the trigger, does not reflow previews, and remains inside content bounds (LN1).
2. Select each layout by pointer/keyboard: applies immediately, closes, one correct check on reopen; Auto requested choice remains checked with truthful detected outcome (LN2, LN3).
3. Toggle each independent item twice: on/check and off/no-check, close after each; physical mode never exposes a still-excluded notes page (LN2, LN3, LN6).
4. Same-layout action with existing annotations changes no core/frame/annotation state. Actual change preserves current core clearing policy and displays the notice before activation. Induced failure leaves old effective checks and local error (LN3, LN4).
5. Escape does not Stop; outside click never reaches drawing/slide beneath; Enter/Space and arrow keys do not leak to global commands; invoker focus and browser/menu exclusivity hold (LN5).
6. zh-CN/en-US and light/dark use existing typography/tokens; no raw debug/boolean copy or new persistence; native observations and automated evidence are distinct (LN6).

## Out of scope
General presentation settings, display assignment, PDF writes, override persistence, changed annotation policy and page-browser functionality.

## Status
Implemented after the page-browser child and independently checked on 2026-10-18. Shared-host integration and normal-window native menu acceptance are complete; see the parent `research/verification.md`. Changes remain uncommitted.
