# Settings execution plan

- [x] Entry: approved parent summary, current diff/specs loaded, select/activate this child. Use Trellis implement/check workflow and explicit ownership; do not start another shared-file child concurrently.
- [x] Capture ordinary General/settings baseline with isolated config. Read root DESIGN and parent research; update only approved typography/settings/component contracts before implementation.
- [x] Add general optional keys, defaults, single authoritative range, leaf parse and merge, docs/example config. Verify old/invalid config behavior and unknown-key roundtrip with real loader/store tests.
- [x] Build resolved typography through the theme layer. Audit interface consumers and explicit font/size overrides; preserve role hierarchy, wordmark/mono exceptions, PDF/raster/audience exclusions.
- [x] Add installed-family selector, bounded filter/list keyboard handling, mixed CJK/Latin preview, size stepper/editor and default restoration. Apply/save via existing preference flow. Verify live update, restart, missing-family fallback and no duplicate activation.
- [x] Refactor only shared settings composition: rail, header, section/row/field groups, local restart/help, save status. Review all seven consumers, including numeric editing, recording conflicts, reset, preview and read-only state.
- [x] Render recent rows with actual basename/parent/order/count, content-sized Clear, empty and same-name/full-path handling. Preserve limit/clear and home synchronization without new actions.
- [x] Add focused meaningful regressions: parse/default/range/future-version/unknown keys/save failure; actual preference update and persistence; fonts excluded from audience/content; history clear/order. Do not snapshot the entire layout or merely assert constants.

```powershell
cargo test -p ferrostage-settings --locked
cargo test -p ferrostage-ui --locked
just ci
just check-shell
git diff --check
```

Run relevant focused GPUI filters first if diagnosing failures. After these pass, do not repeat equivalent gates until new code invalidates them. Normal-window native checks: General before/after, installed family/size/default/fallback preview, save/relaunch, history Clear, settings category navigation and one representative key/pointer/reset interaction. Representative light/dark and Chinese/English text, without size/DPI matrices.

- [x] Independent check compares S1–S6 and parent mapping against source/evidence. Record final source/executable, screenshots, tested actions and native gaps in `evidence.md`. This child cannot alone claim parent AC4's display scroll cleanup passed.
- [x] Repair actual failures, then hand the accepted shared typography/settings baseline to workbench. Rollback points: compatible-key pipeline, typography consumers, settings composition/history. No commit/push/archive without authorization.

Implementation checkpoint: see `evidence.md`. Font selector implementation and GPUI input regressions are complete; native selection/relaunch/fallback checks and independent review remain open for the integrated coordinator pass. Full CI/shell gates are consolidated there per coordinator instruction.


## Coordinated final receipt

Implementation, independent review and requested core/visual acceptance completed on 2026-10-18. The parent [verification](../10-18-workbench-settings-design/research/verification.md) is authoritative for final source/executable hashes, 194 default / 136 GPUI checks, actual screenshots and exact native limits. Native clipboard paste remains inconclusive and is not counted as passed. No commit, push or archive was performed; lifecycle status remains in progress pending that separately authorized step.
