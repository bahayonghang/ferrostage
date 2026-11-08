# 执行计划

前置：`task.py start` 之后才动代码。每步结束跑 `cargo test -p ferrostage-ui --features gpui` 的相关测试；每个检查点单独提交，便于回退。

## 步骤

1. **DESIGN.md**（检查点 A）
   - [ ] 按 design.md §1–2 改写根 `DESIGN.md`：名称、描述、token（浅/深）、层次模型、按钮/分段规则、原生材质说明、对比度表（用脚本实测，不手填）。
   - [ ] 保留 design-atlas 来源注释并注明"customized"；运行 `node <design-atlas>/scripts/slop_check.mjs DESIGN.md` 无 FAIL。
2. **token 层**（检查点 B）
   - [ ] 先改 `theme.rs` 测试为新约束（见 design.md §3.1），确认失败。
   - [ ] 重写 `Palette`、`LIGHT`、`DARK`、`button()`、`over()`；新增 `ButtonKind::Segment`、`indicator`。
   - [ ] 修编译错误时只做机械替换（按 §3.4 映射），不顺手改布局。
3. **材质与根**（检查点 C）
   - [ ] `material.rs`：新 `field()`、`panel()`、`floating()`；删除整窗 `paper()`。
   - [ ] `presenter.rs` 根：场全幅，去掉纸与外圈描边；首页改为居中面板卡片。
4. **按钮与分段**（检查点 D）
   - [ ] `presenter.rs::button`：去描边、去 `✓` 前缀、固定指示槽、`quiet_header` 透明化。
   - [ ] `segmented()` 替换黑白屏组与工具组外壳；`home_view.rs` 打开按钮同步。
   - [ ] 更新受影响的 GPUI 回归（按钮标签文本不再含 `✓`，选中改为可访问状态/调试选择器断言）。
5. **面板与组件**（检查点 E）
   - [ ] 工作台：顶栏、讲稿（panel + well）、预览组、计时胶囊、底栏。
   - [ ] 菜单与浮层：布局与导航、开始模式、工具菜单、页面浏览、跳页、裁剪确认、投影状态。
   - [ ] 设置页、显示器设置、识别窗口。
6. **残色扫除**
   - [ ] `rg -n "card|soft|output|sheet_edge|field_mint|field_lavender" crates/ferrostage-ui/src` 无旧 token；`rg -in "0x(83DFB7|187A56|6D4BD6|C6B4FF|312C44|213A32)"` 为空。
7. **验证**（检查点 F）
   - [ ] `just ci`。
   - [ ] `just dev` 原生截图（复用 `.trellis/tasks/archive/2026-11/11-07-presenter-airy-glass/research/native_appearance.py`，输出改到本任务 `research/after/`）：浅/深 × 毛玻璃开/关 × 首页/就绪/放映/设置/布局菜单；窄窗；默认字体与 `WenYuan Serif SC` 覆盖各一组。
   - [ ] 截图色相检查（绿/紫像素只来自 PDF 区域），记录脚本与结果。
   - [ ] 观众窗口内容区与改前逐像素比较差异为零。
   - [ ] 键盘：Tab 遍历、焦点环可见、Space/Enter 激活分段项、选中切换时按钮宽度不变。
   - [ ] 结果写入 `research/verification.md`，对照 `research/before/`。

## 回退点

- A 后可单独回退文档；B–E 每个检查点独立提交，截图不过时回退到上一个检查点。
- 若 Native 半透明层导致文字对比测试在桌面纯白下失败，优先提高 `layer` alpha，而不是加描边或改回整窗纸。

## 评审门

- 步骤 3 完成后先拍首页与工作台截图给用户确认方向（外圈是否消失、橙色观感），再继续步骤 4–5。
