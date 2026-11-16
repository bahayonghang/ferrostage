# Bug analysis: platform callbacks outlive their owning GPUI Window

Status: F12 is native-confirmed. The executable V14 red/green regression is complete; full product checks, independent review and native outcomes remain separate. This analysis does not assert native repair PASS.

## 1. Root cause category

- B — cross-layer contract: a retained native platform window/callback is not proof that the owning GPUI Window is still registered. Shell foreground-task ownership and GPUI platform callback lifetime are different boundaries.
- C — propagation gap: the V8 lifetime correction covered active-status callbacks. The V13 failure names the unguarded hover callback at `vendor/gpui/src/window.rs:1943`.
- D — coverage gap: four clean V13 closures did not exercise every late callback ordering. A fifth, small light Displays close failed after the poll-cancel log.

Evidence: V13 PID172512 actually closed at06:30:22Z; stdout line17 cancels the poller at06:30:21.915720, then line18 records hover's `window not found` at06:30:21.938645. Its final Sky inventory is empty and exit0, so those observations cannot substitute for clean-retirement acceptance. Source609/EXE were independently checked before release.

Discriminating hypotheses before the V14 regression: late delivery after owner removal95%; live-window leasing/re-entry4%; unrelated source/identity mismatch1%. These are diagnostic priorities, not measured frequencies. The native source location and clean identity check favor owner retirement; a retained-callback test after actual GPUI removal must establish causality, and a live leased-window control must demonstrate that real errors remain visible. Deferred appearance work needs a check when its scheduled body executes, not only when registered.

## 2. Why prior repairs did not cover this event

V8 corrected the specific queued activation lifetime and first-party native close interception. V12 cancelled the recurring Shell foreground task before platform teardown; V13 extended that ownership to pending import-read and chooser waits. Their regressions remain relevant and must stay green. They cannot cancel synchronous/queued platform callbacks owned by a different layer. The new evidence is a scope gap at the callback registration boundary, not permission to discard those repairs or mute errors.

Repeated successful native runs increase confidence only for their observed paths. The fifth failure remains visible, and no run-count rule upgrades the complete native gate automatically.

## 3. Prevention mechanisms

| Priority | Mechanism | Action and status |
| --- | --- | --- |
|P0|Shared lifecycle authority|V14 implementer applies the owning-Window weak token to all17 Window-access event registrations, with an execution-time recheck for deferred appearance; independent review pending|
|P0|Executable regressions|Actual retained TestWindow callbacks after real Window removal reproduce4 failures/1 passing live control, then all5 pass; see the versioned red-final/green receipts below|
|P0|Error preservation|Retired callbacks use their established neutral/default return; live update errors remain logged. No broad log filter or dependency waiver|
|P0|Native verification|Root retests actual titlebar Close from the failing narrow panel, nonowner close/restart and owner close while Presenting, reviewing full post-exit logs|
|P1|Contract capture|Root updated component-guidelines, maintained-dependencies and ADR-008 with the separate boundaries and shared registration-group rule|
|P1|Evidence versioning|V13's five runs,92 files and26 original screenshots are immutable; V14 uses a new source/EXE freeze and run tags|

## 4. Systematic expansion

The bounded audit covers platform callbacks registered by the owning Window that access its handle, including geometry, visibility, input/hit-test, frame and tab callbacks and deferred execution. The same owner rule should be consistent across this group. Accessibility Send channels/foreground receivers have a separate scheduling mechanism; include a repair only when their reachability and an executable failing case establish necessity. Do not expand to unrelated parser/render/dependency changes.

Native capture remains Root-exclusive and serial. Implementation and independent check complete before a new launch. Physical topology limits stay NOT_RUN; missing feasible dark sizes are pending work, not external blockers. None of this changes Gate B, P2 or the acceptance thresholds.

### Executable evidence

`product-v14-retirement-red-final.json` records exit101 at06:43:57Z: retained hover fails at the exact native F12 location, deferred appearance fails after enqueue-while-live followed by removal, and the notification/input/frame groups also fail. The log contains19 removed-window errors: hover1, deferred appearance1, notifications12 and input/hit-test/frame5. The normal-live plus leased-live-error control passes before the fix.

`product-v14-retirement-green.json` records the same five tests passing at06:45:31Z, exit0. Retired callbacks retain their established neutral/default result and the live leased error remains observable. This is a real GPUI lifecycle regression, not a source-string assertion. Accessibility Send/channel callbacks are outside this bounded repair: no executable F12 failure or equivalent TestWindow path has been established for them, so their retirement is not claimed covered.

## 5. Knowledge capture

- Updated `.trellis/spec/frontend/component-guidelines.md` with Shell-task versus platform-callback boundaries.
- Updated `.trellis/spec/frontend/maintained-dependencies.md` with registration-group lifetime and live-error regression requirements.
- Added the V14 bounded repair record to `docs/adr/ADR-008-gpui-dependency-gate.md`.
- This repository has no `src/templates/markdown/spec` directory, so there is no corresponding generated template to synchronize.
- The user's explicit instruction keeps all new work uncommitted. The break-loop skill's commit suggestion is not executed; no permission re-request is necessary.
