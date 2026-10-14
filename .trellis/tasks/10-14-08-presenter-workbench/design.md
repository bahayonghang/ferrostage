# Design — 真实工作台与受控观众画面

## 选择、依赖、输入输出
前置01/02/04/05/06/07与Gate B；W-1…10/R-40/43。排期P1任务内含M1阻塞P0：真实当前/next/notes/计时/时钟/进度/跳页/黑白屏/交换/备注隔离。P1网格/拖拽/高级画笔在M2前完整验收，不能提前归档全部任务。
本任务负责app初次真实导入：05页metadata/只读结构→06 builder构建LoadedDocument/content SlideMap→core Load安装map/crop→Ready；坏build保旧document。07 Shell importer由真实DocumentLoader替换，10后续复用。输入private LoadedDocument/core state/05结果/settings给讲者与投影组装器；audience view唯一输入snapshot，不读取document/cachehandle。输出content变换/纹理/命令，02既有debug/grid键只消费。

## 文件范围
现有app/src/main.rs/07导入协调模块、ui/src/lib.rs、07shell/presenter/audience/actions模块、core/src/{command,state,geometry,snapshot}.rs、ui/Cargo.toml、docs/feature-parity.md。
拟建ui/src/{workbench,projection,texture_cache,thumbnails}.rs和core所需annotation模块；L1GPU管理归ui，render只预算/CPU。不提前加入媒体后端或09设置页。

## 设计
讲者当前左、next/notes右、顶栏计时/时钟/文件/阶段、底栏导航/模式；metadata labels只讲者显示。文档菜单提供五layout、独立notes识别、overlay自动/physical_pages和诊断；调用06 builder重建同代view，08新增core ApplyDocumentView { slide_map,crop }纯命令，保留timer/stage、清preview并取最近有效content页，按新crop帧就绪后完整发布。notes独立P1请求/cache，拼合crop和按当前物理步多notes滚动展示来自06；比例/字号由settings实际应用。
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
