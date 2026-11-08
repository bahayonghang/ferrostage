# 验收记录

构建：`cargo build -p ferrostage-app --features gpui,pdfium --locked`（debug），PDFium 7881，夹具 `fixtures/pdf/perf-100.pdf`，窗口 1440×940 逻辑像素，隔离设置文件。复测命令：

```
python .trellis/tasks/11-07-presenter-warm-glass-restyle/research/native_appearance.py --exe target/debug/ferrostage.exe --run <空目录> --out <输出目录> [--font "WenYuan Serif SC"]
python .trellis/tasks/11-07-presenter-warm-glass-restyle/research/hue_check.py [--exclude x0,y0,x1,y1] <png>...
```

脚本复制自归档任务 `11-07-presenter-airy-glass`，改动有三处：F5 之后先截裁切确认页再点 `crop-confirm`，这一步是 `11-07-notes-crop-confirm` 加入的；等待观众窗口进入 Settled 时先去掉日志里的 ANSI 颜色码；新增 `--font` 参数。

## 截图对照

| 场景 | 改前 | 改后（默认字体） | 改后（衬线覆盖） |
| --- | --- | --- | --- |
| 首页 | [深色](before/dark-home.png) | [浅色·玻璃](after/light-on-home.png)、[浅色·实色](after/light-off-home.png)、[深色·玻璃](after/dark-on-home.png)、[深色·实色](after/dark-off-home.png)、[窄窗](after/narrow-home.png)、[键盘焦点](after/system-keyboard-home.png) | [浅色](after-serif/light-on-home.png)、[深色](after-serif/dark-on-home.png) |
| 就绪工作台 | [深色](before/dark-workbench.png) | [浅色](after/light-workbench.png)、[跟随系统·深色](after/system-workbench.png) | [浅色](after-serif/light-workbench.png)、[深色](after-serif/system-workbench.png) |
| 设置 | — | [浅色](after/light-on-settings.png)、[深色](after/dark-on-settings.png) | [浅色](after-serif/light-on-settings.png)、[深色](after-serif/dark-on-settings.png) |
| 裁切确认 | — | [浅色](after/light-crop-confirm.png)、[深色](after/system-crop-confirm.png) | — |
| 观众窗口 | — | [浅色](after/light-audience.png)、[深色](after/system-audience.png) | — |

## 验收条件逐项

- **无绿/紫色相。** 统计口径：色相 90°–300°，HSV 饱和度大于 25%，亮度大于 12%，并且 3×3 邻域全部命中。邻域条件用来排除 ClearType 文字的 1px 彩边。改前深色首页命中 91.9%。改后首页（玻璃开/关、浅/深、窄窗、键盘焦点）和两张设置截图都是 0。四张工作台截图（默认字体与衬线、浅色与深色）排除两块 PDF 预览后也是 0。
- **外圈。** 底场铺满整个窗口，不再有缩进带和整窗描边。毛玻璃开启时，在背后换纯色背景板，窗口内均值变化为 (8.8, 4.1, 1.8)，说明系统模糊确实生效。关闭时为 (0.002, 0.001, 0)，窄窗时为 (0.003, 0.002, 0)，都回退到了实色。底场不透明度：浅色 0.88，深色 0.82，经用户确认。数据见 `after/native-check.json`。
- **按钮。** 所有按钮都不画描边。主操作为橙底 `#D97757`、深色字 `#1F1712`。黑屏/白屏和四个绘图工具改为分段控件，选中项用固定宽度槽位里的圆点标出。GPUI 回归 `segmented_tools_mark_selection_in_a_fixed_slot_without_width_change` 用真实点击验证：选中时按钮宽度和组宽度都不变。
- **对比度。** `theme.rs` 的单元测试在纯黑、纯白两种桌面上合成"底场 → 面板"，再检查所有文字角色都不低于 4.5:1，焦点和选中标记都不低于 3:1。抬升层、凹区、选中洗底、控件填充和按钮的悬停/按下态也同样检查。原来的"按钮描边 ≥3:1"改成了"无描边，焦点与选中标记 ≥3:1"。对比度表由 `contrast_table.py` 生成。
- **状态。** 悬停、按下、禁用、选中、焦点这五种状态由 `theme.rs` 测试覆盖；键盘焦点见截图 `system-keyboard-home.png`。
- **即时切换。** 浅色和跟随系统·深色两种主题下放映同一页，观众窗口内区（去掉外框 20 物理像素）的逐像素差异为空（`audience_inner_difference_bbox: null`）。观众相关文件 `audience.rs`、`audience/`、`slide_surface.rs` 未修改。
- **顺带修复。** 文稿警告浮层原来被裁切提示遮住。面板改成半透明后，被遮住的文字透出来形成重影。现在警告浮层锚定在提示条上方 8px 处。回归测试 `document_warnings_float_above_the_crop_notice_without_overlap` 在 1440、1100、520 三种宽度下验证两者不相交。
- **CI。** `just ci` 通过（fmt、clippy `-D warnings`、ferrostage-ui 274 项测试、cargo deny）。design-atlas 的 `slop_check`（`crates/ferrostage-ui/src --design DESIGN.md --scope app`）报告 0 FAIL / 0 REFLEX；该工具只扫描网页文件，Rust 源码不在其扫描范围内，实际只校验了 DESIGN.md。

## 未覆盖与已知差异

- Windows 原生标题栏仍是系统黑色，不在本任务范围内。
- PDF 预览图没有圆角，因为 GPUI 只支持矩形裁剪。
- 标准布局下，"当前页"和"下一页"各占一个面板，没有合并成一组。原因是"下一页"和讲稿共用一个分栏，合并需要改布局。
- 放映中工作台、布局菜单、页面浏览没有单独截图，只由 GPUI 回归测试覆盖。

## 检查轮修正（trellis-check）

修了 12 处，修完后已重拍上面全部截图：
- 深色主题下分段控件的选中项原来不凸起，新增 `segment` token：浅色 `#FFFFFF`，深色 `#524D48`。
- 选中的分段项悬停时没有反馈，已补上。
- 绘图菜单的选中行原来看不出来，已修正。
- 菜单行选中时内边距会跳动，已固定。
- 字体选择器补上了固定位置的选中标记和 `aria_toggled`。
- 开始模式菜单行补上了按下态。
- 滑块填充改用 `focus`。原来浅色主题下只有 2.7:1，低于 3:1。
- 所有内凹区的圆角统一为 12。

重拍后复查结果和之前一致：无绿/紫色相；观众窗口内区逐像素无差异。

另外有两项保留不改：
- `success` token 留在 DESIGN.md 里，但代码中已经不再使用。
- 深色菜单和对话框里的次要按钮，填充与浮层几乎同色。按钮靠文字标签识别，悬停态清晰可见；如果加深填充，会和菜单行的悬停高亮冲突。
