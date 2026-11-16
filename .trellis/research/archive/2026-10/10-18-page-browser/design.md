# Page browser technical design

Read parent `../10-18-presenter-navigation-options/design.md` for exact visual and shared-state contracts.

## Ownership and boundaries
Own contextual host and its one-open-surface/focus/dismissal contract; numeric editor; browser data/projection; page toolbar entry; atomic grid preference intent. Main files: `crates/ferrostage-ui/src/presenter.rs`, `presenter/workbench_view.rs`, `shell.rs`, `actions.rs`, `workbench.rs`, focused tests and locales. Extract a small `presenter/page_browser.rs` if it avoids expanding the existing view; no generic popup framework.

Shell owns target edit and committed navigation separately. Presenter consumes captured data; do not synchronously reread Shell during native first draw. Expose a minimal shared contextual host API for the second child. Do not implement that child's menu items here.

## Data flow
Open -> close other contextual surface -> seed current physical number -> reveal current step -> focus/select field.
Edit -> update local string/selection -> checked physical candidate -> content-map membership -> target context/reveal only.
Submit/tile -> reject busy/stale generation/map target -> one core JumpTo -> close -> restore invoker.
Density -> validate both values -> one Settings clone/edit/apply/save for columns and product -> schedule only the visible group.
Close -> discard edit/target -> set grid false -> existing scheduler epoch/texture reconciliation -> no thumbnail retention.

## Grid math
Capacity = columns * rows <=16. Browsing indexes `slide_map.steps()`, not arbitrary PDF integers. Open locates current step. Density changes preserve first visible step, clamp after changes; group movement clamps to last valid start without repeats. Final rows may have empty cells. Wheel advances by columns. Group buttons advance capacity.

Old arbitrary counts stay readable and unchanged until explicit density edit. A custom legacy count shows a custom summary, not an inaccurate selected row. On first column choice choose min(4, ceil(old_visible/old_columns)) rows; on first row choice retain old columns. Both preference leaves commit together. Do not loosen parser ranges or rewrite unrelated settings fields.

## Editor and input routing
Use a focused native numeric text editor with selection/caret and plain-text clipboard insertion. Verify the pinned GPUI API in maintained source before use; no new dependency. Intercept editor keystrokes before configurable presentation bindings, while preserving focused Go/Close/grid behavior. Keep existing shared deferred Shell dispatch, modal Escape priority and accessibility activation. Unsupported AX value/caret APIs must be documented as a gap.

## Render lifetime and rollback
Use existing on-demand flag/scheduler/epoch rather than a second renderer. Target preview is only slide-map context, not a full-resolution staging projection. Generation, crop/map and index identify tiles; stale candidate intents never address a new document. Hide/retire old frames rather than showing mismatched crops.
Revert this child's code as one unit; existing schema keys and physical input remain compatible.
