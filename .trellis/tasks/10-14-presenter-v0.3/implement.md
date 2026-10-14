# Implement — 启动顺序与集成评审

本文件只规划后续实施；本次不执行start、产品代码、提交、归档。父任务不启动。子任务获后续明确实施授权且其prd/design/implement和依赖经过审阅后，才按Trellis启动；01可PRD-only。

## 依赖顺序

可行串行顺序：01 → 02 → 04 → 03 → 05 → 06 → 07 → 08 → 09 → 10 → 11。
并行就绪集合：01后02/04；02后03与04后05；05/04后06；02/03/04/05后07；08后09与满足Gate C的10；09后11可与10独立推进。并行工作不得同时定义共享类型，创建者见design §1。

01先同步契约；02/03/04通过Gate A。07通过Gate B才进入08窗口集成；05/06纯逻辑研究可在此之前推进。08的P0交付通过Gate C，09以及10/11分别按PRD依赖推进。Gate C不以P1全部完成为先决，但M2前补完08全部P1。技术验收不要求先归档；归档只在完整子任务验收及后续收尾授权后进行。

## Gate A — 纯契约（02/03/04）

- 条件：02/03/04交付可用；运行 cargo test -p ferrostage-core -p ferrostage-settings -p ferrostage-platform；core依赖树无IO/OS/GPUI/PDFium/lopdf/notify。
- 覆盖：时间受控Start/Pause/Stop/Restart、Curtain、preview不影响audience、零/一/二/三屏、重复/缺席/退化身份、nested未知设置/非法类型/未来版本/原子失败。
- 证据归属：02的设置测试及Windows替换记录，03的枚举/ID稳定性记录，04的状态/快照测试；父级索引关联命令、HEAD与dirty范围。无三屏数据时纯契约可通过，但03实机验收仍待验，不宣称整个03完成。

## Gate B — 外壳与拓扑 PoC（07）

- 条件：07及其02/03/04/05前置交付可用、Gate A纯契约通过。05提供只读导入元数据；窗口展示占位即可，不验真实PDF像素或备注面板。
- 三屏Windows11：角色由02配置指定A Audience/B Presenter/C None；打开PDF→Ready无观众/不计时；Start→A占位全屏；Stop→关窗；换C配置再复测；拔Audience≤1s安全降级。焦点在观众屏触发动作，弹UI必回讲者。单屏两模式/零屏/歧义拒绝测试。
- 证据归属：07实施时创建 research/poc-1-report.md（当前未创建，不注入JSONL），含设备、DPI、操作/截图、结果、代码版本。GPUI API研究与编译门另记在07；不能声称09设置UI已可用。
- 阻塞范围：未通过实机拓扑/focus门不进入08真实双窗集成；可继续无此依赖的纯数据研究。

## Gate C — M1真实PDF工作台（08）

- 条件：05实际PDFium测试/裁剪门、06自建Beamer夹具、Gate B、08 P0步骤I-1…I-5/I-7通过。
- 真实≥80物理页Beamer，含备注、overlay；≥30min汇报：当前/下一页/备注、受控当前页观众BGRA、Ready/Start/Stop计时、导航/跳页、黑白屏、交换、焦点、真实备注隔离。不以07占位代替。
- 性能/资源：05的4K报告、08按帧/像素/纹理计数与VRAM协议验收；未达到阈值记录阻塞或申请具体目标调整，不能直接写Implemented。
- 证据归属：05 raster/PoC2、06结构夹具、08M1/延迟/VRAM报告；父级traceability关联AC-3/5/7子集。角色暂由配置指定，AC-1最终UI路径未完成。
- M1通过不等于08已完整归档；其P1网格/布局/高级标注仍须在M2前完成。Gate C未通过不做10/11完整集成，可预先研究native依赖。

## Gate D — M2与父级最终集成（09/10/11 + 08余项）

- 条件：09真实角色/设置生效，10外部替换/失败回滚，11媒体/缺库启动/许可证，08全部P1通过；Gate A/B/C证据仍适用于当前代码。
- 逐项执行AC-1…AC-9，三屏设置UI、Identify、交换唯一绑定、保存失败/未来版本、跨代像素与媒体隔离、旧资源退役、安全只读写入审查。
- just ci、PDFium feature实测job、GPUI feature编译、媒体选定feature/job及Windows实机报告都需通过；默认CI绿不能替代native/GPU。
- 证据归属：09设置UI报告，10重载事务报告，11PoC4/依赖许可报告，08剩余增强验证；父级最终验收索引列命令/环境/代码状态/结果。
- 无硬件时只记待人工验证；不造三屏、4K、显存、媒体报告，不勾未测AC。

## 本次规划验证

1. 核对父children与各child.parent、status=planning；01 PRD-only，02–11各有design/implement。
2. 逐个运行 python ./.trellis/scripts/task.py validate <实际目录>（父+11子）。
3. 检查JSONL非空/每行合法/路径存在/无源码或Pympress实现目录；包含真实spec、父契约、子设计和必要研究。
4. 追踪表覆盖全部R/AC，检查Task Map/子依赖/顺序一致、无循环、每个跨任务接线有所有者。
5. git diff --check；确认只有规划/上下文路径改动，产品代码与task状态未变。记录到research/planning-validation.md。validate只验证上下文，不能证明设计正确或实现通过。

## 禁用能力扫描与未来收尾

01与最终AC-4运行：
rg -n -i 'latexmk|xelatex|FreeText|annotation edit|edit annotations' AGENTS.md docs README.md README.zh-CN.md crates
rg -n 'OpenOptions|File::create|write|SHADOW_TMP_SUFFIX|ferrostage.tmp' crates AGENTS.md docs .trellis/spec

rg退出1只代表无匹配；每处命中按baseline的“历史/排除/现行冲突/允许只读结构/写目标”分类并保留处置记录。评审目标是语义合规，不能靠删约束或将词替换为同义词制造零命中。

父级未来完成：所有子任务完整技术验收、AC-1…9、M2、spec沉淀与证据索引齐全；提交/归档需用户另行授权。当前下一可实施任务为01-docs-sync，本次不执行。
