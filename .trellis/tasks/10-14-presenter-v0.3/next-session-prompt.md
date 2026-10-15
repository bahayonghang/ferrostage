# 下一对话执行 Prompt

工作目录：`D:\Documents\Code\Rust\Exp\ferrostage`
目标父任务：`10-14-presenter-v0.3`
恢复索引：父任务实际目录中的 `execution-state.md`。

请沿用现有 Trellis 任务树继续完成剩余实施和验证，不从01重新开始，不创建另一棵任务树。本 Prompt 授权在规划审阅、依赖及前置 Gate 满足后启动各子任务并实施；按依赖连续推进，不在一个子任务完成后结束整个请求。后续新改动留未提交供我审阅，不自动提交、推送、归档或调用会自动提交的 add_session。

先使用 trellis-start，读取 AGENTS.md、RTK.md、workflow 和相关 spec index；核对分支、HEAD、status、diff、task.py current/list、已有会话和仍在运行的构建/测试/代理。current为空不代表尚未开始。核对实际磁盘与证据，再选择首个未完成步骤；先给简短恢复概览，然后执行。

已完成的收尾：产品提交841e287、任务证据提交f246a67；01-docs-sync、02-settings-schema、04-session-lifecycle、05-pdf-raster、06-beamer-document-model已归档到 `.trellis/tasks/archive/2026-10/`。还存在后续归档、上下文与日志提交，当前HEAD必须实时读取。按任务ID搜索活动和archive目录；不得重建、重新实施已归档任务，只在新改动影响契约时回归。历史研究报告和hash清单保留当时路径，先用父 `research/archive-relocation.json` 解析实际位置，不改写旧证据制造一致。

当前实施状态：03-monitor-topology和07-gpui-shell仍in_progress；08–11仍planning；父任务仅协调。Gate A技术通过，Gate B不完整，Gate C/D未运行，父AC-1…AC-10及M1/M2未完成。

07的V6焦点恢复、键盘激活及保存错误反馈修复已完成：49项GPUI自动回归、145项默认测试、严格all-feature/all-target consumer clippy、native build及依赖检查通过；原生模态/Start/Stop/Jump/Retry焦点、实际配置锁失败→重试、选择器导入、角色保存重启和正常退出已有独立证据。优先阅读07 `research/native-v6-independent-closeout.md`、`native-v6-checkpoint.md`、`product-independent-v6-review.md`、对应清单和freeze，不重复修复这些问题。Start→Tab是原生证据，不是自动red→green。V6 exe SHA256为 `d4f2e653ee793176b384034797ad99e1ad665aa0da6535a454ba88a36b055074`；原602输入中仅DESIGN.md第151行在原生退出后同步进度，全部94个crate文件未变，差异记录在native-v6-closeout-identity.json。记录中的旧HEAD与当前提交HEAD不同不等于代码变化，核对实际内容。

先恢复03/07未完成项：物理拔插/重新连接/换接口、身份稳定性及≤1s延迟；真实单屏/零屏与windowed_audience；完整logical-client尺寸、DPI与控件状态矩阵；拖放、live OS主题事件、观众光标和Curtain过渡；调查仍出现的window not found/invalid-window-handle退役诊断。已有A/B放置来自V4，C来自V5，不冒充V6同次实测。上一轮用户无法配合物理操作；若没有新的硬件条件或证据，保持NOT_RUN/待人工，不能用模拟输入、静态枚举或禁用Audience配置替代。需要人工操作时先准备精确步骤；无法完成时保存阻塞，不绕Gate。

原生检查使用computer-use技能和支持的API，先核对并复用真实窗口，保留截图/AX/日志/配置读回及代码身份；所有UI由主会话串行控制，确认上一进程实际退出后再启动下一次。遇到用户停止Computer Use立即停止。不要把截图外框尺寸当logical-client尺寸，不把正常退出当退役诊断已解决。

Gate B通过后按08-presenter-workbench → 09-settings-ui → 10-hot-reload → 11-media-playback推进；Gate C未通过不做10/11完整集成。不得降低验收或移出P2。每个子任务先读PRD/design/implement、上下文和父契约，用task.py start激活；按本项目auto模式串行调度implement→check，首行必须是 `Active task: <task.py current返回路径>`，明确归属并保留其他修改，不同时实施两个子任务。主会话协调规划、spec和实机验收。

继续遵循根DESIGN.md的Claude主题及07共享token；08真实PDF/备注/计时/GPU工作台和09最终设置UI仍未交付，不能把07占位界面当完成。用户已批准五个本地维护包和仅三个精确许可证例外：arrayref@0.3.9/BSD-2-Clause、libbz2-rs-sys@0.2.5/bzip2-1.0.6、cbindgen@0.28.0/MPL-2.0；采用Zed v1.23.3源路线。六项上游HarfRust直接lint失败保留，不增加豁免或扩大许可政策。

每个步骤、检查结果、任务切换和阻塞都增量更新子implement.md及父execution-state.md。缺陷先建立回归，再修复；运行必要的crate检查、just ci和native/GPU门禁。已有证据仍适用于当前代码时复用，validate只证明上下文有效。最后按真实结果报告各任务、Gate A–D、父AC、人工待验和未提交改动；只有全部工作完成、确实外部阻塞或我要求停止时结束。
