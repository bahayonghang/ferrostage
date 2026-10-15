# V12 — own and cancel the Shell poll task before quit

Active task: `.trellis/tasks/10-14-07-gpui-shell`. Read this report and the end of `implement.md` explicitly: the parent design and child implementation document exceed the context-injection cap. Root exclusively owns native operation and acceptance. No native app was launched by this implementer.

## F11 diagnosis and bounded repair

Root's immutable `native/v11-dark.stdout.log:51` records `gpui_windows::dispatcher` at05:36:19.862676Z, pinned `dispatcher.rs:129`, `Invalid window handle (0x80070578)`, after actual titlebar Close. Exit0 does not make this a passing close.

Read-only inspection of the pinned96a1b1b source establishes that dispatcher129 is `PostMessageW` to the platform's hidden dispatch window after a successful queue send. `WindowsPlatform::drop` destroys that hidden HWND; tasks may still hold the dispatcher/queue receiver. Thus queue availability alone is not HWND lifetime evidence. This is separate from the V8 window activation callback owning-token repair.

The first-party Shell started a detached foreground task that awaited a background timer every25ms. It was not cancelled on quit. Its `WeakEntity` check occurred only after resuming on the foreground executor, so it could not prevent the preceding late dispatch. The GPUI regressions below prove that old Shell work continued after both current-Presenter close and App shutdown.

Only `crates/ferrostage-ui/src/shell.rs` changed:

- `Shell::poll_task: Option<gpui::Task<()>>` owns the existing loop; `start_polling` replaces the previous detach.
- `stop_polling` synchronously takes/drops it before the current Presenter requests `cx.quit()` and in `Context::on_app_quit` before platform teardown.
- Pinned scheduler Task wraps `async_task::Task`; its Drop sets `CLOSED` synchronously, then schedules cancellation cleanup if necessary. Cancellation therefore happens while the dispatcher is still valid; a later timer cannot reschedule the cancelled Shell task. This does not rely on a sleep or best-effort entity check.
- Audience, pending and retired window close paths do not cancel the live coordinator. No resize, assignment, draft, persistence, role or presentation behavior changed.
- A debug record reports `shell polling task cancelled before quit` with `presenter closed` or `app shutdown`. Existing dispatcher errors, log levels and all previous cleanup behavior are unchanged.

Root's `.trellis/spec/frontend/component-guidelines.md` records the contract and is included in the new freeze. No vendor source, dependency, policy or ADR package scope changed. The first-party late-wake source is repaired; whether this fully removes the native F11 HRESULT is deliberately left to Root's V12 retest.

## Regression and product checks

Two tests ran red against the old detached loop (`product-v12-retirement-red.*`, exit101), then green against the owned task (`product-v12-retirement-green.*`, exit0):

- `retirement_v12_presenter_close_cancels_pending_poll_wake`: actual GPUI close interception removes the Presenter; retain Shell, deliver a topology event, expire the real executor timer. Old code consumed it after quit (revision0→1); repaired code leaves revision0 and no reopened window.
- `retirement_v12_app_shutdown_cancels_pending_poll_wake`: actual `App::shutdown` followed by the same delayed timer/event cannot resume Shell work, even when the entity remains retained.

`retirement_v12_nonowner_close_keeps_live_polling` additionally closes retired, pending and audience windows through real GPUI callbacks, confirms the current owner and task remain, then proves the live timer still consumes a topology event. Existing V8/V10/V11 regressions remain intact.

| Check | Result |
| --- | --- |
| Full UI test-support suite | PASS73:50 actual GPUI tests +23 pure |
| `just ci` | PASS150 default tests, formatting, strict default lint and dependency policy |
| All-feature/all-target app/UI Clippy `-D warnings` | PASS |
| Full all-feature locked/offline deny | PASS, no waiver |
| Native gpui,pdfium locked/offline build | PASS; not launched |
| Diff and source/binary/config read-back | PASS |

Existing HarfRust dead-code warning remains visible. All command receipts preserve exact commands and output; native HRESULT reproduction/absence is not claimed from headless tests.

## Frozen handoff

- `product-native-freeze-v12.json`:609 inputs,607 byte-identical to V11; only Shell and Root's component contract differ.
- Source aggregate `fa6617505a045fe2aa4dce0a3185eff47c3ed70bb283d1e1c7030e4c04054e73`.
- EXE `c8291c651a485ddff5cc00354dede2c67c25ea1080a4205e4d12fa0796789fda`.
- Freeze SHA256 `4544ec636dd193a23f59007896cc0aa7c39ac506298cf395504c6b2ac7b3d221`.
- PDFium/PDF/Cargo.lock unchanged. Five current isolated config inputs are read back and their exact UTF-8 contents captured without editing. New evidence manifest covers new V12 artifacts; all historical freezes/manifests remain intact.

Next is Root's serial trellis-check, explicitly assessing historical V9 Identify and V11 new-UI evidence across this lifecycle-only change, then native F11 dark resize/panel/Close retest and remaining matrix. Read `native-v11-checkpoint.md` for exact prior evidence: dark config has CRLF→LF only with identical parsed content, not byte identity; future/relaunch configs are byte-identical. V11 initial C fit has actual-bounds proof but lacks an initial undisturbed full-frame screenshot, so that visual item remains unverified.

All Cargo commands returned; full-host process closeout is recorded separately. Gate B remains incomplete, physical unplug/reconnect/ports and true single/zero monitor items NOT_RUN;08–11 remain gated. No commit, push, archive, add_session, native launch or desktop operation occurred.
