# Independent V11 review

Active task: `.trellis/tasks/10-14-07-gpui-shell`.

## Findings (fixed)

None. No product, specification, dependency, configuration or executable changes were made by this reviewer.

## Findings (not fixed)

No source defect was found in the V11 delta. Historical native V10 F10 remains FAIL: its first request produced a smaller actual client, strict confirmation timed out, and Root explicitly terminated the remaining process with exit-1. That is not normal-close PASS. V11 source/model checks cannot establish Windows convergence or native-frame usability; Root must repeat the C-screen startup against this exact build.

Gate B remains INCOMPLETE. Remaining native display-panel, migration/no-play and persistence acceptance is owned by Root. Physical unplug/reconnect/port changes, latency and actual single/zero-display hardware cases remain NOT_RUN; no08–11 integration is authorized by this review.

## Scope and reasoning

- The frozen delta from V10 contains only `shell.rs` and Root's corresponding frontend state-management contract;607 of609 inputs remain unchanged. Policy geometry, vendor code, dependency versions and acceptance tolerances are unchanged.
- The initial windowed resize records its API request and pre-request actual size, and clears the bounds-observed flag before invoking resize. A second request requires both a subsequent bounds observation and a different actual size. An unchanged pre-request event cannot authorize compensation.
- The second request uses `previous.requested + (target - actual)`, with finite per-axis error at most32 logical pixels. The `corrected` bit is never reset, so a pending window can receive at most two requests. Fullscreen records bypass this path. Polling stops correcting after the existing deadline.
- Confirmation still reads current geometry and requires the immutable fitted client target within2px plus existing display/mode and usable-area containment. A callback alone cannot commit a wrong size, position or mode. Fullscreen retains its native-event and full-display authority.
- Correction operates on pending replacement records. Failure continues through existing timeout/rollback before settings, core state or projection commit. No-play cannot stop the running session on an unconfirmed replacement. Existing current/pending/retired ownership, page/blank/timer and shared draft/Identify/save ownership are unchanged.
- The measured Windows-response test uses real GPUI TestPlatform bounds callbacks and rounds each API request at150% before subtracting the observed4×9 physical-pixel frame discrepancy. The retained red log proves V10 failed this new assertion; V11 passes. This model is useful regression evidence, not a Windows desktop substitute.
- The second new test covers an error beyond32px and an in-range response that fails to converge after correction. Repeated polls cannot oscillate or issue another resize; timeout preserves the running core state, settings, original presenter/audience and Live snapshot. The updated V10 no-response test retains its rollback assertions and explicitly proves a stale unchanged callback does not consume the correction path.
- Root's contract accurately records the bounded observed-response rule and native verification requirement. No further specification or template changes are needed for this delta.

## Verification

| Check | Result |
| --- | --- |
| Independent lint | PASS: app/UI all features/all targets, locked/offline Clippy `-D warnings` |
| Independent TypeCheck | PASS: app/UI all features/all targets, locked/offline `cargo check` |
| Independent affected regressions | PASS6: V10 policy/size/no-response rollback and both V11 response paths |
| Independent diff check | PASS |
| Verified complete UI suite | PASS70:47 actual GPUI tests and23 pure; reused final V11 receipt |
| Verified default `just ci` | PASS150 tests plus formatting, default lint and dependency policy |
| Verified full dependency policy | PASS: all features, locked/offline deny |
| Verified native build | PASS: locked/offline `gpui,pdfium`; no reviewer launch |
| Independent frozen read-back | PASS609 source inputs and executable; implementation25/native V10 11/prior review15 manifest records all match |

Reused command arguments, exits and test counts are recorded in `product-independent-v11-reused-checks.json`. The measured-response red test and the implementer's earlier test-expression compile failure remain intact. The existing HarfRust dependency warning remains visible; no waiver was added. No full-suite or build repeat was necessary because no reviewed input changed.

## Exact handoff

- Freeze: `product-native-freeze-v11.json`, SHA256 `db41074638d3d844e8a69c5a1575c1f507c8558ba2523e700048b16058302716`.
- Source aggregate (609 inputs): `fdec41e24793817d526127a3928c9e1f0bfafa021a293dce26cc9125f87c48f1`.
- Executable SHA256: `027ab17fc23a92c00c480148b06c1ac115f81fcfe1a2145ac210dd6dd79a4d6d`.
- V11 implementation25 manifest SHA256: `f74a13797a0375f5c56a44e2be9526b993b237e72d75f342fc5bf0cb40eec970`.
- Native V10 11 manifest SHA256: `ad8cbffb28e7f8a90b0ed89c6ab3cc4f0171f5b0ef724c8982425d69e6426e24`.

Source review is complete and frozen V11 is ready for Root's native retest. All reviewer Cargo commands returned before the explicit handoff message; subsequent actions only wrote new review evidence. Final source/executable read-back still matches. Root exclusively owns native launch and desktop; this reviewer has no running command and made no commit, push, archive or add_session call.
