# Page browser implementation, 2026-10-18

## Scope delivered

Progress/Jump and the renamed Page browser action now open one bounded contextual panel above the progress trigger. The workbench remains rendered underneath. The existing Document modal/menu contents are deliberately left for the next child.

The visible PDF-page field has a real caret, selection, pointer placement/drag, Select All, replacement, Delete/Backspace, Home/End, Shift selection, copy/cut and plain-text clipboard paste. The field is ASCII numeric and bounded to ten digits. GPUI's maintained `EntityInputHandler`/`ElementInputHandler` protocol is installed during paint; OS text, selected-range, replacement, text-length and caret-bounds methods are implemented. No dependency or maintained-source patch was added.

API evidence: `vendor/gpui/src/input.rs` defines `EntityInputHandler`; `vendor/gpui/examples/input.rs` demonstrates shaped-line layout, range geometry, `Window::handle_input` and the adapter. The product code is an intentionally smaller numeric implementation, without the example's general Unicode editor/IME framework. AccessKit receives a TextInput role, localized label and current value. Rich AccessKit text-run/selection-action exposure is not implemented or claimed. Native UIA caret/value automation remains separate from ordinary keyboard and pointer editing.

Shell validates the one-based physical number and content-map membership. Independent notes pages get a specific local error. Valid edits reveal the candidate's group without changing core navigation/projection. Enter/Go or a whole-tile action jumps once and closes. Browser submit/edit intents carry an incrementing per-open id; Shell also checks captured document generation, current map membership and busy state. Stop, Home, Swap (including failure), import and accepted replacement/reconfiguration discard browser state.

Independent 1..4 dimension choices update the existing two settings leaves through one `apply_preferences` call. A 4-column/13-page config and a 1-column/16-page config remain custom on open. First explicit column choice derives `min(4, ceil(visible / old_columns))` rows; row choice keeps columns. Actual SettingsStore readback covers both leaves and preservation of an unknown presenter leaf. Failed saves retain effective dimensions and display local Unsaved/Retry; future-version settings block density edits.

Tiles have one shared native activation target encompassing image/caption, current badge, candidate boundary and an independent focus ring. Stable tile focus handles and scroll anchors expose focused rows in the image scroller. Existing raster scheduling and generation/crop freshness remain in Workbench. Closed browser rendering filters/retire UI thumbnail frames and textures; the existing scheduler advances epochs and cancels queued work. Group controls move one capacity and wheel moves one row, with a short distinct final group.

## Shared host for the next child

- `Shell::contextual: Option<page_browser::ContextualSurface>` is the shared open-surface state; this child introduces its PageBrowser variant only.
- `Shell::close_contextual()` clears it, closes thumbnail scheduling, removes retained thumbnail frames/targets. `abandon_modal()` delegates to it, preserving common Stop/Home/Swap/replacement cleanup.
- `Intent::CloseContextual` dismisses without invoking Stop. Escape consumes a contextual surface before the existing modal/Stop path.
- `Presenter::contextual_host(content, trigger_bounds)` hosts a child above an undimmed occluding backdrop, with an 8px gap and 12px viewport margin. It currently uses BottomLeft alignment; the menu child can extend that parameter for right alignment.
- Presenter captures `BrowserData` in `PresenterData::from_shell`; draw paths never lease Shell. Shared modal/contextual focus return restores a valid invoking control/fallback. The menu child should include its open state in the same focus lifecycle and restrict underlying Tab targets while open.
- `Presenter::contextual_anchor` records progress bounds through a measurement canvas on the activation target. The menu child should measure its own trigger instead of reusing progress bounds.
- Numeric editor raw events are intercepted before configurable shortcuts only while its own Shell-owned focus handle is focused. Tab is handled in capture: GPUI keystroke interception leaves propagation stopped, so waiting for root bubbling would strand focus in the field. A regression caught and fixed that issue.

## Product files

- New `src/page_browser.rs`: numeric edit model, browser state/id, legacy density conversion.
- New `src/shell/page_browser.rs`: guarded open/edit/submit/density/dismissal transitions.
- New `src/presenter/page_browser.rs`: captured browser projection, contextual host, density/real-thumbnail panel.
- New `src/presenter/number_input.rs`: native numeric input and caret/selection drawing.
- `src/presenter.rs`: captured state, focus and contextual composition/anchor.
- `src/presenter/workbench_view.rs`: keep workbench visible, remove duplicate inline grid, filter active thumbnail frames, rename toolbar action through locale.
- `src/actions.rs`, `src/lib.rs`, `src/shell.rs`, `src/shell/reload.rs`: module/actions and lifecycle integration.
- `src/workbench/tests.rs`: 2x3 group requests and short-final-group uniqueness on the existing raster regression.
- `assets/locales/{en-US,zh-CN}/app.toml`: browser labels, context, errors and density copy.

All `src` paths above are relative to `crates/ferrostage-ui`. Root DESIGN/spec edits are coordinator-owned.

## Verification

PASS:

- `cargo check -p ferrostage-ui --features gpui --locked`.
- `cargo fmt --all` (first attempt hit a transient Windows mapped-file lock, retry passed).
- `cargo clippy -p ferrostage-ui --features gpui,pdfium,test-support --all-targets --locked -- -D warnings`.
- `cargo test -p ferrostage-ui --features test-support --lib --locked`: **150 passed, 0 failed** after final product changes.
- Focused numeric/jump, thumbnail pointer cancellation/whole-tile activation and wheel regressions also passed independently during implementation.
- `cargo build -p ferrostage-app --features gpui,pdfium --locked`.

New meaningful GPUI tests include actual caret/selection/paste + digit/Backspace/Space shortcut conflicts, outside dismissal without drawing/click-through, legacy density failure/Retry/readback/read-only, excluded notes/stale ids and tile Tab+Enter/Space activation. Existing first-render, focus, presenter notes, audience privacy and full settings tests remain passing.

The current build is `D:/Documents/Code/Rust/Exp/ferrostage/target/debug/ferrostage.exe`. The existing unused Linux atspi patch warning remains Cargo metadata, not a new lint waiver or dependency change.

NOT_RUN by implementer: default workspace CI, full PDFium runtime scheduling regression and live native smoke. The coordinator/checker owns these remaining gates and PDFium DLL staging. Do not infer native visual quality from headless bounds/input tests.

## Native selectors and next verification

`page-browser`, `page-number-input`, `page-target-context`, `page-browser-images`, `thumbnail-grid`, `thumbnail-{step}`, `thumbnail-image-{step}`, `thumbnail-caption-{step}`, `page-columns-{1..4}`, `page-rows-{1..4}`, `grid-prev`, `grid-next`, `close-browser`, `commit-jump`, `page-retry-save`, `contextual-backdrop`.

Normal-window smoke: open a real PDF, open Page browser/progress or G, replace the selected value with physical 69, inspect unchanged current projection + linked helper, submit once; select 2 columns and 3 rows, browse groups, activate image/caption/keyboard tile, cancel a press, Escape/outside dismiss. Check light/dark and localized copy. Raster fixture independent notes page should produce the local refusal. No size/DPI matrix was added.

Potential review focus: native panel geometry and image legibility at normal window size; rich AccessKit text-selection actions are an explicit gap. The layout/navigation menu is intentionally not implemented by this child.
