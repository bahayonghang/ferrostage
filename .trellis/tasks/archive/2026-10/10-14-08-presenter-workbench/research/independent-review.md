# Task08 independent review — 2026-10-15

Status: resumed independent review has verified the GPU upload transaction repair and repaired local recovery omissions. See the final follow-up below for current verification and executable identity; the initial review and handoff evidence is retained. Desktop acceptance remains parent-owned. All changes remain uncommitted and unarchived.

## Findings fixed

- `presenter/workbench_view.rs`: thumbnail rows had no wheel handler despite the implementation report claiming wheel paging. Wheel input now moves the bounded visible set by one configured column row, consumes the scroll event, and never commits the presentation page. Earlier/Later controls remain.
- `presenter/workbench_view.rs`: previews always used a horizontal split. Below800 logical pixels they now stack, with the existing inner presenter scroll container retaining access to notes. This is required responsive behavior, not completion of the deferred size/DPI matrix.
- `presenter/workbench_view.rs`, both locale catalogs: timer warnings only changed color and used the inherited sans font. The timer uses the existing MONO token and adds localized warning text.
- `presenter/workbench_view.rs`: leaving the drawing region ended the core stroke but retained the local drawing flag. Reentering with the button still down could append a new default pen stroke instead of the selected highlighter. Leaving now clears the local flag; a new press is required for another gesture.
- `presenter/workbench_view.rs`: split drag calculations used viewport coordinates and a hardcoded y origin, which do not describe scrolled or stacked panes. The calculation now uses measured pane bounds. Mouse release outside the split container also ends and saves the edit.
- `shell.rs`: added three actual GPUI input regressions, covering narrow stacking and grid wheel input, pointer leave/reentry, and split release outside with durable settings read-back and future-schema read-only rejection. Extracted the existing mapped-document fixture without weakening its assertions.

## Initial finding, subsequently repaired: GPU upload is not transactional

`slide_surface::paint` currently catches `Window::paint_image` failure, writes a tracing error and returns. `TextureCache::sync` has already retired the previous images. The old complete GPU image cannot be repainted, and the Presenter receives no visible failure. This violates the task design's upload-failure rollback and presenter error contract. No actual GPU upload failure was observed; this finding is established from the real error path, not a native failure receipt.

The parent explicitly requires this existing task08 contract to be completed and approved serial handoff back to the implementer. Do not declare the functional gate complete before reviewing the repair.

Verified maintained GPUI behavior: `vendor/gpui/src/window.rs:4953`, `paint_image`, calls `sprite_atlas.get_or_insert_with(...)?` before scene insertion. Success at that call is the upload acceptance boundary; creating an `Arc<RenderImage>` is not upload success. `Window::drop_image`/`App::drop_image` remain the explicit retirement APIs. Do not change the vendor.

Suggested implementation boundary:

1. Return the real image paint result from `slide_surface::paint`.
2. Stage a candidate while retaining the previous committed complete image/snapshot. Commit and retire the old allocation only after successful paint; retire a failed candidate and repaint the old complete image.
3. Preserve immediate Curtain/black/white precedence and clear mismatched page overlays when holding the old frame. Audience still receives only its controlled snapshot; a narrow emitted error event to Shell may report failure without introducing document/settings/controller input.
4. Show a Presenter-visible error. Test actual production failure/commit paths with controlled upload outcomes, plus retry/replacement/close retirement.
5. Explicitly account for retained-plus-candidate GPU bytes. The current half-budget target allocation does not reserve transaction headroom. Do not silently double L1 accounting or retain arbitrary old images.

This requires a cache/paint/projection transaction decision rather than a mechanical local correction, so the reviewer did not rewrite that ownership boundary.

## Requirement audit

| Requirement | Source/evidence assessment |
| --- | --- |
| W-1 | Current/next/private notes, toolbar, split settings and shared Displays entry implemented; measured split geometry and narrow stacking repaired. |
| W-2 | Split notes and ordered independent note pages use private Presenter requests. Notes scale changes scrollable geometry and canvas render target. |
| W-3 | Timer derives from core elapsed, applies count direction/remaining thresholds, and ticks independently of raster responses. Warning/font presentation repaired. Native elapsed smoke remains parent-owned. |
| W-4/W-10 | Assembler checks generation/page/crop/target/priority, retains complete old raster projection, and applies safety covers immediately. Audience has no document, notes, path, renderer or full cache input. GPU upload rollback remains the finding above. |
| W-5/W-6 | Shared normalized letterbox transform, finite validation, separate page-local strokes, highlighter alpha, segment eraser and clear are implemented. Pointer exit regression repaired. |
| W-7 | Preview is private; numeric notes-page rejection cannot commit an older preview; Commit/Escape paths retain core semantics. Ten-preview projection regression is retained. |
| W-8 | Columns and bounded requested/rendered tile count are independent; P4 thumbnails and page controls implemented. Wheel row navigation repaired and exercised by real GPUI input. |
| W-9 | Explicit image drop on replacement/eviction/release and actual texture byte accounting implemented, with visible-frame priority. Transactional upload rollback remains incomplete. Debug labels distinguish raster and UI paint-call time from actual Present. |
| W-11 | Initial import performs structure/metadata/build/candidate raster/worker commit/mapped install. Failed build aborts candidate and old worker remains usable. Import serialization, token checks and shutdown continuation cancellation prevent superseded install. Same-generation options preserve lifecycle/timer. |
| W-12 | Existing tokens and focus primitives reused; required narrow layout and timer warning semantics repaired. Native screenshots and broad visual matrix are not established by these tests. |

No new product changes outside task08, backend/vendor patches, policy waivers, unsafe code, PDF writes or core native dependencies were found. `audience.rs` forbidden-type search (`SlideNotes|notes_crop|LoadedDocument`) has no matches. Task07 save/Retry/focus assertions remain; the adapted import fixture exercises mapped commit and retained native PendingLoad tests cover the real worker route.

## First-pass verification before upload repair

| Check | Result / receipt |
| --- | --- |
| Strict feature lint/type-check: app/ui gpui,pdfium,test-support, all-targets, locked/offline, `-D warnings` | PASS — `review-feature-lint.log` |
| Full GPUI headless suite | PASS92 — `review-headless.log` |
| Focused actual-input workbench tests | PASS5 — `review-focused.log` |
| Real PDFium UI suite with pinned local DLL | PASS29 — `review-native-ui.log` |
| `just ci` | PASS158 default tests, fmt/default strict lint/deny — `review-ci.log` |
| Diff whitespace | PASS — `git diff --check` |
| Retained native36 / all-feature deny | Implementer receipts remain source-applicable: no PDF/render/document/dependency changes in this review. Default deny reran through CI. |

The first sandboxed Cargo attempt could not write the existing `D:\mbx-cache` lock; the tool initially condensed this to a misleading “No issues found” with exit1. Raw bypass identified the filesystem error; authorized cache access reran the strict check successfully. This was an environment failure, not a product lint pass.

The default CI overwrites `target/debug/ferrostage.exe` with the non-GPUI executable. **Do not launch it for native acceptance.** No final native rebuild/hash was produced because the parent requested GPU transaction implementation before freeze. After that patch, rerun affected checks and build `ferrostage-app --features gpui,pdfium` after default CI, then record the new hash/size and perform parent-owned bounded smoke.

Extended physical, exhaustive size/DPI/theme, performance, VRAM and soak checks remain DEFERRED_BY_USER/NOT_RUN. None is represented as PASS; original combined acceptance boxes stay unchecked.

## Parent follow-up

After the upload repair, sync the frontend state-management/quality guidance to the actual transactional image-commit/error-feedback contract and the task08 managed texture owner. The parent owns shared specs and execution state. Continue with independent review and native PDF/notes smoke before progressing to09.


## Final follow-up — GPU transaction independent review

The initial GPU-upload finding is closed. `slide_surface::paint` returns the real `Window::paint_image` result. `TextureCache` stages and accounts for the candidate, commits only after successful drawing, explicitly retires failures, and repaints the retained complete old image/identity with mismatched overlays cleared. Audience checks the latest Curtain/blank state before either candidate or fallback painting. A rejected shrink retains the previous effective budget and cannot authorize fresh allocations or clear upload-failure suppression. Budget and capability changes match parent design §6 and task09's reviewed consumer contract.

Actual atlas upload failures are injected at the production cache's draw callback in tests. These prove the transaction's failure handling; they do not constitute an observed hardware upload failure or device-loss recovery result. No vendor, unsafe, license or advisory waiver was introduced. Audience still has only the snapshot input and emits a narrow RenderStatus event.

Additional local corrections made during this resumed check:

- A successful repaint of an already committed image now clears its previous transient paint error. A failed new candidate remains suppressed until a new pixel identity or effective budget change.
- Refusal for insufficient available capacity no longer permanently marks pixels as failed even though no upload was attempted. Capacity is rechecked without calling the upload callback, so another surface's retirement can restore the blocked surface. An individually oversized candidate no longer evicts usable lower-priority images that cannot make it fit.
- The notes panel enumerates requested as well as completed surfaces, retaining the old managed notes image across a CPU-raster gap. A known pending notes surface says Rendering rather than No notes.
- A confirmed Audience window replacement clears the retired window's graphics diagnostic. New-window events own subsequent errors; unrelated document/save diagnostics remain intact. The existing integration test now verifies this replacement commit.

The new cache regression checks transient committed repaint recovery, repeated capacity rejection without upload callbacks, recovery after another surface retires, and preservation of a usable lower-priority image when a candidate can never fit. Existing candidate failure/identity/cover/retirement, rejected-shrink suppression, visible-error/read-back and actual-input tests remain unchanged in strength.

No source-review blocker remains. Full native screenshot/functional smoke remains the parent's next gate; physical and extended matrices remain deferred. Task09 must honor the reviewed preflight/application boundary for both windows before saving a budget edit; this review does not claim that future settings UI is implemented.


### Final retained checks and native handoff

- Strict app/UI all-target gpui,pdfium,test-support lint/type-check: PASS (`review-final-feature-lint.log`). The pre-existing dependency HarfRust warning remains; no product waiver.
- Full headless GPUI suite: PASS95 (`review-final-headless.log`).
- Pinned real PDFium UI suite: PASS29 (`review-final-native-ui.log`).
- `just ci`: PASS158 default tests, fmt, strict default lint and default deny (`review-final-ci.log`).
- All-feature cargo deny: PASS advisories/bans/licenses/sources (`review-final-deny.log`).
- Final gpui,pdfium app build: PASS **after default CI** (`review-final-native-build.log`).
- `git diff --check`: PASS. No first-party/vendor/policy changes outside the authorized task scope were found.

Final executable: `target/debug/ferrostage.exe`, **41,183,232 bytes**, SHA256 **2E38E384DEC3065AD01E41640A8A399AE7112D2DDDC686F799B83D12DDE27B1F**. The machine-readable receipt is `review-final-build-identity.json`. This identity supersedes every earlier task08 executable hash. No source edits followed this build; only this review receipt was completed.

Source ownership is released for parent-owned smoke. Launch with the isolated parent-owned `research/native/workbench-smoke.toml`, `--pdfium-library cache/pdfium-7881-win-x64/bin/pdfium.dll`, and `fixtures/notes-functional/pympress-notes.pdf`. Confirm current/next/private notes and audience isolation; navigation, preview cancel/commit, blank, pointer/strokes, timer pause/reset and close. Existing stable control IDs remain; grid wheel paging, notes scroll and split release can be exercised without a full size/performance matrix. Do not rerun default CI before launching this frozen executable unless the feature binary is rebuilt and rehashed afterwards.
