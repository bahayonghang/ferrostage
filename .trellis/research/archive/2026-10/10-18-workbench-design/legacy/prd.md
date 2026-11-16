# Loaded-PDF workbench hierarchy and session toolbar

## Goal

Make the loaded-PDF screen a clear presenter workspace: one visible session action, understandable progress and grouped controls, with the slides and notes retaining priority. Parent `10-18-workbench-settings-design` R1–R3/R9 and AC1–AC3/AC12 apply.

## Requirements

- Extend the approved home/settings design foundation, not a new brand. Resolve the supplied screenshot's competing outlines, pill tools, unbalanced header and separate Start footer.
- Put a large play icon and explicit Start label beside Previous/Next. During presentation, Stop replaces Start at the same location.
- Improve filename/stage/progress/timer hierarchy and tool/blanking grouping without hiding capabilities or changing action semantics.
- Preserve note priority, split values, manual navigation, original document geometry, audience lifecycle and busy/disabled guards.

## Acceptance Criteria

| ID | Observable result / requirement mapping |
| --- | --- |
| W1 | Filename/stage have a shared visual anchor; countdown is stronger than wall clock. Compact progress remains an actionable page-jump control, with complete logical/step/physical information discoverable. Parent R1/AC1. |
| W2 | Exactly one Start has a recognizable large play icon and label in the Previous/Next row. Stop replaces it there during Presenting; busy/disabled reason remains available. No separate full-width session footer or orphan divider. Parent R2/AC2. |
| W3 | Navigation, blanking, tools and secondary actions are visually grouped. Current tool and current black/white blanking have textual/icon state indicators. All original visible actions and shortcuts still work once. Parent R3/AC3. |
| W4 | Current slide and notes retain aspect/letterboxing, note-priority ratio, direct page selection, timer/clock settings, media/ink semantics and Start/Stop/Home lifecycle. The removed footer gives space back to the workspace. Parent R9/AC3/AC12. |
| W5 | Native normal-window before/after and core interaction evidence plus required checks are recorded. Idle retains the shipped home layout; shared typography is respected. Parent R9/AC12. |

## Notes

- User approved the parent plan on 2026-10-18. This child is now activated after the settings implementation checkpoint and its focused checks passed; final independent review/native acceptance remain pending integration.
- Out of scope: content/raster, notes behavior, timer logic, automatic page turns, new shortcuts/tool functions, home redesign, custom titlebar and exhaustive size/DPI testing.
