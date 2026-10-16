# Implement — 10

## Current implementation checkpoint — 2026-10-15

08/09 source implementation and independent retained automatic gates are complete. The user reconfirmed continuing all code first and consolidating remaining native desktop smoke after11; it remains NOT_RUN, not a dependency blocker under the current policy. Start existing10 at I-1. Review the actual candidate/upload/commit/obsolete-ack sequence before edits, including the concrete production content-mask route documented in design.md. Implement all H-1–H-8 and retain meaningful real notify/PDFium integration checks. All new changes stay uncommitted/unarchived. Earlier dependency-blocker paragraphs below are historical.

> Current acceptance policy (2026-10-15 user revision): `.trellis/tasks/10-14-presenter-v0.3/acceptance-policy.md` supersedes older hardware/matrix blockers below. Implement all features first; retain automatic/feature/native-functional checks. Physical, exhaustive native/size and performance/soak matrices are deferred, not PASS. Gate B implementation is satisfied by V16;08 may start. Gate C/D are functional integration gates. Historical results and product contracts remain unchanged.

1. **I-1 事务/依赖评审**：核04descriptor/06map/05active-candidate/08snapshot，审所有线程与Prepare/Commit/Abort/Retire ack、最新token/close，Gate C完整接线前通过。
2. **I-2 watcher/IO**：父目录过滤notify、300ms静默/100ms稳定stat/尾EOF/read后stat、timeout/取消；伪事件源测试不依赖真实30s，fixture只temp，删除SHADOW_TMP_SUFFIX写PDF描述。
3. **I-3 候选构建**：structure worker lopdf+06；render worker唯一PDFium Prepare返回metadata→06建图定位→PreRender明确requests→ready，测试候选不被current-only过滤、active继续服务、load坏/晚到/连续event Abort。
4. **I-4 定位/状态**：唯一label+overlay、重复/缺失/短文档content钳制、笔画存留/删页warning；计时/stage不变，Close/新Load token失效。
5. **I-5 主线程接线**：整组descriptor/document/visual原子发布，commit/retire ack、旧CPU/GPU/doc/candidate回收；注入提交前上传失败保旧代、发布后退役失败有界处理；09有界capability控件启用。
6. **I-6 集成夹具**：temp目标64KiB分块每50ms、同目录tmp rename、截断/错误EOF、随后修复、快速连续替换/删除重建；snapshot只最终完整提交且无混代/中间黑帧；实际Windows路径事件也测。
7. **I-7 审查/AC-8**：写API目标只fixturetemp，用户源hash前后相同；资源counts/ack、page/timer/stage完整报告，run命令，不把snapshot计数替代真实画面测试。

命令：cargo test -p ferrostage-platform -p ferrostage-core -p ferrostage-document -p ferrostage-render；cargo test -p ferrostage-pdf -p ferrostage-render --features pdfium；cargo build -p ferrostage-app --features gpui,pdfium；just ci。
纯事件模拟/nativefixture/真实Windows帧行为分层记录；未实测不称0黑帧实机通过。回滚候选不影响active；代码回滚必须跨platform/pdf/render/document/core/ui/app同范围。

## 2026-10-14 依赖审计 checkpoint

07/08交付及Gate C未通过；完整热重载接线未开始，P2仍在范围内。 本任务保持planning，未start、未写本任务产品代码、未运行本任务验收。现有规划保留；恢复前先核父execution-state.md与07依赖门报告。03物理拔插/换接口本轮不可用，不能用模拟输入替代Gate B。无提交或归档。


## Resume audit - 2026-10-15

I-1/full transaction integration remains blocked by08 and Gate C. Existing05 candidate-worker APIs are delivered, but the watcher and end-to-end publication/retirement flow remain unimplemented. P2 scope is retained. See parent `research/resume-2026-10-15.md` and07 `research/resume-2026-10-15-v16-review.md` for current-code evidence and exact remaining gates. Task status and acceptance checkboxes are unchanged. No commit, push or archive.

## Implementation milestone — 2026-10-15

- I-1 reviewed with parent: opaque reversible worker commit, pure snapshot staging, real paint upload acknowledgement, latest-core mapping and explicit cancellation settlement. Contract recorded in design.md.
- I-2 implementation and real parent-directory notify regression PASS, including chunks, rename, delete/recreate, unrelated paths, EOF tail and simulated timeout. Source PDF never written.
- I-3–I-5 cross-layer source is connected; first gpui,pdfium feature check PASS. Existing108 UI regressions PASS. New real PDFium/two-window TestPlatform test covers stale Commit acknowledgement rollback/rebuild and latest pause/blank/ink preservation. Real paint test verifies atlas insertion/error and no candidate sprite enters the scene.
- I-6/I-7 additional failure/target/settings tests and full retained strict gates remain in progress. Desktop/hardware/performance matrices remain DEFERRED_BY_USER/NOT_RUN. No commit/archive.

## Implementation complete, independent check pending — 2026-10-15T15:01Z

I-1–I-7/H-1–H-8 product implementation and retained automatic gates are complete.112 parent UI tests plus isolated native child PASS;39 native-suite tests PASS;167 default CI tests PASS;all-features strict clippy/deny PASS. Final gpui,pdfium build follows CI. See research/implementation-report.md for exact APIs, resources, failed diagnostic attempts, commands, binary SHA and deferred boundary. Source released for independent review; no task archive/commit. Raw mixed native/matrix ACs remain unchecked.
