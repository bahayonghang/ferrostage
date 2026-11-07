# 讲者窗口的毛玻璃能力

GPUI 依赖锁定在 Zed `96a1b1b5`。FerroStage 在同一讲者窗口上切换 `WindowBackgroundAppearance`。没有重建窗口，观众窗口不调用这个接口。主页、设置和放映工作台共用一个色场和一张纸面；PDF 纹理继续走原有绘制路径。

| 平台 | 锁定后端的行为 | 应用条件 |
|---|---|---|
| Windows | `gpui_windows/src/window.rs::set_background_appearance` 将 `Blurred` 送给原生窗口的 accent state 4。DirectComposition 渲染器允许透明底层。GPUI 忽略 Win32 调用的返回值。 | 使用 Windows `UISettings::AdvancedEffectsEnabled`；系统减少透明度、查询失败、显式设置 `GPUI_DISABLE_DIRECT_COMPOSITION=1/true` 时回退。项目支持的 Windows 10 最低 build 为 18362，高于此 GPUI 路径的 17763 下限。 |
| macOS | `gpui_macos/src/window.rs::set_background_appearance` 在内容视图下添加原生模糊视图，关闭时移除。 | 查询 `NSWorkspace::accessibilityDisplayShouldReduceTransparency`。当前 Windows 工作树未做 macOS 原生截图验收。 |
| Linux | X11 在该接口中仅更新 alpha。Wayland 需要 compositor 公布可选的 blur manager；GPUI 将它保存在私有 globals，应用无法确认。 | 采用同色系实色回退。 |

讲者窗口低于 960 **逻辑像素**或用户关闭材质时直接使用实色。仅在宽窗且效果开启时刷新系统策略，两次查询至少间隔两秒；渲染不会每帧查询系统。设置帮助文字区分「已请求系统窗口模糊」和各类回退，偏好值不会被实际材质覆写。

原生模式使用 0.75 色场叠加系统背景，纸面透明度为 0.78；回退使用完全不透明色场，纸面透明度为 0.96。两层合成保持文字可读，圆角、按钮和语义配色在回退时继续生效。目录中的 20 px CSS 面板模糊在这里映射为系统管理的窗体模糊；GPUI 没有提供逐面板滤镜半径或应用成功回执。
