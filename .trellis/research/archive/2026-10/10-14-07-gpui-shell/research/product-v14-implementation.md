# V14: cancel retained platform events at the owning Window boundary

Product implementation and checks PASS; independent review and V14 native acceptance are pending. V13 F12 remains FAIL. All changes are uncommitted. No native application was launched by the implementer.

## Cause and bounded repair

Root's `native/v13-light-small.stdout.log:18` records `window not found` at the unguarded hover callback (`window.rs:1943` in V13), after Shell's poll cancellation and before exit0. Pinned `gpui_windows/src/events.rs:407` takes and invokes the retained hover callback on mouse leave. Its platform owner outlives the GPUI Window: `WindowsWindow::drop` retains an inner Rc while deferred native destruction waits for dialogs (`window.rs:608–626`). Exit0 and empty windows therefore do not prove error-free retirement.

The V8 token guarded only activation. V14 extends the **same single Window-owned Rc token** to the seventeen event registrations below. Each callback owns only a Weak pointer and checks its strong count on the foreground thread before accessing the handle. No upgraded Rc is retained across an update, so re-entrant teardown cannot prolong the token lifetime. The token field precedes `platform_window`, retaining the required drop order. Appearance checks both before enqueueing and when its deferred body executes. Every live `update(...).log_err()` is unchanged; input retains `DispatchEventResult::default()`, hit-test retains `None`.

| Registration | Executable evidence |
| --- | --- |
| active-status | Existing real GPUI V8 queued activation/replacement regression, retained and passing in full82 suite |
| hover | V14 exact F12 callback after real Window removal |
| visibility, viewport, insets, resize, moved, button layout | V14 retained notification group |
| move tab, merge tabs, next tab, previous tab, toggle tab bar | V14 TestWindow stores the real registered callbacks and delivers all five after removal |
| input, control hit-test, request-frame | V14 retained callbacks, unchanged return defaults and no new frame wake |
| appearance | V14 both delivery after removal and enqueue while live followed by removal before execution |

The intentional `on_close` teardown remains idempotent and still removes system-tab state. `on_window_should_close` and the first-party deferred-close interception remain unchanged. The V12/V13 owned poll/read/chooser tasks remain unchanged and their regressions pass.

Accessibility initialization uses three separate Send/channel producers and detached receivers, outside the synchronous registration group. Windows stores the activation/action handlers in its AccessKit adapter (`gpui_windows/window.rs:1073`); TestWindow's default `a11y_init` is a no-op. No equivalent executable F12 failure is established there, and this patch neither changes nor claims coverage of those receivers. This is an explicit audit boundary, not a claim that all future asynchronous teardown errors are impossible. No sixth maintained package, dependency/version change, logger filter, policy exception or advisory/lint waiver was added.

## Files and tests

- `vendor/gpui/src/window.rs`: sole runtime delta; `_platform_callback_lifetime` and the shared registration-group checks.
- `vendor/gpui/src/platform/test/window.rs`: retained hover/movement/button/tab/hit-test simulation; the five tabs now retain their actual callbacks in TestPlatform.
- `vendor/gpui/src/app/test_context.rs`: existing `test_window` is public within the test-support API to retain the platform owner; `with_window_lease` leases the real Window slot while releasing the App RefCell borrow, enabling the live error control.
- `crates/ferrostage-ui/src/shell.rs`: tests only. Scoped log capture replaces the single global V8 logger so parallel tests do not share capture buffers. Five new `retirement_v14_*` actual GPUI regressions.
- Root separately owns the component/maintenance spec and ADR-008 updates included in this freeze. Five-dimensional `trellis-break-loop` findings and knowledge capture are in `break-loop-f12.md`; no generated spec template exists and the user's no-commit instruction overrides the skill's commit suggestion.

The first two receipts (`retirement-red`, `red-compiled`) are test-harness visibility compile failures, not product-red proof. `red-executable` first reproduced four failing tests but also exposed a mistaken input propagation assertion. `retirement-red-final` preserves the corrected unchanged-default assertion and **4 actual failures / 1 passing live control**, with **19 removed-window ERROR records**: hover1, deferred appearance1, notifications12, input/hit-test/frame5. `retirement-green` runs those same five tests: **5 PASS**, exit0. The live control proves visibility/geometry/appearance still work and a temporarily leased but live Window still logs `window not found`; no blanket error suppression passes the test.

Final checks, all exit0:

| Check | Result |
| --- | --- |
| Scoped V14 real GPUI regressions | 5/5 |
| Full UI test-support suite | 82/82 = 59 actual GPUI +23 pure, no ignored tests |
| Consumer Clippy | app/UI all features, all targets, locked/offline, `-D warnings` |
| Typecheck | app/UI all features and all targets |
| Formatting | workspace fmt plus explicit rustfmt on the three changed vendor files |
| Policy | Fresh full all-feature locked/offline cargo deny, no policy changes |
| Native build | Actual app with `gpui,pdfium`, locked/offline |
| Default CI | V12's150 reused after verified unchanged default source/manifests/toolchain/CI/policy; see `product-v14-default-reuse.json`. This is explicitly not a new just-ci run. Current feature and policy checks were rerun. |

The existing HarfRust dependency `shape` dead-code warning remains visible; this is not a new allow or a claimed clean direct-HarfRust lint run. Its historic direct-lint limitation is unchanged.

## Frozen handoff

`product-native-freeze-v14.json` contains609 source/contract files;602 match V13 exactly. The seven changes are the four code/test files above and Root's three spec/ADR files. PDFium, raster fixture, Cargo.lock and dependency manifests match V13. Current isolated configuration bytes are recorded as new inputs without modifying the configs.

- Source aggregate: `9764bfa0d94764c314fd559d8df2b55e056fc8f6ef77882cdde89662fe5d4413`
- Native EXE: `4be903dc34b4bb47163816cc88a67caf8c9f9806772f931b209bf27a9d3dbcd1`
- Freeze SHA256: `3f00b403a9668d3a841e0ba15564d28ad804934f8b740c3d4f1803b82da41a1c`

The new evidence manifest binds the immutable command receipts, source delta/reuse evidence, configuration inputs, process closeout and final read-back. Root next dispatches serial independent check, then repeats the failing actual light320x640 scrolled Displays/titlebar-close and remaining matrix on this identity, reading full post-exit logs. Gate B remains INCOMPLETE; physical unplug/reconnect/ports and real single/zero stay NOT_RUN. No08–11 implementation, commit, push, archive or add_session.
