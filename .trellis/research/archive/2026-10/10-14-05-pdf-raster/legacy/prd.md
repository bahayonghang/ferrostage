# PDFium BGRA8 光栅化、子区域裁剪与渲染调度

父任务：`10-14-presenter-v0.3`（R-40）。复杂任务，设计和步骤见本目录design.md / implement.md。

## Goal

首次把 PDFium 接入仓库：内存加载 PDF、以 BGRA8 直出任意归一化子区域、在单一专用线程上按优先级调度并支持代际取消，缓存受预算约束且纹理可显式回收。这是观众端 4K 画面与讲者端预览的共同底座。

## Background

- `ferrostage-pdf` 目前只链接 `lopdf` 0.45 做结构检查；`pdfium-render` 0.9.4 已在 `Cargo.toml:46` 注释中 pin 但未链接，绑定本身不提供/自动下载PDFium。
- `ferrostage-render` 只有预算常量与 `is_current` 代际检查（`crates/ferrostage-render/src/lib.rs`）。
- `AGENTS.md`：BGRA8、一个渲染线程；`docs/adr/ADR-002-pdf-backends.md` 记录双擎决策。
- `NormalizedRect::{FULL, LEFT_HALF, RIGHT_HALF}` 已在 `ferrostage-core::geometry`。
- 渲染优先级唯一来源：`docs/PROJECT_PLAN.md` §4.6（P0 当前观众页；P1 讲者当前/下一页/备注区；P2 ±1 观众预取；P3 跳页预览；P4 缩略图）。
- 显存预算：L1 256 MiB（current ±2）、L2 512 MiB（current ±6）、L3 64 MiB。
- GPUI `RenderImage` 为 BGRA 且需显式释放（Zed discussion #54236）；本任务不链接 GPUI，输出 CPU 侧 `BgraBitmap`，由 `07`/`08` 上传。

## Requirements

- **P-1** 只读probe与worker LoadActive/PrepareCandidate接owned bytes；页数/旋转后尺寸/Option<String> PageLabel返回owned metadata。safe load_pdf_from_byte_vec管理bytes，PDFium不在调用者/结构线程创建；07消费probe，06消费标签。
- **P-2** safe PdfRenderConfig matrix/clip输出BGRA，set_origin与custom transform不混用；验证rotation/CropBox/top-left/stride/alpha并必要时逐行打包。BgraBitmap复用04 core BgraFrame，raw BGRA出口，不走RGBA转换。
- **P-3** PDFium 实例与所有 `PdfDocument` 只存在于 `ferrostage-render` 的单一工作线程；对外 API 为消息队列 `RenderHandle::submit(RenderRequest) -> RequestId` 与结果通道。
- **P-4** 请求/结果完整generation/page/crop/target/priority/epoch；priority决定下次出队，epoch取消排队/丢弃旧结果，不中断已跑同步调用；active/candidate独立命名空间，Prepare metadata→06定位→PreRender候选请求→Ready→Commit/Abort/Retire供10用。
- **P-5** L2 CPU 缓存：键 `(generation, page, crop, target_px)`，LRU，字节上限可配置（默认 512 MiB）；L3 缩略图固定高 180 px，上限 64 MiB。L1 由 UI 层持有纹理，但本 crate 提供 `L1Budget` 记账器与淘汰建议，使 `07`/`08` 能在淘汰时调用 GPUI 释放接口。
- **P-6** 字体回退：PDFium 初始化时可注册附加字体目录（为 URW Base35 预留），默认不打包字体（许可证待核）。
- **P-7** 锁PDFium binary构建号、主源、checksum/许可与显式runtime路径，不自动下载/入库。PDFIUM_DYNAMIC_LIB_PATH仅按真实build-link语义；规划pdfium feature/native必跑job、缺库失败。
- **P-8** 自建红色BGRA哨兵、字体/公式/透明/旋转/CropBox/左右上下PDF。左右crop Cw×Ch对FULL 2Cw×Ch裁切；上下对FULL Cw×2Ch，固定运行库/字体与容差，不比不同密度hash。
- **P-9** 新增错误类型 `PdfError`（`thiserror`），遵循 `.trellis/spec/backend/error-handling.md`；不记录 PDF 字节到日志。

## Acceptance Criteria

- [x] 4K目标P95≤25ms；机器/PDF SHA/字体/binary、预热20采样200，测worker render不含排队/上传。`research/poc-2-timing.json`：所有4Kcase PASS，含保留前帧；适用边界见verification.md。
- [x] 同源密度crop/FULL切片非边缘每通道差≤1，排除边缘2px并记录；median耗时比0.465884/0.457172≤0.8。
- [x] 输出缓冲 stride = width×4，首像素字节序为 B,G,R,A（用纯红色页验证为 `[0,0,255,255]`）。
- [x] 可控fake worker出队前排10个P4/新epoch P0，下一出队P0、旧P4取消；已运行同步P4允许完成但旧结果不发布。
- [x] 提交 `generation=1` 任务后把当前代设为 2，该任务结果被丢弃。
- [x] L2 缓存写入超过上限时淘汰最旧项；跨代共用budget并按Vec capacity计量。
- [x] 独立审查后默认纯Rust109tests通过；native42tests通过，独立CI job已配置必跑且缺库负门exit101。Hosted workflow本轮NOT_RUN，GPU另验；无ignore/缺库early-success。
- [x] 默认及all-features `cargo deny` 通过；预编译PDFium及第三方许可另行审阅，不属cargo deny覆盖。

Implementation checkpoint: `research/verification.md`, `research/verification-manifest.json`. Independent check passed after fixing cross-worker readiness-token reuse; current evidence: `research/independent-check.md`, `research/independent-check-manifest.json`. Task05 is technically ready for parent progression; no commit, push, archive or journal operation was performed.

## Dependencies

完整前置：01/04（具体目录见父Task Map）。父子关联不表示依赖。

## Out of Scope

GPUI 纹理上传与 `drop_image`（`07`/`08`）；Beamer 语义解析（`06`）；缩略图 UI（`08`）。
