# PDF 打开与 Beamer 放映的初步诊断

分析版本为 `e6cf226626f2df279d0102a41233659446bb1caf`。以下实测来自本机、校验过 SHA256 的项目固定 PDFium 和仓库内两份只读文稿。报告内记录 CPU、系统、夹具与 DLL 哈希，以及每次采样。测量脚本在本任务内运行，不修改应用或 PDF。

| 阶段（毫秒） | overlays.pdf（3 页），中位 / P95 | perf-100.pdf（100 页），中位 / P95 |
|---|---:|---:|
| 从内存打开 PDFium 文档句柄 | 0.039 / 0.045 | 0.369 / 0.493 |
| 遍历页，取尺寸、旋转和标签 | 0.323 / 0.414 | 89.745 / 108.752 |
| 连续翻步的 CPU 绘制（适配 1920×1080） | 3.10 / 3.49 | 3.26 / 16.92 |

元数据探针每次重新打开内存中的同一文稿，5 次预热、30 次采样，复现 native.rs 所用的一组底层操作；Python 调用成本也计入。两份文稿内容不同，不能据此推断每页固定耗时。翻步探针每份 PDF 打开一次，顺序轮询物理页，每次新建 BGRA 位图并绘制，20 次预热、200 次采样；无应用缓存，无 Rust BGRA 导出，未测 GPU 上传、原生 present 或转场。它与已有 Rust `poc2` 的计时边界不同。两组均为热文件测量。

## 打开到首张讲者预览

1. `ferrostage-ui/src/shell.rs:866` 和 `:884` 已将读文件、lopdf 结构解析放到后台。二者与后续 PDFium 准备在现有入口先后执行。用同一文稿分别计时，避免将解析阶段全部归因于读盘。
2. `ferrostage-pdf/src/native.rs:72` 打开每个物理页，读取尺寸、旋转和标签。100 页夹具的实测让这段成为首要测量点。结构解析提供目录、媒体和转场，原生页几何还有校验职责；不能仅凭有两次遍历就删除一遍。
3. `ferrostage-ui/src/document_load.rs` 依次等待 Metadata、Raster、Commit。shell.rs:270 的计时器间隔是 25 ms，tick 每次只 poll 一次；处理元数据后至少还需两次 tick 才能观察光栅和提交，即使 worker 极快。此处是调度结构，不是实测 UI 耗时。记录 worker 回复到 shell 消费的差，再决定是否让完成事件唤醒。
4. `workbench.rs::request_for` 预渲染首张为 960×540；工作台按实际 pane 分辨率请求。查同一页是否因目标尺寸不同导致重复光栅，区分缓存命中、命中但仍需上传、重复渲染。保留旧文稿直到新候选准备成功。

## 输入到观众目标画面

1. `shell.rs::tick` 每轮观察窗口、轮询结果、调度、发布并通知；`presenter.rs::from_shell` 重取呈现状态。`presenter/workbench_view.rs::capture` 复制整份 steps，`presenter/settings_view.rs::SettingsData::from_shell` 克隆设置。分别测等待、前台执行与静止时无效通知数；计时、断屏、安全遮罩和媒体更新有独立的及时性要求。
2. `workbench.rs::requests/schedule` 同时处理观众、当前、下一步、备注、邻页预取及可选缩略图。请求集合变化会推进 epoch 并清空 pending；worker 能取消排队任务，正在执行的 PDFium 调用不可抢占。区分同像素键的 L2 命中和多次真正光栅；先证实抢占观众关键路径，再调整预取或去重。
3. `texture_cache.rs::paint` 对新纹理调用 `pixels_bgra().to_vec()`；已提交纹理可复用。完整 1920×1080 BGRA 约 7.91 MiB，3840×2160 约 31.64 MiB。按首次上传、同帧重绘分别计数。先保持目标分辨率和 L1/L2 预算，不以画质下降换取速度。
4. `audience/playback.rs::painted` 可区分 FirstVisible 与 Settled。有作者定时转场时，分别报告首个可见目标与转场完成；正常转场时长不应记为系统卡顿。目标未就绪继续显示完整旧画面，过期代数、备注、媒体及批注不得串页。

## 可复测材料

- [元数据探针](probe_metadata.py) 与 [30 次采样](pdfium-metadata-baseline.json)。
- [翻步探针](probe_paging.py) 与 [200 次采样](pdfium-paging-baseline.json)。
- 在仓库根运行 `python -B .trellis/tasks/11-07-presenter-performance/research/probe_metadata.py` 或同目录的 `probe_paging.py`。二者只读输入并输出 JSON；不同机器记录为单独基线。
- 现有 `crates/ferrostage-render/examples/poc2.rs` 是无缓存 Rust worker 光栅对照，见 `docs/packaging.md`；产品打开到首帧、键入到绘制、热缓存、OS Present 和优化后对照仍待测。
