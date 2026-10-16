# Design — 真实工作台与受控观众画面

> Current acceptance policy (2026-10-15 user revision): `.trellis/tasks/10-14-presenter-v0.3/acceptance-policy.md` supersedes older hardware/matrix blockers below. Implement all features first; retain automatic/feature/native-functional checks. Physical, exhaustive native/size and performance/soak matrices are deferred, not PASS. Gate B implementation is satisfied by V16;08 may start. Gate C/D are functional integration gates. Historical results and product contracts remain unchanged.

## 选择、依赖、输入输出
前置01/02/04/05/06/07与Gate B；W-1…10/R-40/43。排期P1任务内含M1阻塞P0：真实当前/next/notes/计时/时钟/进度/跳页/黑白屏/共享屏幕分配入口/备注隔离。P1网格/拖拽/高级画笔在M2前完整验收，不能提前归档全部任务。
本任务负责app初次真实导入：05页metadata/只读结构→06 builder构建LoadedDocument/content SlideMap→core Load安装map/crop→Ready；坏build保旧document。07 Shell importer由真实DocumentLoader替换，10后续复用。输入private LoadedDocument/core state/05结果/settings给讲者与投影组装器；audience view唯一输入snapshot，不读取document/cachehandle。输出content变换/纹理/命令，02既有debug/grid键只消费。

## 文件范围
现有app/src/main.rs/07导入协调模块、ui/src/lib.rs、07shell/presenter/audience/actions模块、core/src/{command,state,geometry,snapshot}.rs、ui/Cargo.toml、docs/feature-parity.md。
拟建ui/src/{workbench,projection,texture_cache,thumbnails}.rs和core所需annotation模块；L1GPU管理归ui，render只预算/CPU。不提前加入媒体后端或09设置页。

## 设计
讲者当前左、next/notes右、顶栏计时/时钟/文件/阶段、底栏导航/模式；metadata labels只讲者显示。文档菜单提供五layout、独立notes识别、overlay自动/physical_pages和诊断；调用06 builder重建同代view，复用06已交付core ReconfigureDocument(descriptor)纯命令（同generation/同physicalcount），保留timer/stage、清preview并取最近有效content页，按新crop帧就绪后完整发布。notes独立P1请求/cache，拼合crop和按当前物理步多notes滚动展示来自06；比例/字号由settings实际应用。
投影组装器核05完整key才发布AudienceVisual，UI保留上次完整已提交snapshot直至新页帧就绪，不将旧像素与新page/crop拼接。Curtain/blank立即发布安全覆盖；新页等待时清错页laser/strokes。首次无帧用受控占位。notes请求不能进入audience结果通道。DPI变化重新请求而非错目标键命中。
两窗分别按内容rect保持ratio/letterbox；normalized stroke/laser是幻灯片裁剪后的[0,1]内容空间，讲者鼠标逆变换，观众正变换，letterbox外忽略/离开清laser。Begin/Append/End/Erase/Clear命令由此补core（per-page会话存储）；颜色宽度有限检查，坏输入typed拒绝。
M1 timer只读04elapsed与target，Ready沿同mode显示冻结elapsed或remaining（初次0或target），再Start清零elapsed，不另累积；UI定时tick不依赖raster返回。Preview发core只改讲者，Commit才请求新观众帧，Esc不提交；既有逻辑边界规则不改。
grid列1..4/可见content页1..16独立，P4 L3低优虚拟列表；拖分割比例修改settings并由协调器save，dirty语义与09一致。debug_overlay实际显示帧时间/cache字节，默认false。
ManagedRenderImage负责上传、replace、evict、DPI/generation/close回收；讲者notes纹理与audience纹理权限隔离，缓存按预算汇总，两窗同源像素Arc共享。幕布/black/white最后覆盖所有layer，不能先blank再画激光笔。

## 错误、性能与回滚
raster失败旧完整投影继续并仅讲者提示，错误键不发布；GPU上传失败回收临时资源且不混代。保存失败保留会话比例/样式标dirty。纯tests测transform/key/privacy/count，硬件另测。
记录机器CPU/GPU/driver/DPI/Hz、PDF SHA≥80页、1080p讲者+3840×2160观众，release build。200热缓存翻页（20warm），输入到actual Present的P95≤16.6ms；60s laser逐帧P99≤18ms；API缺actual Present时先研究可用测量，不能将submit span冒充显示延迟。静止baseline后200翻页、回到起始页、等待5s，专用VRAM回baseline±15MiB，另核managed texture计数/bytes无泄漏。notes隔离截图及1080→4K坐标偏差≤2px。人为raster worker阻塞2s，elapsed core正确，UI下一tick恢复与注入时间差≤100ms。
补充同机协议：100页PDF warm文件缓存后新进程启动30次，进程入口到讲者首Ready实际画面P95≤800ms；50次Start到观众actual Present P95≤300ms；静止仅timer tick、30s进程CPU时间差/逻辑核归一化平均<0.5%。1000content页自建PDF网格在1080p讲者屏持续滚动60s，60Hz目标、P95帧间隔≤16.6ms/P99≤18ms；记录PDF SHA/屏幕/样本与测量API，不凭不卡顿验收。
回滚annotation API需同步core/UI；纹理回收始终走window context，不drop图像假定GPU释放。M1部分交付与完整任务验收状态分开。

## 视觉与交互契约

遵循根 DESIGN.md。07拥有唯一theme.rs及基础控件；08复用工作台布局；09复用主题并消费general.theme，无额外主题设置schema。暖米色/陶土色仅用于讲者UI；原始PDF/媒体/备注光栅与观众安全幕布不着色。字体使用系统替代，交互错误有文字，焦点2px且有间隔。原生窄窗/双主题/DPI截图与键盘验证按DESIGN.md逐项记录，不能以纯测试或浏览器替代。

## Task05 owned render interface

Consume the concrete parent design §5 metadata and RenderHandle protocol. Page dimensions are visible rotated points; labels remain Option<String>. Probe/Load/Prepare run on the existing single render worker. Candidate preparation returns metadata first; document mapping chooses explicit prerender requests before CandidateReady. Results echo the full request identity; cache hits are marked separately from actual render duration. Commit/Abort/Retire acknowledgements and delayed retirement preserve the old active document through preparation. No native document or PDFium handle crosses to consumers.

### Shared frame allocation contract (task05 refinement)

BgraFrame::new accepts an owned Vec<u8>, validates packed BGRA geometry and stores private Arc<Vec<u8>> without copying pixels. pixels_bgra returns &[u8]; clone the BgraFrame to share an existing allocation. No mutable pixels or native handles are exposed. See parent design task05 measured frame-storage refinement; task05 owns core frame tests plus all current constructor call-site updates.

### Task06 shared map/descriptor clarification

Consume parent design task06 concrete content-map interface: private validated SlideMap+crop descriptor, explicit identity compatibility for07, mapped load for08, same-generation/same-physical-count ReconfigureDocument for document viewing options without timer reset, and strictly newer-generation CommitDocument for10 reload. Reconfigure atomically remaps committed content and clears preview; malformed/stale descriptors must leave state unchanged. Labels/notes/media remain in document; snapshot never contains the map.

### Checked crop setter compatibility

Task06 independent review found the pre-existing public PresentationState::set_slide_crop bypassed descriptor crop validation. Preserve the method with Result<(), CoreError>, reuse exactly the descriptor crop predicate/error, and reject invalid geometry without any state mutation; the sole existing lifecycle-test caller must handle the result. This closes the direct input path without changing valid crop semantics. Task08 still uses ReconfigureDocument for document viewing options so map/crop, timer/stage and preview are updated atomically. No new unvalidated crop entry point.

Task07 native startup established the synchronous first-draw constraint documented in the parent design. Workbench views must consume prepared presenter display data during render and send actions through the controller; never synchronously borrow an already-updating coordinator from open_window rendering. This planning clarification does not bypass Gate B.


## Reuse of07 display assignment

The workbench toolbar retains07's Displays entry and shared count/number/application/PDF selection surface. Do not restore a primary Swap button, fork screen numbering, or create another role state/resolver. The configured Swap shortcut remains compatible through07. M1 exercises the shared panel with real PDF content; this does not claim09's complete settings page is delivered.
