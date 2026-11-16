# Implementation plan

Status: implementation and scoped verification finished on 2026-10-18. Checkboxes below record completed work against research/verification.md; that report explicitly distinguishes executed, source-verified and NOT_RUN qualification cases. Commit and push remain outside this request.

## Start gate

- [x] Obtain explicit approval of the latest summary: default enabled, completed left click, normal playback only, existing Next step semantics.
- [x] Recheck task status, branch/diff, and source anchors; preserve unrelated changes.
- [x] Load the Phase 1.4 gate, validate curated manifests, then activate implementation through `task.py start` only after approval. Use the project's Trellis implement/check agents when entering implementation; each dispatch starts with the actual active task path and owns only the files below.

## One cohesive implementation

These are ordered work items inside one independently mergeable change, not separate partial releases.

### 1. Extend settings and compatibility coverage (R2, R5)

- [x] `crates/ferrostage-settings/src/schema.rs`: add the bool and its approved default to DisplaySettings.
- [x] `crates/ferrostage-settings/src/parse/sections.rs`: add the optional bool parser branch with full leaf warning path.
- [x] `crates/ferrostage-settings/src/merge.rs`: write the known leaf while retaining unknown display values and monitor metadata.
- [x] `crates/ferrostage-settings/src/tests/parse.rs`: prove absent/current/future/invalid/explicit false cases.
- [x] `crates/ferrostage-settings/src/tests/store.rs`: prove true/false readback, unknown data retention, failed-save effective value and retry. Reuse the existing scratch-store fixture.
- [x] Run `cargo test -p ferrostage-settings --locked`.

### 2. Expose live settings without placement side effects (R2, R5)

- [x] `crates/ferrostage-ui/src/settings_panel.rs`: add the Displays Toggle field and retain field edit/serialization tests.
- [x] `crates/ferrostage-ui/src/presenter/settings_view.rs`: add field-specific helper text using the existing label/control row, focus, disabled, and accessibility behavior.
- [x] `assets/locales/zh-CN/app.toml` and `assets/locales/en-US/app.toml`: add the labels/helper text named in design.md.
- [x] `crates/ferrostage-ui/src/shell/preferences.rs`: exclude the new non-placement field from the display-layout comparison; keep the committed settings as sole enable authority.
- [x] `crates/ferrostage-ui/src/shell/preferences_tests.rs`: drive the real toggle using pointer/keyboard, prove effective/durable values, unchanged audience/presenter handles, page/timer/monitor roles, and that actual fullscreen/monitor edits still enter placement handling. Cover save-failure and future-read-only paths using existing fixtures.

### 3. Connect audience input through Shell (R1, R3, R4, R6)

- [x] `crates/ferrostage-ui/src/audience.rs`: add payload-free outgoing AdvanceRequested and a stable root on_click handler accepting completed mouse-left input. Keep all incoming view data snapshot-only.
- [x] `crates/ferrostage-ui/src/shell.rs`: subscribe at audience creation, capture source handle, apply all design guards using current state/settings/projection, then dispatch Intent::Next once.
- [x] Add GPUI tests in the existing shell test module using headless_shell/test_window/redraw and VisualTestContext mouse down/up. Assert actual page changes from actual audience events, not only a directly invoked command.
- [x] Cover enabled/disabled; mouse-down-only/right/middle/outside release; presenter surface isolation; letterbox client-area clicks; two completed clicks; Beamer overlay/notes mapping; final page; stopped/pending/retired audience; modal/blank/Curtain; busy load/reload/preferences/transaction; and input after live toggling without reopening the audience.
- [x] Keep existing tool/media input regressions and verify timer/stage invariants around mouse Next. Reuse existing small fixtures; no PDFium dependency is needed for headless input dispatch tests.

### 4. Check and capture evidence

- [x] Run `cargo test -p ferrostage-ui --features test-support --lib --locked`.
- [x] Run `just ci` for the default feature gate.
- [x] Run the product compilation/lint checks: `cargo build -p ferrostage-app --features gpui,pdfium --locked` and `cargo clippy -p ferrostage-app -p ferrostage-ui --features gpui,pdfium --all-targets --locked -- -D warnings`.
- [x] Native smoke executed with the product build and real Beamer PDF: enabled left click advances once; toggle off/on applies to the same window; black/white and a settings dialog prevent mouse navigation; the last page stays put; disk readback/reopened settings/restarted fullscreen session retain the preference. Windowed and fullscreen playback passed on this three-screen host. First-inactive physical click and the one-monitor-only exception are NOT_RUN with the exact tool/host limits recorded in research/verification.md. No window-size/DPI matrices were added.
- [x] Record each check as PASS/FAIL/NOT_RUN with evidence. Headless tests do not prove OS activation/fullscreen delivery; compiler success does not prove interaction. If native tooling is unavailable, record that limitation rather than fabricating acceptance or expanding unrelated validation.
- [x] Update `.trellis/spec/frontend/state-management.md` with the outgoing event, coordinator guards and non-placement preference contract, and `.trellis/spec/backend/database-guidelines.md` with optional-v1 leaf/default/merge rules after implementation evidence exists.
- [x] Run `git diff --check` and inspect complete scoped diff. Changes total approximately 15 existing files plus task evidence. No core/vendor/dependency edits are expected; investigate scope drift before adding any.

## Exit criteria and rollback

All AC1-AC6 have traceable evidence or an explicitly recorded native verification gap requiring user disposition. Final review must cover settings -> effective policy -> audience event -> Next -> projection, plus setting save/restart. Leave commit/push/task archive to separately authorized session follow-through. If runtime behavior is rejected, disable the setting; if implementation is withdrawn, revert the cohesive change and keep the harmless unknown TOML leaf.

## Final verification receipt

See `research/verification.md` and `research/check-review.md`. Reload-specific input is source-verified only; first-inactive physical click and one-monitor-only native route are NOT_RUN. Full-feature deny is blocked by three unchanged existing unmaintained advisories. All implemented behavior and the scoped automatic/native checks passed. Do not interpret the native checklist above as blanket qualification of these explicitly unexecuted cases.
