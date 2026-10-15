# 下一对话执行 Prompt

工作目录：`D:\Documents\Code\Rust\Exp\ferrostage`
目标父任务：`10-14-presenter-v0.3`
当前实施任务：`10-14-07-gpui-shell`

交付基线：产品/规范/真实notes夹具提交 `b44546946fc6241c0896663b39b588e7fd4ef702`；任务证据及本Prompt在随后一笔chore提交。实查git log，不假定HEAD仍是旧3ac1ec2。01/02/04/05/06已归档，本轮无新增可归档任务。

请继续现有任务树，先使用 trellis-start 核对磁盘、Git、task.py current/list、所有未提交改动及仍在运行的进程/代理，从首个未完成步骤恢复。不要重建任务、覆盖现有改动或重新确认已审显示器规划。

## 先读这些当前材料

1. 父 `execution-state.md` 文首、`research/gate-b.md`。
2. 父 `research/display-assignment-plan.md` / `display-assignment-review.md`：已获实施授权的契约。计划内“新UI未实施”是当时状态，当前实现与验收以 execution-state 和下列 V15 材料为准。
3. 07 `research/product-v15-implementation.md`、`product-independent-v15-review.md`、`product-native-freeze-v15.json`。
4. 07 `research/native-v15-checkpoint.md`、`native-v15-plan.md`、`native-v14-core-results.md`。
5. 07 `research/product-v15-a11y-proposal.md`、`product-v15-accesskit-proposal.patch`、`product-v15-root-a11y-results.md`，以及 `docs/adr/ADR-008-gpui-dependency-gate.md`。
6. `fixtures/notes-functional/README.md`、`pympress-notes.tex`、`verification.json`。

长 design/implement 曾超过上下文注入上限，必须显式读取最新报告和 implement.md 尾部，不能只靠截断注入。旧 V6–V14 截图、日志及 manifest 保留原版本身份，不能把当前 EXE 当旧版本，也不能把历史 FAIL 改为 PASS。

## 已完成与当前边界

- 01、02、04、05、06 已提交并归档；不要重新创建或实施。
- 03、07 仍 in_progress；08、09、10、11 尚待其依赖。父树仍5/11完成，本次提交不代表03/07完成或Gate B通过。
- 07已实现主要“显示器设置”入口、完整屏幕检测/编号列表、Identify、应用屏/PDF屏草稿选择、Apply/Cancel、MonitorId持久化、拓扑重验、冲突、未来版本只读、回滚和保存重试；既有Swap快捷键保留。不要重做这些功能或另建编号/角色/窗口事务。
- Presenting有效目标迁移保留演示阶段/页/运行计时；“不播放”必须安全停止投影、关闭Audience、保留当前页并冻结计时。V14已有分项原生证据；旧配置或Swap不能代替新界面验收。
- V15修复F13桌面DC泄漏，真实64次枚举回归从GDI2→66变为1→1，三屏完整MonitorInfo一致。CI151、UI82、真实PDFium2、严格lint/typecheck、完整deny和原生构建通过，串行独立审阅通过。当前GPUI DPI上下文/持续轮询/真实窗口仍需原生验证。
- F14 AccessKit直接COM查询在窗口销毁后可能panic，产品仍未修复。研究候选限定四个文件，6项真实COM/正负控和Root16个自有隐藏HWND销毁后的64次COM查询通过，独立审阅无阻断项；不是并发/句柄复用或产品关闭PASS。
- V15 source622聚合 `fdc00fc36ad02e46234398899ef924892e690810caf24f290cc5b0ec070d835a`；EXE `d40bb0d113e363df6f2bae5d331fd13d8473a8fbcf5820e4438b3da5489e8fab`；freeze SHA `0d6a9c2a7bbb10fe6f6c7d24e7c4f7faf10340d456fd5185fdfe631acdc4370f`。恢复时实查；若字节变化，记录差异并建立新身份，不覆写旧冻结记录。

## 立即恢复的工作

本 Prompt 明确要求恢复 Computer Use 原生验收。上轮 Sky 在启动V15之前收到物理Escape，Root已停止，V15产品原生保持NOT_RUN；不把这次停止当成新的产品FAIL。主会话独占桌面操作，子代理不得操作桌面。先核对没有旧app/build运行，再使用当前已审冻结版本；若工具再次报告用户停止，立即停止输入。

只关注核心功能，不再做宽度/尺寸调整矩阵。使用新真实notes PDF验证显示器列表与实屏对应、Apply/Cancel、角色放置、Start/Stop、页/overlay导航、Presenting迁移/不播放、Audience关闭与Presenter关闭；保存原始截图、实际窗口放置、完整stdout/stderr、配置读回、EXE/PDF/DLL及源码身份。关闭测试设置RUST_BACKTRACE=full；exit0或空窗口不能掩盖panic。

F14候选采用将新增第六个维护包，超出ADR-008已批准的五包范围。本次“提交现有改动”只保存研究证据，不授权接入第六包。先根据已经完成的独立审阅，展示准确四文件补丁及维护范围，只就新增AccessKit维护范围请求一次明确授权，不重新询问显示器规划。无新增license例外、无依赖升级、无关闭A11y或屏蔽所有错误。未获采用授权时可继续不依赖该补丁的V15核心验证，但不能将候选接入产品。获批后按 implement→check→新冻结→Root原生复验推进，保留旧失败证据。

## 必须使用的非空PDF

`fixtures/notes-functional/pympress-notes.pdf`，SHA `e939e35f931c184fd93b2e9b95e560db73ba6a6f6dfccbb0963262f0b359dfeb`。由同目录TeX使用XeLaTeX编译，包含Pympress简介、标准Beamer右侧notes、中文备注、私有校验词和两步overlay；5物理页/4逻辑frame，标签1/2/3/3/4。源码内显式固定物理标签的原因见README；未后处理PDF。

编译、全部5页视觉检查、分区文本与真实document/PDFium裁剪/overlay测试已PASS。07仍为页码占位UI，不能据此宣称讲者notes已可用。08必须真正显示当前页/下一页/notes并实测观众只见正文，使用该PDF检查03A/03B切换和私有词隔离；不要用raster.pdf或空白输入替代。

## 串行实施和后续顺序

按项目auto模式复用现有代理或按需要调度 trellis-implement → trellis-check。子代理Prompt首行必须是：
`Active task: <task.py current返回的实际路径>`
明确文件所有权、不是唯一工作者、保留其他人的改动；主会话负责协调、规范、提交边界和原生验收。优先原生上下文注入，缺失时由子代理按jsonl→prd→design→implement补读。不要创建新的用户会话来替代子代理。

Gate B实际通过后连续推进08；08复用07显示器入口，09再整合完整设置页，不形成07依赖09。按既有依赖继续09、10、11；Gate C未通过不做10/11完整集成，不移出P2。物理拔插、重连、换接口、真实单屏/零屏等人工项目本轮不安排、不反复询问，未执行保持NOT_RUN；缺失必需证据时Gate仍INCOMPLETE，不降低标准。继续所有不依赖这些项目的工作，直到授权范围完成、确实存在无法继续的外部阻塞或用户要求停止。

持续更新父execution-state、子implement和验收记录，明确完成/FAIL/NOT_RUN。本续接之后产生的新改动留未提交供审阅；未经新的明确指令不自动提交、推送、归档，不调用自动提交的add_session。当前已完成的提交/归档情况以本地git log和task列表实查为准。
