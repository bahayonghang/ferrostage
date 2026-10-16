# Task10 implementation handoff — 2026-10-15

All H-1–H-8/P2 product paths are implemented. Source is released for independent check; all new work remains uncommitted and unarchived. No desktop application was launched or controlled. Raw composite PRD acceptance checkboxes remain unchecked because visible/native/hardware/performance pieces are deferred by the user.

## Product changes and contracts

- `ferrostage-platform/src/reload.rs`: parent-directory `notify` watcher, precise target filtering (Windows ASCII filename case), delete/recreate and rename survival, configurable quiet period (default300ms), stat stability≥100ms, tail1024-byte EOF, read then restat, 30s debug-only timeout and owner-drop shutdown. It canonicalizes the parent, not a possibly deleted target. Production operations on the target are metadata/read only. Removed `SHADOW_TMP_SUFFIX` and its misleading write-PDF commentary/test.
- `ferrostage-document/src/reload.rs`: unique original logical label plus overlay ordinal; duplicate/missing labels use physical clamp then greatest valid content≤physical. Annotation mapping never clamps a deleted page onto another page; ambiguous target collisions are dropped. Current document overrides are reused on the structure/document worker.
- `PresentationState::commit_reload(descriptor,position,pages) -> Result<usize,CoreError>`: checked direct entrypoint, rejects Idle with NotReady and stale/malformed inputs without mutation. Applies to latest state at final publication, retains timer/stage/blank and mapped ink, ends gestures/laser, clears preview, reports dropped annotation-page count. Old CommitDocument compatibility remains unchanged.
- `RenderHandle::commit_reversible(CandidateReady) -> Result<Reply<RollbackToken>,RenderError>` and `rollback_committed(RollbackToken) -> Result<Reply<GenerationAck>,RenderError>`: token is opaque/non-clone, bound to issuing worker Arc identity, candidate generation, previous generation+epoch and nonce. Actual active must still match; retirement invalidates capability. Old native document remains until Retire. A restored worker can raster the old generation again; newest-issued generation never decreases.
- Pure `AudienceProjectionSnapshot.staging: Option<AudienceStagingFrame>` carries attempt/generation/content page/crop/public BGRA/budget scalar only. No notes, document, path, settings or controller enters Audience. Committed visual/identity remain unchanged during staging. Audience emits token+Result feedback only.
- Texture staging retains old and pending allocations within each window's existing half-committed/half-swap budget. Actual normal canvas paint uses nonempty image bounds under an empty ContentMask; atlas insertion runs, scene rejects the clipped primitive. On success the identical RenderImage Arcs survive until promote. Failure/cancel releases candidates; promote discards old-generation slots including thumbnails. Same failed attempt does not retry indefinitely. Safety blank/Curtain gate visible drawing.
- `ui/shell/reload.rs`: read/structure work, PDFium Prepare metadata, document build on a structure thread, actual-target P0/P1 requests (Audience/current/next/notes), two-window paint ack, reversible worker Commit ack, latest identity recheck, current-core/session/frames publication, Retire ack. Page/target/options/window changes rebuild retained stable bytes without needing another filesystem event; timer/ink/laser activity alone does not invalidate the plan. Commit-wait cancellation retains the reply and rolls actual active back before retirement. New import queues behind settlement. Whole-app close orders Shutdown after queued commands and joins the only native worker; no projection survives shutdown. Retirement errors remain visible and are retained for recovery on re-import; failed rollback blocks further mixed-generation scheduling and exposes recovery.
- Existing09 Advanced page now has real `hot_reload.enabled` / `hot_reload.debounce_ms` controls, bilingual labels/help, runtime watcher construction before setting application/save, disable drops the watcher. A failed replacement watcher creation keeps effective settings, old watcher and saved file unchanged. UI debounce is bounded1–10000ms; parser's existing positive setting domain is unchanged.
- `Workbench::requests` is shared by ordinary scheduling and candidate planning, avoiding a divergent crop/target/priority calculation. Successful publication changes core generation; task11 can consume that existing generation boundary to invalidate playback, without introducing media implementation in10.

## Actual verification

All final commands below exited0 unless explicitly marked diagnostic:

| Command | Result |
| --- | --- |
| `cargo check -p ferrostage-app --features gpui,pdfium` | PASS after first compile fixes |
| `cargo test -p ferrostage-platform -p ferrostage-document -p ferrostage-core -p ferrostage-render` |83 tests PASS at intermediate milestone |
| `cargo test -p ferrostage-ui --features gpui,pdfium,test-support --lib` |112 parent tests PASS plus the selected native integration replayed once in an isolated child; RTK aggregate113, not113 distinct tests |
| `cargo test -p ferrostage-pdf -p ferrostage-render -p ferrostage-document --features pdfium --all-targets --locked` |39 PASS, including real pinned PDFium |
| `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` |PASS; only existing external harfrust dead-code warning |
| `cargo deny --all-features check` |PASS advisories/bans/licenses/sources; existing advisory-free source/duplicate warnings; no policy/exception/vendor changes |
| `just ci` |PASS fmt, strict default clippy,167 tests,deny |
| `cargo build -p ferrostage-app --features gpui,pdfium --locked` |PASS after default CI |
| `git -c core.fsmonitor=false diff --check` |PASS; ordinary existing CRLF conversion warnings |
| `cargo test -p ferrostage-platform --lib`, `cargo fmt --all -- --check` |Final local check after adding Windows case-match assertion; production source/binary unchanged |

New meaningful regressions:

- `quiet_stable_timeout_and_exact_target`: injected time verifies300ms quiet,100ms stability,30s timeout, exact rename target and Windows filename-case matching.
- `real_parent_watch_handles_chunks_rename_delete_recreate_and_ignores_siblings`: actual OS notify against owned temp files,64KiB writes50ms apart, EOF repair, rename/delete/recreate, sibling exclusion and stale post-read stat rejection.
- `reload_maps_unique_label_ordinal_and_ambiguous_physical_content`: unique/duplicate/missing labels, notes gaps and short documents.
- `reload_applies_to_latest_timer_blank_and_ink_without_reset`: malformed/Idle atomicity, latest paused timer/blank/ink preservation.
- `reversible_commit_restores_active_raster_and_rejects_foreign_or_retired_capabilities`: exact worker recovery, Abort-is-not-rollback, retirement and cross-worker capability rejection.
- `staged_upload_keeps_old_pixels_counts_both_and_promotes_the_same_arc`: fallback pixels, explicit byte accounting, same Arc promotion, failure retirement/no spin and budget rejection before allocation.
- `real_paint_uploads_with_empty_mask_culls_scene_and_reports_atlas_failure`: normal GPUI paint calls real Window::paint_image; custom test PlatformAtlas injects insertion failure; actual scene has zero candidate polychrome sprites. This is scene/atlas-path verification, not hardware GPU-driver or displayed-frame certification.
- `real_pdfium_two_window_commit_obsolete_ack_rollback_and_latest_state`: pinned real PDFium and real Presenter/Audience view paint callbacks on TestPlatform; stale Commit ack→rollback→retire→rebuild; latest timer/blank/ink; page/target change rebuilding without new event; upload failure keeps complete projection; real notify→stable read→lopdf/PDFium→two-window paint via64KiB/50ms temp-file chunks and rename; one complete replacement publishes once; invalid PDF stays old; Stop/newImport waits for Commit settlement; original fixture bytes checked unchanged.
- `runtime_watch_toggle_debounce_and_creation_failure_are_transactional`: actual09 intents, effective watcher lifecycle, persistent settings, failed parent-directory creation rollback, source bytes unchanged.

Diagnostics retained honestly: first native UI suite exposed `RuntimeBusy` when two native tests ran together. A test-only mutex then exposed `LibraryUnavailable` on a second process-global pdfium-render initialization. Final test invokes only the new native integration in its own child process (`FERROSTAGE_RELOAD_TEST_CHILD=1`), preserving production global bindings/config and one worker. The first full coordinator test also exposed repeated Ready notifications causing a redraw loop; acknowledgements now notify only on the first transition, and the full regression passes. Initial lifetime/borrow compile errors and the test's private present call were corrected before final gates; no waivers were added.

## Final feature executable

- Path: `D:\Documents\Code\Rust\Exp\ferrostage\target\debug\ferrostage.exe`
- Features: `gpui,pdfium`; rebuilt after `just ci`.
- Bytes:42201088
- Modified UTC:`2026-10-15T15:00:46.6107341Z`
- SHA256:`47998000D61019BF604AC9275D6B78FD8907EF76135DD84727BAE8B760B3D7A5`
- Original notes fixture SHA256 remains `E939E35F931C184FD93B2E9B95E560DB73BA6A6F6DFCCBB0963262F0B359DFEB`.

## Boundaries and remaining review

Independent check is next. Full desktop visible zero-black-frame acceptance, actual content-changing projection smoke, physical unplug/latency, real single/zero-monitor, complete native/DPI/theme/size and long performance/VRAM matrix remain DEFERRED_BY_USER/NOT_RUN. No test screenshot, scene count, generation counter or headless result is called native visible PASS. No commits/push/archive, no new dependency/waiver/vendor edit in10, no user-PDF writes, no11 implementation.
