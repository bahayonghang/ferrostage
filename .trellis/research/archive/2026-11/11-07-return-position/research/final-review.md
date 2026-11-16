# Return-position final review

## Scope and result

Reviewed the full return-position change set across `ferrostage-settings` and `ferrostage-ui`, its PRD/design/implementation plan and current frontend/backend contracts. Parent-owned product documentation, task-tree bookkeeping and archived empty-artifact cleanup are outside this source review. No outstanding source finding remains.

The anchor is exact generation/PhysicalPage session state, captured only after a changed-page direct JumpTo succeeds. Follow-up jumps and sequential/authored navigation retain the first origin. Current presenter WindowId, complete current request, current expected raster keys, successful desired texture/composite paint and a private anchor-local allocation identity gate Ready consumption. Presenting uses current accepted public settlement or the separate guarded covered-readiness fact. No notes or private navigation control were added to AudienceProjectionSnapshot.

## Findings fixed through coordinated implementation

1. **Explicit same-page Return did not rearm a failed attempt.** A same-page JumpTo refreshed public navigation but skipped `update_render`; failed Workbench requests and failed identical desired GPU images remained latched. The implementer added bounded explicit retry through the existing owners: failed requested Current/Audience keys are rearmed, the private attempt capability renews, and a fresh existing public playback identity authorizes one Audience retry. Ordinary redraws retain their prior failure latches.
2. **Fresh identity could discard the only valid outgoing fallback after failed transition upload.** The cache now transfers only the committed outgoing image into an empty Page fallback before cleanup, retaining the destination's desired pixels, failed bookkeeping and diagnostic. No extra allocation or upload is needed for that transfer.
3. **Rearm must not claim recovery.** Retry clears the attempt latch while retaining the diagnostic until an actual successful paint. This avoids a premature RenderStatus(None) authorizing covered readiness before a failing reattempt.

Main approved the bounded owner/interface changes before implementation. Source edits were owned by the implementer; the reviewer verified them and the regressions. The requested behavior does not add a history stack, new public navigation serial, automatic retry loop or private Audience command path.

## Regression evidence

- Real keyboard/menu/editor input, default and rebound shortcuts, explicit footer clear and exact overlay origin.
- Ready actual successful Current paint, old presenter/old same-page capability rejection, retained atlas image after failure and retry.
- Real same-page Return button retries one-shot raster and latched upload failures without navigating away; Right/Backspace navigation still works after the focused Return control is consumed.
- Actual cache ownership tests retain the outgoing image and diagnostic through a failed retry; repeated identical-identity redraw does not reattempt; a fresh attempt can recover.
- Public pending/settled and masked exact readiness, timer/tool/autoplay-pause preservation.
- Stop/Resume/reset, successful new-open/Home/crop reset, failed crop/reload preservation, and successful native-PDFium reload commit clearing.
- Optional v1 keybinding loading, empty/invalid values, conflict diagnostics, future read-only policy and unknown-key save/reload retention. Settings editor wiring was reviewed; main's native acceptance will exercise editing the new row.

## Final checks

- `cargo clippy -p ferrostage-ui --features test-support --all-targets --locked -- -D warnings`: PASS (`runs/return-review/clippy-test-support.log`).
- `cargo test -p ferrostage-ui --features test-support,pdfium --lib reload --locked`: PASS, **9 top-level tests**, including successful isolated subprocess cases (`runs/return-review/reload-pdfium.log`). The verified PDFium path was supplied explicitly.
- `just ci`: PASS, format, strict workspace Clippy, **251 workspace tests** and cargo deny (`runs/return-review/ci-final.log`).
- `just check-shell`: PASS, gpui/pdfium native product build, strict product Clippy and **268 GPUI tests** (`runs/return-review/check-shell-final.log`). This ran after CI so the final debug executable is the GPUI product.
- `cargo deny --all-features --locked --offline check`: PASS, advisories/bans/licenses/sources (`runs/return-review/deny-product-final.log`). It only checked the cached graph after the product build and did not relink the executable.
- Whitespace check: PASS.

These initial gates were followed by the native footer finding and final rerun below. Native window/compiled-Beamer/Settings-row acceptance remains owned by the main session; these automatic checks do not claim it or physical projector delivery.

## Native footer follow-up and final gates

Main's native acceptance found Return/Clear clipped in stopped, resumable Ready at 1100×760. The outer footer already wrapped, but its navigation child was an unconstrained single row. The surgical production fix adds `flex_wrap()` and `max_w_full()` to `session-navigation`; it changes no navigation, readiness or retry logic. The component guideline now records the bounded internally wrapping group.

The real GPUI regression uses Chinese and the supported 24px interface font (test-platform glyphs are narrower than native fonts), creates a resumable stopped session and anchor, checks all eight navigation controls inside the client bounds with accessible height, retains a Current pane larger than 200px on each axis, and clears through a real button click. Before the fix, Return occupied x=1024..1293 in the 1100px window (`runs/return-footer-before.log`); the fixed focused Return suite passed 10 tests and full UI suite passed 269 (`runs/return-full-ui.log`). Independent review found no further issue.

Final gates after the footer fix:

- Strict test-support Clippy: PASS (`runs/return-review/clippy-test-support-footer.log`).
- `just ci`: PASS, format, strict workspace Clippy, **251 workspace tests**, cargo deny (`runs/return-review/ci-footer-final.log`).
- `just check-shell`, after CI: PASS, GPUI/PDFium product build, strict product Clippy and **269 UI tests** (`runs/return-review/check-shell-footer-final.log`).
- Whitespace: PASS. The unchanged PDFium reload **9 top-level tests** and all-features deny evidence above were reused; the follow-up touches only navigation layout and its regression.

Final executable: `cache/perf-product-target/debug/ferrostage.exe`, 44,972,544 bytes, last write 2026-11-07 06:11:41 local. No source changed after these gates. Main was notified the executable is ready before native retesting; no Cargo build will run during that acceptance.

## Main native completion

Final native/editor/reload acceptance and unresolved Alt-letter observation are now recorded in [verification.md](verification.md); independent final visual review is [ship](visual-review.md). Gate log copies are retained byte-for-byte in `checks/gate-logs.zip`, with member hashes in `checks/manifest.json`. Native raw traces are in `native/traces.zip`.
