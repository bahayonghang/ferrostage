# Validation: presenter notes-first layout

Date: 2026-10-19. Branch `dev`, uncommitted working tree.

| Check                                      | Command or test                                                                                                                                                                                     | Result                                                                          |
| ------------------------------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------- |
| AC1 geometry at 1700x1000                  | `notes_first_geometry_contains_tall_and_wide_notes_pages`                                                                                                                                           | PASS                                                                            |
| AC2 multiple notes pages                   | `original_notes_keep_width_and_natural_height_for_multiple_pages`                                                                                                                                   | PASS                                                                            |
| AC3 settings default, clamp, round trip    | `notes_column_ratio_clamps_and_round_trips_with_legacy_and_unknown_leaves`, `legacy_notes_ratio_and_strip_are_readable_without_affecting_notes_column`                                              | PASS                                                                            |
| AC4 real-input column drag and persistence | `presenter_column_drag_sets_notes_width_and_persists`, `workbench_split_release_outside_saves_and_future_schema_stays_readonly`, `settings_v09_closed_panel_split_waits_for_apply_and_rollback_ack` | PASS                                                                            |
| AC4 settings row edits the same value      | slider assertions in `preferences_tests.rs` (`slider-presenter.notes_column_ratio`)                                                                                                                 | PASS                                                                            |
| AC5 no black letterbox on presenter panes  | `presenter_panes_have_no_black_letterbox`                                                                                                                                                           | PASS                                                                            |
| AC6 timer buttons use quiet header face    | `timer_buttons_share_the_quiet_header_face`                                                                                                                                                         | PASS                                                                            |
| AC7 `just ci`                              | `just ci`                                                                                                                                                                                           | PASS (exit 0; `cargo deny` warnings are pre-existing)                           |
| GPUI lint                                  | `cargo clippy -p ferrostage-app -p ferrostage-ui --features gpui,pdfium --all-targets --locked -- -D warnings`                                                                                      | PASS                                                                            |
| Headless UI tests                          | `cargo test -p ferrostage-ui --features test-support --lib --locked`                                                                                                                                | PASS, 213 tests                                                                 |
| Test-support lint                          | `cargo clippy -p ferrostage-ui --features test-support --all-targets --locked -- -D warnings`                                                                                                       | PASS                                                                            |
| AC8 native dual-display check              | `just dev` with thesis `slides/deck-notes/defense.pdf`, before/after screenshots, glyph height                                                                                                      | NOT_RUN: needs the user's dual-display setup; the thesis PDF is not rebuilt yet |

AC6 keyboard Enter/Space on the timer buttons: the activation path (`activation_target`) is unchanged; no new test for it.

Headless tests do not establish native visual acceptance or standing readability.

## Revision after user review (2026-10-19)

The user reviewed the notes-first layout in the native app and asked to restore the earlier layout: Current large on the left, Next smaller at the upper right, Notes larger at the lower right, left:right = 55:45.

- Reverted f33f097 layout, `notes_column_ratio`, `Intent::Split { notes_column }` and the notes fit rule. `next_slide_ratio` and `notes_ratio` control the layout again.
- Fresh `next_slide_ratio` default is 0.45 (was 0.40). `notes_ratio` default stays 0.65.
- Kept from f33f097: presenter panes letterbox on the workbench canvas (thumbnails stay black); Pause/Resume/Reset timer use the quiet header face. Tests `presenter_panes_have_no_black_letterbox` and `timer_buttons_share_the_quiet_header_face` stay.
- The user's own settings file now has `next_slide_ratio = 0.45`, `notes_ratio = 0.65`; the unused `notes_column_ratio` line is removed.

| Check                     | Command                                                                                                         | Result          |
| ------------------------- | --------------------------------------------------------------------------------------------------------------- | --------------- |
| `just ci`                 | `just ci`                                                                                                       | PASS (exit 0)   |
| Headless UI tests         | `cargo test -p ferrostage-ui --features test-support --lib --locked`                                            | PASS, 212 tests |
| GPUI lint                 | `cargo clippy -p ferrostage-app -p ferrostage-ui --features gpui,pdfium --all-targets --locked -- -D warnings` | PASS            |
| Test-support lint         | `cargo clippy -p ferrostage-ui --features test-support --all-targets --locked -- -D warnings`                  | PASS            |
| Native dual-display check | user check in the native app with the rebuilt thesis PDF                                                         | PASS (user, 2026-10-19) |
