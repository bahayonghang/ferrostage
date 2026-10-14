# Research: 实施规划执行一致性评审

- Query: 对更新后的父级 prd/design/implement 与 02–11 的同类文档做聚焦检查，只列可能导致实施、独立验收或集成失败的可操作问题。
- Scope: internal；只读规划，唯一新增文件为本报告。
- Date: 2026-10-14
- Snapshot: 下列位置是本轮读取时的行号；主会话仍在修改规划，修正后可逐项标记处理。未把 PROJECT_PLAN、JSONL、traceability 的暂未完成部分作为问题。

## Findings

### 复核处置状态

| 项目 | 本轮复核状态 |
| --- | --- |
| PR-01 | 已解决：07 G-5 与 design:14 已明确模态/preview 优先、消费 Esc，普通状态才 Stop。 |
| PR-02 | 已解决：父 design:36–38 与 03 design:13 已加入 DuplicateBinding、排除实际 Presenter、Presenter 回退排除 Audience、单屏 audience=None；09 design:28 / I-3 已同时清旧 Audience 与旧 Presenter。 |
| PR-03 | 已解决：08 I-5 已前移基本 Begin/Append/End 会话画笔，I-6 只留高级增强，M1 画笔 AC 有实际实施步骤。 |
| PR-04 | 已解决：07 design:12 用可替换 Shell DocumentImporter/物理 descriptor；06 design:15 保留 page_count→identity SlideMap 的纯兼容入口供独立 Shell PoC；08 design:5、文件范围:8 与 I-2 明确 app 初次真实 05 metadata/结构→06 builder→Load 显式 map/crop→Ready，坏 build 保旧，不用 identity 默认冒充 Beamer 结果。 |
| PR-05 | 已解决：08 W-11、design:12 / I-4 明确当前文档五 layout、独立 notes、physical_pages、diagnostic 与 core ApplyDocumentView；同代保留 stage/timer，帧齐备完整发布。06 仍规定新 Load 重置/同文件重载保留覆盖。 |
| PR-06 | 已解决：03 M-5、design:14 / I-5 独立负责安全 OS 事件源，接口不足可选明确的 200ms 安全枚举轮询；07 只补触发，不再是 03 交付前置。实际 ≤1s 仍待未来硬件验收。 |
| PR-07 | 已解决：父 design:73 与 11 V-3、design:17 / I-2 均将 playback_epoch 加入私有 SharedFrame；观众仍只得 slot/rect/pixels。 |
| PR-08 | 已解决：05 P-4 / design:11 / I-2、10 H-3（prd:21）/ design:13 / I-3 均为 Prepare 仅 load/metadata→06 建图/定位/crop→PreRender 明确 P0/P1 请求→ready，需求、协议、设计和步骤已同步。 |
| PR-09 | 已解决：父 design:87、10 H-5 / design:14 与 06 design:15 明确定义先钳物理范围，再取不大于该号的最大 content 物理页，无则首 content；共享 closest_content_at_or_before，不能作 content Vec 下标。 |

计时补充复核：父 design:26、08 design:15 已统一 Ready/Presenting 使用同一 timer.mode 格式，冻结 elapsed，count_down 初次显示 target、Stop 后显示冻结 remaining；不再报 Ready 读数格式问题。

本轮新增所有权复核：general.recent_files=[] 已由 02 S-1 / design:5 与父 design:46 创建，07 维护，09 design:25 消费/清空/限制；不存在 07 先于 09 使用尚未引入字段的问题。PR-01…09 全部完成规划复核闭合，未发现新的实质性规划阻塞。以下各节保留首次发现作为处置依据，不能据旧位置误报当前仍有同样冲突；“已解决”只表示规划文字闭合，不表示实现或实测通过。

### PR-01 — Esc 的两项验收相互冲突（07/08）

- 来源：07 `prd.md:23` 的 G-5 指定 `Escape → StopPresentation`；08 `prd.md:24` 的 W-7 及 `design.md:15` 指定预览中 Esc 仅 CancelJump；07 `design.md:14` 的全局 dispatcher 尚未说明上下文优先级。
- 失败场景：Presenting 中打开跳页输入后按 Esc，按 07 的规则会结束演示，按 08 的规则则必须仅取消预览。两个实施者无法同时满足现文字。
- 修正归属：07 定义动作路由优先级，08 消费并测试：录制/模态/跳页输入先处理 Esc 并消费事件；仅没有这些上下文时，Esc 才 Stop。Ready 无模态时的 Esc 也需明确（例如不调用非法 Stop）。
- 验证：Presenting+preview 的 Esc 保持阶段/观众页/计时，只清 preview；普通 Presenting 的 Esc 结束；两次 Esc 分别取消再结束，不触发两个动作。

### PR-02 — 一个物理屏可同时解析成 Presenter/Audience（03/07/09）

- 来源：父 `design.md:38` 无主屏标记时以 ID 首屏作 Presenter，但 Audience 仍选“最大面积非主屏”；同文档 `36–40` 与 03 `design.md:12–13` 只定义同类角色重复/物理 ID 碰撞，未定义同一已保存 ID 被不同 binding 配两种角色。09 `implement.md:5` 只显式清除旧 Audience。
- 失败场景 A：两屏都 `primary=false`，ID 首屏也是面积最大屏，它会被两套选择规则同时选中。失败场景 B：设置有 `{id:X,role:Presenter}` 与 `{id:X,role:Audience}`，两类角色都各只有一项，不触发 DuplicateAudience/Presenter，但指向同一物理屏。
- 修正归属：03 先明确按 MonitorId 仲裁重复 binding，保证每个物理屏恰好一个角色；Audience 候选必须排除**实际解析出的 Presenter**，不能只过滤 primary。增加 ConflictingMonitorBinding 或明确复用某 warning。07 Start 必须再次校验两目标不同；单屏窗口化是显式特例。09 选择新 Presenter 也应同事务清除旧 Presenter，遵守“UI 不产生重复”。
- 验证：无 primary 的两/三屏、同 ID 异角色 binding、重复 Presenter 下改选新屏；普通模式必有 `presenter != audience`，UI 显示角色与实际迁移一致。

### PR-03 — Gate C 跳过 P1 实现却仍要求画笔通过（父/08）

- 来源：父 `implement.md:27` 将 Gate C 定为 08 I-1…I-5/I-7；08 `implement.md:8` 的 I-6 才加入 Begin/End/Erase/highlighter；08 `prd.md:31,33` 仍要求 30min 汇报画笔正常及同笔画截图，`design.md:21` 的 I-7 性能协议仍有笔画偏差测试。
- 失败场景：按计划只做 M1 P0、暂不做 I-6，执行 I-7 和 PRD AC 时必因没有相应画笔能力失败；与父 M1 允许 P1 留到 M2 的定义冲突。
- 修正归属：父/08 将 AC 和 I-7 拆为 M1 P0 子集与 M2 P1 子集；画笔/荧光笔相关汇报和截图归 M2，或明确将最小 Begin/End 笔画提前纳入 I-5。Gate C 引用精确子集，完整 08 验收仍在 M2。
- 验证：只交付 I-1…I-5 时 Gate C 清单没有尚未创建的画笔/网格操作；P1 的截图/笔画验证仍有 M2 所有者，没有被删掉。

### PR-04 — 初次导入未明确将 06 builder 接入 Load 事务（06/07/08）

- 来源：07 `design.md:12`、`implement.md:5` 的导入是 `05 probe → Load → Ready`，且 07 无 06 依赖；06 `implement.md:7` 将 descriptor 扩为 SlideMap，但其 `design.md:8–9` 不拥有 app/UI 初次导入；08 `design.md:5` 假设收到完整 private LoadedDocument，`8–9` 的范围不含 app 导入协调器，I-2 只说消费裁剪/备注。
- 失败场景：07 的占位导入进入 Ready 只有页数/probe；08 加上实际画面后，初次 Load 没有明确步骤构建/安装 SlideMap、NotesMapping、媒体结构和私有 LoadedDocument。只有 10 的重载流程完整调用 06 builder，可能出现“首次导入不识别、重载才识别”。06 先后实施还会改动 07 所用 descriptor 的兼容接口。
- 修正归属：保持 07 不依赖 06 的外壳目的，明确 07 使用可兼容的全物理页默认 descriptor；08 负责升级 app/UI 初次导入流水线：05 owned metadata + pdf 只读结构 → 06 builder → 全部校验通过 → 同次安装 document/core descriptor → Ready。将 app 协调器和必要 Cargo 消费接线加入 08 文件范围/步骤；06 descriptor 扩展保持旧消费方式可编译。
- 验证：不发生重载的第一次打开夹具就有正确 crops/overlay/多 notes；初次 builder 失败旧文档与 core 不变；07 在只有其明确前置时仍可独立构建/验收。

### PR-05 — 手动覆盖只有模型能力，没有明确 UI 交付者（06/08）

- 来源：06 `design.md:12–14` 要求布局覆盖、独立 notes 识别开关和 physical_pages 分组覆盖；父 `design.md:81–83` 要求 UI 显示检测来源。06 的文件范围是模型/结构，07 不做 Beamer UI，08 `prd.md:18–27`、`implement.md:4–8` 未明确这些当前文档控件与重新建图流程。
- 失败场景：用户打开被启发式误判的宽图或重标签 PDF 后，模型支持 override，但实际界面无法选择 None/关闭分组/恢复误判 notes 页，R-41 的手动覆盖不能验收。
- 修正归属：08 增加讲者端“当前文档”布局/独立备注/分组控制、检测来源与诊断展示，使用 06 builder 重建当前图并恢复合法内容位置；定义 Presenting 中布局变化的帧提交方式，沿用完整 snapshot 发布。它不是 09 的全局 settings 控件；同文件重载保留、新 Load 重置。
- 验证：从 UI 将宽图候选改 None、将相邻重标签改 physical_pages、关闭 notes 识别恢复普通标签页；首次 Load 和同文件重载各验证 override 生命周期。

### PR-06 — 03 的事件源交付依赖下游 07 的决定（03/07）

- 来源：03 `prd.md:22,31` 必须交付 watch_topology 和实机 ≤1s 事件；03 `design.md:14` 明确“07 决定安全事件源接线”，03 禁止创建 GPUI/隐藏监听窗口；07 又把 03 列为完整前置并在 `design.md:15` 依赖其 owned 事件。
- 失败场景：按 03 的独立完成条件，先要有真正 OS 事件输入；但事件来源与桥接在尚不能开始的 07 决定，Gate A 的纯测试不能代替 03 watch 实机 AC。
- 修正归属：二选一落实到文档：03 研究并锁定不依赖 GPUI 的安全事件源，独立交付/验收，07 仅订阅；或者 03 只交付 `topology_changed_signal → re-enumerate/diff` 的纯订阅处理器，07 唯一负责真实事件源/桥接与 ≤1s 实机 AC，并相应调整 03 完成门和父 Gate A/B。不能同时保留现在两方等待的归属。
- 验证：03 的全部独立 AC 在不依赖 07 的状态下有执行路径；07 事件桥有明确 API 输入，依赖图不因事件源出现隐含反向边。

### PR-07 — SharedFrame 缺少组装器要求筛选的 playback epoch（11）

- 来源：父 `design.md:73` 与 11 `prd.md:20`、`design.md:17` 定义输出包身份为 generation/page/slot/sequence，但同一行要求组装器筛 playback epoch；11 I-2 又承诺旧 epoch 帧失效。
- 失败场景：同一 generation/page/slot 黑屏停止后显式恢复，上一 decoder 的迟到帧与新播放具有相同三个身份字段，sequence 也可能重新从零开始。没有 epoch 的结果或固定 epoch 的通道身份，组装器无法按设计拒绝旧播放帧。
- 修正归属：11 将 PlaybackEpoch 加入私有 SharedFrame/result envelope，或明确每个结果通道携带不可变 epoch；所有控制/帧消息比较相同字段。AudienceMediaFrame 仍只保留 slot/rect/pixels，不能为了过滤把控制包给观众。
- 验证：同页、同代、同槽位的新播放开始后收到旧 epoch 的有效 BGRA，旧帧被拒；不会因 sequence 数值较大覆盖新实例。

### PR-08 — candidate 元数据、建图、定位和预渲染的先后接口尚未收敛（05/06/10）

- 来源：父 `design.md:85` 的顺序为 candidate 解析→06 建图→预渲染，10 `design.md:13` 却将 PrepareCandidate 一并描述为 load/labels/dimensions/P0/P1 预渲染，再由 06 合并；10 `14` 的当前页定位同样需要新标签/图。05 的 Prepare/Render/ready 协议没有在 10 明示这些阶段消息。
- 失败场景：新的 labels/crops/notes 配对只有 candidate 返回元数据并经过 06 builder 后才知道；如果一次 PrepareCandidate 必须先完成“当前页/next/notes 预渲染”才能返回，当前页目标与裁剪尚未可得，容易退化成按旧裁剪/旧物理页预渲染，不能验证新文档页恢复。
- 修正归属：05/10 明确两阶段：PrepareCandidateLoad 输出 owned metadata token；结构线程合并建图、主协调器确定新 content 页/裁剪/目标；随后发带 candidate token 的 RenderRequest；全部 required-key 帧齐备才产生 CandidateReady，进入主线程提交。也可使用等价状态协议，但必须说明元数据回传在预渲染之前。
- 验证：重载改变 notes 位置、插入独立 notes 页、同标签 overlay 数量改变；每个预渲染 key 都来自新图，candidate 结果不被 active-generation 筛掉。

### PR-09 — “旧 physical 索引钳制到 content”未定义可执行算法（父/10）

- 来源：父 `design.md:87`、10 H-5 / `design.md:14` / I-4。这里同时出现 PhysicalPage 索引与跳过 notes 后的 content 序列，但没有规定是保持物理号还是内容序号，以及目标落在 notes 页时选哪一侧。
- 失败场景：旧 PhysicalPage(8)，新 content 页为 `[0,4,8,12]`；按物理号保持应为 8，若将 8 当 content Vec 下标再 clamp 会变成 12。目标恰好变为 notes 页时，不同实现可选前页或后页，测试没有唯一期望。
- 修正归属：父/10 选定一条纯定位函数契约，例如先钳制到新物理范围，再选不大于该页号的最近 content（不存在则首 content）；或者保存旧 content ordinal 后按该 ordinal 钳制，但必须改名并写清与 PhysicalPage 的区别。同一规则供 core/document/UI 与笔画映射使用。
- 验证：`[0,4,8,12]`、notes 插入/删页、目标成为 notes、标签缺失/重号、首尾与空 content；断言明确物理页结果且永不导航到 notes。

## Files / contracts consulted

- 父 `.trellis/tasks/10-14-presenter-v0.3/{prd,design,implement}.md`：源需求、共享契约、依赖与 Gate。
- 子 `.trellis/tasks/10-14-02-settings-schema/` 至 `10-14-11-media-playback/` 的 `{prd,design,implement}.md`：实际任务前置、文件所有权、步骤与旧 AC。
- 同目录 `behavior-evidence.md` 与本会话已核对的 core/document 代码：预览、标签/备注与数据边界背景；本轮没有重新打开 Pympress。
- 已加载相关 specs：backend/index、quality/database-guidelines；frontend/index、state-management/type-safety/quality-guidelines；guides/cross-layer-thinking-guide。评审以用户新契约与父设计优先，未以旧 spec 冲突重复报错。

## External references

本轮未新增外部来源或运行第三方软件；既有文档行为证据见 `behavior-evidence.md`。本报告的失败场景是根据规划契约推导的待验证用例，不是产品实测结果。

## Caveats / Not Found

- 明示父/子依赖表目前相互一致，没有在这些明示边中发现循环；PR-06 关注的是实际事件源交付的隐含反向依赖。
- 所列问题应由主会话修改其拥有的规划文档；本代理没有改其他规划、上下文、状态、源码、fixtures 或 Git。
- 未执行任何产品测试或硬件验证，也未将“文本已修正”视为实现/门禁通过。
