# Traceability — 需求、所有者、步骤与验证

R/AC来自父prd；D/S/M/L/P/B/G/W/U/H/V为原子PRD编号，保持不重编号。子目录统一.trellis/tasks/10-14-<slug>，所有02–11设计与implement步骤I-1…7真实存在；01为PRD-only。此表是验收索引，不表示通过。

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
| R-20 | 03 M-1/2；09 U-2 | 03身份/I-1/4/6；09I-3 | strong/fallback/碰撞、换接口，真实等级可见；AC-2 |
| R-21 | 02 S-2；03 M-3；07 G-8；09 U-2/10 | 父design §3；02I-2、03I-3、07I-5、09I-3 | role roundtrip/单Audience、交换绑定一致；AC-1/2 |
| R-22 | 03 M-3/4 | 03纯解析/I-3 | 零/一/二/三屏、重复角色/ID、缺Audience不接管；Gate A |
| R-23 | 03 M-5/6；07 G-7；09 U-2 | 03I-5/6；07I-5/6；09I-6 | event/diff、拔线≤1s、重新接入不Start；AC-3 |
| R-24 | 09 U-2/10 | 09角色表/I-3/6 | UI三屏/Identify/来源/warnings；AC-1/2，Gate D |
| R-30 | 02 S-1/3；11 V-6 | 02schema/I-2；11I-5 | 八节+debug/grid两键、media默认旧v1兼容；AC-6 |
| R-31 | 02 S-4/5/6/10 | 02store/I-4/5/6 | config CLI/坏备份/NTFS已有目标和失败；AC-6 |
| R-32 | 02 S-5/7/8/9 | 02merge/I-3/4 | 嵌套/数组unknown、非法type、future readonly、missing写回；AC-6 |
| R-33 | 09 U-1/3…9；10 H-8；11 V-6/8 | 09consumer表/I-4/5；10I-5；11I-5 | 实际效果/重启/dirty retry、future只读、capability控件；Gate D |
| R-40 | 04 L-7；05 P-1…9；08 W-9/10；11 V-3 | 父design §5/6；04I-4、05I-2…7、08I-3/7、11I-4 | 单worker、BGRA/同密度crop、priority/candidate/cache、纹理；AC-5/7/9 |
| R-41 | 06 B-1…8；08 W-2/11 | 06layout/notes/map/I-2…6；08首Load/I-2、文档控制/I-4 | A–E自建夹具、标签反例、首Load安装map、五layout/识别/分组控件实际生效、无notes导航；Gate C |
| R-42 | 07 G-1…11 | 07shell/I-1…7 | compile、配置三屏占位/focus/Start/Stop；Gate B，真实PDF由08 |
| R-43 | 04 L-11；08 W-1…10；09 U-4…6 | 04I-5；08I-2…7；09I-4 | M1 P0汇报/preview/timer/基本pen；M2 grid/高级pen/布局；warm Ready/Start/CPU/grid协议；Gate C/D |
| R-44 热重载 | 10 H-1…8 | 父design §7；10I-1…7 | temp分块/rename/坏PDF/连续替换/Close/回滚/资源退役；AC-8 |
| R-44 媒体 | 06 B-6；11 V-1…8 | 06I-4；11I-1…7 | safe loader/缺DLL启动/共享BGRA/停止/性能许可；AC-9 |

## 父 AC 的证据归属
| AC | 责任 / 最终证据 |
| --- | --- |
| AC-1 | 07配置/占位子集 + 08真实PDF + 09三屏UI最终路径；Gate D汇总 |
| AC-2 | 03 identity稳定性/退化安全 + 02 save + 09重启/UI |
| AC-3 | 03event + 07窗口事务 + 08/09真实投影拔线 |
| AC-4 | 01语义命中清单；08/10/11只读hash与write目标review |
| AC-5 | 04结构/属性 + 06 notes ownership + 08真实视觉筛选 + 11媒体隔离 |
| AC-6 | 02 TOML/NTFS/future + 09dirty/实际UI + 11 media默认兼容 |
| AC-7 | 每个子task just ci；05native job、07GPUI build、11媒体native job、实机证据 |
| AC-8 | 10完整跨platform/pdf/render/document/core/ui/app事务与temp integration |
| AC-9 | 11 backend/许可/缺库启动/PoC4，与08RAII/overlay及10generation失效组合 |

## 接线无主检查
02→03 MonitorRole由02；03→07 MonitorId/DisplayId由03数据+07映射；04→05 BgraFrame由04创建/05复用；05→06metadata由05/06owned构建；08负责首Load full document，10负责reload；Prepare先metadata/建图定位后PreRender；06→08notes由06/08Presenter独占；05→08visual由08组装；07→09roles协调器由07/09UI消费；02/08→09各实际consumer由09接线；10commit/retire由10跨crate协调；11→08 media visual与→09 media控件由11扩展，10只发布generation失效接口不等待11。无循环，完整前置以父Task Map与子Dependencies一致核查。

M1 Alpha只验明确P0工作台及配置角色，不声称所有产品P0完成；最终UI和P1/P2在M2。Gate技术证据与提交/归档分开，当前所有任务仍planning。
