# Execution plan

Dependency: none. Start only after approval of parent design and child planning gates.

1. Inspect current first-render/focus/input and maintained GPUI editor APIs. Record concrete API evidence if any gap affects the required editor contract.
2. Add focused regressions for opening browser, inert edits, numeric key conflicts, excluded notes, submit-once, tile pointer cancel and Escape focus return. Adapt existing Jump tests to observable behavior, not obsolete Modal variants.
3. Implement minimal contextual host and bounded numeric editor; keep workbench underneath, preserve Settings/Displays flows.
4. Unify grid/progress/Jump entry points, add real thumbnails/current-target-focus states, independent row/column choices and group math.
5. Add atomic density apply/save, legacy custom handling, save-failure/read-only cases; verify real store readback.
6. Retain scheduler/texture retirement and hot-reload tests with real PDF fixture. Prove <=16 visible requests and zero when closed.
7. Run focused tests, `just ci`, `just check-shell` and existing PDFium scheduling regression. No repeated broader tests without a new failure/change.
8. Record normal-window native browser interactions, light/dark and relevant locale copy. Update root DESIGN.md and affected specs, communicate shared host API to child 2. No size/DPI matrix.

Rollback: revert editor/host/grid/preference changes together. Do not commit without authorization. Product completion requires concrete tests/native evidence, not this checklist.
