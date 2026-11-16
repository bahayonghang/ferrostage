# Monitor error recovery regression

## Red: existing interface before the product fix

Command: `cargo test -p ferrostage-ui --features test-support --lib --locked monitor_recovery_clears_only_display_error_for_unchanged_topology -- --nocapture`

The regression uses the existing shared panel and sends `Unavailable(IncompleteTopology)` followed by `Changed(original_monitors)`. It also includes a second case with an unrelated PDF error already present.

Result: exit 1; compilation succeeded; 0 passed, 1 failed. Actual assertion:

```text
assertion `left == right` failed: recovered display discovery must remove its stale banner and preserve unrelated errors
  left: Some("device and geometry connection sets disagree")
 right: None
```

The first case fails because healthy topology keeps the prior display error in the shared operation-error slot. Captured full output: `C:/Users/lyh/AppData/Local/rtk/tee/1792376285_cargo_test.log`.

## Green: typed ownership and localized presentation

- The exact regression command now passes: 1 passed, 166 filtered out.
- `cargo test -p ferrostage-ui --features test-support --lib --locked monitor_ -- --nocapture`: 6 passed, 165 filtered out. This includes four new GPUI regressions plus localized platform errors and the existing single-monitor policy test.
- `cargo test -p ferrostage-ui --features test-support --lib --locked locale::tests`: 2 passed, 169 filtered out, including matching locale keys/interpolation fields.
- `cargo fmt -p ferrostage-ui` and `git diff --check`: passed.

The GPUI tests inspect real captured Presenter error data before and after watcher delivery, not just the old operation-error slot. The failure banner is Chinese; after same-topology recovery it is removed. An independently injected startup-watcher failure remains visible, and an unrelated operation error survives both failed and successful topology events. A test-only Presenter probe follows the existing captured-data test pattern.

## Review follow-up: operation entry points must keep the same owner

Independent review found that Apply's fresh discovery, Start's shortcut, and Swap's shortcut could still copy a topology-driven validation error into the operation-error slot. Two further regressions were added and run before the follow-up guards:

- `cargo test -p ferrostage-ui --features test-support --lib --locked monitor_apply_recheck_failure_keeps_discovery_error_owned -- --nocapture`: exit 1, compiled successfully, 0 passed/1 failed. Actual assertion: `left: Some("屏幕检测失败，不能应用上次检测的屏幕。")`, `right: None`. Full output: `C:/Users/lyh/AppData/Local/rtk/tee/1792376715_cargo_test.log`.
- `cargo test -p ferrostage-ui --features test-support --lib --locked monitor_unavailable_start_and_swap_shortcuts_preserve_error_ownership -- --nocapture`: exit 1, compiled successfully, 0 passed/1 failed. Actual F5 assertion: `left: Some("显示器监测不可用；无法安全开始")`, `right: None`. Full output: `C:/Users/lyh/AppData/Local/rtk/tee/1792376779_cargo_test.log`.

Minimal guards now keep invalid discovery owned by monitor state in Start, Swap and Apply; missing-watcher Start also keeps its own existing owner. The new tests cover no prior operation error and an existing unrelated error, actual GPUI F5/S keystrokes, a probe failure between Apply's initial guard and draft application, no pending/audience/config mutation, and same-topology recovery.

Final targeted command: `cargo test -p ferrostage-ui --features test-support --lib --locked monitor_ -- --nocapture` -> **8 passed, 165 filtered out, 0 failed**, 0.47 s. `cargo fmt -p ferrostage-ui` and `git diff --check` also pass after this follow-up. Root owns the subsequent complete UI/product rerun.

A further verified sibling was stale draft validation after a healthy topology change. Its new existing-interface regression, `monitor_stale_draft_validation_recovers_without_operation_error`, failed before the general guard fix: `left: Some("屏幕拓扑已变化。请重新检测，丢弃旧草稿并按当前屏幕重新选择。")`, `right: None` (exit 1, compilation succeeded; `C:/Users/lyh/AppData/Local/rtk/tee/1792376888_cargo_test.log`). The final Apply guard returns without changing the operation-error slot for any live `panel.problem`; the shared validation Alert owns these live problems. The regression verifies actual visible validation, removal after a fresh draft, and retention of unrelated errors. The existing v8 stale-topology test now asserts the live panel reason instead of requiring a duplicate persistent global error.

The complete targeted monitor filter after that fix passes **9 tests, 165 filtered out, 0 failed**, 0.42 s. Distinct delivered Unavailable errors now also emit a structured warning (including `origin = "topology observation"`), deduplicated against the current typed monitor error, so default warning logging retains recurrence diagnostics without logging every poll.

## Full-suite layout regression and correction

Root's first complete GPUI run found **173 passed, 1 failed**: `displays_design_both_hosts_fit_three_rows_and_cancel_only_allocation`. In the embedded normal-window host, the added monitor alert row put Apply at y=665.5..717.5, outside the existing content viewport's y=80..700.5. Full root output is retained in `research/gpui-tests.log`.

The minimal correction presents the localized monitor reason in the existing detection-status slot as a wrapping Alert at helper-text size. Healthy detection still shows its count. This removes the extra alert row while retaining the existing single-scroll ownership and shared allocation actions. No fit assertion, topology size, padding rule or settings-scroll architecture was weakened.

- Exact previously failing test: **1 passed, 173 filtered out**, unchanged assertions.
- `cargo test -p ferrostage-ui --features test-support --lib --locked monitor_ -- --nocapture`: **9 passed, 165 filtered out**, 0 failed.
- `cargo test -p ferrostage-ui --features test-support --lib --locked`: **174 passed, 0 failed, 0 ignored**, 1.30 s.
- `cargo fmt -p ferrostage-ui` and `git diff --check`: passed.

Startup tests construct Shell with an initial TopologyChanging and an independent WatcherStart error in both locales. The first-draw capture reports changing topology; recovered topology removes only that discovery error and exposes the persistent localized watcher-start failure. The test seeds only TestPlatform window lookup to inspect the capture; it does not claim native successful mapping or startup projection.

Both shared panel hosts and both locales render failure-specific recovery help and the existing Detect again control. Real GPUI mouse dispatch exercises disabled Apply and then the existing native retry path, verifies refreshed draft revision and unrelated error retention, and checks that the synthetic old error cannot survive. Native retry success/failure is accepted according to the actual query; a deterministic Changed event separately proves recovery help disappears. Failure during Presenting immediately removes audience output, changes state to Ready, and publishes Curtain. No physical monitor or size matrix is claimed here; root owns full CI, native product checks and the final acceptance record.
