# Traceability — 需求、所有者、步骤与验证

> Current2026-10-15 user revision: Gate B implementation PASS on verified V16 evidence; original extended hardware/native qualification DEFERRED_BY_USER.03/07 are eligible for implementation-scope archive,08 is unblocked. See `.trellis/tasks/10-14-presenter-v0.3/acceptance-policy.md`; older incomplete/blocking statements below are historical under the previous policy. No old test outcome is changed.

R/AC来自父prd；D/S/M/L/P/B/G/W/U/H/V为原子PRD编号，保持不重编号。子目录统一.trellis/tasks/10-14-<slug>，所有02–11设计与implement步骤I-1…7真实存在；07本轮追加I-8并要求在Gate B前完成；01为PRD-only。此表是验收索引，不表示通过。

| 父需求 | 子任务 / 子需求 | 设计 / 步骤 | 测试或手工验收 / Gate |
| --- | --- | --- | --- |
| R-01 | 01 D-1/6/7；10 H-1/6 | 01 PRD；10线程设计/I-2/7 | 语义命中清单/无compiler；AC-4 |
| R-02 | 01 D-1；08 W-6；10 H-6 | 08坐标/I-6；10只读/I-7 | session-only笔画、用户PDF hash、temp写目标；AC-4/8 |
| R-03 | 01 D-1…8 | 01 PRD契约同步 | AGENTS/ADR/spec一致与命中处置；Gate A前 |
| R-10 | 04 L-1…5；07 G-2…5 | 04状态/I-2/3；07事务/I-4 | 全合法/非法边、窗口失败回滚；Gate A/B |
| R-11 | 04 L-7；07 G-9；08 W-4 | 04投影/I-4；08I-3 | 非演示Curtain无visual、窗口关闭；AC-1/5 |
| R-12 | 04 L-6；08 W-3 | 04时间/I-3；08I-4/7 | 受控Start/Pause/Stop/Restart、渲染阻塞；Gate A/C |
| R-13 | 03 M-3/4；04 L-3；07 G-4 | 03拓扑/I-3；07I-4 | 零/单/缺席/映射歧义拒绝，显式windowed；Gate A/B |
| R-14 | 04 L-8/11；08 W-7 | 04预览/I-5；08I-4 | Ready不投影，preview不改committed，Enter/Esc；AC-5 |
| R-20 | 03 M-1/2；07 G-13；09 U-2 | 03身份/I-1/4/6；07 I-8；09I-3 | strong/fallback/碰撞、换接口，真实等级可见；AC-2 |
| R-21 | 02 S-2；03 M-3；07 G-8/13；09 U-2/10 | 父design §3；02I-2、03I-3、07I-5/8、09I-3 | 数量/编号/两项选择、role roundtrip/单Audience、快捷键兼容绑定一致；AC-1/2 |
| R-22 | 03 M-3/4 | 03纯解析/I-3 | 零/一/二/三屏、重复角色/ID、缺Audience不接管；Gate A |
| R-23 | 03 M-5/6；07 G-7；09 U-2 | 03I-5/6；07I-5/6；09I-6 | event/diff、拔线≤1s、重新接入不Start；AC-3 |
| R-24 | 07 G-13；08 W-1；09 U-2/10 | 07 I-8；08 I-4；09 I-3/6 | Gate B最小分配/数量/编号/Identify；Gate D完整设置/真实PDF；AC-1/2 |
| R-30 | 02 S-1/3；11 V-6 | 02schema/I-2；11I-5 | 八节+debug/grid两键、media默认旧v1兼容；AC-6 |
| R-31 | 02 S-4/5/6/10 | 02store/I-4/5/6 | config CLI/坏备份/NTFS已有目标和失败；AC-6 |
| R-32 | 02 S-5/7/8/9 | 02merge/I-3/4 | 嵌套/数组unknown、非法type、future readonly、missing写回；AC-6 |
| R-33 | 07 G-8/13；09 U-1…9；10 H-8；11 V-6/8 | 07 I-8；09consumer表/I-3/4/5；10I-5；11I-5 | Gate B最小分配保存/重启/dirty retry及future只读；Gate D完整设置实际效果/capability控件 |
| R-40 | 04 L-7；05 P-1…9；08 W-9/10；11 V-3 | 父design §5/6；04I-4、05I-2…7、08I-3/7、11I-4 | 单worker、BGRA/同密度crop、priority/candidate/cache、纹理；AC-5/7/9 |
| R-41 | 06 B-1…8；08 W-2/11 | 06layout/notes/map/I-2…6；08首Load/I-2、文档控制/I-4 | A–E自建夹具、标签反例、首Load安装map、五layout/识别/分组控件实际生效、无notes导航；Gate C |
| R-42 | 07 G-1…13 | 07shell/I-1…8 | compile、最小分配UI三屏占位/focus/Start/Stop；Gate B，真实PDF由08 |
| R-43 | 04 L-11；08 W-1…10；09 U-4…6 | 04I-5；08I-2…7；09I-4 | M1 P0汇报/preview/timer/基本pen；M2 grid/高级pen/布局；warm Ready/Start/CPU/grid协议；Gate C/D |
| R-44 热重载 | 10 H-1…8 | 父design §7；10I-1…7 | temp分块/rename/坏PDF/连续替换/Close/回滚/资源退役；AC-8 |
| R-44 媒体 | 06 B-6；11 V-1…8 | 06I-4；11I-1…7 | safe loader/缺DLL启动/共享BGRA/停止/性能许可；AC-9 |

## 父 AC 的证据归属
| AC | 责任 / 最终证据 |
| --- | --- |
| AC-1 | 07数量/编号/最小分配UI与占位 + 08真实PDF + 09完整设置最终路径；Gate D汇总 |
| AC-2 | 03 identity稳定性/退化安全 + 02 save + 07最小分配/重启 + 09完整UI |
| AC-3 | 03event + 07窗口事务 + 08/09真实投影拔线 |
| AC-4 | 01语义命中清单；08/10/11只读hash与write目标review |
| AC-5 | 04结构/属性 + 06 notes ownership + 08真实视觉筛选 + 11媒体隔离 |
| AC-6 | 02 TOML/NTFS/future + 09dirty/实际UI + 11 media默认兼容 |
| AC-7 | 每个子task just ci；05native job、07GPUI build、11媒体native job、实机证据 |
| AC-8 | 10完整跨platform/pdf/render/document/core/ui/app事务与temp integration |
| AC-9 | 11 backend/许可/缺库启动/PoC4，与08RAII/overlay及10generation失效组合 |
| AC-10 | DESIGN.md对比度、07主题基础、08工作台、09设置原生截图与交互；Gate D汇总 |

## 接线无主检查
02→03 MonitorRole由02；03→07 MonitorId/DisplayId由03数据+07映射；04→05 BgraFrame由04创建/05复用；05→06metadata由05/06owned构建；08负责首Load full document，10负责reload；Prepare先metadata/建图定位后PreRender；06→08notes由06/08Presenter独占；05→08visual由08组装；07→08/09共享分配面板与roles协调器由07创建、08/09复用；02/08→09各实际consumer由09接线；10commit/retire由10跨crate协调；11→08 media visual与→09 media控件由11扩展，10只发布generation失效接口不等待11。无循环，完整前置以父Task Map与子Dependencies一致核查。

M1 Alpha验明确P0工作台及07最小角色分配UI，不声称所有产品P0完成；最终UI和P1/P2在M2。Gate技术证据与提交/归档分开；任务状态和当前实施进度以task.json及execution-state.md为准。

## 2026-10-14 主题需求增量

| 需求 | 所有者 | 设计/步骤 | 验证 |
| --- | --- | --- | --- |
| R-45 / AC-10 | 07基础token，08工作台，09设置 | 根DESIGN.md；父design §9；07 I-2/3/7，08 I-2/6/7，09 I-2/5/7 | 原生双主题/尺寸/DPI截图、对比度、键盘/控件状态；Gate D汇总，未运行保持NOT_RUN |

## 2026-10-14 历史证据边界（V6）

01/02/04/05/06技术交付已独立检查，03自动与静态检查通过、人工项待验。06的128默认/63native检查为其交付基线；07新增GPUI集成后，V6当前默认CI145、实际GPUI49、严格feature lint及native build通过，底层未变的PDFium/Beamer实现沿用对应源身份的独立证据。Gate A纯契约PASS，B不完整、C/D未运行，所有父AC最终验收未完成。AC-4的01语义审查和07只读PDF哈希、AC-5的04/06/07隔离、AC-6的02存储和07实际失败/Retry、AC-7的自动/PDFium检查、AC-10的51组对比度及部分原生截图均只是各自子集，不能勾选父AC。

07原始路线的依赖门失败保留为历史；ADR-008维护评估、最终五包及三个精确许可证例外获批后，产品接入及I-2实际门禁已通过。实际A/B/C窗口、主题/语言和导入已部分验证；V6修复焦点与保存反馈并通过独立检查/原生复测，选择器与保存角色重启通过。物理拓扑/延迟和剩余原生矩阵仍待验，Gate B不完整；08–11保留完整范围并受既定依赖/Gate阻塞。报告索引与恢复动作见execution-state.md及07 research/native-v6-checkpoint.md。尚未运行的M1长时汇报、GPU/VRAM、设置UI、热重载及媒体集成不得以已有纯测试替代。


## Current evidence - 2026-10-15 V16 resume audit

The fresh serial review verifies648 source inputs and all four V16 manifests; CI151/UI82/COM8/lint/typecheck/deny/build remain applicable without rerunning unchanged product checks. The approved sixth AccessKit package and07 display panel are implemented; core three-screen/native lifecycle observations are scoped PASS. Gate A remains technical PASS, B INCOMPLETE, C/D NOT_RUN. All parent AC-1 through AC-10 final acceptance remains incomplete, including M1/M2, actual workbench/notes/settings/reload/media integration. Physical single/zero/hotplug and remaining native matrix are not replaced by current automatic tests. Actual11-child paths, dependencies and next steps: `research/resume-2026-10-15.md`.
