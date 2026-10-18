# Display layout and runtime-location design

## Evidence and host boundaries

Parent research F8–F11 identifies the fixed settings embed at `settings_view.rs:97`, nested display scroll at `display_settings.rs:334`, full-width aspect diagram at `:392`, draft-only selected roles at `:461`/`:471`, and repeated diagnostics at `:550`. Actual location is absent from DisplayPanel, although native `window.display(cx)` already exists in shell logging. This task includes a small observation/projection change, not only styling.

Keep the existing shared display module. Extract a body with explicit framing/scroll ownership for its two real hosts only: settings content scroll owns the embedded body; standalone dialog owns its own scroll/action frame. Remove fixed 650/450px and embedded `.size_full()` constraints that manufacture overflow. Retain IDs and Intent routing for Identify, Refresh, SelectPresenter, SelectPdf, Apply and Cancel.

## Compact composition

Top row: detected count, short numbering note, Identify and Refresh. Next: spatial overview beside compact assignment rows when the normal content width permits. This uses horizontal space rather than making the drawing fill the full content width. The diagram is a bounded contain-fit viewport with role summary; all three rows align with shared numbers. With insufficient available space, retain natural stacked content scrolling, never shrink labels/targets or reintroduce an inner scroll. Fullscreen preferences follow the allocation overview as ordinary compact rows; they do not displace the primary display content with tall cards.

The viewport uses at most 420px in the side-by-side form, up to 720px when stacked, with a target height of 240px. Fit desktop rectangles with a single uniform scale and center offset, preserving relative positions, gaps, negative origins and portrait aspect. Do not stretch the world to each monitor independently. Numbers and short role labels stay readable; for a tiny physical rectangle use a number-linked callout rather than overflowing another screen. Current app/PDF roles must be visible on the diagram, not only in a legend. The supplied ordinary three-screen arrangement is the primary visual review case, not a new size qualification matrix.

Each row: screen number, friendly name, concise resolution/orientation/primary metadata, role markers and two compact allocation controls. Role controls indicate what choosing them does (application/PDF destination); selected controls refer explicitly to pending allocation when it differs. Keep “Do not play PDF” and single-monitor windowed option. Technical identity source, binding confidence and GDI ratio live in a keyboard-accessible details disclosure. Active degraded/missing/stale warnings remain outside it. No runtime driver-path paragraphs in the default scan path.

## Three-layer role projection

Parent design section 4 defines the authority table. Compute one view projection from current topology, effective settings, current draft and native observations, consumed by diagram/summary/rows. Suggested fields include actual application/PDF location (`MonitorId` or explicit unknown/not-running), effective resolved targets and pending target labels. Keep it Presenter-only; do not put it into the audience snapshot or persisted settings. Use `MonitorId` as identity, human numbers only for labels.

Actual observation approach: per-owned-window sample cell associated with the window handle. Seed it from `window.display(cx)` during creation and update it from the existing bounds observer. Shell consumes it at its existing tick boundary and maps platform display ID through the current unique mapping. Observe changes, not requested WindowRecord.monitor/display. Only notify on changed effective projection. Re-sample on panel open/Refresh/Apply as appropriate, without OS enumeration on every frame or native window reads from Presenter rendering. Both old and new records remain separately owned during transactions; only confirmed current records contribute actual markers. Retired handle observations cannot revive markers. If bounds events do not cover native moves, verify and add only the required display-change subscription.

Unavailable topology, stale mapping, ambiguous reverse mapping or failed window read yields unknown with a helpful status, not the configured target. Manual OS move changes actual label only; explicit Apply remains the allocation authority. GPUI's current owning display is authoritative for crossing windows, avoiding hand-written mixed-unit hit testing. A Presenting windowed audience on the same screen may show both app and PDF; Ready always says PDF not running, even with a target selected. Stopping/Home retires audience marker immediately with the window lifecycle.

## Interaction and feedback

Solid/checked role marker with text means current observation; configured but not running is explicitly labeled; outlined “pending” means draft. Do not distinguish only by color. Clicking assignment changes draft markers and local pending feedback, never windows. Cancel restores draft from effective settings; it cannot undo separately autosaved fullscreen preferences. Apply reuses existing validation and window-confirmation transaction. Keep selection/errors during pending work per existing guards. Settings Saved status is qualified as preferences saved while allocation still needs Apply. Close/Escape never implicitly Apply.

## Ownership and rollback

Owned targets: `display_settings.rs`, `presenter/settings_view.rs` display host, `shell.rs` window observation/projection and existing transaction boundary integration, necessary display intents/view-local disclosure state, locales, targeted tests and root DESIGN display contract. Preserve preceding children's typography/workbench edits. Do not modify the monitor identification algorithm/platform device API, Audience input model, core or vendor. Restore shared body/host/observation/projection/locales/tests/docs together on rollback; observations are session-only and no new persistent allocation schema is needed.
