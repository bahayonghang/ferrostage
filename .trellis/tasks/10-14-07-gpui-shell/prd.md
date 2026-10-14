# GPUI 双窗口外壳、开始按钮与全局快捷键路由

父任务：`10-14-presenter-v0.3`（R-42 R-13 R-23）。复杂任务，设计和步骤见本目录design.md / implement.md。

## Goal

首次链接 GPUI，交付能跑的最小双窗口应用：讲者窗口可以打开 PDF、显示"开始演示/结束演示"按钮；点击开始后在 Audience 角色屏创建无边框全屏观众窗口；全局快捷键在任何窗口获焦时都生效且绝不在观众屏弹出 UI；Audience 屏拔线即降级。

## Background

- `ferrostage-ui` 只有 pin 常量与 `WindowRole`（`crates/ferrostage-ui/src/lib.rs`）；`ferrostage-app` 只有 `check-config`（`crates/ferrostage-app/src/main.rs`）。
- Pin：Zed tag `v1.23.3` 或 `gpui-kit`/`gpui-component` 0.7.3（`gpui-pre` 0.3.9）；不使用 crates.io `gpui` 0.2.x（`ADR-001`）。二选一是 research 必答项（git 依赖 vs crates.io 预发布）。
- 核心阶段与命令：`04-session-lifecycle`；显示器解析：`03-monitor-topology`；设置：`02-settings-schema`。
- 不变量 2、3、7（`docs/PROJECT_PLAN.md` §4.5）。
- `.trellis/spec/frontend/` 规定 UI crate 不拥有 deck，只发 `PresentationCommand`；观众视图只读快照。

## Requirements

- **G-1** research 结论写入 `research/gpui-display-api.md`：`cx.displays()` 返回的 `PlatformDisplay` 在 Windows 提供哪些字段（id/bounds/scale）；`WindowOptions { display_id, window_bounds: Fullscreen, titlebar: None }` 能否在指定屏创建无边框全屏；显示器变更事件如何接收；keystroke 字符串语法。
- **G-2** `ferrostage-app` 启动：加载设置 → 枚举显示器 → `resolve_roles` → 创建讲者窗口于 Presenter 屏（窗口化，`presenter_start_fullscreen` 时全屏）。CLI 位置参数 `<pdf>` 可选：给出则启动后执行 `LoadDocument` 进入 `Ready`，仍不创建观众窗口。
- **G-3** 讲者窗口最小 UI：标题栏显示文件名与阶段；"打开 PDF"（系统文件对话框，仅在讲者窗口弹出）；"开始演示"按钮（`Ready` 且 `audience_available` 时可用，否则禁用并显示原因）；"结束演示"按钮；当前页占位（真实渲染在 `08`）。
- **G-4** 唯一映射目标并协调创建窗口/Start同次主线程事务；失败回滚窗口与Ready，不启动计时。view只持snapshot Entity，Gate B Live占位，08补BGRA；windowed模式须显式启用且单屏。
- **G-5** 点击结束→StopPresentation→销毁观众。Escape先关闭当前模态/设置，再CancelJump并消费；无模态/预览时才执行stop绑定，不能Cancel后继续传播Stop。
- **G-6** 全局快捷键：按设置 `keybindings` 注册为 App 级 Action；无论焦点在哪个窗口，翻页/黑屏/交换/开始/结束均生效；任何会弹出 UI 的 Action（打开文件、设置、跳页输入）先把焦点激活到讲者窗口再执行。
- **G-7** 热插拔：订阅 `TopologyEvent`，`Presenting` 中 Audience 屏消失 → 销毁观众窗口 → `StopPresentation` → 讲者窗口若在该屏则移回 Presenter 屏并窗口化；新屏接入只刷新解析结果与按钮可用性。
- **G-8** SwapScreens通过唯一app协调器交换display.monitors两项→resolve→迁移，保存失败标未保存且保留会话值。禁止独立swap_roles/core flag决定角色，缺Audience/单屏拒绝交换。
- **G-9** 观众窗口在 `AudienceStage::Curtain` 时整窗纯黑；`Live` 时绘制占位色块（真实页面在 `08`）。
- **G-10** 每个 GPUI 调用收拢在 `ferrostage-ui`；`ferrostage-app` 只做组合；`ferrostage-core` 依赖树不变。
- **G-11** 引入gpui feature/app转发及显式native build/clippy CI编译门，不启动窗口；默认pure suite仍跑。05提供只读import metadata，07不以真实raster当Gate B条件。

## Acceptance Criteria

- [ ] 三屏实机：通过02配置文件指定A屏（09 UI尚未交付）；A屏 Audience、B 屏 Presenter、C 屏 None；启动 → 讲者窗口在 B；打开 PDF → 无观众窗口；点击开始 → 观众窗口只在 A 全屏，C 无变化；结束 → 观众窗口关闭。
- [ ] 把角色改为 C Audience 后重复上一条，观众窗口出现在 C。
- [ ] 单屏且 `presenter_only`：开始按钮禁用并提示"未找到观众显示器"；改为 `windowed_audience` 后可开始且出现窗口化观众窗口。
- [ ] 焦点点在观众窗口上按 `PageDown`/`B`/`Escape`：页面变化/黑屏/结束均生效，观众屏无任何对话框、焦点框或光标变化。
- [ ] `Presenting` 中拔掉 A 屏：观众窗口 ≤ 1 s 内消失，讲者窗口仍窗口化在 B，阶段回到 `Ready`。
- [ ] 观众窗口类型的字段中不存在 `LoadedDocument`、`SlideNotes` 或任何备注类型（代码评审 + `rg`）。
- [ ] PoC-1 结论写入 `research/poc-1-report.md`；`just ci` 与 `cargo build -p ferrostage-app` 全绿。

## Dependencies

完整前置：01/02/03/04/05（具体目录见父Task Map）。父子关联不表示依赖。

## Out of Scope

页面真实渲染、备注、计时器 UI（`08`）；设置界面（`09`，本任务只留一个打开设置窗口的入口占位）；媒体。
