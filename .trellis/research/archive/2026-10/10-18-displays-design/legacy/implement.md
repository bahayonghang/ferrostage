# Display optimization execution plan

- [x] Entry: approved parent summary, preceding settings/workbench checkpoints accepted, current task activated and diff/specs loaded. Use required Trellis implement/check roles with bounded ownership.
- [x] Capture the ordinary three-screen Ready baseline and actual Settings/standalone Ready/Presenting after states; no pre-change Presenting capture was retained. Update root DESIGN display-assignment/scroll/state contracts before code.
- [x] Add actual-window observations at creation/bounds change, handle lifecycle guards and current-mapping resolution in Shell. Publish Presenter-only actual/configured/draft projection. Verify move notification empirically before relying on it; no render-time Shell reads.
- [x] Reuse projection in diagram, role summary and rows, including unknown/not-running/disabled/single-screen cases. Keep stable numbers and explicit text indicators. Do not persist observed OS moves.
- [x] Extract minimal shared display body; settings owns one scroll, standalone owns one scroll. Remove fixed embed and full-width aspect-height trap. Fit topology in bounded region beside rows; default diagnostics collapsed. Keep Identify/Refresh and primary warning paths visible.
- [x] Refine role controls and local pending/Apply/Cancel feedback. Preserve existing preflight/confirmation/rollback, topology revision invalidation and separately autosaved fullscreen preferences.
- [x] Add/extend focused tests for the real projection and lifecycle: Ready vs Presenting, manual move, pending versus actual, successful/failed Apply, Cancel, stale revision, retired-window event, ambiguous/missing mapping, single-monitor windowed and Stop/Home clearing. Use existing topology fixtures; no physical unplug/replug qualification.
- [x] Retain topology coordinate/numbering/persistence regressions and adjust only obsolete geometry assertions. Check settings-host and standalone-host behavior, not only a pure rectangle helper.

```powershell
cargo test -p ferrostage-ui --locked
just ci
just check-shell
git diff --check
```

Native core proof: both display hosts, current three-screen diagram plus all rows/summary/actions in the normal review view, draft selection/Cancel, valid Apply, Start/Stop and manual window move. Check marker agreement on each screen, preference-versus-draft feedback and preserved warnings. Record normal source/executable-bound screenshots. Do not add size/DPI/performance or physical hotplug matrices; fixtures cover stale/missing behavior.

- [x] Independent check maps D1–D6 to parent criteria and reviews observed versus configured authority, single-scroll structure and audience isolation. Source tests do not substitute for native movement/visual proof.
- [x] Store final evidence/gaps in `evidence.md`, then run parent integration gate. Roll back only this child's coherent body/observation/feedback diff if needed. No commit/push/archive without authorization.


## Coordinated final receipt

Implementation, independent review and requested core/visual acceptance completed on 2026-10-18. The parent [verification](../10-18-workbench-settings-design/research/verification.md) is authoritative for final source/executable hashes, 194 default / 136 GPUI checks, actual screenshots and exact native limits. Native clipboard paste remains inconclusive and is not counted as passed. No commit, push or archive was performed; lifecycle status remains in progress pending that separately authorized step.
