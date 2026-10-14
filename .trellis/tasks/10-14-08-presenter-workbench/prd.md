# 讲者工作台：当前页/下一页/备注/计时/缩略图/激光笔/画笔

父任务：`10-14-presenter-v0.3`（R-43）。复杂任务，设计和步骤见本目录design.md / implement.md。

## Goal

在 `07` 的外壳上实现真正可用的讲者工作台与观众画面：讲者看到当前页、下一页、备注、计时器与时钟；观众看到 4K 幻灯片与激光笔/批注图层；两端画面通过同一份渲染结果与归一化坐标保持一致。

## Background

- 布局草图与渲染层级：`docs/PROJECT_PLAN.md` §4（讲者：当前页 | 下一页 / 备注；底栏导航与工具；观众：幻灯片 → 媒体 → 批注 → 激光笔 → 幕布）。
- 渲染来源：`05-pdf-raster` 的 `RenderHandle`；逻辑页与备注裁剪：`06-beamer-document-model`；阶段与计时：`04-session-lifecycle`；布局比例、计时预警、指针样式：`02-settings-schema` 的 `presenter`/`timer`/`pointer` 节。
- GPUI `RenderImage` 为 BGRA 且需显式释放（不变量 6）。
- 已有 `NormalizedStroke { points, color_bgra, width_norm, is_highlighter }`（`ferrostage-core::geometry`）。

## Requirements

- **W-1** 讲者窗口布局：左侧当前页（可绘制区），右上下一页预览，右下备注面板；底栏：上一页/下一页、`逻辑页 n/N（步进 i/k，物理 p/P）`、网格、激光笔、画笔、荧光笔、橡皮、清空、黑/白屏、交换；顶栏：文件名、阶段、时钟、计时器。分割比例来自设置并可拖拽（gpui-base resizable）。
- **W-2** 备注面板：拼合页显示 `notes_crop` 的光栅化（按面板像素请求，P1）；`notes:` 页显示对应备注页；无备注时显示空态；字号缩放来自 `presenter.notes_font_scale`（对光栅备注为缩放倍率）。
- **W-3** 复用04 elapsed，count_up/target-elapsed/超时/黄红阈值。初次Ready elapsed=0（按mode正计00:00、倒计target），Stop后Ready沿同mode冻结读数，再次Start清零；Pause/Resume/Reset遵循父契约，不另存UI计时。
- **W-4** Live只读snapshot.visual受控当前BGRA；组装器缺新帧保持已提交页，letterbox变换统一。Curtain/black/white最高层覆盖媒体/笔画/指针，view无任意页请求/讲者cache。
- **W-5** 归一化坐标：讲者在当前页区域的鼠标位置经当前页内容矩形逆变换为 `(u,v) ∈ [0,1]²`，发送 `UpdateLaserPointer` / `AppendStrokePoint`；观众端按其内容矩形正变换绘制。鼠标离开区域发送 `UpdateLaserPointer(None)`。
- **W-6** 画笔模式：`AppendStrokePoint` 开始新笔画的时机（按下）与结束（抬起）需要核心支持 `BeginStroke { color, width, is_highlighter }` / `EndStroke`，本任务在 `ferrostage-core` 追加这两条命令与橡皮 `EraseStrokeAt(point)`；颜色/宽度来自 `pointer` 节。
- **W-7** 跳页预览：按 `G` 或点击页码弹出输入/网格（仅讲者窗口），悬停发送 `JumpPreview`（P3 预取），回车 `CommitJump`，Esc `CancelJump`；预览期间观众画面不变。
- **W-8** grid_columns为1..4，grid_visible_pages为1..16可见content页，行数ceil；P4/L3异步/虚拟滚动，不把列数当页数。
- **W-9** ManagedRenderImage由UI拥有并显式释放；±2与实际字节预算驱动淘汰，替换/关闭/代际/DPI统一回收。debug_overlay由02定义，08实现字节/帧时间，09控件。
- **W-10** 08投影组装器精确匹配generation/page/crop/target才补snapshot.visual；notes请求/cache仅讲者，真实隔离在08验收，11通过同入口扩展媒体。

- **W-11** 08负责首Load的05metadata/结构→06 builder→LoadedDocument/core SlideMap/crop接线；文档菜单五layout覆盖、独立notes识别、physical_pages与diagnostic，同代ApplyDocumentView保留stage/timer，错build保旧。

## Acceptance Criteria

- [ ] 首载含notes PDF安装content map，菜单覆盖/分组能实际重建裁剪和地图；用一份 ≥ 80 页的真实 Beamer PDF（含拼合备注）完成一场 ≥ 30 分钟的模拟汇报：翻页、黑屏、激光笔、画笔、跳页全部正常，无崩溃。
- [ ] L1 命中翻页延迟 P95 ≤ 16.6 ms（按键到观众 Present，用 tracing span 测量）；激光笔划动 P99 帧时间 ≤ 18 ms。
- [ ] 1080p 讲者屏与 4K 观众屏上同一笔画叠在同一幻灯片元素上（截图对比，偏差 ≤ 2 px@4K）。
- [ ] 跳页预览悬停 10 个不同页，观众画面始终不变；回车后同步。
- [ ] 连续翻页 200 次后进程 GPU 专用显存回落到基线 ±15 MiB（任务管理器/PIX 记录到 `research/vram-report.md`）。
- [ ] 计时器在渲染阻塞（人为 sleep 渲染线程 2 s）期间读秒不偏差。
- [ ] 观众视图模块 `rg "SlideNotes|notes_crop|LoadedDocument"` 无命中；`docs/feature-parity.md` 仅已验条目标Implemented，M1 P0和M2 P1分开；`just ci` 全绿。

## Dependencies

完整前置：01/02/04/05/06/07（具体目录见父Task Map）。父子关联不表示依赖。 真实双窗接线需Gate B；排期P1不降级M1的P0功能。

## Out of Scope

设置界面（`09`）；媒体播放（`11`）；热重载（`10`）。
