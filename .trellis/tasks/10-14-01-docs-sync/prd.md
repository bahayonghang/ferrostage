# v0.3 文档同步：移除 LaTeX 编辑与批注写回描述

父任务：`10-14-presenter-v0.3`（R-01 R-02 R-03）。轻量任务，PRD-only。

## Goal

让 `AGENTS.md`、`docs/`、ADR 与 README 反映 v0.3 计划：产品只做 PDF/Beamer 展示，没有 LaTeX 工具链集成，没有 PDF 注释写回；新增会话生命周期与显示器角色两项架构决策，供后续子任务引用。

## Background

- `AGENTS.md:18` 写有 "Never truncate the user's PDF; write `*.ferrostage.tmp` and rename"，隐含 FerroStage 会写用户 PDF。v0.3 改为 FerroStage 永不写用户 PDF；原子写入规则只保留给设置文件。
- `docs/architecture.md` 的渲染优先级表（P1 neighbor / P2 presenter）与 `docs/PROJECT_PLAN.md` §4.6（P1 presenter / P2 ±1 prefetch）不一致。
- `docs/feature-parity.md` 把 Hot reload 列为 P0；v0.3 为 P2。
- `docs/beamer-notes.md` 只覆盖左右拼合；v0.3 增加上下拼合与手动覆盖。
- `docs/packaging.md`、`README.md`、`README.zh-CN.md` 需核对是否提到 LaTeX/批注编辑（当前 `rg` 未命中，但需复核 tagline 与 topics）。
- 既有 ADR：001 GPUI 来源、002 PDF 后端、003 观众快照、004 无锁重载、005 许可证门禁。

## Requirements

- **D-1** `AGENTS.md` Crate rules：改写热重载条目为"PDF 被外部替换时的通用重载；FerroStage 只以只读方式打开用户 PDF；设置文件走 `.tmp` + rename"；新增一条"观众窗口仅在 `Presenting` 阶段存在，`AudienceProjectionSnapshot` 含 `AudienceStage`"；新增一条"显示器以 `MonitorId` + `MonitorRole` 表示，不用索引"。
- **D-2** `docs/architecture.md`：加入 `SessionStage` 与显示器角色数据流；渲染优先级表与 `PROJECT_PLAN.md` §4.6 对齐；写明 FerroStage 不写用户 PDF。
- **D-3** `docs/feature-parity.md`：Hot reload 降 P2；新增行 "Session lifecycle (Ready → Start)" P0、"Monitor roles / three-screen" P0、"Settings system + UI" P0/P1；删除任何 annotation editing 行。
- **D-4** docs/beamer-notes.md同步父design §7的上下比例候选、局限、None/Left/Right/Top/Bottom覆盖、notes配对和导航。
- **D-5** 新增 `docs/adr/ADR-006-session-lifecycle.md`（Idle/Ready/Presenting；观众窗口存在性；计时器绑定开始）与 `docs/adr/ADR-007-monitor-roles.md`（MonitorId 生成策略、角色解析规则、热插拔降级、单 Audience 限制）。
- **D-6** `docs/adr/ADR-004-lock-free-reload.md`：删除 LaTeX 语境，改为通用"外部替换 PDF"；注明 FerroStage 自身不写 PDF。
- **D-7** README 双语：tagline 保留 "PDF & Beamer presentation stage"；GitHub topics 建议移除 `latex`（保留 `beamer`）；功能列表与 §1 一致。
- **D-8** 英文docs/spec继续英文，README保持双语，PROJECT_PLAN/当前任务文档沿用中文。同步backend/database-guidelines.md / directory-structure.md、frontend/state-management.md / quality-guidelines.md的PDF写回、独立swap flag、Settings全会话只读、Audience缺席自动回退旧条款，不改产品代码；纠正ADR-002中绑定default feature自动提供PDFium的旧假设，native库归05明确获取。

## Acceptance Criteria

- [ ] 禁用扫描每处命中记录path/line、历史/排除/現行分类与处置；历史/排除允许，现行正向能力禁止；保留约束，不以零匹配验收。
- [ ] `rg -n "ferrostage.tmp" AGENTS.md docs` 的每处命中都明确限定为设置文件或 FerroStage 自身产物，不再指向用户 PDF。
- [ ] `docs/adr/ADR-006-session-lifecycle.md` 与 `ADR-007-monitor-roles.md` 存在，Status 为 Proposed，且与 `10-14-presenter-v0.3/design.md` §2–§3 的类型名一致。
- [ ] `docs/architecture.md` 与 `docs/PROJECT_PLAN.md` §4.6 的优先级表逐行一致。
- [ ] git diff --check；产品代码未变，ADR/spec与父契约一致。仅文档任务不声称产品测试通过。

## Dependencies

无；最先实施，本次不执行。

## Out of Scope

产品代码；无关规范清理不在范围。涉及契约的spec文档同步包含在D-8。
