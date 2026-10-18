# 本次中文提交计划

用户已授权提交本次所有相关改动、归档三个任务并推送 origin/dev。

## 功能提交

本地 git commit -F；包含 Why 和 Agent-Task / Agent-Model / Generated-By 尾注，不添加 [AI] 标题。

```text
feat(演示): ✨ 优化跳页浏览与布局导航

Why: 原有跳页缺少明确输入框，文档选项会替换工作台，难以选页和即时查看布局效果。
统一页码输入与真实缩略图浏览，支持独立调整 1–4 行列并保留设置兼容性。
将文档选项改为向上展开的布局与导航菜单，勾选后立即应用并关闭。
补齐焦点、键盘、备注隔离、事务失败和渲染释放回归，同步设计规范与验收记录。
按用户要求忽略本地规划预览，保留测试报告和原生截图。

Tested: 默认 198 项、GPUI 159 项及 PDFium 回归通过；严格 lint、依赖审计和原生交互检查通过
Agent-Task: presenter-navigation-options
Agent-Model: gpt-6
Generated-By: agent
```

## 功能提交文件

- `.gitignore`
- `.trellis/spec/frontend/component-guidelines.md`
- `.trellis/tasks/10-18-layout-navigation-popover/check.jsonl`
- `.trellis/tasks/10-18-layout-navigation-popover/design.md`
- `.trellis/tasks/10-18-layout-navigation-popover/implement.jsonl`
- `.trellis/tasks/10-18-layout-navigation-popover/implement.md`
- `.trellis/tasks/10-18-layout-navigation-popover/prd.md`
- `.trellis/tasks/10-18-layout-navigation-popover/research/check-focus-after.log`
- `.trellis/tasks/10-18-layout-navigation-popover/research/check-focus-before.log`
- `.trellis/tasks/10-18-layout-navigation-popover/research/check-focus-isolated.log`
- `.trellis/tasks/10-18-layout-navigation-popover/research/check-review.md`
- `.trellis/tasks/10-18-layout-navigation-popover/research/check-shell-isolated.log`
- `.trellis/tasks/10-18-layout-navigation-popover/research/check-source-sha256.txt`
- `.trellis/tasks/10-18-layout-navigation-popover/research/implement-build.log`
- `.trellis/tasks/10-18-layout-navigation-popover/research/implement-clippy.log`
- `.trellis/tasks/10-18-layout-navigation-popover/research/implement-gpui-tests.log`
- `.trellis/tasks/10-18-layout-navigation-popover/research/implementation.md`
- `.trellis/tasks/10-18-layout-navigation-popover/research/native-acceptance.md`
- `.trellis/tasks/10-18-layout-navigation-popover/task.json`
- `.trellis/tasks/10-18-page-browser/check.jsonl`
- `.trellis/tasks/10-18-page-browser/design.md`
- `.trellis/tasks/10-18-page-browser/implement.jsonl`
- `.trellis/tasks/10-18-page-browser/implement.md`
- `.trellis/tasks/10-18-page-browser/prd.md`
- `.trellis/tasks/10-18-page-browser/research/check-browser-final.log`
- `.trellis/tasks/10-18-page-browser/research/check-ci-final.log`
- `.trellis/tasks/10-18-page-browser/research/check-ci-native-lock.log`
- `.trellis/tasks/10-18-page-browser/research/check-ci.log`
- `.trellis/tasks/10-18-page-browser/research/check-pdfium-final.log`
- `.trellis/tasks/10-18-page-browser/research/check-pdfium.log`
- `.trellis/tasks/10-18-page-browser/research/check-review.md`
- `.trellis/tasks/10-18-page-browser/research/check-shell-final.log`
- `.trellis/tasks/10-18-page-browser/research/check-shell.log`
- `.trellis/tasks/10-18-page-browser/research/check-source-sha256.txt`
- `.trellis/tasks/10-18-page-browser/research/implementation.md`
- `.trellis/tasks/10-18-page-browser/research/native-acceptance.md`
- `.trellis/tasks/10-18-page-browser/task.json`
- `.trellis/tasks/10-18-presenter-navigation-options/check.jsonl`
- `.trellis/tasks/10-18-presenter-navigation-options/design.md`
- `.trellis/tasks/10-18-presenter-navigation-options/implement.jsonl`
- `.trellis/tasks/10-18-presenter-navigation-options/implement.md`
- `.trellis/tasks/10-18-presenter-navigation-options/prd.md`
- `.trellis/tasks/10-18-presenter-navigation-options/research/commit-plan.md`
- `.trellis/tasks/10-18-presenter-navigation-options/research/integration-check.md`
- `.trellis/tasks/10-18-presenter-navigation-options/research/integration-ci-final.log`
- `.trellis/tasks/10-18-presenter-navigation-options/research/integration-ci.log`
- `.trellis/tasks/10-18-presenter-navigation-options/research/integration-deny-all-features.log`
- `.trellis/tasks/10-18-presenter-navigation-options/research/integration-pdfium-final.log`
- `.trellis/tasks/10-18-presenter-navigation-options/research/integration-shell-final.log`
- `.trellis/tasks/10-18-presenter-navigation-options/research/integration-shell.log`
- `.trellis/tasks/10-18-presenter-navigation-options/research/integration-ui-clippy-final.log`
- `.trellis/tasks/10-18-presenter-navigation-options/research/integration-ui-clippy.log`
- `.trellis/tasks/10-18-presenter-navigation-options/research/native/baseline-input-identities.json`
- `.trellis/tasks/10-18-presenter-navigation-options/research/native/integration-dark-auto-restored.jpg`
- `.trellis/tasks/10-18-presenter-navigation-options/research/native/integration-dark-layout-menu.jpg`
- `.trellis/tasks/10-18-presenter-navigation-options/research/native/integration-dark-physical-checked.jpg`
- `.trellis/tasks/10-18-presenter-navigation-options/research/native/integration-dark-stderr.log`
- `.trellis/tasks/10-18-presenter-navigation-options/research/native/integration-dark-stdout.log`
- `.trellis/tasks/10-18-presenter-navigation-options/research/native/integration-dark-unsplit-applied.jpg`
- `.trellis/tasks/10-18-presenter-navigation-options/research/native/integration-dark-unsplit-checked.jpg`
- `.trellis/tasks/10-18-presenter-navigation-options/research/native/integration-dark.pid`
- `.trellis/tasks/10-18-presenter-navigation-options/research/native/integration-final-identities.json`
- `.trellis/tasks/10-18-presenter-navigation-options/research/native/integration-final-reviewed-identities.json`
- `.trellis/tasks/10-18-presenter-navigation-options/research/native/integration-light-keyboard-toggle.jpg`
- `.trellis/tasks/10-18-presenter-navigation-options/research/native/integration-light-layout-menu.jpg`
- `.trellis/tasks/10-18-presenter-navigation-options/research/native/integration-light-private-notes-rejected.jpg`
- `.trellis/tasks/10-18-presenter-navigation-options/research/native/integration-light-stderr.log`
- `.trellis/tasks/10-18-presenter-navigation-options/research/native/integration-light-stdout.log`
- `.trellis/tasks/10-18-presenter-navigation-options/research/native/integration-light.pid`
- `.trellis/tasks/10-18-presenter-navigation-options/research/native/page-browser-before-native-polish.jpg`
- `.trellis/tasks/10-18-presenter-navigation-options/research/native/page-browser-edit-69-no-jump.jpg`
- `.trellis/tasks/10-18-presenter-navigation-options/research/native/page-browser-final-2x3.jpg`
- `.trellis/tasks/10-18-presenter-navigation-options/research/native/page-browser-final-4x4.jpg`
- `.trellis/tasks/10-18-presenter-navigation-options/research/native/page-browser-final-identities.json`
- `.trellis/tasks/10-18-presenter-navigation-options/research/native/page-browser-final-stderr.log`
- `.trellis/tasks/10-18-presenter-navigation-options/research/native/page-browser-final-stdout.log`
- `.trellis/tasks/10-18-presenter-navigation-options/research/native/page-browser-final-thumbnail-jump.jpg`
- `.trellis/tasks/10-18-presenter-navigation-options/research/native/page-browser-jump-69.jpg`
- `.trellis/tasks/10-18-presenter-navigation-options/research/native/page-browser-source-identities.json`
- `.trellis/tasks/10-18-presenter-navigation-options/research/native/page-browser-stderr.log`
- `.trellis/tasks/10-18-presenter-navigation-options/research/native/page-browser-stdout.log`
- `.trellis/tasks/10-18-presenter-navigation-options/research/native/page-browser-visible-stderr.log`
- `.trellis/tasks/10-18-presenter-navigation-options/research/native/page-browser-visible-stdout.log`
- `.trellis/tasks/10-18-presenter-navigation-options/research/native/page-browser.pid`
- `.trellis/tasks/10-18-presenter-navigation-options/research/native/smoke-light-en.toml`
- `.trellis/tasks/10-18-presenter-navigation-options/research/native/smoke-settings.toml`
- `.trellis/tasks/10-18-presenter-navigation-options/research/planning-verification.md`
- `.trellis/tasks/10-18-presenter-navigation-options/research/precommit-review.md`
- `.trellis/tasks/10-18-presenter-navigation-options/research/source-evidence.md`
- `.trellis/tasks/10-18-presenter-navigation-options/research/verification.md`
- `.trellis/tasks/10-18-presenter-navigation-options/task.json`
- `assets/locales/en-US/app.toml`
- `assets/locales/zh-CN/app.toml`
- `crates/ferrostage-core/src/state.rs`
- `crates/ferrostage-core/src/state/annotation_tests.rs`
- `crates/ferrostage-ui/src/actions.rs`
- `crates/ferrostage-ui/src/lib.rs`
- `crates/ferrostage-ui/src/page_browser.rs`
- `crates/ferrostage-ui/src/presenter.rs`
- `crates/ferrostage-ui/src/presenter/layout_navigation.rs`
- `crates/ferrostage-ui/src/presenter/number_input.rs`
- `crates/ferrostage-ui/src/presenter/page_browser.rs`
- `crates/ferrostage-ui/src/presenter/workbench_view.rs`
- `crates/ferrostage-ui/src/shell.rs`
- `crates/ferrostage-ui/src/shell/layout_navigation.rs`
- `crates/ferrostage-ui/src/shell/page_browser.rs`
- `crates/ferrostage-ui/src/shell/reload.rs`
- `crates/ferrostage-ui/src/workbench.rs`
- `crates/ferrostage-ui/src/workbench/tests.rs`
- `DESIGN.md`

## 后续记账提交

1. chore(任务): 🔧 归档跳页浏览与布局导航任务
   归档两个子任务和父任务，修正移动后的上下文路径，记录功能提交哈希。
2. chore: 🔧 记录跳页与布局导航实施会话
   记录功能提交、检查结果及归档情况。

最后普通推送 dev 到 origin/dev，不重写历史。

## 明确排除

- interaction-preview.html：用户要求加入 gitignore，保留本地，覆盖活动与归档路径。
