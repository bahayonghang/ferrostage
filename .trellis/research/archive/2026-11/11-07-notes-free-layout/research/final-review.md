# Final independent code review

Reviewed the complete notes-free-layout change against its PRD/design/implementation plan and frontend component/state/quality/privacy contracts. This review excludes the main session's future projection-status and return-position planning changes and its archived crop-link correction.

## Findings fixed

- The Presenting menu initially sent keyboard focus from Crop to disabled layout row 0 when settings were read-only and Reset was unavailable. Visible controls and key dispatch now share `layout_row_enabled`; navigation searches only enabled rows and Enter/Space cannot activate a disabled row. `read_only_layout_menu_skips_disabled_reset_and_retains_session_reset` uses real GPUI keyboard input to verify the disabled Reset is skipped, Crop remains reachable, and a notes-free session-only Reset still works without saving read-only settings.
- Narrow DualNotes workbenches render Standard proportions, but Reset initially chose the hidden NotesFirst settings from StartMode. `ResetWorkbench { id, layout }` now captures `Presenter::visible_workbench_layout(window)`, the same decision used by the rendered panes. The coordinator keeps the existing menu id/generation guard. `narrow_notes_first_reset_restores_the_visible_standard_proportions` verifies actual menu keyboard input resets only visible Standard proportions and preserves stored NotesFirst ratios. The main session approved this internal task-local correction.

No remaining source findings. Whole-document layout selection is independent of texture readiness; pending crop confirmation keeps the notes workbench. Current/Next keep contain-fit pixels and independent notes stay private. Session ratios reset only on a successful new open, and saved notes-layout ratios remain separate. Last-page detection correctly handles SlideMap::next_step returning the current page at its end, and the scheduler does not request the current page as Next there.

## Final checks

- `just ci`: PASS, including formatting, strict workspace Clippy, 249 workspace tests and cargo deny. Log: `runs/notes-free-review/ci-final.log`.
- `just check-shell`: PASS, including current GPUI/PDFium application build, strict product Clippy and 250 GPUI tests. Log: `runs/notes-free-review/check-shell-final.log`.
- The independent UI-library run also passed 250 tests: `runs/notes-free-review/ui-final.log`.
- CI ran before the final product build, leaving `cache/perf-product-target/debug/ferrostage.exe` as the final native GPUI product for the main session's source-bound check.
- The initial reviewer CI and an implementer rebuild collided with a running native acceptance executable and stopped at Windows' file lock. These were execution-environment failures; after the main session closed that process, both full final gates passed without source waivers. Earlier logs remain available rather than being presented as passes.

The main session owns native UI acceptance and its final executable hash/observations. Headless tests and this code review do not claim physical projection, native layout acceptance or an additional window/display matrix.
