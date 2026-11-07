# 放映前备注裁切验收

在 Windows 原生 release 程序中使用正常 F5、讲者控件和 Esc。打开同页备注文稿后，先在讲者窗口显示当前规则下的两幅真实 PDF 预览；选择右侧备注时，左半页进入观众预览，右半页进入讲者私有预览。[真实 Beamer 的放映前确认](native/beamer-start-preview.png) · [确认后的观众窗口](native/beamer-projection.png)

| 场景 | 实测结果 |
|---|---|
| F5 → 改方向 → 换样本 → 取消 | 成功创建的观众窗口为 **0**，面板关闭，仅原讲者窗口存在。 |
| 同一路径 → Esc | 成功创建的观众窗口为 **0**。检查面板打开时重复按 F5，也没有绕过确认。 |
| 左、右、上、下四种备注位置 | 各用一份两页双色文稿。分别检查两块预览的原生可访问性边界内的像素：公开预览只有公开标记色，私有预览只有备注标记色。确认后的四个观众窗口均检测到公开标记色，备注标记色均为 **0 像素**。[并排预览](native/paired-preview.png) |
| 普通文稿与已配对独立备注 | 均直接开始，首次投影各创建 1 个观众窗口。 |
| 疑似备注但明确选择整页 | 经“按整页开始演示”后投出完整页面。整页选择会包含原页上的全部像素。 |
| 从布局菜单检查真实 Beamer | 浏览第二个样本、应用裁切后留在 Ready，观众窗口数为 0。随后开始时仍显示原讲述页；停止后从“续讲”直接恢复，不重复询问。[菜单检查](native/beamer-confirmation.png) |
| 演示中外部替换为不同结构 | 新版进入待确认状态，观众继续看到旧代公开画面。停止、选择新的备注方向并确认后，才出现新版公开画面，私有标记为 0 像素。[保留旧画面](native/reload-retained.png) · [确认后输出新版](native/reload-applied.png) |

计数取自每次成功创建 GPUI 窗口、尚未提交放映事务时的日志，并与所属进程的实际窗口清单交叉检查。只看最终窗口数会漏掉创建后又关闭的情况。旧版在相同测试文稿上无需确认即可投出含 **138,519 个备注标记像素**的整页。[旧版基线](native/baseline.json) · [新版逐项数据与哈希](native/native-check.json) · [原生事件日志](native/native-traces.zip) · [可复测脚本](native_crop_check.py)

### 回归

- 预览请求携带文稿代数、页、裁切和目标尺寸。候选的私有预览不产生提交令牌；失败后可重试或选择另一页。旧代、旧样本和已取消操作的返回不能开始放映。
- 被批准的热重载仍走上传和可回滚提交。上传失败保留旧文稿，在讲者面板显示错误；取消或重试会解除原来的开始意图。重试生成新的候选版本并重新要求确认。[热重载回归记录](reload-regressions.md)
- 备注预览布局根据 PDF 页几何和裁切比例计算自然高度。已修复某些文稿因光栅整数舍入而在两个高度间反复发起请求的问题。在静止的双色备注文稿上观察 600 ms，新增光栅请求为 0。
- `just check-shell` 通过，243 项 GPUI 测试通过。`just test-native` 通过，62 项测试通过。另用 `test-support,pdfium` 运行 UI 的 reload 用例，11 项通过。`just ci` 通过，工作区测试汇总 249 项。
- 检查源 PDF 的 SHA256，应用打开与放映未改写输入。热更新场景由测试脚本外部替换自己生成的 PDF。截图和窗口创建结果为 Windows 实机证据。比例识别覆盖现有等分候选范围；不规则排版仍可从布局菜单手动检查。

### Final review, 2026-11-07

- Reviewed the complete document/render/UI change set, including the new crop modules and tests, against the task artifacts and both spec layers. No additional source changes or unresolved findings were required. `git diff --check` and the untracked-text whitespace scan passed.
- `just check-shell` passed: desktop build, product Clippy with `-D warnings`, and 243 UI tests. Current log: `runs/crop-final-review/check-shell.log`.
- The `just ci` formatting, workspace Clippy and 249 workspace tests passed. Its final deny invocation initially failed because the sandbox could not lock the external advisory database. The unchanged `just deny` recipe then passed with cache access granted: advisories, bans, licenses and sources all passed. Current logs: `runs/crop-final-review/ci.log` and `runs/crop-final-review/deny.log`; the first log intentionally retains that environment failure.
- `just test-native` passed all 62 tests with the manifest-matching PDFium DLL (`79d4676b656cfb1abcea88f9ade3b4b0826c5200382db5f4ec72a636c598c118`). Current log: `runs/crop-final-review/test-native.log`.
- These runs used the existing `cache/perf-product-target`, the Rustup Cargo path, and a profile-free PowerShell recipe shell. No product settings, lints or policy checks were relaxed. Earlier `runs/crop-validation/*` failures are historical and are not the final gate evidence. Native window/pixel acceptance above was reviewed against `native/native-check.json`; it was not repeated in this final review.
