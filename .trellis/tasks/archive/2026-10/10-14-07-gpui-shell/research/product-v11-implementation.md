# V11 — observe native client response before bounded correction

Active task: `.trellis/tasks/10-14-07-gpui-shell`. Root explicitly released V10 source after stopping its owned failed native startup process. No desktop interaction or native app launch was performed by this implementer; all prior evidence and uncommitted work remain preserved.

## Actual V10 failure and source boundary

`native/v10-main.stdout.log` proves that V10's first size request did not become the requested client: C initially reported1202.6666×912 at150%; requesting1002.6666×760 produced1000×754. The real discrepancy is4×9 physical pixels. The strict2px confirmation rejected it and timed out at04:54:54.532. Root's subsequent owned-process cleanup exited-1; this is not normal-close PASS. V10's product and independent checks were valid within their headless model, which returned exact resize sizes and did not cover this native frame conversion.

Read-only pinned-source inspection shows `gpui_windows/src/window.rs::resize` converts the logical client request to device pixels and adds cached `WindowBorderOffset` before `SetWindowPos`. `events.rs::handle_dpi_changed_msg` refreshes that cache before applying the suggested new-DPI rectangle; `handle_size_msg` subsequently reports actual client pixels. The observed4×9 difference is consistent with the cached non-client conversion lagging the cross-DPI transition. V11 uses actual client feedback rather than hard-coding this measured offset or changing platform packages.

## Product change and invariants

Only `crates/ferrostage-ui/src/shell.rs` changes relative to V10. Root independently updated the state-management contract before freeze. `policy.rs`, all vendors, dependencies and deny policy remain byte-identical to V10.

- `ClientResizeAttempt` retains the previous API request, pre-request observed client and whether the single corrective request has been used.
- `restore_client_size` first requests the already fitted target client as before. It clears the observation flag before invoking native resize.
- A further request requires a real bounds callback and a client size different from the pre-request observation. Repeated old callbacks cannot count as a response.
- On that response, compensation is `previous request + (target client - actual client)`. Each axis must have a finite discrepancy no larger than32 logical pixels. Only one such correction is allowed: at most two resize calls for a replacement window.
- Pending-transaction polling may advance this bounded correction before the existing deadline. Unresponsive, out-of-range or nonconvergent placement keeps waiting and uses the existing timeout/rollback. It never retries after the correction budget is spent or the deadline expires.
- The actual client must still match its immutable target within2px, and independent `policy::placement_confirmed` still checks the requested monitor, fullscreen state and all edges within usable bounds with the32px decoration allowance. Fullscreen's native-event/full-display gate is unchanged. No dimensions, tolerance or placement rules were relaxed.
- Diagnostics include the immutable target, API request, actual bounds and correction boolean. No existing error logging or close behavior was suppressed.

No startup-failure quit-policy change is included; the task remains the bounded placement correction. If native placement still cannot converge, the existing failure path remains a failure requiring Root's recorded handling.

## Regressions and checks

`placement_v11_windows_frame_delta_is_corrected_from_actual_readback` operates a real GPUI TestPlatform window and injects the measured Windows response via its actual resize callback. Every API request is converted at150%, rounded to device pixels, and reduced by4×9 physical pixels before read-back. It failed against V10 (`product-v11-placement-red.*`, exit101) and passes with compensation (`product-v11-placement-green.*`, exit0). This is a native-boundary model, not a Windows desktop test.

`placement_v11_large_or_nonconvergent_frame_response_stops_and_rolls_back` covers a33px error exceeding the cap, then a small error whose corrected native response reverses instead of converging. Multiple polls do not issue further resizes; deadline expiry preserves the running state/page/timer, settings, original presenter/audience and live snapshot, and retires only the replacement.

The V10 uncorrected-client rollback test retains all prior assertions. Its fixture now explicitly models an unresponsive first resize followed by a stale pre-request bounds callback, and additionally asserts that no correction is authorized. The original V9 overflow guard, V10 geometry checks and narrow client interaction test all remain in the final suite.

| Check | Result |
| --- | --- |
| New measured native-response regression | RED1 then GREEN1 |
| Complete UI test-support suite | PASS70:47 actual GPUI tests +23 pure |
| `just ci` | PASS150 default tests, formatting, strict default lint, policy |
| All-feature/all-target app/UI Clippy `-D warnings` | PASS |
| Full all-feature locked/offline deny | PASS; no waiver |
| Native gpui,pdfium locked/offline build | PASS; not launched |
| Diff and frozen input read-back | PASS |

The initial full-suite attempt had a compile error in the new test's `Size<Pixels> * f32` expression. It is preserved as `product-v11-headless.*`; using typed per-component mapping fixed the test expression, then `product-v11-headless-final.*` passed70. The existing dependency HarfRust dead-code warning remains visible.

## Frozen handoff and remaining work

- `product-native-freeze-v11.json`:609 inputs,607 unchanged from V10; only shell plus Root's contract differ.
- Source aggregate: `fdec41e24793817d526127a3928c9e1f0bfafa021a293dce26cc9125f87c48f1`.
- EXE: `027ab17fc23a92c00c480148b06c1ac115f81fcfe1a2145ac210dd6dd79a4d6d`.
- Freeze SHA256: `db41074638d3d844e8a69c5a1575c1f507c8558ba2523e700048b16058302716`.
- DLL/PDF/Cargo.lock unchanged; five existing isolated configs read back and captured without editing. New V11 evidence manifest seals new receipts only. Historical freeze identities are not claimed to match the newly rebuilt executable path.

All implementer Cargo commands have returned; full-host read-only process closeout is recorded separately. Root next serially dispatches trellis-check, then repeats actual C startup and checks both final client dimensions and native-frame/titlebar visibility. V11 native acceptance, Presenting migration/no-play, save retry/future schema, and actual narrow sizes remain incomplete. Physical unplug/reconnect/ports and real single/zero remain NOT_RUN. Gate B is incomplete; no08–11 integration, commit, push, archive or add_session occurred.
