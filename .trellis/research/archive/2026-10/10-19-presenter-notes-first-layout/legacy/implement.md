# Implement: presenter notes-first layout

Order matters: contracts first, then settings, then layout, then polish.

1. Contracts
   - [x] DESIGN.md line 112 ("08:" paragraph): notes-first, one column ratio 0.60 in 0.40..0.75, notes fit rule, canvas letterbox on presenter panes (replace "Keep slide aspect ratio and letterboxing"), 1 px divider in an 8 px target. Update Responsive Behavior text (line 158 section).
   - [x] `.trellis/spec/frontend/component-guidelines.md` line 59 (workbench ordering and Notes wrapper rule).
   - [x] `.trellis/spec/frontend/state-management.md` lines 29 and 35 (`effective_notes_ratio`, ratios, vertical drag) and the matrix.
2. Settings (`ferrostage-settings`)
   - [x] `schema.rs`: add `notes_column_ratio` + `effective_notes_column_ratio()`; default 0.60; remove `effective_notes_ratio()` after callers are gone.
   - [x] `parse/sections.rs`, `merge.rs`: read/write the key; legacy keys round-trip.
   - [x] Tests: missing key, clamp, round trip with unknown leaves. Update `tests/parse.rs` lines 511-525 and `legacy_notes_ratio_and_strip_are_readable_without_restoring_small_notes` (line 561).
3. Intent and shell
   - [x] `actions.rs` `Intent::Split { notes_column, save }`; `shell.rs:630-640` handler.
   - [x] `shell.rs:2003` initial presenter target width from `1 - effective_notes_column_ratio()`.
4. Workbench layout (`presenter/workbench_view.rs`, `presenter.rs`)
   - [x] New composition (slides column, column split, notes panel with one card layer and 8 px padding).
   - [x] Notes fit rule with measured panel size and 1 px hysteresis.
   - [x] `picture()`: black background only for `Surface::Thumbnail(_)`.
   - [x] Remove vertical split and its drag branch; `split_drag: Option<bool>` becomes `bool` (`presenter.rs:168`, `388`).
5. Settings panel + locales
   - [x] `settings_panel.rs:110-129`: replace two ratio fields with `presenter.notes_column_ratio`.
   - [x] `presenter/settings_view.rs:597-605`: slider whitelist uses the new key.
   - [x] `assets/locales/zh-CN/app.toml`, `assets/locales/en-US/app.toml`: new label; remove labels that no field uses.
6. Header timer buttons
   - [x] `presenter.rs:420-428`: add `Intent::Core(PauseTimer | ResumeTimer | ResetTimer)` to `quiet_header`.
7. Tests
   - [x] `shell/presenter_layout_tests.rs`: replace `presenter_split_drag_tracks_lower_notes_height_and_persists` with a column-drag test (AC4); update `original_notes_keep_width_and_natural_height_for_multiple_pages` (AC2).
   - [x] `shell.rs` tests that read old selectors or ratios: `workbench_split_release_outside_saves_and_future_schema_stays_readonly` (6053), `workbench_narrow_stacks_and_grid_wheel_never_navigates` (6114), `layout_menu_applies_each_layout_and_retains_upward_workbench_geometry` (6348), `layout_menu_keyboard_toggles_and_dismissal_do_not_leak_navigation` (6411), `presenter_bar_shares_one_row_and_notes_fill_the_column` (6855). `page_browser_outside_press_dismisses_without_drawing_or_click_through` (4200) reads only `current-pane`; check it still passes.
   - [x] `shell/preferences_tests.rs`: lines 117-126, 242-274 (slider `slider-presenter.next_slide_ratio`), 489-490, 514 (`horizontal-split`), 545-553 (`Intent::Split`), 641-672, 846.
   - [x] `settings_panel.rs:423` validation test that edits `presenter.notes_ratio`.
   - [x] New geometry test at 1700x1000 (AC1), letterbox test (AC5), timer face test (AC6).
   - [x] After edits: `rg "next_slide_ratio|notes_ratio|horizontal-split|vertical-split|next-notes-pane" crates` lists only legacy schema/parse/merge fields and their round-trip tests.
8. Gates
   - [x] `just ci`.
   - [x] `just dev` with thesis `slides/deck-notes/defense.pdf` (after the thesis task rebuilds it): before/after screenshots, measured glyph height, dual-display Start/Stop smoke. Record PASS/FAIL/NOT_RUN in `research/validation.md`.
   - [x] Superseded: the notes-first layout was reverted at user request (see `research/validation.md`); no separate 16:9 measurement.

Review gates: after step 2 (settings round trip), after step 4 (headless geometry), after step 8.

Rollback points: each step is one commit-sized unit on `dev`; revert in reverse order. No installation or release in this task.
