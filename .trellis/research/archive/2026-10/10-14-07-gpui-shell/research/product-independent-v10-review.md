# Independent V10 placement review

2026-10-15 America/New_York. **Source and automated review PASS.** No additional product defect found; the unchanged V10 executable may proceed to Root's native acceptance. This is not a native F9 fix confirmation or Gate B PASS.

## Findings (fixed)

None required during this review. The reviewer changed only new review/evidence files. Implementation's bounded F9 repair and Root's matching frontend contract are accepted for native retest.

## Findings (not fixed)

No remaining source defect was identified in this review. Native titlebar/decorations, corrected mixed-DPI size and final placement are still UNVERIFIED for V10. The32px inset is an explicit allowance, not a measured guarantee for every Windows frame configuration. Root must observe the complete titlebar/close controls and record actual client bounds, scale and usable display bounds on C and across B↔A↔C. If correction leaves a wrong position, the containment gate must reject it and retain rollback; the resize request alone is not success.

Physical unplug/reconnect/ports/latency and actual single/zero-display hardware remain NOT_RUN. Gate B stays INCOMPLETE and08–11 remain dependency-gated. No source finding is being deferred under those hardware limits; those are separate acceptance items.

## Source and evidence review

- Read current task context, `native-v9-checkpoint.md`, V10 implementation/delta/freeze, and the updated state-management contract. Inspected the actual V9 C screenshot: the right titlebar edge is clipped, agreeing with the recorded1280-wide client on a1066.67-wide logical display. V9's clean-close and other new-panel observations remain scoped to that historical run.
- Verified all609 V10 source/contract files and the source aggregate, current executable, DLL, PDF, lockfile, config inputs and check records. V10 implementation manifest29, V9 native manifest37 and prior independent V8/V9 manifest33 all match. The only frozen input differences from V9 are `policy.rs`, `shell.rs`, and Root's `state-management.md`;606 are unchanged. No dependency/vendor/settings-schema policy was widened.
- `windowed_bounds` computes the fitted client rectangle from the selected GPUI display's usable bounds, including the32px surrounding allowance. It does not use GDI diagnostic scale as effective DPI. Desired presenter/audience sizes remain local existing choices; no new geometry authority or native platform package was added.
- `WindowRecord` records the fitted client size. The post-create correction uses the existing `Window::resize` only for a windowed record whose observed size differs, and does not declare success. Read-only inspection of pinned `gpui_windows` confirms creation calls target placement before returning; WM_DPICHANGED handles nonmaximized suggested geometry synchronously, while resize later queues `SetWindowPos` with `SWP_NOMOVE`. Thus correction of size and verification of final position remain separate, as the implementation requires.
- Windowed confirmation now requires actual size within2 logical pixels of the fitted request, matching display/mode, finite positive geometry and all four client edges inside usable bounds with the same bounded rounding tolerance. A positive client size on the right display is no longer sufficient. Fullscreen retains the distinct observed-native-bounds requirement and uses full display bounds, not work-area bounds or the restore client size.
- The transaction still reads current window/display geometry on each poll. Bounds events only contribute the existing observation flag; a stale event cannot substitute for mismatched size, mode or location. Topology changes retain cancellation/invalidation. Failed or never-observed corrections reach the existing deadline, close only pending replacements and restore the old projection/session/settings; no new retry loop or suppressed platform error was introduced.
- Assignment, Presenting/no-play behavior, timer/page/blank preservation, future-schema refusal, shared draft/Identify, persistence ownership and current/pending/retired close classification remain on the previously reviewed paths. The stricter gate runs before assignment/Start/no-play commit. The new timeout test explicitly verifies a running no-play attempt retains the original presenter/audience, whole core state, settings and Live snapshot when correction does not complete.
- The original two red regressions are retained: exact native C overflow was incorrectly accepted, and actual GPUI scaled client size was incorrectly accepted. Both fail in the executed red log and pass in green. New tests cover fit/work-area edges, scaled-size rejection and correction, pending timeout rollback, and Chinese/English800×600,390×844,320×640 controls. The narrow test actually scrolls to PDF selector3, changes the draft and activates Cancel; it is more than a footer-bounds assertion. Its earlier missing-third-monitor fixture failure is retained separately, and no production layout was changed to satisfy it.
- The updated frontend state-management specification correctly separates client size/usable-area confirmation from fullscreen and requires native outer-frame evidence. No further spec or template synchronization was needed.

## Verification

| Check | Result |
| --- | --- |
| Independent strict lint | PASS: app/UI all features/all targets, locked/offline, `-D warnings` |
| Independent TypeCheck | PASS: explicit app/UI all features/all targets, locked/offline `cargo check` |
| Independent placement tests | PASS4; exact F9 geometry, size repair, work-area policy and running rollback |
| Independent narrow interaction | PASS1; both languages and all3 logical client sizes with scroll/selector/Cancel |
| Independent diff check | PASS |
| Verified complete UI feature suite | PASS68:45 real GPUI and23 pure; reused unchanged V10 receipt |
| Verified default `just ci` | PASS150 tests, formatting, strict default lint and dependency check; reused unchanged V10 receipt |
| Verified full dependency policy | PASS: all features, locked/offline deny; unchanged V10 receipt |
| Verified native build | PASS: `gpui,pdfium`, locked/offline; unchanged V10 build/executable |
| Frozen identity / evidence | PASS609 source inputs,29 implementation artifacts,37 V9 native artifacts,33 prior independent artifacts |

`product-independent-v10-reused-checks.json` records commands and verified counts from the reused receipts. New commands have separate `product-independent-v10-*` receipts. No expensive full-suite/build rerun was needed after the independent targeted checks because the reviewed inputs did not change. The historical HarfRust `shape` warning remains visible; no lint/advisory waiver was added. One read-only review-helper attempt used the host's defaultGBK encoding for Rust text; it was corrected to explicitUTF-8 before test-count verification and did not alter product input or gate results.

## Exact handoff

- Freeze: `product-native-freeze-v10.json`, SHA256 `d911f84ec364ba9600709d1230112a9ed1eb41f063c807c2a588e69198076059`.
- Source609 aggregate: `2ee35522a9f885cf8c62ac5007e7feecc7d5c32f28949fc2f7b65e4052383a67`.
- Executable SHA256: `783d5769785029f11fa73f631720b47f7280e5124b30801d1930ee213dee37af`.
- V10 evidence29 manifest SHA256: `4ee6804e79a537790f1c1bbf891fa2561392b535980121aba7f283839a214eb6`.
- V9 native37 manifest SHA256: `7e9d42e2894e5319770966bf2c602b62b5cbc13cb5771b85d19dbc96ff229822`.

Root received explicit source-review completion before native launch, after all reviewer Cargo commands returned. Only report/manifest writes followed; no source/executable/config changes, native launch or desktop interaction were performed by this reviewer. No commit, push, archive or add_session was invoked. Root owns final native geometry/decoration/cleanup and remaining new-panel acceptance plus parent/child progress updates.
