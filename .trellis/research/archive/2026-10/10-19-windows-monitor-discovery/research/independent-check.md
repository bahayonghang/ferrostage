# Independent implementation check

Reviewed against `prd.md`, `design.md`, `implement.md`, the check manifest, backend error/quality guidelines, and frontend state/quality guidelines. Review scope includes the complete current source/catalog/documentation diff and task research evidence. No product source was edited by this reviewer.

## Findings (fixed)

- File: `crates/ferrostage-ui/src/shell.rs`
  - Issue: Apply rechecked discovery and could copy a newly unavailable topology's derived panel reason into persistent `Shell.error`. The same path copied a stale-draft reason after a healthy topology change, which survived rebuilding a valid draft.
  - Fix: Reported and coordinated with the owning UI worker. `apply_display_draft()` now returns for any live `panel.problem` without altering the unrelated operation-error owner. The panel's validation Alert continues to render the reason. Added red/green failure-during-Apply and stale-draft recovery regressions; updated the existing v8 stale-draft assertion to check its actual live owner.
- File: `crates/ferrostage-ui/src/shell.rs`
  - Issue: Real Start/Swap shortcuts could copy invalid-topology reasons into persistent operation errors even while the corresponding button was unavailable; a failed watcher-start reason could also be copied by Start.
  - Fix: Reported and coordinated with the owning UI worker. Start/Swap now return for known unavailable topology, and Start retains the independent watcher error owner. A real GPUI F5/S input regression covers unchanged configuration, no audience/pending transaction, operation-error preservation and recovery.
- File: `crates/ferrostage-ui/src/display_settings.rs`
  - Issue: The additional full-width monitor Alert pushed the embedded settings Apply footer outside its normal visible area, caught by the unchanged full GPUI regression.
  - Fix: The UI worker placed the localized wrapping Alert in the existing detection-status slot, retaining healthy monitor-count behavior and existing layout boundaries. Independently reviewed the final narrow delta: Alert semantics, localized message, Detect again and failure-specific recovery remain present; no test geometry assertion was relaxed. Full GPUI rerun passed 174/174.

## Findings (not fixed)

No unresolved P1/P2 implementation findings remain after reviewing the coordinated fixes and final detection-status layout.

The screenshot's original transient Windows trigger remains unknown. Native baseline, coverage and DPI experiments did not reproduce incomplete enumeration; the changes correctly avoid replacing native DC acquisition, accepting partial topology, or claiming a hardware cause. This is an evidence limitation, not a request to broaden this repair into a hardware/size matrix.

## Behavior traced

- Startup discovery stores a typed monitor error independently of operation errors; watcher-start errors remain owned by the watcher result and are localized when presented.
- `Unavailable` marks topology invalid and stores only the typed monitor error. The existing mapping/stop path still closes the audience, returns presentation to Ready and publishes Curtain.
- `Changed` clears only the discovery error. Recovery to the original monitor list bypasses the unchanged-topology early return because the preceding failure set `topology_valid` false. Apply's live validation and unavailable shortcuts no longer create secondary persistent owners.
- Presenter capture uses the current locale. Existing graphics/operation messages retain their precedence; the shared display panel independently exposes the monitor reason, preserving unrelated failures.
- A successful synchronous retry cannot hide a watcher-start failure or reenable presentation without a watcher. No role resolver, monitor identity, persistence schema, acquisition or automatic audience-selection policy changed.
- The shared display panel renders Chinese/English recovery instructions and dispatches the existing retry action in both full Settings and standalone Display settings. Instructions require explicit destination selection/Apply and manually restarting presentation; they do not promise automatic role assignment or reset unrelated preferences.
- `TopologyChanging` now represents differing complete before/after Device records, including metadata-only changes with identical keys. Stable device snapshots with missing geometry still return `IncompleteTopology`.
- Platform tests exercise the extracted function called by production enumeration. The existing watcher regression covers same-topology recovery. New GPUI tests inspect presenter-captured visible messages as well as shell state, audience shutdown and both hosts' retry controls; therefore they do more than assert that `Shell.error` was moved.
- Backend/frontend executable contracts and the user-facing troubleshooting instructions are synchronized with the implementation. No generated platform/template registry touch point applies to this scoped application change.

## Verification

- Reviewer `git diff --check`: PASS.
- Platform worker evidence: 31 platform tests PASS; platform clippy PASS; native three-monitor repeated-enumeration GDI check PASS (1 before, 1 after 64 production enumerations).
- UI red evidence: pre-fix same-topology recovery assertion failed with the original English error after recovery. Additional Apply, stale-draft and Start/Swap regressions failed before their fixes; final targeted monitor filter passed 9 tests (165 filtered out) after follow-up fixes.
- Lint: PASS. Root final `cargo fmt --all -- --check` and product `cargo clippy` with `gpui,pdfium`, all targets, locked/offline and denied warnings passed.
- Type-check/build: PASS via product clippy and final `just release`. Release build completed in 12.3 seconds; Setup packaging in 2.5 seconds. Root verified payload, ZIP and checksums. Setup artifact: `target/dist/FerroStage-1.0.0-windows-x64-Setup.exe`, SHA-256 `c652643c883293cf37da51e94ebadcc134e20d84590b7de6f4ce6725730a0673`.
- Tests: PASS. Root default `just ci` passed 203 tests across 18 suites; final complete GPUI suite passed 174/174, zero ignored, in 1.32 seconds. The reviewer intentionally did not duplicate coordinated Cargo jobs.
- Native: PASS for the measured scope. Root rebuilt monitor probe returned all three monitors in 10/10 samples and passed watcher/cancellation. The unchanged platform GDI regression retained 1 to 1 objects over 64 production enumerations. A separate-config rebuilt release process logged DISPLAY1/2/5, complete `mapping=Ok`, confirmed window transaction, and no monitor error/panic. Only its owned process was stopped afterward.
- Native limitations: the hidden process establishes native startup/mapping, not visual interaction or clean-exit acceptance. Physical hotplug, alternate hardware, and native UI clicking remain NOT_RUN. The original screenshot's transient trigger remains unknown. The installed running application's executable was left unchanged; generated release artifacts were not installed.
