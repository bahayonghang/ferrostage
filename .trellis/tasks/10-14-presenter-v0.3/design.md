# Design — 跨任务共享契约

状态：共享设计契约；实现与验证以 execution-state.md 和当前代码为准，初始研究报告是历史证据。父任务只协调。证据见 research/repository-baseline.md、research/runtime-feasibility.md、research/behavior-evidence.md；其中未验证事项是对应子任务的研究门。

## 1. 创建者、消费者与 crate 边界

| 创建者 | 契约 / 范围 | 消费者 |
| --- | --- | --- |
| 02 | core 的 MonitorRole；settings schema/Store/warnings；app 配置 CLI | 03/07/08/09/10/11 |
| 03 | core 的 MonitorId；platform 枚举、纯解析、事件 | 07/09 |
| 04 | core 生命周期、时间输入、投影/BGRA 纯类型；document 消费者 | 05/06/07/08/10/11 |
| 05 | pdf 只读 probe/PDFium；render 协议、调度、CPU cache | 06/07/08/10 |
| 06 | core SlideMap/裁剪；document 构建器；pdf 只读媒体结构 | 08/10/11 |
| 07 | ui/app 外壳、最小屏幕分配面板/Identify、拓扑/快捷键/角色协调器 | 08/09/10/11 |
| 08 | ui 工作台、投影组装/纹理 RAII；core 标注命令 | 09/10/11 |
| 09 | ui 完整设置；复用07屏幕分配；settings/app 运行时应用 | 角色拓扑图与设置集成；11 媒体控件 |
| 10 | platform/pdf/render/document/core/ui 重载事务 | 11 消费 generation invalidation |
| 11 | media/ui 解码共享帧；document 槽位消费；settings 媒体节 | 两个窗口 |

core 只含纯类型、状态机和纯函数，不依赖 IO crate、GPUI、PDFium、lopdf、notify、OS API。document/pdf/render/media/platform/settings 在 core 外侧；app 是组合根，ui 唯一链接 GPUI并管理纹理。settings 不依赖 platform；缓存缺省仍引用 render 常量，render 不反向依赖 settings。拟建模块由子设计标注，JSONL 只注入已存在文档。

## 2. 生命周期、时间与预览

SessionStage = Idle | Ready | Presenting；加载只进 Ready。04 将 new(page_count) 定义为 Ready 的便利构造，生产走 idle()+LoadDocument；不保留隐式 Presenting 测试特例。Presenting 中 Load 被拒；Close 清空；Stop 保留页/笔画并返回 Ready。

core 不读系统时钟：app 在命令边界提供 Instant（或单调时间值）给 apply_at/elapsed_at。每次 Start 清零并运行；Pause 冻结；Resume 续跑；Reset 在 Presenting 清零并保留暂停状态；Stop 冻结读数；Ready与Presenting使用相同timer.mode格式：正计显示冻结elapsed/target，倒计显示冻结target-elapsed/target；初次elapsed=0故正计00:00、倒计target，Stop后显示不跳变；Close 清零。倒计时可显示超时。04 用受控时间输入测试，无 sleep。 04已核当前apply仅由core测试调用；统一迁移为apply_at(command, now)并更新全部当前调用方，不保留无真实消费者的兼容入口。core不读取系统时钟或虚构时间。

StartPresentation { audience_available } 仅验证 Ready 与输入前置。07 主线程协调窗口创建与 Start；创建失败保持 Ready、零增长，无残留窗口；销毁前发布 Curtain。Idle/Ready snapshot 恒 Curtain且不携带 visual。跳页预览只属讲者：Next/Previous 在预览模式只移动预览，Enter Commit 才改 audience current_page，Esc 取消；04 修正当前 audience_snapshot 和 step 的隐式提交。

SwapScreens 是发给 app 角色协调器的意图；04 移除 screens_swapped 作为权威状态，页/crop 不变。04 提供纯 DocumentDescriptor { generation, page_count } 提交入口；06 加入 SlideMap 纯描述；10 将携带 bytes 的 StageShadowDocument 升级为已校验描述提交命令，同时更新消费者。core 不存/解析 PDF bytes。

## 3. 拓扑、角色与安全 API

02 先在 core 拟建 display.rs 定义 MonitorRole，serde 小写转换由 settings 负责；03 在同模块补 MonitorId(String)，02 binding 的 id:String 由03转换。04不定义显示器类型。

platform enumerate_monitors -> Result<Vec<MonitorInfo>, PlatformError>；Info 含 ID、名称、原生像素尺寸、物理桌面位置、DPI/scale、主屏、身份来源。零屏为正常空列表，OS错误单独返回。纯 resolve_roles 返回 ResolvedTopology { presenter:Option<MonitorId>, audience:Option<MonitorId>, ignored, source, warnings }；warnings 包含 NoMonitors、SavedAudienceMissing、DuplicateAudience、DuplicatePresenter、PresenterMissingFallbackPrimary、DuplicateBinding、AmbiguousMonitorId。

无保存记录：主屏（没有则ID排序首屏）作Presenter；多屏中最大面积且ID不同于已解析Presenter的屏作Audience，同面积按ID排序。保存存在则不启用Audience启发式：缺席/显式None不自动接管。Presenter缺席允许主屏安全回退并告警；零屏 presenter=None。先按MonitorId规范化每ID唯一role（重复binding首项权威+DuplicateBinding）；重复角色再按配置首次有效绑定，其余None并告警。多屏已保存Audience先确定，Presenter回退须排除该ID；单屏解析固定Presenter本屏/Audience=None，由显式windowed模式在窗口层允许同屏窗口。两个物理屏退化ID相同，禁止自动恢复Audience。设备路径/EDID/名称退化都不能保证跨接口绝对稳定，实测失败应提示重绑。

权威来源仅 display.monitors。07最小分配面板将应用屏/PDF屏草稿作为一次角色事务应用，清除旧角色并确保每ID一个role；09复用此面板/协调器。既有SwapScreens仅保留快捷键兼容并走同一事务，不作为工具栏主入口。移除计划键 display.swap_roles；若已有外部文件包含此键，则保留为未知历史键但不执行。Presenting失去Audience：Curtain→Stop→销毁观众窗口→Ready；讲者保持有效Presenter屏窗口化。新屏不自动接管；零屏等待恢复，重连不自动Start。

03提供已审阅的安全wrapper；07使用已审阅的GPUI事件/DisplayId与完整UUID桥。身份映射不得退回位置、尺寸、DPI或屏幕索引；实际bounds仅用于创建后的窗口放置确认，映射歧义拒绝Start。workspace unsafe_code=forbid不能被局部allow覆盖。安全接口不足时须先形成最小适配模块、显式lint继承调整与ADR政策提案，评审通过才实施；不改core或全workspace禁令。

03已审阅的Windows实现方案：仅Windows目标链接winsafe =0.0.29（MIT，gdi feature包含user），不引入display-info/fxhash。安全EnumDisplayDevices提供attached adapter/active monitor的owned interface path与当前GDI连接名，迭代引用当次拷贝；安全桌面DC RAII与EnumDisplayMonitors提供每屏DC，GetMonitorInfo名称关联EnumDisplaySettings current的DEVMODE物理位置/尺寸/旋转，核对device caps物理宽度及完整active GDI集合一一匹配，否则返回typed错误。device-interface path身份等级为ConnectionScoped，不称EDID或跨接口不变；缺path的name|WxH为Degraded，碰撞禁止Audience。scale来源显式GdiDeviceCapsRatio，即当前调用DPI context下DESKTOPHORZRES/HORZRES之比，不是已验证effective DPI；07须在实际GPUI context中验证映射，不能将ratio=1当作100%系统缩放设置。

03采用200ms安全枚举轮询，取消订阅释放worker；Changed传owned拓扑，Unavailable传错误并让Audience availability=false。last-known仅用于诊断，不作为旧投影继续有效的依据。07收到Unavailable与目标屏移除同样走Curtain/Stop/销毁，恢复只更新可用拓扑不自动Start；09显示identity/scale等级与警告，不另造回退策略。此方案保持workspace unsafe_code=forbid；native静态枚举、换线/热插拔及≤1s硬件证据分别记录。

## 4. SettingsStore 与 schema

02定义v1八节，增加 general.debug_overlay=false（08消费、09控件）、general.recent_files=[]（07导入/最近文件服务消费）、presenter.grid_columns=4（1..4）、grid_visible_pages=16（1..16）；行数ceil(visible/columns)，列数不等于页数。11增加可选media节 max_decode_height=1080，宽度按比例且不超过1920；同v1缺省添加，不造无意义v2迁移。

load：读取原始TOML树→辨识版本/真实旧格式转换→逐已知叶节点反序列化+校验→返回Settings、原树、warnings、WritePolicy、needs_writeback。非法类型也是单项回退；整个节类型非法时只回退该节。warning记录键路径/原因，不打印完整文件。

未知字段保留值/结构，不承诺注释排版逐字保留。save递归merge已知叶键；未知顶层、已知节未知键、monitors元素未知字段都保留。monitors按id+重复项稳定序号对齐原元素；角色改变不覆盖整节，删除binding只删该元素，新屏不继承别屏未知值。

当前可写版本缺省节由app显式save补全，from_toml/check-config无写入副作用。坏语法文件备份为.broken-时间戳成功后才可写默认；备份失败保留原件并禁写。未来版本只读，不重置/改名/写回。第一分节版本为v1；骨架旧扁平缓存/防抖键按真实映射迁移，索引屏幕键无法稳定转换，保留原值并告警重绑。不实现空迁移链。

save：同目录唯一tmp→完整写入→sync_all→验证过的Windows replace-existing或Unix rename→清理tmp。02必须测试已有目标、锁定/中断、失败不破坏旧文件；普通fs::rename不能代替Windows验证。保存失败保留运行时有效配置并标记“未保存”/可重试，重启取上次成功值；运行时应用失败先回滚该修改再提示，不写入失败配置。未来版本UI只读。所有角色修改走同一协调器。

09维护键→消费者→生效表：布局/计时/指针/cache/debug由08即时应用；快捷键/角色由07；语言重启后确实切换资源；dark/light/system确实切换配色并监听系统主题。hot_reload仅在10接线后可启用，媒体项仅在11接线后显示可编辑；不以能写TOML验收空控件。

## 5. 线程与渲染协议

| 执行域 | 工作 | 跨界值 |
| --- | --- | --- |
| platform文件IO worker | read-only读取后关闭句柄；10稳定性/EOF | Arc bytes、attempt token |
| pdf/document结构worker | lopdf只读媒体结构；合并元数据构建SlideMap | owned metadata/warnings |
| 唯一render worker | PDFium初始化、active/candidate加载、PageLabel、尺寸、预渲染、释放 | owned metadata/BgraFrame/ready token |
| GPUI主线程app/ui | 最新token检查、core/document/visual事务、纹理/窗口 | audience-only snapshot |

PDFium bindings/document/page/bitmap都不逃逸；bytes owner覆盖借用文档，不建unsafe自引用。pdfium-render本身不提供/自动下载PDFium；05锁binary构建号、来源、校验值并传入动态绑定路径，不虚构PDFIUM_DYNAMIC_LIB_PATH为库自动支持变量。 0.9.4绑定使用进程全局OnceCell；05用安全进程级会话互斥与首次成功初始化后的固定native配置保证唯一worker，不静默切换库路径/字体目录，不以失败初始化污染成功状态。不同配置需typed诊断/重启；缺库负测试隔离进程，避免已加载bindings掩盖缺库。

消息：LoadActive/PrepareCandidate（先返回metadata）、PreRenderCandidate（06建图定位后请求）、RenderRequest、CommitCandidate、AbortCandidate、RetireGeneration。请求/结果键包含generation/page/crop/target_px/priority/epoch；P0观众当前、P1讲者当前/下一页/备注、P2±1观众预取、P3预览、P4缩略图。candidate有独立命名空间/结果通道，不能被current-only检查提前丢弃，旧active持续服务。epoch可取消排队项/丢弃旧结果，不能承诺打断同步PDFium调用；优先级只决定下次出队。缓存按bytes预算。

05接口具体化：pdf提供owned PageMetadata（PhysicalPage、可见旋转后width_points/height_points、rotation_degrees、Option<String> label）与DocumentMetadata/PdfError。render的RenderRequest完整携带generation/page/crop/target_px/priority/epoch，RequestId关联结果；结果回显完整请求与BgraFrame、实际render_duration/cache-hit。RenderHandle只传owned消息：probe/load_active/prepare_candidate返回metadata，pre_render_candidate返回CandidateReady token及请求对应owned frames，commit(token)/abort(generation)/retire(generation)明确ack；候选不能被active-only过滤，旧active资源保留到retire。所有native对象在唯一worker作用域内，不为probe另建并发PDFium实例。

性能样本必须是worker真实PDFium render调用，绕过cache，不含queue/upload；cache-hit结果独立标记，不计入20warm/200sample的渲染延迟分布。缓存仍按generation/page/crop/target_px键与字节预算，epoch用于请求失效而非错误地让旧结果发布。

## 6. 观众唯一数据入口与 SharedFrame

04创建纯core BgraFrame { width,height,stride,pixels_bgra:Arc<Vec<u8>> }，checked长度、非零尺寸、紧密width×4 stride；05 BgraBitmap复用。11在media::SharedFrame复用该像素类型并加generation/page/slot/playback_epoch/sequence身份封装；core不依赖media。

AudienceProjectionSnapshot是观众view唯一输入（可Entity包装）：stage、generation/current_page/slide_crop、blank/laser/strokes，visual:Option<AudienceVisual>。visual只有匹配当前键的受控页BGRA与AudienceMediaFrame { slot_id,rect_norm,pixels }，没有LoadedDocument、SlideMap、notes_page/crop、路径、媒体源、讲者预览、计时。Curtain不带visual。04纯audience_snapshot产生投影元数据；08组装器从私有cache按精确键补visual；11补媒体。

观众view没有整个cache句柄或任意页请求接口；snapshot BGRA上传到自己的受预算RAII纹理，讲者缓存权限独立，可共享同源Arc像素。错页/crop/DPI/代际帧不能绘制。组装器保持上一完整已发布投影直至新页帧就绪，不能旧像素配新页metadata；Curtain/blank立即安全覆盖、等待时清错页laser/strokes，首次显示受控占位；幕布/black/white在媒体/标注/激光笔之上。08负责替换/淘汰/关闭/代际纹理释放，11沿用。媒体源只在document/media控制侧解析；跨代/跨页帧在组装边界拒绝。

## 7. Beamer 与重载提交

06独立几何规则：左右W/H落在[3.1,3.6]或[2.4,2.7]；上下落在[0.85,0.95]或[0.63,0.70]，对应8:9/2:3。页尺寸一致才自动给候选Right/Bottom；比例不能证明备注存在/方向，UI显示来源与warning；文档级None/Left/Right/Top/Bottom手动覆盖最终权威，slide取对侧。

PageLabel保留Option<String>，缺失/空串逐页独立；连续相同非空content标签才启发式overlay分组，非连续重号不合并；提供physical_pages覆盖。notes:<suffix>是本项目约定，只与紧邻此前内容步的原始标签精确匹配，不把n当数组索引；多备注按物理步存Vec页号且仅在document，歧义/孤立跳过投影并给讲者warning。core SlideMap只含可导航内容步。自建夹具不需要产品编译器调用，媒体字段按实际PDF字典支持子集测试，未知结构告警，不承诺还原任意导出。

10：外部替换→300ms静默→稳定read→结构/PDFium candidate→06建图→P0/P1预渲染→主线程校验最新attempt→同次提交descriptor/document/audience visual→worker Commit→回收旧UI/CPU缓存→worker Retire旧PDFium。提交前任意失败Abort，旧generation/阶段/计时/画面继续；过时attempt丢弃。关闭窗口/文档令token失效，不能组合不同代metadata和像素。

定位优先唯一旧逻辑label+overlay ordinal，歧义/缺失则旧物理号先钳至新物理范围，取不大于该号的最大content物理页，无则首content，不能作content Vec下标。笔画只保留能映射的页，移除页安全丢弃并讲者告警；阶段/计时不重置。11消费代际失效事件停媒体，尚无媒体实现时该事件无副作用。

## 8. 门禁、兼容、回滚

L0：纯Rust suite必跑just ci（fmt/clippy/test/deny）；core/settings/纯topology/scheduler/document/帧权限必测。L1：05显式pdfium feature，CI native job装经校验DLL并必跑真实raster；开启feature缺库必须失败。L2：07 gpui feature编译门不启动窗口。L3：Windows实机拓扑/DPI/4K/备注隔离/GPU回收/媒体。禁止ignore或缺库return Ok；默认suite绿不等于native交付完成。

性能报告记录机器CPU/GPU/驱动/DPI、PDF SHA、分辨率、binary/build、预热/样本/时间边界，协议见子implement；无数据就待验。01先同步baseline列出的ADR/spec冲突（写PDF、swap flag、settings只读），本次不修改这些实施目标。用户只读和唯一角色来源约束优先于旧spec。

共享API回滚需连同消费者；设置失败不覆盖旧文件；重载回滚candidate；native安全API不足停在研究门，其他无依赖工作可继续。ADR/lint政策和媒体loader选型要形成具体方案评审，不能以待定技术为理由削减已授权需求。

## 9. 讲者界面主题（R-45）

根 DESIGN.md 为视觉唯一来源，采用design-md brand:claude并适配FerroStage原生布局、可读对比度及系统字体。07创建ui/src/theme.rs与基础控件状态，08复用布局与工具条，09将已有general.theme接到同一theme resolver并验证system事件。theme与Store仅在讲者控制侧；不向audience snapshot添加主题字段，PDF/备注/媒体像素及黑白幕布不变。原生截图/键盘/字体回退/响应尺寸/DPI均须实际运行验证。只修改现有07/08/09规划，不增加另一任务树或改变Gate阻塞范围。

### Task05 measured frame-storage refinement

The retained-previous-frame PoC2 exposed a 33MiB Vec-to-Arc-slice copy (P95 25.602ms). BgraFrame now accepts an owned Vec<u8> in its checked new constructor, stores a private Arc<Vec<u8>>, and exposes pixels only as &[u8]. Cloning BgraFrame shares the existing allocation. No public mutable access, channel conversion, legacy dual-storage enum or unsafe initialization is needed: the unreleased callers are updated together. Task05 removes the scratch buffer so retained and released frames follow the same production allocation path; native PdfBitmap borrows the Vec only until bitmap destruction, then the Vec moves into the checked frame. Keep geometry/stride/length validation and geometry-only Debug unchanged, prove allocation sharing and retained-frame immutability, and rerun core plus native/performance checks. This refinement remains task05 work and does not reopen the task04 lifecycle implementation.

CPU frame-cache byte accounting uses BgraFrame::allocation_bytes(), the owned Vec capacity, not only its visible pixel length. This is a read-only scalar; pixel access stays immutable and checked geometry stays based on length. L1 texture accounting remains width×height×4 for actual texture storage, not the CPU Vec capacity. Oversized CPU allocations are returned uncached rather than violating the global budget.

CandidateReady is scoped to the worker instance that issued it, in addition to generation, epoch and candidate-batch identity. A token from another worker or a restarted worker must be rejected even if its numeric fields happen to match. Keep this identity private to render; consumers only pass the opaque token back to its original handle.

### Task06 concrete content-map interface

Core SlideMap owns validated, nonempty, strictly increasing content steps grouped into logical slides; it contains no labels, notes or media. Constructors accept physical page_count and ordered groups, rejecting empty/out-of-range/duplicate/reordered maps before state mutation. Core DocumentDescriptor uses private validated map+crop fields with identity(generation,count) and mapped(generation,map,crop) constructors. The existing count-based LoadDocument path remains the explicit identity compatibility used by07; LoadMappedDocument installs the real document builder output. CommitDocument keeps the strictly newer-generation rule and preserves timer/stage while clamping to valid content. The temporary StageShadow adapter preserves existing map/crop, without parsing bytes.

Document-local layout/notes-identification/physical-pages overrides require a same-document reconfiguration path: ReconfigureDocument(descriptor) must match the current generation and physical page_count, preserve lifecycle/timer, remap committed content to closest_content_at_or_before and clear preview atomically. It must reject mismatched identity or invalid geometry without mutation; do not fake a new PDF generation or reset the session merely to change a viewing option. Task08 uses this path and publishes only exact generation/page/crop frames; task10 uses the newer-generation commit path.

Document LoadedDocument::build consumes owned PDF metadata plus read-only structure and DocumentOverrides. It owns raw optional labels, resolved layout/source, SlideMap, ordered page-notes mapping, original markdown notes, media slots and diagnostics. notes_for retains its markdown semantics. New load uses default overrides; same-file reload passes current overrides explicitly. Media structure source/paths/embedded data remain on the document side; raw PDF rectangles/boxes/rotation are converted there to visible top-left/crop coordinates. Unsupported or unresolved times are diagnostic, never guessed seconds.

### Checked crop setter compatibility

Task06 independent review found the pre-existing public PresentationState::set_slide_crop bypassed descriptor crop validation. Preserve the method with Result<(), CoreError>, reuse exactly the descriptor crop predicate/error, and reject invalid geometry without any state mutation; the sole existing lifecycle-test caller must handle the result. This closes the direct input path without changing valid crop semantics. Task08 still uses ReconfigureDocument for document viewing options so map/crop, timer/stage and preview are updated atomically. No new unvalidated crop entry point.

### Task07 pinned Windows display bridge

For the adopted Zed v1.23.3 source route, task07 maps the existing owned MonitorInfo.connection_name to the pinned public GPUI display UUID: encode into exactly32 zero-filled UTF-16 code units, serialize big-endian bytes and compute UUIDv5 with the DNS namespace. Reject overlong/unrepresentable input, duplicate MonitorIds/names/GPUI IDs/UUIDs and incomplete set/count matches before Start. This is a pin-specific transient mapping, not a persistent MonitorId or effective-DPI estimate. No platform API expansion or geometry fallback is needed. The user-approved maintained tagged source under ADR-008 has passed actual product dependency/build checks. Actual A/B/C placement has partial native evidence; complete mixed-DPI/physical topology acceptance remains pending. Source feasibility, candidate approval and automatic checks alone are not Gate B.

### Task07 role migration without a native position setter

The evaluated Zed tag public GPUI API has no general window-position setter. If that route clears the dependency gate, role migration prepares replacement windows using the existing presenter/snapshot entities, stages audience output behind Curtain, and verifies each actual display before retiring old windows. Keep core stage/timer/page unchanged during Swap; do not Stop/Start merely to move windows. On partial failure close replacements, restore old complete snapshot/handles and keep old authoritative role bindings. Distinguish retired-window close callbacks from current user-close events by window identity, and update Presenter focus/modal targets to the replacement handle. Save only after the runtime transaction succeeds; save failure retains session values with an unsaved warning. Pure transaction tests and real placement/focus verification remain separate evidence.

### Task07 asynchronous activation and fullscreen boundary

The independent source review found that Windows activation and fullscreen requests schedule executor work; returning from activate_window/open_window does not establish completion. The native path chooser captures the active window synchronously. The coordinator must therefore confirm the current Presenter is active before opening a chooser/modal, and keep a new Audience behind Curtain until its actual display and required window mode are confirmed. Start/time begins only after successful confirmation. Apply the same boundary to replacement-window migration before retiring old windows. Cancellation, topology changes, superseding requests or a bounded confirmation failure must invalidate pending callbacks and clean up replacements without opening a late modal or publishing Live. A cached/requested mode alone is not placement evidence. The audited public activation path is Context::observe_window_activation plus Window::is_window_active and a fresh App::active_window identity check (Windows reads GetActiveWindow at that call). For placement, Context::observe_window_bounds plus Window::bounds/display provide event-updated observations. Window::is_fullscreen alone is insufficient: the Windows flag is set before SetWindowPos, whose failure is logged rather than returned. No public completion Result/future establishes native fullscreen success. The implementation must establish a bounded confirmation protocol using actual placement observations and verify it natively; until then this remains an implementation prerequisite, not a proven solution.

### Task07 approved source and policy delivery

User explicitly approved the final independently reviewed five-package local maintenance set and only arrayref@0.3.9/BSD-2-Clause, libbz2-rs-sys@0.2.5/bzip2-1.0.6, cbindgen@0.28.0/MPL-2.0 exceptions. The product adopted the final reconstruction, not the original defective maintenance patch alone, and passed I-2 actual graph/build verification. Preserve upstream notices and test provenance; do not broaden global policy, add advisory/lint ignores or change core purity. Parent research/gpui-license-review.md and ADR-008 record exact hashes and retained limits; original failures and six upstream direct HarfRust lint failures remain disclosed.

### Task07 synchronous first draw boundary

The pinned App::open_window performs a synchronous first draw before it returns. A presenter created while Shell is leased by Entity::update must not read or update that controller during render. Task07 supplies a small presenter display-data value prepared from the current Shell borrow; later model notifications refresh that value after the update lease ends. It contains only fields actually rendered, not a duplicate PresentationState, SettingsSession, worker or deck. Actions retain a WeakEntity<Shell>. Audience still receives only AudienceProjectionSnapshot. Cover the actual GPUI open_window/draw path with a feature-scoped harness and repeat native startup after rebuilding; a pure policy test does not cover this entity-borrow contract. Initial native failure and source identity are retained in07 research/poc-1-report.md and native/initial-freeze.json.

Window removal can also synchronously invoke App::on_window_closed before the calling Shell update releases its entity lease. Defer controller mutation from that callback until the current update completes, carrying the original WindowId and applying the existing current-versus-retired identity check at consumption time. Test actual GPUI removal under a coordinator lease, including timeout rollback and replaced-window cleanup. Do not suppress the callback, swallow the panic or relax placement confirmation.

The shell owns explicit application exit: closing a pending/retired window during topology cancellation or migration must not trigger last-window automatic quit. Closing the current presenter is an intentional user exit; closing the audience safely stops projection while retaining the presenter. A zero-screen state waits for topology recovery without creating an audience or exiting implicitly.

Input actions may run while GPUI has leased the originating window. Dispatch shared shell intents after that event update completes, before requesting activation, opening/replacing windows or closing projection. Do not call AnyWindowHandle::update on the same leased window from its key/mouse/accessibility callback. Exercise real input dispatch (keyboard and mouse), not only direct controller methods, and retain presenter-owner activation confirmation before a chooser.

### Native-driven input correction (2026-10-15T01:46:29.323959+00:00)

07 native v4 exposed focused Start + Space dispatching global Next and Enter doing nothing. The existing DESIGN.md Enter/Space rule is unchanged: enabled focused controls consume unmodified activation keys before presentation bindings; disabled controls stay noninteractive; actual GPUI focus/key regression must fail before repair. Shared deferred action dispatch remains mandatory.

Open uses the existing import_allowed contract consistently at button, global intent, chooser and import entry. During Presenting/incompatible transactions it must not open a chooser whose result is guaranteed to be rejected. Audience-origin refusal confirms Presenter activation before localized feedback. No mid-presentation document replacement is added. Root owns this contract clarification;07 owns the local repair.


### Native focus and save-feedback recovery
When a focused presenter control disappears (modal Close/Commit, Start/Stop, Retry after success), restore a valid focus target in the same presenter so Tab and presentation shortcuts remain usable without a mouse click. Preserve existing valid button focus; do not steal audience focus as a side effect of unrelated model updates. Cover actual GPUI focus/key dispatch across removal, not only stored focus handles.

A failed settings save is one unsaved-state message with Retry. A successful Retry clears the corresponding stale persistence error while retaining unrelated current errors. Runtime role changes survive persistence failure; durable role changes require successful read-back. Native FileShareRead-lock evidence is distinct from mocked save errors.


## Screen assignment planning correction — 2026-10-14

User intent: detect available displays, identify their numbers, then explicitly choose the application/Presenter display and PDF/Audience display. The toolbar action is “显示器设置” / “Displays”; it opens a presenter-owned panel and does not move windows merely by opening it.

03 remains the owner of complete MonitorInfo/ResolvedTopology/TopologyEvent data. 07 owns the shared minimal display-selection model, numbered list, two labeled selectors, Apply/Cancel, current effective assignments and 3s Identify overlays. 08 reuses its entry point; 09 embeds it in the full settings page and adds the desktop topology visualization and other settings. No07→09 dependency is introduced.

Number labels are a presentation mapping over the current complete topology, consistent between the list and Identify overlays. Keep the same mapping while that topology is unchanged; do not claim that these are Windows Settings numbers unless verified from an actual OS identifier. Persist MonitorId only, never the UI number or enumeration index. Refresh on open and topology events; distinguish no displays from enumeration failure. Topology change invalidates a stale draft and forces revalidation against current IDs before Apply. Display identity confidence and scale source stay truthful; a GdiDeviceCapsRatio is not an effective scale measurement.

The two selectors edit a draft; Apply validates both destinations together and invokes the existing07 coordinator. With multiple screens, application and PDF destinations must be distinct; unselected screens are None. “不播放” leaves no Audience and Start disabled. On a single screen, the PDF selector explicitly offers “不播放” (presenter_only) or “在本屏窗口播放” (windowed_audience); Apply updates single_monitor_mode together with the role draft, never adding a duplicate physical-role binding. Cancel leaves runtime and disk unchanged. Both “不播放” and a selected physical PDF display set single_monitor_mode=presenter_only; only the explicit single-screen windowed choice sets windowed_audience, preventing an inherited windowed mode from silently enabling a different target after topology loss. Apply preserves the current page/stage/timer during a valid migration. Choosing “不播放” while Presenting instead follows Curtain→Stop→audience close, retaining the page and freezing the timer in Ready; creation/placement failure restores old roles/windows, and save failure keeps effective roles with one unsaved/Retry owner. Future settings remain read-only. Identify is unavailable while Presenting and carries only number/name, no PDF/notes. All dialogs/selectors stay on the confirmed Presenter.

New source UI tests and native count/number/Identify/assignment/restart observations are mandatory before Gate B PASS. Existing configuration-driven A/B/C and Swap receipts remain historical mechanism evidence, not acceptance of this new UI. Physical gates and P2 remain unchanged.
