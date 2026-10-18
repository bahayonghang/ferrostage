# Task 04 implementation verification

Recorded: 2026-10-14T14:45:09.302752+00:00 (UTC). Branch `main`, HEAD `ab78a84d96c7cf4a0bbc8dfd4f8e45da180bafb3`. Working tree intentionally uncommitted; source hashes and complete status are adjacent files.

| Check | Result | Evidence |
| --- | --- | --- |
| Regression-first `cargo test -p ferrostage-core preview_` | Expected FAIL: two new privacy/navigation regressions, one existing PASS | `preview-red.log` |
| `cargo test -p ferrostage-core -p ferrostage-document` | PASS: 21 core + 1 document, no ignored tests | `core-document-tests.log` |
| `cargo tree -p ferrostage-core` | PASS: production thiserror/tracing only; proptest is dev-only | `core-tree.log` |
| `just ci` | PASS exit 0: fmt, clippy -D warnings, 70 workspace tests, deny | `just-ci.log` |
| `git diff --check` | PASS exit 0 | Executed after final code/docs edits |
| Production core source inspection | No clock sampling, file/thread IO, swap flag or old apply call | `state.rs`, `timer.rs`, source hashes |

The initial sandbox regression attempt could not open the existing target build lock. The same command then executed with approved escalation and produced the expected failures. Targeted and CI checks also used that cache access. No failed or running command is counted as a pass.

## Delivered contract

- `idle()` plus `LoadDocument` enters Ready; `new(page_count)` is Ready. Explicit Start gates audience availability and starts zero elapsed time; Stop preserves page/strokes and freezes elapsed; Close clears state.
- Every command uses `apply_at(command, Instant)`; only tests sample a clock. Pause/Resume are idempotent in Presenting; Reset preserves pause; illegal commands preserve the entire state.
- Snapshot stage is Curtain outside Presenting, Live inside; core metadata always has `visual=None`. Preview/preview navigation never changes committed audience page until Commit.
- Packed immutable `BgraFrame` validates dimensions, multiplication overflow, stride and byte count. Source-free `AudienceVisual` / `AudienceMediaFrame` contain only allowed pixels and geometry; Debug omits raw frame bytes.
- `CommitDocument(DocumentDescriptor)` is the only metadata commit implementation. It validates strict newer generation/nonempty pages, clamps committed page, clears preview, preserves stage/timer. Temporary StageShadowDocument delegates to it without storing bytes; task10 replaces the legacy byte payload.
- Core rejects SwapScreens with ScreenSwapRequiresCoordinator and has no independent role flag. Task07 must intercept it.
- All current callers updated. The document notes test now exercises Ready/Live/Stop, checks projection equality with/without notes, and retains the secret-note assertion. Random command sequences check error atomicity, preview privacy and stage invariants.

## Files and boundary

Code: core `command.rs`, `error.rs`, `lib.rs`, `snapshot.rs`, `state.rs`, new `session.rs`, `timer.rs`, `frame.rs`, `state/{tests,lifecycle_tests,property_tests}.rs`; document `lib.rs` test consumer. The pre-existing core `display.rs` and settings/app implementations were preserved.

Docs/spec: lifecycle/snapshot/role ADR current-state statements, architecture current-state statements, frontend state/type/hook guidelines, backend quality/logging guidelines. Task04 design/implementation/acceptance plus these evidence files updated; parent progress index belongs to the main session.

## Remaining work

Independent Trellis check passed on 2026-10-14 14:49:28 UTC; see `independent-check.md`. All 51 code/manifest hashes still match this implementation evidence. The reviewer independently reran formatting and Clippy/type-check, fixed only the PRD's stale elapsed accessor reference, and found no implementation defects. This fulfills task04's pure core implementation/automated checks, not parent Gate A (task03 topology still required). Actual window lifecycle, multi-monitor behavior, PDF pixels/notes isolation, GPU/performance and media evidence are NOT_RUN here and remain tasks07/08/11. No task status was manually changed and no commit/push/archive/journal was performed.
