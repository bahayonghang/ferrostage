# 翻页笔（演示遥控器）按键兼容

## Goal

得力激光翻页笔在 FerroStage 中无法翻页。键值探测（WinForms WndProc 记录
WM_KEYDOWN）确认：该笔短按上/下键发送的是方向键 `Up`（vk 0x26, scan 0x48, ext）
和 `Down`（vk 0x28, scan 0x50, ext），不发送 PageUp/PageDown，也没有修饰键。
GPUI Windows 层能把它们解析为 `up` / `down`，但 FerroStage 默认只把
`next` 绑定到 PageDown/Space/Right、`previous` 绑定到 PageUp/Backspace/Left，
所以按键被忽略。

## Requirements

- 默认绑定：`next` 增加 `Down`，`previous` 增加 `Up`。
- 已有设置文件：`settings.toml` 会持久化完整的绑定列表，只改默认值不会生效。
  加载时如果 `keybindings.next` / `keybindings.previous` 与旧默认值逐项相同
  （`["PageDown","Space","Right"]` / `["PageUp","Backspace","Left"]`），
  就原地升级为新默认值；前提是 `Down` / `Up`（不区分大小写）没有被其他动作占用。
  自定义过的列表保持原样。
- 不提升 `schema_version`，不改用户 PDF。
- 菜单、大纲、页码输入、设置滑块等现有上下文对 ↑/↓ 的拦截保持不变
  （上下文导航优先于翻页）。
- 同步更新 `docs/PROJECT_PLAN.md` 中的默认绑定示例。

## Acceptance Criteria

- [x] `Keybindings::default()` 的 next/previous 包含 `Down` / `Up`。
- [x] 旧默认列表在解析时升级；自定义列表不变；`Down`/`Up` 被其他动作占用时不升级。
- [x] UI 测试：演示中按 `down` 前进、`up` 后退；布局菜单打开时 ↑/↓ 仍不翻页（已有测试）。
- [x] `just ci` 通过。

## Out of Scope

- Shift+F5、`.` 黑屏等其他翻页笔键值（该设备未发送）。
