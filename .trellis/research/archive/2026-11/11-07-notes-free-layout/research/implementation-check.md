# Notes-free workbench implementation check

The layout decision reads the whole loaded document's effective same-page crop and independent-notes mapping, plus crop confirmation and pending replacement review. Raster availability does not select the layout. A notes-free document uses `WorkbenchLayout::Previews` in Dual, DualNotes and Single modes; mixed documents keep their existing Standard/NotesFirst layout when a particular page has no notes.

The two-pane layout starts with Current/Next shares of 70/30. Next has a 240 logical-pixel minimum above the existing 800 logical-pixel stacking boundary. Its ratio lives only in `Workbench::previews_ratio`; a successful new import resets it, while switching presentation modes or temporarily using a notes layout preserves the saved settings. The layout menu's reset action checks its menu id and document generation and changes only the current layout's ratios. Read-only settings still permit the session-only split/reset.

The terminal Next pane remains present with an end-of-presentation label. `SlideMap::next_step` clamps to the current page at the terminal step, so both the presenter and render scheduler compare the next page with the current page rather than interpreting `Some` as a subsequent step. Failure copy is keyed to the exact current raster request; retained frames keep the existing fallback behavior.

## Automated coverage

- Ordinary document in all three modes; wide layout and both sides of the existing breakpoint; minimum Next width and final-step placeholder.
- Real divider drag followed by mouse release; real menu keyboard wrap/activation and invoker focus return; unchanged saved note ratios, core snapshot and settings dirty state; reopening another document resets the session ratio.
- A mixed document's empty page and notes-loading page keep identical Notes panel bounds.
- Unconfirmed same-page notes keep the original Notes workbench; a confirmed whole-page rule can use the two-preview composition.
- Note raster failure state follows current generation/page/crop/target identity and excludes a stale page's failure.
- The independent reviewer added disabled-row keyboard eligibility coverage for read-only Presenting and read-only notes-free session reset.

The first completed `just check-shell` passed app build, product Clippy with `-D warnings`, and 248 UI tests. Log: `runs/notes-free-check-shell.log`. A final run after the reviewer's keyboard fix is recorded separately in `runs/notes-free-check-shell-final.log`.

Native window acceptance is owned by the main session and is not claimed by these headless checks.

## Historical handoff (resolved)

The attempted final `check-shell` product relink was blocked by the main session's running native acceptance process (`Access is denied` on `debug/ferrostage.exe`); it did not replace that executable. The independent reviewer owns the remaining build/Clippy gate after native acceptance exits. The reviewer's full UI-library run passed 249 tests after the keyboard eligibility fix (`runs/notes-free-review/ui.log`).

Review also identified a narrow DualNotes reset mismatch: the stacked view uses Standard proportions while the reset coordinator initially chose NotesFirst from StartMode. The reviewer and main session are coordinating the correction; this receipt does not claim that edge case is closed yet.

Final resolution: both review fixes passed the full 250-test UI suite and final product build/Clippy; CI passed 249 workspace tests and cargo deny. The final native narrow-reset retest is recorded in [verification](verification.md).
