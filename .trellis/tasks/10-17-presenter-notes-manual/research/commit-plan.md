# Commit plan

One local work commit on codex/presenter-notes-manual. No push, merge, dependency upgrade, waiver or task archive.

Message: fix(ui): 🐛 优化讲稿布局并改为手动直接翻页

Includes the reviewed implementation, tests, locales, design/spec/docs and task evidence listed below. No unrecognized dirty files were found.

## Exact file scope

- `.trellis/spec/backend/directory-structure.md`
- `.trellis/spec/backend/logging-guidelines.md`
- `.trellis/spec/frontend/component-guidelines.md`
- `.trellis/spec/frontend/state-management.md`
- `.trellis/tasks/10-17-presenter-notes-manual/check.jsonl`
- `.trellis/tasks/10-17-presenter-notes-manual/design.md`
- `.trellis/tasks/10-17-presenter-notes-manual/implement.jsonl`
- `.trellis/tasks/10-17-presenter-notes-manual/implement.md`
- `.trellis/tasks/10-17-presenter-notes-manual/prd.md`
- `.trellis/tasks/10-17-presenter-notes-manual/research/baseline.md`
- `.trellis/tasks/10-17-presenter-notes-manual/research/check-review.md`
- `.trellis/tasks/10-17-presenter-notes-manual/research/check-shell-final.log`
- `.trellis/tasks/10-17-presenter-notes-manual/research/check-shell-reviewed.log`
- `.trellis/tasks/10-17-presenter-notes-manual/research/check-shell.log`
- `.trellis/tasks/10-17-presenter-notes-manual/research/ci.log`
- `.trellis/tasks/10-17-presenter-notes-manual/research/commit-plan.md`
- `.trellis/tasks/10-17-presenter-notes-manual/research/deny-all-features.log`
- `.trellis/tasks/10-17-presenter-notes-manual/research/focused-tests-final.log`
- `.trellis/tasks/10-17-presenter-notes-manual/research/focused-tests.log`
- `.trellis/tasks/10-17-presenter-notes-manual/research/gpui-tests.log`
- `.trellis/tasks/10-17-presenter-notes-manual/research/native-tests.log`
- `.trellis/tasks/10-17-presenter-notes-manual/research/native/accepted-build-launch.json`
- `.trellis/tasks/10-17-presenter-notes-manual/research/native/accepted-build-stderr.log`
- `.trellis/tasks/10-17-presenter-notes-manual/research/native/accepted-build-stdout.log`
- `.trellis/tasks/10-17-presenter-notes-manual/research/native/baseline-desktop-exit.json`
- `.trellis/tasks/10-17-presenter-notes-manual/research/native/baseline-desktop-launch.json`
- `.trellis/tasks/10-17-presenter-notes-manual/research/native/baseline-desktop-stderr.log`
- `.trellis/tasks/10-17-presenter-notes-manual/research/native/baseline-desktop-stdout.log`
- `.trellis/tasks/10-17-presenter-notes-manual/research/native/baseline-desktop.toml`
- `.trellis/tasks/10-17-presenter-notes-manual/research/native/baseline-exit.json`
- `.trellis/tasks/10-17-presenter-notes-manual/research/native/baseline-launch.json`
- `.trellis/tasks/10-17-presenter-notes-manual/research/native/baseline-stderr.log`
- `.trellis/tasks/10-17-presenter-notes-manual/research/native/baseline-stdout.log`
- `.trellis/tasks/10-17-presenter-notes-manual/research/native/baseline.toml`
- `.trellis/tasks/10-17-presenter-notes-manual/research/native/final-visible-end.json`
- `.trellis/tasks/10-17-presenter-notes-manual/research/native/final-visible-launch.json`
- `.trellis/tasks/10-17-presenter-notes-manual/research/native/final-visible-stderr.log`
- `.trellis/tasks/10-17-presenter-notes-manual/research/native/final-visible-stdout.log`
- `.trellis/tasks/10-17-presenter-notes-manual/research/native/final.toml`
- `.trellis/tasks/10-17-presenter-notes-manual/research/native/review-end.json`
- `.trellis/tasks/10-17-presenter-notes-manual/research/native/review-launch.json`
- `.trellis/tasks/10-17-presenter-notes-manual/research/native/review-stderr.log`
- `.trellis/tasks/10-17-presenter-notes-manual/research/native/review-stdout.log`
- `.trellis/tasks/10-17-presenter-notes-manual/research/native/run.ps1`
- `.trellis/tasks/10-17-presenter-notes-manual/research/perf100-native.log`
- `.trellis/tasks/10-17-presenter-notes-manual/research/thumbnail-click-check.log`
- `.trellis/tasks/10-17-presenter-notes-manual/research/verification.md`
- `.trellis/tasks/10-17-presenter-notes-manual/task.json`
- `assets/locales/en-US/app.toml`
- `assets/locales/zh-CN/app.toml`
- `crates/ferrostage-app/src/main.rs`
- `crates/ferrostage-core/src/command.rs`
- `crates/ferrostage-core/src/state.rs`
- `crates/ferrostage-core/src/state/lifecycle_tests.rs`
- `crates/ferrostage-core/src/state/mapping_tests.rs`
- `crates/ferrostage-core/src/state/property_tests.rs`
- `crates/ferrostage-core/src/state/tests.rs`
- `crates/ferrostage-render/src/protocol.rs`
- `crates/ferrostage-settings/src/schema.rs`
- `crates/ferrostage-settings/src/tests/parse.rs`
- `crates/ferrostage-ui/src/actions.rs`
- `crates/ferrostage-ui/src/presenter.rs`
- `crates/ferrostage-ui/src/presenter/workbench_view.rs`
- `crates/ferrostage-ui/src/settings_panel.rs`
- `crates/ferrostage-ui/src/shell.rs`
- `crates/ferrostage-ui/src/shell/preferences_tests.rs`
- `crates/ferrostage-ui/src/shell/reload.rs`
- `crates/ferrostage-ui/src/theme.rs`
- `crates/ferrostage-ui/src/transactions.rs`
- `crates/ferrostage-ui/src/workbench.rs`
- `crates/ferrostage-ui/src/workbench/tests.rs`
- `DESIGN.md`
- `docs/architecture.md`
- `docs/beamer-notes.md`
- `README.md`

## Validation

Default CI, final native build/lint and 123 GPUI tests, 40 native PDFium tests and the real perf-100 resource regression passed. Full-feature dependency audit fails on three unchanged unmaintained advisories. User waived further visual checks; they are not recorded as passing.
