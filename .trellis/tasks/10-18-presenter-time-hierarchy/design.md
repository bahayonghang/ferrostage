# Proposed design

Status: approved for implementation on 2026-10-18. Product verification remains pending.

## Visual direction

- Visual thesis: quiet instrument typography on the existing warm presenter canvas; presentation time is a secondary focal point below slide content.
- Content plan: left clock, document identity and stage; flexible space; labeled presentation timing and its existing controls; utility actions; a toolbar without a separate neutral-tool button, a three-row layout menu, and numbering choices inside the existing jump panel.
- Interaction thesis: no new motion. These are continuously changing readouts; animation would add distraction. Existing control activation and focus feedback remain shared.

## Recommended treatment: A, unboxed instrument readout

At the default 16px UI setting, propose these logical pixel roles, all scaled through `Typography::size`:

| Element | Typography | Color | Placement |
| --- | --- | --- | --- |
| Local clock | Existing Consolas, 14px regular | `p.muted` | First header item, before filename/stage |
| Timing label | Existing UI font, 12px medium | `p.muted` | Immediately before presentation digits, 8px gap |
| Presentation digits | Existing Consolas, 28px medium | `p.ink`, existing warning/error role at thresholds | Existing timing region before actions |
| Pause/warning information | Existing localized UI typography | Existing semantic roles plus text | Timing group, preserving controls |

Retain header 16px horizontal / 8px vertical padding and existing 12px group gaps. Keep default header height at its current control-driven height; do not stack a new caption above the digits. No background, border, glow, or shadow for either readout. The size change is 24px to 28px, approximately 17%, rather than a display-sized timer. Dark tokens remain canvas `#181715`, ink `#FAF9F5`, muted `#B9B3A8`; light equivalents remain unchanged.

Layout sketch, not a rendered preview:

```text
11:17:50  defense.pdf  [Ready]       Elapsed 00:00    Home  Open PDF  Settings  Displays
11:18:23  defense.pdf  [Presenting]  Elapsed 00:33    Pause  Reset  Home  ...
```

During Presenting, preserve the existing pause/reset controls in the timing region. The sketch omits translation and actual control geometry. The clock is visually quiet but has an accessible current-time label. Readouts are not focusable buttons, and second updates must not become repeated accessibility announcements.

Use the existing captured timer mode and overtime condition to choose localized semantic labels. Keep textual threshold warnings. Existing Resume text remains visible while paused; add explicit paused text only if the native review finds that control label insufficient. Do not invent an additional timer state machine.

Use monospaced digits, stable value alignment, and a minimum reservation large enough for a sign and three-digit minute count. Let longer values grow fully rather than enforcing a clipping maximum. Reserve the sign/digit space without modifying the formatted value. Do not truncate numerical readouts. Existing compact header composition remains; this task does not create a breakpoint system.

## Alternative: B, soft inset

The same 28px labeled timer inside `p.soft`, 8px radius and restrained padding. This increases discoverability but also increases visual weight and can suggest clickability. Prefer A for the user's explicit request that timing not compete with slides. Do not implement both or add a setting to switch them.

## Code boundaries

- `crates/ferrostage-ui/src/presenter.rs:596-711`: separate clock placement from timer, retain the filename's shrinkable region, stage and action identities, and existing narrow-header branch.
- `crates/ferrostage-ui/src/presenter/workbench_view.rs:21-24,130-153,375-438`: keep captured strings and warning/pause state; split out clock rendering and style the presentation readout. Capture the minimal mode/label discriminator if needed. Do not read Shell during rendering or add a poll loop.
- `crates/ferrostage-ui/src/theme.rs:14-16,23-50`: reuse font roles, size scaling and palettes. Name new timing size constants only if useful for multiple consumers; avoid a new token framework.
- `assets/locales/{zh-CN,en-US}/app.toml`: add paired timing/clock labels through the existing catalog contract.
- `DESIGN.md`: synchronize its timer role and the bounded header composition after approval and implementation.

The time-display change needs no core, settings schema, audience snapshot, PDF loading, rasterization, or dependency-source changes. The added default-policy change affects the document layer as described below. Existing preferences probes may be extended only to support meaningful rendered-placement/behavior checks.

## Toolbar and notes-layout simplification

Remove the `("navigate", "wb-navigate", Tool::Navigate)` entry from `Presenter::tools_panel`. Retain `Tool::Navigate` as the default neutral internal state and preserve `Shell::dispatch(Intent::Tool)` toggling back to it when the selected tool is reactivated. That existing path ends a stroke and clears the laser. Do not change Escape semantics or introduce a replacement navigation button/shortcut.

Reduce the layout menu to three visible rows:

| Row | Existing meaning |
| --- | --- |
| Automatic | Explicit opt-in aspect-ratio layout detection, `layout=None` |
| No split | `layout=Some(NotesLayout::None)`, proposed new default |
| Identify independent notes pages | Existing `identify_independent_notes` toggle, default true |

Remove directional choices from `LAYOUTS` and the physical-navigation checkbox from this menu; reduce focus/scroll-anchor arrays and keyboard traversal from eight entries to three. Derive the row count from the bounded menu structure rather than retaining obsolete `% 8`, End=7, separator=6 or layout-radio boundary=6 values. Preserve two layout radio rows and the independent-notes checkbox with their accessible roles. Keep the existing trigger name for this bounded scope. Do not reuse `DocumentOverrides.physical_pages` for the new numbering selector: it controls overlay grouping, a different concern. Keep lower-level compatibility and current grouping unchanged when switching browser numbering.

The detected-layout description cannot depend on finding a matching item in the reduced choices: explicit Automatic can still detect Right/Bottom, and internal callers can still supply directional overrides. Use a complete localized layout-description mapping independent of visible row choices. Keep description-only directional locale keys where needed; remove only keys with no remaining consumer. Never leave the current `find(...).unwrap()` pointing into an incomplete options list.

Change `DocumentOverrides::default().layout` from `None` to `Some(NotesLayout::None)` while retaining independent-note recognition and physical navigation defaults. This makes `LoadedDocument::load` consistently safe for new imports across entry paths. It preserves explicit `layout=None` as Automatic and preserves `reload`'s existing reuse of document-local overrides. Do not serialize this session policy into settings or rewrite PDFs.

Revise current tests that assume automatic crop on `LoadedDocument::load`: automatic-detection tests should request it explicitly, while new default tests must prove wide/tall pages remain full-frame. Existing mixed-geometry warnings tied to Automatic remain tested under explicit Automatic. Structure diagnostics and independent-note warnings remain intact. Do not remove crop/layout enums or clean-room document support from the document/render layers.

Additional affected sources:

- `crates/ferrostage-ui/src/presenter/workbench_view.rs`: tool list only; preserve pointer handling and ordinary slide navigation.
- `crates/ferrostage-ui/src/shell.rs`: existing tool-toggle contract and real-input regressions; change dispatch only if a verified regression requires it.
- `crates/ferrostage-ui/src/presenter.rs`: menu focus/anchor arrays and initial focus.
- `crates/ferrostage-ui/src/presenter/layout_navigation.rs`: three-row composition, selection, key handling and independent detected-layout labeling.
- `crates/ferrostage-document/src/layout.rs`, `builder/tests.rs`: default policy and regression cases; `builder.rs` remains the reference for explicit Automatic and notes mapping.
- `crates/ferrostage-ui/src/shell/layout_navigation.rs`, `shell/reload.rs`, `workbench/tests.rs` and relevant GPUI tests: preserve guarded reconfiguration/reload and adapt menu expectations to three rows.
- `DESIGN.md`, `.trellis/spec/frontend/component-guidelines.md` and directly contradicted specs: synchronize six-layout/eight-row claims and independent-note default after implementation.

## Risks and rollback

Presenting adds pause/reset controls that are absent from the supplied Ready screenshot. Verify that actual composition, not only Ready. Preserve long filename tooltip/accessibility and numerical values when allocating space. Default/no-GPUI CI cannot establish correctness of this GPUI surface.

Rollback the time treatment independently if needed. For the controls/default scope, revert only the default override, affected menu/tool composition and corresponding tests/docs together. Keep core timing, settings schema and audience types untouched.

## Dual-mode jump panel

See `page-browser-design.md` for the panel composition, numbering contract, state flow, thumbnail ownership and approved D1 notes rejection. Treat this as a separate implementation slice after the safe defaults/control cleanup, within the same user-requested task. It needs both UI and scheduling changes; editing the numeric label alone is incomplete. Recheck integration across all slices before completion. Implement as one coordinated task; D1 is resolved and does not block execution.
