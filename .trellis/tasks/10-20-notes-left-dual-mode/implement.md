# Implement: notes-left dual-screen mode and toolbar/badge polish

Order matters: contracts, then settings, then policy, then UI. Each step ends with its own check.

## Checklist

1. [x] Contracts (R10): update `DESIGN.md` (Layout 08, button-secondary/tool-button, status/timer), `.trellis/spec/frontend/component-guidelines.md:59`, `.trellis/spec/frontend/state-management.md:35,149`. No code yet.
2. [x] Settings (`crates/ferrostage-settings`): `StartMode::DualNotes` + `as_str` `dual_notes`; parse choice list; `PresenterSettings` two fields, defaults, `effective_*` clamps; parse + merge; tests (AC1).
   - Check: `cargo test -p ferrostage-settings --locked`.
3. [x] Policy (`crates/ferrostage-ui/src/policy.rs`): `Dual | DualNotes` delegate to `audience_target`; extend matrix test (AC2).
4. [x] Layout type: `WorkbenchLayout { Standard, NotesFirst }` in `workbench.rs`, `StartMode::workbench_layout` mapping (in UI as a free fn if keeping settings free of UI types).
5. [x] Split intent: `Intent::Split { layout, primary, secondary, save }` in `actions.rs`; shell handler (`shell.rs:659`); `split_drag: Option<SplitAxis>`; `finish_split`; update existing Standard drag tests to the new shape.
6. [x] Workbench composition: branch in `workbench_panel`; Notes-first tree with `previews-column`, `column-split`, `row-split`; `pane_caption` helper; keep Standard path byte-for-byte in behaviour (AC4, AC5, AC6).
7. [x] Raster seed: layout-aware width at `shell.rs:2062`.
8. [x] Start-mode menu: 3 rows, `start_row_focus: [_; 3]`, width 360, locale keys in zh-CN and en-US, `wb-current-slide` (AC3).
9. [x] Focus reservation: transparent rest border in `activation_target` (AC7).
10. [x] Toolbar faces: chevron icon + trailing-icon support in `button`; trigger kinds -> Navigation; Destructive radius 8; trigger height in Ready (AC7). Run `theme.rs` contrast tests.
11. [x] Stage badge dot and caption style (AC8).
12. [x] Full check: `just ci`. Then `cargo test -p ferrostage-ui --features test-support --lib --locked`.
13. [x] Native (AC10): `just dev` with `defense.pdf`, dual-display, dark theme. Screenshots: Standard Ready, Notes-first Ready, Notes-first Presenting, open menu. Record in `research/validation.md` (PASS/FAIL/NOT_RUN with reason).

## Validation commands

```bash
cargo test -p ferrostage-settings --locked
cargo test -p ferrostage-ui --features test-support --lib --locked
just ci
just dev
```

## Review gates

- After step 1: contracts reviewed against prd.md R1-R10.
- After step 6: Standard geometry tests unchanged and green before touching styling.
- After step 11: trellis-check full-scope review (spec compliance, privacy boundary, no audience change).

## Rollback points

- Steps 2-8 are one feature commit; steps 9-11 are a separate styling commit, so either can be reverted alone.
- Settings rollback needs no migration (see design.md, Mode model).
