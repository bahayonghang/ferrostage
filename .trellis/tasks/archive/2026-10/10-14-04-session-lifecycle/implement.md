# Implement — 04

1. **I-1 契约评审**：读父design §2/6与behavior证据；列合法/非法边、timer展示、preview和swap所有权；核01已同步旧spec，不引入OS。
2. **I-2 回归**：先以受控now测试Load→Ready/Curtain、Start、Stop/Close、无Audience拒绝；添加当前preview泄漏能失败的测试，避免镜像实现断言。
3. **I-3 状态/时间**：idle/newReady/apply_at/elapsed_at、Pause/Resume/Reset；Start5s→Pause5s→Resume2s→Stop7s、Ready冻结、再次Start0，非法命令状态不变。
4. **I-4 投影/frame**：新增stage、BgraFrame/AudienceVisual受控纯类型；0/溢出/错stride拒绝，Curtain无visual；document投影测试用secret notes确认不复制，review签名不靠Debug文本单独证明。
5. **I-5 预览/swap/descriptor**：分离preview/committed页、首尾钳制、Commit/Cancel；移除独立swap flag，声明app intent；纯DocumentDescriptor与strict generation，不存PDFbytes。
6. **I-6 全消费者/属性**：更新core name/error及document；随机命令序列stage不为Presenting⇒Curtain/无visual，preview不改audience，Start失败原状态不变；保留现有generation/crop性质测试。
7. **I-7 Gate A**：运行命令并检查core依赖/no clock read/no notes；记录API变更给05/06/07/08/10/11。真实窗口/像素隔离留07/08。

命令：cargo test -p ferrostage-core -p ferrostage-document；cargo tree -p ferrostage-core；just ci。
回滚点：I-3状态API、I-4snapshot与document同批回滚，I-5旧bytes桥接由10最终收敛，不能留下两套提交权威。本轮 I-1…I-7 已实施并通过自动检查和独立审查；真实窗口/像素验收由07/08负责。

## Execution evidence
- I-1 reviewed shared lifecycle/timer/preview/swap/descriptor contracts and current callers; no production caller yet uses PresentationState.
- I-2 reproduced: `cargo test -p ferrostage-core preview_` returned two expected regression failures and one existing PASS; see `research/preview-red.log`. First sandbox attempt could not access the Cargo build lock; approved escalation executed the test.

- I-3/I-4/I-5 implemented; I-6 tests added. I-7 targeted check PASS: `cargo test -p ferrostage-core -p ferrostage-document` (21 core + 1 document), `cargo tree -p ferrostage-core` (only thiserror/tracing production dependencies), formatting PASS. Reports: `research/core-document-tests.log`, `research/core-tree.log`.
- I-7 PASS: `just ci` exit 0 (fmt, clippy with `-D warnings`, 70 workspace tests, cargo deny); `research/just-ci.log`. Existing duplicate `syn`/`windows-sys` warnings are non-failing. `git diff --check` PASS. Core production source scan found no clock sampling, file/thread IO, old `apply` calls or swap flag.
- Implementation, automated checks and independent Trellis check complete. Parent Gate A still needs task 03 topology delivery. Real windows, PDF pixels, multi-monitor, GPU and media acceptance are NOT_RUN here and remain downstream scope. No commit, push, archive or journal performed.
- Current source evidence: `research/verified-code-sha256.json` (workspace Rust/manifests SHA-256, branch/HEAD/time), `research/worktree-status.txt`, `research/verification.md`.

Last verification record (UTC): 2026-10-14T14:45:09.302752+00:00.

## Independent check

- PASS (2026-10-14 14:49:28 UTC): all 51 recorded Rust/manifest hashes match the working tree, with no missing or extra code inputs; branch and HEAD match the implementation evidence. Lifecycle/timer/preview/descriptor and document projection paths reviewed. No implementation defects found. The L-4 accessor spelling was synchronized to `elapsed_at(now)`; no product source changes were needed.
- Independent formatting and Clippy/type-check PASS. Initial sandbox build-lock denial was recovered by approved escalation. Existing 21+1 targeted tests, 70-test full CI and license evidence remain applicable to unchanged code; no broad tests repeated. `git diff --check` and task context validation PASS. Full report: `research/independent-check.md`.
- Task 04 is implementation-verified and independently checked, awaiting user review/authorized commit/archive. It does not block selecting the next dependency-ready task. Gate A still requires 03, and downstream hardware acceptance remains NOT_RUN.
