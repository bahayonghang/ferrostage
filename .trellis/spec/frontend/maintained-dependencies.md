# Maintained native dependencies

## 1. Scope / Trigger

Task07 adopts a reviewed local maintenance set because the two original GPUI routes fail the full dependency policy. This specification applies when changing vendor source, native feature wiring or its locked dependency graph. The concrete approval is ADR-008; license scope is ADR-005. A source or manifest change needs a new delta and relevant verification, even if package versions are unchanged.

## 2. Signatures

The workspace selects `gpui = { path = "vendor/gpui", default-features = false }` and `gpui_platform` from Zed tag `v1.23.3`, resolved to `96a1b1b5389efff473fc2306f7f8c690f2e1a417`. The Git patch replaces its GPUI package with the same local source. Crates.io patches select local pulp0.22.3, metal0.33.0, cosmic-text0.19.0 and harfrust0.12.0. The separately approved F14 adoption adds accesskit_windows0.34.0 as the sixth maintained package. Do not introduce a second GPUI family.

Product gates:

```text
cargo build -p ferrostage-app --features gpui,pdfium --locked --offline
cargo clippy -p ferrostage-app -p ferrostage-ui --all-targets --features gpui,pdfium --locked --offline -- -D warnings
cargo deny --all-features --locked --offline check
just ci
```

Core remains free of GPUI, native/file IO and parser dependencies. App forwards optional features; all GPUI calls belong in UI.

## 3. Contracts

- Approved runtime changes are the GPUI resvg/usvg0.48.1 + skrifa0.44.0 migration and Unicode coverage repair, bit-identical geometry constant, pulp/metal pastey0.2.3 manifest aliases, cosmic-text fontdb0.24.0 manifest, and HarfRust checked format2 selection/mapping in two files.
- Task `10-15-app-icon-harfrust-warning` (2026-10-15) cfg-gates the free function `vendor/harfrust/src/hb/face.rs` `shape` with `experimental_font_api`, the same condition as its `lib.rs` re-export. `Shaper::shape` and the two format2 files stay unchanged. The product graph does not enable the feature. No new lint allow.
- The explicitly authorized V8 retirement repair adds a bounded owning-Window/weak-callback lifetime guard to the existing GPUI active-status callback. Cancel only after the owning window is dropped; preserve error reporting for live callbacks and drop the lifetime token before platform-window teardown. See ADR-008's V8 boundary. This does not authorize modifying another platform package or suppressing arbitrary handle errors.
- V13 native F12 demonstrates the same boundary on a late hover callback. Treat the owning-window platform callback registration group consistently: callbacks that access the handle must consult the same owner lifetime, including deferred execution; do not protect only the callback named in the latest log. Retired callbacks return their existing neutral/default result. Live callbacks retain error reporting and normal behavior. Retained platform handles and callbacks can outlive the GPUI Window, so their own existence is not proof the Window remains registered. Changes remain inside the already maintained GPUI package; separately scheduled accessibility receivers require their own reachability and cancellation evidence before expanding the repair.
- V14 F14 reaches AccessKit COM providers directly, outside GPUI's Window-event and action-receiver guards. A provider can hold an upgraded context during native HWND teardown. On 2026-10-15 the user separately approved the reviewed four-file accesskit_windows0.34.0 repair in ADR-008. `util::client_top_left(WindowHandle) -> windows::core::Result<Point>` checks the actual `ClientToScreen(...).ok()` result; only `ERROR_INVALID_WINDOW_HANDLE` maps to `UIA_E_ELEMENTNOTAVAILABLE`. Context and the four COM coordinate callers propagate this Result. Preserve other errors, valid coordinates, missing-box zero rectangles and empty-range null results. Do not replace this with an earlier `IsWindow`, broad error suppression, or disabled accessibility. This is not a complete retirement/drain or HWND-reuse protocol.
- Preserve SVGZ and codec features, Unicode record/filter semantics, encoding priority and malformed-input rejection. A package node disappearing from the graph does not remove all parser ancestry: fontdb0.24 retains an upstream-maintained metadata subset derived from ttf-parser.
- Record upstream identity, complete approved source hashes, product-only manifest/path/fixture adaptation and original notices. Keep original font licenses/RFNs; construct modified collection fixtures in test memory, not as redistributed font files. Do not copy evaluation executables into product sources.
- `deny.toml` has only these extra exact exceptions: arrayref@0.3.9/BSD-2-Clause, libbz2-rs-sys@0.2.5/bzip2-1.0.6, cbindgen@0.28.0/MPL-2.0. MPL remains file-level copyleft; Apple build-only use is not a policy exemption. Preserve the global allowlist, full target/build graph, private checks and GPL/AGPL exclusions. No advisory or lint waiver was approved.

## 4. Validation & Error Matrix

| Condition | Required behavior |
| --- | --- |
| Missing/different approved input hash | Stop adoption and resolve source identity; do not label the new bytes as the old tested set |
| Original maintenance-preparation script without independent repairs | Reject as historical defective input; use final reconstruction collection |
| Unsupported/malformed cmap, invalid offsets or truncation | Checked failure/old compatible result; no unsafe parser shortcut or panic |
| Another rejected package/version/license | Fail policy and investigate; existing exceptions do not extend to it |
| Direct HarfRust strict lint has the six reproduced original diagnostics | Record FAIL with baseline comparison; consumer/project lint remains mandatory and no new allow is permitted |
| Native build passes but no window was observed | Compile PASS only; no focus/placement/GPU/theme acceptance |
| Queued activation runs after its Window is dropped | Cancel by owner lifetime; prove with actual GPUI queued-callback regression |
| Live activation callback fails | Preserve existing error reporting; do not classify every failed handle update as retirement |
| Hover/visibility/input/frame/geometry or tab callback arrives after owner retirement | Use the shared owning-window boundary; exercise retained callbacks after actual GPUI Window removal, including deferred work |
| Live callback accesses a temporarily leased Window | Keep the diagnostic visible; an owner token must not become a blanket error filter |
| Actual AccessKit coordinate conversion reports invalid HWND | Return UIA_E_ELEMENTNOTAVAILABLE through the existing COM Result boundary; never unwind across COM |
| Actual conversion reports another native error | Propagate the original error without relabeling or hiding it |
| Valid HWND, missing bounding box, or empty text range | Preserve exact coordinate values and existing zero/null semantics |

## 5. Good / Base / Bad Cases

Good: compare the final repaired source to the approved snapshot, record product fixture/path adaptations, rerun affected regressions, then audit the actual product graph. Base: no vendor/runtime-input change means existing source-bound regression evidence can remain applicable. Bad: importing only the first patch, suppressing full-graph Apple dependencies, or interpreting nonempty SVG output as equivalent rendering.

## 6. Tests Required

GPUI glyph-presence tests preserve formats0/2/4/6/10/12/13, multiple Unicode record behavior, symbol exclusion, collection indices, zero/delta behavior and malformed offsets. HarfRust tests exercise both cached and alternate constructors with the same format2 inputs. Strict SVG regressions compare entire RGBA buffers and fallback counts with original outlines; a nonzero painted-pixel count alone once passed while the wrong glyph was rendered. Preserve EXR channel/compression/tail checks and required original font fixtures when adapting test paths. Compile/typecheck/lint the actual app/UI and check core dependency purity. Default tests do not replace native PDFium or GPUI checks.

AccessKit regressions must invoke actual COM BoundingRectangle, ElementProviderFromPoint, RangeFromPoint and GetBoundingRectangles with retained Context, covering invalid HWND and live coordinates/text/SAFEARRAY values plus zero/empty semantics. Root alone executes owned-window destruction and native app verification. Deterministic queries after DestroyWindow do not prove simultaneous conversion/destruction, handle reuse, or full product shutdown. Native close acceptance requires full logs/backtrace, completed owned processes and fresh window inventory; exit0 alone is insufficient. Maintain exact upstream provenance, reconstruction diff and original manifest/licenses, and remove the patch only after equivalent upstream behavior and these regressions pass.

## 7. Wrong vs Correct

Wrong: `skrifa::Charmap::map(ch).is_some()` replaces the old multi-record Unicode presence query without comparison, or an assertion checks only `painted_pixels > 0`.

Correct: `vendor/gpui/src/svg_renderer.rs::font_has_char` preserves the audited record/filter and Option semantics through checked raw APIs; `vendor/harfrust/src/hb/{tables,charmap}.rs` preserves actual format2 consumption. Compare exact glyph/failure results and strict rendered output before accepting a future migration. Do not expand these private adapters into a general font parser.
