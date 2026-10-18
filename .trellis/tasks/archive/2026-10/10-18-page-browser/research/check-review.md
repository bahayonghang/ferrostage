# Page-browser independent check, 2026-10-18

Scope: `.trellis/tasks/10-18-page-browser`, approved parent design, tracked diff and all four untracked product modules. This checker reviewed and repaired directly without spawning another agent. Root owns native UI automation, DESIGN/spec edits and child 2. No commits, pushes or task lifecycle changes were performed.

## Findings fixed

1. `presenter/workbench_view.rs`, `picture`: the shared 70px minimum raster height exceeded the browser's 38px thumbnail image slot at the normal 1100x760 window. A real-frame GPUI regression failed with a 70px raster in a 38px slot. The minimum now applies to non-thumbnail surfaces only. Image/caption containment passes after repair.
2. `presenter/page_browser.rs`, `workbench.rs`, locales: failed thumbnail work previously retained the ordinary `Rendering` placeholder indefinitely. Added a small captured list of failed current thumbnail surfaces and localized failure placeholders of the same geometry. The list derives only from current requests and exact generation/page/crop/target/priority pixel identity; unrelated or retired requests cannot inherit the error. Same-pixel failures remain reusable across epochs consistently with the existing scheduler. Tests cover crop/generation/group retirement and the rendered placeholder.
3. `presenter/page_browser.rs`: selected density buttons received two check prefixes because the shared button already adds one. Removed the redundant prefix.
4. Native-review layout follow-up: compacted title/current context into the header and capacity into the density row, as specified by the accepted parent composition. The 72% height is a preference: normal-window review proved it could not fit four meaningful image rows plus controls. With coordinator approval, height now expands only into available space above the invoking toolbar, reserving a 12px viewport inset and 8px trigger gap. Row/caption/focus overhead drives image sizing; images target 38px or larger and retain a 30px floor under the available-space cap. Legacy tall/custom arrangements can still scroll. Caption padding is 2px vertically, and tile widths include both panel borders.
5. The anchor measurement canvas reported the trigger's inner content box, leaving a 4px gap. Dilated its captured bounds by the existing shared focus border/padding so the contextual host uses the trigger's outer bounds. A normal-window regression now checks all 16 tiles, the image floor, horizontal containment, top inset and actual 8px toolbar separation.

Strict checks also caught two local follow-ups in the new regression/accessor: initialize Workbench fields inside its default initializer; compile the failure query only for GPUI or tests. Both were corrected without lint waivers.

## Source audit

- Editor uses maintained GPUI EntityInputHandler/ElementInputHandler, ASCII selection/caret offsets, pointer placement/drag, replacement, deletion, Home/End, clipboard snapshot at input and bounded text. Capture/interception precedes configurable global actions; Tab has its explicit focused-field route.
- Shell owns the edit target separately from committed core state. Submission checks per-open id, captured generation, busy state, range and current content-map membership. Private notes pages are rejected locally. Whole-tile image/caption shares one deferred activation path.
- Contextual drawing consumes captured data, retaining synchronous first-render borrow safety. Outside left press is occluded/consumed. Escape dismisses before Stop. Stop/Home/Swap/import/replacement and successful reconfiguration discard browser state. Shared valid-focus restoration remains in Presenter.
- Density changes edit both existing schema-v1 leaves in one preferences transaction/save. Legacy 4/13 and 1/16 remain custom until explicit edit. Read-only state guards mutation; failed saves keep effective values and Retry. Unknown TOML leaves survive actual readback.
- Workbench retains the single renderer, request epochs and pixel identity. Presenter excludes closed/off-group thumbnails before texture synchronization. No schema, dependency, native-source patch, extra renderer or budget increase was introduced.
- Coordinator-owned root DESIGN and frontend component contract match these flows; no concrete spec drift requiring a checker edit was identified.

## Verification receipts

- Initial default `just ci`: exit 0, 15.30s. Log: `check-ci.log`.
- Initial verified-DLL native regression: exit 0, 10.44s. Log: `check-pdfium.log`.
- `just check-shell` after first repairs: exit 0, 16.69s, 152 GPUI tests. Initial receipt: `check-shell.log`; superseded by final results below.
- Post-feature-gate default fmt/strict Clippy passed. Default workspace test invocation hit Windows OS error 5 while the coordinator's native exe was running. This was an executable file lock before test execution, not a failed assertion. Receipt preserved as `check-ci-native-lock.log`; rerun after native close passed as recorded below.
- PDFium-only regression after first repairs: exit 0, 4.10s; one test passed. The final refreshed receipt is recorded below.
- DLL `target/debug/pdfium.dll` matches `docs/pdfium-runtime.json`: SHA256 `79D4676B656CFB1ABCEA88F9ADE3B4B0826C5200382DB5F4EC72A636C598C118`.
- Real PDFium ledger: old four-strip request model 4 thumbnails/17,510,400 retained bytes; closed 0/16,588,800; open16 16/20,275,200; closed again 0/16,588,800. These are retained-frame bytes, not CPU/FPS/RSS measurements.

## Remaining verification boundaries

Native smoke is coordinator-owned and must be bound to the final rebuilt exe after the row-fit fix. Earlier coordinator screenshots live in the parent task `research/native/`; they establish the original defects and actual editor/commit behavior, not final visual acceptance. No size/DPI matrix was added. Rich AccessKit text-run/caret/selection-action exposure remains the documented unsupported gap; ordinary field role/label/value and keyboard/mouse editing are implemented. This does not claim native assistive-technology acceptance.

## Normal-window geometry receipt

`page_browser_configured_rows_fit_normal_window`, 1100x760, 100 content pages: panel origin (208,12), size 880x586.5; image viewport 846x284.5; complete four-row grid 846x278; final image slot 195.5x30. All 16 tiles fit without scrolling, and the toolbar remains uncovered. This is one normal-window interaction regression, not a size matrix. Seven focused browser tests passed after the final anchor correction; log: `check-browser-final.log`.

## Final gates and source identity

- Lint/format/default TypeCheck: **PASS**, final `just ci` exit 0, 13.17s. `check-ci-final.log` includes workspace Clippy with `-D warnings`, **197 default tests passed** and successful cargo-deny checks.
- Native TypeCheck/build/strict Clippy/GPUI tests: **PASS**, final `just check-shell` exit 0, 12.86s; **153 passed, 0 failed**. Log: `check-shell-final.log`.
- Real PDFium retention: **PASS**, final `cargo test -p ferrostage-ui --features pdfium --lib --locked native_perf100_grid_retention -- --nocapture`, exit 0, 1.31s; one test passed, unchanged [4,0,16,0] thumbnail ledger. Log: `check-pdfium-final.log`.
- Final `git diff --check`: exit 0. Git's existing LF-to-CRLF informational warnings do not report whitespace errors.
- Final executable: `D:/Documents/Code/Rust/Exp/ferrostage/target/debug/ferrostage.exe`, 43,535,872 bytes, built UTC 2026-10-18T14:01:23.2886319Z, SHA256 `B7D001F82ABF24003D166A07BAADD7CD664F9B6D30A4214663FAF5A7CC4C408C`.
- Base HEAD: `5ec8437e4ff82c3cb7a915250b428d79bd148e00`, branch `dev`, uncommitted approved product changes. Hashes of every changed/new product source and locale at freeze are in `check-source-sha256.txt`.

No remaining page-browser code defect was found after these fixes and checks. The final coordinator native resmoke and documented rich AccessKit text-selection gap remain separate verification boundaries. Child 2 was not started by this checker.
