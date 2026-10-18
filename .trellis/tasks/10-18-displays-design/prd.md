# Compact display topology and actual window-role feedback

## Goal

Let the presenter understand all detected displays and application/PDF allocation at a glance, without oversized topology, repeated diagnostics or misleading pending-state labels. Parent `10-18-workbench-settings-design` R4/R7–R9 and AC4/AC8–AC12 apply.

## Requirements

- Reuse a compact display body in Settings and the standalone Displays dialog. Preserve one scroll per host and existing shared numbering/Identify behavior.
- Every detected screen shows its number and application/PDF/unused role with matching corresponding rows. Distinguish actual native window placement, configured destination and pending draft.
- Ready/Idle has no audience window; show PDF destination as not presenting. Changes to selection remain draft until Apply; Cancel never mutates effective allocation.
- Move routine identity/scale-source details into disclosure while retaining warnings and recovery actions visibly.
- Preserve MonitorId/MonitorRole, existing validation, single-screen mode and window transaction/lifecycle behavior.

## Acceptance Criteria

| ID | Observable result / requirement mapping |
| --- | --- |
| D1 | Settings removes the fixed 650/450px nested embed and second content scroll. Both display hosts use the same compact body. Normal three-screen review shows overview, role summary, all three assignment rows and Apply/Cancel together with diagnostics collapsed. Parent R4/R7/AC4/AC8. |
| D2 | Every schematic rectangle has a readable number and role marker agreeing with its row; orientation/relative layout are retained, unused screen is explicit. Identify numbers match and are not mislabeled Windows numbers. Parent R7/AC8. |
| D3 | Ready/Idle shows actual application monitor and configured PDF target as not running; Presenting shows actual application/PDF monitors. OS window moves update actual labels without saving assignments. Mapping ambiguity/unavailability says unknown. Parent R7/R8/AC9. |
| D4 | Pending selection is explicitly pending, does not move actual markers/windows and does not show as committed by Saved. Cancel discards only allocation draft. Successful Apply updates actual labels after native window confirmation; failure leaves existing transaction/error recovery intact. Parent R4/R8/AC10–AC11. |
| D5 | Stale revision, missing/duplicate targets, unavailable topology, no PDF target and single-monitor windowed audience remain safe and truthfully labeled. Optional details do not hide actionable warnings or Retry. Stop/Home clears audience actual location. Parent R8/R9/AC10–AC12. |
| D6 | Focused regressions plus normal-window native diagram/selection/Cancel/Apply/move/Start/Stop evidence are recorded for final source. No notes/settings controller leaks to audience; default/native gates pass or precise gaps are reported. Parent R9/AC12. |

## Notes

- User approved the parent plan on 2026-10-18. Activated after settings and workbench implementation checkpoints passed; their shared theme, typography, buttons and Presenter composition must be preserved. Independent review/native acceptance follow integration.
- Out of scope: changing monitor identity/heuristics, Windows display configuration, drag-to-reassign, new auto-swap/reassignment, OS DPI measurement, physical unplug/replug matrix, core/document/raster changes and new dependencies.
