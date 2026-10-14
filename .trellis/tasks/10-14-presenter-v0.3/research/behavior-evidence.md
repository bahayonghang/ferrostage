# Research: 展示行为证据与 Beamer 识别规则

- Query: 为 04/06/08/09/10/11 建立“文档位置 → R-xx → 独立 Rust 设计 → 验收”映射，解决拼合比例、PageLabel、备注配对与手动覆盖的歧义。
- Scope: mixed；仅规划研究，不启动任务，不改产品代码。
- Date: 2026-10-14
- Evidence status: 下表 Pympress 内容均为**文档描述，未实际运行验证**；本次没有显示器、PDFium、GPU 或媒体实测结果。

## Findings

### Files found / related specs

| 文件 | 作用与已核对位置 |
| --- | --- |
| `ref/pympress/README.md` | 仅阅读允许的用户文档；备注与覆盖 195–202，跳页 205–209，指针/手绘 204/212，文件重载 214，交换屏幕 216，持久化/预览数量 226–239，媒体 203/242–260。 |
| `ref/pympress/docs/options.md` | 用户配置文档；路径 5–11、布局可调整 25–26、退出保存比例 93–94。未复制其中配置示例；94 行比例算式存在明显笔误，不能据其数值设计。 |
| `docs/beamer-notes.md:3–7` | 当前说明明确尚未实现，并只覆盖默认右侧备注与临时一物理页导航。 |
| `docs/PROJECT_PLAN.md:235–236` | 现行上下比例遗漏 4:3 的上下拼合；与 06 的倒数判定不一致。 |
| `.trellis/tasks/10-14-{04-session-lifecycle,06-beamer-document-model,08-presenter-workbench,09-settings-ui,10-hot-reload,11-media-playback}/prd.md` | 核对各子任务范围、输入和现有验收；本研究只读这些 PRD，不加载 implement/check JSONL。 |
| `crates/ferrostage-core/src/state.rs:86–94,107–157` | 快照当前读取预览页，且普通翻页会自动提交预览；不符合 R-43 的预览隔离要求。 |
| `crates/ferrostage-core/src/geometry.rs:1–33` | 坐标原点左上；已有左右半幅，未定义上下半幅。 |
| `crates/ferrostage-core/src/snapshot.rs:13–26` | 观众快照不含备注；尚无阶段/像素资源入口。 |
| `crates/ferrostage-document/src/lib.rs:8–51` | 备注、媒体描述归 document；`project` 不读备注；尚无标签、步进、备注配对模型。 |
| `fixtures/beamer/README.md:3` | 尚无 Beamer PDF 样本，不能把 A–D 夹具当作已存在或已通过。 |
| `docs/adr/ADR-002-pdf-backends.md:13–23` | 只有 lopdf 已链接；PDFium 未链接，裁剪/标签功能仍待 05。 |
| `docs/adr/ADR-003-audience-snapshot.md:9–15` | 备注隔离与现有交换标志契约；交换标志需按父级角色契约调整。 |
| `.trellis/spec/backend/{index,database-guidelines,quality-guidelines}.md` | 文档数据归属、禁止 GPL 来源复制、必跑 CI 与禁止以 ignore 变绿。 |
| `.trellis/spec/frontend/{index,state-management,type-safety,quality-guidelines}.md` | 单一核心状态、观众只读快照、纯 newtype、单渲染线程与两窗共享 BGRA 帧。 |
| `.trellis/spec/guides/index.md` / `cross-layer-thinking-guide.md` | 跨 document/core/render/UI 的同一契约与边界检查。 |

### 文档行为到需求/验收的追踪

“独立设计”栏是 FerroStage 的规划建议，均不是对 Pympress 实现的描述。

| 来源位置 / 文档描述 | 父需求 / 子任务 | 独立 Rust 设计 | 验收方式与归属 |
| --- | --- | --- | --- |
| README:195–202,239：备注位置自动判断，可按 none/left/right/top/bottom/after 覆盖；独立备注例子使用 `notes:` 前缀。 | R-41；06 建模，08 展示 | 文档构建器输出检测来源、布局候选和配对诊断；当前文档的显式布局覆盖优先，独立备注识别单独控制。标签/备注关联留 document，纯可导航内容映射归 core。 | 06：自建五种布局、独立备注与反例夹具；08：菜单覆盖马上改变讲者预览，观众取同一已提交裁剪。 |
| README:205–209：预览只在讲者窗变化；Enter 确认，Esc 取消；标签导航区别于物理页。 | R-14/R-43；04 隔离，06 映射，08 交互 | 预览位置与已提交位置分开；观众快照恒使用已提交页。预览期间翻页只移动预览，不隐式提交；Commit/Cancel 是唯一出口。标签不是索引，重复标签显示候选位置。 | 04：预览任意合法页时观众 generation/page/crop 不变，Cancel 不改已提交页，Commit 改页；08：连续预览十页、Enter/Esc 和首尾边界。 |
| README:195,222：讲者显示演讲时间/时钟、目标时长与颜色提示；**允许文档未说明 Pause/Stop/再次 Start 细节**。 | R-10–R-12/R-43；04/08 | 按本项目定义生命周期与单调计时，命令显式注入 now 供确定性测试。建议每次 Start 开始新演讲并清零；Stop 保留停止读数，Ready 显示该读数，Close/Load 清零；暂停只在 Presenting 内，Resume 不等于再次 Start。 | 04：Start→5s→Pause→5s→Resume→2s→Stop 得 7s，Ready 再过 5s 仍 7s，再 Start 为 0；08：显示同一核心读数与阈值，渲染阻塞不影响读秒。 |
| README:204,212：当前幻灯片手绘与软件指针。 | R-02/R-14/R-43；08 与 core 命令 | 内容区域的归一化坐标经过两端各自内容矩形变换；笔画仅在会话内存。黑/白幕布覆盖媒体、笔画、指针全部层。 | 08：不同 DPI/纵横比屏幕对齐、letterbox 区不产生点、鼠标离开清除指针、幕布最高层、用户 PDF 哈希不变。硬件检查必须后续实测。 |
| README:216：交换屏幕；217–221 还描述自动把非主屏全屏。 | R-21–R-24/R-43；03/07 提供契约，08/09 消费 | 只吸收显式交换的结果。交换通过权威角色绑定更新→resolve_roles→窗口协调器迁移，不能再让 `screens_swapped` 独立决定角色。FerroStage 打开文件不接管屏幕，保存 Audience 缺席不自动转用其他屏。 | 03：角色解析纯测试；08/09：交换后设置、解析结果和窗口目标相同，内容页/裁剪/计时不变；07/09 三屏测试留 Gate B/最终集成。 |
| README:224,226–229；options:5–11,25–26,93–94：可改布局、配置持久化，1–16 下一页预览。 | R-30–R-33/R-43；02 数据，08 使用，09 UI | 使用本项目 TOML schema，不复制配置模板。独立定义 `grid_columns` 为列数、可见页数为单独设置，避免把列数当页数。保存失败保留本次会话实际值并显示未保存状态，不声称重启后保留。 | 02：递归未知键/非法单项/缺省节/未来版本只读/Windows 原子替换；08/09：每个控件改值→实际呈现→保存→重开一致，save 失败不会虚报成功。 |
| README:214：文件修改后重载，保持当前幻灯片和时间；未规定防抖、EOF、影子线程或错误 UI。 | R-01/R-40/R-44；10，完整接线依赖 04/05/06/07/08 | 外部 PDF 替换事件→文件读取稳定检查→结构解析→单 PDFium 线程影子加载/预渲染→主线程整体提交。标签匹配只用唯一候选，重复标签不能任取第一个；否则按内容序列位置回退并钳制。计时/阶段不变，失败继续服务旧代际。 | 10：独立临时目录分块写入/rename/损坏文件；旧代际直到完整新首帧可提交都有效，旧 PDFium 对象在渲染线程回收。标签缺失/重复/页数缩短的恢复路径分别测试。 |
| README:203,242–260：嵌入/外链视频、音频、GIF，起止时间和循环；**允许文档未说明切页停止及 ≤100ms 指标**。 | R-41/R-44；06 只读槽位，11 播放 | 参数来自独立 PDF 结构规范/自建夹具核对，不能从 README 反推完整媒体字典。单解码器共享 BGRA8 帧；观众仅得受控 slot ID/矩形/帧，不能得文件路径或媒体描述器。切页、黑/白屏、Stop、重载、Close 使播放 epoch 失效，丢弃迟到帧。 | 06：本项目制作的媒体 PDF 与期望元数据比对；11：假解码器验证生命周期、迟到帧和单路音频，再做真实媒体同步/音频停止/显存实测。PoC-4 仍是未完成研究门禁。 |

### 拼合几何与覆盖规则（06 的推荐统一定义）

外部文档证据：Beamer 3.78 用户指南 §19.3（印刷页 199）说明默认备注在右，可改左/上/下；§22（页 209）允许再自定义 pgfpages 布局。[Beamer 用户指南](https://tug.ctan.org/macros/latex/contrib/beamer/doc/beameruserguide.pdf#page=199)。PGF 页面管理 §90.1 的双屏布局说明：左右为宽度加倍，高度不变；上下为高度加倍。[PGF 页面管理](https://tikz.dev/pages)。以下比例是根据这些尺寸关系计算的**本项目推导**。

设基础幻灯片比例 `r = w/h`，解析后的显示方向页尺寸为 `W,H`（含页面旋转，05 统一提供）：

| 基础比例 | 左右拼合 `W/H=2r` | 上下拼合 `W/H=r/2` |
| --- | --- | --- |
| 16:9 | 32:9 ≈ 3.5556 | 8:9 ≈ 0.8889 |
| 4:3 | 8:3 ≈ 2.6667 | 2:3 ≈ 0.6667 |

- 06 B-1 的“高宽比与左右区间相同”得到约 9:32 / 3:8，是方向判定错误；PROJECT_PLAN 的 `[0.8,1.0]` 又遗漏 2:3。建议统一为表中四个中心值的显式相对容差（例如 2%，这是待夹具验证的项目策略，不是 Beamer 标准）。
- 自动检测只能产生左右/上下**候选**，不能区分 Left/Right 或 Top/Bottom，也不能证明另一半确实是备注。按默认约定可选 Right/Bottom，但必须向讲者显示“比例推测”来源和手动覆盖入口。普通超宽页面、竖版页也可能撞上候选比例；测试必须包含这些反例。
- 文档页尺寸不一致、旋转结果混合、落在容差外时不应对整份文档凭多数投票强裁剪；返回 None/诊断，用户可显式覆盖。首版不承诺识别自定义 pgfpages 或任意混合布局。
- `NotesLayout` 表示**备注所在半幅**：Right→slide LEFT/notes RIGHT；Left→slide RIGHT/notes LEFT；Bottom→slide TOP/notes BOTTOM；Top→slide BOTTOM/notes TOP；None→FULL/无裁剪备注。文档 override 优先于检测；新 Load 重置，监听同一文件的重载保留显式 override。
- 独立备注识别与拼合裁剪是两条规则；None 只关闭拼合裁剪，不能隐式让 `notes:` 页进入观众导航。需要单独的“独立备注识别”开关解决标签误判；只在当前文档生效即可，不增加全局文档历史存储。

### PageLabel、overlay 与独立备注（独立规则，不能当作已验证事实）

允许的 README 只说明按标签导航对多页 Beamer frame 有用（205–206）；Beamer 用户指南解释了 overlay 和备注输出，但本次在该手册未找到 `page label` / `pdfpagelabel` 的普适保证。因此“相同标签必定是同一 frame”只能作为启发式，不能作为 PDF 的可靠唯一标识。

| 情况 | 推荐规则 | 最小验收数据 |
| --- | --- | --- |
| 缺失/空标签 | 保存为 `None`；每个内容物理页独立逻辑页，UI 可显示物理页号，但不得把合成显示号当原始标签去分组/配对/重载匹配。 | `None,None,"",None` 不合成一个 overlay。 |
| 非数字标签 | 原始 `Option<String>` 逐字保存；标签与 `LogicalSlide(usize)` 分离，不解析成数组下标。 | `"i","A-1","Intro","01","1"` 往返保留；`01` 与 `1` 不相等。 |
| 连续重复标签 | 仅连续的、非空同标签内容页分成一个候选逻辑组，保留各物理步；组内夹着已识别 notes 页不打断内容步。记录来源为启发式；提供 physical_pages 覆盖关闭分组。 | `1,1,notes:1,2` 内容步为 0/1/3，按步导航仍能逐页。 |
| 非连续重号/相邻不同 frame 重号 | 不全局合并非连续组；相邻同标签而实际不同 frame 无法仅凭允许证据辨认，用户关闭分组。标签跳转返回候选组列表或明确选择，不任取第一处。 | `1,2,1` 为三组；反例 `Title,Title` 关闭分组后为两页。 |
| `notes:<suffix>` | 默认依明确前缀从内容导航剔除；suffix 按原始字符串匹配紧邻此前内容步的标签，不能把 `notes:15` 当作第 15 个 Vec 项。README 的已文档化例子使用数字 frame 编号；支持非数字 suffix 是本项目字符串匹配规则，须单独测试。 | 内容 label `15`/备注 `notes:15` 能配对；内容 `01`/备注 `notes:1` 不配对。 |
| 重号造成配对歧义 | 只关联紧邻此前内容步且标签相等的连续备注页，不搜索整份文档把备注配到任意同名页；不存在匹配则保留 orphan 诊断，仅讲者可见。 | 开头 `notes:1`、`1,2,notes:1` 不配对；均不出现在观众导航。 |
| 多备注页、各 overlay 的不同备注 | document 按内容**物理步**记录有序备注页列表；core 纯 SlideMap 只含可导航内容步，不带 notes_page。08 随当前步显示关联备注页，可在原备注面板内滚动展示。单个 `LogicalSlideEntry.notes_page: Option<_>` 会丢失信息，应调整。 | `1,notes:1,notes:1,1,notes:1`：第一步关联两页，第二步关联一页；观众只见两个内容步。 |
| 重载标签重复 | 只有新文档中唯一同标签逻辑组才作为可靠位置线索；否则保留原内容序列位置并钳制，step 索引也须钳制。 | 标签缺失、重号、被删除，及新文档变短；不得落到 notes 页。 |

Beamer §19.3 描述普通 notes 可穿插、拼合模式可能只输出最后一页备注；这一版本行为不能用来承诺任意 PDF 都只有一个 notes 页。[Beamer 用户指南](https://tug.ctan.org/macros/latex/contrib/beamer/doc/beameruserguide.pdf#page=199)。FerroStage 只能消费 PDF 中实际存在的页；不能恢复已在 PDF 导出时丢失的备注。

06 的属性测试应限定：在存在后继的内容位置上 `prev_step(next_step(p)) == p`，在存在前驱时反向成立；首尾分别测试钳制/返回 None 的既定规则。若 next_logical 返回新组首步，前后逻辑导航只比较逻辑组，不要求还原先前任意 overlay 步。全部可导航位置必须是内容页且在范围内。

裁剪验收必须按相同源像素密度比较：左右半幅输出 `Cw×Ch` 时，全页对照至少为 `2Cw×Ch`；上下半幅为 `Cw×2Ch`，再裁切对应半幅比较；不能把同分辨率的全页缩小图当无损对照。PDFium/抗锯齿差异的容差由 05 用实际夹具说明。

### 必须在后续规划/实现中处置的当前代码差异

- 04：`state.rs:89` 把 jump preview 直接投影，`129–133` 会自动提交；必须先修共享状态契约，08 才能兑现 Enter/Esc。
- 06：`ids.rs:28–33` 的 LogicalSlide 仍为临时标识；`document/lib.rs:27–30` 无模型，B-4 的全部保证尚未实现。
- 08/09：08 W-3 “Ready 固定 00:00”与 04 L-4 Stop 保留读数冲突；采用同一生命周期读数。09 的 grid 列数/可见页数以及 debug_overlay 等键由父级 schema 契约统一；控件验收必须包含实际效果。
- 10：H-3 在渲染线程外加载 PDFium 与单线程规则冲突；README 的行为描述不能为此线程设计背书。全接线依赖 04/05/06/07/08。
- 11：V-5 需要覆盖所有幕布和 stage/generation/page epoch；V-8 媒体禁用状态仍需 UI 实效。README 不能证明绑定/API/许可证/Windows 分发已满足。
- 01/主会话：旧 spec 中 PDF 原子写（backend/database-guidelines:29）、settings 全会话只读与 SwapScreens flag（frontend/state-management:10–12）、缺屏退主屏（frontend/quality-guidelines:26），均与此次用户明确契约冲突。应记录 spec 同步责任，执行上下文明确父级新契约优先；本研究不修改 spec。

## External references

- [Beamer 用户指南 3.78](https://tug.ctan.org/macros/latex/contrib/beamer/doc/beameruserguide.pdf)：2026-08-20，§8.3（页 75）、§19.3（页 199–200）、§22（页 209–210）；仅查公开用户手册，不读取实现。
- [PGF/TikZ 页面管理手册](https://tikz.dev/pages)：§90.1 双屏布局；HTML 页标示更新 2026-08-07；尺寸倍增规则作为上述几何推导依据，未读取包实现。
- 本地 Pympress 两份允许用户文档只按路径/行号定位；没有用其 Python、automodule、配置模板、UI 资源或测试作为设计输入。

## Caveats / Not Found

- 未运行 Pympress。允许文档没有确认计时暂停/结束/再次开始、媒体离页停止及其延迟、PDF 未写完的重载方式。相关条目必须标为 FerroStage 自定义要求/后续待验证。
- 尚无自建 Beamer PDF；所有夹具/测试、GPU 纹理回收、三屏迁移和媒体性能是后续验收计划，不是已通过证据。
- 比例容差、重复标签分组、notes suffix 配对和多备注页处理是本项目可审查的规则；不能保证无损恢复任意 PDF 的原始 frame 结构。保留诊断与手动覆盖。
- 媒体 PDF 字典和后端 API 需要 06/11 的独立主源研究与 PoC；没有依据 README 推断字段位置、SDK 可用性或安全封装已成立。
- 本次只写本研究文件；没有修改 task 状态、产品源码、fixtures、spec、JSONL 或 Git。
