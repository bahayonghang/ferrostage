# Task08 implementation handoff — 2026-10-15

Status: implementation delivered for serial independent review. No commit, push, archive or journal operation. Native desktop smoke is owned by the parent session and remains NOT_RUN in this report. Extended hardware/size/DPI/theme/performance/soak qualification remains DEFERRED_BY_USER under acceptance-policy.md.

## Implemented scope

- W-1/W-2/W-11: real candidate import (owned read-only bytes and structure → existing single PDFium worker metadata → LoadedDocument → prerender → worker commit acknowledgement → mapped core/document install); current/next and ordered split/independent raster notes; notes scale changes both rendered dimensions and scrollable content; same-generation layout/notes recognition/physical navigation controls retain stage/timer. Failed builder candidates abort and preserve the old active renderer. The existing Displays/Identify and save/Retry owner are reused.
- W-3/W-7: elapsed-based up/down timer, overtime and warning colors, pause/resume/reset, local wall clock, logical/overlay/physical progress, numeric and thumbnail previews, explicit Commit/Cancel. Numeric commit to a private notes page cannot commit an older valid preview by accident.
- W-4/W-10: exact generation/page/crop/target/priority frame authorization. Waiting preserves the old complete projection identity/pixels and clears wrong-page overlays. Curtain and black/white cover immediately. Audience only receives its snapshot and owns no document/cache request capability.
- W-5/W-6: shared letterbox transform, inverse normalized input, per-page session strokes with Begin/End, finite bounds validation, configured BGRA/width, highlighter alpha, segment-aware erasure, page-local clear, configured LaserStyle. No PDF writeback. Pointer settings flow via the approved checked core command, not settings access from Audience.
- W-8: independent grid column/visible-page counts, asynchronous Thumbnail-priority/180px L3 jobs, bounded visible rows with wheel/paged virtual navigation and a compact strip. Hover previews do not navigate the committed audience.
- W-9: explicit GPUI image retirement on replacement/eviction/release; visible frame priority precedes retained thumbnails; actual width×height×4 GPU accounting; audience and presenter each use half the configured L1 allocation budget (Audience target is fitted to its allocation). Generation/DPI/desired-window changes retire obsolete frames. ±2 prefetch uses the render worker. Debug shows owned bytes, raster duration and measured UI paint-call duration; this is not an actual-Present performance claim.
- W-12: existing theme tokens, existing accessible button/focus primitive, stable ids, bilingual new control catalog, unmodified BGRA PDF pixels. Full native visual matrices are deferred.

No unsafe/vendor/waiver changes. The optional direct image0.25.10 and chrono0.4.45 dependencies reuse versions already in Cargo.lock. Core still has no IO/GUI/PDF dependencies.

## Concrete files

Core: command.rs, state.rs, geometry.rs, snapshot.rs, error.rs, lib.rs and state/annotation_tests.rs.
UI: actions.rs, shell.rs, presenter.rs, presenter/workbench_view.rs, audience.rs, document_load.rs, workbench.rs and workbench/tests.rs, texture_cache.rs, slide_surface.rs, lib.rs and Cargo.toml.
Shared dependency declaration/lock and assets/locales/{en-US,zh-CN}/app.toml.
This report, initial failing receipts, final CI/build logs and the task08 implement checkpoint. Parent policy/design/state and task09 design changes belong to the parent session.

## Verification receipts

| Check | Result |
| --- | --- |
| `just ci` | PASS;158 default workspace tests, fmt, strict default clippy, deny; full output implementation-ci.log |
| `cargo clippy -p ferrostage-app -p ferrostage-ui --features gpui,pdfium,test-support --all-targets --locked --offline -- -D warnings` | PASS; unchanged dependency HarfRust dead-code warning remains, no product waiver |
| `cargo test -p ferrostage-ui --features test-support --lib --locked --offline` | PASS89, including retained07 actual GPUI focus/window lifecycle tests and new real-control/mapped-note/texture tests |
| `cargo test -p ferrostage-ui --features gpui,pdfium --lib --locked --offline` | PASS29 with verified local PDFium DLL; includes actual compiled Beamer notes pixels, candidate commit/abort and five thumbnail renders |
| `cargo test -p ferrostage-pdf -p ferrostage-render -p ferrostage-document --features pdfium --all-targets --locked --offline` | PASS36 retained native checks; missing-DLL negative integration remains included |
| `cargo deny --all-features --locked --offline check` | PASS advisories/bans/licenses/sources; existing duplicate/source warnings preserved |
| `cargo build -p ferrostage-app --features gpui,pdfium --locked --offline` | PASS; final receipt implementation-native-build.log |
| `git diff --check` | PASS; ordinary LF/CRLF informational warnings only |
| Audience forbidden-type search | No SlideNotes/notes_crop/LoadedDocument matches |

The final notes-surface ordering change is GPUI-only and received strict all-feature/test-support lint plus a rebuilt product after the test suites; the default158 tests do not compile that view. No check is claimed to establish physical displays, native pixels, VRAM or actual Present timing.

Initial headless test failures are retained in implementation-initial-headless-failures.log and initial lint findings in implementation-initial-feature-lint.log. All local lint findings were repaired.

### Three historical07 headless failures

- `unsupported_shortcuts_use_user_visible_feature_names`: renamed `workbench_shortcuts_are_wired_without_stale_feature_warnings`; laser/pen now have real bindings, so the old expectation of unwired warnings is no longer valid. Both locales remain checked.
- `v5_nonbutton_navigation_and_disabled_controls_keep_their_scope`: kept its assertions unchanged. Real workbench tools require a LoadedDocument; the historical identity-only shell fixture still exercises its original Displays/disabled-Start tab order. New `workbench_controls_reconfigure_preview_and_keep_note_pages_private` exercises an actual tool click with a real built map.
- `v6_retry_success_clears_save_feedback_and_restores_keyboard`: kept all save/read-back and keyboard restoration assertions unchanged; the same proper LoadedDocument eligibility keeps its identity-only fixture's Retry tab order.

`v6_import_and_swap_keep_runtime_results_when_saving_fails` now supplies a validated owned metadata+structure session to the new final commit path; it retains dirty/Retry/durable role/read-back assertions. The separate native PendingLoad test now covers the real worker preparation/raster/commit path removed from that older metadata-only fixture.

## Next consumers and review focus

09 should consume Workbench settings through the existing SettingsSession; ratios/notes scale/clock/grid/timer/pointer/debug/L1 already have consumers. Surface target changes are deferred intents using actual canvas bounds×DPI. The same save owner handles ratio edits. A readonly future schema disables ratio changes.

10 should extend `document_load::PendingLoad`/Shell finish_import with its external-replacement commit protocol, map-position/annotation remapping and transactional retirement. The08 new-import operation is serialized by shared import eligibility while loading, so another import cannot orphan a pending Commit acknowledgement. Closing the application cancels shell foreground tasks and exits; there is no surviving shell with a mismatched active worker. Scheduler desired-set changes advance the worker epoch before replacement jobs; retained completed frames are reused only for the same pixel identity. Executing PDFium calls remain non-preemptible.

11 should add authorized media via `workbench::assemble` and the shared surface painter, preserving Curtain/blank precedence and source isolation. Current visuals contain no media backend or placeholder media implementation.

## Native handoff

Final executable: `target/debug/ferrostage.exe`,41,012,224 bytes, SHA256 `B96AAD418AC58CA6818216DA17FE1685FD6BC43784FA1A7D706ACF37CCB06A58`. It was rebuilt after default CI (default all-target builds overwrite the binary with the non-GPUI variant).

Launch with the parent-owned isolated `research/native/workbench-smoke.toml`, `--pdfium-library cache/pdfium-7881-win-x64/bin/pdfium.dll`, and positional `fixtures/notes-functional/pympress-notes.pdf`. Controls have stable ids: workbench-progress, previous/next, black/white, laser/pen/highlighter/eraser, clear-strokes, grid, document-options, layout-*, independent-notes, physical-pages, horizontal-split, vertical-split, timer-pause/reset, commit-preview/cancel-preview. G uses the retained numeric modal; thumbnails hover without committing; Enter or Show preview commits; Escape cancels. Displays is unchanged.

Native desktop smoke and independent review are required before the parent reports the functional gate passed. No combined original acceptance checkbox was changed.


## Upload recovery follow-up — 2026-10-15

This section supersedes the earlier W-9 texture lifecycle description and executable handoff. The earlier binary hash is **not** the current source identity. The independent checker owns final full CI and the subsequent gpui,pdfium executable build/freeze; the parent owns desktop smoke. No UI was launched or operated during this follow-up.

### Transaction and ownership

- `slide_surface::paint` returns the real `Window::paint_image` result. A candidate is committed only after that call succeeds. `TextureCache<K>` owns one committed complete image/snapshot per active surface. A failed candidate is explicitly `drop_image`d and the old image is painted in the same callback. Its old page/generation/crop/visual travel together, with stale strokes/laser cleared. Presenter input bounds are disabled while fallback pixels are shown.
- `Workbench::requested_surfaces` keeps cache ownership alive while an active surface awaits its replacement CPU raster. Closed/evicted surfaces and released windows explicitly retire owned images. The requested-set change therefore cannot destroy the sole rollback image before upload starts.
- Audience still receives only `AudienceProjectionSnapshot`, with the parent-approved pure `AudienceVisual.texture_budget_bytes` scalar. `paint_projection` checks the latest Curtain/blank state before either candidate or fallback drawing; covers are immediate. Curtain without visual does not request a zero budget or retire its retained fallback.
- Audience emits a narrow `RenderStatus(Option<String>)` event. Shell subscribes using production `AnyWindowHandle::read` and accepts events only from its active/preparing Audience. A separate `graphics_error` preserves independent shell/document/save errors. Presenter renders localized `wb-upload-error` feedback for Audience or its own managed texture failure. Events are deduplicated; no frame-by-frame error-event loop.
- Failed pixel identity is suppressed until a new pixel allocation arrives or the effective budget changes. A new page/new raster/new document permits a fresh attempt; closing/recreating the Audience releases the cache. There is no automatic unbounded upload retry or device-loss reset mechanism.

### Exact budget semantics and09 seam

Existing settings parsing permits positive `u32` MiB, so the configured minimum remains **1 MiB** (no new minimum or schema change). The total L1 is split equally between Presenter and Audience. Within each window, at most half its budget is committed image capacity and the other half is reserved for a synchronous candidate. Thus Audience raster bounding target is capped at **total L1 /4** bytes. At the minimum setting, each window has512 KiB total and256 KiB settled capacity; overly large Presenter surfaces fail visibly instead of exceeding their allocation.

GPU accounting is `width × height ×4` for all committed images plus the current candidate. Every candidate's bytes enter the count before the upload callback. Lower-priority retained cache entries can be evicted to admit visible higher-priority content, but the replacing surface's rollback image is never evicted to admit its own candidate. Priority eviction is explicit retirement. A candidate exceeding settled capacity is refused, leaving the prior image and effective budget intact.

`TextureCache::accepts_budget(proposed_window_bytes)` is the read-only preflight for the current committed allocation; `effective_budget()` returns the still-effective window budget. `sync` rejects a shrink when retained committed bytes exceed half the proposal, leaves the old effective budget unchanged, suppresses new candidate allocation while rejected, and reports the rejection. Repeated rejected proposals do not retry uploads. Returning to the old budget clears the budget rejection immediately. This check protects existing allocations;09 must also validate any newly requested raster targets through their normal transaction before save.

For09, Shell must preflight **both windows before changing Settings**: use the existing handles' `downcast::<Presenter/Audience>()` and production `read`/`read_with`, add the small read-only view wrappers around their private `textures.borrow().accepts_budget(bytes)` and `effective_budget()`, then apply only when both accept. The cache API is live and used by08; unused view wrappers are deliberately not added ahead of their09 caller. Do not use `WindowHandle::root`: it is test-support-only. A partial two-window budget application must not be reported/saved as success; keep or restore the previously effective settings if runtime raster/upload application fails.

### Regression and check evidence

- `upload_failure_repaints_complete_identity_retires_candidate_and_safety_wins`: injects failure at the production cache's actual paint boundary; observes attempted new pixels followed by old pixels/old identity, verifies cleared overlays, retired candidate and retained old image via weak handles, suppresses same-identity retries, checks Curtain/black/white prevent all paint callbacks, succeeds with a new identity, retires replaced/closed images, and verifies a32-byte peak for old16+candidate16. Also paints the fallback through a CPU-raster gap.
- `budget_shrink_rejects_without_losing_fallback_and_priority_accounts_for_swap`: preflight rejects32 from effective64 while32 committed bytes remain; old pixels still paint; restoring64 converges; priority eviction admits a32-byte candidate with16-byte rollback (48 peak); oversize candidate is rejected without destroying fallback; final empty-set release reaches zero.
- `audience_budget_failure_is_localized_visible_and_recovers_without_losing_shell_errors`: actual GPUI TestPlatform Audience paint, budget rejection, event subscription, Presenter Alert layout, error recovery and preserved unrelated shell error; Curtain without visual produces no false budget rejection.
- Full GPUI headless suite: PASS94 (`upload-headless.log`). Existing checker wheel/stacked layout/timer/drag-release/pointer tests are preserved.
- Real PDFium UI suite, production gpui,pdfium feature combination, pinned local DLL: PASS29 (`upload-native-ui.log`). This is native library verification without desktop acceptance.
- Strict product/app all-target gpui,pdfium,test-support clippy/type-check `-D warnings`: PASS (`upload-lint.log`); existing dependency HarfRust warning remains, no waiver or vendor change.
- `cargo fmt --all` and `git diff --check`: PASS. Full `just ci`, all-feature deny and executable build/freeze are checker-owned next steps; no new receipts are fabricated here.

The first lint exposed an unused budget getter caused by missing Curtain wiring; the first production-feature test build exposed a test-only root API call. Both were repaired and the final checks above reran. All changes remain uncommitted/unarchived. Actual hardware upload failure, device-loss recovery, full native matrices/performance/VRAM/soak remain NOT_RUN/DEFERRED_BY_USER; injectable failure tests are not native GPU-failure observations.
