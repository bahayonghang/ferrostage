# 验证：答辩备注页大字号深色版式

日期 2026-10-19。论文仓库 `dev` 工作区，未提交。FerroStage 布局已恢复为当前页在左、右上下一页、右下讲稿，左右 55:45（FerroStage 提交 e6ed7e4）。

| 检查 | 命令或方法 | 结果 |
| --- | --- | --- |
| AC1 回归测试 | `uv run python -B -m unittest tests.test_defense_notes_sync -v` | PASS，7 项；断言 18pt、备注页 453.5×382.7 bp、内容页 453.5×255.1 bp、备注页底色 ±2、span 在页内 |
| AC2 页数与标签 | `just slides-notes` 两遍后用 PyMuPDF 读标签 | PASS：190 页，95 个 `notes:N`，均与前一内容页配对；无 `Overfull \vbox` |
| AC3 正文底边 | PyMuPDF 统计 95 页备注最大 span 底边 | PASS：中位数 215.0 bp，P90 286.7 bp，最大 365.6 bp（`notes:6`）≤ 372.7 bp；所有 span 在页内 |
| AC3 默认可见高度 | 底边 > 352 bp 的页 | 1 页（`notes:6`），在默认讲稿区高度下需滚动约 14 bp |
| AC4 目视 | 第 6、45、49、54 帧 96 dpi 拼图（scratchpad，未写入 `preview-check/`） | PASS：无裁切；公式（$t$、$H_p$、$M_\phi$、Top-$k$）完整；深底浅字 |
| AC5 底色 | 备注页左下与右上角像素；内容页右下角像素 | PASS：备注页 (36,34,31)，目标 #252320 容差 ±2；内容页角点颜色集合与改动前 PDF 相同 {(255,255,255), (31,40,106)} |
| AC6 口播哈希 | `just slides-notes-export` 后 SHA256 | FAIL：开工前 `6f557531…0d7c`，导出后 `985244d7…a122`。见下文 |
| AC7 双屏实测 | 用户在 FerroStage 实机查看新 PDF | PASS：用户 2026-10-19 确认「检查没问题了」 |

## 页高变更

计划页高 13 cm（368.5 bp）。首次编译后 `notes:6` 底边 365.6 bp，超出 358 bp 验收线。按 design.md 预案改为 13.5 cm（382.7 bp），生成器、测试、`slides/AGENTS.md`、讲稿规范和本任务文档同步修改。

## 同步方式

`just slides-notes-sync` 会把 `deck/` 正文覆盖到 `deck-notes/chapters/`。在 scratchpad 副本上试跑后，`chapters/chapter2.tex` 会被改动：用户未提交的「三项映射构成运行指标决策的计算基础」会被 `deck/` 的「决策底座」替换。因此没有在真实目录运行同步。只把副本中生成器写出的 `defense.tex` 复制回 `slides/deck-notes/defense.tex`；该文件与原文件只在 notes-sync 段不同。`deck/` 与 `deck-notes/` 的 chapter2 正文差异留给用户处理。

## AC6 失败原因

导出只读取各章 `\note`，本任务没有改动章节文件。开工前工作区的 `speech.md` 相对 HEAD 只改 3 行，不等于当前章节备注的导出结果（当前章节有未提交的备注改动）。导出覆盖了这份未提交的 `speech.md`。导出前没有备份文件内容，只记录了哈希；在编辑器本地历史和临时目录中未找到哈希匹配的副本，原 3 行改动无法恢复。现文件是当前章节备注的完整导出（95 页，约 14680 字）。

用户在 AC7 检查后确认结果，接受 AC6 的现状，`speech.md` 以当前导出版本提交。

## 未改动

`slides/deck-notes/chapters/*.tex`、`notes.md`、`deck/`、口播措辞。
