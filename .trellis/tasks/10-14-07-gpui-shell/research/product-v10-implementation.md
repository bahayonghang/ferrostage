# V10 — mixed-DPI windowed placement repair

Active task: `.trellis/tasks/10-14-07-gpui-shell`. Authorized implementation continued only after Root closed V9 PID51832 through actual Alt+F4 and released the frozen source. No native application or desktop operation was performed by this implementer. All V6–V9 evidence remains unchanged.

## Failure and cause

Root's V9 `native/v9-main.stdout.log` at04:38:21 records C requested/actual display65735, client bounds `[1708,-273.33334,1280,912]`, full display `[1706.6666,-761.3333,1066.6666,1706.6666]`, actual scale1.5. `native/v9-c-app-ready-0.jpg` visibly clips the right side and titlebar controls. B→A previously produced1320×912 from1100×760, the exact1.5/1.25 ratio.

Read-only inspection of pinned `gpui_windows/src/window.rs` shows `WindowsWindow::new` creates with `CW_USEDEFAULT` first, then `retrieve_window_placement` converts the requested logical rectangle with the target monitor's DPI before `SetWindowPlacement`. In `events.rs::handle_dpi_changed_msg`, the resulting cross-monitor DPI transition uses the system's suggested rectangle. This native evidence and source path account for a second scaling of the requested logical size. No Cargo checkout or new platform package was patched.

The first-party open path used full monitor bounds, without work-area/titlebar room. Its windowed confirmation only checked the actual monitor, fullscreen flag and positive finite dimensions, so incorrect dimensions could commit. Fullscreen already had a separate strict native-bounds gate.

## Bounded change

- `crates/ferrostage-ui/src/policy.rs:245`: `windowed_bounds([x,y,w,h], [width,height])` fits/centers the requested client in the target GPUI `visible_bounds`, with32 logical pixels around the client for native decorations. It does not use the platform topology's GDI scale metadata as effective DPI.
- `shell.rs:62,670,743`: `WindowRecord::client_size` retains the fitted request. `Shell::restore_client_size` invokes existing `Window::resize` after creation only if the actual client size differs. At this point the native window has its target DPI. Resize is asynchronous on Windows; no successful completion is inferred from the request.
- `shell.rs:776` and `policy.rs:259`: the existing transaction reads back actual client size (within2 logical pixels of the request) and verifies all four client edges inside the usable area with the decoration inset (same2px rounding tolerance). An oversized, wrong-position or uncorrected window cannot commit; the existing deadline/rollback remains authoritative. Fullscreen still compares full display bounds and requires its actual bounds event.
- Placement diagnostics add requested client size and actual usable bounds. Live callback error reporting, owning callback lifetime, close interception and retirement behavior remain unchanged from the reviewed V8/V9 implementation.
- Root separately updated `.trellis/spec/frontend/state-management.md` to this contract before freeze. No other frozen input differs from V9:606 unchanged inputs,2 UI files plus1 Root spec changed. No dependency, policy, schema, public platform API or design change.

The32px inset is a conservative native-frame allowance. Exact outer-frame/titlebar visibility still requires Root's Windows screenshots; headless tests are not proof of HWND decorations.

## Regression and validation

The first two new regressions ran against the old implementation and both failed in real executed tests (`product-v10-placement-red.*`, exit101). They then passed (`product-v10-placement-green.*`, exit0). The final full suite includes:

1. `placement_v10_portrait_native_overflow_is_not_confirmation`: exact V9 C geometry must fail.
2. `placement_v10_scaled_client_size_cannot_commit`: real GPUI TestPlatform window receives1320×912, confirmation rejects it; existing resize restores expected size and confirmation then succeeds.
3. `placement_v10_uncorrected_client_times_out_and_keeps_running_owner`: actual oversized replacement remains pending, then times out; original presenter/audience, settings, full running state/page/timer and live snapshot remain; replacement is retired.
4. `placement_v10_work_area_and_narrow_clients_leave_native_frame_space`: portrait fit, work-area/taskbar/top-decoration rejection, and800×600/390×844/320×640 model rectangles.
5. `narrow_v10_display_panel_keeps_apply_cancel_inside_client`: actual GPUI draws in Chinese and English at all three specified client sizes with three list entries, confirms noncollapsed visible Apply/Cancel, scrolls to the last PDF selector, clicks it into the draft, and clicks Cancel. No production layout change was needed.

The first expanded scroll test incorrectly initialized only two synthetic monitors then requested selector3; that test-fixture failure is preserved as `product-v10-narrow-scroll.*`. Initializing the base monitor corrected the fixture; final test and complete suite pass. The earlier one-monitor footer-only test pass remains a narrower historical receipt.

| Check | Result |
| --- | --- |
| Full UI `test-support` suite | PASS68:45 real GPUI tests and23 pure tests |
| `just ci` | PASS150 default tests, formatting, strict default lint and policy check |
| All-feature/all-target app/UI Clippy with `-D warnings` | PASS |
| Full all-feature locked/offline `cargo deny check` | PASS; no policy or advisory waiver |
| Native `gpui,pdfium` build, locked/offline | PASS; not launched |
| `git diff --check`, frozen source/binary/fixture read-back | PASS |

Existing dependency `harfrust::shape` dead-code warning stays visible. All final checks returned; no overlapping Cargo/native process is left by the implementer.

## Frozen handoff

- `product-native-freeze-v10.json`:609 inputs; aggregate `2ee35522a9f885cf8c62ac5007e7feecc7d5c32f28949fc2f7b65e4052383a67`.
- Executable SHA256 `783d5769785029f11fa73f631720b47f7280e5124b30801d1930ee213dee37af`.
- Freeze SHA256 `d911f84ec364ba9600709d1230112a9ed1eb41f063c807c2a588e69198076059`.
- PDFium/PDF/Cargo.lock hashes remain unchanged. Current five isolated `v8-*.toml` inputs are read back and their UTF-8 contents captured in `product-v10-config-inputs.json`; v8-light carries Root's V9 C-presenter/B-audience result. No configuration was edited here.
- `product-v10-evidence-manifest.json` seals NEW implementation/check records. Historical V9 freeze and native manifests are referenced, never rewritten. The shared executable path now holds V10, not V9.

Root next serially dispatches independent trellis-check, then native C startup/B↔A↔C assignment and complete titlebar/work-area/client geometry acceptance. Pinned Windows `WM_GETMINMAXINFO` scales the configured320×240 client minimum and adds its native border offsets; source therefore permits320-wide client sizing without lowering the minimum. Real800×600/390×844/320×640 read-back is still NOT_RUN for V10.

V9 independently recorded conflict-disabled Apply, Cancel unchanged config/window, visible Identify1/2/3 and clean close; those historical findings do not establish V10 placement. Presenting migration/no-play, save retry/future schema and remaining native matrix remain Root's work. Physical unplug/reconnect/port and real single/zero monitor tests remain NOT_RUN. Gate B remains incomplete;08–11 were not implemented. All edits are uncommitted; no push, archive or add_session.
