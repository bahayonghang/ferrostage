# Presenter workbench and settings design optimization

## Goal

Extend the shipped home design into the loaded-PDF workbench and settings. A presenter should recognize the main action immediately, scan settings without oversized empty blocks, and understand where the application and audience PDF actually are, including changes waiting to be applied.

## Background and decisions

The user explicitly requested deep analysis and Trellis task creation using `ui` and `design-md`. Four supplied screenshots and current-source findings F1–F11 are recorded with path:line evidence in `research/visual-and-source-audit.md`. Root `DESIGN.md` already defines a customized `brand:claude` foundation; extend it and the single Rust token layer. No new catalog/brand direction is requested.

The user confirmed in this conversation that the font feature controls interface family and size only. The preceding home task's explicit user correction excludes exhaustive dimensions/DPI/non-core validation; retain core regression and normal-window native visual checks. On 2026-10-18 the user explicitly approved this plan and requested sequential Trellis sub-agent implementation followed by independent review and native core/visual verification.

## Requirements

| ID | User-visible outcome | Owning child / evidence |
| --- | --- | --- |
| R1 | Loaded-PDF filename, stage, logical page/step and timer have a clear hierarchy; actionable progress is recognizable without looking like a passive badge. | workbench, F1 |
| R2 | Start has a large play icon and explicit label on the same control row as Previous/Next. Stop replaces it in that location while presenting; no full-width separate Start row. | workbench, F2–F3 |
| R3 | Navigation, blanking, drawing modes and secondary document actions form distinct groups with coherent control shapes and explicit selected/error states. | workbench, F2 |
| R4 | All seven settings categories share a compact readable layout, quiet category navigation, sensible typography, one content scroll and truthful save/draft feedback. | settings then displays, F4–F5, F8, F10 |
| R5 | General offers installed UI font selection and size, preview/default restoration, live application and persistence. PDF, raster notes, audience content and OS titlebar stay outside this setting. | settings, F7 |
| R6 | Recent files are readable filename/path rows with count/limit and a content-sized Clear action, not raw path paragraphs or full-width commands. Preserve history order/limit/clear behavior. | settings, F6 |
| R7 | Displays provides a compact spatial overview and corresponding rows. Each detected screen indicates application/PDF/unused role; actual location, configured destination and draft are distinguishable. | displays, F8–F9, F11 |
| R8 | Actual location follows owned native windows; Ready shows configured PDF destination and “not presenting”. Apply/Cancel, Identify numbering, hotplug and missing/stale/ambiguous states remain correct. | displays, F9–F10 |
| R9 | Preserve home identity/entry, notes-priority split, manual navigation, timer/media/ink behavior, read-only user PDFs, audience isolation, themes/locales and settings compatibility. | all children |

## Acceptance criteria

| ID | Observable acceptance | Requirement / mechanism / evidence owner |
| --- | --- | --- |
| AC1 | At a normal native window the filename/state are grouped, countdown has stronger emphasis than clock, and concise page progress still opens the existing direct-jump dialog. Full logical/step/physical context remains discoverable. | R1, workbench header/progress, W1 |
| AC2 | Ready has exactly one Start with recognizable large play icon plus label beside Previous/Next; Presenting replaces it with Stop in the same strip. Disabled/busy reason is visible. No standalone full-width Start footer or orphan divider remains. | R2, unified session strip, W2 |
| AC3 | Current tool and black/white states are explicit by text/indicator; all original actions and keyboard activation work once; destructive Clear is secondary and labeled. Slide/notes content gains the removed footer space without changing configured split semantics. | R3/R9, grouped semantic controls, W3–W4 |
| AC4 | All seven categories use the shared settings row/nav/header vocabulary; no row stretches a tiny control into an empty card. Settings Displays has one content scroll, no fixed-height nested embed, and Close remains unambiguous. | R4, settings host + shared display body, S1/D1 |
| AC5 | Font and size changes update interface text while settings stays open; persist across restart; System default and size reset work. Unavailable family uses fallback with notice, retaining the saved request. Chinese/Latin text remains readable. | R5, schema + resolver + font picker, S2–S3 |
| AC6 | Font changes leave PDF/raster notes/audience pixels and OS titlebar untouched; home wordmark and timer retain their distinct font roles. Old v1 config defaults, invalid-leaf handling, unknown-key preservation, read-only and save failure/Retry pass. | R5/R9, typography boundary + settings pipeline, S3–S4 |
| AC7 | Recent section shows real filename and parent context, distinguishable same-named files, order/limit and empty state; Clear updates both settings and home history. No invented metadata or new file-management actions. | R6, history presentation, S5 |
| AC8 | In the supplied three-screen arrangement a compact diagram and all three normal assignment rows, plus role summary and Apply/Cancel, are simultaneously visible at the normal review window with diagnostics collapsed. Every box's number/role agrees with its row. | R7, bounded diagram + compact rows, D1–D2 |
| AC9 | Ready/Idle shows actual application location and configured PDF destination marked not running; Presenting shows actual PDF location. Moving an owned window updates the actual marker without silently saving a new assignment; unknown mapping is explicit. | R7/R8, observed location projection, D3 |
| AC10 | Draft edits never move actual markers or windows before Apply. Cancel leaves windows/config unchanged; successful Apply updates current markers only after confirmation. Stale/missing/ambiguous topology, disabled PDF and single-screen windowed audience remain correctly labeled. | R8, three-layer state + existing transaction, D4–D5 |
| AC11 | “Saved” refers to saved preferences; pending display allocation is explicitly pending. Errors/Retry and actionable topology warnings stay visible, while technical details are optional disclosure content. | R4/R8, local versus global feedback, S4/D4–D5 |
| AC12 | Required builds/lint/focused regressions and normal-window native visual/core interaction checks have recorded results for the final source. Native before/after is distinguished from attachments and headless tests. Home and audience lifecycle remain correct. | R9, child evidence plus final integration, parent |

The mapping W/S/D refers to the local acceptance IDs in each child PRD. Implementation verification must report failures or unrun checks honestly; these criteria are not already passed by this plan.

## Task map and scope

| Order | Task | Deliverable |
| --- | --- | --- |
| 1 | `10-18-settings-design-font` | Settings layout/typography foundation, compatible font keys, recent-file presentation, common feedback |
| 2 | `10-18-workbench-design` | Header/progress hierarchy and unified session/tool strip using that foundation |
| 3 | `10-18-displays-design` | Shared compact display body, actual/configured/draft role projection and allocation feedback |
| Final | This parent | Cross-child acceptance and requirement/evidence review |

Out of scope: PDF/notes-font editing, document/raster/media changes, new history open/pin/search/delete features, OS titlebar redesign, new framework/dependencies/vendor patches, device identity-policy changes, auto reassignment, new performance or dimensions/DPI matrices. Existing default CI and relevant native checks remain required during implementation. No commit, push or archive is authorized by the implementation request.

## Planning status and risks

All requirements have an owning child, design mechanism and acceptance path. Four tasks contain PRD/design/implementation plans and curated manifests. The user approved the latest summary; execution is authorized in the listed order. Native font resolution, window-move observation and final visual density remain implementation proof obligations until recorded in final evidence. The original planning validation receipt remains historical.
