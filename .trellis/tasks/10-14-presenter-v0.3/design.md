# Design — 跨任务共享契约

状态：规划设计；API 尚未实现，PoC 尚未通过。父任务只协调。证据见 research/repository-baseline.md、research/runtime-feasibility.md、research/behavior-evidence.md；其中未验证事项是对应子任务的研究门。

## 1. 创建者、消费者与 crate 边界

| 创建者 | 契约 / 范围 | 消费者 |
| --- | --- | --- |
| 02 | core 的 MonitorRole；settings schema/Store/warnings；app 配置 CLI | 03/07/08/09/10/11 |
| 03 | core 的 MonitorId；platform 枚举、纯解析、事件 | 07/09 |
| 04 | core 生命周期、时间输入、投影/BGRA 纯类型；document 消费者 | 05/06/07/08/10/11 |
| 05 | pdf 只读 probe/PDFium；render 协议、调度、CPU cache | 06/07/08/10 |
| 06 | core SlideMap/裁剪；document 构建器；pdf 只读媒体结构 | 08/10/11 |
| 07 | ui/app 外壳、拓扑/快捷键/角色协调器 | 08/09/10/11 |
| 08 | ui 工作台、投影组装/纹理 RAII；core 标注命令 | 09/10/11 |
| 09 | ui 设置；settings/app 运行时应用 | 最终角色交互；11 媒体控件 |
| 10 | platform/pdf/render/document/core/ui 重载事务 | 11 消费 generation invalidation |
| 11 | media/ui 解码共享帧；document 槽位消费；settings 媒体节 | 两个窗口 |

core 只含纯类型、状态机和纯函数，不依赖 IO crate、GPUI、PDFium、lopdf、notify、OS API。document/pdf/render/media/platform/settings 在 core 外侧；app 是组合根，ui 唯一链接 GPUI并管理纹理。settings 不依赖 platform；缓存缺省仍引用 render 常量，render 不反向依赖 settings。拟建模块由子设计标注，JSONL 只注入已存在文档。

## 2. 生命周期、时间与预览

SessionStage = Idle | Ready | Presenting；加载只进 Ready。04 将 new(page_count) 定义为 Ready 的便利构造，生产走 idle()+LoadDocument；不保留隐式 Presenting 测试特例。Presenting 中 Load 被拒；Close 清空；Stop 保留页/笔画并返回 Ready。

core 不读系统时钟：app 在命令边界提供 Instant（或单调时间值）给 apply_at/elapsed_at。每次 Start 清零并运行；Pause 冻结；Resume 续跑；Reset 在 Presenting 清零并保留暂停状态；Stop 冻结读数；Ready与Presenting使用相同timer.mode格式：正计显示冻结elapsed/target，倒计显示冻结target-elapsed/target；初次elapsed=0故正计00:00、倒计target，Stop后显示不跳变；Close 清零。倒计时可显示超时。04 用受控时间输入测试，无 sleep。

StartPresentation { audience_available } 仅验证 Ready 与输入前置。07 主线程协调窗口创建与 Start；创建失败保持 Ready、零增长，无残留窗口；销毁前发布 Curtain。Idle/Ready snapshot 恒 Curtain且不携带 visual。跳页预览只属讲者：Next/Previous 在预览模式只移动预览，Enter Commit 才改 audience current_page，Esc 取消；04 修正当前 audience_snapshot 和 step 的隐式提交。

SwapScreens 是发给 app 角色协调器的意图；04 移除 screens_swapped 作为权威状态，页/crop 不变。04 提供纯 DocumentDescriptor { generation, page_count } 提交入口；06 加入 SlideMap 纯描述；10 将携带 bytes 的 StageShadowDocument 升级为已校验描述提交命令，同时更新消费者。core 不存/解析 PDF bytes。

## 3. 拓扑、角色与安全 API

02 先在 core 拟建 display.rs 定义 MonitorRole，serde 小写转换由 settings 负责；03 在同模块补 MonitorId(String)，02 binding 的 id:String 由03转换。04不定义显示器类型。

platform enumerate_monitors -> Result<Vec<MonitorInfo>, PlatformError>；Info 含 ID、名称、原生像素尺寸、物理桌面位置、DPI/scale、主屏、身份来源。零屏为正常空列表，OS错误单独返回。纯 resolve_roles 返回 ResolvedTopology { presenter:Option<MonitorId>, audience:Option<MonitorId>, ignored, source, warnings }；warnings 包含 NoMonitors、SavedAudienceMissing、DuplicateAudience、DuplicatePresenter、PresenterMissingFallbackPrimary、DuplicateBinding、AmbiguousMonitorId。

无保存记录：主屏（没有则ID排序首屏）作Presenter；多屏中最大面积且ID不同于已解析Presenter的屏作Audience，同面积按ID排序。保存存在则不启用Audience启发式：缺席/显式None不自动接管。Presenter缺席允许主屏安全回退并告警；零屏 presenter=None。先按MonitorId规范化每ID唯一role（重复binding首项权威+DuplicateBinding）；重复角色再按配置首次有效绑定，其余None并告警。多屏已保存Audience先确定，Presenter回退须排除该ID；单屏解析固定Presenter本屏/Audience=None，由显式windowed模式在窗口层允许同屏窗口。两个物理屏退化ID相同，禁止自动恢复Audience。设备路径/EDID/名称退化都不能保证跨接口绝对稳定，实测失败应提示重绑。

权威来源仅 display.monitors。07 SwapScreens 同一设置事务交换两个角色并迁移窗口；09 新选Audience同次将旧Audience设None。移除计划键 display.swap_roles；若已有外部文件包含此键，则保留为未知历史键但不执行。Presenting失去Audience：Curtain→Stop→销毁观众窗口→Ready；讲者保持有效Presenter屏窗口化。新屏不自动接管；零屏等待恢复，重连不自动Start。

03研究安全wrapper；07研究GPUI事件/DisplayId。native identity优先；位置+尺寸回退需统一物理/逻辑单位和DPI，映射歧义拒绝Start。workspace unsafe_code=forbid不能被局部allow覆盖。安全接口不足时须先形成最小适配模块、显式lint继承调整与ADR政策提案，评审通过才实施；不改core或全workspace禁令。

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

PDFium bindings/document/page/bitmap都不逃逸；bytes owner覆盖借用文档，不建unsafe自引用。pdfium-render本身不提供/自动下载PDFium；05锁binary构建号、来源、校验值并传入动态绑定路径，不虚构PDFIUM_DYNAMIC_LIB_PATH为库自动支持变量。

消息：LoadActive/PrepareCandidate（先返回metadata）、PreRenderCandidate（06建图定位后请求）、RenderRequest、CommitCandidate、AbortCandidate、RetireGeneration。请求/结果键包含generation/page/crop/target_px/priority/epoch；P0观众当前、P1讲者当前/下一页/备注、P2±1观众预取、P3预览、P4缩略图。candidate有独立命名空间/结果通道，不能被current-only检查提前丢弃，旧active持续服务。epoch可取消排队项/丢弃旧结果，不能承诺打断同步PDFium调用；优先级只决定下次出队。缓存按bytes预算。

## 6. 观众唯一数据入口与 SharedFrame

04创建纯core BgraFrame { width,height,stride,pixels_bgra:Arc<[u8]> }，checked长度、非零尺寸、紧密width×4 stride；05 BgraBitmap复用。11在media::SharedFrame复用该像素类型并加generation/page/slot/playback_epoch/sequence身份封装；core不依赖media。

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
