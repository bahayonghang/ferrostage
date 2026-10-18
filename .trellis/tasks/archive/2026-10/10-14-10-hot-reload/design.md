# Design — 外部 PDF 替换事务

> Current acceptance policy (2026-10-15 user revision): `.trellis/tasks/10-14-presenter-v0.3/acceptance-policy.md` supersedes older hardware/matrix blockers below. Implement all features first; retain automatic/feature/native-functional checks. Physical, exhaustive native/size and performance/soak matrices are deferred, not PASS. Gate B implementation is satisfied by V16;08 may start. Gate C/D are functional integration gates. Historical results and product contracts remain unchanged.

## 选择、依赖、输入输出
完整前置01/04/05/06/07/08与Gate C；H-1…8/R-01/02/44。09不是hard依赖，设置可由配置指定；若UI存在由本任务启用09的对应控件。
输入target path/notify events/settings；输出attempt/candidate owned metadata/frame和主线程新generation事务。用户PDF只读，测试覆盖写入只在独立temp目录。

## 文件范围
现有platform/src/lib.rs/Cargo.toml、pdf/render/document/core/ui各src/lib.rs及05worker/06builder/07app协调器/08projection、core command/state、app/main.rs、09settings capability映射（若存在）、ADR-004。
拟建platform/src/reload.rs、app/ui reload coordinator及各crate就近integration tests。SHADOW_TMP_SUFFIX删掉及其误导注释/测试，不挪给settings制造第二tmp策略。

## 执行线程/数据流
notify监听父目录过滤文件（支持tmp rename），300ms静默事件合并；IO worker两次大小/mtime间隔采样（≥100ms）+尾1KiB EOF提示，read完整bytes关句柄后复核stat，30s未就绪debug放弃本轮。稳定+EOF不保证valid。
structure worker用lopdf解析媒体/结构；唯一render worker PrepareCandidate先PDFiumload/label/dimensions并返回owned metadata；06 builder合并metadata/structure建图/定位/crop，再worker PreRenderCandidate(P0/P1 requests)等待ready，所有PDFium对象仍在worker。active候选同时存在，由05两命名空间服务，候选结果不触发current-only丢弃。
AttemptToken随新事件/Close/新Load递增；ready结果先验证token/desired generation/仍同path。标签定位唯一组+ordinal，否则旧physical号钳到新物理范围，选≤该号最大content物理页，无则首content，不作content Vec下标，不掉notes；可映射页笔画保留、删页丢弃diagnostic，timer/stage不变。
提交主线程：prepare完整descriptor/document/visual/纹理→两窗实际paint上传ack→worker commit_reversible ack→再次校验latest attempt/窗口/请求目标→在当前core上应用映射，同次发布owned document/snapshot并promote同一纹理Arc→worker Retire旧PDFium ack。Commit等待期间过期必须消费ack并用opaque capability回滚，再退役新代；不得以Abort冒充回滚。详细接口与错误矩阵见末尾已通过的I-1实施契约。
11尚未实现时generation失效事件无op，11接后先停旧媒体；不引入对11的循环依赖。计时只读04now无重启。watcher运行中开关重建并取消旧attempt，debounce动态更新。

## 错误与回滚
半写、缺EOF、结构解析失败或无效候选PDF只debug/日志，不显示错误横幅或模态窗、不黑屏；EOF仅提示，残留EOF不能证明写完。GPU上传/容量及worker通信、回滚/退役等运行故障仍通过讲者端显示可恢复反馈，不能把真正的事务失效静默成文件未就绪。旧服务不提前清缓存。并发候选只保最新，过时native文档在worker释放。资源回收计数/ack验证而非凭generation增加宣称完成。swap/focus/window/错误由07/08协调，重载不直接创建第二窗口。
回滚候选的bytes/metadata/frame/纹理在各线程释放；主线程发布后旧代只作退役不可重复提交。无任何以target path写方式打开的调用，fixture harness路径强制temp前缀。

## Task05 owned render interface

Consume the concrete parent design §5 metadata and RenderHandle protocol. Page dimensions are visible rotated points; labels remain Option<String>. Probe/Load/Prepare run on the existing single render worker. Candidate preparation returns metadata first; document mapping chooses explicit prerender requests before CandidateReady. Results echo the full request identity; cache hits are marked separately from actual render duration. Commit/Abort/Retire acknowledgements and delayed retirement preserve the old active document through preparation. No native document or PDFium handle crosses to consumers.

CandidateReady tokens are opaque capabilities bound to the originating worker instance as well as generation/epoch/batch. A restarted or replacement worker requires a fresh Prepare/PreRender result; never reuse an earlier worker token.

### Implementation-start transaction review

Further production API evidence2026-10-15: `Window::paint_image` inserts the nonempty image into the sprite atlas **before** reading `snapped_content_mask`; `Scene::insert_primitive` then rejects an empty clipped region (`vendor/gpui/src/window.rs:4953`, `vendor/gpui/src/scene.rs:86`). Therefore a normal canvas paint callback may use nonempty image bounds inside `Window::with_content_mask(Some(empty_mask), ...)`: real atlas insertion/error occurs, while no candidate scene primitive is visible. This is different from empty image bounds, which bypass upload. I-1 should consider this bounded production staging route, with retained image ownership, per-window acknowledgements and explicit abort/drop. Do not use test-only atlas inspection in production or claim successful Present from upload success. Audience staging, if needed, must still arrive only as public pixels and pure identity in its authorized snapshot; it must never receive a document/controller/cache handle or private Presenter notes. The concrete shared snapshot/API and worker commit ordering still require implementation-start review before editing.

Pinned GPU API check2026-10-15: `Window::paint_image` (`vendor/gpui/src/window.rs`) asserts the paint phase and returns early for empty bounds before atlas insertion. A zero-sized/off-phase call cannot establish candidate upload success. `has_image_atlas_entry` is test-support-only. The reviewed I-1 contract below defines a production paint/acknowledgement route for the required pre-publication upload failure test without leaking notes or giving Audience a controller/cache handle. Do not claim a CPU candidate or an empty paint proves GPU readiness. Preserve the complete old projection and bounded image ownership throughout any preparation; coordinate any necessary pure snapshot refinement with the parent before changing shared types.

Current08 adapter audit2026-10-15: `document_load::PendingLoad` advances Metadata→Raster→Commit, waits for the worker commit acknowledgement, then returns `DocumentSession`; `Shell::finish_import` installs mapped core/document state and retires the former generation. Normal imports prohibit another import while loading and prohibit import while Presenting. Hot reload cannot merely reuse this adapter unchanged: it needs target-page P0/P1 prerender, timer/stage preservation, latest-event invalidation and explicit cleanup when a newer event/Close arrives during Commit acknowledgement. Track the worker's actual committed generation even when a result becomes obsolete; never discard an in-flight commit reply and assume Abort can undo an already active generation. Review and regression-test the concrete ordering before product edits; the reviewed I-1 contract below selects the opaque reversible-commit API.

Task05 confirms commit errors return NotReady before mutation; successful commit switches active generation, invalidates old queued work/CPU entries, and retains the old PDFium document until explicit retire. Task10 must concretely review ordering of worker commit acknowledgement and main-thread descriptor/document/visual publication, including a newer attempt or Close arriving during the wait. A failed or stale acknowledgement must not leave mixed generations published. The reviewed I-1 section below resolves this ordering and selects a minimal render API; acceptance remains separate.

### Task06 shared map/descriptor clarification

Consume parent design task06 concrete content-map interface: private validated SlideMap+crop descriptor, explicit identity compatibility for07, mapped load for08, same-generation/same-physical-count ReconfigureDocument for document viewing options without timer reset, and strictly newer-generation CommitDocument for10 reload. Reconfigure atomically remaps committed content and clears preview; malformed/stale descriptors must leave state unchanged. Labels/notes/media remain in document; snapshot never contains the map.

## I-1 reviewed implementation contract (2026-10-15)

Root reviewed the production content-mask upload and the following minimal API refinement. This is an implementation contract, not a native acceptance result.

- `AudienceProjectionSnapshot.staging: Option<AudienceStagingFrame>` carries attempt/generation/content page/crop/public pixels only. Existing committed projection identity/pixels stay unchanged until publication. Presenter stages its own P1 images separately; notes never enter Audience staging.
- `RenderHandle::commit_reversible(CandidateReady) -> Reply<RollbackToken>` returns an opaque, non-clone capability bound to worker instance, candidate generation, previous generation/epoch and one transaction nonce. `rollback_committed(RollbackToken) -> Reply<GenerationAck>` consumes it; active mismatch, retirement, cross-worker or reuse cannot restore arbitrary generations. Publication discards the rollback capability and retires the previous generation with an acknowledged command.
- Read/structure -> Prepare metadata -> document map/current-location plan -> explicit actual-target P0/P1 raster -> both live windows' real paint upload acknowledgements -> reversible Commit -> latest identity/target validation -> apply reload mapping to the **current** core state, promote the same uploaded image Arcs, publish session and projection -> Retire old acknowledgement. Metadata-time clones never overwrite later timer/blank/stroke input.
- New event/import/close/window migration/target or document-option changes invalidate the attempt. Before Commit: Abort acknowledgement; while Commit pending: keep the reply, then rollback actual committed worker state and await acknowledgement, then Retire candidate acknowledgement. Never confuse Abort with reverting an active generation. Stable bytes can be rebuilt for navigation/target changes without another filesystem event.
- GPU staging uses nonempty image bounds under an empty ContentMask during normal paint. Successful staged Arcs remain owned until promote/cancel; old plus staged allocations obey the configured budget. Failure drops staged allocations and keeps old complete pixels/identity; blank/Curtain dominate immediately. A missing/closed window never supplies a successful acknowledgement.
- Errors before publication keep the old generation; timeout is debug-only. Failed rollback enters an explicit recoverable error state that suppresses mixed-generation rendering until re-import. Failed retirement is reported, not hidden as a pass. No forward core generation is rolled back.

Resource matrix: IO/parse failure has no native candidate; Prepare/raster/upload failure requires Abort; successful stale Commit requires Rollback then Retire; published Commit requires Retire(previous); window cancellation releases staged GPU images in that owning window. All replies remain owned until settled.
