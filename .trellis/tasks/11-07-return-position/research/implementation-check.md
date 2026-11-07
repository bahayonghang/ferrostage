# Return-position implementation receipt — 2026-11-07

## Implemented boundary

- Shell owns one optional session anchor containing the original committed document generation and exact physical content step. Only a successful direct jump that changes the selected page establishes it; subsequent navigation preserves the target. Number, outline and thumbnail navigation retain their common `Shell::apply` boundary.
- The footer exposes `return-position` and `clear-return-position` only while the anchor exists. The configurable settings leaf is `keybindings.return_position`, default `Alt+Left`; existing modal, menu, editor and deferred-focus ownership remains in force.
- Presenting completion reuses accepted current-window public playback receipts. Covered completion uses guarded exact public-frame readiness and keeps the cover. Ready completion requires a successful current presenter texture paint with the exact current render request, current window and anchor-local capability.
- The private capability is renewed on navigation and explicit retry. This rejects a delayed A paint after A→B→A even when cached pixels legitimately retain an earlier raster request epoch, without forcing a new raster or adding another public navigation counter.
- Explicit same-page Return retries only failed current-target Current/Audience raster requests and failed desired texture allocations. Rearming retains the diagnostic until a successful paint. Audience retry inherits an existing committed outgoing image only when Page lacks one, preserving desired-request/failure bookkeeping and valid fallback pixels through another failed attempt. Ordinary identical redraws remain latched.
- Stop/Resume and timer reset preserve the anchor. Home, successful new open/reload and an actual notes-mapping/crop change clear it; failed reload/crop preserve it. Return preserves drawing state, timer and explicit autoplay pause.
- Settings v1 parsing, known-leaf merge, configurable UI, conflict diagnostics and English/Chinese strings include the new action without a schema bump.

## Implementation verification

Commands used the real Cargo binary from `C:\Users\lyh\.cargo\bin` and the absolute `cache/perf-product-target` target directory.

| Check | Result | Evidence |
| --- | --- | --- |
| `cargo test -p ferrostage-settings --lib --locked` | PASS, 55 tests | `runs/return-settings.log` |
| `cargo test -p ferrostage-ui --features test-support --lib shell::tests::return_ --locked` | PASS, 9 tests | `runs/return-focused.log` |
| `cargo test -p ferrostage-ui --features test-support --lib --locked` | PASS, 268 tests | `runs/return-full-ui.log` |
| `cargo test -p ferrostage-ui --features test-support,pdfium --lib shell::reload:: --locked` | PASS, 8 parent tests, with 2 real-PDFium child scenarios | `runs/return-reload-native.log`; this run preceded the final bounded texture retry changes, so the independent checker is rerunning it |

The reload run set `FERROSTAGE_TEST_PDFIUM` to the verified absolute `cache/pdfium-7881-win-x64/bin/pdfium.dll` path. The existing successful commit and injected upload-failure rollback test now assert the anchor lifecycle at those transaction boundaries.

The return tests cover real GPUI keyboard/button dispatch, exact overlay target retention, menu/editor ownership, rebind/unbind, explicit clear, stop/resume/reset, successful/failed lifecycle changes, late same-page/retired-window callbacks, masked readiness, actual Ready paint, capacity recovery, one-shot raster retry and latched upload retry without navigating away. After consuming the focused Return button, real Right and Backspace input verifies clicker navigation still works without creating an anchor. Cache/playback ownership tests separately cover retained diagnostics and outgoing fallback survival through another failed retry.

## Handoff

Source and implementation tests handed to `return_check` after the 268-test full pass. The checker owns strict test-support Clippy, the final PDFium reload rerun, then `just ci` followed by `just check-shell`; this order leaves the debug executable as the actual GPUI/PDFium app. The parent owns native real-Beamer acceptance, final spec/product documentation, commits and task archival. This receipt does not claim native acceptance or final gate completion.

## Native follow-up: stopped footer overflow

Native Chinese 16px acceptance found that the Ready navigation group containing the resumable Start split plus Return/Clear extended beyond the normal 1100×760 viewport. The outer control strip already wrapped, but its navigation child was an unbounded single row. The surgical production fix adds only internal wrapping and a full-width maximum to `session-navigation`.

A real GPUI bounds regression uses the same stopped session, A step 2 anchor/current C, 1100×760 viewport and a supported 24px UI size. The larger size compensates for the test platform's narrower synthetic glyph metrics; the parent retests native 16px. Before the fix, the regression failed because Return occupied x=1024..1293 (`runs/return-footer-before.log`). After the fix, every Resume/Restart/mode/previous/next/progress/Return/Clear control is inside the client viewport, the Current pane remains larger than 200×200, and a real Clear click consumes the anchor.

Updated implementation checks: focused return **10 passed**, full UI **269 passed** (`runs/return-focused.log`, `runs/return-full-ui.log`). Source/Cargo returned to the independent checker for final gates/build after this layout-only change. No navigation, receipt, paint, retry or cache behavior changed in this follow-up.
