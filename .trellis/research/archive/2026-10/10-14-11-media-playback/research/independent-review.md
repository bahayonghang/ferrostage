# Task11 independent review — 2026-10-15

Decision: V-1–V-8/P2 product implementation and independent automatic checks PASS after the local fixes below. No required product feature is intentionally deferred. Root still owns consolidated foreground/native functional observations; this review does not claim displayed D3D frames, real speaker output, physical hardware, latency, matrix or soak acceptance. Latest parent acceptance-policy overrides historical implementation blockers. All changes remain uncommitted, unpushed and unarchived.

## Findings (fixed)

1. **`crates/ferrostage-media/src/supervisor/child.rs`: selected scanner path did not match the shipped closure.** Native configured `libexec/gstreamer-1.0/gst-plugin-scanner.exe`, while the reviewed manifest stages `libexec/gst-plugin-scanner.exe`. Existing registry/in-process fallback could hide this packaging error. Use the exact selected path, guard it against the production availability list, and run actual native fixtures with a unique initially absent registry. Packaging layout documentation now matches it.
2. **`crates/ferrostage-media/src/animation.rs`: cumulative preallocation omitted expanded contain canvases.** A narrow decoded frame can produce a much wider annotation canvas. Reuse the exact canvas geometry for the pre-decode peak check; account for retained frames, metadata, decoder/source/output scratch, full contain canvas and cropped result before decoding another frame. Public caps/settings remain unchanged. The real GIF regression first proves an individual canvas peak rejects, then decodes a two-frame GIF with a public half crop: the first 512-byte cropped frame fits, but its retained allocation plus the next canvas/crop peak rejects on the second iteration. Larger budget decodes both frames successfully.
3. **`crates/ferrostage-media/src/supervisor/child.rs`: child frames lacked monotonic sequence rejection.** The stdout decoder now rejects equal/regressing sequences as typed Protocol errors without replacing the retained valid frame. The regression feeds actual serialized IPC packets through the production reader for duplicate and regressing cases and checks retained pixels/sequence. The separate process-wide epoch boundary is preserved.
4. **`scripts/stage-media-runtime.py`: staging into a previously populated output could retain an unselected DLL/plugin.** Before writing any selected entries, reject existing DLL/EXE files outside the exact manifest. No files are deleted and no new runtime restrictions are applied to compatible dynamic replacements. The audit creates an owned temporary output with an extra DLL and proves failure occurs before writes, leaving the original intact. Manifest, 47-file closure and license policy are unchanged.

## Findings (not fixed)

No unresolved scoped code defect or outstanding design decision was found. Root's foreground functional smoke remains NOT_RUN at this handoff, including actual complete/partial/absent-runtime GUI behavior. Physical unplug, true single/zero screen, exhaustive native/size/DPI/theme matrices, sound-stop latency, 1080p60 pacing and long-run VRAM/performance remain DEFERRED_BY_USER/NOT_RUN; none is relabeled PASS.

## Requirement and boundary review

| Scope | Actual consumers and independent evidence |
| --- | --- |
| V-1 backend/packaging | ADR-009, helper native module and Cargo feature boundary reviewed. Only helper links GStreamer; final GUI PE imports audited. Exact 47 staged files/18 plugins match manifest and production availability list byte/hash; 15 full notices match original research notices and staged copies, with 15 recipes. No vendor edit, new unsafe block, license/advisory waiver or second backend introduced by the review. |
| V-2 GIF/WebP | Content sniffing selects pure image decoders, including extensionless embedded payload. Actual fixtures exercise delays, loops, BGRA, pause, bounded geometry and epoch cancellation. Revised peak test uses actual decoded GIF frames rather than duplicated arithmetic. |
| V-3 shared frames/privacy | Private document slots retain source/path/metadata. UI accepts generation/page/controller identity, then exposes only public opaque instance ID/rect/BgraFrame through the snapshot. Both windows clone the same immutable pixel allocation. Source is contained into transparent physical annotation geometry before conservative public cropping; notes-only slots cannot start audio. Public pixel/crop fixtures and original PDF metadata tests passed. |
| V-4 playback/control | One retained UI Controller owns one background decoder; another slot reaps the old child first. Ready produces no media; first public autostart wins in Presenting. Actual Beamer fixture drives start=2s/duration=8s and loop through the real helper. Presenter Navigate gestures and controls reach play/pause/progress; ink gestures remain ink. Jump preview hides media and ignores media control intent. |
| V-5 retirement | Global checked opaque playback IDs prevent controller recreation from reusing TextureCache fallback identity. Page/stage/blank/Close/published generation and same-generation crop changes invalidate acceptance. Unblank does not resurrect old playback. Real controller/GPUI regressions, stale cache isolation and composition ordering passed. Stdout/stderr drain off UI; bounded Stop/kill/reap and parent stdin EOF paths are tested with real child processes, including 8 MiB stderr and an unresponsive helper. No UI join or child HWND added. |
| V-6 settings | v1 media height default1080/range1..2160, width cap1920, next-instance consumer traced through settings panel → accepted settings → playback request → pure/native decode/canvas. Unknown-key and future-read-only behavior remains. Existing settings section-deletion coverage includes media with sibling preservation. |
| V-7 missing sources | Relative source resolves against PDF parent; external PDF/media is read-only. Embedded bytes use create-new owned temporary payload removed after decoder retirement. Missing/unsupported slot reports Presenter error/placeholder while Audience retains PDF. Failures and retry independence passed actual fixtures. |
| V-8 missing runtime | Complete production availability list is checked before child launch, including partial DLL absence, avoiding an OS loader attempt for those cases. Actual controller tests cover missing/partial closure and subsequent pure animation recovery. Final GUI has no Gst/GLib PE imports. These checks support isolation but do not replace root's pending actual GUI launch/control observations. |
| 08–10 preservation | 119 parent GPUI tests retain closed-modal Split/apply/rollback serialization, keymap replacement, save retry/future-read-only, upload fallback/error recovery, atomic hot reload and real worker rollback/retire ownership. The real PDFium replay is a child re-execution of one parent test, not an extra independent case. Two old07 focus/save tests retain their existing assertions; no assertion removal or coverage relaxation was introduced by this review. |

## Verification and raw receipts

All following commands exited **0**. Full command output is preserved in the linked files; [command receipts](review-command-receipts.json) record their exit codes and log hashes. Existing vendor HarfRust dead-code and cargo-deny advisory/source/duplicate warnings remain; no project warning waiver was added. The initially sandbox-denied temporary audit file creation was an environment diagnostic; the same owned-workspace audit completed successfully with normal build/test permissions.

| Command | Result / raw receipt |
| --- | --- |
| `cargo test -p ferrostage-media --lib --locked --offline` | 8 unit PASS, including strengthened retained-frame peak regression; [log](review-media-unit.log) |
| `just test-media` with helper/runtime and initially absent isolated registry | 8 unit +9 actual silent native/pure integration PASS; [log](review-native-media.log) |
| `cargo test -p ferrostage-media --features native-tests --locked --offline` against the final rebuilt helper, second fresh registry | 8 unit +9 integration PASS; [final log](review-final-native-media.log). Repeated to bind actual native tests to the exact final helper identity, not to increase the distinct count. |
| `cargo test -p ferrostage-ui --features test-support,pdfium,media --lib --locked --offline` | 119 parent tests +1 isolated real-PDFium replay PASS; [log](review-ui-tests.log) |
| `cargo test -p ferrostage-pdf -p ferrostage-render -p ferrostage-document --features pdfium --all-targets --locked --offline` | 40 PASS; [log](review-pdfium-tests.log) |
| `cargo clippy -p ferrostage-app -p ferrostage-ui -p ferrostage-media -p ferrostage-pdf -p ferrostage-render -p ferrostage-document --features gpui,pdfium,media,native-tests --all-targets --locked --offline -- -D warnings` | Strict production configuration without test-support PASS; [log](review-production-clippy.log) |
| `cargo clippy --workspace --all-targets --all-features --locked --offline -- -D warnings` | PASS with reviewed local native import libraries; [log](review-all-feature-clippy.log) |
| `cargo check --workspace --all-targets --all-features --locked --offline` | TypeCheck PASS; [log](review-all-feature-check.log) |
| `powershell -NoProfile -File scripts/media-native.ps1 -Action clippy` | Helper strict production lint PASS; [log](review-helper-clippy.log) |
| `powershell -NoProfile -File scripts/media-native.ps1 -Action test` | Native targets compile/load PASS; 0 helper-local tests, not an extra functional test count; [log](review-helper-tests.log) |
| `cargo deny --all-features --locked --offline check` | advisories/bans/licenses/sources PASS; [log](review-all-feature-deny.log) |
| `just ci` | SDK-independent fmt/strict lint/180 default tests/deny PASS; [log](review-just-ci.log) |
| `cargo build -p ferrostage-app --features gpui,pdfium,media --locked --offline` | Final GUI built after default CI; [log](review-final-gui-build.log) |
| `powershell -NoProfile -File scripts/media-native.ps1 -Action build` | Final helper; [log](review-final-helper-build.log) |
| `powershell -NoProfile -File scripts/check-media-imports.ps1 -Binary target/debug/ferrostage.exe` | No Gst/GLib GUI imports; [log](review-final-imports.log) |
| Exact manifest/list/hash/notices/recipes and unselected-output rejection audit | PASS; [receipt](review-runtime-audit.json) |
| `git diff --check` | PASS; [log](review-diff-check.log) |

PDFium env uses `cache/pdfium-7881-win-x64/bin/pdfium.dll`; media env uses final `target/debug/ferrostage-media-helper.exe` and `cache/media-runtime`. Native Controller fixtures explicitly set silent fakesink. Production remains autoaudiosink. Helper/all-feature builds use the seven SYSTEM_DEPS_* entries from `scripts/media-native.ps1`, against `cache/media-import-libs`; no SDK install/global PATH change. Helper test target loading only adds this reviewed runtime bin to its process PATH. Both isolated registry paths and hashes are recorded in the runtime receipt; no previous user registry was required.

## Final source and binary handoff

[review-build-identity.json](review-build-identity.json) records exact final executables and Git HEAD; [review-source-identity.json](review-source-identity.json) binds every product crate, script, packaging and locale file plus Cargo/configuration to SHA256. Review documentation changes after those checks do not change that product source set.

- GUI `target/debug/ferrostage.exe`: **42,889,216 bytes**, SHA256 **4BEBEFA5810F93CCB2E9FDA3E8134AACB6EBC24B55E46D0A0D06308387D9CE55**.
- Helper `target/debug/ferrostage-media-helper.exe`: **799,232 bytes**, SHA256 **312752079A67C00C03F7FCF6C843D57D9E19996FC1445AD82026B56A51B96641**.

Use both current binaries for root's owned launcher and the already verified47-file runtime beneath `media-runtime/`. Root's complete/partial46/absent0 packages remain valid because the manifest is unchanged. Final root smoke should preserve its fresh-registry setup and test public crop/notes-only refusal, play/pause/navigation/blank/reload retirement, overlays and missing-runtime normal PDF functionality. Reviewer did not launch/focus the desktop or produce speaker audio. Source/build ownership is released to root; no commands remain running.
