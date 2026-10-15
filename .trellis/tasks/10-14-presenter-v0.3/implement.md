# Implement — 启动顺序与集成评审

2026-10-14 已获连续实施授权。每次恢复先审计 execution-state.md、实际代码和证据；子任务规划/依赖审阅后使用 task.py start，按 implement→check 串行推进。父任务不启动；01可PRD-only。所有代码、文档与记录留未提交，不推送、不归档。

## 依赖顺序

可行串行顺序：01 → 02 → 04 → 03 → 05 → 06 → 07 → 08 → 09 → 10 → 11。
理论依赖就绪集合（本轮按用户要求串行实施，不同时启动多个实施子代理）：01后02/04；02后03与04后05；05/04后06；02/03/04/05后07；08后09与满足Gate C的10；09后11可与10独立推进。并行工作不得同时定义共享类型，创建者见design §1。

01先同步契约；02/03/04通过Gate A。07通过Gate B才进入08窗口集成；05/06纯逻辑研究可在此之前推进。08的P0交付通过Gate C，09以及10/11分别按PRD依赖推进。Gate C不以P1全部完成为先决，但M2前补完08全部P1。技术验收不要求先归档；归档只在完整子任务验收及后续收尾授权后进行。

## Gate A — 纯契约（02/03/04）

- 条件：02/03/04交付可用；运行 cargo test -p ferrostage-core -p ferrostage-settings -p ferrostage-platform；core依赖树无IO/OS/GPUI/PDFium/lopdf/notify。
- 覆盖：时间受控Start/Pause/Stop/Restart、Curtain、preview不影响audience、零/一/二/三屏、重复/缺席/退化身份、nested未知设置/非法类型/未来版本/原子失败。
- 证据归属：02的设置测试及Windows替换记录，03的枚举/ID稳定性记录，04的状态/快照测试；父级索引关联命令、HEAD与dirty范围。无三屏数据时纯契约可通过，但03实机验收仍待验，不宣称整个03完成。

## Gate B — 外壳与拓扑 PoC（07）

- 条件：07及其02/03/04/05前置交付可用、Gate A纯契约通过。05提供只读导入元数据；窗口展示占位即可，不验真实PDF像素或备注面板。
- 三屏Windows11：07显示器设置列出数量/编号并Identify，通过应用屏/PDF屏选择并Apply指定A Audience/B Presenter/C None；打开PDF→Ready无观众/不计时；Start→A占位全屏；Stop→关窗；用同一面板改选C Audience再复测；拔Audience≤1s安全降级。焦点在观众屏触发动作，弹UI必回讲者。单屏两模式/零屏/歧义拒绝测试。
- 证据归属：07 research/poc-1-report.md（已记录实际启动失败及修复恢复点，纳入07 check上下文），含设备、DPI、操作/截图、结果、代码版本。GPUI API研究与编译门另记在07；旧配置文件/Swap证据不能代替新增分配UI；07最小面板不代表09完整设置页可用。
- 阻塞范围：未通过实机拓扑/focus门不进入08真实双窗集成；可继续无此依赖的纯数据研究。

## Gate C — M1真实PDF工作台（08）

- 条件：05实际PDFium测试/裁剪门、06自建Beamer夹具、Gate B、08 P0步骤I-1…I-5/I-7通过。
- 真实≥80物理页Beamer，含备注、overlay；≥30min汇报：当前/下一页/备注、受控当前页观众BGRA、Ready/Start/Stop计时、导航/跳页、黑白屏、显示器分配与快捷键兼容、焦点、真实备注隔离。不以07占位代替。
- 性能/资源：05的4K报告、08按帧/像素/纹理计数与VRAM协议验收；未达到阈值记录阻塞或申请具体目标调整，不能直接写Implemented。
- 证据归属：05 raster/PoC2、06结构夹具、08M1/延迟/VRAM报告；父级traceability关联AC-3/5/7子集。角色由07共享面板指定，AC-1的09完整设置与真实PDF组合仍须最终验证。
- M1通过不等于08已完整归档；其P1网格/布局/高级标注仍须在M2前完成。Gate C未通过不做10/11完整集成，可预先研究native依赖。

## Gate D — M2与父级最终集成（09/10/11 + 08余项）

- 条件：09真实角色/设置生效，10外部替换/失败回滚，11媒体/缺库启动/许可证，08全部P1通过；Gate A/B/C证据仍适用于当前代码。
- 逐项执行AC-1…AC-10，三屏数量/编号、两项角色选择、Identify、唯一绑定与旧交换快捷键兼容、保存失败/未来版本、跨代像素与媒体隔离、旧资源退役、安全只读写入审查。
- just ci、PDFium feature实测job、GPUI feature编译、媒体选定feature/job及Windows实机报告都需通过；默认CI绿不能替代native/GPU。
- 证据归属：09设置UI报告，10重载事务报告，11PoC4/依赖许可报告，08剩余增强验证；父级最终验收索引列命令/环境/代码状态/结果。
- 无硬件时只记待人工验证；不造三屏、4K、显存、媒体报告，不勾未测AC。

## 规划基线验证（初次规划历史协议；当前执行审计另记）

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

父级未来完成：所有子任务完整技术验收、AC-1…10、M2、spec沉淀与证据索引齐全；提交/归档需用户另行授权。实际恢复点按 execution-state.md：01/02/04/05/06技术交付已独立复核，03自动与静态检查通过但物理验收待验；07产品依赖/编译与自动检查已通过；首次启动/关闭/输入再入panic已修复；v6独立49项GPUI/145项默认测试通过；实际A/B/C、Jump隔离、主题/语言、错误导入已部分验证，F7焦点恢复/F8保存重试反馈已修复并通过原生复测，选择器导入与保存角色重启通过；08–11依赖未满足。不从01重做；docs/adr/ADR-008-gpui-dependency-gate.md 的受限维护评估已获用户批准，隔离候选及独立检查已完成，实际修复后入口为07 research/maintenance-independent-prepare-final.py；最终五包与三个精确许可例外已按research/gpui-license-review.md获明确批准，实际产品I-2门已通过，继续07 native与独立审查，直接HarfRust既存lint失败及native限制如实保留。

## Claude 主题实施验收

07 I-2/I-3建立 DESIGN.md 单一Rust主题层、系统主题入口和基础状态；08 I-2/I-6复用token组合工作台；09 I-2/I-5接入主题设置与系统变化。三任务分别记录双主题截图、尺寸、DPI、焦点及error/disabled/hover/loading，按DESIGN.md阈值核对。Gate B仍阻塞08窗口集成；主题规划不绕过该门，不用浏览器截图冒充原生验收。AC-10在Gate D汇总。


### 2026-10-15T03:03:18.786383+00:00 — 独立原生证据复核结束 / 本轮外部阻塞保存
- 新独立报告：07 `research/native-v6-independent-closeout.md`。限定V6 F7/F8、选择器、保存角色重启及正常关闭证据成立；3项PRD勾选对应原V4 A/B、V5 C与当前only-snapshot边界，不冒充同次V6全矩阵。
- 独立清单 `native-v6-independent-manifest.json` 213/213匹配，SHA256 `0aec8d0e6f68591adcc695f3cd89fdfc29efb601f0bcda575b0e47c66910fb5e`；54/108/26/141既有清单也分别匹配。V6 native结束后只有DESIGN第151行状态更新，全部94crate及可执行文件/DLL/fixture未变，复用49GPUI/145default、lint/build/deny证据。
- 父PRD/design/traceability、feature-parity及PROJECT_PLAN已校正过期I-2状态和采用版本；13份当前文档9个本地链接检查通过，2个历史代理UUID伪链接已改为标识文本。初次失败与修正后报告均保留于父research/native-v6-document-closeout.json和-v2.json。12/12任务上下文validate仍适用（上下文清单无后续改动）；不等于产品验收。
- 所有受控V6 app/锁助手及实施/check代理均已返回；没有未完成命令或活跃native进程。本会话current仍07；父planning、01–07 in_progress、08–11 planning，未手改状态或启动受阻任务。
- Gate A技术PASS；Gate B INCOMPLETE；C/D NOT_RUN，父AC-1…10、M1/M2未完成。03物理项和07剩余原生矩阵/退役窗口诊断保留待验，08–11依赖受阻；用户本轮无法物理配合，不重复询问或绕过Gate。恢复从这些未完成验收继续，不重做已验证实现。
- 分支main、HEAD ab78a84d96c7cf4a0bbc8dfd4f8e45da180bafb3；全部代码、文档、task与证据保留未提交。无commit、push、archive或add_session。

### 2026-10-15T03:16:37.932220+00:00 — 归档后检查与续接准备完成

五项归档已由task.py完成；14份运行用上下文JSONL已迁移到实际archive路径，父+全部11子任务共12/12 validate通过（research/post-archive-context-validation.json）。研究原始内容与历史hash不重写，目录映射见archive-relocation.json。补充保留此前被通用*.log忽略的125份验收日志（retained-acceptance-logs.json），采用仅覆盖任务research的ignore例外，未强制加入缓存。native54份证据此前已全部跟踪。另仅对冻结清单中被上游规则忽略的vendor/cosmic-text/Cargo.lock执行精确git add -f，文件字节不变，未改写上游ignore规则。当前602源码与Git index双层绑定见committed-source-binding.json；Git LFS指针与文件字节按上游pointer SHA/size验证。

下一轮可直接使用next-session-prompt.md；03/07继续待验，08–11保持依赖门禁。随后提交上下文/续接记录并按trellis-finish-work记录一次本次已授权收尾日志；不推送。


## Current planning correction

Before completing07/Gate B, execute07 I-8: replace the toolbar Swap workflow with detected/numbered display assignment. Review the bounded `research/display-assignment-plan.md`, then perform the existing serial implement→check flow. Keep the task tree and archived01/02/04/05/06 intact.08 reuses this surface;09 integrates the complete settings page later, so no dependency cycle is introduced. The present correction changes planning only; V7 diagnostics are a separate uncommitted change already checked before this request.
