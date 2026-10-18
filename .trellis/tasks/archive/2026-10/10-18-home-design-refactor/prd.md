# FerroStage home identity and compact file entry

## Goal

Make the native Idle home screen an intentional file entry surface: recognizable FerroStage identity, one clear Open PDF action, and easily scanned recent files. Correct the oversized, repetitive screenshot appearance without extending this into a workbench redesign.

## Background

- The user requested a Trellis planning task using `ui`, `design-md`, and internet research. The subsequent user message on 2026-10-18 explicitly approved implementation via Trellis sub-agents and native UI verification.
- The screenshot shows competing product names, document status, two time values, two Open controls, full-width recent-file buttons, and a disabled Start footer.
- Root `DESIGN.md` already defines a customized `brand:claude` foundation, light/dark/system themes, installed fonts, 44px minimum interactive targets, and one Rust token layer. Reuse this source instead of replacing it with a catalog preset.
- Idle, Ready, and Presenting share presenter code and button interactions. Idle shows at most three saved paths. Native minimum size is 320x240 logical pixels.
- Current source/reference evidence and the original screenshot are in `research/baseline-and-references.md`.

## Requirements

| ID | Requirement |
| --- | --- |
| R1 | Give FerroStage a dedicated wordmark treatment, distinct from filenames and interface labels. Show it once in app content on Idle; simplify the native Idle title. |
| R2 | Review every home text role. Use concise Open PDF, Recent files, Settings, and Displays labels; sans interface text; explicit brand/section/filename/path/feedback hierarchy. Remove redundant instructions and status. |
| R3 | On Idle omit countdown, wall clock, No document status, duplicate toolbar Open, and disabled Start footer/reason. Preserve Ready/Presenting timer, clock preference, warnings, and Start/Stop. Visibility changes must not mutate timer/settings state. |
| R4 | Use one content-sized, left-aligned primary Open action. Recents become compact left-aligned rows with parent-path context. Preserve order, maximum of three visible recents, and import behavior. |
| R5 | Bound content width, apply consistent spacing, and support the native window range. Keep >=44x44 logical interactive targets, keyboard access, and visible focus. |
| R6 | Retain chooser/drop opening, busy guards, cancellation, recoverable import errors, topology warnings, unsaved settings/Retry, both locales, and Settings/Displays. Idle must not create an audience window. |
| R7 | Verify real native output and shared-component consumers. Preserve maintained GPUI, read-only PDFs, audience notes isolation, and current workbench/manual-navigation contracts. |

## Acceptance Criteria

| ID | Observable result |
| --- | --- |
| AC1 | Idle has one dedicated app-content FerroStage wordmark and native title `FerroStage`. No duplicate document-title fallback or welcome brand heading appears. |
| AC2 | Home text follows the role table in `design.md` with correct Chinese/English glyphs. Idle has no time values, No document badge, duplicate instructional sentence/Open action, disabled Start footer, or orphan footer divider. |
| AC3 | At 1100x760 and 1440x900 logical client sizes, the sole primary Open face is 120 to 200px wide for shipped labels, natural-width and left-aligned; no full-client-width CTA remains. |
| AC4 | Recent rows occupy the bounded content column; filename and parent context align left. Default-scale row targets are 48 to 56px high with <=8px gaps and no permanent command-button outline. Same-named files are distinguishable by path; full paths are discoverable through focus/hover and accessibility text. |
| AC5 | With 0/1/3 recents, valid files use existing import behavior, busy/loading prevents duplicate opens, and a missing/invalid recent reports a recoverable error. No fabricated dates, page counts, thumbnails or nonexistent shortcuts appear. |
| AC6 | Tab/Shift+Tab follow visible enabled controls; Enter/Space invoke once; hidden controls cannot receive focus. Settings/Displays restore focus to a visible control or existing root fallback. Pointer press gives immediate feedback; keyboard activation adds no animation. |
| AC7 | The matrix in `implement.md` passes without horizontal overflow or clipped essential labels/focus rings. At 320x240 home scrolls vertically. Targets remain >=44x44; normal text contrast >=4.5:1, large text/focus/essential boundaries >=3:1 in both themes. |
| AC8 | Errors, warnings, busy/loading and unsaved/save-error Retry stay visible/actionable. Chooser cancellation returns to usable home. Drag/drop/global Open retain guards. |
| AC9 | Ready/Presenting retain workbench geometry, notes priority, direct manual navigation, timer/clock preferences, Start/Stop and audience lifecycle. Home return preserves recents. Shared consumers do not acquire unintended styling/dimensions. |
| AC10 | Default CI and native-feature build/lint/GPUI regressions pass. Native before/after screenshots and geometry include source/executable identity, client size, scale, locale, theme and state. Headless checks/documents are not native visual acceptance. |

## Scope Boundaries

In scope: Idle composition, dedicated brand typography, home copy/hierarchy, compact actions/file rows, home-specific title/visibility, necessary locale/theme/component extensions, regressions and native visual acceptance.

Out of scope: whole-app rebranding; custom OS titlebar; logo/icon redesign; font/asset downloads; new framework/dependency; history search/pinning/clearing/expansion; thumbnails; PDF/notes/media changes; timer logic; settings schema; raising minimum size; a mobile/web product.

## Decisions and Planning Status

- Proposed direction: warm editorial utility, current brand foundation with a dedicated FerroStage wordmark. Other catalog directions and tradeoffs are research only.
- One cohesive task: brand, hierarchy, visibility, layout and shared-control safety require joint acceptance; child tasks would create artificial independent completion points.
- `design.md` and `implement.md` are proposed contracts, subject to user review. The user subsequently approved the complete plan; task is now `in_progress`.
- No unresolved technical question blocks the plan. Direction/visibility changes were approved; native visual acceptance remains an independent gate.

## User scope correction, 2026-10-18

The user explicitly directed: do not validate dimensions or similar non-core behavior, and persist this preference globally. Stop further native size/breakpoint/minimum-window/DPI matrices. Their unexecuted cases are no longer blockers for this task. Existing passing regression evidence is retained without more matrix expansion. Continue only core behavior, normal-window visual smoke and necessary build/lint/relevant regressions. No exhaustive accessibility or physical-display qualification is required for this home polish task. The unrelated existing all-features dependency-policy failure remains truthfully reported separately; do not add waivers or expand this UI task into dependency migration.
