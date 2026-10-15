# Task 03 implementation verification

## Current result after independent review — 2026-10-14

Technical review PASS after one local correction. winsafe `has(mask)` tests any overlapping bit; the adapter now requires all four DEVMODE geometry fields. The regression failed before the correction and passes afterward. No public API, dependency or lint-policy change. See independent-check.md for the review and remaining scope.

| Current command / evidence | Result |
| --- | --- |
| `just ci` / independent-just-ci.log | exit0; fmt, workspace clippy, 93 tests, deny PASS |
| `cargo check --workspace --all-targets --locked` / independent-typecheck.log | exit0 |
| `cargo run -p ferrostage-platform --example monitor_probe --locked` / independent-native-static-probe.log | exit0; ten stable three-screen samples, required fields and width consistency accepted, first Changed, stop/join |
| Regression / independent-regression-before.log → independent-regression-after.log | exit101 with missing POSITION accepted → exit0 with each required field enforced |

The fresh CI includes Gate A's **83 tests**: core21 + settings38 + platform22 + settings-topology integration2. There are no ignored tests. Current code/manifest fingerprints (62 files), inspected dependency sources (9 files), and the rebuilt probe are in independent-code-sha256.json, independent-dependency-source-sha256.json and independent-native-binary-sha256.json. Only windows.rs and windows/tests.rs changed from the previous code record. The initial sandbox regression attempt failed to open the build lock and did not run tests; its log is independent-regression-sandbox.log. Authorized cache-access retries completed the commands above. No checks remain running.

Physical display changes and actual GPUI placement remain NOT_RUN. Task03 is technically ready for downstream integration, with its manual acceptance open. This is not full task03, parent AC-2/AC-3 or Gate B completion.

## Implementation handoff evidence before the independent correction

Recorded: 2026-10-14T15:16:55.316060+00:00. Branch `main`, HEAD `ab78a84d96c7cf4a0bbc8dfd4f8e45da180bafb3`. Worktree remains uncommitted; other tasks' changes were preserved. Exact current code/manifests are recorded in `verified-code-sha256.json` (62 files). No commit, push, archive or journal operation.

The following original records are retained as history for their fingerprints. The current post-review code is covered by the fresh evidence above.

### Initial result

Implementation and automated/native-static checks PASS. Full task acceptance is pending the explicitly unavailable physical display operations. User cannot perform unplug/port changes this turn. Independent Trellis check is the coordinator's next step.

| Command / evidence | Result | Scope |
| --- | --- | --- |
| `cargo test -p ferrostage-platform -p ferrostage-core` / targeted-tests.log | exit0, 42 tests | Initial adapter + pure contracts; superseded native adapter thereafter |
| `cargo test -p ferrostage-core -p ferrostage-settings -p ferrostage-platform --all-targets` / gate-a-tests.log | exit0, 82 tests | Initial adapter; final equivalent suite also ran in final CI below |
| `cargo tree -p ferrostage-core --edges normal` / core-tree.log | exit0 | Only thiserror/tracing production roots; no IO/native dependency, unchanged core manifest |
| `cargo clippy -p ferrostage-platform --all-targets -- -D warnings` / platform-clippy.log | exit0 | Initial adapter; final workspace clippy covers final implementation |
| `cargo run -p ferrostage-platform --example monitor_probe --locked` / native-static-probe.log | exit0 | Final winsafe-only adapter: ten stable three-screen samples, complete physical geometry/identity, initial Changed and stop/join |
| `just ci` / just-ci.log | exit0 | Final fmt, workspace clippy, 92 tests, cargo deny advisories/bans/licenses/sources PASS |
| `git diff --check` | exit0 | No whitespace errors (line-ending notices only) |

Final CI includes the Gate A subset: core21 + settings38 + platform21 + actual settings-to-topology integration2 = **82 tests**. Other workspace tests total10. No ignored tests. Non-failing duplicate syn/windows-sys warnings are unchanged policy. Default Windows builds include the native adapter and compile the developer probe; the probe is run separately, not assumed from test compilation. Linux/macOS native execution is NOT_RUN and returns typed UnsupportedPlatform by design.

## Changes and data flow

- Core owns `MonitorId`; existing MonitorRole and lifecycle remain intact. Settings is only a platform dev-dependency for real TOML-to-resolver integration tests.
- Platform owns owned MonitorInfo, explicit source/confidence, typed errors, pure role resolution, sorted diff/ambiguity, and one cancellable worker. Saved binding order/None/absence take precedence. Any collision disables Audience; only Presenter falls back.
- Windows-only winsafe `=0.0.29` gdi/user safe APIs; no project unsafe or lint exceptions. Exact source/license evidence in api-evidence.md and dependency-source-sha256.json.
- The watcher is a documented polling fallback: 200 ms cadence plus stable-change interval. Unavailable is immediate and retains diagnostic last_known only; 07 must disable projection/stop safely and never auto-start on recovery. In-flight OS calls are not interruptible.
- Native geometry comes from DEVMODE. GdiDeviceCapsRatio is current-context ratio, not effective-DPI evidence. 07 must validate actual GPUI coordinates/focus/window placement.

## Failures retained, not relabeled

- `targeted-sandbox.log`: sandbox registry DNS failed before tests; authorized retry succeeded.
- `native-probe-compile-first.log`, `native-winsafe-first-compile.log`: developer probe borrow error and winsafe trait/enum compile corrections; later runs passed.
- `just-ci-first-deny-failure.log`: tests92 passed but cargo deny rejected display-info's fxhash via RUSTSEC-2025-0057. The dependency was removed, not ignored. `native-static-superseded-display-info.log` describes that abandoned adapter, not final evidence.

## Remaining acceptance

See stability.md for exact commands and observations. Physical unplug/replug ≤1s, port-change identity recovery, same-model hardware, actual missing-identity hardware, rotation/DPI mutation and actual GPUI placement remain NOT_RUN. Existing portrait geometry and mixed ratio values are static observations, not mutation tests. Gate B / parent AC-2 and AC-3 cannot be claimed from these results. The technical topology delivery is ready for 07 to consume after independent review; its manual gates remain external prerequisites at their stated scope.
