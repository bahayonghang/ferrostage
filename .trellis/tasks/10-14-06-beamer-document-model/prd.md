# Beamer 文档模型：页标签、备注识别与 overlay 分组

父任务：`10-14-presenter-v0.3`（R-41）。复杂任务，设计和步骤见本目录design.md / implement.md。

## Goal

从一份 Beamer PDF 构建讲者所需的全部映射：哪些物理页属于同一逻辑幻灯片（overlay 步进）、哪些页是独立备注页、拼合页的幻灯片半幅与备注半幅在哪里、每页的逻辑页码标签是什么、哪些页含媒体。观众只看到幻灯片区域，讲者得到备注。

## Background

- `ferrostage-document` 现状：`SlideNotes { page, markdown }`、`MediaSlot { page, label }`、`LoadedDocument { page_count, notes, media }`（`crates/ferrostage-document/src/lib.rs`）；没有逻辑页映射。
- 核心把 `NextLogicalSlide` 当作移动一个物理页（`state.rs:107-112`，`docs/beamer-notes.md` 已注明是临时行为）。
- `LogicalSlide`、`PhysicalPage` newtype 已在 `ferrostage-core::ids`；`NormalizedRect::{LEFT_HALF, RIGHT_HALF}` 已存在，缺 `TOP_HALF`/`BOTTOM_HALF`。
- PageLabel 由 `05-pdf-raster` 的 `PdfDocument` 提供；媒体字典由 `lopdf` 0.45 读取（`ADR-002`）。
- 判定规则：`docs/PROJECT_PLAN.md` §4.6（32:9 / 8:3 左右拼合；上下拼合；手动覆盖 `None/Left/Right/Top/Bottom`；`notes:<n>` 标签跳过）。
- `fixtures/beamer/README.md` 预留了 LaTeX 源码目录；本任务需要的测试 PDF 只作为编译产物夹具提交（含 `\pause`、`show notes on second screen=right`、`\movie[autostart,loop,start=2s,duration=8s]`），仓库不集成任何 LaTeX 工具链。

## Requirements

- **B-1** NotesLayout表示备注侧；尺寸一致文档W/H∈[3.1,3.6]或[2.4,2.7]候选Right，[0.85,0.95]或[0.63,0.70]候选Bottom。比例非可靠证明，来源/warning可见，手动覆盖优先；独立备注识别另有关闭选项。
- **B-2** `slide_crop(layout) -> NormalizedRect` 与 `notes_crop(layout) -> Option<NormalizedRect>`；在 `ferrostage-core::geometry` 补 `TOP_HALF`/`BOTTOM_HALF`。
- **B-3** notes:<suffix>跳过投影，只配紧邻此前content物理步的原始标签，不作Vec下标；多备注有序Vec留document，孤立/不匹配warning。None仅关拼合裁剪，不隐式关独立备注识别。
- **B-4** 保留Option<String> PageLabel；缺失/空串逐页独立；连续同非空content标签启发式分组，非连续重号不合并；非数字允许且01≠1；physical_pages覆盖，不承诺同标签必同frame。
- **B-5** core SlideMap只含可导航content steps/组，不带notes；document构建器与按物理步NotesMapping持备注。Step首尾钳制，Logical导航到组首，逆向仅比较组。
- **B-6** lopdf只读Movie/Screen/RichMedia支持子集，依据PDF规范/自建夹具记录字段路径、Rect底部原点到归一化顶部原点、source/start/duration/loop/autostart。缺失未知诊断，不保证任意导出100%还原。
- **B-7** `LoadedDocument` 扩展为 `{ page_count, labels, layout, slide_map, notes, media }`；`notes_for` 语义不变；`project` 继续只返回 `state.audience_snapshot()`。
- **B-8** 仍然没有任何代码路径把备注文本、`notes_page`、`SlideMap` 放入 `AudienceProjectionSnapshot`。

## Acceptance Criteria

- [ ] 夹具 A（16:9 常规 + `\pause`）：`SlideMap` 逻辑页数等于 frame 数，步进数等于物理页数。
- [ ] 夹具 B（`show notes on second screen=right`）：`layout == Right`，`slide_crop == LEFT_HALF`，`notes_crop == Some(RIGHT_HALF)`。
- [ ] 夹具 C（`notes:` 标签独立页）：翻页序列跳过全部备注页，document按content物理步的有序NotesMapping正确关联。
- [ ] 自建D对已记录字段路径的媒体支持子集还原参数，未知结构诊断。
- [ ] 8:9/2:3、宽图/竖版反例、手动覆盖、缺失/非数字/重号/多备注/孤立notes均有期望模型。
- [ ] 手动覆盖 `NotesLayout::None` 时拼合页按整页投影。
- [ ] 有后继位置prev(next(p))=p，有前驱next(prev(p))=p；首尾钳制单测，逻辑逆向只比组，导航全为content且在范围内。
- [ ] `format!("{snapshot:?}")` 不包含任何夹具备注文本；`just ci` 全绿。

## Dependencies

完整前置：01/04/05（具体目录见父Task Map）。父子关联不表示依赖。

## Out of Scope

播放媒体（`11`）；讲者端备注渲染（`08`）；Beamer `animate`。
