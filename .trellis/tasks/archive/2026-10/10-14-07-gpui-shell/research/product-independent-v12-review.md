# Independent V12 review and V13 self-fix

Active task: `.trellis/tasks/10-14-07-gpui-shell`. Root owns desktop operation, parent/child status updates and specifications. This reviewer preserved V12 and all historical evidence, then made the authorized surgical V13 repair below. No native application was launched.

## Findings (fixed)

- File: `crates/ferrostage-ui/src/shell.rs`.
  Issue: V12 correctly cancelled its recurring poller, but the import-read and file-chooser foreground continuations remained detached. Pending filesystem work can outlive a normal Presenter close. Application shutdown can also occur before a chooser receiver resolves; native dialog cancellation is asynchronous. Weak entity, import-token and presenter-owner checks execute only after foreground dispatch, so they cannot prevent that late dispatch. App shutdown retains enough Shell state for a late chooser response to attempt import.
  Fix: V13 retains both task handles and cancels them alongside the poller in the same synchronous `stop_tasks` path, before current-Presenter quit and in `on_app_quit`. Existing weak/token/owner checks remain. `await_import_read` separates the existing continuation from its filesystem producer so tests can control delivery without changing production I/O policy. Superseding an operation drops its old handle; no new dependency, vendor patch or logging suppression was introduced.

Three actual GPUI regressions failed before the ownership repair: pending read survives Presenter close, pending read survives `App::shutdown`, and late TestPlatform chooser delivery mutates Shell after shutdown. All three pass after repair. A positive control proves retired-window close preserves normal read-result/error propagation and chooser-result handling. The red receipt is `product-v13-retirement-red.json` (1 pass,3 fail,exit101); green is `product-v13-retirement-green.json` (4 pass,exit0). `product-v13-before-shell.rs` preserves exact V12 Shell bytes and `product-v13-shell.patch` records the final delta.

## Findings (not fixed)

No remaining first-party defect was identified in this bounded review. V13 source/model PASS is not native F11 PASS: the historical V11 dark close emitted `0x80070578` despite exit0, and Root must retest the new executable. The repair cancels foreground continuations; it does not promise interruption of an already executing synchronous `fs::read`, synchronous destruction of a native dialog, or cancellation of every platform-owned task. Pinned dialog owner/cancellation/destruction coordination is unchanged.

Initial undisturbed C full-titlebar visibility remains UNVERIFIED. Physical unplug/reconnect/ports/latency and actual single/zero-monitor hardware remain NOT_RUN. Gate B INCOMPLETE, C/D NOT_RUN,08–11 gated and P2 unchanged.

## Lifecycle review

- Pinned `scheduler::Task` owns `async_task::Task`. `async-task4.7.1` Task Drop calls `set_canceled`, which atomically sets CLOSED before scheduling any cleanup; both wake implementations reject CLOSED. Cancellation is synchronous even when dropping the future itself requires executor cleanup.
- `Context::on_app_quit` invokes the entity callback synchronously when App collects quit futures. `App::shutdown` runs those observers before clearing windows and before platform teardown; returning `async {}` does not defer cancellation. Windows `quit` schedules PostQuitMessage, so the current-Presenter close path cancels before that request as well.
- Pinned Windows dispatcher129 posts to the hidden platform HWND after queue insertion; WindowsPlatform Drop destroys that HWND. This explains why an after-await weak check is insufficient. The V12 poller repair itself is sound and its regressions remain green in V13.
- Audience, pending and retired close branches do not call `stop_tasks`. Explicit quit mode remains; startup with no displays still starts polling and topology recovery does not cancel it. The existing topology-recovery tests and new nonowner controls pass. No projection/session/timer/settings/placement code changed.
- Pinned Windows `show_dialog` has owner and receiver cancellation plus asynchronous dialog completion. This provides platform ownership coordination, not a guarantee that the product receiver always resolves before application shutdown. V13 drops the product awaiter early while retaining the platform path.
- Root's component contract now covers all three Shell-owned waits, retains the weak/token guards and separates continuation cancellation from I/O/dialog/native acceptance. It matches the implementation.

## Historical native evidence and PRD checkboxes

Independently verified all115 V11 native manifest records, all37 V9 native records, launch identities, before/after configs, exits, empty final inventories and logs. `product-independent-v12-native-audit.json` records independently recomputed comparisons.

- V11 main/relaunch/future logs contain no retirement error; main retains expected settings-lock warnings. V11 dark has exactly the recorded dispatcher error. All four exits are0, but dark remains a failed close.
- Relaunch/future configs are byte-identical. Dark differs only by CRLF→LF and parses identically; it cannot be called unchanged-byte Cancel proof. Main contains the intentional role edits. No-play read-back and locked retry share `ea08e183…`; successful retry and relaunch use `23b646a9…` with B Presenter/C Audience and A None, identified by MonitorId.
- The new-panel B/A Apply commits Ready with no audience; Start creates A. B/C migration commits Presenting page2 with running elapsed79649→80112ms, and the screenshots retain white blank. No-play commits Ready page2/elapsed138110ms with no audience; later panel logs retain that time. These are07 placeholder projection observations, not08 PDF raster acceptance.
- Viewed the three separate V9 Identify screenshots (1,2,3) and V11 assignment/relaunch panel screenshots. V9's stable list/number/Identify code, presenter/locales and role resolver are unchanged through V13; intervening policy/Shell deltas concern geometry or owned task lifetimes. These are applicable, explicitly version-scoped functional observations, not newly run V13 native tests.
- The first two new-UI PRD checkboxes are supported: the primary display-settings entry/list/Identify and real new-panel B/A→B/C Apply/Start plus MonitorId persistence/relaunch have evidence. Their adjacent version-scoped index is essential. They do not assert full titlebar/close/matrix completion. The remaining combined checkboxes correctly stay unchecked, since physical single-screen and the full native state/size/theme matrix remain incomplete.
- V11 initial C actual size converged, but later external moves preceded its screenshot; initial full-frame/titlebar visibility remains unverified. Near800×600.8 and389.6×844 remain exact reported observations, not silently rounded acceptance at800×600 or390×844.

## Verification

| Check | Result |
| --- | --- |
| V12 independent identity | PASS609 inputs/executable; implementation23, V11 native115, prior independent13 and V9 native37 records match |
| V12 independent lint/typecheck/retirement | PASS strict all-feature/all-target lint and explicit check;6 related GPUI regressions |
| V13 red/green | RED3 plus positive control; GREEN4 |
| V13 complete affected suite | PASS77:54 actual GPUI tests plus23 pure |
| V13 lint | PASS app/UI all features/all targets, locked/offline Clippy `-D warnings` |
| V13 TypeCheck | PASS explicit app/UI all features/all targets, locked/offline |
| V13 formatting/diff | PASS |
| V13 native build | PASS locked/offline `gpui,pdfium`; not launched |
| Reused unaffected V12 checks | Default150 tests/CI and full all-feature deny PASS; explicitly reused, not rerun |

The default source path and dependency graph are unchanged; Shell is GPUI-feature gated. New formatting, all affected UI tests, strict feature lint/typecheck and native build were rerun. Exact commands and applicability are in `product-v13-reused-checks.json`; historical HarfRust warning remains visible. No additional full-suite or policy run was needed.

## Final V13 handoff

- Source609 aggregate: `77aa180c7f60bcc39de7315554d9781d5f2f39effd211078a1da43cf7ed40d4a`.
- EXE: `d1164919dcdc09053df4fc6caae09be249329cccd0ad8bd482a67347561a6435`.
- `product-native-freeze-v13.json` SHA256: `d3689820edf788e4e55d485b644f58007c4616e6ca49262de9aa057d5e86fb91`.
- Only Shell and Root's component contract differ from V12;607 inputs remain identical. V12 and older freezes were not overwritten.

All reviewer commands returned before native handoff. Full-host Cargo/rustc/FerroStage process inventory was empty at05:58:00Z. Root received explicit source-review completion and may run frozen V13; only report/evidence writes follow. No commit, push, archive or add_session was invoked. The freeze generator inherits an implementer-labelled NOT_RUN field; V13 was produced by the reviewer, and neither role launched it.
