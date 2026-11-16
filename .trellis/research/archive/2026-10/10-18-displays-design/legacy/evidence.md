# Displays implementation checkpoint

Date: 2026-10-18. Branch: `codex/workbench-settings-design`. Uncommitted third sequential child after settings/font and workbench. Independent review, integrated CI, final application build and native acceptance remain coordinator-owned.

## Implemented

- Root `DESIGN.md` extends its existing brand/palette and shared theme contracts with compact two-host allocation composition, contain-fit spatial preview, actual/configured/pending role meanings, collapsed diagnostics and scoped Cancel/save feedback. No catalog reapply or new design system.
- Both Settings and standalone Displays use one shared allocation body. Settings removes the 650/450px embed and delegates all scrolling to `settings-content`; standalone owns one `display-panel-content` scroll and a local allocation action footer. Fullscreen preferences follow the allocation body in Settings. Normal content uses overview beside rows, not a full-width aspect-height diagram. The preview fits one uniform physical-desktop scale into at most 420x240 logical pixels (up to 720px wide when stacked), retaining negative origins, relative position, gaps and portrait aspect.
- Diagram, number-linked callouts and corresponding rows consume the same Presenter-only `RoleProjection`. Actual role labels have checks and text, configured-but-not-running PDF says so explicitly, and pending role labels have an outline and explicit Pending text. Compact schematic labels use the same semantic keys; full descriptions stay available in linked callouts and accessible diagram labels. All detected monitors keep shared Identify numbers and explicit unused state.
- `WindowRecord` now owns an `Rc<Cell<Option<u64>>>` observation associated with that exact native handle. Creation, existing bounds observer and placement confirmation sample `window.display(cx)`. Existing tick window reads refresh the sample without adding a polling loop; failed reads clear the sample. Refresh/mapping preparation also samples current owned windows. `WindowRecord.monitor/display` remain requested placement only. Projection resolves the observation only through the current valid unique reverse mapping and current monitor identity, otherwise Unknown. Only current committed handles contribute. No audience in Ready/Idle means not running; configured None says Disabled. Stop/Home and retired records cannot revive a PDF marker. No observation is saved.
- The existing GPUI `Window::on_moved` registration (`vendor/gpui/src/window.rs`) calls `bounds_changed`, which refreshes the cached display and invokes the existing observer. A real TestPlatform movement callback regression verifies sample refresh; no vendor or extra display subscription was added. Empirical Windows crossing behavior remains native acceptance.
- Assignment controls still edit `DisplayDraft`. Apply still uses existing validation, native confirmation, rollback and persistence paths. In Settings, allocation Cancel now restores only the allocation draft and leaves the panel and already saved fullscreen preferences intact; standalone Cancel closes its panel. Close/Escape do not Apply. Saved preference feedback explicitly says allocation is pending when needed. Busy allocation controls preserve the coordinator guards.
- Per-display keyboard-accessible Details disclosure hides routine identity/GDI descriptions. Degraded identity, stale/missing/duplicate topology warnings, validation/operation errors and recovery remain visible. No platform monitor algorithm, core, PDF/raster, media, audience snapshot, dependency or vendor change.

## Checks

| Check | Result |
| --- | --- |
| `cargo test -p ferrostage-ui --locked` | PASS: 38 unit tests and doc tests. Three new meaningful model tests cover unique/unknown/unavailable/duplicate mapping, actual versus configured versus pending, manual-observation changes without allocation mutation, Cancel and single-screen shared app/windowed PDF. |
| `cargo test -p ferrostage-ui --features test-support --lib display --locked` | PASS: 22 focused tests, including existing Apply/failure rollback/save retry/no-play/Identify/read-only/stale revision tests. Two new actual GPUI tests cover native move callback sampling and current/retired owner lifecycle, plus one ordinary 1400x900 three-screen sample through both real hosts, all three rows/summary/actions visible, bounded topology, correct scroll ownership, disclosure input, draft-only selection and scoped Cancel preserving saved preference. Existing narrow regression is retained unchanged; no new dimension or DPI matrix. |
| `cargo clippy -p ferrostage-ui --features test-support,pdfium --all-targets --locked -- -D warnings` | PASS on final source: native GPUI production/test type and strict lint checks. |
| `cargo fmt --all` and `git diff --check` | PASS. |

The sandboxed Cargo invocation could not access the existing mbx managed cache; authorized cache access succeeded. The first default test pass warned about new test-only model code before its new tests were added; final default tests and strict native lint are clean. New two-host test setup initially omitted GPUI debug selectors, causing measurement lookups to be absent; selectors were added and the complete focused set rerun successfully. No guard/waiver was weakened.

## Acceptance and remaining native gaps

D1-D5 have implementation, pure model and focused native-input/coordinator evidence. D6 final native acceptance remains pending. Source tests and TestPlatform movement callbacks do not prove Windows OS monitor crossing or visual legibility. Coordinator owns the final default-launch ordinary-window Settings and standalone Displays screenshots (baseline `research/native/before-*.jpg` under parent), selection/Cancel, valid Apply, manual window move, Ready/Presenting, Start/Stop and source/executable receipts. Verify practical compactness at the actual ordinary window and agreement of diagram/callout/row actual and pending labels. No size/DPI/performance/physical-hotplug expansion is requested.

Full `just ci`, `just check-shell`, final application build, independent review and parent integration are intentionally deferred to the coordinator. This child did not launch or automate the desktop, commit, push, archive, activate another task or mark native acceptance complete. There are no placeholder product assets or copy to replace.

## Files changed by this child

`DESIGN.md`; `crates/ferrostage-ui/src/display_settings.rs`, `shell.rs`, `actions.rs`, `theme.rs`, `presenter.rs`, `presenter/settings_view.rs`; both locale `app.toml` files; this `evidence.md`. Earlier settings/workbench changes and coordinator-owned spec changes are preserved.


## Coordinated final acceptance

The implementation checkpoint above is historical. Independent review and actual native core/visual verification subsequently completed. See the parent [final verification](../10-18-workbench-settings-design/research/verification.md) for AC mapping, final source/executable identity, screenshots, 194 default and 136 GPUI checks, and exact limits. Font restart/default/fallback, full recent paths, workbench Start/Stop and compact display allocation/movement were observed. Native clipboard paste remains inconclusive, not PASS. No commit/push/archive.

