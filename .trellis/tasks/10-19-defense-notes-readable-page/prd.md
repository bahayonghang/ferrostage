# 答辩备注页大字号深色版式

## 目标

预答辩时讲者站在笔记本前不到 0.5 m，大屏幕在右侧。讲者要在 FerroStage 演讲者视图里顺畅读出讲稿。本任务改 `slides/deck-notes/defense.pdf` 的独立备注页版式：页面加高、字号加大、底色改为深色纸。

## 仓库位置

本任务目录存放在 FerroStage 仓库 `.trellis/tasks/`，改动对象是论文仓库 `D:\Documents\LYH\200-Learning\00博士毕业\毕业论文\thesis`（下称「论文仓库」）。文中相对路径、`just` 与 `uv` 命令都以论文仓库根目录为工作目录。

配套任务：同目录下 `10-19-presenter-notes-first-layout`（讲稿优先布局，讲稿栏占工作区宽 60%，按页面宽高比整页适配）。本任务只改论文仓库。

## 字号收益分工

按用户截图像素估算（见 design.md）：FerroStage 任务单独完成时，现有 16:9 备注页 16pt 字高从 23 px 到约 41 px（1.80 倍）；两边都完成后约 45 px（1.97 倍）。本任务对字高的贡献约 9%，主要收益是：18pt 下最长备注不溢出、备注页填满讲稿栏高度（16:9 页会在栏下方留约 410 px 空白）、深色纸降低眩光。

## 现状证据（2026-10-19 实测）

- 备注页与投屏页同为 453.5×255.1 bp（16:9），正文 16pt、行距 20pt、段距 0.4em（`src/thesis_analysis/defense_notes_sync.py` 的 `NOTES_BLOCK`）。
- 95 页备注：正文末行底边中位数 160.5 bp（页高 63%），P90 213.9 bp，最大 260.1 bp（第 49 帧，已超出页底 5 bp）。字数中位数 167，最大 284。
- FerroStage 按备注栏宽度缩放备注页。用户截图里备注栏约 650 px 宽，16pt 字约 23 px。
- 试验：在 note page 模板里写 `\special{pdf:pagesize ...}`，XeLaTeX 只改备注页尺寸，投屏页仍为 453.5×255.1 bp；正文从页顶排起，超出原 16:9 高度的部分不被裁切。
- 论文仓库工作区已有未提交改动：`slides/deck-notes/chapters/chapter1.tex`、`chapters/front.tex`、`notes.md`、`speech.md`（2026-10-19 `git status`）。这些改动不属于本任务，实施时不覆盖、不回退。
- 契约文字位置：`slides/AGENTS.md:56`、`.trellis/spec/backend/defense-slides/22-speaker-notes-guidelines.md:71`、`tests/test_defense_notes_sync.py:148`（`16 * 72 / 72.27`）。生成器 `NOTES_BLOCK` 在 `src/thesis_analysis/defense_notes_sync.py:34-47`。
- FerroStage 侧：`DocumentOverrides::default()` 为「不拆分」，混合页尺寸不产生 `MixedGeometry` 警告；独立备注页按自身页尺寸整页渲染。FerroStage 深色主题 `card` 也是 `#252320`，备注页在深色主题下与卡片无可见边界。

## 已确认决定（用户，2026-10-19）

- 布局采用讲稿优先（FerroStage 侧实现）。
- 备注页底色采用深色纸：底色 `#252320`，正文 `#E6DFD8`，加粗 `#FAF9F5`。三者取自 FerroStage 深色主题 token。
- 只用原版 PDF 备注，不做文字抽取、重排或缩放工具（沿用 1.1.0 任务的决定）。

## 需求

- R1 独立备注页尺寸改为 16 cm × 15 cm（453.5×425.2 bp，宽高比约 1.067），与讲稿栏在笔记本全屏下的宽高比相近。投屏页尺寸不变。
- R2 备注正文 18pt、行距 24pt、段距 0.4em，左右留白保持现有 0.5 cm 规则。
- R3 备注页整页铺深色底，正文与加粗按上述三色；公式颜色随正文。
- R4 改动写在生成器 `NOTES_BLOCK`，经 `just slides-notes-sync` 写回 `defense.tex`，不手改生成段。
- R5 页标签契约不变：内容页 `N`，备注页 `notes:N`，95 帧 190 页。
- R6 同步更新三处写死 16pt/20pt 的契约文字：`slides/AGENTS.md`、`.trellis/spec/backend/defense-slides/22-speaker-notes-guidelines.md`、回归测试 `tests/test_defense_notes_sync.py`。
- R7 不改任何 `\note` 措辞，不缩字、不截断来掩盖溢出。

## 验收标准

- AC1 `uv run python -B -m unittest tests.test_defense_notes_sync -v` 通过，测试断言备注字号 18pt、备注页 453.5×425.2 bp、内容页 453.5×255.1 bp、所有 span 位于页内。
- AC2 `just slides-notes` 连续两次后，`defense.pdf` 为 190 页，95 个 `notes:N` 标签与前一内容页配对。
- AC3 95 页备注正文末行底边均 ≤ 415 bp（保留约 10 bp 底边距）；报告最大值及其帧号。
- AC4 抽查第 7、49、54 帧（最密）与含公式的帧，渲染 PNG 目视：无裁切，公式完整，深底浅字。
- AC5 备注页背景像素为 `#252320`（容差 ±2），投屏页背景不变。
- AC6 开工前记录当前工作区 `slides/deck-notes/speech.md` 的 SHA256；改动后 `just slides-notes-export` 生成的 `speech.md` 哈希与之相同（本任务不改口播）。不用 `git diff` 判断，因为该文件已有未提交改动。
- AC7 FerroStage 配套任务完成后，在用户双屏环境下用新 PDF 截图对比，讲稿字高与改动前之比 ≥ 1.8（预计约 1.97）；站立可读性由用户判断。

## 不在范围

`deck/` 投屏稿、`slide_plan.yaml`、口播措辞、`notes.md`、FerroStage 代码、FerroStage 浅色主题下的备注页观感、PDF 压缩、提交与推送（另行授权）。
