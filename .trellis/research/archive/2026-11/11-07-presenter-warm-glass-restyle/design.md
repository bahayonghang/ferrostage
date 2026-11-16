# 技术设计：暖橙毛玻璃

## 1. 设计语言（写入根 DESIGN.md 的内容摘要）

名称：**暖橙毛玻璃**（派生自 design-atlas `style:airy-glass`，保留其"明亮场 + 半透明层 + 大圆角 + 填充即边界"的材质，换掉色彩系统）。

- 一个强调色相：暖橙。只用于主操作、焦点、选中标记、计时运行点、链接。
- 中性色全部带暖色倾向（象牙、砂、暖炭），不再有青绿或紫色调的面。
- 层次靠**透明度和明度差**，不靠描边和第二色相。
- 原生 GPUI 没有逐元素 `backdrop-filter`。玻璃 = 窗口级模糊 + 半透明层，整窗不再画"纸"。

### 1.1 起始 token（实现时以单元测试为准微调，不得降低对比度）

| 角色 | 浅色 | 深色 | 说明 |
| --- | --- | --- | --- |
| `field_top` / `field_bottom` | `#F7F2EB` / `#EFE6DB` | `#1D1B19` / `#26221E` | 160° 渐变；Native 时整体 alpha ≈0.72 |
| `layer`（面板） | `#FFFFFF` @0.66 | `#2C2824` @0.74 | 顶栏、讲稿、预览组、底栏；不描边 |
| `layer_solid` | `#FBF8F4` | `#2A2622` | 非 Native 时面板实色 |
| `raised`（浮层/分段选中项） | `#FFFFFF` | `#38332E` | 菜单、对话框、选中分段 |
| `well`（面板内凹区） | `#F3EEE7` | `#221F1C` | 讲稿文字区、输入框底 |
| `control` / `_hover` / `_pressed` | ink @0.06/0.10/0.14 | 白 @0.08/0.12/0.16 | 次要按钮、分段轨道；实现为预合成 u32 |
| `hairline` | ink @0.08 | 白 @0.08 | 分隔线（装饰，不作边界） |
| `border` | `#8C847A` | `#8F877C` | 仅输入框等需要可辨边界的控件，≥3:1 |
| `ink` / `body` / `muted` | `#1F1B17` / `#3D3833` / `#6B645C` | `#F5F1EB` / `#E2DBD2` / `#B8B0A6` | |
| `accent`（主按钮填充） | `#D97757` | `#D97757` | |
| `on_accent` | `#1F1712` | `#1F1712` | 5.65:1；白字只有 3.12:1，禁止 |
| `accent_hover` / `accent_pressed` | `#E0866A` / `#C9694A` | 同左 | on_accent 在两者上均需 ≥4.5 |
| `accent_text` / `focus` | `#B5532F` | `#E8A487` | 橙色文字、焦点环、选中标记 |
| `accent_wash` | `#F7E6DC` | `#3A2A22` | 选中行、选中菜单项底 |
| `success` / `warning` / `error` | `#2F6B45` / `#8A5A12` / `#A63A32` | `#8DCEA0` / `#E8B774` / `#FFA5A0` | 只用于状态文字与标记 |
| `error_wash` | `#F8E3DF` | `#3B2523` | 停止/危险按钮悬停与填充 |

半径：面板 16、浮层 16、内凹区 12、按钮与分段 pill。间距沿用 8/12/16/24/32/48。阴影：只有浮层（菜单、对话框、弹出面板）有一层 `0 12px 32px rgba(31,27,23,0.12)`（深色 0.40）；面板无阴影。

## 2. 层次模型

```
L0 窗口场 field          全幅铺满，无 inset、无描边；Native 时半透明透出模糊桌面
 └ L1 面板 layer         顶栏 / 讲稿 / 预览组 / 底栏 / 首页卡片；圆角 16，无描边
     └ well / control    面板内的凹区和控件填充，同色相
 L2 浮层 raised          菜单、对话框、跳页、裁剪确认；唯一带阴影 + hairline
```

规则：文字只落在 L1/L2/well 上，不落在 L0 场上（Native 时场底是任意桌面）。首页因此改为"场 + 居中 760px 面板卡片"，而不是整窗纸。

## 3. 代码边界与改动

### 3.1 `theme.rs`
- 重写 `Palette` 字段为上表角色；删除 `field_mint`、`field_lavender`、`sheet`、`sheet_edge`、`soft`、`card`、`output`。半透明 token 以"alpha 常量 + 基色"表示，并提供 `over(front, back, alpha)` 合成函数供渲染和测试共用。
- `ButtonKind` 保持枚举名以减少调用点改动，新增 `ButtonKind::Segment`（分段项）。`ButtonStyle` 去掉 `border` 或允许 `None`；新增 `indicator: Option<u32>`（选中标记色）。
- 规则：
  - `Primary`：`accent` / `on_accent`，无描边。
  - `Navigation`、`Utility`、`Tool`、`Black`、`White`：`control` 填充 + `body` 字，无描边。
  - `Segment`：透明填充；选中 → `raised` 填充 + `ink` 字 + `accent_text` 指示点。
  - `Destructive`：透明/`control` 填充 + `error` 字；会话"停止"用 `error_wash` 填充。
  - 禁用：`control` 填充 + `muted` 字。
- 测试改写：`semantic_button_states_*` 改为验证文字/填充 ≥4.5（含 hover/pressed）、`focus` 对每种填充与面板 ≥3、选中指示点对其底 ≥3、`border` 对 `well`/`layer_solid` ≥3；`paper_over_each_field_*` 改为在桌面纯黑/纯白两极下合成 field→layer 再测所有文字角色。

### 3.2 `presenter/material.rs` 与 `presenter.rs` 根
- `field()` 返回新渐变，Native alpha 由 `FIELD_ALPHA` 定义。
- 删除 `paper()` 的整窗绝对定位纸；改为 `panel(p) -> Div`（`layer`/`layer_solid` 依 Surface 选择，圆角 16，无描边）和 `floating(p) -> Div`（`raised` + hairline + 一层阴影）。
- `presenter.rs:1302-1327`：去掉 `.child(self.material.paper(p, inset))`；`inset` 改为面板之间的外边距（首页 24、工作台 8–12、窄窗 4），场本身全幅。

### 3.3 按钮与分段控件（`presenter.rs::button`、`home_view.rs` 打开按钮）
- 去掉 `border_1()`/`border_color`；焦点仍由 `activation_target` 的 2px 环提供。
- 去掉 `✓ ` 前缀。可切换按钮（工具、黑白屏、偏好项、裁剪布局）在标签前保留固定宽度指示槽，选中时显示 `accent_text` 实心点，未选中为空（宽度不变）。黑屏/白屏的色块即指示槽，选中时色块加 2px `focus` 环。
- 新增 `segmented(children) -> Div`：`control` 轨道、内边距 4、pill；替换 `workbench_view.rs:1071-1098` 的两个 `bg(p.soft)` 外壳。
- `quiet_header`（计时器内按钮）改为透明填充 + `control_hover`。

### 3.4 面板与组件替换映射

| 旧用法 | 新用法 |
| --- | --- |
| `p.card` 计时胶囊 | `control` 填充 pill；运行点 `accent_text`；数字 `ink`，警告色不变 |
| `p.card` 讲稿面板 + 内部黑底 | `panel` + `well` |
| `p.card` 菜单/对话/设置卡/裁剪 | `floating`（浮层）或 `well`（设置内分组） |
| `p.soft` 悬停/勾选行 | 悬停 `control_hover`；勾选 `accent_wash` + 指示点 |
| `p.soft` 跳页输入、分段外壳 | 输入 `well` + `border`；分段 `segmented` |
| `p.border` 分隔线、表头底线 | `hairline` |
| `p.border` 输入框/预览框边界 | `border`（保留 ≥3:1） |
| `p.output` | `accent_text` 或 `ink`（按语义） |
| `p.focus` 作填充 | `accent` |

涉及文件（按 grep 结果）：`presenter.rs`、`presenter/{home_view,workbench_view,layout_navigation,start_mode,utility_menu,page_browser,settings_view,number_input,crop,projection_status}.rs`、`display_settings.rs`、`identify.rs`。`audience.rs`、`audience/`、`slide_surface.rs` 不改。

### 3.5 预览区
当前页/下一页包进同一 `panel`，标题用 `muted` 12px；PDF 图像四角 12 圆角（面板内边距 4 时外 16 = 内 12 + 4）。图像本身不着色。

## 4. 兼容性
- 设置 schema、默认值、Surface 回退判定（`Surface::resolve`）不变。
- `ButtonKind` 名称保留，调用点只在需要分段时改。
- 观众窗口不读 `theme.rs`（文件头注释已约束），验收用逐像素比较确认。

## 5. 风险与取舍
- **品牌橙 vs 白字**：选择深字压 `#D97757`，保留品牌观感；若用户偏好白字，改 `accent=#B5532F`，其余不变（只改两个 token）。
- **无元素级模糊**：玻璃感取决于 Native 合成；关闭毛玻璃时靠暖色渐变场 + 层明度差维持"轻"。
- **原生标题栏**：Windows 深色模式下为黑色，与暖炭场仍有差异，本任务不处理（见 PRD 范围）。
- **回滚**：token、材质、按钮、组件替换分提交；任一步截图不过可单独回退该提交。
