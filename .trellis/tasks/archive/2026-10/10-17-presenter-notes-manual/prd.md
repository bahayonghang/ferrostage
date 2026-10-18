# Presenter notes-first and manual navigation

## Goal

Make FerroStage comfortable for reading speaker notes during a manually controlled presentation. Opening a PDF leaves it ready; neither time passing nor timer expiry advances slides. Improve functional control recognition and reduce unnecessary runtime work and console output.

## Background and decisions

- The user supplied a native screenshot and just-dev log on 2026-10-17. Next slide currently sits above small notes, a permanent thumbnail strip and recent-file rows consume workbench height, and most controls look identical.
- The user authorized task creation, planning, and subsequent implementation, then explicitly chose to remove manual select-preview-commit: choosing a thumbnail or submitting a page number jumps directly.
- Existing root DESIGN.md uses brand:claude provenance with accessible clay actions, warm neutrals and system fonts. Extend that source.
- Inspection has not established an automatic advance mechanism. Import loads Ready; the shell poller publishes state; page changes use explicit intents. The reported automatic behavior remains a runtime reproduction target, not a proven timer defect.

## Requirements and acceptance

| ID | Requirement | Observable acceptance |
| --- | --- | --- |
| R1 | Open stays Ready; Start explicitly begins presentation. Keep useful timer/clock controls. | Import creates no audience and starts no timer. Repeated ticks in Ready, and Presenting across countdown zero/overtime, leave the page unchanged. Explicit Next/Previous still work. |
| R2 | Remove manual preview/commit workflow. | A grid click navigates once immediately. Page dialog typing does not navigate; valid Enter/Go submits once, invalid input and Escape leave the page unchanged. No preview banner/confirm/cancel-preview controls or hidden preview routing remain. Ready/Presenting stage is preserved. |
| R3 | Current slide left, large current notes upper right, next slide lower right. | Normal-size native captures show notes initially about two thirds of the right column. Old defaults cannot leave notes tiny or below next slide. Divider dragging follows the new geometry and persists. Notes follow navigation and remain scrollable. No permanent thumbnail strip or recent-file rows in the workbench; recent files remain on Idle, grid is on demand. |
| R4 | Distinguish functional controls by shape, fill/border and label/state. | Start/stop, navigation, blanking, drawing tools, clear and utilities are recognizable. Selection is not color-only. Hover/press/focus/disabled behavior works in both themes; Tab and Enter/Space operate enabled controls. Text contrast >=4.5:1, focus/UI >=3:1. |
| R5 | Quiet runtime logging by default. | Default startup/open/start/stop emits no routine INFO/DEBUG app/dependency logs; genuine warnings/errors remain. Explicit RUST_LOG restores diagnostics. Compiler/build-tool output is not application logging. |
| R6 | Remove hidden thumbnail work and reduce measured redundant refresh. | Grid closed requests zero thumbnails even with legacy show_thumbnail_strip=true. Opening bounds requests, closing releases obsolete frames; unchanged scheduling submits no duplicate raster jobs. Record before/after request counts and retained bytes for the 100-page fixture. If snapshot suppression changes, unchanged snapshots stay quiet while frame/stroke/blank/media/staging updates remain visible. Do not infer latency/FPS gains from counters. |
| R7 | Behavior tests and native evidence. | Default and native-feature build/lint/test/license gates pass. Real notes fixture checks cover notes freshness/privacy, direct jump, time stability, light/dark, supported sizes and control states. Missing native measurements are explicitly NOT_RUN; compile is not visual acceptance. |

## Out of scope

No GPUI pin/family changes, dependency upgrades, audience notes exposure, PDF writes, media decoder changes, global animation system, or mobile product. Do not remove embedded-media playback under the slide-navigation request. Retain on-demand navigation grid. Do not mutate actual user config during development or add a new switch for a removed feature.

## Status

User approved the final plan and implementation, then accepted the result and explicitly waived further visual checks on 2026-10-17. Implementation removes the confirmed thumbnail-hover navigation path; no timer-driven advance was found. Automated verification and independent review receipts are under research/. Native screenshot cases remain recorded as NOT_RUN rather than PASS. The existing full-feature advisory-policy blocker remains open; do not archive as fully verified.
