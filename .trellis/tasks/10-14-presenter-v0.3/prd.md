# FerroStage v0.3 演示器实施计划（PDF/Beamer 展示）

> Current acceptance policy (2026-10-15 user revision): `.trellis/tasks/10-14-presenter-v0.3/acceptance-policy.md` supersedes older hardware/matrix blockers below. Implement all features first; retain automatic/feature/native-functional checks. Physical, exhaustive native/size and performance/soak matrices are deferred, not PASS. Gate B implementation is satisfied by V16;08 may start. Gate C/D are functional integration gates. Historical results and product contracts remain unchanged.

父任务持有需求、共享契约、依赖图与最终集成验收，不是代码实施目标。2026-10-14 用户已授权在规划审阅与依赖满足后连续实施全部11个子任务；保留未提交供审阅，不提交、不推送、不归档。实际进度见 execution-state.md，父任务仍只协调。

## Goal 与已核对事实

交付可逐项启动、独立验证并最终集成的 PDF/Beamer 展示器任务树；支持显式开始、三屏角色、设置持久化和安全的讲者/观众隔离。既有父任务与11个子任务均存在，不补建同义任务。

初始规划基线（历史，非当前完成度）：9-crate骨架：core有页导航、预览、标注数据和代际检查；settings只有6个扁平键；platform只有notify类型与重载常量；document只存notes/media；PDFium、GPUI未链接。实际pins为Rust1.99.0/edition2024、lopdf0.45.0、notify8.2.0、toml1.1.8；PDFium0.9.4、Zed v1.23.3/gpui-kit0.7.3仅记录pin。详见 research/repository-baseline.md，不将此前结语视为测试证据。

## Requirements

编号保留，用于追踪。现行需求以本PRD和父design为准，旧spec冲突由01同步；技术细节在design，步骤在implement。

- **R-01** 产品代码及现行正向功能/验收不引入LaTeX编辑、编译器调用或工具链监听；热重载只处理外部替换PDF。历史/排除说明允许提及被删除能力，扫描逐条语义审查。
- **R-02** 用户PDF永远只读，禁止PDF注释编辑/写回；会话画笔仅在内存。测试写入只在独立临时目录，设置可原子保存。
- **R-03** AGENTS、README、docs、ADR及冲突spec与v0.3一致；PDF临时写回规则删除或改为仅设置文件；不删除历史说明制造“零命中”。
- **R-10** Idle→Load成功→Ready→显式Start→Presenting；Stop→Ready，Close→Idle，非法边有明确错误。
- **R-11** 观众窗口仅Presenting存在；非演示态snapshot恒Curtain，且不携带可绘制visual。
- **R-12** Start每次清零并启动计时；Stop保留冻结读数，初次Ready为0、Stop后Ready显示保留读数，Close/Load清零；暂停/继续/重置语义统一。
- **R-13** 仅存在有效Audience，或显式单屏windowed_audience时可开始；零屏/映射歧义/缺席Audience安全拒绝。
- **R-14** Ready可导航、预览、标注、黑屏以提前定位，但不输出观众像素；预览不改变已提交观众页。
- **R-20** 每屏MonitorId，来源及稳定性可见；设备路径/EDID优先，退化和歧义告警，跨接口稳定性由实测确认。
- **R-21** 每屏一个MonitorRole，单Audience；display.monitors为唯一权威，交换修改绑定，不另存独立swap反转。
- **R-22** 纯resolve_roles覆盖零/单/双/多屏、缺席、重复角色、身份歧义；保存Audience缺席不自动接管。
- **R-23** Audience拔除立即幕布/关闭/停止回Ready，讲者窗口化；新屏不自动接管或Start。
- **R-24** 主入口为“显示器设置”：先显示已检测屏幕数量和可识别编号，再分别选择“应用窗口所在屏幕”和“PDF播放屏幕”；其余屏幕为不使用。07交付最小分配面板/识别编号，09复用并整合拓扑图、解析来源与warnings；“交换屏幕”不再作为此处主要设置方式。
- **R-30** v1八节schema（general/display/presenter/timer/pointer/keybindings/cache/hot_reload）包含debug_overlay、明确grid列数/可见页数；11向v1缺省追加media节。
- **R-31** 平台配置路径和CLI覆盖；同目录临时文件原子替换；坏文件备份失败不得覆盖；Windows已有目标/锁定/失败验证。
- **R-32** 非法值及非法类型单项回退；未知顶层、已知节未知键及数组元素未知字段保留；真实旧格式转换、未来版本只读、缺省节显式写回。
- **R-33** 设置只在讲者窗口；修改有实际消费者、生效方式和保存失败“未保存/重试”状态，未来版本只读。
- **R-40** PDFium单线程内存加载、BGRA8裁剪、P0–P4调度/代际取消、预算和纹理回收；观众只得受控当前页结果；分层native/GPU门禁。
- **R-41** 拼合备注候选+手动覆盖、notes标签配对/跳过、overlay启发式+physical_pages覆盖、原始PageLabel、只读媒体槽位；不保证识别任意PDF。
- **R-42** GPUI外壳、显式开始/结束、指定屏窗口、全局动作/焦点防护；Gate B允许占位，真实PDF在08。
- **R-43** 工作台当前/下一页/备注/计时/时钟/进度、跳页、黑白屏、显示器设置入口；网格/布局/激光笔/画笔；P0功能阻塞M1，P1增强在M2前完成。
- **R-44** P2通用文件热重载和GIF/离屏视频音频；旧代际失败继续服务，媒体共享BGRA且不泄露路径/备注，无子HWND。

- **R-45** 07/08/09 的讲者界面遵循根 DESIGN.md（brand:claude 适配）：统一浅/深/系统主题、可见焦点、完整控件状态、可读对比度与原生窗口截图验证；不改变PDF/媒体像素或观众隔离。

## Task Map 与依赖

优先级列是既有任务排期，不重写功能矩阵的业务P0/P1。所有任务实施前须通过01的文档契约评审；表中列完整前置（01已合并到每行）。父子关联不表示依赖。

| 子任务（完整目录前缀10-14-） | 需求 | 排期 | 前置 | 里程碑责任 |
| --- | --- | --- | --- | --- |
| 01-docs-sync | R-01/02/03 | P0 | 无 | 契约与历史命中审查 |
| 02-settings-schema | R-21/30/31/32 | P0 | 01 | 配置与MonitorRole |
| 03-monitor-topology | R-20/21/22/23 | P0 | 01/02 | MonitorId、warnings、零屏 |
| 04-session-lifecycle | R-10/11/12/13/14/43 | P0 | 01 | 阶段、时间、预览、纯投影类型 |
| 05-pdf-raster | R-40 | P0 | 01/04 | BGRA/worker/probe/测试分发 |
| 06-beamer-document-model | R-41 | P0 | 01/04/05 | 纯SlideMap与只读构建 |
| 07-gpui-shell | R-13/21/23/24/33/42/45 | P0 | 01/02/03/04/05 | 外壳拓扑PoC及最小屏幕分配UI/Identify；05仅需import metadata，无真实画面要求 |
| 08-presenter-workbench | R-40/43/45 | P1 | 01/02/04/05/06/07，Gate B | M1所需P0工作台、真实观众画面；P1增强 |
| 09-settings-ui | R-24/33/45 | P1 | 01/02/03/07/08 | M2完整设置页复用07角色面板并使全部设置实际生效 |
| 10-hot-reload | R-01/02/44 | P2 | 01/04/05/06/07/08，Gate C | 完整重载接线；09非硬依赖 |
| 11-media-playback | R-40/44 | P2 | 01/02/04/06/07/08/09，Gate C | 媒体+设置扩展；不依赖10实施 |

05新增07导入所用owned metadata/probe，使07不直接依赖lopdf/PDFium。06可以先用注入元数据单测，但完整任务交付依赖05。10/11可在各自前置就绪后独立实施，代际接口提前定义。顺序见implement.md；需求→设计→步骤→验证见traceability.md。

## Acceptance Criteria（跨任务）

- [ ] **AC-1** Windows11三屏真实Beamer：打开后Ready/无观众/elapsed=0；07共享面板及09显示器页先显示数量/编号，再指定应用屏/PDF屏/不使用；显式Start后仅指定屏全屏，Stop关窗并保留冻结计时。Gate B须通过07最小分配UI+占位子集，不能以旧配置文件测试代替新UI，也不能代替本条真实PDF与完整设置集成。
- [ ] **AC-2** 重启按MonitorId恢复角色；记录换接口结果及身份等级。强ID不匹配/退化碰撞时告警、无自动Audience接管，不以“ID永不变”作不可验证承诺。
- [ ] **AC-3** Presenting拔Audience后≤1s关窗回Ready、讲者窗口化、不泄露备注；新屏/重新连接不自动开始；零屏恢复也须显式Start。
- [ ] **AC-4** 禁用能力扫描每处命中记录path/line/分类/处置：历史与排除允许，现行正向能力禁止。写入API评审无用户PDF目标；自建测试只写临时目录；只读前后PDF哈希一致。
- [ ] **AC-5** audience view只接收snapshot/Entity，受控当前页BGRA及媒体像素仅在snapshot内；无document/notes/path/讲者cache；属性+结构评审+截图验证不变量1/4/7/8。
- [ ] **AC-6** 任意缺省节回退并在可写模式补全；未知嵌套/monitor字段往返保留；非法类型单项回退；坏文件备份、未来版本只读、Windows替换失败原件完整，UI准确标记未保存。
- [ ] **AC-7** just ci及交付feature的PDFium native测试、GPUI编译门全部通过；实机报告关联代码版本，无ignore/缺库成功跳过。
- [ ] **AC-8** 外部替换完整/分块/临时rename/损坏PDF：失败旧代服务、无混代/中间黑帧、保持页和计时、旧资源释放，测试文件位于独立临时目录。
- [ ] **AC-9** GIF和所选单视频后端共享BGRA、无子HWND/路径泄漏；切页/blank/Stop/代际失效停播；缺native库仍能启动；性能/许可证/回收报告通过。

- [ ] **AC-10** DESIGN.md 对比度检查、07外壳/08工作台/09设置的双主题原生截图、键盘焦点、禁用/错误状态及尺寸/DPI验证通过；截图关联代码版本，不以mockup代替。

## M1 / M2 准出

M1（Gate C）是有范围的Alpha：01–08技术验收及Gate A/B，真实PDF、备注隔离、显式生命周期、计时/导航/跳页/黑白屏/显示器分配/真实双窗为必需P0；M1使用07最小分配UI指定屏幕。M1不宣称完整设置交付，09仍负责R-24拓扑图/设置集成与R-33其余设置。08的网格/拖拽/高级画笔等P1可以在同任务M2收尾记录，但不得归档为完整任务或标全部Implemented。

M2（Gate D）：09完整设置集成和全部P0/P1、08剩余增强、10/11全部P2、AC-1…AC-10完整通过。P2不得静默移出范围。技术Gate按已验证交付物触发，与提交/归档分开；当前Gate A纯契约已通过，证据见research/gate-a.md；Gate B/C/D与03剩余物理验收仍待验。

## Out of Scope 与决策状态

不做LaTeX源码编辑/编译/监听、PDF注释编辑/写回、统计导出、animate、双媒体后端、多Audience镜像、PPTX、云/账号/遥测；跨平台安装包在Phase3另行规划。只读媒体字典不是注释编辑。

单屏默认presenter_only、窗口观众显式开启、单Audience、热重载P2已由本次要求确认，无需再猜用户意图。文档沿用现有任务/PROJECT_PLAN中文，01所改英文docs/spec继续英文、中文README保留双语。03安全OS封装已有技术交付，物理验收待验；07两条原始依赖路线的政策失败保留为历史。ADR-008受限维护评估及最终候选独立检查后，用户明确批准五个本地维护包及三个精确许可证例外，已接入Zed v1.23.3路线且产品依赖/编译门通过。V6自动检查、焦点与保存反馈修复已独立验证并通过原生复测，完整实机拓扑/主题矩阵及Gate B仍待验，详见execution-state.md。11媒体loader/许可证仍待前置就绪后研究；其他unsafe/许可证政策及验收范围不变，后续政策变更仍须具体ADR和用户授权。
