# Gate A — core, settings and topology contracts

Recorded: 2026-10-14T15:21:39.084016+00:00. Branch `main`; HEAD `ab78a84d96c7cf4a0bbc8dfd4f8e45da180bafb3`. All changes are uncommitted. This is a technical gate; commit/archive is outside this authorization.

Status: PASS for the pure-contract technical gate after task03 independent review and regression fix. Task03 full manual acceptance remains pending.

Latest applicable regression: task06 independent CI passes128 tests, including90 Gate A contract cases. Task07 has not changed those product inputs. The93/83 counts below describe the original task03 checkpoint; later refinements and superseding hashes are recorded at the end of this report.

## Initial task03 checkpoint evidence

- Final task03 `research/independent-just-ci.log`: `just ci` exit0, formatting, Clippy/type-check, 93 workspace tests and cargo deny PASS. It contains the full Gate A subset: core21 + settings38 + platform22 + settings-topology integration2 = 83 tests, zero ignored.
- Task03 `research/independent-code-sha256.json` records 62 source/manifest inputs for that run, including the final winsafe-only adapter. The earlier standalone `gate-a-tests.log` used the superseded adapter; the final CI is the current-code evidence.
- `cargo tree -p ferrostage-core --edges normal` in task03 `research/core-tree.log`: only thiserror/tracing production roots; no OS/IO/GPUI/PDFium/lopdf/notify. The core manifest is unchanged by the adapter replacement.
- Task02 `research/check-review-2026-10-14.md`, `windows-persistence.log`: recursive settings preservation, typed invalid-value fallback, future-version read-only behavior, read-only CLI, and actual Windows NTFS replacement/failure/retry checks. Current full CI reruns the settings tests.
- Task04 `research/independent-check.md`: lifecycle/timer/preview/descriptor/frame isolation reviewed; its core tests reran in the final task03 CI.

## Cross-task coverage

Actual settings TOML parsing is mapped into core MonitorId/MonitorRole tuples and resolved by platform. Core owns the pure display types; settings remains a platform dev-dependency only. Tests cover zero/single/multiple displays, absent or explicit-none Audience, duplicate bindings/roles, collision refusal, deterministic area ties, and Presenter fallback without Audience takeover. Time is injected into lifecycle tests; preview cannot change the committed audience page. Error paths preserve valid state.

## Native boundary

Windows x86_64 MSVC, Rust1.99.0; hardware inventory in native-environment.json. Task03 final read-only static probe observed ten consistent three-screen samples, complete per-monitor geometry/cap checks, initial Changed and explicit cancellation. See task03 research/stability.md and independent-native-static-probe.log. This is not physical unplug, port-change, same-model identity, effective-DPI, actual GPUI placement/focus or window timing evidence.

The user cannot perform the physical operations this turn. Task03 manual acceptance and parent AC-2/AC-3 remain pending; Gate B/C/D are not passed. Gate A's pure-contract scope permits a separate technical result while those manual checks remain open. No acceptance threshold was relaxed.

## Initial task03 independent sign-off

Task03 research/independent-check.md reports no remaining code defects after fixing the any-bit DEVMODE field-mask predicate. A failing regression became passing; each required field is now independently enforced. Independent CI/type-check/native probe passed, all62 checkpoint code hashes and the executable hash matched. Those commands returned. The superseded92-test run and initial82-test standalone gate log are historical only; the accepted task03 checkpoint used93/83. Later cross-task evidence follows.

## Subsequent cross-task regression

Task05 refines the pure BgraFrame storage to private Arc<Vec<u8>> and read-only slice/capacity access. Its independent final `just ci` (`.trellis/tasks/10-14-05-pdf-raster/research/independent-ci-closeout.log` from repository root) passes109 workspace tests, including the existing83 GateA tests; core/pdf/render native suite passes42. The current task05 `independent-check-manifest.json` supersedes the earlier whole-workspace hashes for changed core frame and raster files. GateA behavior remains passed; this is no new physical-monitor or GPUI evidence.

Task06 adds checked content maps/reconfiguration and closes the existing direct crop-setter validation bypass. Its independent final `just ci` (task06 `research/independent-ci-closeout.log`) passes128 tests including core28 + settings38 + platform22 + settings-topology2 =90 relevant contract tests. Native suite63, typecheck and pure-core dependency audit also pass. Task06 `independent-check-manifest.json` records current inputs; earlier hashes/counts are historical. No physical-monitor or GPUI acceptance is implied.
