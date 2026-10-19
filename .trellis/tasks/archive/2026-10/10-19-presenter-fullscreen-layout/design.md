# Window lifecycle and layout design

## Source anchors

`shell.rs:1251` starts the audience transaction; `shell.rs:1464` applies app-start fullscreen only while creating Presenter. `workbench_view.rs:739` lays out panes; `schema.rs:234` defines persisted ratios. See parent research for full repository-relative paths.

## Fullscreen transaction

Capture a presenter restore record at Start (window identity, MonitorId, normal bounds and maximized/fullscreen state). Reuse the established placement confirmation/timeout mechanism. Prefer changing the existing presenter window in place so focus, drafts and note scroll survive; verify available maintained-GPUI window APIs before coding. If recreation is required, preserve those states in Shell and explicitly review that mechanism first.

Treat presenter fullscreen confirmation and audience placement as prerequisites for committing Start. Handle any partial failure by curtain/retiring the candidate audience and restoring Presenter. Do not derive success from a toggle request. Suppress recovery logic that would recreate a window because its cached fullscreen record is stale. Preserve the existing core/session transaction boundaries rather than introducing a second independently committed Start path.

Stop closes/curtains audience first, then restores the saved presenter state when the same display remains available. The restore record is one-shot; no per-Start settings save. If display topology changed, use the existing MonitorId recovery policy and never restore private content onto the assigned audience role. Startup fullscreen setting remains independent. Finish any pending restore before accepting a new Start; retain usable fallback on native restore failure.

## Layout and control hierarchy

Use theme.rs and root DESIGN tokens. New default right width 0.40; notes height fraction remains 0.65. The upper-right Next pane uses the complement of `notes_ratio`. Its image is aspect-fit within that allocation. The lower Notes pane is scrollable and consumes the rest. Persisted custom ratios override defaults.

After reordering panes, vertical drag computes `notes_ratio = clamp(1 - y_within_right / right_height, 0.55, 0.8)`, accounting for actual divider/padding bounds consistently. Horizontal drag retains current right-width semantics and clamps. No implicit inversion of stored notes_ratio. Test both drag directions and round-trip persistence, not just visual order.

Presentation header: quiet clock/title, prominent elapsed timer with pause/reset, secondary More and display access. More holds Open, Home and Settings with existing stop/import guards. Footer keeps one End control, Previous/position/Next, blackout and drawing/grid actions; group secondary drawing choices without removing functionality. Target a roughly 48-56px header and 56-64px footer at default text scale, but preserve >=44px interactive targets and allow text preferences to grow. Compactness comes from fewer competing controls, not tiny hit areas. Ready retains useful editing/management controls.

Notes source typography is owned by the Notes child. No mode/size controls are added. Original note images use full available width and natural aspect height inside the existing vertical scroll container. Keep component IDs stable or explicitly migrate focus routes. No animated panel movement; pointer press acknowledgment uses existing native states.

## Rollback

Keep layout/window lifecycle changes in a bounded change set. On native failures prefer a usable windowed presenter and ended/cancelled presentation over an ambiguous partially committed Start. Revert this child without deleting custom settings or changing audience privacy boundaries.
