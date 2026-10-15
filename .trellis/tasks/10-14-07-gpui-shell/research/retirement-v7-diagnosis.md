# V7 retirement diagnosis prepared against frozen V6

Date: 2026-10-14 America/New_York (source receipts use UTC 2026-10-15).

**Status: diagnosis prepared against frozen V6; subsequently Root released the freeze and authorized diagnostic instrumentation only.** The new V7 diagnostic handoff is `product-v7-diagnostics.md`. No lifecycle/vendor/config fix or native acceptance is claimed. Source pointers below refer to V6 (the two changed first-party files have retained V6 copies). Root owns native runs and task status. Physical plug/unplug, connector changes and latency remain NOT_RUN; Gate B is incomplete and 08–11 remain dependent on it.

## Evidence and identity

Root's `resume-v6-identity-20261015.json` confirms executable SHA256 `d4f2e653ee793176b384034797ad99e1ad665aa0da6535a454ba88a36b055074`, all 94 crate files, PDFium and fixture unchanged. The only delta among 602 frozen inputs is the already documented DESIGN.md closeout. This investigation does not replace the source-bound V6 F7/F8 acceptance.

`retirement-v7-evidence.json` records hashes and exact ERROR lines from the two old V6 logs and the new `resume-v6-light` log, plus inspected source hashes. Pinned external source directory is `C:/Users/lyh/.cargo/git/checkouts/zed-a70e2ad075855582/96a1b1b/crates/`; `Cargo.lock` resolves Zed to `96a1b1b5389efff473fc2306f7f8c690f2e1a417`. Read-only Git commands in that external checkout returned permission denied; no claim of independently checking its Git worktree cleanliness is made. Source bytes were readable and hashed.

| Receipt | Distinct observation |
| --- | --- |
| `native/v6-focus-save.stdout.log` | Startup replacement: bare ERROR at 02:35:35.748029Z. Confirmed Swap retires old Presenter: bare ERROR at 02:39:23.263525Z. Normal-close cluster at 02:45:52: bare ERROR, `gpui_windows::window` HRESULTs `0x80040102` and `0x80070578`, then `gpui_windows::dispatcher` `0x80070578`. |
| `native/v6-relaunch.stdout.log` | Startup replacement: two bare ERRORs at 02:46:11.611875Z/647990Z. Normal-close cluster at 02:46:51: bare ERROR plus the same two window HRESULTs. |
| `native/resume-v6-light.stdout.log` | First identical topology event cancels pending RecoverPresenter, replaces `WindowId(1v1)` with `WindowId(1v3)`, commits Ready, then bare ERROR at 03:22:37.886952Z. Exit receipt is 0, but Root did not successfully issue a close action and the exit trigger is UNKNOWN. This is not normal-close acceptance. |
| `native/resume-v6-dark.stdout.log` | Startup and later bare ERRORs at 03:24:09.277329Z and 03:25:30.224160Z; actual AX Close then bare ERROR and paired window HRESULTs at 03:26:14.281–282Z. Root confirmed exit0 and released the freeze only after this run ended. |

Empty stderr does not mean a clean run: these ERROR records are in stdout. Prior normal-close exit0 and successful role/focus behavior remain valid observations, but do not prove cleanup correctness.

## 1. Bare `window not found`: producer identified; exact callback pending

The application close hooks are not the direct ERROR emitter:

- `crates/ferrostage-ui/src/shell.rs:172–188`: `install_close_handler` defers `Shell::closed(id)` until after the Shell lease; its failure is DEBUG `closed retired shell`.
- `shell.rs:715–721`: `Shell::close` requests actual `window.remove_window()`; its failure is DEBUG `window already retired`.
- `shell.rs:1013–1047`: current/pending/retired WindowId classification retains the V6 contract. A retired ID does nothing; current Presenter closure calls Stop and quit.

The generic error is constructed by `vendor/gpui/src/app.rs:2013–2026` when `update_window_erased` cannot resolve the ID. The implementation at `app.rs:2945–3001` also treats a temporarily leased window as unavailable, so the string alone cannot distinguish reentrant access from a retired ID. Typed reads have the same text at `app.rs:3128` and `window.rs:7264`; tracing callbacks actually use `handle.update`.

The empty log target has a specific explanation. Pinned `gpui_util/src/lib.rs:300–330` derives the target from a source path containing `crates/`. This project's GPUI source is under `vendor/gpui/`, so its `ResultExt::log_err` calls have an empty target. The helper **still preserves caller file and line in the log Record**. `crates/ferrostage-app/src/main.rs:48–54` currently does not print those fields.

Strong, reachable late-callback candidate:

1. Native `WM_ACTIVATE` enters pinned `gpui_windows/src/events.rs:803–850`; the activation callback is delivered by a detached foreground task holding `Rc<WindowsWindowInner>`.
2. The application removes the old GPUI window during pending cancellation (`shell.rs:765–777`) or replacement commit (`shell.rs:949–971`). `app.rs:2962–2983` removes its ID before invoking closed observers and dropping the Window.
3. Pinned `gpui_windows/src/window.rs:608–629` clears the visibility callback during Drop, but leaves the active-status callback in that retained inner object.
4. The queued callback reaches `vendor/gpui/src/window.rs:1896–1914`; `handle.update(...).log_err()` sees the old ID missing and writes the bare ERROR.

This is a source-proven reachable sequence, **not yet unique runtime attribution of every V6 ERROR**. GPUI accessibility activation/deactivation/action tasks (`window.rs:1652–1686`), deferred appearance, frame, input and bounds callbacks also call `handle.update(...).log_err()`. The fresh light run has no Accessibility activated records but still has the ERROR, so accessibility activation is not necessary for at least that reproduction. Capture exact caller location before choosing a vendor repair.

Startup churn is independently explained: `Shell::tick` (`shell.rs:977–1005`) unconditionally retires pending windows on every `TopologyEvent`, including the initial Changed snapshot equal to the synchronously enumerated topology. It creates a retirement opportunity at every observed startup. Deduplicating that event may avoid startup churn but cannot fix the separate Swap and user-close paths; it must not be represented as resolving retirement errors generally.

## 2. Native close HRESULTs: distinct OS-first destruction path

No first-party `on_window_should_close` handler is installed. Pinned `gpui_windows/src/events.rs:336–341` therefore returns None for WM_CLOSE, allowing `DefWindowProcW` to destroy the native window (`window.rs:1463–1477` delegates unhandled messages).

`WM_DESTROY` calls the GPUI close callback (`events.rs:343–366` → `vendor/gpui/src/window.rs:1689–1700`). That callback removes the GPUI Window. Its platform Drop then schedules `RevokeDragDrop(handle)` and `DestroyWindow(handle)` after `dialog_owner.when_idle()` (`gpui_windows/src/window.rs:608–629`). The original Windows destruction can finish before this task runs. The two adjacent calls are the strongly matching source sites for `0x80040102` followed by `0x80070578`. This is distinct from the late activation callback and explains why programmatic Swap has the bare error without the paired HRESULTs.

The dispatcher error has a further lifetime: `gpui_windows/src/dispatcher.rs:117–130` posts a wake message to the platform's hidden HWND after queuing a runnable; `gpui_windows/src/platform.rs:1245–1255` destroys that hidden HWND during platform Drop. Late task wakeup after teardown is a reachable explanation. The exact wakeup producer is **UNVERIFIED**, and the sporadic dispatcher error must not be silently grouped with a fixed presenter callback.

## Minimal next sequence after Root releases the freeze

1. **DONE after Root's release:** add `.with_file(true).with_line_number(true)` to the existing app tracing formatter, and WindowId/bounds/viewport size/scale/display/fullscreen to existing bounds-observer DEBUG diagnostics. This only exposes existing metadata; no log suppression, vendor patch, logging dependency or behavior change. V7 is separately frozen; actual native attribution remains pending.
2. Reproduce three independent cases with owned processes and isolated config: initial topology cancellation, confirmed Swap while preserving page/blank state, and actual title-bar close. Record source line, WindowId/lifecycle context where available, HWND inventory and exit. Keep the current V6 logs unchanged. Require no unexplained window/dispatcher ERRORs as a separate cleanup assertion; do not reinterpret an exit0 as that assertion passing.
3. For the OS-first close branch, the smallest candidate first-party fix is installing the existing safe GPUI `Window::on_window_should_close` API for both roles in `Shell::open`: defer removal of that original window handle, return false to prevent default Windows destruction, and let the existing deferred `on_window_closed` coordinator retain current/pending/retired semantics. `vendor/gpui/src/window.rs:6688–6697` provides the API. Do not mutate Shell from inside the originating window lease or duplicate role policy. This is a candidate pending the diagnostic run and regression, not an implemented or verified fix.
4. For a confirmed stale active-status callback, first-party close interception alone is insufficient because programmatic retirement already reproduces it. A proper callback-lifetime repair belongs at the owning GPUI/platform boundary; prepare the exact reviewed delta only after source-location evidence. Do not replace `.log_err()` with silent success, broad catches or an untyped text check. Altering another platform package requires recording the newly maintained source scope and affected checks; existing ADR-008 approval is not proof that arbitrary platform edits were reviewed.

## Regression proposals and verification boundaries

- **Actual GPUI late activation regression:** retain an active presenter, remove it through the real coordinator, activate the replacement, then drain the foreground executor. The test platform also defers active-status delivery (`vendor/gpui/src/platform/test/platform.rs:295–316`) and retains the old `TestWindow`, so a narrowly captured log record at the suspected callback can make the stale-handle failure observable. Assert current replacement, state/page and no stale callback after draining. This tests the real handler, not a policy helper. It is proposed, NOT_RUN; Windows callback source-line/native retest is still required.
- **Actual close-interception regression:** open the real presenter/audience, invoke `VisualTestContext::simulate_close`, then drain deferred work and assert the corresponding window is gone and the existing coordinator classification occurred once. V6 has no should-close handler, so the test helper returns false and retains the window. The helper (`app/test_context.rs:1040–1067`) only invokes/restores a should-close callback; it does not emulate Windows default destruction. Therefore the post-drain window assertion tests the new first-party interception, not native HRESULT absence. Removal must be deferred: the helper itself restores the callback through a window update immediately after the callback returns.
- Preserve the existing real-GPUI lease/rollback/replacement tests at `shell.rs:2171–2279`, and add the missing title-bar request path rather than replacing them. Cover a pending window and an old retired WindowId as well as current Presenter/Audience. Test user-close during projection without leaking audience, and keep explicit quit/zero-screen waiting semantics.
- The current retained logs already fail a native cleanup assertion that rejects unexplained `window not found` or invalid-handle ERRORs. They are reproduction evidence, not a newly executed automated red test. New red/green test and native runs remain NOT_RUN under this freeze.

Initial diagnosis checks were read-only log/source tracing and bounded source/hash collection. Following Root's explicit freeze release, the narrowly authorized instrumentation passed the checks in `product-v7-diagnostics.md`. No lifecycle repair, native process launch/termination, physical operation, commit, push, archive or journal write occurred.
