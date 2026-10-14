# 演示会话生命周期：导入 → 就绪 → 点击开始演示

父任务：`10-14-presenter-v0.3`（R-10 R-11 R-12 R-13 R-14）。复杂任务，设计和步骤见本目录design.md / implement.md。

## Goal

打开 PDF 不再等于开始演讲。核心状态机新增阶段：`Idle`（无文档）→ `Ready`（已导入，可预览与调整）→ `Presenting`（讲者点击开始后才接管观众屏并启动计时）。观众窗口的存在与像素输出在类型层面与阶段绑定。

## Background

- `PresentationState::new(page_count)` 直接构造出落在第 0 页的可演讲状态（`crates/ferrostage-core/src/state.rs:37-52`），无阶段字段。
- `AudienceProjectionSnapshot`（`snapshot.rs:19-26`）无阶段信息，观众窗口无法区分"幕布"与"正在演示"。
- `PresentationCommand`（`command.rs:12-29`）没有加载/开始/结束/计时相关命令。
- 已有不变量测试 `audience_snapshot_tracks_generation_and_has_no_notes_field`（`state.rs:226-244`）需保留。
- `ferrostage-document::LoadedDocument::project` 调用 `state.audience_snapshot()`（`crates/ferrostage-document/src/lib.rs:47-52`），其测试会受快照字段变更影响。
- 共享契约见父任务 `design.md` §2；阶段表见 `docs/PROJECT_PLAN.md` §4.2。
- `ferrostage-core` 禁止依赖 GPUI / PDFium / lopdf / notify（`AGENTS.md`）。

## Requirements

- **L-1** 新增 `SessionStage { Idle, Ready, Presenting }` 与 `AudienceStage { Curtain, Live }`；`PresentationState::stage()` 可读。
- **L-2** 新增构造 `PresentationState::idle()`；`LoadDocument { generation, page_count }` 在 `Idle` 或 `Ready` 下进入 `Ready`，重置页位置、预览、批注、计时；`page_count == 0` 返回 `EmptyDocument`；在 `Presenting` 下返回 `NotReady`。
- **L-3** `StartPresentation { audience_available: bool }`：仅 `Ready` 可用；`audience_available == false` 返回 `NoAudienceDisplay`；成功后 `Presenting`，计时器启动。
- **L-4** `StopPresentation`：仅 `Presenting` 可用；回到 `Ready`；计时器停止但 `elapsed()` 保留；批注、当前页保留。
- **L-5** `CloseDocument`：任何阶段可用；回到 `Idle`；清空全部状态与计时。
- **L-6** 核心只接受app注入单调now，经apply_at/elapsed_at纯计算。Start每次清零；Pause/Resume仅Presenting；Reset清零保留pause；Stop冻结，Ready显示停止值；Close/Load清零，测试不用sleep。
- **L-7** snapshot新增stage与受控visual纯类型；非Presenting恒Curtain且visual=None。Live元数据只用已提交页，JumpPreview不影响观众。04定义core BgraFrame/AudienceVisual，08组装像素、11补媒体，core不依赖render/media。
- **L-8** 翻页、预览、黑屏、激光笔、批注命令在 `Ready` 与 `Presenting` 都接受；在 `Idle` 返回 `NotReady`。
- **L-9** 影子提交仅有文档时使用、不改变stage；04提供纯DocumentDescriptor入口，06补SlideMap，10最终替换bytes载荷并更新消费者，core不保留PDF bytes。
- **L-10** `PresentationCommand::name()` 覆盖全部新命令；`CoreError` 新增 `NotReady { stage }`、`NoAudienceDisplay`。
- **L-11** 更新document project及测试，协调01 ADR-003。SwapScreens仅app角色意图，移除screens_swapped权威；页/crop不变。预览中Next/Previous只移动preview，Commit/Cancel唯一出口，不定义MonitorRole/Id。

## Acceptance Criteria

- [ ] 状态转换表的每条合法边与每条非法边都有单测（含 `Presenting` 下 `LoadDocument` 被拒）。
- [ ] `idle()` 的快照 `stage == Curtain`；`LoadDocument` 后仍为 `Curtain`；`StartPresentation { true }` 后为 `Live`；`StopPresentation` 后回到 `Curtain`。
- [ ] `StartPresentation { audience_available: false }` 返回 `NoAudienceDisplay` 且阶段不变。
- [ ] Start5s→Pause5s→Resume2s→Stop=7s，Ready再5s仍7s；再Start=0，Reset保持pause，Close/Load=0。
- [ ] preview导航/Cancel观众不变、Commit才变；Idle/Ready visual=None；0尺寸/长度溢出帧被拒。
- [ ] proptest：任意命令序列下，`stage != Presenting ⇒ snapshot.stage == Curtain`（不变量 7）。
- [ ] 既有测试 `audience_snapshot_tracks_generation_and_has_no_notes_field` 与 `projecting_drops_speaker_notes` 仍通过。
- [ ] `ferrostage-core` 依赖树不含 GPUI / PDFium / lopdf / notify；`just ci` 全绿。

## Dependencies

完整前置：01（具体目录见父Task Map）。父子关联不表示依赖。

## Out of Scope

GPUI 窗口创建/销毁（`07`）；开始按钮与快捷键绑定（`07`/`08`）；显示器解析（`03`）。
