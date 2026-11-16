# Implement: presentation timer resume and header timer redesign

Order: contracts, core, shell wiring, views, copy. Each step ends with its own check.

## Checklist

1. [x] Contracts: update `DESIGN.md` per design.md §5 (Layout 08 header, button-secondary/tool-button split session control, status/timer capsule, layout-navigation-menu downward, `timer.fontSize` 30px). Update `.trellis/spec/frontend/state-management.md` with the resumable-session lifecycle (Stop ≥1 s → resumable; Load/Close/Start clear; hot reload keeps). No code yet.
2. [x] Core (`crates/ferrostage-core`): `resumable` flag, `resumable_elapsed(now)`, `PresentationCommand::ResumePresentation { audience_available }` + `name()`, allowed-stage table, Stop sets flag (≥1 s), Start/Load/Close clear it. Tests in `state/lifecycle_tests.rs` (resume continues, restart zeroes, <1 s not resumable, load/close clear, commit keeps, invalid stage/no audience leaves state unchanged) and the generator in `state/property_tests.rs` (invariant `resumable ⇒ Ready`).
   - Check: `cargo test -p ferrostage-core --locked`.
3. [x] Shell wiring (`crates/ferrostage-ui`): `StartTimer { Restart, Resume }`; `TransactionKind::Start(StartTimer)`; `Shell::start(timer, cx)`; apply `StartPresentation` / `ResumePresentation` on placement confirm; `Intent::Resume`; F5 `Start` action resolves Resume when resumable (`actions.rs`). Update every `TransactionKind::Start` match and the shell tests that drive Start.
   - Check: `cargo test -p ferrostage-ui --features test-support --lib --locked`.
4. [x] Presenter data: `WorkbenchData` timer state enum (`Fresh | Running | Paused | Stopped`), label key, warning level, `resumable: Option<String>` formatted with the shared timer formatter.
5. [x] Header (`presenter.rs`): identity / centered `timer` / `actions` (`flex_1`, `justify_end`); move `document-options` trigger into actions with hairline divider; narrow stacked branch keeps a centered capsule row.
6. [x] Timer capsule (`workbench_view.rs::timer_panel`): soft capsule, canvas-drawn state mark, fixed label slot, 30px digits with fixed min width, inline Pause/Reset with divider (Presenting only).
7. [x] Bottom strip (`workbench_view.rs::tools_panel`): split Resume/Restart when `Ready` + resumable, else existing Start/Stop; remove `document-options` from `secondary`; busy/disabled/focus order per design.md §4.4.
8. [x] Layout & navigation menu anchoring (`presenter/layout_navigation.rs` and its placement in `presenter.rs`): downward, right-aligned under the header trigger; chevron orientation; focus restoration unchanged.
9. [x] Copy: new keys in `assets/locales/zh-CN/app.toml` and `en-US/app.toml` (design.md §4.6); locale parity test passes.
10. [x] UI tests: split button presence and dispatch; F5 resolution in both states; header contains `document-options`, bottom strip does not; menu opens below its trigger; timer label per state; capsule digits x-position independent of filename length (debug-selector bounds). Theme contrast tests for the new pairs in `theme.rs`.
11. [~] Full check: `just ci`. fmt, clippy and tests pass; `cargo deny` NOT_RUN because `cargo-deny` is not installed (see research/validation.md).
12. [x] Native (user-confirmed 2026-10-20, screenshots not archived): `just dev` with a real deck, dual-display, dark + light. Scenario: Start → wait → Esc → frozen → Resume continues → Esc → Restart zeroes; open Layout & navigation from header. Screenshots for Ready fresh, Presenting running, Presenting paused, Ready stopped (split button), open menu. Record in `research/validation.md` (PASS / FAIL / NOT_RUN with reason).

## Validation commands

```bash
cargo test -p ferrostage-core --locked
cargo test -p ferrostage-ui --features test-support --lib --locked
just ci
just dev
```

## Review gates

- After step 1: contracts reviewed against prd.md R1–R4.
- After step 3: core + shell green before any view change; Audience snapshot type untouched (grep diff).
- After step 10: trellis-check full-scope review (core boundary, audience isolation, no settings schema change, copy parity).

## Rollback points

- Steps 2–3 (behavior) are one commit; steps 4–10 (visual) a second commit; either reverts alone. Core change is additive.
