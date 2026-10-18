# Layout and navigation implementation, 2026-10-18

## Delivered behavior

The toolbar now says **布局与导航 / Layout & navigation** and opens a presenter-owned upward menu over the existing workbench. The former full-content `Modal::Document` branch and internal Debug/boolean dump are removed. Six exclusive notes-layout rows share a reserved check column; two independent check items control notes recognition and PDF physical-page navigation. Automatic keeps the requested `None` override checked and describes the current effective layout beneath it. Real document and structure warnings are localized into ordinary presenter feedback.

The menu uses existing semantic palette and configured UI typography, 320px preferred width, 12px radius/inset and an 8px gap above its measured toolbar trigger. It right-aligns to that trigger, then clamps inside the window. Internal menu scrolling handles insufficient height. Whole rows use the existing deferred pointer/keyboard/accessibility activation boundary. MenuItemRadio/MenuItemCheckBox roles expose selected/toggled state, and the trigger exposes expanded state. The reserved check is drawn once, without passing through the shared button's check-prefix behavior.

Successful selection applies immediately and closes. Identical complete overrides close before candidate construction, preserving core state, annotations, frames and projection. A real change builds the candidate, builds its descriptor, asks core to accept it, assigns the session document, invalidates frames, publishes the new snapshot and closes. Failure preserves old document/options/core/frame state, keeps the menu open and presents a localized menu-local error; technical failure detail goes to tracing. Each open has an incremented id plus captured document generation so deferred clicks from a retired menu cannot affect a replacement/current session. Busy selections are refused. Overrides never enter SettingsStore.

Opening either contextual surface replaces the other. Arrow Up/Down, Home/End and Tab/Shift-Tab move among the eight rows, scrolling focused rows into view. Left/Right/PageUp/PageDown are consumed. Enter/Space activate once, held repeats are refused, and configurable global navigation cannot intercept these menu keys. Escape closes without Stop. Outside dismissal consumes the slide/tool click, and the invoking toolbar control regains focus. Existing Stop/Home/Swap/import/replacement cleanup remains shared.

## Shared interfaces and supporting scope

- `ContextualSurface::LayoutNavigation { id, generation, error }` joins PageBrowser in existing Shell state. `Shell::layout_menu_open()` and the narrow `reconfigure_layout` handler own menu transitions.
- `Presenter::contextual_host` now takes a `gpui::Anchor`; PageBrowser retains BottomLeft and its measured progress bounds, while this menu uses BottomRight and independently measured `document-options` bounds. No second popup framework or browser geometry retuning was introduced.
- Presenter captures `LayoutMenuData` before drawing, adds eight stable focus handles/scroll anchors and a stable trigger focus handle, and includes the menu in existing contextual focus/return handling. First draw still does not read Shell.
- The coordinator approved `PresentationState::has_annotations()` as necessary read-only supporting scope. It is true for a live laser or any stroke with at least one point across all pages. Empty strokes and empty retained per-page vectors are false. It mutates nothing, adds no UI/IO dependency and changes no core clear-on-reconfiguration policy. This makes the notice truthful even when ink exists only on a different page.

## Files changed by this child

All paths below are relative to the repository root. Earlier page-browser product edits remain intact in the shared checkout.

- `crates/ferrostage-ui/src/presenter/layout_navigation.rs` (new): captured menu data, localized warning projection, menu composition and raw-key routing.
- `crates/ferrostage-ui/src/shell/layout_navigation.rs` (new): guarded no-op/transaction/error transitions.
- `crates/ferrostage-ui/src/page_browser.rs`, `shell/page_browser.rs`: contextual variant and exhaustive browser accessor.
- `crates/ferrostage-ui/src/presenter.rs`, `presenter/page_browser.rs`: shared anchoring, measured trigger, focus lifecycle and composition.
- `crates/ferrostage-ui/src/presenter/workbench_view.rs`: renamed trigger and removal of old document form/debug projection.
- `crates/ferrostage-ui/src/actions.rs`, `shell.rs`: guarded typed selection intents, input interception, lifecycle integration and real GPUI regressions.
- `crates/ferrostage-core/src/state.rs`, `state/annotation_tests.rs`: read-only annotation-presence query and off-page/empty/laser regression.
- `assets/locales/en-US/app.toml`, `assets/locales/zh-CN/app.toml`: localized menu/help/error/warning copy.

Root DESIGN/spec/task lifecycle changes are coordinator-owned and were not edited by this child. No dependency, manifest, lockfile, vendor, persistence-schema or core command-policy change was made.

## Verification

PASS:

- `cargo check -p ferrostage-ui --features gpui --locked`.
- `cargo fmt --all`, followed by final `cargo fmt --all -- --check`.
- `cargo clippy -p ferrostage-ui --features gpui,pdfium,test-support --all-targets --locked -- -D warnings`. Receipt: `implement-clippy.log`. Initial needless-range-loop findings were fixed with ordinary iteration; no lint waiver was added.
- `cargo test -p ferrostage-ui --features test-support --lib --locked`: **158 passed, 0 failed**, retaining the prior 153-test baseline. Receipt: `implement-gpui-tests.log`.
- `cargo test -p ferrostage-core --lib --locked annotation_presence -- --nocapture`: **1 passed**, checking off-page ink, empty stroke/vector, read-only query and laser presence/removal.
- `cargo build -p ferrostage-app --features gpui,pdfium --locked`. Receipt: `implement-build.log`.
- Final `git diff --check`: exit 0; existing Git LF-to-CRLF informational warnings are not whitespace failures.

The five new GPUI tests exercise real rendered controls and native input dispatch:

1. `layout_menu_applies_each_layout_and_retains_upward_workbench_geometry`: six pointer choices, one selected layout check after reopen, live crop/snapshot update, notes exclusion, 320px menu, all eight targets >=44px, unchanged workbench bounds on open, viewport-clamped right alignment and 8px upward gap (within one physical-pixel rounding).
2. `layout_menu_keyboard_toggles_and_dismissal_do_not_leak_navigation`: arrow/Home/End/Tab order, no underlying page movement, each independent toggle on/off with Enter/Space, effective checks on reopen, notes remain excluded while recognition is enabled, Escape keeps Presenting, outside click does not draw or navigate, trigger focus returns.
3. `layout_menu_noop_preserves_off_page_annotations_frames_and_failure_retains_checks`: off-page ink notice, identical Auto leaves exact state/frames untouched, induced metadata failure keeps original checks/frames/state and local error, reopen drops stale error, restored valid metadata permits a successful change that clears ink/frames under existing policy and publishes the new crop.
4. `layout_menu_and_browser_are_exclusive_and_lifecycle_actions_discard_menu`: browser/menu replacement, browser scheduling closed, correct initial focus, actual open menu discarded by Stop/Swap/Home.
5. `layout_menu_rejects_busy_and_retired_selection_without_persistence`: deferred stale open-id selection and busy selection cannot mutate state; successful session override leaves settings and dirty state unchanged.

Normal 1100x760 GPUI receipt: the toolbar trigger wraps near the left at `(164,676.5)`, size `101x52`; the menu is `(12,161)`, size `320x508`. Its preferred right alignment would extend beyond the left edge, so the host correctly clamps it to the 12px inset. The gap is 7.5px after raster rounding. This is a single normal-window behavior regression, not a size/DPI matrix.

## Frozen build and remaining acceptance

Product edits frozen for checker/native handoff. Executable:

- `D:/Documents/Code/Rust/Exp/ferrostage/target/debug/ferrostage.exe`
- 43,603,968 bytes; UTC modified `2026-10-18T14:16:14.2824180Z`
- SHA256 `82484AC4699D6A0C34B01F3570A492BB98F35C86358A6A553D00E0F3D909FD5C`

NOT_RUN by this implementer: full default workspace CI and final `just check-shell` (independent checker), native real-PDF menu/theme/locale acceptance (coordinator). No GUI process was launched. The existing unused atspi-connection patch warning remains unchanged Cargo metadata. The coordinator independently passed full-feature cargo-deny without graph edits; its receipt is in the parent research folder.

Native selectors: `document-options`, `layout-navigation-menu`, `layout-menu-body`, `layout-row-0` through `layout-row-7`, `layout-check-0` through `layout-check-7` (only checked rows), `layout-annotation-notice`, `layout-menu-error`, and existing `contextual-backdrop`. Layout row indices are Auto, Unsplit, Left, Right, Top, Bottom, Recognize independent notes, Physical navigation.

The maintained GPUI TestPlatform does not deliver platform accessibility callbacks. Accessibility roles/state and the shared deferred click handler are implemented and compile-checked, but actual screen-reader/AX activation is not claimed as tested. Native normal-window review should inspect real preview updates and single checks in zh-CN/en-US/light/dark as applicable. No placeholders are shipped: the workbench remains the existing real PDF/notes surface.
