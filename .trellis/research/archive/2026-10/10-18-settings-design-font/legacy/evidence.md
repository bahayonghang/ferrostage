# Settings implementation checkpoint

Date: 2026-10-18. Branch: `codex/workbench-settings-design`. Changes remain uncommitted. This is the first sequential implementation child; independent review and integrated native acceptance remain coordinator-owned.

## Implemented

- Root `DESIGN.md` retains `brand:claude` provenance and warm palette. Settings now use sans headings, quiet check-marked category navigation, a bounded main column, section headings/hairlines and label/control rows, instead of per-field cards. Close is in the header; preference save feedback is scoped to the content column and does not advertise immediate effect for restart-only settings.
- General groups Appearance, Startup and Recent files. Language/startup/fullscreen explanations are local. Choices and numeric controls use grouped soft surfaces, shared deferred activation and visible focus, preserving 44px faces.
- Compatible schema-v1 `general.ui_font_family` (empty system-default sentinel) and `general.ui_font_size` (integer 12..=24, default 16), authoritative range, leaf diagnostics/defaults, checked edit rejection and known-leaf merge. Config example updated. Unknown keys, future read-only policy and failed-save runtime retention remain intact.
- Installed family names are cached by Shell for the session. Enumeration is deferred until the selector first opens, except a persisted override requires resolution before first draw. The bounded inline selector offers filtering, six visible choices, keyboard Up/Down/PageUp/PageDown/Enter/Escape, pointer page controls and explicit System default. Filter changes do not save. Closing selection returns focus to its invoker.
- `theme::Typography` resolves the saved family without rewriting a missing request; fallback warning is local to the family row. Presenter captures resolved typography, and headings/labels/control text/errors/tooltips/home copy/display settings/timer sizes consume scaled roles. Home Georgia wordmark stays fixed; timer keeps Consolas. No audience, PDF/raster, platform, core, media or dependency code changed. Identify overlays retain their pre-existing fixed diagnostic typography.
- Mixed Chinese/Latin preview, live size stepper/direct editing and independent family/size restoration. Recent history retains actual order/limit, displays basename and full parent context, includes accessible full-path labels and tooltips, shows count/limit and content-sized disabled-when-empty Clear. No history opening/pinning/removal feature added.

## Checks

| Check | Result |
| --- | --- |
| `cargo test -p ferrostage-settings -p ferrostage-ui --locked` | PASS: 42 settings and 35 UI tests, plus doc tests. Includes font old/default/invalid/future parse, real store roundtrip with unknown data, failed-save effective fonts, resolver and family/history model checks. |
| `cargo test -p ferrostage-ui --features test-support --lib settings_ --locked` | PASS: 15 tests. Existing real keyboard recording, numeric editing/slider, reset/read-only/save retry, all-category rendering and shared display transaction tests, plus new real font filter/select/size/default/Escape/history test. No new size/DPI test matrix was added; one unchanged existing category regression includes its historical dimensions. |
| `cargo test -p ferrostage-ui --features test-support --lib settings_fonts_filter --locked` | PASS after strengthening the test to inspect actual observed Presenter typography following real keyboard edits, persistence and default restoration. |
| `cargo clippy -p ferrostage-ui --features test-support,pdfium --all-targets --locked -- -D warnings` | PASS: native GPUI production/test compilation and strict lint. |
| `cargo fmt --all` / `git diff --check` | PASS. |

The initial sandboxed Cargo invocation could not access the existing managed target cache. Authorized elevated Cargo runs succeeded. An initial test used a nonexistent `load_session` helper and native checks exposed a missing alert element ID and a test initializer lint; all were fixed and the passing gates above rerun. No waiver or dependency change was introduced.

## Remaining integration acceptance

Coordinator preserved baseline captures under parent `research/native/before-*.jpg` and the baseline launch/exit receipts before implementation. This child did not launch or automate the native desktop and has no after screenshot or rebuilt application receipt. Compile/headless GPUI input tests do not establish aesthetic quality, installed-native-font selection/fallback, restart rendering, or Windows save-lock behavior. Coordinator will run ordinary-window core/visual checks after all three children and the independent review. Full integrated `just ci` and `just check-shell` are deferred to that checkpoint to avoid repeated equivalent full gates.

Displays still has its old fixed-height/nested-scroll body at this checkpoint, intentionally owned by the final displays child. The workbench child may now consume the typography and control foundation. No commit, push, archive or task completion status was performed here.


## Coordinated final acceptance

The implementation checkpoint above is historical. Independent review and actual native core/visual verification subsequently completed. See the parent [final verification](../10-18-workbench-settings-design/research/verification.md) for AC mapping, final source/executable identity, screenshots, 194 default and 136 GPUI checks, and exact limits. Font restart/default/fallback, full recent paths, workbench Start/Stop and compact display allocation/movement were observed. Native clipboard paste remains inconclusive, not PASS. No commit/push/archive.

