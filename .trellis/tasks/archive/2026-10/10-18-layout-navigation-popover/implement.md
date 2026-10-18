# Execution plan

Dependency: child `10-18-page-browser` shared contextual host API is implemented and checked. Start only after final design approval.

1. Read parent contract and first child's host API; record any necessary shared-file integration changes.
2. Add tests for upward contextual render, checked groups, close after success, identical-layout no-op, error retention and independent toggle on/off.
3. Replace full-content Document branch with anchored menu and localized Layout & navigation trigger.
4. Apply transactional selection/no-op/error close behavior, detected Auto result and annotation notice. Preserve raw diagnostics in internal tracing if needed, and actual localized warnings in existing feedback.
5. Exercise real keyboard/pointer/AX and outside-click/Escape: no presentation leakage, no click-through drawing, focus restored, one contextual surface.
6. Retain notes/overlay privacy and core reconfigure regressions; run focused tests, `just ci` and `just check-shell`. Parent performs final combined verification after any integration changes.
7. Normal-window native apply/reopen with real deck and relevant locale/theme states; record build checks separately from native acceptance. Update root DESIGN.md/specs, no size/DPI matrix.

Rollback is menu-only relative to child 1. No product PASS until actual results exist. Commit only when authorized.
