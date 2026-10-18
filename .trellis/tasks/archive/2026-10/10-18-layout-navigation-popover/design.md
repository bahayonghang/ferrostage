# Layout menu design

Parent shared visual/state spec is authoritative: `../10-18-presenter-navigation-options/design.md`.

## Boundaries and dependency
Reuse the contextual host/focus/dismissal added by child 1. This child owns layout-menu body/anchoring, document reconfiguration result handling and matching locale changes/tests. Primary files: `presenter/workbench_view.rs`, `presenter.rs`, `shell.rs`, `actions.rs`, locale files. Do not create another overlapping modal system or alter core annotation policy.

## State and transaction
Open captures current DocumentOverrides and detected layout result, not a draft requiring Apply. Clicking one layout forms next override; toggle flips only its own bool.
If identical -> close only. Otherwise build candidate document -> descriptor -> core ReconfigureDocument -> assign session document -> invalidate frames -> publish -> close.
On error, retain original document/override/check states, keep menu open and show local error.
Reopen reads actual effective session data. No SettingsStore write for overrides.

## Semantics and visuals
320px preferred width, anchor right-aligned 8px above toolbar trigger, 12px radius and existing palette. Six >=44px single-choice rows, separator, two >=44px check items with short helpers. All share a reserved check column and visible keyboard focus. Auto check indicates None override, while quiet detected label describes effective layout. No diagnostics dump.
If strokes exist, contextual notice explains existing clear-on-reconfigure effect. Same-mode close does not clear strokes. Localized real warnings survive in ordinary feedback. If available height is insufficient, scroll inside menu body; do not replace/reflow workbench.

## Events and privacy
Menu owns arrow/Home/End focus movement; Enter/Space activates once. Tab follows visible focusable order. Escape/outside close only, consume event/click and return trigger focus. Shared host guarantees only one contextual surface. Stop/Home/Swap/replace dismiss and clear stale menu errors. Existing deferred dispatch and first-frame captured data remain mandatory.
Physical navigation changes overlay grouping, not independent-notes exclusion while recognition remains enabled. Audience sees only authorized content/crop snapshot.

## Rollback
Revert menu/copy/reconfiguration UI paths together, keeping child 1 contextual browser intact. No settings/dependency migration.

## Supporting annotation-state query

Current audience snapshots expose only current-page strokes, while document reconfiguration clears strokes on every page. Add one read-only `PresentationState::has_annotations()` query so menu warning data can reflect off-page annotations without exposing note content or adding any core UI/IO dependency. Empty per-page stroke containers must not imply visible annotations. Getter tests cover off-page state and the existing clear behavior. This changes no command, reconfiguration or persistence policy. Exact getter semantics are recorded in implementation handoff/spec.
