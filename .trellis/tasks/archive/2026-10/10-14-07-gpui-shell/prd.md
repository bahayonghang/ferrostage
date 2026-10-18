# GPUI 双窗口外壳、开始按钮与全局快捷键路由

> Current acceptance policy (2026-10-15 user revision): `.trellis/tasks/10-14-presenter-v0.3/acceptance-policy.md` supersedes older hardware/matrix blockers below. Implement all features first; retain automatic/feature/native-functional checks. Physical, exhaustive native/size and performance/soak matrices are deferred, not PASS. Gate B implementation is satisfied by V16;08 may start. Gate C/D are functional integration gates. Historical results and product contracts remain unchanged.

父任务：`10-14-presenter-v0.3`（R-42 R-13 R-23 R-24 R-33）。复杂任务，设计和步骤见本目录design.md / implement.md。

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
- **G-3** 讲者窗口最小 UI：标题栏显示文件名与阶段；"打开 PDF"（系统文件对话框，仅在讲者窗口弹出）；"开始演示"按钮（`Ready` 且 `audience_available` 时可用，否则禁用并显示原因）；"结束演示"按钮；“显示器设置”入口（替代工具栏“交换屏幕”，见G-13）；当前页占位（真实渲染在 `08`）。
- **G-4** 唯一映射目标，由同一主线程协调器跨异步回调完成创建/实际放置及模式确认/Start事务；失败回滚窗口与Ready，不启动计时。view只持snapshot Entity，Gate B Live占位，08补BGRA；windowed模式须显式启用且单屏。
- **G-5** 点击结束→StopPresentation→销毁观众。Escape先关闭当前模态/设置，再CancelJump并消费；无模态/预览时才执行stop绑定，不能Cancel后继续传播Stop。
- **G-6** 全局快捷键：按设置 `keybindings` 注册为 App 级 Action；无论焦点在哪个窗口，翻页/黑屏/交换/开始/结束均生效；任何会弹出 UI 的 Action（打开文件、设置、跳页输入）先请求讲者窗口激活，确认当前讲者确已active后再执行；调用返回不等于已激活。
- **G-7** 热插拔：订阅 `TopologyEvent`，`Presenting` 中 Audience 屏消失或枚举 `Unavailable` → 发布 `Curtain` → `StopPresentation` → 销毁观众窗口 → 讲者窗口若在失效屏则移回有效 Presenter 屏并窗口化；零屏等待恢复。新屏接入或枚举恢复只刷新解析结果与按钮可用性，不自动开始。
- **G-8** 屏幕分配通过唯一app协调器一次应用应用屏/PDF屏角色草稿→resolve→迁移→保存；失败回滚运行时，保存失败保留会话值并标未保存/重试。既有SwapScreens快捷键保留兼容并复用同一事务，不再提供主要工具栏交换按钮。禁止独立swap_roles/core flag决定角色，缺Audience/单屏拒绝交换。
- **G-9** 观众窗口在 `AudienceStage::Curtain` 时整窗纯黑；`Live` 时绘制占位色块（真实页面在 `08`）。
- **G-10** 每个 GPUI 调用收拢在 `ferrostage-ui`；`ferrostage-app` 只做组合；`ferrostage-core` 依赖树不变。
- **G-11** 引入gpui feature/app转发及显式native build/clippy CI编译门，不启动窗口；默认pure suite仍跑。05提供只读import metadata，07不以真实raster当Gate B条件。

- **G-12** 遵循根 DESIGN.md（R-45）：统一Claude风格token、浅/深配色、系统字体、完整控件状态与键盘焦点；原生截图/对比度/尺寸验证记录本任务research。

- **G-13 显示器设置**：打开讲者侧面板先读取当前完整拓扑，显示“检测到 N 个屏幕”、编号及名称/分辨率/主屏标记/当前角色；“识别屏幕”显示与列表同号的3s标记，仅Idle/Ready可用。分别选择“应用窗口所在屏幕”和“PDF播放屏幕”，其余不使用；Apply前无窗口或配置变化，Cancel不写入。多屏同屏角色冲突明确拒绝，单屏PDF选择明确区分“不播放”（presenter_only）和“在本屏窗口播放”（windowed_audience），Apply同时更新single_monitor_mode，不创建重复物理角色；“不播放”和多屏指定物理PDF屏均设置presenter_only，只有显式本屏窗口播放设置windowed_audience；无屏/枚举失败/目标离线/未来版本均有准确状态。编号仅作显示，持久化使用MonitorId。拓扑变化刷新列表并使旧草稿失效；Apply重验目标、复用G-4/G-8确认事务，有效迁移保持页/阶段/计时；Presenting改为“不播放”则Curtain→Stop→关观众，保留页/冻结计时回Ready，保存失败只显示一个未保存/Retry状态。

## Acceptance Criteria

- [x] 三屏实机：通过02配置文件指定A屏（09 UI尚未交付）；A屏 Audience、B 屏 Presenter、C 屏 None；启动 → 讲者窗口在 B；打开 PDF → 无观众窗口；点击开始 → 观众窗口只在 A 全屏，C 无变化；结束 → 观众窗口关闭。
- [x] 把角色改为 C Audience 后重复上一条，观众窗口出现在 C。
- [x] 观众窗口类型的字段中不存在 `LoadedDocument`、`SlideNotes` 或任何备注类型（代码评审 + `rg`）。


2026-10-15 V6验收索引：A Audience/B Presenter完整放置链对应 `research/poc-1-report.md`、`research/native/v4-a-cli.stdout.log`（Ready audience=None、Presenter65731、Audience131073全屏）和 `research/native/v4-a-audience-page1.json`；C Audience对应 `research/native-v5-checkpoint.md` 与 `research/native/v5-c-vertical.stdout.log`。这些是各自冻结版本的实际观测，不标成V6同次实测；跨版本源码/回归适用性见 `research/product-independent-review.md` 与 `research/product-independent-v6-review.md`，only-snapshot隔离由当前独立代码评阅核对。`research/native-v6-checkpoint.md` 补焦点、保存重试、原生选择器和重启。PageDown/B/Escape行为已观测，但观众光标完整矩阵未验，不勾组合条目；三屏配置禁用Audience不算单屏实机。自动CI/编译已通过，PoC-1整体实机结论仍不完整，保留该组合条目未勾；Gate B未通过。

- [x] 工具栏显示“显示器设置”而非直接“交换屏幕”；点击只打开面板。先列当前屏幕数量和编号，三屏Identify与列表一致，再明确选择应用屏和PDF屏，第三屏不使用。
- [x] 三屏分别选择B应用/A播放及B应用/C播放，通过Apply验证真实放置；Idle/Ready不自动开始，重启按MonitorId恢复。旧配置/Swap截图不能替代此项。

2026-10-15 V11增量验收索引：前两项新UI勾选对应V9 `research/native-v9-checkpoint.md` 的面板/草稿及三次单独Identify1–3截图，以及V11 `research/native-v11-checkpoint.md` 的新面板B/A→B/C Apply、Ready无观众、Start实际目标和B/C保存重启。不同冻结版本如实标注；不以旧Swap/配置证据替代。V11已验证迁移/不播放计时、实际保存锁Retry、未来schema只读和部分主题/尺寸，但组合条目包含的单屏/完整尺寸/状态仍待验，保持未勾。V11深色窄窗Close出现F11 dispatcher错误，Gate B仍INCOMPLETE；修复及后续版本需独立复核证据适用性。

## Dependencies

完整前置：01/02/03/04/05（具体目录见父Task Map）。父子关联不表示依赖。

## Out of Scope

页面真实渲染、备注、计时器 UI（`08`）；完整设置分类/布局/计时/指针等界面（`09`）；媒体。最小屏幕分配面板与Identify属于07，09复用；普通设置入口仍可明确标为占位。


2026-10-15 V13增量索引：research/native-v13-checkpoint.md封存五run/26截图。C初始完整标题栏、精确light1440/800/390/320及dark320、英文新面板、实际chooser成功和双窗口关闭有版本限定证据；第五light320关闭F12 FAIL，故不勾整体PoC/设计/关闭组合项。其余dark尺寸和可行状态检查在V14修复后继续；物理NOT_RUN不变。

## Implementation acceptance under the 2026-10-15 user revision

- [x] Required implementation and automatic/feature checks are complete; current applicability independently audited in task07 research/resume-2026-10-15-v16-review.md.
- [x] V16 bounded native core verifies roles/Identify/Apply/Cancel/conflict, actual placement, navigation/modal routing, migration/no-play/timer/persistence and clean owner/Audience closure. Real PDF/notes UI remains08 work, not07 acceptance.

## Deferred full-qualification criteria (not current archive blockers)

The following original composite criteria remain unaccepted as a whole. Their verified functional subsets are documented above; remaining physical/native/matrix portions are DEFERRED_BY_USER. No unchecked criterion is converted to PASS.

- DEFERRED_BY_USER (original full criterion): 单屏且 `presenter_only`：开始按钮禁用并提示"未找到观众显示器"；改为 `windowed_audience` 后可开始且出现窗口化观众窗口。
- DEFERRED_BY_USER (original full criterion): 焦点点在观众窗口上按 `PageDown`/`B`/`Escape`：页面变化/黑屏/结束均生效，观众屏无任何对话框、焦点框或光标变化。
- DEFERRED_BY_USER (original full criterion): `Presenting` 中拔掉 A 屏：观众窗口 ≤ 1 s 内消失，讲者窗口仍窗口化在 B，阶段回到 `Ready`。
- DEFERRED_BY_USER (original full criterion): PoC-1 结论写入 `research/poc-1-report.md`；`just ci` 与 `cargo build -p ferrostage-app --features gpui,pdfium` 全绿；默认无feature构建不替代GPUI编译门。
- DEFERRED_BY_USER (original full criterion): 本任务界面在DESIGN.md各窗口尺寸及双主题下完成原生截图、键盘/焦点、hover/disabled/error/loading、字体回退和对比度检查；未执行不勾选。
- DEFERRED_BY_USER (original full criterion): Cancel不改配置/窗口；冲突/离线/枚举失败/旧草稿不可应用；有效目标迁移保持页/阶段/计时，Presenting选“不播放”安全停止，单屏两种选项正确控制Start并在重启后恢复；运行时失败回滚、保存失败显示未保存/Retry，未来版本不写入。
- DEFERRED_BY_USER (original full criterion): 共享面板键盘/焦点/双主题/窄窗/真实角色操作通过；Identify在Presenting禁用，观众屏无设置UI和备注。
