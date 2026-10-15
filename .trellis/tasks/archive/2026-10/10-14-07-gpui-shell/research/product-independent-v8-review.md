# Independent V8 review and V9 test correction

2026-10-15 America/New_York. **Source/automated review PASS after one coverage correction.** V9 is ready for Root's native acceptance. This is not task07 delivery or Gate B acceptance. All changes remain uncommitted.

## Findings (fixed)

### F1 — Required conflict/offline input coverage stopped at pure draft validation

- File: `crates/ferrostage-ui/src/shell.rs`, `displays_v9_conflict_and_offline_draft_refuse_apply_from_real_controls`.
- Issue: V8's conflict/missing-ID assertions exercised `DisplayDraft`, while its real GPUI display cases used a single-display fixture. I-8 also requires the shared panel's conflicting selections and invalidated target to reject Apply through actual control dispatch.
- Fix: added one real GPUI TestPlatform input regression. It clicks PDF2 and application2, asserts the localized conflict, attempts the disabled Apply, then fixes the application choice and delivers a topology event removing the selected PDF target. The draft retains that target and becomes stale; Apply remains ineffective. Settings/core/window handles and on-disk bytes remain unchanged throughout. Synthetic extra monitor records only drive the list and input path; they do not establish native multi-display enumeration or placement.
- Verification: the full UI feature suite passes 63 tests (42 actual GPUI, 21 pure), including the new test. Strict all-feature/all-target Clippy, explicit type-check, native build, fmt and diff checks pass after this correction.
- Scope proof: removing only the added test block reproduces the exact V8 `shell.rs` SHA256. The other 608 source/contract inputs are unchanged. No runtime behavior, public interface, vendor source, settings schema or dependency was changed by this reviewer. `product-independent-v8-v9-source-delta.json` records this proof. V8 remains immutable; V9 has a new source/build freeze.

## Findings (not fixed)

No remaining source defect was identified in the reviewed task07 scope. Native Windows cleanup and the new panel's real hardware acceptance remain **NOT_RUN by this reviewer**, assigned to Root. The TestPlatform close regression does not emulate Windows `RevokeDragDrop`/`DestroyWindow` or the sporadic dispatcher teardown error. Do not treat successful exit or the feature suite as proof those native errors are gone.

Physical unplug/reconnect/connector/latency and actual single/zero-display scenarios remain NOT_RUN. Gate B remains INCOMPLETE; no08–11 work or Gate C/D acceptance is implied. The historical upstream HarfRust direct-lint failures remain historical failures; this review adds no waiver or vendor modification.

## Full-scope source review

- Read the supplied complete hook artifact, current child G-13/I-8 requirements/design/implementation tail, parent display-assignment plan and independent review, parent shared-contract tail, V7 diagnosis/checkpoint, current frontend specs and maintained-source boundary. The task remains the exact dispatched07 path; no task was created or switched when the isolated child pointer was empty.
- `DisplayDraft` is a presenter-owned edit. `numbered` is the single MonitorId-sorted mapping used by rows/selectors/Identify. `settings` validates revision, unique selected IDs, same-screen conflict and single-screen windowed eligibility before constructing existing `DisplaySettings` bindings. No-play/physical target use `PresenterOnly`; only explicit windowed playback uses `WindowedAudience`. UI numbering is not serialized.
- Opening/selecting/canceling does not invoke role migration or save. Open, Apply and Identify probe current topology; older queued watcher events are consumed first. A changed or unavailable topology increments the revision and invalidates existing drafts. Repeated identical complete snapshots leave drafts/pending windows intact. Unavailable and successful zero-display results have distinct panel states. Future schemas disable mutation; relevant mouse/keyboard/dispatch tests assert no change.
- Apply and compatible Swap share `assign_displays` and the existing prepare/confirm/commit/rollback/save owner. New windows remain behind Curtain until actual placement confirmation. Valid migration does not reset core page/blank/stage/timer. No-play does not Stop on preparation: after confirmation it publishes Curtain, stops/freezes the core, retires the audience and saves the committed roles. Failed creation/placement restores the old complete session without saving. Save failure keeps runtime settings and one SettingsSession error/Retry owner.
- Explicit Escape cancels the draft and Identify without window/settings mutation. Stop and assignment use the existing modal abandonment/preview cancellation path; pending callbacks are still invalidated by owner/identity. The presenter retains the shared deferred input dispatch and focus recovery behavior. UI-opening actions still confirm the current presenter is active. No second resolver, teardown policy, saved swap flag or authority was introduced.
- Identify only accepts number/name, uses the same numbering and strict mapped targets, requests nonfocused popup windows, expires after three seconds, and is closed on cancellation, topology changes, Start/Stop and assignment. It is refused while Presenting; Start closes any earlier labels before projection can begin. Audience remains snapshot-only, with no document/notes/path/settings controller input. Native focus/actual placement remains a separate assertion.
- The nine-line GPUI repair places its owning token before the platform-window field, so it drops before platform teardown; its callback holds only a Weak token. Only a dropped owner cancels delivery. The live callback retains the original update and `log_err` path. The real queued-callback red log contains the exact `window.rs:1914` failure, and the green test verifies the replacement remains active with no stale callback error.
- First-party should-close returns false and schedules removal of the original handle after returning from the callback. Existing deferred closed handling continues to distinguish current presenter/audience, pending replacement and retired IDs. Real regressions cover each case, including closing the current presenter while Presenting. No extra platform package, error-text filter or logging suppression was added.
- Older topology tests retain their safety assertions: the changed fixtures now actually contain a lost monitor or a prior nonempty topology, instead of sending an identical snapshot that correctly deduplicates. The toolbar traversal assertion now expects Displays activation, matching the requested primary-entry replacement. Locale keys are supplied in both locales; the lock delta is only the existing tracing-subscriber test dependency edge. Current frontend component/state/maintenance specs and ADR-008 already describe the authorized behavior; the test-only correction needs no further spec change.

## Verification

Independent commands are retained under `product-independent-v8-*`; no receipt was overwritten.

| Check | Result / evidence |
| --- | --- |
| Lint | PASS, V8 and final V9 all-feature/all-target app/UI `cargo clippy ... -- -D warnings` |
| TypeCheck | PASS, V8 and final V9 explicit app/UI `cargo check --all-features --all-targets --locked --offline` |
| Tests | PASS, independently rerun V8 62 and final V9 63 UI feature tests |
| Native compile | PASS, final V9 `cargo build -p ferrostage-app --features gpui,pdfium --locked --offline`; no launch |
| Formatting / whitespace | PASS, final V9 `cargo fmt --all -- --check` and `git diff --check` |
| Default workspace `just ci` | PASS, verified V8 receipt/log, 148 tests. Reused because the sole V9 change is in `cfg(all(test, feature = "test-support"))`, excluded from the default path |
| Full dependency policy | PASS, verified V8 all-feature locked/offline deny receipt/log. Exact manifests/lock/policy are unchanged in V9 |
| V8 integrity | PASS, all609 source inputs, executable/DLL/PDF/config/check records and44 new evidence files matched before the reviewer edit |
| Historical integrity | PASS, six preserved manifests have22/213/54/108/26/141 entries, all matching; V6/V7 evidence was not relabeled as V9 |
| V9 delta | PASS, only the new test block differs from V8;608 other inputs unchanged |

The first independent Clippy command failed to open the external target cache's build lock under the workspace sandbox. Its exit101/log is preserved. Approved cache access produced exit0; this was an environment failure, not a source repair. Existing dependency `harfrust::shape` dead-code output remains visible; strict first-party lint passes without waivers.

## Frozen handoff

Use `product-native-freeze-v9.json` with Root's existing isolated `v8-*.toml` inputs and distinct V9 native tags. It retains the V8 reference and explicitly records reuse of unaffected default/policy checks.

- Source/contract inputs:609; aggregate `e0ac2709f9c307ae5f1f7627486c54e985058e9d3c6de3639f87a546b302f8cf`.
- Native executable: `2b050737e90a9cf5a1966034937ffee0e83f5af5b703396889308c9cd08ffe4e`.
- V9 freeze SHA256: `89a66e4b1bd1523295f7c5bd41c5d8117ed36f9e467d2c231341a176a2211184`.
- Preserved V8 freeze SHA256: `4bd0a8840a4eeb6ca8d3be1976f8fe527eadd9c8c5851a61c4a2bf101f2a4130`.
- PDFium and PDF fixture retain their V8 hashes. All five input config hashes remain unchanged at handoff. V8's executable path now contains the separately rebuilt V9 bytes; the historical V8 binary identity is not claimed current.

Root owns source-bound native screenshots, A/C audience destinations, application migration, Identify numbering/expiry/focus, running migration/no-play, save lock/retry/read-back/restart, actual close-log checks and parent/child acceptance updates. No native process was launched, no desktop was operated, and no commit/push/archive/journal action was performed by the reviewer. All reviewer commands returned before handoff.
