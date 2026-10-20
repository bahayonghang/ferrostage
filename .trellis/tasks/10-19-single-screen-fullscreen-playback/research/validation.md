# Validation

| Check | Command / test | Result |
|---|---|---|
| AC1 | `policy::tests::single_monitor_defaults_to_shared_fullscreen_and_keeps_windowed_mode` | pass |
| AC2 | headless: `shell::tests::single_screen_fullscreen_playback_takes_input_and_projects_slides_only` (injected committed state) | partial; placement and commit activation are native-only |
| AC3 | same test: right/left navigation from the audience window, `ctrl-o` leaves `focus_intent` empty, `escape` returns to `Ready` with only the presenter window | pass |
| AC3 guard probe | removed the `single_screen_playback()` check in `request_focus`, reran the test | test failed on `focus_intent.is_none()`; file restored byte-identical |
| AC4 | same test: `slide_crop == LEFT_HALF`, navigation 0 -> 2 -> 0 -> 4 skips notes pages 1 and 3 | pass |
| AC5 | `tests::parse::single_monitor_mode_reads_legacy_presenter_only_as_fullscreen` | pass |
| AC6 | `display_settings::tests::single_screen_choices_roundtrip_without_duplicate_role_or_inherited_mode`, `stale_missing_ambiguous_and_conflicting_destinations_are_rejected` | pass |
| AC7 | `cargo metadata --locked --offline --no-deps`: 10 packages, versions `['1.2.0']` | pass |
| AC8 | `just ci` (`just-ci.log`) | exit 0 |
| AC10 | `start_mode_menu_selects_single_persists_and_hides_while_presenting`, `single_start_mode_plays_fullscreen_on_the_presenter_at_any_monitor_count`, `start_mode_defaults_to_dual_and_round_trips` | pass |
| AC11 | `about_tab_shows_version_author_build_time_license_and_repository` | pass |
| AC12 | `single_screen_start_accepts_an_untoggled_maximized_presenter`: failed before the fix (probe), passes after | pass |
| AC13 | `event_error_flag_closes_and_persistent_monitor_error_has_no_close_button` | pass |
| Native build/lint | `just check-shell` (`check-shell.log`): gpui,pdfium build, clippy `-D warnings`, 218 UI tests (run with `CARGO_TARGET_DIR=target/ci-alt` because a running `just dev` locked `target/debug/ferrostage.exe`) | exit 0 |
| AC9 | native one-monitor / Duplicate playback with `defense.pdf` | NOT_RUN: no tool in this session can drive the native window |

## Existing tests changed

Five no-play tests used one monitor. No-play on one monitor is now invalid (D1), so they add a second test monitor through `add_spare_monitor` before the no-play draft. Their assertions are unchanged.

## AC9 checklist

1. One monitor or Windows Duplicate mode; `just dev`; open `defense.pdf`.
2. Start: slides fill the screen, no taskbar, no notes half, no presenter controls.
3. Right/PageDown/Space/left click advance; Left/PageUp go back; B/W blank.
4. Ctrl+O and the settings shortcut do not bring up the presenter window.
5. Escape returns to the presenter window in Ready with the page kept.
