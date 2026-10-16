# 显示器名称便于区分

## Goal

讲者在显示器设置里不用对着三行相同的 “Generic PnP Monitor” 猜哪块是哪块。列表标题、应用窗口/PDF 选择按钮、识别浮层都要能直接看出差别。身份仍然只认 `MonitorId`，编号仍然和“识别屏幕”是同一套。

## Confirmed facts

截图里的设置弹层是 `detailed = false`（`crates/ferrostage-ui/src/presenter.rs:527`）。卡片第二行已有分辨率和角色，但标题和红框里的选择按钮只有 `编号 · monitor.name`（`crates/ferrostage-ui/src/display_settings.rs:299`、`:397`、`:433`）。识别浮层大号下面也是这个名字（`crates/ferrostage-ui/src/identify.rs:26`）。

`monitor.name` 来自 `EnumDisplayDevices` 的 `DeviceString`（`crates/ferrostage-platform/src/windows.rs:73`、`:208`）。这台机器上三块活动屏的 `DeviceDesc` 都是 `monitor.inf` 的 “Generic PnP Monitor”，所以名字相同。这不是三块屏共用一个 `MonitorId`。

活动拓扑（2026-10-15 本机只读核对；几何来自已保存的枚举日志，名称来自 PnP / WMI / 注册表）：

| 编号 | 硬件 ID | 几何 | 桌面位置 | `DeviceString` | Windows FriendlyName | EDID 名称 |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | `BKX2700` | 2560×1440，主屏，旋转 0 | `(0, 0)` | Generic PnP Monitor | Generic Monitor (BKX270) | BKX270 |
| 2 | `BOE0D5A` | 2560×1600，旋转 0 | `(-2560, -169)`，主屏左侧 | Generic PnP Monitor | Integrated Monitor (NE160QDM-NM9) | NE160QDM-NM9 |
| 3 | `RTK2560` | 1600×2560，旋转 270 | `(2560, -1142)`，主屏右侧 | Generic PnP Monitor | Generic Monitor | 空 |

编号按 `MonitorId` 排序（`display_settings.rs:94-102`），不是从左到右。从左到右是 2、1、3。`\\.\DISPLAYn` 不是 Windows 设置里的显示器编号；现有文案已经声明 FerroStage 编号与识别标记一致，不代表 Windows 编号。

注册表 `FriendlyName` 是间接字符串，不是现成显示名，例如 `@System32\drivers\dxgkrnl.sys,#303;Generic Monitor (%1);(BKX270)`。PnP 解析后才是 “Generic Monitor (BKX270)”。`DeviceDesc` 的回退段就是 “Generic PnP Monitor”。

`winsafe 0.0.29` 已固定，feature 目前只有 `gdi`。它有 `RegOpenKeyEx` / `RegGetValue`（`advapi`，当前未启用），没有 `QueryDisplayConfig`、`DisplayConfigGetDeviceInfo` 或 `SHLoadIndirectString`。`display-info` 已在任务 03 因 fxhash 建议被拒绝。工作区 `unsafe_code = "forbid"`。

应用时 `friendly_name` 原样复制 `monitor.name`（`display_settings.rs:79`）。角色匹配只看 `id`。计划文档里的示例名是型号，例如 “LG ULTRAFINE”（`docs/PROJECT_PLAN.md:147`），不是驱动描述。

离线的重复实例（另一条 BOE、另一条 RTK、`IPS2800`）状态不是 OK，当前枚举不会把它们放进这三行。

## Requirements

- **R1** 同一套当前拓扑下，列表标题、应用窗口按钮、PDF 按钮、识别浮层使用同一条可区分标签。三块名字相同的屏，三条标签必须两两不同。
- **R2** 标签至少包含当前分辨率。主屏要标出主屏。高度大于宽度或旋转为 90/270 时标出竖屏。多屏时用当前 `position` 标出相对位置（左/右/上/下），让编号不是从左到右时仍然能对上物理屏幕。
- **R3** 标签不得把 `\\.\DISPLAYn` 或 FerroStage 编号说成 Windows 设置编号。
- **R4** `MonitorId`、角色解析、`numbered()` 的排序保持不变。编号仍然不写入身份。
- **R5** 读不到更好的显示名时，枚举仍然成功，并退回 R2 的几何标签。单块屏的名称查询失败不能让整份拓扑失败。
- **R6** 若采用系统显示名，写入 `friendly_name` 的是给人看的型号或内置标记，不是整句按钮文案，也不是未解析的 `@dll,#id;...` 字符串。没有更好名称时，可以继续保存现在的驱动描述。
- **R7** 中文和英文文案都要有对应说法。

## Acceptance criteria

- [ ] 用这台机器的三块活动屏数据做纯测试：三条标签互不相同，分别能看出 BKX270 主屏 2560×1440、内置屏 NE160QDM-NM9 在左侧、右侧竖屏 1600×2560。
- [ ] 两块屏名称、分辨率都相同但位置不同时，标签仍然不同。
- [ ] 名称查询失败或返回空时，对应屏幕仍出现在列表中，标签仍满足 R2。
- [ ] 应用后的 `display.monitors[].id` 与应用前的设备路径一致；测试不把 UI 编号当成 id。
- [ ] 识别浮层上的文字和列表使用同一条标签。
- [ ] `zh-CN` 与 `en-US` 都能表达主屏、竖屏和相对位置。

型号来自已选定的 FriendlyName 解析。没有型号的屏幕不显示 Generic PnP Monitor 作为标题。

## Out of scope

- 更换 `MonitorId` 算法，或把 EDID 序列号、端口、GPU 写进身份。
- 把编号改成从左到右。位置用文字标出。
- 自定义别名编辑框。
- 新增 crate、放宽 `unsafe_code`、恢复 `display-info`，或升级 `winsafe` 去调用 CCD。
- 热插拔、换线后的身份稳定性。那些仍由既有重绑规则负责。
- 把离线的重复 PnP 实例显示出来。

## Decision

名称使用系统 FriendlyName 里的型号，再加上分辨率、主屏、竖屏和相对位置。这台机器的可见结果是 BKX270、内置屏 NE160QDM-NM9，以及没有型号的竖屏。查不到型号时只保留几何标签，枚举不能因此失败。不做自定义别名。
