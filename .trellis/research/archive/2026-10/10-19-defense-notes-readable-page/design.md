# 设计：答辩备注页大字号深色版式

## 字号模型

FerroStage 把备注页缩放到讲稿栏内，屏幕字高 = 字号 / 页宽 × 页面显示宽度。页宽保持 16 cm 不变，所以字高只随字号和讲稿栏宽度变化。

单位为用户截图像素（2000×1253，DPI 缩放未知，倍数与缩放无关）。工作区约 1990×1110。恢复后的布局：当前页占 55%，分隔线 6 px、两侧间距各 6 px，右栏约 878 px 宽；右栏内下一页与讲稿按 35:65 分高，扣除 6 px 分隔线与间距后讲稿面板约 712 px 高；面板内边距 12 px、标签行约 24 px，讲稿区内尺寸约 854×664，宽高比约 1.29。备注页按讲稿区宽度显示，超出高度的部分纵向滚动。

| 情形 | 页面显示宽度 | 字高 | 倍数 |
|---|---|---|---|
| 改动前：右栏 35%，16:9 页，16pt | 约 650 px | 23 px（实测） | 1.00 |
| 右栏 45%，16:9 页，16pt | 约 854 px | 约 30 px | 1.31 |
| 右栏 45%，16×13.5 cm 页，18pt | 约 854 px | 约 34 px | 1.47 |

页高取 13.5 cm 的理由：讲稿区默认可见高度对应页内约 664/854×453.5 ≈ 352 bp。页高只决定能否容纳最长备注，正文从页顶排起，底边不超过 352 bp 的备注无需滚动。计划先取 13 cm（368.5 bp），实测第 6 帧底边 365.6 bp，超出 358 bp 验收线，按预案改为 13.5 cm（382.7 bp），保留约 10 bp 底边距。15 cm 页在此布局下只会增加需要滚动的空白。

## 容量核算

正文宽约 425 bp，18pt 每行约 23 个汉字；可用高度约 362 bp，24pt 行距约 15 行，容量约 340 字。现有最长备注 284 字，含段距后约 13 至 14 行，余量不足 1 行。最坏情况按现有最大底边 260 bp 推算：260 × (24/20) × (16/18 的每行字数比倒数 1.125) ≈ 351 bp，低于 372.7 bp 的验收线，余量小，以 AC3 实测为准。

## 生成器改动（`NOTES_BLOCK`）

```tex
% notes-sync:begin
\setbeameroption{show notes}
\definecolor{ysuNoteBg}{HTML}{252320}
\definecolor{ysuNoteInk}{HTML}{E6DFD8}
\definecolor{ysuNoteStrong}{HTML}{FAF9F5}
\setbeamertemplate{note page}{%
  \special{pdf:pagesize width \the\paperwidth\space height 13.5cm}%
  % 深色底：从页左上角铺 16 cm × 13.5 cm 色块，不占排版空间
  ...（tikz overlay 或零尺寸 \rlap 色块，实施时二选一并验证像素）
  \hspace*{-0.5cm}%
  \begin{minipage}[t]{\dimexpr\textwidth+1cm\relax}
    \raggedright\color{ysuNoteInk}%
    \linespread{1}\fontsize{18}{24}\selectfont
    \setlength{\parindent}{0pt}\setlength{\parskip}{0.4em}%
    \renewcommand{\textbf}[1]{{\bfseries\color{ysuNoteStrong}##1}}%
    \insertnote\par
  \end{minipage}\hspace*{-0.5cm}%
}
\addtobeamertemplate{note page}{}{\thispdfpagelabel{notes:\insertframenumber}}
% notes-sync:end
```

要点：

- `pdf:pagesize` 的 `pt` 由 xdvipdfmx 按 TeX pt 解释，`\the\paperwidth` 得到 453.54 bp，与内容页同宽。
- 颜色定义放在 notes-sync 段内，只影响备注页；`\color` 写在模板内部，不改全局颜色。
- `\textbf` 的重定义限定在 minipage 组内。若 `\renewcommand` 与 ctex 加粗机制冲突，改为在模板内 `\addfontfeature` 或保留默认加粗、只着色，实施时以编译与像素结果为准。
- 标签追加顺序不变：先 `\setbeamertemplate`，后 `\addtobeamertemplate` 追加 `\thispdfpagelabel`。

## 背景实现的两种候选

1. TikZ overlay：`\tikz[remember picture,overlay]\fill[ysuNoteBg](current page.north west) rectangle ++(\paperwidth,-13.5cm);`。需两遍编译取得 `current page` 坐标（`just slides-notes` 本已要求两遍）。
2. 零尺寸色块：`\rlap{\smash{\raisebox{...}{\textcolor{ysuNoteBg}{\rule{\paperwidth}{13.5cm}}}}}`，偏移量按 note page 原点实测。无需 remember picture。

先试 2（无辅助文件依赖），像素不对齐再用 1。

## FerroStage 兼容

- 内容页与备注页尺寸不同。FerroStage `DocumentOverrides::default()` 为 `layout: Some(NotesLayout::None)`（`crates/ferrostage-document/src/layout.rs:23-30`），走手动分支，不触发 `MixedGeometry` 警告（`builder.rs:128-131`）；只有用户手动选「自动」才显示该警告。
- 独立备注页经 `notes_pages_for(page)` 以 `Surface::Notes(index + 1)`、整页裁切渲染（`crates/ferrostage-ui/src/workbench.rs:303`），页尺寸变化不需要 FerroStage 改动。
- FerroStage 深色主题 `card` 为 `#252320`，与备注页底色相同；浅色主题下备注页是浅色卡片上的深色块，不在本任务范围。
- 下一页预览与页码只走内容页，备注页尺寸不影响。
- pympress 读取同一 PDF 时备注页按自身尺寸显示，不作验收对象。

## 回滚

恢复 `NOTES_BLOCK` 与测试的旧值，`just slides-notes-sync`，再 `just slides-notes` 两遍。

## 风险

- `\textbf` 重定义可能影响备注中的公式加粗（`\bm` 不受影响，`\textbf` 在数学模式外才生效）。实施时检查含 `\textbf` 的 17 处。
- 底边超过 352 bp 的备注在默认讲稿区高度下需要滚动；用户可在 FerroStage 拖动右栏分隔线加高讲稿区。
