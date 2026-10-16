# Task11 implementation handoff — 2026-10-15

Status: V-1–V-8/P2 product implementation complete, automatic checks below passed; serial independent review pending. No desktop launch, focus/input, real speaker output, commit, push or archive was performed. Original composite acceptance checkboxes remain unchecked where they contain deferred desktop, hardware or soak requirements.

## Implemented behavior and files

- `ferrostage-media/{protocol,animation,supervisor}.rs` and `supervisor/child.rs`: pure GUI-side identity/controller plus one background owner; bounded40-byte framed IPC validates type/geometry/body size before allocating; command queue4, latest single frame/status, stderr continuously drained with4096-byte retained diagnostic limit. Native child is stopped, given300ms to exit, then killed/reaped before another native owner starts. No UI join. Parent pipe EOF exits the helper even during startup; crashed/missing/stalled helpers produce typed errors, and another selection can retry.
- `ferrostage-media-helper`: only package linking exact GStreamer0.25.4/app0.25.2/video0.25.2, optional `native` feature and required-feature executable. Safe mapped BGRA/stride handling, encoded decoder caps checked before buffers, output≤1920×configured height, real clocked start/duration/pause/repeat and audio-only progress. A5s preroll timeout is explicit; controller initialization is bounded10s. Default workspace CI needs no native SDK. Final GUI PE imports exclude GStreamer/GLib.
- GIF/WebP use image0.25.10 and content sniffing, including unnamed embedded data. Decoding/scaling/crop runs off UI, preserves exact delays and intrinsic loop count unless the PDF overrides it. Source bytes≤256MiB, dimensions≤8192 per axis and≤64Mi pixels, cumulative animation/frame metadata/scratch≤256MiB and≤10,000 frames. Geometry preserves aspect via transparent annotation-aspect contain/letterbox before conservative public crop (ceil left/top, floor right/bottom); canvas bounded before allocation. Nearest scaling avoids interpolation of private pixels across the border.
- `document/media_slots.rs`, builder and tests: source_crop plus physical annotation aspect derived separately from public slot rectangle. Notes-only annotations cannot start audio. Original PDFs/media are read-only; embedded payload uses create-new own temporary directory removed after decoder retirement.
- `ui/media_playback.rs`, Shell/Presenter/Audience, shared slide surface and TextureCache: only Presenting enters autostart; first public autostart in document order wins. Selecting another slot retires old owner. Presenter rectangle click in Navigate tool and explicit localized play/pause/progress controls; drawing tools keep gestures. Jump preview neither exposes media controls nor changes live media. Missing/unsupported source shows Presenter placeholder/error, Audience keeps PDF.
- Page/stage/crop/published generation/blank/Close invalidate epoch and accepted frames synchronously; leaving blank does not resurrect playback. Same-generation Notes layout change is covered. Public slot_id is a checked process-wide monotonically increasing playback-instance ID, never PDF annotation index; controller recreation cannot reuse retired texture identity. Audience receives only snapshot slot/rect/public BgraFrame and both windows share that allocation. Each window uses existing accounted TextureCache/swap budget. Composition is PDF→media→ink/laser and refuses media against mismatching complete PDF fallback identity.
- `settings/{schema,parse,merge}`, `ui/settings_panel.rs`, locales: v1 media.max_decode_height default1080, range1..2160, next instance consumer; old v1 defaults, unknown preservation and future-read-only remain. Existing section deletion regression expanded to include media.
- `packaging/media/runtime.json`,15 full original notices, source recipes/mapping; stable scripts prepare/stage/native-build/import inspection; ADR-009, packaging docs, justfile and CI. Exact47-file/18-plugin closure, no tools/generators/whole-wheel copy, no new waiver. Download/staging verify original hashes; runtime permits compatible dynamically replaced LGPL libraries. Stable `fixtures/media` copies preserve original hashes; product tests/build scripts do not depend on an active task directory.

## Final command results

All listed final commands exited0. Existing external HarfRust dead-code and cargo-deny advisory-level source/duplicate warnings remain; no project lint allowance or dependency-policy change was added.

| Command | Result |
| --- | --- |
| `powershell -NoProfile -File scripts/prepare-media-runtime.ps1 -Archives cache/media-research -Offline` | selected47 originals verified/staged and7 safe MSVC import libraries generated; no installed SDK/global PATH edit |
| `powershell -NoProfile -File scripts/media-native.ps1 -Action build` | native helper built |
| `powershell -NoProfile -File scripts/media-native.ps1 -Action clippy` | native helper strict all-target lint |
| `cargo test -p ferrostage-media --features native-tests --locked` with helper/runtime env below |14 tests:5 unit+9 integration, all silent actual native process/animation fixtures |
| `cargo test -p ferrostage-ui --features test-support,pdfium,media --lib --locked` with PDFium env |119 parent tests plus1 isolated real-PDFium replay;120 executions, not120 distinct tests |
| `cargo test -p ferrostage-pdf -p ferrostage-render -p ferrostage-document --features pdfium --all-targets --locked` |40 passed |
| `cargo clippy -p ferrostage-app -p ferrostage-ui -p ferrostage-media -p ferrostage-pdf -p ferrostage-render -p ferrostage-document --features gpui,pdfium,media,native-tests --all-targets --locked -- -D warnings` | production feature and native-test lint passed |
| `just ci` | fmt, strict default lint, complete default test suite and cargo deny passed; no SDK env required |
| `cargo deny --all-features check` | advisories/bans/licenses/sources passed |
| `cargo build -p ferrostage-app --features gpui,pdfium,media --locked` | final complete GUI rebuilt **after** default CI |
| `powershell -NoProfile -File scripts/check-media-imports.ps1 -Binary target/debug/ferrostage.exe` | no native media DLL imports in GUI |
| manifest/runtime-files comparison + SHA/length verification of all staged files |47 pass; exact production availability list |
| `git diff --check` | passed; line-ending notices only |

`cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` also exited0 with the same seven native import-library environment entries. This verifies feature-unified helper and test-support code; it does not add native imports to the GUI package. All stable binary fixture hashes also rechecked successfully. The generated identity list initially included the old README hash; that editable provenance document was removed from the binary fixture list, without changing any media/PDF hashes. No repeated broad suite is required for report-only changes.

### Meaningful new regressions

Media integration names: `actual_animation_delays_bgra_limits_pause_and_epoch_cancellation`, `actual_missing_runtime_is_typed_and_does_not_poison_animation`, `unnamed_embedded_gif_webp_use_pure_decoder_and_output_bounds`, `contain_letterbox_then_crop_is_bounded_and_never_stretches_private_half`, `actual_helper_video_seek_repeat_pause_audio_and_reap`, `partial_runtime_real_helper_and_embedded_video_are_independent_failures`, `unresponsive_child_stderr_is_drained_then_killed_and_reaped_off_caller`, `helper_parent_pipe_eof_terminates_process_during_startup`, `original_beamer_movie_metadata_drives_real_helper_playback`.

UI: `real_public_crop_ready_blank_notes_page_generation_and_close_isolate_media`, `successful_same_generation_notes_reconfigure_invalidates_old_public_pixels`, `original_beamer_movie_uses_actual_generated_video_and_parameters`, TextureCache instance-isolation regression, `real_composite_paints_pdf_then_media_then_laser_and_blocks_wrong_page_fallback`. The last executes actual normal GPUI canvas paint and atlas insertion, verifies scene order, injects new-entry failure while retaining old atlas content, and proves wrong-page media is absent. Headless scene correctness is not real D3D or display acceptance.

Protocol tests reject hostile frame headers before payload allocation and preserve BGRA/sequence; embedded-source unit test proves owned create-new cleanup. Document real media PDF regression verifies cross-notes/public/notes-only geometry and embedded bytes; settings regression verifies default/range/unknown/future behavior.

### Intermediate corrections (not hidden PASS)

Initial UI run omitted the PDFium test environment and correctly failed native tests; provided the pinned DLL on final run. A geometry assertion incorrectly required exact f32 1.6 versus1.5999999 and now uses1e-6 tolerance. The new composition test initially used a subpixel laser culled by scene and an atlas mock that rejected retained entries as well as new insertions; both test setup issues corrected before final PASS. Actual media tests briefly launched several independent native owners concurrently and two hit initialization failure. Shared native initialization contention was suspected, not proven; native process cases now serialize, matching the product single-owner model, and the complete final native suite passed. Preroll timeout is explicitly typed. Default CI first exposed the old eight-section test's exhaustive branch missing new `media`; expanded that test with a nondefault media value and sibling-preservation assertions, then full CI passed.

## Runtime and final identities

Exact final binary identities are in `implementation-build-identity.json`. GUI: `target/debug/ferrostage.exe`,42,888,192 bytes, SHA256 `FEC7E2A1286C29C16E73230B1149FF0819AB0F219B759EB21279866833775A3A`. Helper: `target/debug/ferrostage-media-helper.exe`,799,232 bytes, SHA256 `48D3801C3DDC3A563B56F7E56FBAD3CE4EEB89CE983992C2273D7DCC5D0C1F70`.

For native tests set:

```powershell
$env:FERROSTAGE_TEST_PDFIUM = (Resolve-Path cache/pdfium-7881-win-x64/bin/pdfium.dll).Path
$env:FERROSTAGE_TEST_MEDIA_HELPER = (Resolve-Path target/debug/ferrostage-media-helper.exe).Path
$env:FERROSTAGE_TEST_MEDIA_RUNTIME = (Resolve-Path cache/media-runtime).Path
just test-media
```

`just test-media` builds the adversarial ipc_stall example first. Native helper build/lint scripts set seven SYSTEM_DEPS_*_NO_PKG_CONFIG/LIB/SEARCH_NATIVE entries against `cache/media-import-libs`; exact names are in `scripts/media-native.ps1`. Controller tests use Runtime.silent=true→fakesink; production runtime uses autoaudiosink.

For root's owned desktop bin copy both final EXEs; stage exact selected runtime beneath `media-runtime/` beside GUI using the stable staging script, with notices/recipes/manifest/source mapping. GUI resolves helper by executable sibling; child receives isolated runtime bin PATH and plugin/scanner paths. Complete runtime absence and a selected missing DLL return RuntimeUnavailable before native playback; partial/load/codec/crash failures remain media-only. Do not infer visible GUI startup success from this background evidence. Root should test media-enabled actual GUI complete/partial absence and normal PDF behavior after independent check.

## Remaining review/native scope

No known required product feature is intentionally omitted. Independent review must check the final actual consumers, runtime closure, framing limits and lifecycle. Root owns consolidated desktop controls, dual-window playback/ink/laser and missing-runtime GUI observations. Physical unplug, real single/zero display, complete native/size/DPI/theme matrices, actual audio-stop latency,1080p60/15min pacing and long-run VRAM/performance remain DEFERRED_BY_USER/NOT_RUN. No PoC4 performance PASS or distribution/publication approval is claimed.
