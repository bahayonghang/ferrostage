# FerroStage 实施计划 v0.3

2026-10-15 当前实施进度：01–07已按修订范围归档；08–11全部功能源码及串行独立自动检查完成。最终180项默认CI、119项父UI测试加一次隔离重放、40项PDFium、17项媒体测试以及严格feature/lint/typecheck/deny/build通过。按用户快速验收要求，最终构建已完成工作台核心、设置主题保存重启、实际外部替换热重载、双窗GIF/MP4及缺失DLL降级冒烟，四次运行均正常退出。其余桌面动作保持NOT_RUN/NOT_VERIFIED，扩展矩阵DEFERRED_BY_USER；具体范围见父research/consolidated-functional-report.md。用户本轮已授权提交并按实施范围归档08–11；父任务继续承接未验收清单。下文原始验收目标和历史状态不代表当前阻塞或全量PASS。

> 2026-10-15 implementation-first revision: `.trellis/tasks/10-14-presenter-v0.3/acceptance-policy.md` is authoritative for current gates.03/07 meet the revised implementation scope;08 may proceed, followed by09/10/11. Physical, exhaustive native/size and performance/soak qualification is deferred by the user, not passed. Functional requirements and automatic/native-functional checks remain required. The detailed original targets below are retained for later qualification.

状态：01/02/04/05/06技术交付已独立验证；03自动与静态检查通过，物理验收待验；Gate A已通过。07已采用原获批五包维护源及三个精确许可证例外；2026-10-15另获明确授权接入第六包accesskit_windows0.34.0的已审四文件F14修复，验证以当前恢复索引为准。历史v6产品自动检查通过（49 GPUI/145默认测试、严格lint、native build），焦点恢复与保存重试反馈已修复并通过独立检查和原生复测；原生A/B/C、主题/语言、错误导入、选择器与保存角色重启已部分验证。物理拓扑、完整原生矩阵与Gate B仍未完成；08–11、Gate C/D、M1/M2未完成。上游HarfRust六项既存直接lint失败保留，无lint/安全通告豁免。当前恢复索引见 `.trellis/tasks/10-14-presenter-v0.3/execution-state.md`。本文件是 v0.2 方案（`ref/implement_plan.md`，未入库）的修订版，也是 Trellis 父任务 `10-14-presenter-v0.3` 的来源需求。

FerroStage 是一个 clean-room 的 PDF / Beamer 双屏演示器。目标行为：讲者端看到当前页、下一页、备注、计时器；观众端只看到幻灯片与激光笔/批注图层。Pympress 源码不是输入。许可证 MIT OR Apache-2.0。

## 0. v0.2 → v0.3 修订摘要

| # | v0.2 的问题 | 影响 | v0.3 处置 |
| --- | --- | --- | --- |
| 1 | 版本基线过期：Rust 1.88–1.92、pdfium-render 0.8.27、lopdf 0.34、gpui-kit 0.7.0、hayro-syntax 0.7.1、Zed rev 未落地 | 与仓库实际 pin 冲突，子任务会按错误版本写代码 | 以 §2 的仓库 pin 为准：Rust 1.99.0、pdfium-render 0.9.4、lopdf 0.45.0、gpui-kit 0.7.3 / Zed `v1.23.3`、hayro-syntax 0.8.0（未链接） |
| 2 | 大量 LaTeX 编辑场景：`latexmk -pvc`、`xelatex` 边改边讲、PoC-5 十次重编译、"LaTeX 重编译自动热重载"作为 P0 | 偏离"只做 PDF/Beamer 展示"的产品定位，膨胀 P0 范围 | 删除现行正向 LaTeX 工具链功能与 PoC-5，保留明确历史/排除说明。文件热重载降为 P2 通用能力（"磁盘上的 PDF 被替换"），验收不涉及任何 LaTeX 工具 |
| 3 | PDF 注释编辑（/Text、/FreeText 增删改、lopdf 写回 /Annots、`.ferrostage.tmp` 原子替换） | 这是 PDF 编辑功能，不是展示；且是 R06"损坏用户文件"风险的唯一来源 | 整体删除。新增强约束：FerroStage 永不向用户 PDF 写入任何字节；手绘批注仅存于会话内存。本行 `.ferrostage.tmp` 只追述已删除的旧路径；现行原子替换仅用于设置文件（唯一同目录临时文件，如 `settings.toml.tmp-<pid>-<seq>`）与 FerroStage 自身产物（ADR-004） |
| 4 | 没有应用生命周期：`PresentationState::new` 即进入演讲态，观众窗口隐含随文档打开 | 一打开 PDF 投影屏就被接管；无法先检查备注、调整设置再上场 | 新增 `SessionStage`：`Idle → Ready → Presenting`。只有导入 PDF 后点击"开始演示"才创建观众窗口并启动计时器（§4.2） |
| 5 | 显示器只用 `Option<u32>` 索引表示，"显式屏幕角色绑定"只有一句话 | 三屏环境下索引不稳定、无法指定第三块屏闲置，重启后角色漂移 | 新增显示器拓扑模型：稳定 `MonitorId`、每屏一个 `MonitorRole`、持久化到设置、1/2/3+ 屏与热插拔回退规则（§4.3） |
| 6 | 设置系统只有 6 个键、无文件位置、无保存、无 UI | "主题/快捷键自定义"列为 P2 但无落地路径 | 新增完整设置系统：分节 schema、配置目录、原子保存、`schema_version` 迁移、设置界面（§4.4） |
| 7 | 范围过宽：排练统计导出、中英双语、主题、Linux/macOS 安装包、Beamer animate、libVLC+GStreamer 双后端并列 | 单人项目无法在 M1 前收敛 | 统计导出、animate、第二媒体后端移出 v1.0；i18n/主题采用最小真实生效路径；本轮追加的Claude视觉规范由根`DESIGN.md`统一，07/08/09分别实施基础主题/工作台/设置；跨平台打包留在 Phase 3 |
| 8 | 渲染优先级表在 v0.2 §6.3 与 `docs/architecture.md` 不一致 | 实现时二选一会造成文档漂移 | 以本文 §4.6 为唯一来源，`architecture.md` 随 `01-docs-sync` 同步 |
| 9 | 不变量 2 要求"弹窗前把焦点切回讲者窗"，但未规定非演示态观众窗口的存在性 | 观众窗口在 Ready 态是否存在无定义 | 新增不变量 7：观众窗口仅在 `Presenting` 存在；任何其他阶段若窗口存在只能绘制纯黑幕布 |

## 1. 产品定位与边界

**一句话**：以答辩与授课可靠性为核心的 GPU 加速 PDF / Beamer 演示器，Windows 11 优先。

**纳入 v1.0**

- 打开 PDF（CLI、对话框、拖放、最近文件）；导入后处于就绪态，讲者显式点击开始。
- 讲者/观众双窗口分离；先检测屏幕数量/编号，再显式选择应用屏/PDF屏；黑/白屏；全屏。
- Beamer：同页左右/上下拼合备注、`notes:` 独立备注页、`\pause` overlay 步进、PageLabel 逻辑页码。
- 讲者工作台：当前页、下一页、备注、计时器（正计/倒计/预警）、时钟、跳页预览、1–16 页网格缩略图。
- 归一化矢量激光笔与手绘批注（会话内存，不写文件）。
- 完整设置系统与设置界面；显示器识别与角色分配。
- PDF 文件被替换时的通用热重载（P2）。
- GIF 与离屏视频播放（P2）。

**不纳入 v1.0**

- 任何 LaTeX 源码/编译工具链集成；PDF 注释编辑/写回（只读媒体结构发现保留）；排练统计导出；Beamer `animate` 帧动画；云协作、账号、遥测；PPTX。

**实施原则**：Rust 原生应用（PDFium/媒体原生动态库另审）；Clean-Room；`ferrostage-core` 不依赖 GPUI/PDFium/lopdf/notify；观众窗口只读 `AudienceProjectionSnapshot`。

## 2. 已锁定技术决策

- Rust edition 2024，工具链 1.99.0（`rust-toolchain.toml`）。
- UI：采用基于 Zed tag `v1.23.3` 的已审 GPUI 维护源与六个限定维护包，精确版本/补丁/许可见 ADR-008；不混用其他 GPUI 家族，不使用 crates.io `gpui` 0.2.x。
- PDF 光栅：`pdfium-render` 0.9.4，`PdfBitmapFormat::BGRA`，内存加载（读入 `Arc<Vec<u8>>` 后立即关闭文件句柄），单一专用渲染线程。
- PDF 结构：`lopdf` 0.45.0（已链接）读取 /Annots 中的媒体字典与页树；`hayro-syntax` 0.8.0 为备选第二读取器，未链接。PageLabel 由单渲染线程内的 PDFium 安全接口读取，以 owned metadata 返回。Rust 绑定不提供/自动下载 PDFium 动态库；05 单独锁 binary 来源、构建号与 checksum。
- 媒体：离屏 BGRA 帧，单解码器/单音频双窗口共享 `Arc`，无子 HWND。ADR-009 锁定独立助手中的 GStreamer；GUI 仅依赖纯 Rust 控制/协议与 GIF/WebP，不引入第二后端或原生媒体导入。
- 文件监听：`notify` 8.2.0，300 ms 防抖，等待 `%%EOF`，影子文档，严格更新 `DocumentGeneration`。FerroStage 不写用户 PDF。
- 设置：TOML，`toml` 1.1.8 + `serde`；配置目录由平台约定决定（见 §4.4）。
- 缓存预算：L1 GPU 256 MiB（current ±2），L2 CPU 512 MiB（current ±6），L3 缩略图 64 MiB。
- 首发 Windows 11（DX11）；Ubuntu 24.04、macOS 15 在 Phase 3。无遥测。
- `cargo deny` 拒绝 GPL/AGPL（无 MuPDF）。URW Base35 字体仅在许可证核查后才入库。

## 3. 功能矩阵

| 类别 | 功能 | 优先级 | 验收要点 |
| --- | --- | --- | --- |
| 文件 | 打开/拖放/最近文件/CLI 路径 | P0 | 打开后进入 Ready；不创建观众窗口；不启动计时 |
| 生命周期 | 开始演示 / 结束演示按钮与快捷键 | P0 | 点击开始：观众窗口在 Audience 角色屏全屏出现，计时器从 0 启动；结束：观众窗口关闭，回到 Ready，计时停止且 Ready 显示保留读数；每次重新 Start 清零 |
| 显示 | 显示器设置、数量/编号、应用屏/PDF屏选择、Identify | P0 | 先列检测数量和编号，再选择应用窗口/PDF播放目标，其余不使用；应用后生效，重启按 `MonitorId` 恢复；07提供最小面板，09整合完整设置 |
| 显示 | 热插拔降级 | P0 | 拔掉 Audience 屏：观众窗口立即销毁，讲者窗口保持窗口化，不把备注全屏到剩余屏 |
| 渲染 | 4K BGRA8 直出、纵横比保持、子区域裁剪 | P0 | 目标 P95≤25ms；机器/PDF/运行库与20次预热、200次采样见05协议；不经RGBA转换 |
| Beamer | 拼合备注自动检测 + 手动覆盖；`notes:` 映射；overlay 步进 | P0 | 观众端只看到幻灯片半幅；翻页跳过独立备注页；`NextLogicalSlide` 跳过同帧 overlay |
| 导航 | 物理页/逻辑页、跳页预览、回车确认 | P0 | 预览期间观众画面不变 |
| 工作台 | 当前页、下一页、备注、计时器、时钟、进度 | P0 | 计时基于 `Instant`，渲染阻塞不影响读秒 |
| 双屏安全 | 焦点防护、屏幕分配、黑/白屏与旧交换快捷键兼容 | P0 | 观众窗口获焦时任何快捷键都不在观众屏弹对话框 |
| 设置 | 分节 TOML、原子保存、迁移、设置界面 | P0/P1 | schema见§4.4；M1复用07最小角色面板，M2完整设置集成；设置只在讲者窗口 |
| 工作台 | 1–16 页网格缩略图、分割条布局 | P1 | 千页 PDF 滚动 60 FPS，缩略图低优先级异步生成 |
| 标注 | 激光笔、手绘、荧光笔、橡皮、清空 | P1 | `[0,1]²` 归一化坐标；双屏位置重合；会话内存，不落盘 |
| 热重载 | 磁盘 PDF 被替换后自动重载 | P2 | 半写状态不报错不黑屏；重载后保持逻辑页与计时 |
| 媒体 | GIF、音视频（/Movie /Screen /RichMedia） | P2 | 离屏帧，激光笔可叠加其上，切页即回收 |
| 工程 | 主题、语言、快捷键自定义、Linux/macOS 包 | P2 | 全离线；`cargo deny` 通过 |

## 4. 架构

### 4.1 Crate 边界

app为组合根，ui唯一链接GPUI；core在最内侧，不依赖任何IO crate。
document/pdf/render/media/platform/settings均位于core外侧；document拥有备注/媒体源，pdf负责只读结构/PDFium适配，render唯一worker/CPU缓存，ui拥有GPU纹理。
settings不依赖platform，复用core角色类型；MonitorRole由02创建，MonitorId由03补充；04定义生命周期与纯BGRA/投影类型，不重复定义显示器类型。
PresentationCommand经app协调器路由；core变更走apply_at，角色变更走唯一绑定协调器。观众view只接AudienceProjectionSnapshot/Entity，受控页BGRA/媒体像素由08/11组装入snapshot，无整个LoadedDocument或讲者cache。
完整契约及创建者/消费者见父任务design.md §1–6。

### 4.2 会话生命周期

```text
Idle ──LoadDocument(ok)──▶ Ready ──StartPresentation──▶ Presenting
 ▲                          │ ▲                            │
 └──CloseDocument───────────┘ └────StopPresentation────────┘
```

| 阶段 | 讲者窗口 | 观众窗口 | 计时器 |
| --- | --- | --- | --- |
| `Idle` | 空态：打开文件、最近文件、设置 | 不存在 | 停止，0 |
| `Ready` | 显示当前页/下一页/备注，可翻页预览、改设置、选显示器；"开始演示"按钮可用 | 不存在 | 停止；按mode初次elapsed=0或remaining=target，Stop后沿同mode冻结读数 |
| `Presenting` | 完整工作台；"结束演示"按钮 | 在 Audience 角色屏无边框全屏 | 运行（可暂停/重置） |

规则：

- `StartPresentation` 的前置条件：`Ready`，且显示器角色解析出至少一个 Audience 屏；否则命令返回错误并由讲者窗口提示（例如"未找到观众显示器，请在设置中分配"）。单屏模式下允许以窗口化观众窗口演示，但必须由设置项 `display.single_monitor_mode = "windowed_audience"` 显式允许。
- 翻页、激光笔、黑屏等命令在 `Ready` 和 `Presenting` 都接受（讲者可提前定位），但只有 `Presenting` 会产生观众像素。
- 热重载不改变阶段；`StopPresentation` 不丢弃批注与计时读数；`CloseDocument` 全部清空。
- `AudienceProjectionSnapshot` 新增 `stage: AudienceStage { Curtain, Live }`。非 `Presenting` 阶段的快照恒为 `Curtain`，观众窗口对 `Curtain` 只绘纯黑。

### 4.3 显示器角色模型（解决三屏问题）

core纯类型：02先创建MonitorRole { Presenter, Audience, None }；03补MonitorId(String)。platform创建MonitorInfo（身份/稳定性等级、名称、native_size、物理position、DPI/scale、primary）。
enumerate_monitors返回Result<Vec<MonitorInfo>,PlatformError>，零屏正常空列表；纯resolve_roles返回presenter:Option<MonitorId>/audience:Option<MonitorId>/ignored/source/warnings。

- 无保存记录：主屏（缺失则ID排序首屏）Presenter，多屏最大面积且不同于已解析Presenter的屏Audience，tie按ID；单屏audience=None。
- 保存记录权威优先；保存Audience缺席或明确None不启发式接管。Presenter缺席可在排除已保存Audience后回退主屏/其他屏并告警；零屏presenter=None。
- 每MonitorId先取唯一role（重复binding首项权威并warning），再对重复Audience/Presenter按配置首次有效binding，其余None并统一warnings；身份退化碰撞则禁止自动恢复Audience。
- 设备路径/EDID优先、名称/分辨率退化，标实际稳定等级；不承诺换接口必然不变，换线实测有变化须警告重绑。
- 唯一权威display.monitors：Swap修改两个role绑定并重新resolve，不另有swap_roles或core独立反转flag。
- 热插拔Audience消失：Curtain→Stop→关观众窗→Ready，讲者窗口化；新屏/重连不自动接管或Start；零屏等待恢复。
- 03先研究安全wrapper，workspace forbid不能被局部allow覆盖；完整API不足先审最小ADR/lint方案。07核GPUI DisplayId/native identity/物理与逻辑坐标/DPI，歧义映射拒绝Start。

07在Gate B前交付“显示器设置”最小面板：数量/编号列表、应用窗口/PDF播放两项选择、Apply/Cancel和同号Identify；编号仅显示，持久化MonitorId。09复用面板并补桌面拓扑图、稳定等级/解析来源/warnings及完整设置。07仍允许PDF占位，旧配置/Swap证据不替代新角色UI或真实画面验收。

### 4.4 设置系统

配置文件：`<config_dir>/ferrostage/settings.toml`。`config_dir` 为 Windows `%APPDATA%`、Linux `$XDG_CONFIG_HOME`（默认 `~/.config`）、macOS `~/Library/Application Support`。CLI `--config <path>` 覆盖。保存写唯一同目录临时文件 `<config-file-name>.tmp-<pid>-<seq>`（默认如 `settings.toml.tmp-<pid>-<seq>`），sync、关闭后原子替换配置文件；自定义配置名沿用其实际文件名。读取失败回退默认并在讲者窗口提示，绝不静默覆盖损坏文件（改名为 `settings.toml.broken-<timestamp>`）。

```toml
schema_version = 1

[general]
language = "zh-CN"            # zh-CN | en-US（assets/locales 已有）
theme = "dark"                # dark | light | system
reopen_last_document = false
recent_files_limit = 10
recent_files = []              # 02定义，07维护，09清空/限制
debug_overlay = false          # 02定义，08实现，09控件

[display]
single_monitor_mode = "presenter_only"   # presenter_only | windowed_audience
audience_fullscreen = true
presenter_start_fullscreen = false

[[display.monitors]]
id = "DISPLAY#GSM5B09#...|3840x2160"
friendly_name = "LG ULTRAFINE"
role = "audience"

[presenter]
next_slide_ratio = 0.35
notes_ratio = 0.40
notes_font_scale = 1.0
show_clock = true
show_thumbnail_strip = true
grid_columns = 4               # 1..4列
grid_visible_pages = 16         # 1..16可见content页，与列数独立

[timer]
mode = "count_down"            # count_up | count_down
target_minutes = 45
warn_yellow_remaining_min = 10
warn_red_remaining_min = 3

[pointer]
laser_color_bgra = [0, 0, 255, 255]
laser_radius_norm = 0.008
pen_color_bgra = [0, 0, 255, 255]
pen_width_norm = 0.004
highlighter_alpha = 0.5

[keybindings]
next = ["PageDown", "Space", "Right"]
previous = ["PageUp", "Backspace", "Left"]
start_presentation = ["F5"]
stop_presentation = ["Escape"]
blank_black = ["B"]
blank_white = ["W"]
swap_screens = ["S"]
toggle_laser = ["L"]
toggle_pen = ["P"]
jump = ["G"]

[cache]
l1_gpu_mib = 256
l2_cpu_mib = 512
l3_thumb_mib = 64

[hot_reload]
enabled = true
debounce_ms = 300
```

规则：02实现v1八节/键级校验、LoadOutcome与WritePolicy、真实旧扁平cache/debounce转换；不造空迁移链。未知顶层/已知节内部/monitors元素字段经递归叶键merge保留，不整节覆盖；保证语义值，不保证注释排版原样。
未来版本只读，不自动改名/写回/重置；缺省节由app在可写当前版本显式save补全，check-config纯读。坏语法备份成功后才可写默认，备份失败不覆盖原文件。Windows同目录tmp/sync/关闭/安全replace需已有目标/锁定/失败实测；失败保留原文件。
09运行时修改成功后尝试保存，save失败保留会话值、显示未保存/重试，重启回上次落盘值；runtime应用失败先回滚该修改。主题dark/light/system真实生效，语言重启后真实选资源。
11追加可选media.max_decode_height=1080（同v1缺省添加，宽≤1920）；09初版不提供空媒体控件，由11接线。hot_reload控件由10接线后才可启用。
详见父design §4及02/09/11设计。

### 4.5 双屏安全不变量（CI 属性测试约束）

1. **零讲稿持有**：观众窗口类型只接收 `AudienceProjectionSnapshot`；该类型没有备注字段。
2. **预览与弹窗隔离**：跳页预览、文件对话框、设置窗口、错误 Toast 只挂在讲者窗口；若焦点在观众窗口，先切回讲者窗口。
3. **热插拔降级**：Audience 屏断开即销毁观众窗口；讲者窗口保持窗口化；绝不自动全屏到剩余屏。
4. **代际一致性**：异步光栅化与媒体帧携带 `(DocumentGeneration, PhysicalPage, NormalizedRect)`；`generation < current` 即丢弃并回收纹理。
5. **媒体生命周期**：切页、黑屏、重载、关窗同步终止非当前页媒体。
6. **图集显存有界**：从 L1 淘汰或被新帧替换的 `RenderImage` 必须显式释放。
7. **阶段门控**：观众窗口仅在 `Presenting` 存在；任何阶段若收到 `AudienceStage::Curtain` 只绘纯黑。
8. **只读源文件**：FerroStage 对用户 PDF 只打开读取；没有任何代码路径以写模式打开该路径。

### 4.6 PDF 双擎、渲染调度与缓存

- PDFium：内存加载、`FPDF_RenderPageBitmapWithMatrix` 子区域光栅化（观众取 `LEFT_HALF` 按投影分辨率；备注取 `RIGHT_HALF` 按面板像素）、`FPDF_GetPageLabel`。
- lopdf：读取 /Annots 中 `/Movie` `/Screen` `/RichMedia` 的 `/F` `/EF` `/Start` `/Duration` `/Mode`，只读。
- Beamer候选：尺寸一致文档左右W/H∈[3.1,3.6]或[2.4,2.7]，上下∈[0.85,0.95]或[0.63,0.70]（8:9/2:3，高度翻倍而非左右比例倒数）；比例不是备注证明，Right/Bottom为默认猜测，手动None/Left/Right/Top/Bottom优先；混合尺寸/普通宽图/竖版反例必须测试。
- 原始PageLabel缺失/空串逐页独立、非数字保留；连续同非空content标签启发式overlay成组，重号不全局合并，可physical_pages覆盖。notes:<suffix>只配紧邻此前content原始标签，document按物理步存多备注，孤立warning；core SlideMap无notes页。
- 渲染优先级（唯一来源）：P0 当前观众页；P1 讲者当前页/下一页/备注区；P2 current ±1 观众预取；P3 跳页预览悬停页；P4 缩略图。每次翻页递增 epoch 取消过期低优任务。
- 单张3840×2160 BGRA8=31.64MiB；current±2五张像素约158.2MiB，另计GPU开销/讲者纹理，不把页数当唯一预算。

## 5. 性能基线

| 指标 | 目标 | 测量 |
| --- | --- | --- |
| L1 命中翻页延迟 | P95 ≤ 16.6 ms，极限 ≤ 50 ms | 按键到观众窗口 Present |
| 激光笔帧率 | 60 FPS，P99 帧时间 ≤ 18 ms | 4K 画面上快速划动 |
| 热启动到 Ready | ≤ 800 ms（100 页 Beamer） | 进程启动到讲者窗口显示首页 |
| 开始演示到观众首帧 | ≤ 300 ms | 点击开始到观众窗口 Present |
| 静止 CPU | < 0.5 % | 仅计时器跳动 |

## 6. 测试策略与 PoC 门禁

Phase 0 必须实机通过的 PoC（对应子任务见 §7）：

1. **PoC-1 三屏角色与热插拔**：三块不同 DPI 的屏，任意指定 Audience/Presenter/None；点击开始后观众窗口出现在指定屏；拔线后降级符合不变量 3。
2. **PoC-2 BGRA8 子区域光栅化**：含中文、公式、半透明 TikZ、32:9 拼合页的 PDF，4K 单幅 ≤ 25 ms，无 RGBA 转换。
3. **PoC-3 Beamer 语义**：`\pause` + `show notes on second screen=right` + `\movie[autostart,loop,start=2s,duration=8s]` 的 PDF，自建支持子集的logical/notes/媒体字段期望输出一致，未知结构诊断（媒体只读解析不播放，不保证任意导出100%还原）。
4. **PoC-4 离屏视频 + 激光笔叠加 + 显存零泄漏**（P2，`11-media-playback` 前置）。

删除 v0.2 PoC-5（xelatex 十次重编译）。热重载验收改为：仅在独立临时目录用脚本把夹具PDF分块覆盖到临时目标路径，FerroStage 0 次报错、0 次黑屏、保持逻辑页与计时。

测试目录：`tests/invariants`（proptest，不变量 1–8）、`tests/hot_reload`（分块覆盖写入）、`tests/golden`（光栅化像素基准）。

## 7. 里程碑与 Trellis 任务映射

| 阶段 | 子任务 | 准出 |
| --- | --- | --- |
| Phase 0 文档与契约 | `01-docs-sync`、`02-settings-schema`、`03-monitor-topology`、`04-session-lifecycle` | 纯 Rust 单测通过；现行正向能力无禁用功能；历史/排除命中逐条审查 |
| Phase 1 可用 Alpha | `05-pdf-raster`、`06-beamer-document-model`、`07-gpui-shell`、`08-presenter-workbench` | **M1 / Gate C**：01–08的P0 Alpha真实汇报（Gate B先占位、角色通过07最小分配UI）；08 P1增强仍待M2完整验收，不宣称所有产品P0完成 |
| Phase 2 Beta | `09-settings-ui`、`10-hot-reload`、`11-media-playback` | **M2 / Gate D**：09完整角色/设置集成、08全部P1、10热重载/11媒体、父AC-1…10全部通过（含Claude原生主题验收） |
| Phase 3 Stable | 跨平台与安装包（另行建任务） | **M3**：三平台 CI 全绿，120 分钟压测 |

依赖关系写在各子任务 `prd.md`，父子结构本身不表示顺序。

## 8. 风险矩阵

| 编号 | 风险 | 等级 | 缓解 |
| --- | --- | --- | --- |
| R01 | GPUI API 随 Zed 演进 | 高 | 锁定 `v1.23.3`；GPUI 调用收拢在 `ferrostage-ui`/`ferrostage-render` |
| R02 | SpriteAtlas 显存泄漏 | 极高 | 不变量 6；`ManagedRenderImage` RAII 守卫 |
| R03 | 观众端泄露备注或弹窗 | 极高 | 不变量 1、2、7；类型隔离 + 属性测试 |
| R04 | 三屏 `MonitorId` 在某些驱动下不稳定 | 中 | 多级退化键；设置界面显示来源；不匹配时不自动接管 |
| R05 | GPUI 无法按 display 指定全屏窗口或 DPI 信息不足 | 中 | `07-gpui-shell` 首先做 research 验证 `displays()` / `WindowOptions.display_id`；安全wrapper不足时先提出最小FFI/lint/ADR政策评审，不能allow绕过forbid |
| R06 | 4K 视频 CPU 回调带宽 | 中 | 解码输出上限 1080p，可配置 |
| R07 | GPL/专利污染 | 高 | Clean-Room；`cargo deny`；无 MuPDF |

## 9. 参考

- Zed GPUI：<https://github.com/zed-industries/zed/tree/v1.23.3/crates/gpui>
- gpui-kit：<https://github.com/longbridge/gpui-kit>
- pdfium-render：<https://docs.rs/pdfium-render/0.9.4>
- lopdf：<https://docs.rs/lopdf/0.45.0>
- notify：<https://docs.rs/notify/8.2.0>
- Pympress（仅作黑盒行为对照，GPL-2.0-or-later，不读源码）：<https://pympress.github.io/README.html>
- GPUI RenderImage 为 BGRA 且需显式回收：<https://github.com/zed-industries/zed/discussions/54236>

## 10. 规划一致性与验证规则（2026-10-14）

Task Map/前置与Gate A–D统一见父prd.md/implement.md；需求逐项见traceability.md。10完整依赖04/05/06/07/08，09实际设置效果依赖08；05依赖04帧类型，07依赖05只读导入元数据。技术Gate按已验证交付物触发，不以“已归档”当测试前置。
线程：IO读取/稳定检查、lopdf结构worker、唯一PDFium渲染worker加载/候选预渲染、主线程整体提交。失败旧代继续；commit后旧CPU/GPU/native资源分别在所有者线程退役。
默认just ci与PDFium native feature测试、GPUI编译、Windows/GPU/media实机门分开；禁止ignore/缺库成功跳过。性能表均为目标，机器/PDF SHA/分辨率/预热/采样/计时范围按05/08/11 implement协议记录，无数据标待验。
禁用词扫描不是全文零匹配：历史/排除允许，现行正向功能禁止；每个命中记录path/line/语义/处置，写API必须核目标。研究仅使用Pympress README/options用户文档和主源手册，文档描述与实测明确区分。
