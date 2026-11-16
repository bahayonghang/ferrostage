# Audience click to advance

Status: approved for implementation on 2026-10-18. The user approved default-enabled behavior, ignoring clicks during black/white blanking, dialogs and transitions, then explicitly requested implementation. The subsequent delivery request authorizes split commits, task archival, push, CI repair, a dev-to-main PR/merge, one manual CI run, and closure of any related resolved issues. See research/delivery-plan.md for the separately approved Linux maintenance scope.

## Goal

Allow the speaker to advance the PDF by left-clicking the audience playback window shown in user screenshot 2, with a persistent setting that can disable this interaction. Screenshot 1 is the presenter workbench and is outside the new click target.

## Background

- The audience currently renders projection snapshots and emits render/staging status, with no mouse navigation (`crates/ferrostage-ui/src/audience.rs:9`, `:16`, `:95`).
- Existing Next routes to `NextStep`, respecting Beamer steps and content-page mapping (`crates/ferrostage-ui/src/shell.rs:609`, `crates/ferrostage-core/src/state.rs:237`, `:314`).
- Current display preference comparisons initiate display assignment transactions (`crates/ferrostage-ui/src/shell/preferences.rs:426`); the new preference must apply without that transaction.

## Requirements

| ID | Requirement |
| --- | --- |
| R1 | A completed left click anywhere inside the audience client area, including letterbox margins, advances exactly one existing Next step. Mouse-down alone, right/middle clicks, or release outside the window do not advance. No double-click timing or debounce is introduced; two completed clicks may advance twice. |
| R2 | Provide one toggle in Settings > Displays, adjacent to PDF-window fullscreen, named `单击播放窗口切换下一页` / `Click playback window to advance`. Default is enabled, including old settings without this key. Changes apply immediately to the existing playback window and persist across restart. |
| R3 | Accept clicks only from the currently committed audience window, while Presenting, enabled, projection Live, not black/white blank, no modal, and not busy with load/reload/preferences/window transactions. Pending, replaced, or stopped windows cannot change the page. |
| R4 | Preserve existing Next semantics: one Beamer step rather than skipping a logical slide, notes pages excluded, last page stays put, no looping or automatic stop. Preserve existing timers, annotations/media cleanup, and raster publication paths. |
| R5 | Preserve existing settings behavior: leaf validation, unknown-key retention, schema v1, future-version read-only mode, and Unsaved/Retry feedback on save failure. Runtime value remains effective after a save failure; restart reflects the last successfully saved value. |
| R6 | Preserve the audience input boundary: projection snapshot remains its sole incoming product data; it receives no settings, controller, document, notes, or cache access. Presenter preview and tools retain current input behavior. |

## Acceptance criteria

| ID | Observable result | Requirements |
| --- | --- | --- |
| AC1 | With the setting enabled, an actual audience mouse click advances exactly once; both windows converge on the same committed page. Mouse-down alone, non-left input, outside release, and presenter-preview clicks do not activate this feature. | R1, R6 |
| AC2 | The toggle is discoverable in both locales, supports pointer and existing keyboard toggle navigation, applies without changing window handles/page/timer/display mapping, and survives settings-store readback and fresh session loading. | R2, R5 |
| AC3 | Disabled, modal, blank, Curtain, loading/reload/transaction, pending/retired-window, and non-Presenting cases leave the page unchanged. | R3 |
| AC4 | An overlay/notes fixture advances to the next content step, and the final content page does not wrap. Presenter tools, keyboard/button Next, stage and timer behavior retain their existing contracts. | R4, R6 |
| AC5 | Missing key uses the approved default; invalid type warns and falls back; explicit false roundtrips; unrelated unknown keys survive save; future settings bytes remain untouched; failed save retains the effective toggle with existing retry feedback. | R2, R5 |
| AC6 | Native playback smoke check covers focused and initially inactive audience clicks, fullscreen and existing single-monitor windowed playback, and the same live window after toggling. Unavailable native automation is reported as NOT_RUN, never PASS. | R1, R2, R3 |

## Out of scope

Presenter-preview click-to-advance; new pen/laser/media input on audience; right-click previous; wheel/touch/custom gesture mappings; PDF hyperlinks; new keyboard shortcuts; new dependency or GPUI patches; PDF writes; window-size/DPI matrices; performance claims.

## Approval

The user explicitly approved the latest behavior package and implementation plan. No blocking product decision remains. Default-enabled applies to old configurations missing this leaf. Black/white blanking, open dialogs, and transitions ignore clicks.
