# Validation

## Headless (2026-10-20)

| AC | Evidence | Result |
| --- | --- | --- |
| AC1 | `ferrostage-settings` `start_mode_defaults_to_dual_and_round_trips` (adds `dual_notes`), `notes_first_ratios_default_clamp_and_round_trip_without_touching_standard_ratios` | PASS |
| AC2 | `policy::tests::dual_notes_start_mode_places_exactly_like_dual` (0/1/2 monitors x both single-monitor modes) | PASS |
| AC3 | `start_mode_menu_selects_notes_first_and_switches_the_ready_layout` (3 ordered rows, 360 px menu, saves `dual_notes`, layout switches); `start_mode_menu_selects_single_persists_and_hides_while_presenting` updated to row 2; `start_mode_menu_keyboard_reaches_every_row_without_navigating` (Down/Down/Enter selects row 3 Single, Up/Enter selects Notes first, page unchanged). Check fix: the menu previously lost keyboard focus to the unrendered modal handle and arrows were not captured; both repaired. | PASS |
| AC4 | `notes_first_layout_puts_notes_left_and_larger_current_above_next` at 1700x1000: notes 0.55 of width, full height, Current above and taller than Next at 0.62 share | PASS |
| AC5 | `notes_first_dividers_drag_live_and_persist_only_their_ratios`: column to 0.62, row to 0.70, saved once, Standard ratios unchanged | PASS |
| AC6 | Same AC4 test: Single renders Standard; 700 px width stacks Current, Next, Notes in Notes-first mode. Existing Standard drag/notes/privacy tests unchanged and green. | PASS |
| AC7 | `start_mode_menu_selects_notes_first_and_switches_the_ready_layout` (trigger height = Start height, menu width 360, drawn `start-mode-chevron-closed/open` and `document-options-chevron-closed` elements); `polished_faces_keep_semantic_badge_and_regular_clear_radius` (Clear radius 8); `theme.rs` contrast tests. Transparent rest border is source-reviewed, not asserted by pixel. | PASS (partial: no pixel assertion of the rest border) |
| AC8 | `polished_faces_keep_semantic_badge_and_regular_clear_radius` (dot colours); body-on-soft contrast is covered by `used_text_and_focus_pairs_meet_design_contrast` | PASS |
| AC9 | `just ci`: fmt, clippy `-D warnings`, workspace tests pass; the `deny` recipe fails with `no such command: deny` because cargo-deny is not installed on this host. No Cargo.toml/Cargo.lock/deny.toml change. Re-verified: `cargo test -p ferrostage-ui --features test-support --lib` 224 passed; GPUI clippy clean. | PASS except deny NOT_RUN (tool missing) |
| AC10 | Native user check. User reply on 2026-10-20 after the implementation report: "没问题". No screenshots were attached to this session. | PASS (user acceptance) |
