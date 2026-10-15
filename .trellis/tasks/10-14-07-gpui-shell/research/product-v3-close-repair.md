# Native close-callback repair (v3)

Root's v2 launch exited with `0xC0000409` and `cannot update Shell while it is already being updated`. The full stack in `native/v2-trace.stderr.log` establishes `Shell::tick` handling a topology event → `retire_pending` → `Shell::close` → GPUI `update_window_erased` → synchronous `on_window_closed` → a second Shell lease. It does **not** establish a placement timeout: the stack enters topology cancellation before the placement poll. The previous interpretation was corrected before implementation. Root owns the native receipts and desktop operations.

Only `crates/ferrostage-ui/src/shell.rs` changes between frozen v2 and v3; all601 other source inputs, the lock, vendor sources, policies and features are identical. `product-v3-evidence.py` reconstructs v2 Shell from retained startup-red source plus the v2 repair patch, verifies its exact frozen hash, and emits the focused v3 patch. This avoids presenting the complete uncommitted task tree as the bug-fix delta.

The production close handler now schedules the original WindowId with `App::defer`. Consumption occurs after the current entity lease, using the existing current/pending/retired classification. No callback is discarded and no panic is caught. The same registration function sets the public `QuitMode::Explicit`: topology cancellation can temporarily close the only pending window, and zero-screen recovery must keep the app alive. Closing the current Presenter still invokes the coordinator's explicit quit branch; current Audience close curtains/stops while preserving Presenter. Retirement of old or cancelled windows does neither.

Placement predicates are unchanged. Diagnostic observations now include requested and actual display IDs, requested and actual fullscreen state, actual window/display bounds and bounds-event flag. Timeout and other pending cancellation log their reason before removal. Topology events log previous monitors/validity and the complete incoming Changed/Unavailable event. The v3 freeze was created with the existing freeze writer, whose observations list still names the former `window placement confirmed` line: v3's actual line is `window placement observation` with `reason="confirmed"` (or the cancellation reason).

## Regressions and results

The three new tests use the production close-handler registration, real GPUI `open_window` and `remove_window`, and actual coordinator branches. All three reproduced the native re-entry before the fix. They exercise topology cancellation of a pending Presenter, timeout cleanup of a pending Audience while retaining Ready/current Presenter, and committed replacement cleanup while preserving Live/page/timer. The replacement test then closes current Audience and Presenter through real GPUI removal to prove the callback still stops/quits through the correct identity branch. The pre-existing first-draw test now also installs the real close callback.

TestPlatform creates no OS windows. Its platform quit method is a no-op: the tests prove entry into the current-Presenter quit branch through coordinator state effects, not native process termination. Native survival across startup topology events, actual placement, focus and shutdown remain Root's v3 acceptance work.

| Command / receipt | Actual result |
| --- | --- |
| `cargo test -p ferrostage-ui --features test-support --lib close_hook --locked --offline`; product-v3-close-red.log | exit101, sandbox shared-target lock denied; no test executed |
| Same command with approved cache access; product-v3-close-red-retry.log | exit101, all3 tests reproduce entity_map.rs164 update re-entry |
| `cargo test -p ferrostage-ui --features test-support --lib --locked --offline`; product-v3-close-green.log | exit101, new diagnostic variable `display` collided with tracing macro expansion; no test executed |
| Same command after `observed_display` correction; product-v3-close-green-2.log | exit0,22 tests |
| `just ci`; product-v3-just-ci.log | exit0,145 default tests, fmt/strict lint/default deny pass |
| `cargo clippy -p ferrostage-app -p ferrostage-ui --all-features --all-targets --locked --offline -- -D warnings`; product-v3-clippy.log | exit0 |
| `cargo build -p ferrostage-app --features gpui,pdfium --locked --offline`; product-v3-build.log | exit0 |
| `git diff --check`; product-v3-diff-check.log | exit0 |

The unchanged complete dependency graph retains v2 full all-features deny evidence in product-test-support-deny.log; no additional dependency/license waiver is requested. Unchanged vendor compatibility tests, core dependency-tree and read-only CLI evidence retain their recorded source applicability. The upstream HarfRust direct-lint limitation remains as previously reported. No unrelated performance or hardware tests were repeated.

## Bug analysis and prevention

1. **Root cause categories:** cross-layer contract, implicit assumption and test coverage gap. GPUI synchronously renders during open and invokes close subscribers during remove; neither callback inherently runs after the controller's entity lease.
2. **Why v2 was incomplete:** the first-draw repair addressed its reproduced read lease. The original headless test opened and removed windows without installing the application's global close subscriber, so it did not execute the failing update path. A green view-only harness could not establish application lifecycle safety.
3. **Prevention:** the tests now share the production close-handler registration and execute actual transaction commit/rollback and current-versus-retired callbacks. Deferred event consumption is at the callback boundary, not a catch around crashes. Strict tracing compilation also checks all new diagnostic expressions; compilation failures are separate from behavioral red results.
4. **Broader boundary:** replacement, timeout, Stop and topology all eventually use the same close path. Explicit quit is required whenever temporary window absence is part of a supported lifecycle. No claim is made about native focus/placement from these headless tests.
5. **Knowledge capture:** Root already added both synchronous-open and synchronous-close contracts to frontend/component-guidelines.md and the parent/task07 designs. This report records the coverage gap and tested remedy; shared spec ownership remains with Root. No commit or journal operation is authorized.

## Frozen handoff

`product-native-freeze-v3.json` SHA256 `0c0cd228e6c8f00de75a9976b6ed653d93cb4dc1f745c8f59d44cd9e74ae0e5e` records602 source inputs with aggregate `7504bbec590d2c3df76a125350f8142838dc5f894ca8e76c1ee4802ee871c27a`. Executable SHA256 `fa8a85611c42a1039f172534c3b1200cfc525cb857459ef0d10fa1e3d359d74b`,36,945,408 bytes. Lock SHA256 remains `2557e10b37fa5cfe63e78ce19965b8bbbd83438bdd540eed50405fbbc3dbc23e`. Focused repair patch SHA256 `a60cd6e38024a49a51a8185e533b912d98331aa27d9db556f8e7a18711041b35`.

Executable resolves to `D:/mbx-cache/targets/v1/b228c6f37bcb27af7d1a2d6821dcfcb308164a0bd33ec7c00848c486151f07e0/debug/ferrostage.exe`. Use the existing Root-owned isolated `--config`, exact `--pdfium-library cache/pdfium-7881-win-x64/bin/pdfium.dll`, and optional PDF positional path. No implementer native launch has occurred. Product code and binary are frozen for Root; only evidence documents are being finalized. Physical unplug/port changes and unavailable hardware modes remain NOT_RUN, and Gate B remains incomplete.
