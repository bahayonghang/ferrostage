# Current source evidence, 2026-10-18

Clean `dev` at planning start. Current source/screenshots override historical memory. No product or dependencies changed.

| Finding | Evidence |
| --- | --- |
| Full-content Jump/Document replacement | `crates/ferrostage-ui/src/presenter.rs:815-890` |
| Jump is digit/backspace String editing | `crates/ferrostage-ui/src/shell.rs:633-644` |
| Range/content-map validation | `crates/ferrostage-ui/src/shell.rs:586-600`; `transactions.rs`, `jump_page` |
| Logical/step/physical progress data | `crates/ferrostage-ui/src/presenter/workbench_view.rs:43-80` |
| Secondary grid/document triggers | `crates/ferrostage-ui/src/presenter/workbench_view.rs:595-623` |
| Fixed 135px grid tiles and two click targets | `crates/ferrostage-ui/src/presenter/workbench_view.rs:978-1058` |
| Direct thumbnail jump and grid close | `crates/ferrostage-ui/src/shell.rs:485-488` |
| Six layouts, booleans and debug string | `crates/ferrostage-ui/src/presenter/workbench_view.rs:1062-1124` |
| Reconfigure success leaves document modal open | `crates/ferrostage-ui/src/shell.rs:489-508` |
| Explicit-open-only thumbnail work | `crates/ferrostage-ui/src/workbench.rs:241-255` |
| Scheduler epoch change retires stale work | `crates/ferrostage-ui/src/workbench.rs:265-275` |
| Columns 1..4, visible count 1..16 | `crates/ferrostage-settings/src/schema.rs:24-28` |
| Known-key merge for both grid leaves | `crates/ferrostage-settings/src/merge.rs:85-101` |
| Notes exclusion before physical overlay grouping | `crates/ferrostage-document/src/builder.rs:127-160` |
| Reconfigure clears annotations and installs map | `crates/ferrostage-core/src/state.rs:437-446` |

## Retain/adapt existing regressions

Search `crates/ferrostage-ui/src/shell.rs` by names after edits:

- `v5_jump_editing_precedes_global_backspace_and_numeric_bindings`
- `v5_jump_numeric_text_precedes_configured_navigation`
- `v5_stop_discards_jump_input_before_following_navigation`
- `v5_swap_failure_discards_abandoned_jump_input`
- `v5_escape_from_jump_discards_input_without_stopping`
- `v6_removed_jump_commit_restores_keyboard_traversal`
- `workbench_controls_jump_directly_and_keep_note_pages_private`
- `thumbnail_hover_and_cancelled_press_are_inert_both_click_targets_jump`

Retain workbench frame/scheduling tests, settings tests for 13 visible pages, session-strip/first-render/focus/privacy regressions. Select new tests from final behavior changes, not an unrelated qualification matrix.

## Decisions and limits

Unify duplicate page-finding surfaces. Label actual layout/navigation scope. Distinguish mutually exclusive choices and independent toggles. User explicitly confirmed <=4x4. Repeated current selection must not cause existing annotation loss.

Referenced `C:/Users/lyh/.codex/RTK.md` was absent; no rules inferred. UI and Trellis startup/brainstorm skills and relevant project specs were read. This is planning/source evidence only: no implementation, Cargo results or native acceptance.
