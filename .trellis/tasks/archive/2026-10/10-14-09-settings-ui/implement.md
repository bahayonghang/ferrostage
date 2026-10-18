# Implement — 09

## Current checkpoint — 2026-10-15

I-1–I-5 and I-7 implementation/independent automatic gates are complete for U-1–U-11. Review fixed closed-panel layout writes during pending settings application/rollback and verified108 GPUI/real-loader,37 native and162 default CI tests, strict feature/production lint/typecheck, all-feature deny and post-CI build. See `research/independent-review.md` and `review-build-identity.json`. I-6 bounded desktop smoke follows the user's explicit consolidated schedule after11; extended matrices stay DEFERRED_BY_USER and original composite ACs unchecked.08/09 remain unarchived, and execution has proceeded to existing10. No commit/push/archive.

> Current acceptance policy (2026-10-15 user revision): `.trellis/tasks/10-14-presenter-v0.3/acceptance-policy.md` supersedes older hardware/matrix blockers below. Implement all features first; retain automatic/feature/native-functional checks. Physical, exhaustive native/size and performance/soak matrices are deferred, not PASS. Gate B implementation is satisfied by V16;08 may start. Gate C/D are functional integration gates. Historical results and product contracts remain unchanged.

1. **I-1 消费者评审**：核02/03/07/08接口与上表，语言restart、theme/system事件明确；hot_reload/media capability未就绪禁用，不做循环依赖。
2. **I-2 界面骨架**：讲者内分类、WritePolicy只读、未保存/重试，先测试focus/disabled/SaveError，不接audience树。
3. **I-3 复用角色面板**：先核07数量/编号/两项选择/Apply/Cancel/Identify交付；嵌入显示器页并补拓扑图/稳定等级/warnings/single mode，编号、事务和Identify不重复实现，迁移失败回滚。
4. **I-4 实际设置生效**：布局/grid两键/计时/指针/cache/debug/快捷键/recent；值改变→实际consumer呈现→save→restart一致，非法类型与冲突呈现准确。
5. **I-5 语言/主题/版本**：语言restart真正选择现有locales；dark/light/system配色和系统改变；future只读禁止save/reset，save失败dirty与retry可见。媒体未实现不渲染空control。
6. **I-6 三屏UI实机**：任意选Audience/Presenter/None/Identify→Start真实PDF；Presenting迁移≤500ms、拔线≤1s、保存重启/共享面板binding一致及旧交换快捷键兼容，焦点观众打开settings只在讲者；AC-1/2最终UI路径证据。
7. **I-7 评审/门**：mapping测试+实际效果人工/自动验证，保存失败/role失败回滚，feature-parity只标已验；run命令，Gate D settings部分记录。

命令：cargo test -p ferrostage-settings -p ferrostage-ui -p ferrostage-core；cargo build -p ferrostage-app --features gpui,pdfium；just ci。
回滚：I-3role事务失败原窗口/binding，I-4save失败旧文件保留；字段/资源接线回滚同步ui/app/settings。10/11后续补相应capability实际验收，本任务不称其功能已完成。

## 主题执行补充（2026-10-14）

在I-2/I-5/I-7中先核DESIGN.md与07主题层所有权，实施对应token/布局/设置消费者后保存原生截图、对比度结果与键盘/状态检查。无运行窗口时记NOT_RUN，保持原有Gate依赖。

## 2026-10-14 依赖审计 checkpoint

07外壳和08工作台均未交付；真实设置消费者前置未满足。 本任务保持planning，未start、未写本任务产品代码、未运行本任务验收。现有规划保留；恢复前先核父execution-state.md与07依赖门报告。03物理拔插/换接口本轮不可用，不能用模拟输入替代Gate B。无提交或归档。


## Resume audit - 2026-10-15

Task07 minimal display panel is implemented; it is not the complete09 settings UI. I-1 remains blocked by08 delivery. Do not fork the shared role/Identify transaction or start planning-status product code. See parent `research/resume-2026-10-15.md` and07 `research/resume-2026-10-15-v16-review.md` for current-code evidence and exact remaining gates. Task status and acceptance checkboxes are unchanged. No commit, push or archive.


## Implementation checkpoint — 2026-10-15 (09 implementer)

I-1–I-5 implemented: seven categories with real consumers and transactional application; shared07 role/Identify path; actual raw-key interception and full keymap replacement; runtime CPU/L3 acknowledgement; dual-window GPU preflight and snapshot-only budget application; localized dirty/retry/read-only state. I-6 remains DEFERRED_BY_USER/NOT_RUN per consolidated smoke instruction. I-7 automatic evidence: 107 GPUI/native-loader tests passed; 37 native PDF/document/render tests passed; strict all-feature and production feature clippy passed; final just ci (162 tests + deny) and post-CI gpui,pdfium build passed. Independent check is pending. No commit/push/archive. Raw composite native ACs remain unchecked. See research/implementation-report.md.
