# Presenter fullscreen/layout implementation evidence

Implemented on 2026-10-19 after explicit complete-plan approval.

## Behavior

- Start captures the existing Presenter identity, MonitorId, display identity, bounds, fullscreen and maximized state. It requests fullscreen in place and the existing audience transaction cannot commit until both windows pass native placement readback. The view, current page and focus remain in place.
- Cancellation/failure/Stop retires audience projection and restores Presenter once. Restore participates in the busy guard. The Windows implementation queues fullscreen toggles, so restoration waits to observe entry before issuing exactly one exit; pre-entry windowed state cannot falsely complete restoration. Timeout reports the existing placement failure and allows safe windowed recovery.
- Confirmed display assignment still uses the existing windowed Presenter migration policy. Retiring the original Presenter discards its obsolete restore record, so old bounds are never applied to a new role/window. No display bindings or startup-fullscreen preference are rewritten by Start/Stop.
- Fresh right-column default is 0.40. Existing stored ratios remain authoritative. Next is upper-right and original Notes lower-right. The notes share remains 0.65, clamped to 0.55–0.8. Drag math accounts for the divider/gaps and maps downwards to less Notes height.
- Each original Notes raster uses the full inner width and its own aspect ratio. Multiple pages retain their natural heights inside one vertical scroll. No text extraction, reflow, font/mode selector or note-content mutation was introduced.
- Presenting has a compact status/header and session strip. More contains Home/Open/Settings; Drawing contains pen/highlighter/eraser/clear. Laser, page navigation, progress, blanking, grid and layout remain direct. Shared contextual dismissal, focus return, native press states and disabled guards are retained. Ready keeps the existing management toolbar.

## Checks

PASS: `cargo check -p ferrostage-ui --features test-support --tests --locked --offline`.

PASS: `cargo test -p ferrostage-ui --features test-support --lib --locked --offline`, 186 tests at the pre-integration snapshot. New tests cover flag-only Start refusal/timeout rollback, repeated/already-fullscreen restoration, actual divider input and settings readback, multiple original Notes raster geometry, and real menu keyboard/Escape/focus behavior.

PASS: `cargo test -p ferrostage-ui --features test-support --lib presenter_ --locked --offline`, 15 passed after asynchronous native-toggle handling. A separate delayed-entry regression is also recorded in `shell/presenter_layout_tests.rs`; its final result is in the implementation handoff.

PASS: `cargo test -p ferrostage-settings --locked --offline`, 48 tests. `git diff --check` passed at this checkpoint.

The scoped strict clippy run reached these changes without an owned-file diagnostic, but the shared working tree was still integrating transitions: unread PlaybackEvent fields in audience.rs, collapsible_if in texture_cache.rs, and chunks_exact_to_as_chunks in transitions.rs prevented a full PASS. Parent owns the final unified strict gate; no warnings were waived.

NOT_RUN here: actual dual-display fullscreen success, normal/maximized native restoration, monitor-loss recovery, rebuilt Notes visual readability and native screenshot comparison. Parent integration owns those checks. Headless GPUI readback/geometry does not establish native Windows placement or standing readability.

## Owned files

- `crates/ferrostage-ui/src/shell.rs` (narrow lifecycle/dispatch hooks, updated old UI tests, new test include)
- `crates/ferrostage-ui/src/shell/presenter_fullscreen.rs`
- `crates/ferrostage-ui/src/shell/presenter_layout_tests.rs`
- `crates/ferrostage-ui/src/presenter.rs`
- `crates/ferrostage-ui/src/presenter/workbench_view.rs`
- `crates/ferrostage-ui/src/presenter/utility_menu.rs`
- `crates/ferrostage-ui/src/presenter/page_browser.rs` (shared host supports downward header menus)
- `crates/ferrostage-ui/src/actions.rs`, `page_browser.rs` (menu intent/context)
- `crates/ferrostage-settings/src/schema.rs`, `src/tests/parse.rs`
- `assets/locales/{en-US,zh-CN}/app.toml`

Parent follow-up: update DESIGN/state/component contracts from old Notes-above-Next and fixed Start/Stop geometry to this approved hierarchy; document Start fullscreen as independent from startup preference. Final version/build/spec/native evidence remains parent-owned.
