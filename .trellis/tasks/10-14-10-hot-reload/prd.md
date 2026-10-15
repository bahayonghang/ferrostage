# PDF 文件变更热重载（通用，不绑定 LaTeX 工具链）

> Current acceptance policy (2026-10-15 user revision): `.trellis/tasks/10-14-presenter-v0.3/acceptance-policy.md` supersedes older hardware/matrix blockers below. Implement all features first; retain automatic/feature/native-functional checks. Physical, exhaustive native/size and performance/soak matrices are deferred, not PASS. Gate B implementation is satisfied by V16;08 may start. Gate C/D are functional integration gates. Historical results and product contracts remain unchanged.

父任务：`10-14-presenter-v0.3`（R-44 热重载部分，R-01）。P2。复杂任务，设计和步骤见本目录design.md / implement.md。

## Goal

当已打开的 PDF 在磁盘上被外部程序替换（任何来源：导出工具、同步盘、复制覆盖）时，FerroStage 在文件写完后自动切换到新版本，不报错、不黑屏、保持当前逻辑页与计时。FerroStage 自身永不写该文件。

## Background

- 现状 `ferrostage-platform` 有 `HOT_RELOAD_DEBOUNCE`（300 ms）、`PDF_EOF_MARKER`、`has_pdf_eof`、`notify` 8.2.0 类型引用（`crates/ferrostage-platform/src/lib.rs`），监听循环未启动。
- 核心 `StageShadowDocument { new_generation, bytes }` 要求严格更新的代际且非空缓冲（`state.rs:181-207`）。
- `ADR-004` 记录无锁重载；`01-docs-sync` 会把其中的 LaTeX 语境删除。
- `has_pdf_eof` 当前扫描整个缓冲；大文件应只检查尾部。
- 设置：`hot_reload.enabled` / `hot_reload.debounce_ms`（`02`）。

## Requirements

- **H-1** `watch_pdf(path, debounce, sink)`：`notify` 监听父目录（应对"写临时文件再 rename"的替换方式），过滤目标文件名；事件合并后等待 `debounce` 的静默期。
- **H-2** 300ms静默，大小/mtime间隔采样稳定、尾1KiB EOF，再read+复核stat；EOF仅提示，结构/PDFium完整验证才提交。30s放弃debug无弹窗，旧画面继续。
- **H-3** IO worker读bytes关句柄；结构worker lopdf+06构建；唯一render worker PrepareCandidate仅PDFium影子加载/PageLabel/尺寸并返回metadata→06建图/定位/crop→PreRenderCandidate明确P0/P1请求→ready，对象不逃逸/另开线程。
- **H-4** active/candidate独立结果；候选P0/P1齐备，主线程最新attempt校验，同次core descriptor/document/visual提交，再worker Commit/Retire和UI/CPU回收。失败Abort旧代继续，重复/过时不提交。
- **H-5** 唯一原始label组+overlay ordinal优先；歧义/缺失旧物理号钳至新物理范围，取≤号的最大content物理页，无则首content（不作Vec下标）。阶段/计时不变，可映射笔画保留/删页丢弃告警，generation失效停媒体。
- **H-6** 只读保证：本 crate 与调用链对目标路径只使用 `File::open`/`fs::read`；增加单测/评审项确保无写路径（不变量 8）。
- **H-7** EOF只查尾部；移除SHADOW_TMP_SUFFIX及写PDF注释/测试，设置tmp由Store独立管理，不复用误导名称。
- **H-8** 设置关闭时不启动监听；运行中切换设置可启停。

## Acceptance Criteria

- [ ] `tests/hot_reload`：仅在独立新建临时目录复制夹具，用脚本把另一份 PDF 以 64 KiB 分块、每块间隔 50 ms 覆盖写入打开的路径，期间 FerroStage 0 次错误提示、0 次黑屏帧（观众快照代际只在最终完整写入后变化一次）。
- [ ] 用"写 `.tmp` 再 rename"方式替换文件，同样只触发一次重载。
- [ ] 覆盖为损坏文件（截断一半且无 `%%EOF`），30 s 内不重载、无弹窗；随后覆盖为完整文件，正常重载。
- [ ] 重载前在逻辑页 15（PageLabel "15"）、计时 03:21；重载后仍在 PageLabel "15"、计时继续。
- [ ] 旧L1/L2/L3/worker文档/candidate释放计数；连续替换/Close/短文档/重复label/提交失败回滚通过。
- [ ] `rg "OpenOptions|File::create|write" crates/ferrostage-platform crates/ferrostage-pdf` 无指向监听路径的命中；`just ci` 全绿。

## Dependencies

完整前置：01/04/05/06/07/08（实际目录见父Task Map）；父子关系不代表依赖。 完整接线需Gate C，可先行无硬件研究。

## Out of Scope

任何 LaTeX 工具链检测或集成；监听多个文件；网络路径的特殊处理。
