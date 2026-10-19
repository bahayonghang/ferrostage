# Implementation and verification

1. Under the user approval of 2026-10-19, load frontend state/component/quality and maintained-dependency specs. Recheck Shell Start/Stop/transaction/recovery flows and current GPUI fullscreen/maximize APIs.
2. Add restore state and confirmed fullscreen transition to the existing lifecycle. Cover already-fullscreen, normal/maximized, timeout, repeated input, Escape precedence and monitor-loss rollback with meaningful lifecycle tests.
3. Reorder panes, adjust new default ratio, invert vertical drag geometry, preserve stored meaning, lay out original note rasters at pane width/natural height with existing scrolling, and compact Presenting controls. Update focus routes/localized labels only where necessary. Test actual divider input and persisted round-trip.
4. Run `just ci` and `just check-shell`. Address the actual native feature graph, not only default-feature compilation.
5. Native smoke at the user's ordinary laptop view and full screen with assigned audience: Start/Stop twice, timer/page continuity, correct pane order, resizing via splitters, keyboard access, failed placement/recovery. Record requested vs actual fullscreen and monitor identity.
6. Record native evidence separately from headless tests. Update affected root DESIGN/spec contracts under the user approval of 2026-10-19. No size/DPI qualification matrix as a prerequisite.

Risk files: `crates/ferrostage-ui/src/shell.rs`, `shell/preferences.rs`, `presenter/workbench_view.rs`, `presenter.rs`, settings schema/parse/merge and locale/focus routes as needed. Do not broaden into home/recent changes. No product code is changed during this planning task.
