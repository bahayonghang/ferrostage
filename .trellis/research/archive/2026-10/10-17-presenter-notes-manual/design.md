# Technical design

## Boundary and evidence

One integrated task is appropriate because navigation, pane selection, render scheduling and regression expectations change together. Use ordered work packages rather than independently shipping incompatible children. Do not modify maintained GPUI packages.

The source anchors below describe the pre-change baseline bf9e87cd0c2cd96df41457dae7dd7d0c09b10a48; line positions move in the implementation.

| Source | Evidence |
| --- | --- |
| crates/ferrostage-ui/src/shell.rs:721 and :747 | Poll/finish import applies LoadMappedDocument without calling Start. |
| crates/ferrostage-ui/src/shell.rs:1719 | Tick publishes and notifies each cycle; recurring timer is 25 ms. No timer-to-Next call found. |
| crates/ferrostage-ui/src/shell.rs:469, :558, :899 and jump_key | Grid/page input stages preview, later committed. |
| crates/ferrostage-core/src/state.rs:324 | Step chooses preview versus committed page from hidden state. |
| crates/ferrostage-ui/src/presenter/workbench_view.rs, workbench_panel | Next precedes notes; vertical drag measures bottom area. |
| crates/ferrostage-ui/src/workbench.rs:254 | Grid OR strip setting requests thumbnails. |
| crates/ferrostage-ui/src/presenter.rs:207 | Shared button exposes only primary versus secondary styling. |
| crates/ferrostage-app/src/main.rs:49 | Subscriber falls back to info. |

## Navigation and timer

Introduce one validated direct-jump core command using content-page map validation and logical-slide synchronization. Route grid selection and submitted page input through it. Invalid input must not partially mutate page, annotations or timer. Grid selection closes the grid and restores presenter focus.

Remove staged-preview UI/data branches. Migrate all workspace consumers before removing orphaned preview core state/commands. Replace old feature tests with direct-navigation assertions while preserving independent generation, annotation, lifecycle and isolation coverage. Do not delete broad tests just because preview assertions changed.

Page dialog is input only: typing and Escape do not navigate; Enter/Go commits once. Keep configured commit-key compatibility as dialog submission, with no action outside a relevant dialog. Preserve focused-button consumption before global Space/Next.

Timer remains display-only. Reproduce unsolicited movement with the real fixture before attributing a cause; inspect queued/held input and import/start behavior if observed. Do not invent a timer removal where no timer navigation exists.

## Visual thesis and implementation

- Visual thesis: quiet warm-neutral native presentation console, readable notes as the dominant support pane, restrained clay presentation actions.
- Content plan: compact filename/status/timer header; current slide left; labeled notes upper right and next slide lower right; grouped controls; recent files only on Idle.
- Interaction thesis: no timed animation for frequent navigation. Immediate press feedback, explicit selected state and direct divider dragging.

Use existing theme.rs as the only style/token layer, no CSS. Update root DESIGN.md before code and preserve provenance. Retain system sans fonts and monospace timer. Never tint PDF pixels.

Variants: filled clay Start, semantic Stop/clear, strong neutral navigation, labeled black/white blanking swatches, segmented/outlined drawing tools with selected marker, quiet utilities. Reuse palette semantic roles; any new role needs both-theme contrast tests. Shared button owns handlers/focus/disabled/accessibility; do not duplicate them by variant.

## Layout and settings compatibility

Reverse notes/next order and measure vertical drag from the top. Give notes initial effective share near 0.65. Normalize legacy low shares to a readable minimum in one shared effective-layout calculation; align drag limits and settings UI with that range and record it in DESIGN.md. Existing right-column width remains adjustable; check normal/minimum native dimensions before changing its default.

Remove persistent strip from composition and scheduling. Keep show_thumbnail_strip as a deprecated read-compatible field for the existing schema, hide its editable settings control, ignore it at runtime and document the no-op. Do not change schema just for this removal. Grid page-count bounds remain.

Recent files render only on Idle. Preserve aspect fit and notes scrolling. Labels/grouping must not consume the space recovered from strip/recent rows. Report unsupported size probes honestly.

## Performance and logs

First eliminate hidden thumbnails and record closed/open/closed-grid requested/submitted work and retained bytes. Preserve single-thread PDFium, exact generation/page/crop/size matching and shared BGRA allocations.

Investigate redundant snapshot notification using existing equality/identity contracts. Suppress unchanged audience publication only with transition tests. Do not deep-copy image bytes or whole documents every tick. Presenter must still display second-boundary clock/timer, new frames, media, settings/errors, topology and reload staging. Do not slow polling indiscriminately. If safe suppression requires wider lifecycle redesign, retain the proven thumbnail optimization and explicitly report remaining work.

Implementation investigation: BgraFrame equality compares image content, so whole-snapshot equality would introduce deep pixel comparisons on the 25 ms path. Defer notification suppression and retain the poll cadence. The bounded performance result for this task is hidden-thumbnail elimination and unchanged-schedule/retention regressions; native CPU/FPS improvements are not claimed.

Default tracing filter becomes warn, preserving RUST_LOG override and startup failures. Test through an isolated subscriber or subprocess, not racing global environment changes. Update logging spec. Compiler failures and native shutdown errors remain observable.

## Risks and rollback

Automatic advance remains unconfirmed by static inspection. Preview tests encode obsolete behavior and need careful replacement. Notification suppression can hide asynchronous updates. Larger raster notes do not reflow text, so real note content must be checked. Native screenshot/interaction capability is unconfirmed.

Keep navigation, visual/settings, and logging/performance changes independently reviewable. Revert only a faulty task-owned block if necessary, never unrelated user changes. No actual user-config mutation or policy waiver.
