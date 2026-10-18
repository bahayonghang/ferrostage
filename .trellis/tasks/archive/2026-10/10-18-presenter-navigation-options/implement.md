# Integration execution plan

Planning only. Product steps wait for explicit approval of the latest summary and the owning child's `task.py start` gate. Parent is not an implementation target.

1. Review PRD/design and task-local prototype; verify child boundaries and real context manifest paths.
2. Implement/check `10-18-page-browser`: contextual host, input, thumbnails, atomic preferences, old-config behavior and focused event regressions.
3. Implement/check `10-18-layout-navigation-popover` after contextual host API stabilizes: menu state, anchors, no-op/error and localization.
4. Integration: browser -> options and reverse, Escape without Stop, outside click without drawing/navigation, document replacement without stale targets, Stop/Home/Swap discard edits, audience notes isolation.
5. Run `just ci`, `just check-shell`, and existing PDFium-backed real-deck scheduling/retention tests with a verified staged DLL. Capture exits/evidence. Environment-blocked checks are not PASS.
6. Normal-window native smoke: visible input/caret, 2x3 and 4x4 real thumbnails, direct jump, upward menu apply/reopen, notes and audience correctness, relevant light/dark and zh-CN/en-US focus/copy. Do not add size/DPI/performance matrices.
7. Review combined diff, update root DESIGN.md and executable spec contracts during implementation, map A1-A10 to evidence and preserve explicit gaps. Commit only when authorized.

## Rollback

- Child 1 reverts browser/input/preference plumbing together, preserving original schema keys/physical navigation.
- Child 2 can revert independently without reversing child 1.
- Runtime reconfigure failure retains mapping; persistence failure retains effective dimensions with Unsaved/Retry.

Browser prototype review is planning evidence, not native product validation. No tests or build results are implied by these checklists.
