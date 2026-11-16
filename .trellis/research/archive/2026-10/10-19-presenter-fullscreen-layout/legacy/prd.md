# Presenter fullscreen and spatial hierarchy

Parent: [presenter-defense-experience](../10-19-presenter-defense-experience/prd.md), owns R1/R2 and integrated R5. Implementation approved on 2026-10-19 and complete. Integrated 1.1.0 gates and native samples passed; see parent research/validation.md for qualification limits.

## Goal and requirements

Start presentation with a spacious laptop console: current slide left, Next upper-right, current Notes lower-right. Fullscreen belongs to the presentation lifecycle, not merely application launch. Preserve assigned monitor roles and existing timer/navigation semantics.

## Acceptance criteria

- L1: Successful Start enters native fullscreen on the presenter monitor; assigned audience output is independently confirmed. Failed Start rolls back without leaving a half-started session or losing the usable presenter.
- L2: Stop restores the prior normal bounds/maximized/fullscreen state. Repeated Start/Stop is idempotent; starting while already fullscreen does not toggle out. Existing Escape precedence (dismiss overlays before ending a presentation) remains intact. Missing original monitor uses the existing safe presenter recovery policy.
- L3: Current/Next/Notes appear in the requested positions. Proposed fresh defaults are 60/40 horizontal and 35/65 Next/Notes vertical; existing custom ratios remain honored. Dragging the divider downward makes Next taller and Notes shorter, with persistence/restoration reflecting the same physical sizes.
- L4: Slide previews preserve full content and aspect ratio. Notes retain independent scrolling; multiple original note images use available width and natural aspect height rather than equal flex shrinking. No duplicate Next pane. Normal Ready view retains document management; Presenting uses a compact status row and a single session/navigation row, with management under More.
- L5: End, navigation, page position, timer, blackout, drawing and page grid retain working keyboard/focus/pressed behavior. Fullscreen never moves private content to the audience role or resets elapsed time/page position.
- L6: Native ordinary-window/fullscreen dual-display smoke includes success, failed start, Stop, and recovery. Headless geometry or a requested-fullscreen flag alone does not establish success.

## Out of scope

Original Notes source typography (Notes child), transition renderer (Transitions child), home redesign, exhaustive min-size/DPI matrices. Existing compact-window behavior must remain usable but is not a new redesign scope.

## Dependencies and evidence

Uses parent [findings](../10-19-presenter-defense-experience/research/findings.md). Can be implemented before the other children under the user approval of 2026-10-19; final results and qualification limits are recorded in the parent research/validation.md. Notes always remain original PDF rasters; this child owns their width-based natural-height layout and existing scrolling.
