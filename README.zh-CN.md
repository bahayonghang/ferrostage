# FerroStage

中文 | [English](README.md)

基于 Rust 与 GPUI 的现代 GPU 加速无锁 PDF & Beamer 演示舞台。

Windows 11 优先。应用完全离线运行：无遥测、无网络连接、无需账户。

可选桌面构建已接入讲者工作台、单线程 PDFium 光栅化、完整设置系统、外部 PDF 热重载、GIF/WebP 动图播放与隔离进程视频解码。默认工作区检查不依赖原生 SDK。实施与分项验证记录见[功能对照](docs/feature-parity.md)。

## 功能特性

- **文档管理**：支持命令行参数、系统对话框、拖放与最近文件列表打开 PDF。载入后处于就绪态（Ready），由讲者显式触发开始演示。
- **双窗口演示**：讲者工作台与观众投影窗口独立运行。支持按显示器角色（讲者、观众、关闭）分配投影、屏幕数字标识遮罩、物理拓扑展示、一键切换屏幕角色，以及播放窗口单击翻页。
- **Beamer 深度集成**：自动识别左右或上下同页备注；提供布局导航菜单支持手动覆盖（无备注、左、右、上、下）；支持独立 `notes:` 备注页配对、`\pause` overlay 步进合并（SlideMap）与 PageLabel 逻辑页码。
- **讲者工作台**：左侧展示当前页，右侧上方展示可缩放滚动的私有讲者备注，右侧下方展示下一页预览；集成正计时、预警倒计时与系统时钟；支持直接跳页与按需展开 1–16 页（1–4 行列）真实缩略图网格。
- **内存手绘批注与激光笔**：归一化矢量激光笔与荧光笔、画笔、橡皮擦工具，双窗口实时同步。所有笔画仅保存在当前会话内存中，不向 PDF 原文件写回任何内容。
- **外部 PDF 通用热重载**：使用 `notify` 监听文件变动，结合 300 ms 防抖与尾部 `%%EOF` 稳定性校验；后台异步光栅化并执行原子替换，保留当前演播进度、窗口状态与手绘笔画。
- **多格式媒体播放**：内置纯 Rust GIF / WebP 动图播放；提供隔离进程的 GStreamer 原生视频解码助手，通过共享 BGRA 离屏帧渲染，无子窗口句柄，缺失解码环境时静默回退。
- **设置系统与外观主题**：基于 TOML 的配置系统，包含图形化设置界面与 CLI `check-config` 验证器；支持深色、浅色与跟随系统的 Claude 视觉风格主题。

## GitHub topics

建议仓库 topics 保留 `beamer`，并移除 `latex`（若已设置）。`beamer` 指消费 Beamer 导出的 PDF，不是 LaTeX 工具链。

## 工作区架构

| Crate | 职责 |
| --- | --- |
| `ferrostage-app` | 主程序二进制入口与命令行协调器（`ferrostage`）。 |
| `ferrostage-ui` | 基于 GPUI 的演示窗口、讲者工作台与设置界面（基于 Zed `v1.23.3` 维护分支）。 |
| `ferrostage-core` | 演示状态机、命令流、几何计算与生命周期调度；不依赖 UI 或 PDF 库。 |
| `ferrostage-document` | 文档模型、幻灯片映射、Beamer 布局识别与讲者备注管理。 |
| `ferrostage-pdf` | 只读 PDF 结构解析（`lopdf`）与可选 PDFium 动态光栅化适配。 |
| `ferrostage-render` | 单光栅化工作线程、切片调度及 L1/L2/L3 CPU/GPU 有界缓存预算。 |
| `ferrostage-media` | 纯 Rust GIF/WebP 解码器、共享 BGRA 帧管理与媒体助手进程管理。 |
| `ferrostage-media-helper` | 独立进程原生 GStreamer 视频解码助手；无子窗口句柄。 |
| `ferrostage-platform` | 显示器枚举、屏幕标识与基于 `notify` 的外部文件重载监听。 |
| `ferrostage-settings` | TOML 设置项定义、存取、校验与配置迁移。 |

## 命令

```text
just ci       # 默认 features：fmt、clippy、test、cargo deny
just dev      # 把已校验的 PDFium 放到 target/debug，再以 gpui,pdfium 运行
just build    # 发布 gpui,pdfium；双击 target\release\ferrostage.exe
just release  # 生成 Windows x64 Setup.exe、便携 ZIP 和 SHA256 校验文件
just install  # 构建、打包并静默安装到当前用户，无需点击
just uninstall # 调用正式卸载程序，保留用户配置
```

工具链：Rust 1.99.0，edition 2024。

普通用户双击 `just release` 生成的 `FerroStage-<version>-windows-x64-Setup.exe` 即可安装。安装位置为当前用户的 `%LOCALAPPDATA%\Programs\FerroStage`，无需管理员权限；安装后提供桌面图标、开始菜单入口和 Windows“已安装的应用”卸载项。可直接通过图标或 Windows 搜索打开，无需终端或 Rust 工具链。卸载可使用系统设置中的卸载项，开发者也可以运行 `just uninstall`；用户配置和 PDF 会保留。

`just ci` 保持默认 features。桌面构建启用 `gpui,pdfium`，携带哈希校验过的 PDFium 及完整原始许可文件。Setup、便携 ZIP 和校验文件位于 Cargo 实际 target 目录下的 `dist`，遵循 `CARGO_TARGET_DIR`；安装版和便携版使用同一套已校验文件。工具链固定版本、安装行为与运行前提见[打包说明](docs/packaging.md)。应用进程不下载 PDFium，也不写用户 PDF。

开发构建可双击 `just build` 生成的 `target\release\ferrostage.exe`；自定义 target 时以实际输出路径为准。视频仍要单独准备媒体助手和运行库，构建命令是 `cargo build -p ferrostage-app --features gpui,pdfium,media --locked`。

`just dev` 保留控制台。发布版可执行文件使用 Windows GUI 子系统，资源管理器打开 `target\release\ferrostage.exe` 时不分配控制台。`just dev -- check-config <path>` 在终端打印只读的设置解析结果。交互式的 release `check-config` 没有控制台。把 release `check-config` 的 stdout 重定向后，解析结果仍会写出。

GPUI 使用基于 Zed 标签 `v1.23.3` 的已审维护源（ADR-008），不使用 crates.io 上的 `gpui` 0.2.x。唯一原生媒体后端为独立助手中的 GStreamer（ADR-009）。

## 许可

MIT OR Apache-2.0。这是对演示行为的净室实现，不移植 Pympress 源码。`cargo deny` 拒绝 GPL / AGPL。

## 相关文档

- [docs/PROJECT_PLAN.md](docs/PROJECT_PLAN.md) — 架构设计与里程碑计划
- [docs/architecture.md](docs/architecture.md) — 系统架构与模块划分
- [docs/feature-parity.md](docs/feature-parity.md) — 功能实施与分项验证清单
- [docs/packaging.md](docs/packaging.md) — PDFium 运行时获取与打包说明
- [AGENTS.md](AGENTS.md) — 贡献者与开发规范
