# 实施：答辩备注页大字号深色版式

工作目录：论文仓库 `D:\Documents\LYH\200-Learning\00博士毕业\毕业论文\thesis`。以下路径与命令均相对该目录。任务文档在 FerroStage 仓库 `.trellis/tasks/10-19-defense-notes-readable-page/`，`research/validation.md` 也写在该任务目录。

0. 基线
   - [ ] `git status --short` 记录现有未提交改动（预期含 `slides/deck-notes/chapters/chapter1.tex`、`chapters/front.tex`、`notes.md`、`speech.md`），实施中不覆盖、不回退。
   - [ ] 记录 `slides/deck-notes/speech.md` 的 SHA256（AC6 基线）。
   - [ ] 复制当前 `slides/deck-notes/defense.pdf` 到 scratchpad，作为 AC5、AC7 的改动前样本。
1. 契约先行
   - [ ] `slides/AGENTS.md:56`：16pt/20pt 改为 18pt/24pt，补写 16×15 cm 备注页与深色纸三色。
   - [ ] `.trellis/spec/backend/defense-slides/22-speaker-notes-guidelines.md:71` 同步。
2. 回归测试先改
   - [ ] `tests/test_defense_notes_sync.py:148`：字号断言 `16 * 72 / 72.27` 改为 18；新增备注页 453.5×425.2 bp 与内容页 453.5×255.1 bp 断言；保留标签与 span 在页内断言。先跑一次确认按预期失败。
3. 生成器
   - [ ] `src/thesis_analysis/defense_notes_sync.py:34-47` 的 `NOTES_BLOCK` 按 design.md 改写；背景先试零尺寸色块。
   - [ ] `uv run python -B -m unittest tests.test_defense_notes_sync -v` 通过。
4. 同步与编译
   - [ ] `uv run thesis-sync-defense-notes --dry-run` 确认只改 notes-sync 段（`just slides-notes-sync` 配方不接参数），再 `just slides-notes-sync` 正式同步。
   - [ ] `just slides-notes` 两遍。
5. 检查（写入任务目录 `research/validation.md`）
   - [ ] 页数 190、`notes:N` 95 个且配对（AC2）。
   - [ ] scratchpad 脚本统计 95 页正文底边，最大值 ≤ 415 bp（AC3）。
   - [ ] 第 7、49、54 帧及含公式帧渲染 PNG 到 `slides/deck-notes/preview-check/`，目视（AC4）。
   - [ ] 采样备注页背景像素与内容页背景像素，内容页与第 0 步样本比较（AC5）。
   - [ ] `just slides-notes-export` 后 `speech.md` 的 SHA256 与第 0 步相同（AC6）。
6. 集成
   - [ ] 等 FerroStage 任务完成后，用新 PDF 在用户双屏环境截图对比（AC7）。

回滚点：第 3 步前所有改动均为文字与测试；第 4 步后回滚见 design.md。提交、推送另行授权。
