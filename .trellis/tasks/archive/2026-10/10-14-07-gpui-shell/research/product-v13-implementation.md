# V13 — reviewer self-fix for pending import/chooser shutdown

Active task: `.trellis/tasks/10-14-07-gpui-shell`. This is the explicitly authorized trellis-check self-fix following V12 review, not a new implementation task. Full reasoning, native evidence applicability and PRD assessment are in `product-independent-v12-review.md`.

V12 owns/cancels its poller but leaves import-read and chooser foreground awaiters detached. Their weak/token checks run after a dispatch. Controlled pending read plus real GPUI Presenter close/App shutdown reproduced two failures; actual TestPlatform chooser response after App shutdown reproduced a third. The positive nonowner-close completion case already passed. All four now pass. Red exit101 and green exit0 receipts remain separate.

Shell retains `import_read_task` and `path_prompt_task`, and `stop_tasks` drops them together with the poller before Presenter quit and in the synchronous quit observer. The existing read continuation is isolated as `await_import_read` for controlled delivery tests; production still uses background `std::fs::read`. Existing result/token/owner checks remain. Completed tasks need no eager release; replacing or dropping their owner releases them. No projection, transaction, settings, geometry, renderer, dependency or vendor policy changed. Root extended the component contract.

Cancellation protects foreground continuation lifetime; it cannot interrupt a synchronous filesystem read already running or synchronously destroy a platform-native dialog. Pinned platform owner/cancellation coordination remains in place. Native F11 HRESULT absence still requires Root's new-build retest.

Final UI77 (54 actual GPUI+23 pure), strict all-feature/all-target lint/typecheck, format/diff and native `gpui,pdfium` build pass. Unchanged V12 default150/CI and all-feature deny receipts are explicitly reused because their compiled source/dependency inputs are unchanged; affected GPUI checks were rerun. No extra policy exception or warning suppression was added.

Frozen609 aggregate `77aa180c7f60bcc39de7315554d9781d5f2f39effd211078a1da43cf7ed40d4a`; EXE `d1164919dcdc09053df4fc6caae09be249329cccd0ad8bd482a67347561a6435`; freeze SHA256 `d3689820edf788e4e55d485b644f58007c4616e6ca49262de9aa057d5e86fb91`. Only Shell and Root's component contract differ from V12;607 inputs remain unchanged. The native runner may use `product-native-freeze-v13.json` with the existing isolated inputs.

V9 Identify and V11 new-panel role/migration/save/relaunch observations remain historical functional evidence across this lifecycle delta; they are not V13 native results. V11 dark close F11 remains FAIL; initial undisturbed C titlebar visual UNVERIFIED; Gate B incomplete; physical and true single/zero hardware NOT_RUN. No08–11 work, native launch, commit/push/archive/add_session. Host process closeout was empty before Root handoff; all reviewer commands returned.
