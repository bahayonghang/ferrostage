# Final integrated review — 2026-10-19

Reviewed the completed transition and timing integration after the timing worker
released Shell ownership. This completes the code review begun in
`review-stage1.md` and `review-stage2.md`; native acceptance and version/release
remain parent-owned. No commit, version edit or task archive performed here.

## Findings (fixed)

### Failed reload worker could restart timed playback from retained old pixels

- File: `crates/ferrostage-ui/src/shell/reload.rs`.
- `reload_blocks_render()` included the `failed_worker` recovery latch, but
  `reload_blocks_playback()` did not. After a failed commit rollback removed the
  pending transaction, retained old pixels could receive a fresh playback identity
  while the worker remained invalid. A matching paint acknowledgement could then
  restart timed navigation before recovery.
- Added the same latch to the playback gate. Recovery must clear it and obtain a
  fresh identity/paint acknowledgement before automatic progression can resume.
- Added `timed_reload_worker_failure_latch_blocks_retained_pixels_until_recovery`:
  it verifies retained visual continuity, absent playback metadata during failure,
  rejected old events, no navigation and fresh identity after recovery.
- RED: the new test failed at the absent-playback assertion before the guard fix
  (`1792386522_cargo_test.log` in the local RTK tee directory).
- GREEN: the three `timed_reload_` regressions passed after the fix; the current UI
  test inventory is 205 tests. This focused run is not a claim that this reviewer
  reran all 205 tests.

The earlier enlarged-Fly endpoint and deferred paint/blank acknowledgement fixes
remain integrated. Their independent red/green proofs and fixture extensions are
recorded in `review-stage2.md`.

## Findings (not fixed)

No unresolved production-code blocker found in this review. The earlier timing
standards discrepancy was resolved by the timing worker: positive `/Dur` now starts
at successful `Settled.at`, after the transition. The reviewed implementation and
current task documents agree.

Native all-style motion, dual-display interruption, standing Notes readability,
the thesis example and final version 1.1.0 verification are still parent-owned
acceptance work. This review does not promote headless scene/raster checks into
native visual evidence. Consult the parent's final native receipt for their result.

## Integrated flow checked

- Structure metadata becomes owned core data and is exposed only for mapped content
  pages. Compiled overlay steps retain the existing physical/content/notes mapping.
- Shell selects an identity for accepted navigation, then attaches it only to an
  exact generation/page/crop/Audience-priority/target raster. Pending destinations
  retain the old image and old identity; no premature destination acknowledgement.
- Audience uploads before playback and acknowledges successful paints. Deferred
  delivery rechecks Live/unblanked identity before consuming its delivered flags.
  Failed upload retains the previous public image and emits no false acknowledgement.
- Shell rejects stale source-window, generation, navigation and stage events.
  Placement, loading and reload gates prevent obsolete clocks from arming.
- Positive dwell uses settlement plus full duration. Zero dwell requires both paint
  phases and a later poll turn. No recursive stepping, catch-up or final-step wrap.
- Explicit pause survives manual navigation. Focus, modal/contextual UI, blanking
  and render failure freeze the remainder. Annotation/pointer interaction settles
  effects and explicitly pauses. Stop/reload/window/document changes invalidate old
  identities; failed-worker recovery is now included.
- Audience snapshots and metadata lookup exclude Notes; render/transition resources
  stay under the existing cache and shared budget. No dependency or vendor changes.
- Known active-content detection is bounded, reports through existing warning UI,
  does not execute or log scripts, and does not mistake static OCG for animation.

## Spec synchronization

Added `.trellis/spec/frontend/pdf-playback.md` with executable signatures,
cross-layer invariants, error matrix, examples and regression requirements.
Linked it from the frontend index, state-management contract and task reading map.
Clarified that the existing no-timer-navigation rule concerns the talk timer;
authored PDF dwell is a separate scheduler.

## Verification

- Tests PASS: `cargo test -p ferrostage-ui --features test-support --lib timed_reload_ --locked --offline` — 3 passed, 202 filtered.
- Lint and native type-check PASS after the final guard change:
  `cargo clippy -p ferrostage-ui --all-targets --features gpui,pdfium --locked --offline -- -D warnings`.
- Formatting PASS: scoped `rustfmt --edition 2024 --config skip_children=true --check crates/ferrostage-ui/src/shell/reload.rs`.
- Whitespace PASS: `git diff --check`.
- Parent reported `just ci`, the 204-test shell suite, native PDF tests and native
  Clippy passed before this final guard/test addition. Those are parent evidence,
  not independent reruns here; root was notified production edits were finished
  before its latest native rebuild.
