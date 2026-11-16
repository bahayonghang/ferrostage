# 下一对话续接 Prompt

工作目录：`D:\Documents\Code\Rust\Exp\ferrostage`

请先使用 trellis-start，核对 Git HEAD/status、task.py current/list、实际任务目录及以下文件：
- `.trellis/tasks/10-14-presenter-v0.3/execution-state.md`
- 同目录 `acceptance-policy.md`
- `research/implementation-closeout.md` 与 `research/archive-08-11-map.json`
- `research/consolidated-functional-report.md`、`research/quick-native-diagnosis.md`

01–11 已按修订后的实施范围完成并归档（08–11本次归档），不要重新实施、重建任务树或把旧Gate B阻塞结论当作当前门禁。父任务仍进行中，因为原始综合AC和剩余功能观察尚未全部验收。归档不代表全部实机、矩阵、性能验收通过。

从父任务剩余清单中首个未完成且未被延期的步骤继续。先核对实际代码与有效证据，区分“已有实现、尚未观察”和“真实缺陷”；不要凭未勾选AC编造缺功能。按项目auto模式串行使用Trellis实施→独立检查；只修复确认的问题，必要时创建聚焦后续修复子任务，不重开01–11的完整实施流程。

优先顺序：
1. 复核工作台网格/缩略图与媒体控制的滚动可达性。当前默认窗口首屏有内容位于滚动区下方，但代码允许滚动；尚未确认Grid事件误路由或暂停清空帧，不能把推测写成已确认Bug。
2. 补齐有意义的功能检查：网格预览/提交/取消、清空笔画、计时恢复/重置、文档选项；设置实际消费者及只读/保存失败恢复；真实分块/损坏PDF替换与恢复；WebP连续帧、Beamer Movie/内嵌媒体/裁剪隐私与生命周期、缺整个运行库降级。按现有真实自动测试覆盖去重，不重复运行已通过且源码未变的检查。
3. 实际缺陷先复现，再最小修复、回归和严格检查；同步父任务进度及证据。

上一轮我正在做其他工作，因此优先后台代码/证据检查，不自动抢占桌面。需要真实桌面补验时集中安排短时核心冒烟，先确认当前可以进行，再使用Computer Use；不要连续展开完整矩阵。已有四次核心native运行正常退出，之前分块写入producer未产生实际写入，必须保留NOT_RUN；网格NOT_VERIFIED、WebP桌面PARTIAL，不能冒充PASS。

物理拔插及延迟、真实单屏/零屏、完整原生矩阵、尺寸/DPI/主题矩阵、长时性能与显存测试继续DEFERRED_BY_USER/NOT_RUN，不作为实施门禁。不得删除功能、用占位实现代替，或放宽只读PDF、备注隔离、单线程PDFium、代际/资源退役与许可证约束。

保留有意义的自动回归、严格lint/typecheck、feature编译、cargo deny、just ci及必要真实功能检查。最新独立结果：180默认CI、119父UI测试加1次隔离重放、40PDFium、17媒体测试；产品源码169项身份已复核。改源码后按影响重新验证，不直接复用旧二进制的native结果。运行GUI必须在默认CI之后重新构建gpui,pdfium,media，核对最终GUI/helper/PDFium/runtime身份。

归档研究报告中的旧路径和哈希是当时证据，按archive映射解析，不改写封存字节。任务上下文validate仅证明路径有效，不证明产品验收。持续更新父任务进度；本次之后的新改动保留未提交供审阅，不提交、不推送、不归档，除非我再次授权。
