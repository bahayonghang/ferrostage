# Gate B — current checkpoint

Latest V15: **INCOMPLETE**. F13已有真实DC红→绿、产品检查和串行独立审阅；真实TeX notes文件和document/PDFium裁剪/overlay验证PASS，07 UI仍placeholder。F14四文件AccessKit候选只在research，Root16个实际隐藏窗口销毁/64次COM查询通过，不算产品修复；第六维护包未授权采用。Root Computer Use窗口清点收到physical Escape停止，本轮未启动V15应用，V15原生项NOT_RUN。读execution-state文首及07 native-v15-checkpoint.md。尺寸用户暂缓、物理NOT_RUN、P2和08–11依赖边界不变。

Latest08:05Z: **INCOMPLETE**. 用户暂缓进一步尺寸验收，优先核心功能。V14新UI迁移/不播放计时冻结/no-play重启/冲突/Start-Stop/Audience关闭及两轮干净关闭有分项PASS，但长跑触发F13桌面DC泄漏，随后Close出现F14 AccessKit Invalid HWND panic；整体不能PASS。3个进程已退出，125项45原图封存；读07 native-v14-core-results.md及父execution-state文首。既有implement正处理真实回归与最小修复，之后串行check/native。未完成尺寸属于用户暂缓；物理NOT_RUN与P2/后续依赖边界不变。以下为历史检查点。

Latest06:53Z: **INCOMPLETE**. V14实现与产品检查完成（UI82、严格lint/typecheck/fmt/deny/native build，default150核验复用），已交既有gpui_check串行独立审阅。Root尚未启动V14 native。实际F12关闭与剩余dark尺寸/键盘状态按07 native-v14-plan继续；下面的V13失败保留，不把产品回归PASS当原生PASS。最新身份与34项证据清单见父execution-state文首和07 product-v14-implementation.md。物理NOT_RUN、P2、08–11依赖及不提交边界均不变。

## Retained V13 native checkpoint

Latest06:30Z: **INCOMPLETE**. V13独立检查完成：UI77（54 GPUI+23 pure）、严格lint/typecheck/fmt/native build PASS，未变default150/deny核验复用。Root五轮原生中main/dark/en/open实际关闭exit0、空窗口、干净日志；第五light-small的320x640面板关闭虽exit0，却在poll取消后报F12 vendor/gpui/src/window.rs:1943 window not found（hover回调）。总体关闭验收FAIL。所有Root app已退出，609输入/exe在06:31:45复核后释放给既有implement代理，进行V14回归与同生命周期回调审计，随后必须串行独立check、新freeze、原生复验。

V13历史source609=77aa180c7f60bcc39de7315554d9781d5f2f39effd211078a1da43cf7ed40d4a；EXE=d1164919dcdc09053df4fc6caae09be249329cccd0ad8bd482a67347561a6435。当前字节不能直接当V13。详见07 research/native-v13-checkpoint.md；92项清单/26原图SHA e08f7bb83832a0e6ccbd23b0ea5abb6a489181baad8ef85f27bc224475d4aedf。

新实测包括C初始未移动前完整标题栏与严格放置、新UI C/B→B/C、精确light1440x900/800x600/390x844/320x640及dark320x640、英文新面板、Audience关闭→重新开始→Presenting主窗关闭、原生chooser取消与成功导入。light320几何截图不覆盖该run关闭FAIL。V9 Identify与V11新UI迁移/不播放计时/真实锁Retry/未来只读/重启按独立审阅适用性复用，不标成V13同次运行。

剩余可执行：dark1440x900/800x600/精确390x844及有限键盘/控件状态（含窄窗Presenting/Stop），修复后继续，不列为外部阻塞。物理拔插/重连/接口/延迟及真实单屏/零屏NOT_RUN；live OS主题、跨窗口拖放、瞬态Curtain/loading和完整光标矩阵仍有限制/UNVERIFIED。Gate B INCOMPLETE，C/D NOT_RUN，08–11依赖受阻、P2不变。所有新改动未提交，不push/archive/add_session。

## Retained V11 checkpoint (historical)

Latest05:36Z: **INCOMPLETE**. V11 passes independent product checks (UI70/default150) and its bounded resize converges at C startup. NewUI actual Presenting migration/no-play/timer/save-lock Retry/relaunch/future readonly and native dark320x640 /389.6x844 are recorded in07 `research/native-v11-checkpoint.md`. Four completed runs exit0; main/relaunch/future have clean retirement logs. Dark run Close emits dispatcher.rs129 Invalid window handle (F11), so close acceptance FAIL despite exit0/empty inventory. Existing implementer works V12 regression repair, then serial check/new freeze. Initial C full-titlebar capture and remaining exact sizes/native matrix are still pending. Physical items remain NOT_RUN;08–11 gated, P2 unchanged.

## Retained V10/V9 checkpoint (historical)

Latest native checkpoint04:55Z: V10 passed independent source/check gates but failed actual C startup (F10). The existing GPUI resize produced1000x754 rather than requested1002.67x760; strict placement timed out. Root cleaned up ownedPID128376 (exit-1, final native inventory empty), then released source for V11 regression-backed resize correction. `07/research/native-v10-checkpoint.md` and its11-record manifest preserve this result. Do not label V10 native PASS or loosen size tolerance. Remaining new-interface acceptance is pending.

Current continuation (2026-10-15 America/New_York): **INCOMPLETE**. V8 implemented the reviewed shared Displays panel/Identify and regression-backed retirement repair. V9 independent review passed (63 feature tests, strict lint/typecheck/native build;148 default tests/full deny remain applicable). Root V9 native verified new-panel conflict/Cancel, separate matching Identify1/2/3 screenshots and actual A/C application placement. Full logs are clean and actual Alt+F4 exited0 with empty final inventory. Native F9 FAIL: the C replacement client width1280 exceeds its1066.67 logical display width; right titlebar clips. Root released the V9 freeze only after exit for scoped implement then check repair and a new freeze. See task07 `research/native-v9-checkpoint.md`, `native-v9-evidence-manifest.json` and `product-independent-v8-review.md`. Presenting/no-play/timer, new-panel save Retry/relaunch, future readonly and sizes/themes remain pending. Physical operations and actual single/zero-display hardware remain NOT_RUN under the user's instruction. Tasks08-11 remain dependency-gated; P2 and parent acceptance are unchanged. All new edits remain uncommitted.

## Retained V6 checkpoint (historical)

2026-10-15T02:51Z. **INCOMPLETE**; do not start08 real dual-window integration. Gate A technical PASS. V6 source/strict consumer lint/native build,49 actual GPUI regressions and145 default tests independently verified; unchanged product dependency policy remains passed. F7/F8 repairs also passed the targeted native retest, successful chooser import and saved-role relaunch. All controlled V6 app/lock processes exited0.

Evidence: [native v6 matrix](../../10-14-07-gpui-shell/research/native-v6-checkpoint.md), [native v6 receipt index](../../10-14-07-gpui-shell/research/native-v6-evidence-manifest.json), [retained native v5 matrix](../../10-14-07-gpui-shell/research/native-v5-checkpoint.md), [PoC history](../../10-14-07-gpui-shell/research/poc-1-report.md), [independent v6 review](../../10-14-07-gpui-shell/research/product-independent-v6-review.md), [v6 identity](../../10-14-07-gpui-shell/research/product-native-freeze-v6.json), [post-native documentation delta](../../10-14-07-gpui-shell/research/native-v6-closeout-identity.json).

- Observed: A/B start/stop/swap, C vertical fullscreen/white/stop, audience navigation, v5 focused Space Start and Enter Stop, Jump Backspace/commit/cancel isolation, Open refusal while Presenting, light/dark/system initial appearance, en-US/zh-CN labels, recent valid/malformed/missing imports, configured-no-audience disabled Start, saved roles on relaunch.
- V6 F7 repair PASS: Settings Close, focused Start/Stop, Jump Commit/Cancel and disappearing Retry all retain usable keyboard traversal after the action; Start→Tab is explicitly native evidence, not automated red/green coverage.
- V6 F8 repair PASS: actual Windows config read-lock causes one unsaved warning; runtime Swap preservesReady2 while disk bytes remain unchanged. After release, focused Retry clears warning/button, persists Aaudience/Bpresenter and restores Tab to Open. Relaunch confirms PresenterB actual display65731. Repository PDF SHA stays unchanged after native chooser import to Ready1/9.
- Partial/UNVERIFIED: drag/drop (cross-window drag not established with available CU API), exact full DESIGN logical-client size/state matrix (observed partial sizes only; unsuccessful resize attempts are not enforced-minimum evidence), transient loading/Curtain visual capture, live OS theme event, complete audience cursor behavior and framework retired-window diagnostics.07 pixels remain placeholders; no08 PDF/GPU evidence claimed.
- Physical unplug/replug/interface and latency, actual single/zero-display acceptance remain NOT_RUN. User already said physical cooperation unavailable; no repeated request or simulation substitute. This restriction persists even after executable checks pass.

Gate C/D remain NOT_RUN; M1/M2 and parent AC remain incomplete. No technical PASS implies commit/archive approval.

Environment: Windows11, actual A/B/C displays including a rotated C, NVIDIA RTX5090 Laptop GPU, Zed v1.23.3 GPUI (package version0.2.2) with the approved five-package maintenance patches and PDFium7881. The kit/component0.7.3 route was evaluated but not adopted. Exact MonitorIds/bounds/config/argv are retained with each run. `GdiDeviceCapsRatio=1.0` is not effective-DPI proof; the relaunch log reports GPUI displayB bounds2048x1152 and client1100x760. No fake event, disabled role or screenshot dimensions replace physical topology/DPI acceptance. Generic `window not found`/invalid-handle diagnostics persist around window retirement despite normal exit; their cause is not claimed fixed or harmless.

No new source edits followed the independent V6 freeze. Only the stale status paragraph in root DESIGN.md changed after all native processes exited; its original bytes and the exact delta are retained. This documentation correction does not justify another Cargo run or invalidate the unchanged compiled-code evidence.


## Display-assignment requirement added

User screenshot feedback makes07 G-13/I-8 minimal count/number/application/PDF assignment and Identify a Gate B requirement.09 later reuses it in full settings. Old config/Swap evidence remains historical only. This newUI is PLANNED; Gate B remains INCOMPLETE, with physical/native requirements unchanged. Current plan: display-assignment-plan.md. V7 diagnostic native source locations are recorded in07 research/resume-v7-native-checkpoint.md; no repair claimed.
