# Validation: dismissible document-warning flag

Date: 2026-10-20. Toolchain 1.99.0. Headless results only. Dark-theme paint was not opened in a Ready window.

## Commands

| Command | Result |
| --- | --- |
| `cargo test -p ferrostage-ui --features test-support --lib --locked` | PASS: 231 passed, 0 failed |
| `cargo clippy -p ferrostage-ui --features gpui,test-support --all-targets --locked -- -D warnings` | PASS |
| `just ci` → `fmt-check` | PASS |
| `just ci` → `clippy` (`--workspace --all-targets -D warnings`) | PASS (default features; does not compile `gpui`) |
| `just ci` → `test` (`--workspace --all-targets`) | PASS |
| `just ci` → `deny` (`cargo deny check`) | NOT_RUN: `cargo-deny` is not installed (`error: no such command: deny`). No dependency or `deny.toml` change. |

## Acceptance items (prd.md)

| Item | Status | Evidence |
| --- | --- | --- |
| OverlayHeuristic sentence is `document-warning-0` with the flag chrome; content height stays within 0.5px after dismiss | PASS | `shell::tests::document_warning_flag_floats_and_dismisses_for_one_generation` |
| Close hides that string for the current generation; layout reconfigure keeps it hidden; a new generation shows it again | PASS | same test: generation stays 2 across `reconfigure_layout`, then generation 3 shows `document-warning-0` |
| Two document warnings stack upward with an 8px gap; closing one leaves the other | PASS | flag gap within 1px of 8; after dismissing mixed geometry, `document-warning-1` is absent and `document-warning-0` remains |
| Flag bottom is at or above `workbench-controls`; left inset is 12px | PASS | both flag bottoms `<=` tools origin; `flag0.origin.x - content.origin.x` within 1px of 12 |
| A `shell.warnings` string is `shell-warning-0` and stays after document-warning dismiss | PASS | same test, Ready. Idle still paints `shell_warnings` outside the document branch |
| Tab reaches Close, Enter dismisses, Escape leaves the flag up | PASS | focus walk to `warning_dismiss_focus(0)`, then Escape, then Enter |
| Card, border, and warning colors come from `palette()` | PASS by code | `document_warning_flags` uses `p.card`, `p.border`, `p.warning`. No separate light/dark frame was captured |
| Audience window has no `document-warning-0` | PASS by code path | The flag is a child of the presenter content branch only. `audience.rs` has its own `render` and was not given the warning string. The test does not open an audience window |

## Deviations

- The test loads with `Command::LoadMappedDocument` and assigns `workbench.document`. It does not call `finish_import`, which can save settings and push watch errors.
- `DocumentWarning::MixedGeometry` is pushed only to build a two-flag stack. Reconfigure rebuilds the document, so that pushed warning disappears. Survival uses `OverlayHeuristic` from two pages that share the label `"3"`.
- The close face is 44×44 inside `activation_target`'s focus reservation. Text wraps at `max_w(312px)` so the row stays inside the 420px flag.
