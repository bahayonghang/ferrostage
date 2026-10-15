# Maintained native dependencies

## 1. Scope / Trigger

Task07 adopts a reviewed local maintenance set because the two original GPUI routes fail the full dependency policy. This specification applies when changing vendor source, native feature wiring or its locked dependency graph. The concrete approval is ADR-008; license scope is ADR-005. A source or manifest change needs a new delta and relevant verification, even if package versions are unchanged.

## 2. Signatures

The workspace selects `gpui = { path = "vendor/gpui", default-features = false }` and `gpui_platform` from Zed tag `v1.23.3`, resolved to `96a1b1b5389efff473fc2306f7f8c690f2e1a417`. The Git patch replaces its GPUI package with the same local source. Crates.io patches select local pulp0.22.3, metal0.33.0, cosmic-text0.19.0 and harfrust0.12.0. Do not introduce a second GPUI family.

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

## 5. Good / Base / Bad Cases

Good: compare the final repaired source to the approved snapshot, record product fixture/path adaptations, rerun affected regressions, then audit the actual product graph. Base: no vendor/runtime-input change means existing source-bound regression evidence can remain applicable. Bad: importing only the first patch, suppressing full-graph Apple dependencies, or interpreting nonempty SVG output as equivalent rendering.

## 6. Tests Required

GPUI glyph-presence tests preserve formats0/2/4/6/10/12/13, multiple Unicode record behavior, symbol exclusion, collection indices, zero/delta behavior and malformed offsets. HarfRust tests exercise both cached and alternate constructors with the same format2 inputs. Strict SVG regressions compare entire RGBA buffers and fallback counts with original outlines; a nonzero painted-pixel count alone once passed while the wrong glyph was rendered. Preserve EXR channel/compression/tail checks and required original font fixtures when adapting test paths. Compile/typecheck/lint the actual app/UI and check core dependency purity. Default tests do not replace native PDFium or GPUI checks.

## 7. Wrong vs Correct

Wrong: `skrifa::Charmap::map(ch).is_some()` replaces the old multi-record Unicode presence query without comparison, or an assertion checks only `painted_pixels > 0`.

Correct: `vendor/gpui/src/svg_renderer.rs::font_has_char` preserves the audited record/filter and Option semantics through checked raw APIs; `vendor/harfrust/src/hb/{tables,charmap}.rs` preserves actual format2 consumption. Compare exact glyph/failure results and strict rendered output before accepting a future migration. Do not expand these private adapters into a general font parser.
